# Plan: `base adopt <pack>`

## Files to change

1. **`src/cli.rs`** — add `Adopt(AdoptArgs)` to `Command` (doc comment: "Copy a global-library
   pack into the project canon.") and:
   ```rust
   pub struct AdoptArgs {
       /// Pack ID (folder name under BASE_HOME/canon/packs/).
       pub pack: String,
   }
   ```
2. **`src/commands/mod.rs`** — `mod adopt;` plus dispatch
   `Command::Adopt(args) => adopt::run(&find_project_root(&start)?, args, cli.json)`.
3. **`src/commands/adopt.rs`** (new) — the whole behavior:
   - Resolve `base_home()?/canon/packs/<pack>`. If absent, bail listing available pack IDs
     from `canon/packs/` (or "no packs installed" when the folder is missing/empty).
   - `Config::load(project_root)` up front — its existing error ("run `base init`") covers
     un-based projects, matching pack.md's adoption step 0.
   - Walk the pack folder recursively; collect every `.md` file except top-level `pack.md`,
     keyed by its pack-relative path (`rules/x.md`, `knowledge/y.md`, nested paths allowed —
     mirrors canon layout by construction).
   - Empty pack (no files after exclusions): bail — nothing to adopt is an error, not a no-op.
   - **Refuse-before-copy** (init.rs idiom): if any destination
     `.base/canon/<relative>` exists, bail listing every conflict; write nothing.
   - Copy all files (create parent dirs), then report: copied paths, plus follow-ups the
     helper cannot do — one INDEX.md routing line per copied `knowledge/` file, `base sync`,
     commit copies and regenerated surfaces together, and pack.md for adoption notes.
   - `--json`: `AdoptReport { pack, root, copied: Vec<String>, follow_ups: Vec<String> }`.
4. **`tests/cli.rs`** — four tests using a fake pack written into the temp `BASE_HOME`
   (`canon/packs/testpack/{rules,knowledge}/*.md` + `pack.md`):
   - `adopt_copies_pack_into_project_canon` — files land, `pack.md` not copied, stdout names
     INDEX/sync/commit follow-ups.
   - `adopt_refuses_existing_files` — second run fails listing the conflict; a second pack
     file that would have been new is not written (no partial copy).
   - `adopt_unknown_pack_lists_available` — bad ID errors and names `testpack`.
   - `adopt_requires_project` — empty dir (no `.base/base.toml`, with `.git` so root
     discovery succeeds) errors with `base init` guidance.

## Not changing

- `sync`, `check`, templates, canon loader — packs stay invisible to the compiler (D-020).
- `README.md`/docs — CLI help is the behavior doc; CLAUDE.md is generated and untouched.
- The global `pack.md` adoption instructions keep the manual steps (they remain true; the
  helper automates steps 1–2 of them). Optional post-run follow-up, not this change.

## Verification (acceptance checks in task.md)

- `cargo test` — new tests plus the existing 20 stay green.
- Live end-to-end in a temp dir against the real library: `base init --project`,
  `base adopt sitecore`, `base sync`, `base check` → `evidence/live-adopt.txt`.
- `base sync --check` in this repo → green line in `evidence/verification.txt`.
- `cargo fmt --check`.

## Risks

- Recursive copy trusts pack layout to mirror canon kinds; a malformed pack (stray folder)
  copies harmlessly but `base check` in the adopting project will flag invalid canon — that is
  the existing validation surface doing its job, so no new schema validation here (lean core).
- Path separators: build pack-relative keys with `/` (as templates.rs does) and join with
  `MAIN_SEPARATOR_STR` on write, so output and JSON stay stable across platforms.
