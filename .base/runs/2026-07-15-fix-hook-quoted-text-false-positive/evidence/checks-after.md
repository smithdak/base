case 1: replay of exact W-0002 compound - expect silence now
case 2: minimal quoted-text repro - expect silence now
case 3: real push controls - expect deny
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base standing denial: never push directly to `main`"}}
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base standing denial: never push directly to `main`"}}
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base standing denial: never push directly to `main`"}}
case 4: tests and drift
cargo: 19 unit + 13 integration + 1 doc passed; clippy -D warnings clean
sync check passed (13 generated files)
