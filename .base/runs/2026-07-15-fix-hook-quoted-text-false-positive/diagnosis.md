# Diagnosis: hook false-positives on quoted text (W-0003)

## Reproduction (evidence/repro-before.md)

1. Exact replay of the W-0002 commit+push compound → wrongful deny.
2. Minimal case `git commit -m "x; git push remains"` → wrongful deny. Nothing in that command
   pushes anywhere; the message is pure text.
3. Live meta-reproduction: while capturing case 2, the *echo label quoting it* tripped the live
   hook and blocked the diagnosis command itself.

## Root cause — `pushes_default_branch`, src/commands/hook.rs:64

Two defects compound:

1. **Quote-blind splitting.** The command is split on `;` / `|` / `&&` / `||` over the raw string
   before any tokenization, so a separator *inside a quoted string* (the `;` in a commit message)
   cuts the quoted region in half, producing fragments with unbalanced quotes.
2. **Substring fallback.** An unbalanced fragment fails `shell_words::split`, and the fail-closed
   fallback tests raw substring containment: `contains("git") && contains("push") &&
   contains("main")`. Prose satisfies all three — "Bash git push" supplies the first two and
   "remains" contains `main`.

## Affected surface

Any Bash compound in which quoted text combines a separator character, the words git/push, and a
`main` substring — which is precisely what agent workflows emit: commit messages about pushes,
echo'd progress labels, printf'd JSON. The gate becomes unusable for self-describing work.

## Nearby risks for the fix

- The raw-string separator replacement is what currently catches no-space forms
  (`cd x&&git push origin main`); a quote-aware rewrite must keep them caught.
- The fail-closed posture for genuinely unparseable input must survive — narrowed to
  word-boundary matching, not deleted.
