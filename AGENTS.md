<!-- sxmc:begin cli-ai:portable -->
## sxmc CLI Surface: `sxmc`

Use `sxmc` as a portable terminal workflow across AI tools in this repo.

Summary: AI-agnostic Skills × MCP × CLI

Recommended startup guidance:
- When the exact CLI surface is unclear, start with `sxmc inspect cli <tool> --depth 1 --format json-pretty`.
- For this CLI specifically, `sxmc` `--help` is still a good follow-up once you know you are on the right command.
- When the MCP surface is unknown, start with `sxmc stdio "<cmd>" --list` or `sxmc mcp grep <pattern>`.
- When the API surface is unknown, start with `sxmc api <url-or-spec> --list`.
- Prefer machine-friendly flags like `--json` when available.
- Keep bulky output in files or pipes instead of pasting it into chat context.
- Re-check auth or environment requirements before write actions.
- Host profile conventions in this repo were last verified on 2026-03-21.

High-confidence subcommands:
- `serve`: Start the MCP server (serves skills over MCP)
- `skills`: Manage skills
- `stdio`: Connect to an MCP server via stdio (MCP Server → CLI)
- `http`: Connect to an MCP server via HTTP (MCP Server → CLI)
- `mcp`: Use baked MCP connections in a token-efficient, mcp-cli-style workflow
<!-- sxmc:end cli-ai:portable -->
