# CLI -> AI Surfaces

This document turns the `CLI -> AI surfaces` idea into a concrete product model
for `sxmc`.

Current shipped path:

```bash
sxmc inspect cli gh --format toon
sxmc init ai --from-cli gh --client claude-code --mode preview
sxmc init ai --from-cli gh --coverage full --mode preview
sxmc init ai --from-cli gh --coverage full --host claude-code,cursor --mode apply
sxmc inspect profile examples/profiles/from_cli.json --format toon
sxmc inspect profile examples/profiles/from_generated_cli.json --pretty
```

That keeps JSON as the canonical profile format while allowing a more compact,
TOON-style view when you want to inspect large profiles quickly.

## Goal

Take a real CLI or a generated CLI wrapper, inspect it into a normalized JSON
profile, and generate agent-ready scaffolds from that profile.

## What Counts As An AI Surface

`AI surfaces` in `sxmc` should stay intentionally small and explicit.

Supported target surface types:

- `skill_markdown`
  - a `SKILL.md`-style artifact
- `agent_doc_snippet`
  - a suggested block for files like `AGENTS.md`, `CLAUDE.md`, or similar
- `mcp_wrapper_scaffold`
  - a scaffold for turning the CLI into an MCP-facing wrapper
- `client_config_snippet`
  - a small example block for a client or agent setup file

Current full-coverage host outputs:

- portable `AGENTS.md`
- `CLAUDE.md`
- `.cursor/rules/sxmc-cli-ai.md`
- `GEMINI.md`
- `.github/copilot-instructions.md`
- `.continue/rules/sxmc-cli-ai.md`
- `opencode.json`
- `.aiassistant/rules/sxmc-cli-ai.md`
- `.junie/guidelines.md`
- `.windsurf/rules/sxmc-cli-ai.md`
- host config scaffolds for Claude Code, Cursor, Gemini CLI, OpenAI/Codex, and generic stdio/http MCP

Out of scope by default:

- rewriting an entire existing `AGENTS.md` / `CLAUDE.md`
- autonomous multi-file project refactors
- automatic commits of generated docs without review

## Pipeline Model

The intended pipeline is:

```text
real CLI -> normalized JSON profile -> chosen AI surface(s)
```

Concrete shipped pipeline:

```text
CLI binary
  -> sxmc inspect cli <command>
  -> JSON profile
  -> sxmc init ai / sxmc scaffold ...
  -> startup-ready AI files
```

### Stage 1: `sxmc inspect cli <command>`

`inspect cli` runs a real CLI, reads its help surface, and normalizes that into
the canonical JSON profile.

The profile is designed to capture things agents and startup docs actually need:

| Field area | What it captures |
|---|---|
| `summary` / `description` | high-level purpose and one-line description |
| `subcommands` | names, summaries, and confidence levels |
| `options` | long flags, short flags, value names, and required/optional hints |
| `examples` | usage examples recovered from help text |
| `environment` | auth or env requirements when they are visible |
| `output_behavior` | whether the command looks machine-friendly or human-oriented |
| `confidence_notes` | how much of the profile was observed vs inferred |
| `provenance` | generator, version, source command, and generation time |

Because this comes from CLI help, some fields are directly observed while
others are heuristic. That is why the profile carries confidence metadata.

Current parser hardening focuses on keeping shallow inspection useful for real
developer CLIs:

- richer `--help` output stays primary when available
- `man` pages are used as targeted fallback/supplementation, especially for BSD
  and macOS tools
- grouped command layouts like `CORE COMMANDS` and npm-style command lists are
  normalized into subcommands
- obvious environment-variable symbols are filtered so they do not become fake
  subcommands
- top-level global flags are recovered even when a CLI mostly emphasizes
  subcommands

### Stage 2: `sxmc init ai --from-cli <command> --client <host>`

`init ai` takes the profile and generates startup-facing files for a selected
host profile.

Current native targets:

| AI host | Agent doc path | Config path |
|---|---|---|
| Claude Code | `CLAUDE.md` | `.sxmc/ai/claude-code-mcp.json` |
| Cursor | `.cursor/rules/sxmc-cli-ai.md` | `.cursor/mcp.json` |
| Gemini CLI | `GEMINI.md` | `.gemini/settings.json` |
| GitHub Copilot | `.github/copilot-instructions.md` | none |
| Continue | `.continue/rules/sxmc-cli-ai.md` | none |
| OpenCode | `AGENTS.md` | `opencode.json` |
| JetBrains AI Assistant | `.aiassistant/rules/sxmc-cli-ai.md` | none |
| Junie | `.junie/guidelines.md` | none |
| Windsurf | `.windsurf/rules/sxmc-cli-ai.md` | none |
| OpenAI/Codex | `AGENTS.md` | `.codex/mcp.toml` |
| Generic stdio MCP | `AGENTS.md` portable fallback | `.sxmc/ai/generic-stdio-mcp.json` |
| Generic HTTP MCP | `AGENTS.md` portable fallback | `.sxmc/ai/generic-http-mcp.json` |

