# Plan: merge PR #23

## Files / components touched

No source changes. Repository state operations only:

1. `gh pr merge 23 --merge --delete-branch` — merge commit matching repo convention (#20–#22),
   delete the remote and local head branch `feat/w-0018-coherence`.
2. Delete remaining redundant stack branches (`feat/w-0015-base-start`,
   `feat/w-0016-native-migration`, `feat/w-0017-canon-authoring`) — their commits are contained in
   the merged head, so nothing is lost.
3. `git checkout main && git pull` locally.
4. Verification on updated `main`: `cargo fmt --all -- --check`, `cargo clippy --all-targets
   --all-features -- -D warnings`, `cargo test --all-targets --all-features`,
   `base verify base --run 2026-08-24-merge-onboarding-stack`.
5. `result.md` + ledger entry appended to `.base/history.jsonl` (outcome `completed`).

## Risks

- Branch protection or a race could make the merge fail → report honestly, no retry loops.
- Deleting stacked branches is safe only after MERGED is confirmed; sequence enforces that.

## Out of scope

Work-item verdicts (human), release tagging, README/announcement updates.
