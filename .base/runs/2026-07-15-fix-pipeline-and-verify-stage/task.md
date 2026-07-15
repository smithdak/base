# Task: add a fix pipeline and evidence-backed verify stage

## Outcome

The canon gains its second pipeline family: `fix` (diagnose → repair → prove), built from two new
reusable stages, `diagnose` and `verify`. The `build` pipeline adopts the `verify` stage so its
behavior matches its own description ("Plan, approve, implement, verify, and record"). All three
harness surfaces compile and pass drift checks.

## Constraints

- Author in the **project** canon (`.base/canon/`), not the global canon: generated outputs are
  hash-stamped in this repo's manifest, so their sources must be reachable from the project alone
  or `sync --check` becomes environment-dependent (fails on any clone lacking the same global
  canon). The freshly initialized global canon seeds future projects instead.
- No Rust changes; canon and generated surfaces only.
- Commit on a feature branch and open a PR — the default branch is gated.

## Assumptions

- The `verify` stage writes proof under the reserved `runs/<slug>/evidence/` folder (D-008 left
  this door open deliberately).
- Stage-approval gating reuses the existing `plan-approval` gate; no new gates.

## Acceptance checks

1. `base check` reports 6 stages and 2 pipelines, canon valid, enforcement matrix unchanged.
2. `base sync` emits fix surfaces for all three targets (`.claude/skills/fix/SKILL.md`,
   `.agents/skills/fix/SKILL.md`, `.github/prompts/fix.prompt.md`) and `base sync --check` passes.
3. A fresh `git clone` of the branch passes `base sync --check` (environment independence holds).
4. The compiled build skill contains a numbered Verify section between Execute and Record.
5. One history line for this run is appended to `.base/history.jsonl` and ships with the PR.
