case 1: replay of exact W-0002 compound - expect wrongful deny
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base standing denial: never push directly to `main`"}}
case 2: minimal quoted-text repro - expect wrongful deny
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base standing denial: never push directly to `main`"}}
case 3: control feature-branch event - expect silence
