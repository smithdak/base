# Result: hook quoted-text false positive fixed (W-0003)

## Changed paths

- `src/commands/hook.rs` — `pushes_default_branch` rewritten: separators are space-padded, then
  the whole command is tokenized quote-aware (`shell_words`) and split into commands at separator
  tokens, so quoted text stays a single token and never matches `git`/`push`. The unparseable-input
  fallback narrows from substring containment to word-boundary matching (fail-closed preserved).
  Per-command analysis extracted to `command_pushes_default_branch`, logic unchanged. Six new
  regression assertions across three tests.

## Acceptance checks (proof in `evidence/repro-before.md` and `evidence/checks-after.md`)

1. **pass** — exact W-0002 compound: denied before, silent after.
2. **pass** — minimal quoted-text case and echo-label case: denied before (unit-tested), silent
   after; live-fire: this run's own commit message contains the poison pattern and was permitted
   by the installed hook.
3. **pass** — real denials intact: `git push origin main`, implicit `git push`, `HEAD:main`,
   `npm test && …`, and no-space `cd x&&git push origin main` all still deny; unbalanced real push
   `git push origin main "oops` still fails closed.
4. **pass** — clippy `-D warnings` clean (after collapsing the replace chain), 19 unit + 13
   integration + 1 doc test, `sync --check` untouched (no generated output in scope).

## Limitations

- `&` is now treated as a command separator (it is one in shell); unquoted `&` in exotic arguments
  splits analysis — strictly widens coverage, never narrows.
- Fallback permits unparseable prose lacking the branch as a word; that prose was the bug. A real
  push always tokenizes or names the branch as a word.
