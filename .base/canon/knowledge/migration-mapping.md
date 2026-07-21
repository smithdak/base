# Migrating another system into a base pack

How to turn another agent system into a base pack that reproduces its behavior and improves it by
adding base's operating model. `base ingest` models the source and reports fidelity; this doctrine
governs the authored rewrite that follows. Claude Code source formats verified against
code.claude.com/docs on 2026-07-21; re-verify before changing this mapping (D-025).

## Plugin ≈ pack

A Claude Code plugin (`.claude-plugin/plugin.json`) bundles agents, skills, commands, hooks, and MCP
into one distributable — near 1:1 with a base pack. The manifest's name/version/description become
`pack.md`; author/homepage/keywords become provenance in the manifest body. When a plugin manifest is
present it is the primary source; otherwise the reader falls back to loose `.claude/` directories.

## Source → canon

| Source (Claude Code) | Canon kind | Notes |
|---|---|---|
| plugin manifest | `pack.md` | near 1:1; carry provenance into the body |
| `.claude/agents/<id>.md` | agent | `name`→`id`; `tools`, `skills`; `permissionMode: plan`→`access: read-only` |
| `.claude/skills/<id>/` | skill *or* pipeline | single capability → skill; ordered, gated procedure → pipeline |
| `.claude/commands/<id>.md` | skill / pipeline | legacy (merged into skills v2.1.145+); a same-named skill wins |
| `.claude/settings.json` hooks | policy | only `session-start`/`pre-tool-use`/`post-tool-use`/`session-end` and `type: command` map |
| `.claude/settings.json` `permissions.deny` | gate (standing-denial) | e.g. a default-branch push deny |
| `CLAUDE.md` prose | rules + knowledge | split durable rules from reference knowledge; resolve `@imports` |
| `.mcp.json` / inline `mcpServers` | out of canon | MCP stays harness config, admitted by rule (D-015) |

## Fidelity is honest, never faked

The reader labels each mapping `native | partial | manual | out-of-canon`. Modern subagents (~16
frontmatter fields) and hooks (~30 events, 5 hook types) exceed what vendor-neutral canon represents.
Claude-only knobs (`model`, `effort`, `background`, `isolation`, `memory`, `color`, per-agent
`hooks`, non-`command` hook types, hook events outside the four canon events) have no canon home:
report them and decide deliberately. Never dress a partial mapping up as native, and never drop a
surface without recording the decision.

## What base adds

A faithful reproduction is the floor. Every migration also gains:

- work items + kanban with explicit human verdicts;
- stage-approval gates recorded as artifacts, not utterances;
- runs + an append-only history ledger;
- typed verifiers (pass | fail | inconclusive), never assumed success;
- durable handoff + pickup for cross-session continuity;
- cross-harness compilation — a Claude-only source now also emits Codex and Copilot surfaces;
- drift-protected generated output via the sync manifest.
