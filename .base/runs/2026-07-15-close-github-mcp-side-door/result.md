# Result: close the GitHub MCP side door (W-0002)

## Changed paths

- `src/commands/hook.rs` — `denial_reason` now handles two event shapes: Bash `git push` commands
  (unchanged logic) and `mcp__github__*` tools, denied when `tool_input.branch` (with
  `refs/heads/` normalized) equals the default branch; five unit tests cover deny/permit paths
- `src/render.rs` — generated `.claude/settings.json` gains a second PreToolUse entry with matcher
  `mcp__github__.*` wired to the same hook command
- `src/commands/check.rs` — claude fidelity note names both hook surfaces and the review-path
  exception
- `docs/ADAPTERS.md` — standing-denials cell and fidelity bullet updated
- `tests/cli.rs` — asserts both matchers in generated settings
- `.claude/settings.json` — regenerated (manifest restamped)

## Acceptance checks (proof in `evidence/checks.md`)

1. **pass** — `push_files`/`create_or_update_file` targeting `main` (and `refs/heads/main`) →
   deny JSON with the standing-denial reason
2. **pass** — `feature/x` writes and `merge_pull_request` → silent (permitted); Bash
   `git push origin main` regression → still denied
3. **pass** — generated settings has PreToolUse matchers `['Bash', 'mcp__github__.*']`
4. **pass** — `base check` claude note covers the MCP surface and the review-path exception
5. **pass** — fmt/clippy `-D warnings`/16+13 tests clean; `sync` + `sync --check` clean;
   fresh-clone check appended to evidence

## Limitations

- MCP path verified by simulated stdin events; the GitHub MCP server loads after the next session
  restart, so no live-fire MCP denial yet (Bash path was live-fire verified 2026-07-15).
- `merge_pull_request` still lands content on `main` by design (D-015 residual, review path).
- Rebuilt release binary reinstalled to `~/.cargo/bin`, so the live hook has the new behavior.
