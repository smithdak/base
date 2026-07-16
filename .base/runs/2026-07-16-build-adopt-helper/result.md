# Result: `base adopt <pack>` (W-0010)

Outcome: **completed**. All four task acceptance checks pass.

## Changed paths

- `src/cli.rs` — `Adopt(AdoptArgs)` subcommand (`pack` positional).
- `src/commands/mod.rs` — module + dispatch through `find_project_root`.
- `src/commands/adopt.rs` — new: pack resolution with available-pack listing, project
  requirement via `Config::load`, recursive `.md` collection excluding top-level `pack.md`,
  refuse-all-collisions-before-copy, follow-up checklist (INDEX routing lines per knowledge
  file, sync, commit together, pack.md pointer), `--json` report.
- `tests/cli.rs` — four new tests (copy + follow-ups, refusal without partial copy, unknown
  pack lists available, uninitialized project gets `base init` guidance).
- `docs/SPEC.md` §7 — verb table row + count prose for `adopt`.
- `.base/work/W-0010-base-adopt-pack-helper/item.md` — criterion 3 amended at intake to record
  Dakota's 2026-07-16 build directive superseding D-020's tedium trigger.

## Deviation from plan

Plan said "no docs changes"; `tests/spec.rs` (the D-016 tether) failed twice — first on the
missing verb row, then on the stale "six verbs" prose — so SPEC §7 was updated deliberately,
citing D-020/W-0010. The tether working exactly as designed is itself evidence.

## Verification

| Check | Verdict | Evidence |
|---|---|---|
| `cargo test`: 45 tests pass (23 unit, 21 CLI incl. 4 new, 1 spec tether) | pass | `evidence/verification.txt` |
| Live e2e: scratch project adopts the real sitecore pack — 4 files copied, second adopt refused, `sync` + `check` green | pass | `evidence/live-adopt.txt` |
| `base sync --check` stays green in this repo (no compiler surface changed) | pass | `evidence/verification.txt` |
| `cargo fmt --check` clean | pass | `evidence/verification.txt` |

## Limitations

- The installed `base` binary predates this change; reinstall (`cargo install --path .`) after
  merge so the new verb reaches PATH — the compiled gate hooks keep working either way.
- The global `pack.md` adoption instructions still describe the manual copy; they stay true,
  but pointing them at `base adopt` is a sensible authored follow-up in the global library.
