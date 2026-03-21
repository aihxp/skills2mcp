# Validation run — **sxmc v0.1.8** (2026-03-21)

**Newer run:** see [`VALIDATION_RUN_v0.1.9.md`](VALIDATION_RUN_v0.1.9.md) for **v0.1.9** (JSON checks, `sxmc mcp session`, updated test counts).

This document records a **maintainer-style validation pass**: automated tests,
release certification, optional npm MCP smoke, wall-clock benchmarks,
**feature behavior** (not only performance), five real skills, five official
MCP servers, promptless “dialog” checks, **MCP -> CLI**, and **baked CLI ->
agent-style MCP** workflows.

## Environment

- **Host:** Linux x86_64
- **sxmc:** **0.1.8** — validated primarily with **`target/release/sxmc` built
  from this repo** (`cargo search sxmc` shows **0.1.8** on crates.io)
- **Node:** `npx` available for `@modelcontextprotocol/*` smoke scripts

## 1. Automated tests (`cargo test`)

| Suite | Count | Result |
|-------|------:|:------:|
| Library unit tests | **70** | pass |
| `src/main.rs` unit tests | **5** | pass |
| `tests/cli_integration.rs` | **44** | pass |
| Doc tests | **1** | pass |
| **Total** | **120** | **pass** |

**Finding:** Full suite matches the product claims in
[`PRODUCT_CONTRACT.md`](PRODUCT_CONTRACT.md) for covered paths.

## 2. Release certification (`scripts/certify_release.sh`)

```bash
bash scripts/certify_release.sh target/release/sxmc tests/fixtures
```

**Result:** **Passed**.

Includes packaging sanity, startup smoke, and client-style stdio/HTTP flows.

## 3. Real-world MCP smoke (`scripts/smoke_real_world_mcps.sh`)

```bash
bash scripts/smoke_real_world_mcps.sh target/release/sxmc
```

**Result:** **Passed**.

Covers:

- `@modelcontextprotocol/server-everything`
- `@modelcontextprotocol/server-memory`
- `@modelcontextprotocol/server-filesystem`
- `@modelcontextprotocol/server-sequential-thinking`
- `@modelcontextprotocol/server-github`

## 4. Benchmarks

Benchmarks were recorded for regression sanity only, not as the primary product
signal. Local one-shot paths remained fast; networked Petstore calls remained
network-dominated.

## 5. Five real-world skills

Five real-world skills were exercised through discovery, scan, and nested
`skills -> MCP -> CLI` flows.

**Finding:** Skills -> MCP -> CLI behaves as described in
[`USAGE.md`](USAGE.md): discovery works, nested stdio works, and hybrid tools,
prompts, and resources are visible on `--list`.

## 6. Feature focus

### 6.1 MCP -> CLI

- prompt-less/resource-less servers degrade cleanly and still exit `0` for
  supported discovery paths
- zero-argument tool calls work without manual `_={}` workarounds
- tool invocation returns expected text or structured output

### 6.2 Promptless “dialog”

Each `sxmc stdio ...` invocation starts a **new MCP session**. Two back-to-back
calls to a stateful tool like `sequentialthinking` both succeed, but do **not**
share session state.

**Finding:** Repeated tool calls work for automation; stateful multi-turn flows
belong in a long-lived host or custom client, not in separate one-shot CLI
invocations. The supported terminal path for that continuity is now
`sxmc mcp session <server>`.

### 6.3 CLI -> agent workflow (`bake` + `sxmc mcp`)

The token-aware workflow was validated:

```bash
sxmc bake create valrun018 --type stdio --source '["…/target/release/sxmc","serve","--paths","…/tests/fixtures"]'
sxmc mcp servers
sxmc mcp grep skill --limit 5
sxmc mcp call valrun018/get_skill_details '{"name":"simple-skill","return_type":"content"}' --pretty
sxmc bake remove valrun018
```

**Finding:** The `sxmc mcp` workflow works and matches the “CLI -> agent”
positioning: operators can drive MCP from the terminal the same way an agent
would discover and call tools, without pasting full schemas into chat.

## 7. Gaps / non-claims

- performance alone does not prove broad IDE/agent compatibility
- single-shot `sxmc` processes do not preserve MCP session memory between
  invocations; use `sxmc mcp session <server>` when continuity matters

## 8. Related

- [`VALIDATION.md`](VALIDATION.md)
- [`USAGE.md`](USAGE.md)
- [`PRODUCT_CONTRACT.md`](PRODUCT_CONTRACT.md)
