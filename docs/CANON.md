# Canon contract

Canon is the vendor-neutral source of truth. It composes in this fixed order:

1. `$BASE_HOME/canon/` supplies personal seed definitions but never sole-source committed output.
2. Each ordered `[[packs]]` entry loads from `.base/packs/<id>/`; later packs win by canonical ID.
3. `.base/canon/` is the final project overlay and always wins.

Only pack and project definitions are repository-resident and renderable. `base check` reports
global-only definitions instead of silently committing machine-local bytes, and reports every
canonical ID override with its replaced and winning layer.

## Common document rules

Canonical IDs start with a lowercase letter and contain only lowercase letters, digits, and
hyphens. Except for skill resources and knowledge, definitions are Markdown with YAML frontmatter.

| Kind | Location | Required frontmatter | Optional frontmatter |
|---|---|---|---|
| rule | `rules/*.md` | `id` | `description` |
| agent | `agents/*.md` | `id`, `description` | `tools`, `skills`, `access` |
| skill | `skills/<id>/SKILL.md` | `id`, `description` | none |
| stage | `pipelines/stages/*.md` | `id` | `description` |
| pipeline | `pipelines/*.md` | `id`, `description`, `stages` | none |
| policy | `policies/*.md` | `id`, `description`, `event`, `mode`, `command` | `match-tools`, `timeout-seconds`, `fail-closed` |
| verifier | `verifiers/*.md` | `id`, `description`, `checks` | none |

Knowledge files under `knowledge/` are ordinary Markdown and do not require frontmatter. All
structured canonical frontmatter rejects unknown fields so misspellings fail validation.

## Agents and skills

An agent declares a portable role, referenced repository skills, and an access posture:

```yaml
---
id: dependency-auditor
description: Audits dependency changes without editing the repository.
access: read-only
skills:
  - evidence-review
tools:
  - Read
  - Grep
---
```

`access` is `inherit`, `read-only`, or `workspace-write`. Codex maps it to native `sandbox_mode`.
Claude maps `read-only` to `permissionMode: plan`; Copilot restricts a read-only agent to its native
`read` and `search` tools. Workspace-write remains a required posture on Claude and Copilot because
neither target supplies Codex-equivalent filesystem sandbox boundaries. `tools` is emitted as the
Claude allowlist and, using Copilot's compatible aliases, as its tool list when access is not
read-only. An agent may not reference a missing skill or a global-only skill from a
repository-resident definition.

A skill is a directory. `SKILL.md` carries canonical `id` and `description`; every other file below
the directory is a resource and is copied byte-for-byte to target skill folders. Skill and pipeline
IDs may not collide because they share target skill paths in Claude and the Codex/Copilot Agent
Skills directory.

## Pipelines

A pipeline is an ordered composition of authored stage prose:

```yaml
---
id: build
description: Plan, approve, implement, verify, and record a change.
stages:
  - use: intake
  - use: plan
    gate: plan-approval
  - use: verify
    verifier: project
    agent: auditor
    independent-review: true
  - use: record
---
```

Every pipeline has at least one stage and ends in `record`. `gate` must name a configured
`stage-approval` gate. `verifier` must name a repository-resident verifier suite; adapters compile
an explicit `base verify <suite> --run <run>` instruction at that stage. `agent` must name a
repository-resident agent. `independent-review: true` requires an assigned agent and tells native
runtimes to use a checker context that did not implement the change; an unavailable separation is
reported as assisted.

## Lifecycle policies

Policies bind a direct argv command to `session-start`, `pre-tool-use`, `post-tool-use`, or
`session-end`:

```yaml
---
id: session-context
description: Load current work and the durable handoff.
event: session-start
mode: context
command: [base, state, context]
timeout-seconds: 10
---
```

Modes have distinct contracts:

- `context`: valid only for `session-start`; stdout becomes target-specific `additionalContext`.
- `observe`: stdout is ignored by Base. The invoked command owns any durable observation record;
  Base only warns when the command fails.
- `guard`: valid only for `pre-tool-use`; stdout must be one JSON object with
  `{"decision":"allow|deny","reason":"..."}`. Base emits no explicit allow, preserving later
  permission layers. `fail-closed: true` converts launch, exit, timeout, and protocol failures into
  denial; otherwise failures warn and fall through.

Policy timeouts are 1–55 seconds, leaving time for Base to translate the declared failure posture
before the outer host timeout. Optional `match-tools` is a YAML sequence of portable full-name
globs, for example `[Edit, NotebookEdit, "mcp__github__*"]`. `*` and `?` are the only
metacharacters. Base escapes all remaining text and emits one anchored regex; raw host regex is not
canon. Base translates exact mutation aliases (`Edit`, `Write`, `NotebookEdit`, `apply_patch`) to
the target's native tool name before compiling and removes duplicates.

