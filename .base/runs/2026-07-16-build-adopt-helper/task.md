# Task: Build the `base adopt <pack>` helper (W-0010)

Date: 2026-07-16 · Pipeline: build · Harness: claude · Work item: W-0010

## Outcome

`base adopt <pack>` copies a global-library pack (`~/.base/canon/packs/<pack>/`) into the
current project's `.base/canon/`, refusing to overwrite any existing file, and prints the
manual follow-ups it cannot do: INDEX.md routing lines for copied knowledge files, `base sync`,
and committing copies plus regenerated surfaces together. Adoption stays a visible copy
(D-018/D-020); zero compiler concepts are added — `sync` never learns packs exist.

## Constraints

- Declarations stay data; the helper is a copy convenience only (D-020 guardrails in W-0010).
- Match existing CLI idioms: clap subcommand in `src/cli.rs`, dispatch through
  `src/commands/mod.rs` with `find_project_root`, `--json` via `print_json`, refuse-all-
  collisions-before-writing like `init` so a failed adopt never leaves a partial copy.
- Tests follow the `tests/cli.rs` harness (temp project + temp `BASE_HOME`).

## Assumptions

- Provenance note: W-0009's friction evidence found the copy low-tedium on n=1; Dakota
  directed this build on 2026-07-16 anyway (criterion 3 amended accordingly). D-020 already
  records the helper as the upgrade path, so no new decision entry is needed.
- The pack manifest (`pack.md`) is documentation, not canon content — never copied.
- The one real pack today is `sitecore`; a live end-to-end against it is part of verification.

## Acceptance checks

1. `cargo test` passes including new CLI tests: successful adopt (files land under
   `.base/canon/`, follow-ups printed), conflict refusal (second adopt fails, no partial
   writes), unknown pack (error names available packs), adopt outside a base project fails
   with guidance.
2. Live end-to-end: temp project + real `~/.base` library — `base adopt sitecore` copies the
   four files; `base sync && base check` then succeed; output captured as evidence.
3. `base sync --check` in this repo stays green (no compiler surface changed).
4. `cargo fmt --check` clean.
