# Upgrading a Base v0.1 project to v0.2

This procedure migrates an existing v0.1 repository to the v0.2 contract. It is for repositories
already running Base; new projects should follow the [Quickstart](../README.md#quickstart) instead.

> **Do this on a branch, with one operator.** Do not ask the v0.1 binary to migrate itself — the
> order below is load-bearing, and a concurrent second operator can lose config updates.

## Why the order matters

`requires-base` cannot protect a project from a v0.1 runtime until schema `version = 2` first forces
that old binary to stop. So the schema bump (step 2) must land before any content migration, and the
v0.2 binary must be invoked explicitly until it is confirmed on `PATH`.

## Procedure

1. **Install v0.2 and pin it.** Build or install v0.2 and invoke it by explicit path until
   `base --version` reports `0.2.0`.

2. **Bump the schema and fence the runtime.** Edit `.base/base.toml` to `version = 2` and add
   `requires-base = ">=0.2.0, <0.3.0"`. This fence makes v0.1 reject the project before it can
   mutate anything.

3. **Preserve target-owned files.** Move existing target-owned files that must survive into the
   allowlisted `.base/native/` mirror (for example, `CLAUDE.md` becomes `.base/native/CLAUDE.md`
   and `.claude/settings.json` becomes `.base/native/.claude/settings.json`). Promote portable
   content into `.base/canon/`. Add `.base/.gitignore` containing `.lock`.

4. **Refresh the library and re-adopt.** Refresh the bundled library with the explicit v0.2 binary
   using `base init --global --packs-only --force`, then run `base adopt software-delivery`.

5. **Validate and commit together.** Run `base check`, `base sync`, and `base sync --check`, then
   review and commit the config, pack, overlays, work reservations, and generated surfaces in one
   change.

## Native migration overlays

Base owns generated target files, but existing project-specific configuration can be kept under
`.base/native/` at one of six mirrored paths:

| Overlay path | Composes into |
|---|---|
| `.base/native/CLAUDE.md` | `CLAUDE.md` |
| `.base/native/AGENTS.md` | `AGENTS.md` |
| `.base/native/.github/copilot-instructions.md` | `.github/copilot-instructions.md` |
| `.base/native/.claude/settings.json` | `.claude/settings.json` |
| `.base/native/.codex/hooks.json` | `.codex/hooks.json` |
| `.base/native/.github/hooks/base.json` | `.github/hooks/base.json` |

Markdown supplements are appended to Base output after a visible source marker. JSON inputs must be
objects: recursive object keys compose, overlay arrays run **before** Base arrays, and Base wins
scalar/type conflicts so required hooks cannot be replaced. An overlay for a disabled target is
invalid. Agent or skill files that collide by generated ID must still be promoted into canon or
renamed. Prefer promoting portable rules into `.base/canon/`; the native escape hatch is for
target-specific configuration only.

See the [canon contract](CANON.md#native-migration-overlays) for the full merge semantics.
