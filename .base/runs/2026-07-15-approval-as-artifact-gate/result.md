# Result: stage-gate approval is now an artifact

Implemented per the approved plan. One deviation from the plan's letter, none from its intent:
the default config now ships `plan-approval` with `satisfied-by` (new projects get enforcement
out of the box), which surfaced two integration tests asserting the old assisted-everywhere
world — updated to assert the new semantics deliberately, not silenced.

## Changed

- `src/config.rs` — `Gate.satisfied_by` (+ path safety validation; Windows `has_root`/drive
  guard), `approval_path()`/`request_path()` derivation, default plan-approval declares its
  artifact.
- `src/cli.rs`, `src/commands/approve.rs`, `src/commands/mod.rs` — sixth verb `base approve
  <run> <gate> [--deny] [--by] [--note]`, immutable stamped records, `--json`.
- `src/commands/hook.rs` — pending-request scan (fail-open on IO; standing denial untouched
  and fail-closed), deny reason names run, gate, file, and both clearing commands.
- `src/render.rs` — `enforced` cell for artifact gates on Claude; artifact-protocol gate prose
  (request file → stop → `base approve` outside the session → denied routes to record aborted;
  forging the verdict artifact is forbidden by prose *and* blocked by the hook); settings.json
  gains `Edit|Write|NotebookEdit` matcher and unconditional Bash matcher when artifact gates
  exist; permission deny rules stay tied to the standing denial.
- `src/commands/check.rs` — claude note describes artifact enforcement; PATH downgrade warning
  now covers both enforced cells.
- `.base/base.toml`, `docs/SPEC.md` §7 (six verbs + approve row, tether green),
  `docs/DECISIONS.md` D-021.
- 14 regenerated surface files (gate prose ripple across build/fix/writing/automation × 3
  targets, settings.json, instruction files).

## Verification (task.md acceptance checks)

1. D-021 recorded. **pass**
2. `approve` writes stamped record; `--deny` first-class; duplicate refused — unit +
   integration tests and `evidence/hook-loop.md` §3–6. **pass**
3. Hook denies Bash/Edit while pending, allows after either verdict; matrix reads
   enforced/assisted/advisory for plan-approval — `evidence/hook-loop.md` §1–4 and
   `base check` output. **pass**
4. Compiled gate prose demands recorded standing approvals (`--note` citing the source);
   this run's own conversational approval was recorded retroactively at
   `approvals/plan-approval.md`. **pass**
5. `cargo fmt` / `clippy -D warnings` / 41 tests including the SPEC tether / `base sync` /
   `sync --check` all green. **pass**

## Limitations

- Whole-session block on any pending request (recorded in D-021).
- Enforcement requires the installed `base` on PATH to be ≥ this build; `base check` warns
  when it cannot resolve, but cannot detect a stale binary version. Follow-up candidate if it
  bites.