Claude Code, Codex, and Copilot receive native repository hook bindings for equivalent events.
Codex `session-end` stays assisted because its `Stop` event is turn-scoped rather than an equivalent
lifecycle point. Codex project hooks also require explicit trust of the generated definition.
Codex does not hook every hosted tool: a pre/post-tool policy is `native-hook` only when every
matcher is `Bash`, `apply_patch`, or an `mcp__*` name. Empty/broader matchers are `partial-hook` and
retain their prose fallback.

Copilot's current built-in GitHub MCP tools use sanitized names such as
`github-mcp-server-push_files`, so Base maps canonical `mcp__github__*` globs explicitly. Arbitrary
canonical `mcp__<server>__*` names are `partial-hook` for Copilot until the adapter has an explicit
server-name mapping; Base does not guess one.

## Verifiers

A verifier is a sequential list of direct argv checks:

```yaml
---
id: project
description: Project completion contract.
checks:
  - id: tests
    run: [cargo, test, --all-targets]
    timeout-seconds: 600
    retain-output: false
  - id: generated
    run: [base, sync, --check]
    cwd: tools/harness
---
```

`cwd` is project-relative. Timeouts are 1–3600 seconds. Zero exit is `pass`, non-zero exit is
`fail`, and launch/wait/timeout failure is `inconclusive`. Suite verdict precedence is fail, then
inconclusive, then pass. Checks run in a process group/Windows Job Object; Base terminates surviving
descendants on timeout or leader completion. Evidence always includes argv, cwd, exit code,
duration, byte counts, and SHA-256 for both streams. Raw stdout/stderr is omitted unless the check
sets `retain-output: true`, and retained output is capped at 1 MiB per stream with explicit
truncation state.

## Packs and configuration

A library pack lives at `$BASE_HOME/canon/packs/<id>/` and contains `pack.md` plus canonical kind
folders. Its manifest is:

```yaml
---
id: software-delivery
version: 1.2.0
description: Reusable evidence-led software delivery operating model.
---
```

`base adopt` copies the entire pack into `.base/packs/<id>/` and records its semantic version and
per-file SHA-256 hashes in ordered `[[packs]]` entries. Installed pack drift fails load. Upgrade
requires a clean installed copy and a strictly newer version; changed bytes under the same version
are rejected. Put all project changes in `.base/canon/` overrides. Packs are trusted repository
code: inspect policy and verifier commands before adoption or upgrade because generated lifecycle
hooks can execute them in a developer or cloud-agent environment.

`base init --global --packs-only --force` refreshes bundled pack bytes without touching personal
seed rules, agents, or stages. Full global `--force` remains a scaffold replacement operation, not
the routine pack-upgrade path.

Pack replacement and config persistence use drift preflight, a retained backup, unique staging,
and atomic config replacement. A multi-file pack upgrade is not a database transaction: a process
or machine crash between pack replacement and config replacement is detected as drift on the next
load and may require manual recovery from the backup.

The rest of `.base/base.toml` declares active `targets`, `default_branch`, and `[[gates]]`. The
`[generated]` table is owned by `base sync` and maps every generated project-relative path to its
canonical SHA-256 hash. UTF-8 generated content is hashed after CRLF-to-LF normalization; this makes
Windows and Unix Git checkout conversion equivalent without taking ownership of a repository's
root `.gitattributes`. Non-UTF-8 skill resources remain byte-exact. Schema `version = 2`
deliberately fails on v0.1 runtimes; top-level `requires-base`
then pins compatible v0.2+ CLI ranges. Generated paths must be
normal project-relative components; `sync` refuses symlink/reparse-point components before reads,
writes, or stale-output removal.

Authored relative paths use `/` separators on every platform. `default_branch` must satisfy Base's
conservative Git branch-name validation before it can be interpolated into any native surface.
Windows-reserved path components, invalid filename characters, trailing dots/spaces, and
case-colliding pack or skill-resource paths fail validation.

## Native migration overlays

Base owns generated target files, but existing project-specific target configuration can be kept
under `.base/native/` at one of six mirrored paths: `CLAUDE.md`, `AGENTS.md`,
`.github/copilot-instructions.md`, `.claude/settings.json`, `.codex/hooks.json`, or
`.github/hooks/base.json`. Markdown is appended after a visible source marker. JSON inputs must be
objects; recursive object keys compose, overlay arrays precede Base arrays, and Base wins scalar or
type conflicts. An overlay for a disabled target is invalid. Use canon rather than this escape hatch
for portable rules, agents, skills, pipelines, policies, or verifiers.
