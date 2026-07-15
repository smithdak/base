# Result: global canon backup automated

Implemented per the approved plan, no scope change: `~/.base/bin/backup-canon.sh` (POSIX sh,
idempotent init + snapshot-on-change) and `~/.base` now a local git repository.

## Acceptance checks

1. Script exists and is executable — created, `chmod +x` applied. **pass**
2. First execution initialized the repo and committed the full library (16 files, commit
   `a39f834`) — `evidence/run-1.md`. **pass**
3. Second execution reported "no changes", exit 0, no new commit — `evidence/run-2.md`. **pass**
4. `git -C ~/.base log` shows the snapshot commit — captured in `evidence/run-1.md`. **pass**

## Limitations

- History is local-only; no remote. Adding one (private GitHub repo) is the natural next step
  but a separate decision.
- Snapshots run on demand, not on schedule. If manual invocation proves unreliable, a scheduled
  task wrapping this same script is the follow-up — the mechanism was built to be wrapped.
- LF→CRLF autocrlf warnings on first add are cosmetic; content is committed unmodified.
