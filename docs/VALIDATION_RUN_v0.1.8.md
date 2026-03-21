# Validation run — **sxmc v0.1.8** (2026-03-21)

This document records a **maintainer-style validation pass**: automated tests, release certification, optional npm MCP smoke, wall-clock benchmarks, **feature behavior** (not only performance), five real skills, five official MCP servers, promptless “dialog” checks, **MCP → CLI**, and **baked CLI → agent-style MCP** workflows.

## Environment

- **Host:** Linux x86_64  
- **sxmc:** **0.1.8** — validated primarily with **`target/release/sxmc` built from this repo** (`cargo search sxmc` shows **0.1.8** on crates.io)  
- **Node:** `npx` available for `@modelcontextprotocol/*` smoke scripts  

---

## 1. Automated tests (`cargo test`)

| Suite | Count | Result |
|-------|------:|:------:|
| Library unit tests | **70** | pass |
| `src/main.rs` unit tests | **5** | pass |
| `tests/cli_integration.rs` | **44** | pass |
| Doc tests | **1** | pass |
| **Total** | **120** | **pass** |

**Finding:** Full suite **matches product claims** in [`PRODUCT_CONTRACT.md`](PRODUCT_CONTRACT.md) for covered paths (skills → MCP, MCP → CLI stdio/http, `sxmc mcp`, bake, APIs, scan, etc.).

---

## 2. Release certification (`scripts/certify_release.sh`)

```bash
bash scripts/certify_release.sh target/release/sxmc tests/fixtures
```

**Result:** **Passed** (`Release certification checks passed.`).

Includes packaging sanity, startup smoke, and client-style stdio/HTTP flows.

---

## 3. Real-world MCP smoke (`scripts/smoke_real_world_mcps.sh`)

```bash
bash scripts/smoke_real_world_mcps.sh target/release/sxmc
```

**Result:** **Passed** (`Real-world MCP smoke checks passed.`)

Covers the same **five** official servers as in-repo docs:

| Server | Checks |
|--------|--------|
| `@modelcontextprotocol/server-everything` | `--list` contains tools + prompts; `get-sum` → **5** |
| `@modelcontextprotocol/server-memory` | `--list`; `read_graph` **without** extra dummy args (see §6) |
| `@modelcontextprotocol/server-filesystem /tmp` | `--list`; `list_allowed_directories` |
| `@modelcontextprotocol/server-sequential-thinking` | `sequentialthinking` tool call |
| `@modelcontextprotocol/server-github` | `--list` non-empty |

---

## 4. Benchmarks (`scripts/benchmark_cli.sh`, 5 runs, **median ms**)

**Not the main product signal** (see [`VALIDATION.md`](VALIDATION.md)); recorded for **regression sanity** only.

| Scenario | Step | Median (ms) |
|----------|------|------------:|
| A | `stdio` → `serve --paths tests/fixtures` → `skill_with_scripts__hello` | **11** |
| B | `api` Petstore `--list` | **612** |
| B | `api` `findPetsByStatus` | **1095** |
| B | `curl` only | **408** |
| C | Nested `stdio` `serve` `--list` | **11** |
| D | `scan` `malicious-skill` | **11** |
| Micro | Local OpenAPI + tiny HTTP `listPets` | **14** |

Petstore numbers are **network-dominated** (large run-to-run variance is expected).

---

## 5. Five real-world skills (local symlinks)

Bundle: `/tmp/sxmc-realworld-skills` → `system-info`, Cursor `create-skill` / `shell`, OpenClaw `github` / `summarize` (see [`USAGE.md`](USAGE.md) for agent-oriented guidance).

| Check | Result |
|-------|--------|
| `sxmc skills list --paths …` | **OK** — five skills discoverable |
| `sxmc scan --paths … --skill <name>` | **All [PASS]** at default severity |
| Nested bridge with **JSON-array** stdio spec (0.1.x): `sxmc stdio '["…/sxmc","serve","--paths","/tmp/sxmc-realworld-skills"]' --list` | **exit 0** — tools / prompts / resources listed |

**Finding:** **Skills → MCP → CLI** behaves as described in [`USAGE.md`](USAGE.md): discovery and nested stdio work; hybrid tools + prompts + resources visible on `--list`.

---

## 6. Feature focus: does it match the description?

### 6.1 MCP → CLI (ad hoc `stdio` / `http`)

- **`--list` / optional prompts & resources:** Prompt-less servers skip surfaces with notices; **exit 0** (smoke + integration tests).  
- **Zero-argument tools:** **`read_graph`** and **`list_allowed_directories`** succeed in **`smoke_real_world_mcps.sh`** with no ` _={}` workaround — aligns with **v0.1.7** changelog: *“zero-argument MCP tool calls now send `{}` by default”*.  
- **Tool invocation:** `get-sum`, `sequentialthinking`, etc. return structured or text output as expected.

### 6.2 “Dialog” on promptless MCP (multiple invocations)

Each `sxmc stdio …` run is a **new MCP session** (by design). Two back-to-back **`sequentialthinking`** calls:

- Both **exit 0**  
- JSON shows **`thoughtHistoryLength": 1`** each time — **no shared session state** between processes (expected).

**Finding:** **Repeated tool calls work** for automation; **stateful multi-turn** chains belong in a long-lived host (IDE/agent) or custom client—not multiple standalone `sxmc` invocations.

### 6.3 CLI → agent workflow (`bake` + `sxmc mcp`)

Validated the flow from [`USAGE.md`](USAGE.md) and [`examples/agent-docs/AGENTS.md.snippet`](../examples/agent-docs/AGENTS.md.snippet):

```bash
sxmc bake create valrun018 --type stdio --source '["…/target/release/sxmc","serve","--paths","…/tests/fixtures"]'
sxmc mcp servers
sxmc mcp grep skill --limit 5
sxmc mcp call valrun018/get_skill_details '{"name":"simple-skill","return_type":"content"}' --pretty
sxmc bake remove valrun018
```

**Result:** **Servers listed**, **grep** finds hybrid tools, **`mcp call`** returns the **simple-skill** body as structured output.

**Finding:** The **token-aware MCP workflow** (`mcp servers` → `grep` / `tools` → `info` → `call`) **works** and matches the **“CLI → agent”** positioning: operators can drive MCP **from the terminal** the same way an agent would discover and call tools, without pasting full schemas into chat.

---

## 7. Gaps / non-claims

- **Performance** alone does not prove **IDE/agent compatibility**; use [`PRODUCT_CONTRACT.md`](PRODUCT_CONTRACT.md) + real client testing.  
- **Single-shot `sxmc` processes** do not preserve MCP session memory between invocations (§6.2).  

---

## 8. Related

- [`VALIDATION.md`](VALIDATION.md) — ongoing checklist  
- [`USAGE.md`](USAGE.md) — intended workflows  
- [`PRODUCT_CONTRACT.md`](PRODUCT_CONTRACT.md) — support boundary  
