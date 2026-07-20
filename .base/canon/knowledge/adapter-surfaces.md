# Harness pipeline surfaces

Compile repository skills to Claude Code (`.claude/skills/`) and the open Agent Skills surface
(`.agents/skills/`) shared by Codex and GitHub Copilot. Compile agents to each target's native
project surface. Pipelines use a Claude skill, a shared Codex/Copilot Agent Skill, and a separate VS
Code prompt-file profile for Copilot.

All three targets have repository lifecycle hooks for equivalent events. Codex project hooks
require explicit trust; Codex session-end remains assisted because `Stop` is turn-scoped. Hook
execution requires a `requires-base`-compatible binary in the target environment.

Report fidelity, product profile, scope, and runtime/trust prerequisites per target. `native-hook`
means a documented lifecycle binding, not an authorization boundary; `hybrid-hook` means pending
approval is mechanical while denial routing remains behavioral; `partial-hook` means the native
binding does not cover the complete declared tool domain. Allowlisted target-specific migration
input lives under `.base/native/` and is composed into hash-owned output. Protect default branches
at the Git server. Copilot sanitizes the current built-in GitHub MCP namespace to
`github-mcp-server-*`; map it explicitly, and report arbitrary unmapped Copilot MCP matchers as
`partial-hook` rather than guessing.
