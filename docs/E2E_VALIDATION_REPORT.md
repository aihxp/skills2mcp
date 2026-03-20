# End-to-end validation report

This document records a full validation pass of **sxmc** (skills, MCP stdio bridge, OpenAPI `api` command, scan, bake) and contrasts **crates.io v0.1.1** behavior with the fixes now landed on `master` and prepared for **v0.1.2**.

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
- **21** `tests/cli_integration.rs` integration tests
- **1** doc test

Added coverage: `test_extract_base_url_relative_server` in `src/client/openapi.rs`.

## Manual end-to-end checks (release binary)

Build:

```bash
cargo build --release
# Binary: target/release/sxmc
```

| Check | Command / expectation | With fixes on `master` / v0.1.2 |
|-------|----------------------|----------------------|
| Skills | `sxmc skills list` / `info` / `run` | OK |
| Scan | `sxmc scan`, `sxmc scan --json` | OK |
| MCP list tools | `sxmc stdio "sxmc serve" --list` | OK |
| MCP run script | `sxmc stdio "sxmc serve" <tool>` **without** `--paths` | OK (prints script output) |
| OpenAPI list | `sxmc api https://petstore3.swagger.io/api/v3/openapi.json --list` | OK |
| OpenAPI call | `sxmc api … findPetsByStatus status=available` | OK (JSON from API) |
| Fixtures | `sxmc skills list --paths tests/fixtures` and `sxmc stdio "sxmc serve --paths $(pwd)/tests/fixtures" --list` | OK |

**Contrast with crates.io v0.1.1 (unpatched):** MCP script invocation without `--paths` and live `api` calls (same Petstore URL) reproduce the two issues above.

## External service note: Petstore `getInventory`

Calling `getInventory` against the public Petstore may return **HTTP 500** or an error JSON body depending on load and server state. That indicates the **HTTP client successfully built and sent a request** (unlike the “builder error” above). Prefer **`findPetsByStatus`** for a stable positive response when smoke-testing the OpenAPI path.

## Installing the validated build locally

Until crates.io `v0.1.2` ships the fixes:

```bash
git fetch origin
git checkout master
cargo install --path . --force
```

Or build and use `target/release/sxmc` directly.

## References

- Fix commit on `master`: `54d58c2` — *Fix script path resolution and API builder error*
- Related maintainer doc: [SMOKE_TESTS.md](SMOKE_TESTS.md)
