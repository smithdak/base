# Second execution — idempotence

Command: `sh ~/.base/bin/backup-canon.sh` (immediately after run 1)

```
no changes: library already snapshotted
```

Exit code 0, no commit created — `git log` still shows exactly one snapshot commit.
