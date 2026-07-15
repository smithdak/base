$ push_files branch=main
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base standing denial: never write to `main` via GitHub MCP; push a feature branch and open a review instead"}}
$ create_or_update_file branch=refs/heads/main
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base standing denial: never write to `main` via GitHub MCP; push a feature branch and open a review instead"}}
$ push_files branch=feature/x (expect silence)
$ merge_pull_request (expect silence)
$ Bash git push origin main (regression)
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base standing denial: never push directly to `main`"}}

$ settings.json PreToolUse matchers
['Bash', 'mcp__github__.*']

$ base check (claude note)
claude: standing denial uses a project permission deny plus PreToolUse hooks over Bash git pushes and GitHub MCP branch writes (PR merges stay the review path); stage approval is prompt-assisted

$ cargo test / clippy / sync
cargo test: 16 unit + 13 integration passed; clippy -D warnings clean (run 2026-07-15)
sync check passed (13 generated files)

$ fresh clone of feat/w-0002-mcp-gate, BASE_HOME=<nonexistent>, base sync --check
sync check passed (13 generated files)
