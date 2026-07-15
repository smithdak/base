# Verification evidence

Passed on Windows on 2026-07-14:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
  12 unit tests passed
  6 integration tests passed
cargo build --release
target/release/base --directory . check --json
target/release/base --directory . sync --check --json
  10 generated files unchanged; no drift
codex execpolicy check --rules .codex/rules/base.rules -- git push origin main
  decision: forbidden
claude --settings .claude/settings.json --version
  2.1.191
```

