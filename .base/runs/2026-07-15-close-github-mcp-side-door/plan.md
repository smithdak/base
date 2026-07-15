# Plan: close the GitHub MCP side door (W-0002)

## Changes

**`src/commands/hook.rs`** — extend `claude_pre_tool` to handle two event shapes:
- existing: `tool_input.command` (Bash) → `pushes_default_branch` check, unchanged;
- new: when `tool_name` starts with `mcp__github__`, read `tool_input.branch`; normalize by
  stripping a `refs/heads/` prefix; if it equals the default branch, emit the same
  `permissionDecision: "deny"` JSON with reason "base standing denial: never write to `<branch>`
  via GitHub MCP; push a feature branch and open a review instead".
- Tools without a `branch` argument (e.g. `merge_pull_request`) and feature-branch values pass
  silently. Unit tests for: `push_files`/`create_or_update_file` + `main` denied;
  `refs/heads/main` denied; `feature/x` permitted; `merge_pull_request` (no branch) permitted;
  Bash shape unaffected.

**`src/render.rs`** — `render_claude` writes a second PreToolUse entry in `.claude/settings.json`:
matcher `mcp__github__.*` (regex over tool names), same `base __hook claude-pre-tool
--default-branch <branch>` command, no `if` clause (the matcher already scopes it; the hook is a
deterministic ~10ms exec that exits silently on non-default-branch input).

**`src/commands/check.rs`** — claude fidelity note updated to name both surfaces: Bash git pushes
and GitHub MCP branch writes are hook-checked; PR merges remain the review path.

**`docs/ADAPTERS.md`** — standing-denials cell and fidelity bullet updated to match.

**`tests/cli.rs`** — assert generated settings contains the `mcp__github__` matcher.

## Verification

1. `cargo fmt` && `cargo clippy --all-targets -- -D warnings` && `cargo test`
2. `base sync` (settings.json rewrite) && `base sync --check` && `base check` (note text)
3. Simulated hook events piped to the rebuilt binary → captured in `evidence/checks.md`
4. Fresh clone passes `sync --check`
5. Reinstall rebuilt release binary to `~/.cargo/bin` so the live hook gains the behavior

## Risks / limitations

- The GitHub MCP server is not loaded in this session (approval happens next restart), so the MCP
  path is verified by simulated stdin events — the identical mechanism was live-fire verified for
  Bash on 2026-07-15. A live MCP write test can follow once the server is approved.
- Generic `branch == default` matching slightly overreaches (`create_branch` naming a new branch
  `main` is denied); accepted — that call fails server-side anyway.
- `merge_pull_request` still lands content on the default branch by design (D-015 residual).

## Delivery

Branch `feat/w-0002-mcp-gate`; commit code + regenerated settings + run artifacts + history line;
PR to `main`.
