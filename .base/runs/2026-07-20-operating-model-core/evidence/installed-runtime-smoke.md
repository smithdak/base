# Installed runtime smoke evidence

Date: 2026-07-20

## Installed core and library

- `base --version` -> `base 0.2.0`
- `base __hook capabilities --require ">=0.2.0, <0.3.0" --require-feature pre-tool --require-feature policy`
  -> protocol 1, version 0.2.0, features `pre-tool` and `policy`
- `base check --json` -> `valid: true`, `warnings: []`
- `base sync --check --json` -> `drift: []`
- `base init --global --packs-only --force` -> 18 bundled pack files unchanged; global
  `software-delivery` version 1.2.0 present; existing `sitecore/pack.md` present

## Hook protocol probes

The same explicit `git push origin main` event returned a deny decision for `claude`, `codex`, and
`copilot`. The Claude/Codex response used `hookSpecificOutput`; Copilot used its flat
`permissionDecision` response.

A Copilot event named `github-mcp-server-push_files` with `branch: main` returned a deny decision.
The corresponding `github-mcp-server-get_file_contents` event reading the configured approval path
returned no denial. The `session-context` policy returned both W-0011 and
`2026-07-20-operating-model-core` for all three target response protocols.

## Live Copilot falsifier

A local Copilot CLI 1.0.72 probe exposed the built-in GitHub tool as
`github-mcp-server-get_file_contents` / `github-mcp-server/get_file_contents` and successfully
returned a tool result. This falsified the earlier `mcp__github__*`-only Copilot matcher. The adapter
now maps canonical GitHub MCP globs to `github-mcp-server-*`; arbitrary unmapped Copilot MCP servers
report `partial-hook`.

## Scope limit

These direct protocol probes do not prove a live Claude session, Codex hook trust, Linux execution,
or a post-fix Copilot cloud image. Copilot's outer host timeout remains fail-open by contract.
