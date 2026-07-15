# Task: hook false-positives on quoted text mentioning git push (W-0003)

## Outcome

`base __hook claude-pre-tool` stops denying Bash commands whose *quoted text* (commit messages,
echoed strings) mentions `git push` or contains `main`-like substrings, while still denying every
real default-branch push — including implicit pushes and no-space separators.

## Constraints

- Fail-closed posture is preserved: genuinely unparseable input still errs on the deny side, just
  with word-boundary matching instead of raw substring containment.
- No behavior change for the MCP event shape (W-0002, under review in PR #3).

## Assumptions

- The defect reproduces against the currently installed binary by replaying the exact compound
  command the hook denied during the W-0002 run.

## Acceptance checks

1. Replayed W-0002 compound (commit message mentioning "git push", `;`, and "remain") → permitted.
2. Minimal quoted-text cases (`git commit -m "x; git push remains"`) → permitted.
3. Real denials regress-tested: `git push origin main`, `git push`, `HEAD:main`,
   `npm test && git push origin main`, and no-space `cd x&&git push origin main` → denied.
4. `cargo fmt` / `clippy -D warnings` / full test suite clean; `sync --check` clean.
