# Client Setup

`sxmc` is designed first for stdio-based MCP clients, but it can also run as a
remote streamable HTTP MCP server at `/mcp`.

## Support Matrix

| Client | Local stdio MCP | Remote HTTP MCP | Status with `sxmc` |
|--------|------------------|-----------------|--------------------|
| Codex CLI / Codex IDE | Yes | Yes | Supported |
| Cursor | Yes | Yes | Supported |
| Gemini CLI | Yes | Yes | Supported |
| Claude Code and other local coding agents | Yes | Yes | Supported |
| ChatGPT Apps / Claude.ai connectors | No local stdio | Yes | Use the remote `/mcp` endpoint when those products accept remote MCP URLs |

## Codex

Codex can register local MCP servers directly from the CLI.

```bash
codex mcp add sxmc -- sxmc serve --paths /absolute/path/to/skills
```

Codex can also connect to a remote HTTP MCP server:

```bash
codex mcp add sxmc-remote --url http://127.0.0.1:8000/mcp
```

To confirm it is registered:

```bash
codex mcp list
```

If you need environment variables for skill execution, add them when registering:

```bash
codex mcp add sxmc --env FOO=bar -- sxmc serve --paths /absolute/path/to/skills
```

## Cursor

Cursor supports stdio servers through `mcp.json`. You can configure either:
- project-local: `.cursor/mcp.json`
- user/global: the Cursor MCP config location for your installation

Example:

```json
{
  "mcpServers": {
    "sxmc": {
      "type": "stdio",
      "command": "sxmc",
      "args": ["serve", "--paths", "/absolute/path/to/skills"]
    }
  }
}
```

After reloading Cursor, the `sxmc` prompts, tools, resources, and hybrid skill
retrieval tools should appear in the MCP tools UI.

If you host `sxmc` remotely, use Cursor's HTTP MCP configuration and point it at
`http://HOST:PORT/mcp`.

## Gemini CLI

Gemini CLI supports MCP servers from `.gemini/settings.json` or
`~/.gemini/settings.json`.

Example:

```json
{
  "mcpServers": {
    "sxmc": {
      "command": "sxmc",
      "args": ["serve", "--paths", "/absolute/path/to/skills"]
    }
  }
}
```

Then launch Gemini CLI and run:

```text
/mcp list
```

Gemini CLI can also package `sxmc` as part of a local extension if you want to
bundle a skills directory and a `GEMINI.md` context file together.

For a remote server, configure the MCP server URL as `http://HOST:PORT/mcp`.

## Claude Code and Similar Local MCP Clients

For local coding agents that accept a stdio MCP server definition, point them at:

```text
command: sxmc
args: ["serve", "--paths", "/absolute/path/to/skills"]
```

For remote-capable clients, host:

```bash
sxmc serve --transport http --host 0.0.0.0 --port 8000 --paths /absolute/path/to/skills
```

and use:

```text
http://YOUR_HOST:8000/mcp
```

Because `sxmc` exposes a hybrid surface, these clients can use:
- native prompts for skill bodies
- native resources for `references/`
- native tools for `scripts/`
- generic retrieval tools for `skills -> MCP -> CLI` compatibility

## Recommended Pattern

For broadest compatibility, prefer the hybrid pattern already implemented by
`sxmc serve`:

- `get_available_skills`
- `get_skill_details`
- `get_skill_related_file`

Those generic tools are the most portable across clients that are better at
tool calling than prompt/resource handling.
