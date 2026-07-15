# Harness pipeline surfaces

Compile reusable repository workflows to skills for Claude Code (`.claude/skills/`) and Codex
(`.agents/skills/`). Codex custom prompts are deprecated and user-scoped. GitHub Copilot repository
prompt files remain useful but are IDE-dependent and public preview.

Report gate fidelity per gate and target. A declared policy is not mechanically enforced until the
adapter emits and verifies a native binding for that exact policy. A binding that shells out to an
external binary also depends on that binary resolving at runtime — harnesses treat a missing hook
command as non-blocking, so enforcement degrades silently. Fidelity reporting must probe the live
environment, not just the emitted config.

