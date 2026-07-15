# Result: the whole global layer is seed-and-adopt (W-0005)

## Changed paths

- `src/render.rs` — every render surface layer-filters to project-resident definitions: rules,
  agents, and pipelines in instruction files; per-agent files; per-pipeline skills/prompts for all
  three targets (`project_agents` / `project_pipelines` helpers)
- `src/canon.rs` — validation error when a project pipeline references a global-only stage
  (stages inline into skills; a warning would still commit foreign bytes)
- `src/commands/check.rs` — one warning per excluded global-only definition: kind, id, adoption
  path; stages worded as "not usable by project pipelines"
- `docs/DECISIONS.md` — D-018 records the extension and the explicit narrowing of D-007 to
  seed-and-adopt semantics
- `docs/SPEC.md` §2 — generated output is a pure function of the repo alone
- `tests/cli.rs` — two new tests: byte-identical output under a hostile global canon with all four
  warning kinds asserted; the cross-layer stage reference fails validation

## Acceptance checks (proof in `evidence/repro-before.md` and `evidence/checks-after.md`)

1. **pass** — D-018 recorded, including the stage-reference rule.
2. **pass** — diagnosis scenario re-run: sync against the global-only canon writes 0 files
   (was 7); fresh-machine `sync --check` exits 0 (was 1 with 3 drifts + 4 stale entries).
3. **pass** — `check` emits 4 exclusion warnings (rule/agent/stage/pipeline, each with adoption
   path); live `shipwreck` pipeline referencing global-only `deploy` fails validation with the
   copy-to-adopt message.
4. **pass** — clippy `-D warnings`, 19 unit + 16 integration + 1 spec-tether tests; this repo:
   sync writes 0, `sync --check` green.

## Limitations

- A repo whose only pipelines live in the global canon renders no skills (legible via check
  warnings); accepted for v1's single-user reality, revisit with vendoring if it bites.
- D-007's original render-time merge promise is now formally narrowed; D-018 owns that.
