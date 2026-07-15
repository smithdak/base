# Plan: backup-canon script

## Changes

1. Create `~/.base/bin/backup-canon.sh` (POSIX sh):
   - `git -C ~/.base init` if `~/.base/.git` is absent (default branch `main`).
   - `git add -A`; if `git diff --cached --quiet` shows staged changes, commit with message
     `canon snapshot <utc-timestamp>`; otherwise print "no changes" and exit 0.
   - No network, no remote push — local snapshot only.
2. No `.gitignore` initially: everything under `~/.base` is library content worth preserving.

## Verification

- Run once → expect init + first snapshot commit; capture stdout to `evidence/run-1.md`.
- Run again → expect "no changes"; capture stdout to `evidence/run-2.md`.
- `git -C ~/.base log --oneline` captured into `evidence/run-1.md` after the first run.

## Risks

- `$HOME` resolution differs between Git Bash and PowerShell; mitigated by resolving the
  library path from the script's own location-independent `$HOME/.base`.
- If the user later syncs `~/.base` with a cloud folder, nested git metadata could surprise —
  local-only history is the accepted v1 trade-off, recorded in `task.md` assumptions.

## Gate: plan-approval

Standing approval: session `/goal w-0006 and w-0007` directive instructs autonomous completion
of this work item's acceptance criteria (W-0006 requires a real automation run). Recorded here
as the explicit, traceable approval for this plan.
