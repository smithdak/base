# Plan: fix hook quoted-text false positive (W-0003)

Scope: `src/commands/hook.rs` only — `pushes_default_branch` and its tests. No render/settings
change, so no generated-file or manifest changes.

## Rewrite `pushes_default_branch`

1. **Normalize separators first, spaced:** replace each of `;`, `|`, `&` with ` ; ` on the raw
   string. Single-character replacements have no ordering hazards (`&&` → ` ;  ; `, harmless empty
   command). Inside quoted regions this only mutates the text *content* — quote structure stays
   balanced, so the quoted message remains one token.
2. **Tokenize the whole command** with `shell_words::split` (quote-aware):
   - **Ok(tokens):** split the token stream at `;` tokens into commands; run the existing
     per-command analysis (git → push → refspec check, implicit-push fail-closed) on each. A
     commit message is now a single token that never equals `git` or `push`, killing the false
     positive; `cd x&&git push origin main` still splits and denies.
   - **Err (genuinely unbalanced):** fail-closed fallback, narrowed from substring to
     **word-boundary** matching: words are runs of `[alphanumeric _ - /]`; deny only if `git` and
     `push` appear as words and some word equals the default branch or ends with `/{branch}`.
     "remains" and "never-push-default-branch" no longer match; a real unparseable
     `git push origin main "oops` still denies.

## Tests (all in hook.rs unit tests)

- W-0003 regressions: minimal repro permitted; compact W-0002-style compound (heredoc message
  with `git push;` + "remain") permitted; echo-label case permitted.
- Still denied: all four existing deny cases, plus `cd x&&git push origin main` and unbalanced
  `git push origin main "oops`.
- Unbalanced prose without a branch word → permitted (documents the narrowed fail-closed).

## Verification

1. `cargo fmt` && `cargo clippy --all-targets -- -D warnings` && `cargo test`
2. Rebuild release, reinstall to `~/.cargo/bin`, replay all three diagnosis events → case 1 and 2
   silent, real-push controls denied (captured to `evidence/checks-after.md`)
3. `base sync --check` (expect untouched — no generated output in scope)

## Risks

- Treating `&` as a separator is *more* correct (it is one in shell) but changes analysis for rare
  unquoted `&` in arguments; acceptable, and strictly widens denial coverage, never narrows.
- The narrowed fallback permits unparseable prose that the old code denied; that prose was the bug.
  Real pushes always tokenize or contain the branch as a word.

## Delivery

Branch `fix/w-0003-hook-quoted-text` **stacked on `feat/w-0002-mcp-gate`** (same file under review
in PR #3; GitHub retargets the PR to `main` when #3 merges). Commit code + run artifacts + history
line; PR with base `feat/w-0002-mcp-gate`.
