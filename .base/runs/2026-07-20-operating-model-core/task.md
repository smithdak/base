# Promote Base into the cross-harness operating-model core

## Outcome

Evolve Base from its personal walking skeleton into a team-usable, vendor-neutral development
operating model. Preserve Base as the compiler and state substrate; generalize the proven rezidnt
harness disciplines as reusable canon and a standard software-delivery pack.

## Constraints

- Keep Claude Code, Codex, and GitHub Copilot as the execution engines; Base does not run agent loops.
- Generated surfaces remain reproducible from repository-resident inputs and protected by hashes.
- Adapter fidelity is reported per feature and target; unsupported behavior never masquerades as native.
- Existing schema-1 projects have an explicit, ordered manual migration; schema 2 must make v0.1
  binaries fail before mutation rather than silently discard v0.2 state.
- Pack upgrades must refuse local edits and mutable same-version releases.

## Acceptance checks

- W-0011 acceptance criteria are satisfied.
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets`
- `cargo build --release`
- `cargo run -- check --json`
- `cargo run -- sync --check --json`
