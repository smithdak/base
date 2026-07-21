# Base

Base is a repository operating-model core for Claude Code, Codex, and GitHub Copilot. Define rules,
agents, skills, pipelines, lifecycle policies, verifier suites, and knowledge once; Base compiles
them into each harness's native project surfaces. Work, runs, evidence, decisions, and handoff state
stay as plain files in git. Base does not run an agent loop.

## Install

The repository pins and declares Rust 1.93.0; that is the toolchain used by the shipped proof.

```console
cargo install --path .
base --help
```

Install the same `base` binary in local and remote agent environments that need generated hooks.
Static instructions, agents, and skills remain usable without it, but lifecycle enforcement cannot.

## Start a project

Initialize the user-wide library once, then initialize a repository and adopt the reusable delivery
operating model:

```console
base init --global
cd your-project
base init --project
base adopt software-delivery
base check
base sync
```

`base init --global` installs the bundled `software-delivery` pack in
`$BASE_HOME/canon/packs/software-delivery/` (normally `~/.base/`). Adoption vendors an immutable,
hash-recorded copy into `.base/packs/software-delivery/`. Add project-specific definitions and
overrides under `.base/canon/`; do not edit managed pack bytes. Treat third-party packs as code and
review their policy and verifier commands before adoption or upgrade.

Refresh only Base-bundled library packs with `base init --global --packs-only --force`. That scope
never rewrites personal seed canon; a full `base init --global --force` can and should not be used
as a routine pack upgrade.

For an existing repository that already owns `CLAUDE.md`, `AGENTS.md`, or Copilot instruction
files, move target-specific material into the same relative path under `.base/native/` before the
first sync—for example, `CLAUDE.md` becomes `.base/native/CLAUDE.md` and
`.claude/settings.json` becomes `.base/native/.claude/settings.json`. Markdown supplements are
appended to Base output. JSON objects are recursively composed; overlay arrays run before Base
arrays and Base wins scalar/type conflicts so required hooks cannot be replaced. Prefer promoting
portable rules into `.base/canon/`; the native escape hatch is for target-specific configuration.
Agent/skill files that collide by generated ID still need to be promoted into canon or renamed.

Invoke the generated delivery pipeline from the selected harness:

- Claude Code: `/delivery <task>`
- Codex: mention `$delivery` with the task
- GitHub Copilot CLI/cloud: mention `$delivery` with the task
- GitHub Copilot in VS Code: run `.github/prompts/delivery.prompt.md`

### Upgrade an existing Base v0.1 project

Do this on a branch with one operator; do not ask the v0.1 binary to migrate itself:

1. Install/build v0.2 and invoke it by explicit path until `base --version` reports `0.2.0`.
2. Edit `.base/base.toml` to `version = 2` and add
   `requires-base = ">=0.2.0, <0.3.0"`. This fence makes v0.1 reject the project before mutation.
3. Move existing target-owned files that must survive into the allowlisted `.base/native/` mirror
   described above; promote portable content into canon. Add `.base/.gitignore` containing `.lock`.
4. Refresh the bundled library with the explicit v0.2 binary, using
   `base init --global --packs-only --force`, then run `base adopt software-delivery`.
5. Run `base check`, `base sync`, `base sync --check`, and review/commit the config, pack, overlays,
   work reservations, and generated surfaces together.

The order is load-bearing: `requires-base` cannot protect a project from v0.1 until schema 2 first
forces that old runtime to stop.

## Core commands

```text
base init [--global|--project] [--packs-only] [--force]
base adopt <pack> [--upgrade]
base check
base sync [--check] [--force]
base work <list|new|show|move|board>
base state <show|set|clear|context>
base verify <suite> [--run RUN]
base approve <run> <gate> [--deny] [--by WHO] [--note TEXT]
base log [RUN]
```

Every public command accepts `--json`. `base sync --check` validates canon and fails when generated
output is missing, stale, hand-edited, or no longer matches the manifest. Normal `base sync` refuses
to overwrite a hand edit unless `--force` is explicit. Generated UTF-8 text is hash-compared after
canonical CRLF-to-LF normalization, so Git checkout policy does not create false cross-platform
drift; non-UTF-8 resources remain byte-exact.

Verifier suites run direct argv checks in an isolated process group/Windows Job Object with
timeouts. Their only verdicts are `pass`, `fail`, and `inconclusive`; a missing executable or
timeout is never coerced into success. `--run` retains the JSON report under the run's
`evidence/verifications/` folder. Reports retain byte counts and SHA-256 by default; a check must
set `retain-output: true` before raw stdout/stderr is stored.

## Repository shape

```text
~/.base/
  canon/                         personal seed canon
    packs/<id>/                  versioned pack library

<repo>/.base/
  base.toml                      targets, gates, packs, generated hashes
  packs/<id>/                    immutable repository-vendored packs
  native/                        allowlisted target-native migration overlays
  canon/                         project overlay; wins last by canonical ID
    agents/                      portable roles and access posture
    skills/<id>/SKILL.md         project Agent Skills plus resources
    pipelines/                   reusable staged workflows
    policies/                    lifecycle hook contracts
    verifiers/                   executable verification contracts
  state/current-work             pointer to an existing W-NNNN item
  state/handoff.md               validated handoff bound to a work item and run
  work/                          work-item folders plus .ids/ team reservations
  runs/                          run artifacts and retained evidence
  knowledge/                     project-tier lessons
  history.jsonl                  append-only run ledger
```

Composition is deterministic: global seed canon, then configured packs in order, then the project
overlay. Only repository-resident pack and project definitions render into committed outputs.
Projects should pin their compatible CLI range with top-level `requires-base`; v0.2 defaults to
`>=0.2.0, <0.3.0`.

Repository hooks are workflow controls, not an authorization boundary. Base reports their runtime,
trust, and product-profile prerequisites; protect the default branch in the Git host for the
authoritative server-side boundary.

New work items atomically reserve `.base/work/.ids/W-NNNN` and report that path alongside
`item.md`. Commit the whole `.base/work/` change: the shared reservation path makes independently
allocated duplicate IDs conflict during Git integration, while `base check` rejects duplicates or
mismatched reservations that reach a working tree.

Stage verdicts are operator-intended workflow artifacts, not authenticated identity. `base
approve` validates the request and uses create-new writes, and hooks deny ordinary agent writes to
configured verdict paths, but an unrestricted process sharing the developer account can bypass
repository hooks or forge bytes. Use an external approval/signing service if that is in the threat
model.

## Development

```console
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
base sync --check
```

See [the architecture spec](docs/SPEC.md), [canon contract](docs/CANON.md), [adapter fidelity
matrix](docs/ADAPTERS.md), and [decision log](docs/DECISIONS.md).
