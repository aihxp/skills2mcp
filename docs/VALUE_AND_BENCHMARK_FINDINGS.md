# sxmc — value proposition & benchmark findings

This note collects **why sxmc is useful**, **how to measure it**, and **representative timing** from automated CLI benchmarks. Token figures are **estimates** unless you instrument your own LLM client.

## What These Benchmarks Are And Are Not

The benchmark harnesses in this repository are meant to answer:

- how much local overhead `sxmc` adds on a normal machine
- whether common one-shot CLI paths regress noticeably
- whether process startup stays cheap enough to be negligible vs agent latency

They are not meant to prove:

- cross-client compatibility
- hosted deployment correctness
- Windows/macOS parity from a single Linux timing table
- product value from raw Petstore latency changes alone

Keep those concerns separate:

- compatibility and client coverage: [`COMPATIBILITY_MATRIX.md`](COMPATIBILITY_MATRIX.md)
- transport and startup pass/fail smoke checks: [`SMOKE_TESTS.md`](SMOKE_TESTS.md)
- manual real-world integration behavior: [`REAL_WORLD_SKILLS_AND_MCP_REPORT.md`](REAL_WORLD_SKILLS_AND_MCP_REPORT.md)

## Added value (summary)

| Capability | What it avoids / replaces |
|------------|---------------------------|
| **Skills → MCP** (`serve`) | A custom MCP adapter per skill repo; repeating SKILL.md bodies in chat |
| **MCP → CLI** (`stdio`, `http`) | Ad-hoc JSON-RPC clients; debugging only inside an IDE |
| **OpenAPI / GraphQL → CLI** (`api`, `spec`, `graphql`) | Pasting large specs; hand-written `curl` for every operation |
| **Security** (`scan`) | LLM-only “please audit this skill” passes (slow, variable) |
| **Distribution** | One Rust binary (plus optional wrappers) instead of several stacks |

**Even a single, narrow MCP server** often benefits from **`sxmc stdio …` /
`sxmc http …`**: scriptability, `--list` / `--pretty` inspection, CI,
debugging outside a full agent, and on-demand prompt/resource retrieval.

In practice, the bridge is still most valuable for **tool surfaces**, but the
same CLI now reaches prompts/resources directly with `--prompt` and
`--resource`, which helps when the useful context is descriptive rather than executable.

For API responses specifically, `sxmc` also supports `--format toon` as a
Rust-native TOON-style rendering for structured JSON. That is most useful when
responses contain repeated object keys, because the rendered output can compress
those keys into a tabular layout that is easier for both humans and models to scan.

## Representative wall-clock results (CLI)

Latest captured numbers: **[BENCHMARK_RUN_v0.1.6.md](BENCHMARK_RUN_v0.1.6.md)** (**sxmc 0.1.6**, **5 runs**, **median ms**, `scripts/benchmark_cli.sh`).  
Prior runs: [BENCHMARK_RUN_v0.1.5.md](BENCHMARK_RUN_v0.1.5.md), [BENCHMARK_RUN_v0.1.3.md](BENCHMARK_RUN_v0.1.3.md).

These timings reflect the default one-shot command paths. The optional
development feature `sxmc serve --watch` is intentionally outside this table;
it adds background polling for skill reloads, but does not change the default
startup/bridge path that the benchmarks are describing.

Environment: **Linux x86_64**. Petstore steps are **network-dominated**.

| Scenario | Command / step | Median (ms) @ v0.1.6 | Notes |
|----------|------------------|----------------------|--------|
| A | `sxmc stdio "sxmc serve --paths tests/fixtures" …` → `skill_with_scripts__hello` | **~11** | Fixture script; user-global skills can be a few ms higher |
| B | `sxmc api <petstore openapi> --list` | **~613** | Fetch + parse + network (varies by run) |
| B | `sxmc api … findPetsByStatus` | **~1227** | Same |
| B | `curl` to known Petstore URL only | **~409** | Lower bound: no spec in process |
| C | `sxmc stdio "sxmc serve --paths …/tests/fixtures" --list` | **~10** | Nested MCP bridge |
| D | `sxmc scan --paths … --skill malicious-skill` | **~11** | Exits non-zero when findings exist (by design) |
| Micro | Local OpenAPI + ephemeral HTTP + `sxmc api … listPets` | **~14** | Reduces WAN jitter |

**Takeaway:** sxmc adds **small** local overhead on top of I/O. The **big win** is usually **fewer agent turns and smaller prompts**, not microseconds saved on disk.

For startup-only timing, use the cross-platform helper:

```bash
python3 scripts/benchmark_startup.py /tmp/sxmc-startup-benchmark.md
```

That benchmark isolates `sxmc --version` and `sxmc --help`, which is useful for
platform-specific regressions that do not show up in longer transport timings.

## Token impact (order-of-magnitude estimates)

Not measured in-repo; use your provider dashboard.

| Workflow | Typical directional effect |
|----------|-----------------------------|
| MCP exposes skills vs **pasting SKILL.md + references each turn** | **Material drop** in repeated **input** tokens on later turns |
| `sxmc api` vs **attaching full OpenAPI JSON** | Often **thousands–tens of thousands** fewer input tokens when the spec would otherwise enter context (large specs can be **100k+ characters**) |
| `sxmc scan` vs **LLM security review** | **Near-zero** LLM tokens for the scan itself vs **hundreds–thousands+** for a serious review |

Rule of thumb for pasted English-ish text: **characters ÷ 4 ≈ rough token equivalents** (vary by tokenizer).

## Reproducing timings

See [E2E_VALIDATION_REPORT.md](E2E_VALIDATION_REPORT.md) for release validation. For a **repeatable local harness**, keep a script that:

1. Runs each command **N** times (e.g. 5).
2. Records **median** wall time (mean is ok; median resists Petstore flakes).
3. Uses a **local OpenAPI + tiny HTTP server** slice to separate client logic from WAN variance.

Use **`scripts/benchmark_cli.sh`** in this repository to regenerate timings.
Use **`scripts/benchmark_startup.py`** when you want a startup-only benchmark
that can run the same way on Linux, macOS, and Windows.

## Related docs

- [BENCHMARK_RUN_v0.1.6.md](BENCHMARK_RUN_v0.1.6.md) — v0.1.6 crates.io + `cargo test` counts
- [BENCHMARK_RUN_v0.1.5.md](BENCHMARK_RUN_v0.1.5.md) — prior captured run
- [BENCHMARK_RUN_v0.1.3.md](BENCHMARK_RUN_v0.1.3.md) — older capture
- [E2E_VALIDATION_REPORT.md](E2E_VALIDATION_REPORT.md) — v0.1.1 vs v0.1.2 validation
- [SMOKE_TESTS.md](SMOKE_TESTS.md) — MCP transport smoke tests
- [CLIENTS.md](CLIENTS.md) — Cursor / Codex / etc.
