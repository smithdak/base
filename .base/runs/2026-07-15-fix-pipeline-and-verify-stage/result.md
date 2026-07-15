# Result: fix pipeline and verify stage

## Changed paths

- `.base/canon/pipelines/stages/diagnose.md` (new)
- `.base/canon/pipelines/stages/verify.md` (new)
- `.base/canon/pipelines/fix.md` (new)
- `.base/canon/pipelines/build.md` (verify inserted between execute and record)
- Regenerated surfaces: fix skills/prompts for claude, codex, copilot; updated build skills and
  instruction files; manifest restamped (13 generated files total)

## Acceptance checks (proof in `evidence/checks.md`)

1. **pass** — `base check`: canon valid, 6 stages, 2 pipelines, enforcement matrix unchanged.
2. **pass** — `base sync` wrote 9 files including all three fix surfaces; `base sync --check` clean.
3. **pass** — fresh clone of the feature branch passes `base sync --check` (see evidence tail).
4. **pass** — compiled build skill orders sections Intake, Plan, Execute, Verify, Record.
5. **pass** — this run's history line ships in the same commit as the change.

## Limitations

- The global canon at `~/.base` duplicates the default scaffold IDs and is not version-controlled;
  follow-up candidate, out of scope here.
- The fix pipeline has not yet been exercised on a real defect; its first live run is the real test.
