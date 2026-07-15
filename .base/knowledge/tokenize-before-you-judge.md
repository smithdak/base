# Tokenize before you judge

Lesson from run `2026-07-15-fix-hook-quoted-text-false-positive` (W-0003).

Enforcement heuristics that pattern-match raw command strings will false-positive on
self-describing agent work. Agent workflows routinely *quote the very commands the gate watches* —
commit messages about pushes, echo'd progress labels, printf'd JSON events — so a substring check
for `git` + `push` + branch name fires on prose ("…Bash git push; changes remain…" satisfies all
three; "remains" contains `main`). The gate blocked the commit that described it, then blocked the
diagnosis of its own bug.

Parse structure before judging content:

1. Space-pad separators (`;`, `|`, `&`) on the raw string — inside quoted regions this mutates
   only token *content*; quote structure survives.
2. Tokenize the whole command quote-aware (POSIX word splitting), then split into commands at
   separator *tokens*. A quoted message is one token and can never equal `git` or `push`.
3. Analyze each command's token sequence, not its characters.
4. Keep a fail-closed fallback for genuinely unparseable input, but match on **word boundaries**,
   never substrings — and require the specific target (the branch name as a word), not just the
   verb vocabulary.

Transfers to any deny-listing over shell commands, tool arguments, or log lines: the more an agent
narrates its work, the more its legitimate output resembles the forbidden action's syntax.
