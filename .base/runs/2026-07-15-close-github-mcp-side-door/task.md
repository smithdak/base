# Task: close the GitHub MCP side door in the default-branch gate (W-0002)

## Outcome

The `never-push-default-branch` standing denial covers write-capable GitHub MCP tools on Claude,
not just Bash `git push`. A `mcp__github__*` tool call whose `branch` argument targets the default
branch is mechanically denied by the same deterministic hook that guards git pushes. Review-path
operations (feature-branch writes, PR creation, PR merges) stay permitted.

## Constraints

- Enforcement must inspect tool arguments; permission deny rules cannot (they would block the
  tools wholesale and violate the review path). Hook-only for the MCP surface.
- `base check` notes must state the new coverage honestly, including what remains open.
- Follow D-015's accepted residual: `merge_pull_request` landing content on the default branch is
  the intended review path, not a violation.

## Assumptions

- GitHub MCP direct-write tools carry the target branch in a `branch` string argument
  (`create_or_update_file`, `push_files`, `delete_file`); values may appear as `main` or
  `refs/heads/main`.
- Denying any `mcp__github__*` call with `branch` == default branch is acceptable overreach for
  tools where `branch` names a new ref (e.g. `create_branch` naming a branch `main` would fail
  server-side anyway).
- Hook config supports multiple PreToolUse matchers; MCP tool names match regex matchers.

## Acceptance checks (from W-0002, made testable)

1. Simulated hook event for `mcp__github__push_files` with `branch: "main"` → deny JSON with the
   standing-denial reason; same for `create_or_update_file` and `refs/heads/main` normalization.
2. Simulated events for `branch: "feature/x"` and for `merge_pull_request` (no branch argument) →
   no output (permitted).
3. Generated `.claude/settings.json` contains a PreToolUse entry matching `mcp__github__` tools
   wired to `base __hook claude-pre-tool`.
4. `base check` claude note mentions MCP branch-write coverage.
5. `cargo fmt`/`clippy -D warnings`/`cargo test` clean; `base sync` + `sync --check` clean;
   fresh-clone `sync --check` clean.
