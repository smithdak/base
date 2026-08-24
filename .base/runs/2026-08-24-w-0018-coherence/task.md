# W-0018 — Coherence pass for the onboarding and authoring flow

## Outcome sought

Documentation, CLI hints, and dogfood state agree with the new `base start` /
`base canon` surface so the out-of-box story is consistent end to end.

## Changes

- CANON.md native-overlay section documents `--migrate-native` instead of hand-migration only.
- `base init --project` next-step hints name pack adoption and one-command onboarding.
- Dogfood state: current-work re-pointed from merged W-0011 to live W-0017; W-0011 returned to
  `review` with verdict pending — the human pass/fail call is not the agent's to make.

## Acceptance checks

- [x] No stale verb counts remain anywhere in docs or source prose.
- [x] init hints point at adoption and `base start`.
- [x] current-work references an existing in-flight work item.
