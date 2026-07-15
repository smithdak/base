# Task: automate version-controlled backup of the global canon (automation)

## Outcome

`~/.base` — the personal seed-and-adopt library, which as of today includes the Sitecore pack —
is under git version control, with a repeatable one-command mechanism that snapshots any changes
as a commit. Manual procedure being replaced: remembering to hand-init/hand-commit the library
(currently done never — the library has zero history and no recovery path).

## Constraints

- The mechanism must be idempotent: safe to run when nothing changed (reports, exits 0, commits
  nothing) and safe on first run (initializes the repository).
- Touches only `~/.base`; nothing in any project repo changes.
- Plain files and git only (D-004 posture) — no services, no scheduler dependency; scheduling
  can be layered on later precisely because the mechanism is a single script.

## Assumptions

- Git Bash is available on this machine (it runs this harness's shell), so a POSIX sh script is
  the portable choice.
- A local-only git history is acceptable v1; a remote is a later decision.

## Acceptance checks

1. `~/.base/bin/backup-canon.sh` exists and is executable.
2. First real execution initializes the repo (if needed) and commits the current library state;
   captured output shows the commit.
3. Second execution reports no changes and exits 0; captured output shows it.
4. `git -C ~/.base log` shows the snapshot commit(s).
