# W-0015 — One-command onboarding with `base start`

## Outcome sought

`base start` takes an empty or existing repository from zero to a validated, synced
operating model in one command: ensure the global library, scaffold the project,
adopt the default pack, validate canon, and sync all three harness surfaces.

## Constraints

- Flags only; no interactive prompts (CI/agent friendly, `--json` throughout).
- Idempotent: rerun on an initialized project reports no-op stages.
- Never overwrites unowned files; failures carry next-step guidance.
- Spec tether, SPEC §7, and README updated in the same change.

## Acceptance checks

- [x] Empty directory → validated project with adopted pack and synced surfaces.
- [x] Rerun on an initialized project is an idempotent no-op.
- [x] `--json` emits a machine-readable stage report.
- [x] `--no-pack` skips adoption; unknown pack fails with available-pack listing.
- [x] Existing harness files are refused, not clobbered.
