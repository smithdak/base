# base

Define an agentic system once—rules, agents, pipelines, gates, and knowledge—and compile it into
native surfaces for Claude Code, Codex, and GitHub Copilot. All definitions and run state are plain
files in git; the Rust CLI never runs the agent loop.

The walking skeleton is implemented. One `build` pipeline compiles to all three harnesses, generated
files are protected by a hash manifest, canon and gate fidelity are validated, and work/run state is
inspectable from the CLI.

## Install

Rust 1.85 or newer is required to build from source.

```console
cargo install --path .
base --help
```

## Quick start

Initialize global defaults once, then initialize and compile a project:

```console
base init --global
cd your-project
base init --project
base check
base sync
```

`base init` auto-detects scope: inside a git/base project it initializes the project; elsewhere it
initializes the global canon. `--global` and `--project` make the choice explicit. Set `BASE_HOME` to
override the default global location (`~/.base`).

Invoke the generated pipeline from the harness:

- Claude Code: `/build <task>`
- Codex: mention `$build` with the task
- GitHub Copilot: run `.github/prompts/build.prompt.md`

The pipeline creates `.base/runs/YYYY-MM-DD-<slug>/`, writes task/plan/result artifacts, stops for
explicit plan approval, and always appends an outcome to `.base/history.jsonl`.

## Commands

```text
base init [--global|--project] [--force]
base sync [--check] [--force]
base check
base work list [--status todo|doing|review|done] [--json]
base work new "Title" [--tags tag,tag] [--criterion "Text"]... [--json]
base work show W-0001 [--json]
base work move W-0001 todo|doing|review|done [--verdict pass|fail] [--json]
base work board [--json]
base log [RUN-SLUG]
```

Every public command accepts `--json`. Use `base sync --check` in CI: it validates canon and fails
when generated output is missing, stale, hand-edited, or no longer matches the manifest. Normal
`base sync` refuses to overwrite hand edits unless `--force` is explicit.

## Repository shape

```text
~/.base/canon/                  global definitions
<repo>/.base/
  base.toml                     targets, gates, generated-file hashes
  canon/                        project overlay (wins by canonical ID)
  work/                         work-item folders (W-0001-slug/item.md + attachments)
  runs/                         durable pipeline artifacts
  knowledge/                    project-tier lessons
  history.jsonl                 append-only run ledger
```

Generated target files are listed in `base.toml` and should never be hand-edited. Change canon and
run `base sync` instead.

## Development

```console
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Architecture and founding constraints are in [docs/SPEC.md](docs/SPEC.md), the canon schema is in
[docs/CANON.md](docs/CANON.md), current target mappings are in [docs/ADAPTERS.md](docs/ADAPTERS.md),
and decisions are recorded in [docs/DECISIONS.md](docs/DECISIONS.md).
