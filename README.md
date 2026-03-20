# sxmc

AI-agnostic Skills x MCP x CLI — a single Rust binary that turns skills into MCP servers, MCP servers into CLI commands, and any API into a CLI.

```
Skills  -->  MCP Server    (serve skills to any MCP client)
MCP Server  -->  CLI       (turn any MCP server into CLI commands)
Any API  -->  CLI           (OpenAPI & GraphQL auto-detection)
```

## Install

```bash
cargo install --git https://github.com/aihxp/sxmc
```

Or build from source:

```bash
git clone https://github.com/aihxp/sxmc.git
cd sxmc
cargo build --release
# Binary at target/release/sxmc
```

## Quick Start

### Serve skills as an MCP server

```bash
# stdio (for MCP client configs)
sxmc serve

# HTTP/SSE for remote access
sxmc serve --transport sse --port 8000
```

Add to any MCP client config:

```json
{ "mcpServers": { "skills": { "command": "sxmc", "args": ["serve"] } } }
```

### Run a skill directly

```bash
sxmc skills list
sxmc skills run pr-review 42
```

### Any MCP server as CLI

```bash
# stdio server
sxmc stdio "npx @mcp/github" --list
sxmc stdio "npx @mcp/github" search-repos query=rust

# HTTP server
sxmc http https://mcp.example.com/sse --list
sxmc http https://mcp.example.com/sse my-tool key=value
```

### Any API as CLI

```bash
# Auto-detect (OpenAPI or GraphQL)
sxmc api https://petstore.swagger.io/v3/openapi.json --list
sxmc api https://petstore.swagger.io/v3/openapi.json listPets limit=10

# Explicit modes
sxmc spec ./openapi.yaml listPets limit=10
sxmc graphql https://api.example.com/graphql users limit=5
```

### Security scanning

```bash
sxmc scan                                     # scan all skills
sxmc scan --skill my-skill                    # scan one skill
sxmc scan --severity critical                 # filter by severity
sxmc scan --json                              # JSON output
```

### Bake and reuse connections

```bash
sxmc bake create pets --type spec --source https://petstore.swagger.io/v3/openapi.json
sxmc bake list
sxmc bake show pets
```

### Generate skills from APIs

```bash
sxmc skills create https://api.example.com/openapi.json
# Creates a SKILL.md with all operations documented
```

## Skills

Skills are directories containing a `SKILL.md` file with YAML frontmatter and a markdown body. They can optionally include `scripts/` (executable tools) and `references/` (context resources).

```
my-skill/
  SKILL.md          # Required: frontmatter + instructions
  scripts/           # Optional: become MCP tools
    deploy.sh
  references/        # Optional: become MCP resources
    style-guide.md
```

### SKILL.md format

```markdown
---
name: my-skill
description: "What this skill does"
argument-hint: "<repo> [--dry-run]"
allowed-tools:
  - Bash
  - Read
---

Instructions for the AI when this skill is invoked.

Use $ARGUMENTS for user-provided arguments.
```

### Skill discovery

Skills are discovered from (in priority order):
1. `--paths` flag (explicit)
2. `.claude/skills/` (project-local)
3. `~/.claude/skills/` (user-global)

## Security Scanning

sxmc includes a native Rust security scanner that analyzes skills and MCP servers for threats. Scans run automatically when serving skills or connecting to servers.

### What it detects

**Skill scanning:**
- Prompt injection patterns (ignore instructions, role switching, jailbreak attempts)
- Hidden Unicode characters (zero-width spaces, RTL overrides, homoglyphs)
- Hardcoded secrets (AWS keys, GitHub tokens, API keys, passwords)
- Dangerous script operations (rm -rf, chmod 777, eval, curl|bash)
- Data exfiltration patterns (webhook posts, DNS exfil)
- Overly broad tool permissions (wildcard `*`, dangerous tool names)

**MCP server scanning:**
- Tool shadowing (servers overriding trusted tools)
- Prompt injection in tool descriptions
- Excessive permission requests
- Overly permissive input schemas

### Severity levels

| Level | Meaning |
|-------|---------|
| `info` | Informational, no action needed |
| `warning` | Potential issue, review recommended |
| `error` | Likely security problem |
| `critical` | Definite threat, blocks execution |

## Architecture

```
sxmc
├── Security Layer
│   ├── Skill Scanner — prompt injection, secrets, hidden chars
│   └── MCP Scanner  — tool shadowing, response injection
├── Server Side
│   └── Discovery → Parser → Security Scan → MCP Server (rmcp)
└── Client Side
    ├── MCP Client — stdio & HTTP/SSE transports
    ├── OpenAPI    — spec parsing + HTTP execution
    ├── GraphQL    — introspection + query building
    ├── Bake       — saved connection configs
    └── Cache      — file-based with TTL
```

Built on [rmcp](https://github.com/nicepkg/rmcp) (official Rust MCP SDK).

## CLI Reference

```
sxmc [subcommand] [options]

SERVER:
  serve [--paths ...] [--transport stdio|sse] [--port 8000]

SKILLS:
  skills list [--paths ...] [--json]
  skills info <name> [--paths ...]
  skills run <name> [args...] [--paths ...]
  skills create <api-url> [--output DIR] [--auth-header K:V]

CLIENT:
  stdio <command> [tool] [args...] [--list] [--search] [--pretty] [--env K=V]
  http <url> [tool] [args...] [--list] [--search] [--pretty] [--auth-header K:V]
  api <source> [operation] [args...] [--list] [--auth-header K:V]
  spec <source> [operation] [args...] [--list] [--auth-header K:V]
  graphql <url> [operation] [args...] [--list] [--auth-header K:V]

SECURITY:
  scan [--paths ...] [--skill <name>] [--severity warn|error|critical] [--json]

BAKE:
  bake create <name> --type <stdio|http|spec|graphql> --source <src> [--description ...]
  bake list
  bake show <name>
  bake update <name> [--source ...] [--description ...]
  bake remove <name>
```

## Development

```bash
# Run tests (66 total: 52 unit + 14 integration)
cargo test

# Build
cargo build --release

# Run directly
cargo run -- skills list --paths tests/fixtures
cargo run -- scan --paths tests/fixtures
```

## License

MIT
