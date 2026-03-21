# End-to-end validation report

This document records a full validation pass of **sxmc** (skills, MCP stdio bridge, OpenAPI `api` command, scan, bake) and contrasts **crates.io v0.1.1** behavior with fixes shipped in **v0.1.2+** through **v0.1.3** on `main`.

## Environment (representative)

- **OS:** Linux (x86_64)
- **Rust:** stable (e.g. 1.93.x) via rustup
- **Skill under test:** `system-info` in `~/.claude/skills/system-info/` (SKILL.md + `scripts/sysinfo.sh` + `references/usage-guide.md`)
- **Fixture skills:** `tests/fixtures/` in this repository

## Issues found in crates.io v0.1.1

### 1. Skill script execution via `sxmc stdio "sxmc serve"` fails

**Symptom:** Running a skill script exposed as an MCP tool fails with:

```text
Script execution failed: Failed to run .claude/skills/<skill>/scripts/<script>: No such file or directory (os error 2)
```

**Cause:** Skills discovered under the project-relative path `.claude/skills` were stored with **relative** `base_dir` and script paths. Spawning `sxmc serve` as a subprocess did not reliably resolve those paths for `Command::new(script_path)`.

**Workaround on v0.1.1:** Use an absolute `--paths` when serving, e.g. `sxmc serve --paths /home/you/.claude/skills`.

**Fix:** Canonicalize skill directories in discovery/parsing so script paths are always absolute.

### 2. `sxmc api` operation calls fail with “builder error” for some OpenAPI 3 specs

**Symptom:**

```text
[sxmc] Detected OpenAPI API
Error: HTTP request failed: builder error
```

**Cause:** Some specs declare `servers[0].url` as a **relative** path (e.g. `/api/v3`). The client concatenated that with operation paths, producing a non-absolute URL; **reqwest** then failed while building the request.

**Example spec:** `https://petstore3.swagger.io/api/v3/openapi.json` — `servers[0].url` is `/api/v3`.

**Fix:** Resolve relative `servers[0].url` values against the **spec source URL** so the base URL is absolute.

## Automated test suite (from repo root)

```bash
cargo test
```

**Result:** All tests pass, including:

- **61** library unit tests
- **22** `tests/cli_integration.rs` integration tests
- **1** doc test

Added coverage: `test_extract_base_url_relative_server` in `src/client/openapi.rs`.

## Manual end-to-end checks (release binary)

Build:

```bash
cargo build --release
# Binary: target/release/sxmc
```

| Check | Command / expectation | `master` / **crates.io v0.1.2** |
|-------|----------------------|----------------------|
| Skills | `sxmc skills list` / `info` / `run` | OK |
| Scan | `sxmc scan`, `sxmc scan --json` | OK |
| MCP list tools | `sxmc stdio "sxmc serve" --list` | OK |
| MCP run script | `sxmc stdio "sxmc serve" <tool>` **without** `--paths` | OK (prints script output) |
| OpenAPI list | `sxmc api https://petstore3.swagger.io/api/v3/openapi.json --list` | OK |
| OpenAPI call | `sxmc api … getInventory` | OK (HTTP + JSON; server may return 500 body) |
| OpenAPI call | `sxmc api … findPetsByStatus status=available` | OK (JSON from API) |
| Bake | `sxmc bake list` | OK |
| Fixtures | `sxmc skills list --paths tests/fixtures` and `sxmc stdio "sxmc serve --paths $(pwd)/tests/fixtures" --list` | OK |

**Contrast with crates.io v0.1.1 (unpatched):** MCP script invocation without `--paths` and live `api` calls (same Petstore URL) reproduced the two issues above.

## Edge-expansion coverage on `master`

The integration suite now covers the main areas that previously needed extra
manual confidence checks:

- local OpenAPI autodetect list + operation call via `sxmc api`
- local GraphQL introspection list + operation call via `sxmc graphql`
- `sxmc skills create` from a live local OpenAPI document
- `sxmc serve --watch` reloading a changed skill body over remote HTTP MCP

This pass also surfaced and fixed a CLI parsing bug: `api`, `spec`, and
`graphql` were treating trailing arguments too greedily, so flags like
`--format toon` and `--pretty` placed after operation arguments could be
silently swallowed instead of applied.

### Post-release check: **crates.io v0.1.2** (2026-03-20)

Re-validated using a clean install from crates.io:

```bash
cargo install sxmc --force   # → sxmc 0.1.2
```

- **`cargo test`** on `origin/master` at validation time: **61 + 22 + 1** tests, all passed.
- **16 manual E2E checks** against `~/.cargo/bin/sxmc` (version line reported **0.1.2**): all passed, including MCP script tool **without** `--paths`, **`api getInventory`** (no client “builder error”), **`api findPetsByStatus`**, and fixture `--paths` flows.

### Post-release: **crates.io v0.1.3** (2026-03-20)

Re-checked after **`cargo install sxmc --force` → 0.1.3**:

- **`cargo test`:** **65** library + **28** integration + **1** doc = **94** tests, all passed.
- **Automated CLI timings:** see [BENCHMARK_RUN_v0.1.3.md](BENCHMARK_RUN_v0.1.3.md) (median ms, `scripts/benchmark_cli.sh`).
- **Release notes:** [CHANGELOG.md](../CHANGELOG.md) `[0.1.3]` — MCP bridge `--prompt` / `--resource`, safer stdio spawn, TOON-style `--format toon`, `/healthz` inventory, etc.

## External service note: Petstore `getInventory`

Calling `getInventory` against the public Petstore may return **HTTP 500** or an error JSON body depending on load and server state. That indicates the **HTTP client successfully built and sent a request** (unlike the “builder error” above). Prefer **`findPetsByStatus`** for a stable positive response when smoke-testing the OpenAPI path.

## Installing the validated build locally

From crates.io (recommended; latest validated in this doc: **v0.1.3**):

```bash
cargo install sxmc
```

From a git checkout:

```bash
git fetch origin
git checkout main
cargo install --path . --force
```

Or build and use `target/release/sxmc` directly.

## References

- Fix commit on `master`: `54d58c2` — *Fix script path resolution and API builder error*
- Related maintainer doc: [SMOKE_TESTS.md](SMOKE_TESTS.md)
