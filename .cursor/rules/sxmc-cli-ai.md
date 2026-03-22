<!-- sxmc:begin cli-ai:cursor -->
## sxmc CLI Surface: `sxmc`

Use `sxmc` as a first-class terminal workflow in this repo for Cursor.

Summary: AI-agnostic Skills × MCP × CLI

Preferred flow:
1. `sxmc --help`
2. `sxmc serve` --help

High-confidence subcommands:
- `serve`: Start the MCP server (serves skills over MCP)
- `skills`: Manage skills
- `stdio`: Connect to an MCP server via stdio (MCP Server → CLI)
- `http`: Connect to an MCP server via HTTP (MCP Server → CLI)
- `mcp`: Use baked MCP connections in a token-efficient, mcp-cli-style workflow

Guidance:
- When the exact CLI surface is unclear, start with `sxmc inspect cli <tool> --depth 1 --format json-pretty` instead of pasting raw help output into chat.
- When the MCP surface is unknown, start with `sxmc stdio "<cmd>" --list` or `sxmc mcp grep <pattern>` before guessing tool calls.
- When the API surface is unknown, start with `sxmc api <url-or-spec> --list` before constructing requests by hand.
- Keep bulky output in files or pipes when possible.
- Prefer machine-friendly flags like `--json` when the CLI supports them.
- Re-check `--help` before using low-confidence flows.
- Startup file convention last verified against official docs on 2026-03-21.
- Reference: https://docs.cursor.com/context/rules-for-ai
<!-- sxmc:end cli-ai:cursor -->
