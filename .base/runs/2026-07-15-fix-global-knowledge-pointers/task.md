# Task: reconcile global knowledge pointers with reproducible output (W-0004)

## Outcome

Generated instruction files are reproducible from the repo alone even when the global canon holds
knowledge entries the project lacks, and the SPEC's knowledge-promotion loop (DoD 7) is achievable
without reintroducing environment-dependent output. The governing choice is recorded as a decision
(W-0004 criterion 1).

## Constraints

- D-014 is the boundary: committed surfaces must compile from repo-resident sources.
- Fail honestly: if global-only definitions are excluded from output, `base check` must say so,
  not silently drop them.
- The fix pipeline requires reproduction before planning; if the drift cannot be reproduced,
  stop and report.

## Assumptions

- The hazard generalizes beyond knowledge (a global-only rule/agent/pipeline also renders into
  committed files); W-0004's criteria scope this run to knowledge, with the general case filed as
  follow-up work rather than fixed here.

## Acceptance checks (from W-0004)

1. A decision entry records how global canon knowledge reaches projects without
   environment-dependent generated files.
2. Fresh-clone `sync --check` stays green while the active global canon holds a knowledge entry
   this project lacks (reproduced failing first, then passing).
3. The DoD-7 promotion loop is achievable under the recorded decision (spec text and mechanics
   agree; demonstrated or precisely described).
4. `cargo fmt` / `clippy -D warnings` / full test suite / `base sync --check` clean.
