# Result: global knowledge is a library, adopted by copy (W-0004)

## Changed paths

- `src/render.rs` — instruction files render only `Layer::Project` knowledge; global-only entries
  never enter committed output
- `src/commands/check.rs` — warning per global-only knowledge entry with the adoption instruction
- `docs/SPEC.md` — §5 knowledge bullet reframed (global canon = personal library; adoption = copy);
  DoD 7 now reads "adopted by copy into another project, and visible there after sync"
- `docs/DECISIONS.md` — D-017 records the decision, the vendoring upgrade path, and the W-0005
  residual
- `tests/cli.rs` — global-only knowledge: output byte-identical with/without global canon, no
  pointer leaks, check warning asserted
- New work item W-0005 — same hazard for global-only rules/agents/pipelines (larger payload)

## Acceptance checks (proof in `evidence/repro-before.md` and `evidence/checks-after.md`)

1. **pass** — D-017 records how global knowledge reaches projects (seed new; adopt-by-copy for
   existing) without environment-dependent output.
2. **pass** — the diagnosis scenario re-run: sync with a global-only lesson writes nothing,
   commits no pointer, and fresh-machine `sync --check` exits 0 (was exit 1 before the fix).
3. **pass** — DoD-7 loop achievable: capture (done, W-0003 lesson) → promote to global library →
   adopt by one file copy in the target project → its next `sync` renders the pointer. Spec text
   and mechanics now agree; full demonstration awaits a second real project.
4. **pass** — clippy `-D warnings`, 19 unit + 14 integration + 1 spec-tether tests, `sync` (0
   written — this repo's output never depended on global entries) and `sync --check` clean.

## Limitations

- W-0005 remains open: a global-only rule, agent, or pipeline still renders into committed output.
- The end-to-end promotion demonstration needs a second project; mechanics are in place.
