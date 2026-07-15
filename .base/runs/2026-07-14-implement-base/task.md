# Task

Scaffold and implement `base` in the repository from the architecture spec.

## Constraints

- Rust single-binary CLI; the harness remains the agent engine.
- Plain files in git for definitions and state.
- Global canon plus project overlay.
- Claude Code, Codex, and GitHub Copilot targets.
- Honest gate fidelity and protected generated output.

## Acceptance checks

- All five public v1 verbs work and support JSON.
- The `build` pipeline compiles to all three targets.
- Hand-edited output is detected and protected.
- Global/project overlay precedence and state inspection are tested.
- Formatting, lint, tests, release build, canon validation, and sync drift checks pass.

