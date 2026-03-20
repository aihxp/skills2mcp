# Skills → MCP → CLI — sample outputs & findings

**Date:** 2026-03-20  
**sxmc:** 0.1.x (`sxmc --version`)  
**Purpose:** Capture **real terminal output** for the end-to-end pipeline where **skills are served as an MCP server** and **the same binary** acts as **MCP client → CLI**.

## Design (how the pipeline is supposed to behave)

1. **`sxmc serve --paths …`** loads skills and exposes them as an **MCP server** (hybrid model: meta tools like `get_available_skills`, per-skill **prompts**, **resources** where defined, and **script tools** when `scripts/` exists).
2. **`sxmc stdio "sxmc serve --paths …" …`** spawns that server and runs as an **MCP client**; results are printed to the terminal.
3. **Stderr** normally carries **startup / diagnostic** lines (`[sxmc] Loaded skill: …`). **Stdout** carries **tool results**, **`--list`** / **`--describe`** output, or **prompt/resource** content when requested.

This matches the **nested bridge** pattern used in `scripts/benchmark_cli.sh` (scenario **A** / **C**).

## Not the same path: `sxmc skills run`

**`sxmc skills run <name>`** reads the skill from disk and prints the **SKILL.md body** (with `$ARGUMENTS` substitution). It does **not** start MCP or call tools. See **§5** below for a contrast.

---

## 1. Skills → MCP → CLI: script tool invocation

**Command:**

```bash
cd /path/to/sxmc/repo
sxmc stdio "sxmc serve --paths tests/fixtures" skill_with_scripts__hello
```

**Sample output** (stderr + stdout):

```text
[sxmc] Loaded skill: malicious-skill
[sxmc] Loaded skill: simple-skill
[sxmc] Loaded skill: skill-with-references
[sxmc] Loaded skill: skill-with-scripts
[sxmc] Loaded 4 skills with 1 tools and 1 resources
Hello from script! Args: 
```

**Finding:** The **script’s stdout** appears on **stdout** after the child server’s load messages on **stderr**. **`--pretty`** may or may not change presentation depending on how the tool result is structured; for this fixture the text result was unchanged.

---

## 2. Skills → MCP: `--list` (tools, prompts, resources)

**Command:**

```bash
sxmc stdio "sxmc serve --paths tests/fixtures" --list
```

**Sample output** (abbreviated; stderr lines omitted after first block):

```text
[sxmc] Loaded skill: malicious-skill
[sxmc] Loaded skill: simple-skill
[sxmc] Loaded skill: skill-with-references
[sxmc] Loaded skill: skill-with-scripts
[sxmc] Loaded 4 skills with 1 tools and 1 resources
Tools (4):
  get_available_skills
    List available skills with their prompt, tool, and resource metadata
  get_skill_details
    Get detailed information for a skill, including its prompt body and file listing
  get_skill_related_file
    Read a file from within a skill directory using a safe relative path
  skill_with_scripts__hello
    Run hello.sh from skill 'skill-with-scripts'

Prompts (4):
  malicious-skill
    A test skill with security issues
  simple-skill
    A simple test skill
  skill-with-references
    A skill with reference documents
  skill-with-scripts
    A skill with executable scripts

Resources (1):
  style-guide.md (skill://skill-with-references/references/style-guide.md)
    Reference from skill 'skill-with-references'
```

**Finding:** **Hybrid exposure** is visible in one place: **four meta/helper tools**, **one script tool**, **four prompts** (one per skill), **one resource** from the fixture that defines references.

---

## 3. MCP → CLI only (external stdio server, no skills)

Shows the same **client → CLI** surface against a third-party server:

**Command:**

```bash
sxmc stdio "npx -y @modelcontextprotocol/server-everything" get-sum a=2 b=3 --pretty
```

**Sample output:**

```text
Starting default (STDIO) server...
The sum of 2 and 3 is 5.
```

**Finding:** Vendor banner may appear on **stderr** or **stdout** depending on the server; the **tool result** is still usable for scripting. Prefer **exit codes** and stable tool names for automation.

---

## 4. Contrast: direct `skills run` (disk → stdout, not MCP)

**Command:**

```bash
sxmc skills run simple-skill --paths tests/fixtures
```

**Sample output:**

```text
Hello , welcome to sxmc!
```

**Finding:** This is the **markdown body** of `simple-skill` with empty `$ARGUMENTS` — **not** execution via MCP. Use **`serve` + `stdio`** when you need **MCP tools/prompts/resources** semantics.

---

## 5. Summary table

| Entry point | MCP involved? | Typical stdout |
|-------------|---------------|----------------|
| `sxmc serve` | Server side only | (stdio/SSE transport; not the “CLI result” column) |
| `sxmc stdio "sxmc serve …" <tool>` | Yes (nested) | Tool / prompt / resource result |
| `sxmc stdio "sxmc serve …" --list` | Yes | Human-readable discovery |
| `sxmc skills run …` | **No** | Skill body text |

---

## Related docs

- [MCP_TO_CLI_VERIFICATION.md](MCP_TO_CLI_VERIFICATION.md) — bridge contract and manual checks  
- [CONNECTION_EXAMPLES.md](CONNECTION_EXAMPLES.md) — copy-paste connection patterns  
- [REAL_WORLD_SKILLS_AND_MCP_REPORT.md](REAL_WORLD_SKILLS_AND_MCP_REPORT.md) — five real skills + npm MCPs  
- [VALUE_AND_BENCHMARK_FINDINGS.md](VALUE_AND_BENCHMARK_FINDINGS.md) — benchmark methodology  