With `--coverage full`, `sxmc` generates the broad host set together. With
`--coverage full --mode apply`, selected hosts are applied and the rest stay as
sidecars.

### Stage 3: `sxmc scaffold`

`scaffold` produces deeper artifacts from the same profile:

| Command | Output |
|---|---|
| `scaffold llms-txt` | `llms.txt` export |
| `scaffold skill` | `SKILL.md` scaffold |
| `scaffold mcp-wrapper` | `README.md` + `manifest.json` scaffold |
| `scaffold agent-doc` | managed markdown block for one host |
| `scaffold client-config` | host config scaffold for one host |

Optional broader product graph:

```text
Skills -> MCP -> Generated CLI -> AI surfaces
MCP -> Generated CLI -> AI surfaces
real CLI -> AI surfaces
```

Each hop is explicit. `sxmc` should not silently chain multiple hops.

## Generated CLI Boundary

`sxmc stdio` / `sxmc http` are runtime bridge commands, not generated CLI
artifacts.

If `sxmc` later supports `MCP -> Generated CLI`, that generated CLI should be
treated as a separate artifact class with provenance metadata.

Two distinct categories:

- `runtime_bridge`
  - ephemeral CLI behavior like `sxmc stdio "..." get-sum`
- `generated_cli`
  - a standalone wrapper script, binary scaffold, or command project emitted by `sxmc`

Only `generated_cli` should be considered a new inspectable artifact, and even
then only with explicit user opt-in.

## Provenance And Loop Prevention

Every generated artifact should carry provenance metadata.

Minimum fields:

- `generated_by`
- `generator_version`
- `source_kind`
- `source_identifier`
- `profile_schema`
- `generation_depth`
- `generated_at`

Loop-safety rules:

1. Real sources are inspectable by default.
2. Generated sources are not inspectable by default.
3. `generation_depth` defaults to `0` for real sources and increments for each generated artifact.
4. Default maximum generation depth is `1`.
5. Going beyond `1` should require explicit opt-in.
6. Self-targeting should be blocked by default for `sxmc` itself unless explicitly allowed.

## Review / Apply Model

Default outputs should be reviewable, not silently applied.

Preferred order of operations:

1. `stdout` preview
2. sidecar output file
3. patch preview
4. explicit write/apply

Current modes map to that model like this:

| Mode | Behavior |
|---|---|
| `preview` | prints what would be generated; writes nothing |
| `write-sidecar` | writes under `.sxmc/ai/...`; does not touch real startup files |
| `patch` | prints a patch-style preview for apply-capable targets |
| `apply` | updates real files via managed blocks or mergeable config shapes |

Full-coverage rule:

- use `--coverage full` for preview or sidecar generation across multiple tools
- use `--coverage full --host ... --mode apply` only for the host-native files you actually want to update
- non-selected hosts should stay as sidecars during `apply`

Additional optional export:

- `sxmc scaffold llms-txt --from-profile ...`
  - emits an `llms.txt`-style summary from the same CLI profile

Recommended default behavior:

- `--print`
  - print the generated artifact or profile to stdout
- `--output`
  - write a sidecar file
- `--patch`
  - emit a patch or diff preview
- `--write`
  - write new files only
- `--apply`
  - reserved for explicit future mutation of existing docs

For agent-doc outputs, `--apply` should never be the default.

Existing startup docs are not overwritten wholesale. `AGENTS.md`, `CLAUDE.md`,
and similar files are updated with managed `sxmc` blocks so generated guidance
can coexist with human-written project context.

## Intermediate Representation

The CLI profile JSON is the product contract.

It should include:

- command identity
- summary/description
- subcommands
- options/flags
- positionals
- examples
- auth/environment requirements
- output behavior
- inferred workflows
- provenance

This schema should be versioned. The initial schema name should be:

- `sxmc_cli_surface_profile_v1`

## Confidence Model

CLI inspection is inherently imperfect. Generated output should reflect that.

Suggested levels:

- `high`
  - directly observed in help or examples
- `medium`
  - inferred from structure or naming
- `low`
  - heuristic guess that should be reviewed carefully

Generated artifacts should prefer high-confidence inputs and clearly separate
inference from observation.

## Practical Product Shape

The current command family is:

- `sxmc inspect cli <command>`
- `sxmc inspect profile <file>`
- `sxmc init ai --from-cli <command> --client <profile>`
- `sxmc scaffold skill --from-profile <file>`
- `sxmc scaffold agent-doc --from-profile <file> --client <profile>`
- `sxmc scaffold client-config --from-profile <file> --client <profile>`
- `sxmc scaffold mcp-wrapper --from-profile <file>`

This keeps inspection deterministic and generation reviewable while making the
startup-discovery path explicit.

## Recommended 1.0 Scope

Current shipped scope:

- one versioned JSON profile schema
- one deterministic inspection path
- one skill scaffold target
- one host-aware agent-doc target
- one host-aware client-config scaffold target
- one MCP-wrapper scaffold target
- provenance on every generated artifact
- preview/sidecar/patch/apply modes
- no default mutation of existing repo docs
