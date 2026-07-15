# End-to-end hook loop (new binary, synthetic run `2026-07-15-hook-demo`)

Request file written: `.base/runs/2026-07-15-hook-demo/approvals/plan-approval.md.request`

## 1–2. Pending request denies mutating tools (Bash and Edit events)

```
$ echo '{"tool_name":"Bash","tool_input":{"command":"echo hi"}}' | base __hook claude-pre-tool --default-branch main
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"base stage gate: 2026-07-15-hook-demo plan-approval awaits a human verdict (.base/runs/2026-07-15-hook-demo/approvals/plan-approval.md.request has no response). Record one from your own terminal: `base approve 2026-07-15-hook-demo plan-approval` or `base approve 2026-07-15-hook-demo plan-approval --deny`. Mutating tools stay denied until the verdict artifact exists."}}
```

Identical denial for `{"tool_name":"Edit","tool_input":{"file_path":"src/lib.rs"}}`.

## 3. Verdict recorded

```
$ base approve 2026-07-15-hook-demo plan-approval --note "evidence loop for W-0008"
recorded approved for gate `plan-approval` on run 2026-07-15-hook-demo at .base/runs/2026-07-15-hook-demo/approvals/plan-approval.md
```

## 4. Same Bash event after the verdict → no denial emitted (allow)

## 5. Immutability

```
$ base approve 2026-07-15-hook-demo plan-approval
error: approval record already exists at .base/runs/2026-07-15-hook-demo/approvals/plan-approval.md; verdicts are immutable — a changed decision belongs in a new run
```

## 6. Stamped artifact

```
# Gate decision: approved

- gate: plan-approval
- run: 2026-07-15-hook-demo
- verdict: approved
- by: Dakota Smith
- at: 2026-07-15T22:14:20Z
- note: evidence loop for W-0008
```

Synthetic run folder deleted after capture (a stale request would hold future sessions).
