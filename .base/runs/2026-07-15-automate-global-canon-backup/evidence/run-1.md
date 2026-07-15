# First execution — init + snapshot

Command: `sh ~/.base/bin/backup-canon.sh`

```
Initialized empty Git repository in C:/Users/dakot/.base/.git/
initialized git repository in /c/Users/dakot/.base
[16 LF→CRLF working-copy warnings from core.autocrlf — cosmetic, content committed as-is]
committed snapshot: a39f834 canon snapshot 2026-07-15T21:02:21Z
```

Command: `git -C ~/.base log --oneline` / `git -C ~/.base ls-files`

```
a39f834 canon snapshot 2026-07-15T21:02:21Z

bin/backup-canon.sh
canon/agents/builder.md
canon/agents/reviewer.md
canon/knowledge/INDEX.md
canon/knowledge/adapter-surfaces.md
canon/packs/sitecore/knowledge/sitecore-content-modeling.md
canon/packs/sitecore/knowledge/sitecore-governance-operability.md
canon/packs/sitecore/knowledge/sitecore-platform-decisions.md
canon/packs/sitecore/pack.md
canon/packs/sitecore/rules/sitecore-conventions.md
canon/pipelines/build.md
canon/pipelines/stages/execute.md
canon/pipelines/stages/intake.md
canon/pipelines/stages/plan.md
canon/pipelines/stages/record.md
canon/rules/working-agreements.md
```

16 files snapshotted, including the Sitecore pack delivered earlier today.
