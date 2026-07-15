# Plan

1. Define the Rust CLI, config model, canon parser, merge rules, and validation.
2. Scaffold a usable default canon with rules, two agents, four stages, and one gated pipeline.
3. Compile current native surfaces for Claude Code, Codex, and GitHub Copilot.
4. Add SHA-256 manifest drift protection and safe stale-output cleanup.
5. Implement work and history commands plus deterministic Claude gate-hook handling.
6. Dogfood the project, document schemas and adapter decisions, and verify all paths.

## Risks

- Harness discovery surfaces can change; record exact current mappings.
- A declared gate can be mistaken for enforcement; report fidelity per gate and target.
- Generated files can collide with user files; preflight and refuse unowned/modified output.

