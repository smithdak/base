# Base operating-model core — architecture spec

**One sentence:** define a repository operating model once—rules, agents, skills, pipelines,
policies, verifiers, knowledge, and state—and compile it to Claude Code, Codex, and GitHub Copilot
without making Base an agent orchestrator.

Founding and amending decisions live in `DECISIONS.md`; this document specifies the shipped v0.2
contract.

## 1. Principles

1. **Git is the substrate.** Definitions, adopted packs, work, runs, evidence, and handoffs are
   plain files: diffable, cloneable, and harness-legible.
2. **One canon, native adapters.** Target output is compiled, never independently authored. Native
   target asymmetries are reported rather than hidden behind a least-common-denominator claim.
3. **Base owns lifecycle, not inference.** The CLI composes, validates, renders, gates, verifies,
   and records. Claude Code, Codex, or Copilot owns the model loop.
4. **Evidence has types.** `fail` and `inconclusive` are distinct non-passing outcomes. A local pass
   cannot prove unavailable infrastructure or an external release gate.
5. **Reusable core, explicit project edge.** Versioned packs hold shared operating models;
   `.base/canon/` holds project-specific overrides and executable verification contracts.

## 2. Composition and flow

```text
$BASE_HOME/canon/          .base/packs/<id>/          .base/canon/
personal seed library  ->  ordered vendored packs  -> project overlay (wins)
                                     |
                                     | base check / base sync
                                     v
              Claude Code          Codex             Copilot
              CLAUDE.md             AGENTS.md         .github/*
              .claude/*             .codex/*          .agents/skills/*
                                     |
                                     v
              .base/{state,work,runs,evidence,history,knowledge}
```

Global-only definitions are library inputs, not reproducible project inputs, and never render.
Adopted packs are immutable repository inputs recorded by semantic version and SHA-256. Packs load
in config order; later IDs win. The project overlay wins last.

The v0.2 contract requires `.base/base.toml` schema `version = 2`. Older v0.1 binaries reject it
before mutation instead of silently dropping `requires-base`, pack records, or generated surfaces.

## 3. Canon

Canon kinds are rules, agents, skills, pipeline stages, pipelines, lifecycle policies, verifier
suites, and knowledge. Markdown bodies carry authored reasoning; YAML frontmatter carries bounded
structure. Base interprets references and direct commands but does not implement pipeline branching
or a workflow language.

Skills retain their full directory tree. Agents reference skills and declare an access posture.
Pipeline stages may reference a stage-approval gate, verifier suite, assigned agent, and an
independent-review requirement. Every pipeline ends with `record`, preserving one ledger entry for
completed, aborted, and failed exits.

Policies have four lifecycle events (`session-start`, `pre-tool-use`, `post-tool-use`,
`session-end`) and three modes (`context`, `guard`, `observe`). Verifiers are sequential direct-argv
checks with bounded timeouts and `pass | fail | inconclusive` outcomes. Exact schemas are in
`CANON.md`.

## 4. Adapters

Adapters are pure functions of repository-resident canon plus config. `sync` computes all outputs,
compares them with the prior manifest, refuses hand-edited collisions unless `--force`, removes
obsolete generated files, writes the new set, and stamps hashes in `.base/base.toml`. Owned output
paths must be normal repository-relative components; symlink/reparse-point traversal is refused.

An allowlisted `.base/native/` mirror composes existing target-specific instruction and JSON
configuration into `CLAUDE.md`, `AGENTS.md`, `.github/copilot-instructions.md`,
`.claude/settings.json`, `.codex/hooks.json`, and `.github/hooks/base.json`. Markdown is appended;
JSON is recursively merged with native arrays before Base arrays and Base winning scalar/type
conflicts. The committed overlay is input; the target file remains manifest-owned output.

Non-hook CLI commands take a shared or exclusive advisory file lock at `.base/.lock`; config,
pack, generated-surface, work, approval, and state mutations are exclusive. Reads/verifiers are
shared. Acquisition is bounded at 30 seconds, and `.base/.gitignore` excludes the lock file.
Global bundled-pack refresh uses `$BASE_HOME/.lock`; adoption holds the project lock and a shared
global-library lock. This prevents same-working-copy Base processes from losing config updates.

Claude Code, Codex, and Copilot receive native custom agents. Claude gets project skills under
`.claude/skills`; Codex and Copilot share open Agent Skills under `.agents/skills`. All three receive
native lifecycle hooks for equivalent events. Codex `session-end` remains assisted, and Codex
project hooks require explicit trust. Copilot pipelines have separate CLI/cloud Agent Skill and VS
Code prompt-file profiles. The current matrix and verified-as-of references are in `ADAPTERS.md`.

## 5. State and evidence

- `work/W-NNNN-slug/item.md` (`W-0001` through `W-9999`): fixed
  `todo | doing | review | done` workflow. `done` requires a
  human `pass | fail` verdict; checked criteria are evidence, not an inferred verdict. New items
  atomically create `work/.ids/W-NNNN`; committing that reservation forces a Git integration
  conflict for independently allocated duplicate IDs. Duplicate metadata IDs and reservation
  mismatches fail `base check`.
- `state/current-work`: pointer to an existing work item. `state/handoff.md`: optional UTF-8 handoff
  with `work-item: W-NNNN` and `run: <slug>` frontmatter matching that pointer and an existing run,
  `# Handoff`, and a non-empty `## Next action`. Switching work rejects a stale handoff; clearing
  state removes both files.
- `runs/<slug>/`: one auditable attempt. `evidence/verifications/*.json`: verifier reports with
  stream hashes/counts by default and raw output only after per-check opt-in.
- `history.jsonl`: append-only one-line run summaries.
- `knowledge/`: project lessons; canon knowledge supplies durable operating guidance.

## 6. Gates and failure posture

Stage approval uses an operator-intended verdict artifact written with `base approve`. The command
requires the exact request, validates one-line fields, and uses create-new reservation so concurrent
deciders get one CLI-level winner. Hooks deny ordinary agent writes to configured response paths
and deny covered mutation while a request lacks a response. Approval resumes mechanically; denial
is terminal for that plan, but routing it to `record aborted` remains behavioral, so fidelity is
`hybrid-hook`. The scanner accepts a well-formed artifact; `by` is self-asserted, and an unrestricted
same-account process can bypass hooks or forge/change bytes. This is a trusted-working-copy
workflow control, not authenticated human identity, cryptographic authorization, or filesystem
immutability.

The built-in standing denial guards common direct default-branch writes through native pre-tool
hooks, with Codex rules as defense in depth. It recognizes ordinary/forced refspecs, `HEAD`/`@` on
the default branch, and GitHub MCP branch writes—including Copilot's sanitized
`github-mcp-server-*` tool namespace. Aliases, wrappers, hook disablement, and other host escape
paths remain possible; authoritative protection belongs at the Git server.

Canonical guard policies may opt into fail-closed behavior. Generated Codex/Copilot wrappers probe
Base hook protocol 1: built-in and fail-closed guards emit a native deny when Base is absent or
incompatible, while context/observe/fail-open policies skip. This makes compatible Base bootstrap a
mandatory prerequisite for mutation in those profiles. `base check` reports the runtime, profile,
scope, and trust prerequisites it can inspect; product-level timeouts can still be fail-open.
Policy commands and verifiers run in process groups/Windows Job Objects; timeouts and leader
completion terminate descendants while pipe handling stays deadline-bounded.

## 7. The CLI

Rust, single binary, eleven verbs. Every verb supports `--json`; mutations touch Base-owned files,
managed pack bytes, or manifest-listed generated output.

| Verb | Job |
|---|---|
| `base init [--global] [--project] [--packs-only] [--force]` | scaffold the global library/project, or refresh only bundled packs |
| `base sync [--check] [--force]` | compile canon to active targets; stamp or verify generated hashes |
| `base check` | validate composition and report gate plus definition-surface fidelity |
| `base adopt <pack> [--upgrade]` | vendor or safely upgrade an immutable versioned pack |
| `base ingest <path> [--run]` | inverse-read another system's harness surfaces into a portable inventory and canon mapping/fidelity report |
| `base pack <new\|check>` | scaffold a library pack skeleton or validate a drafted pack before adoption |
| `base work <list\|new\|show\|move\|board>` | manage folder-backed work items and the kanban board |
| `base log [<slug>]` | inspect run history or one run folder |
| `base approve [--deny] [--by] [--note]` | write a create-new operator stage-gate verdict |
| `base verify <suite> [--run]` | execute typed verifier checks and optionally retain evidence |
| `base state <show\|set\|clear\|context>` | manage current work and emit portable session context |

`tests/spec.rs` tethers the visible verb set, documented flags, nested verb alternations, global
`--json` promise, and prose count to the clap definition.

## 8. v0.2 definition of done

1. A bundled versioned `software-delivery` pack can be adopted, drift-checked, and safely upgraded.
2. Rules, agents, skills plus resources, pipelines, policies, verifiers, and knowledge compose with
   deterministic pack/project precedence.
3. All three targets receive current native agent, skill, pipeline, and equivalent lifecycle-hook
   surfaces; product profile, scope, trust/runtime prerequisites, and degradation are explicit.
4. Current work and a validated work/run handoff can restore context in a new human or agent session.
5. Verifier evidence retains direct commands and typed outcomes; absence of execution never passes.
6. Generated UTF-8 drift is invariant to Git CRLF checkout conversion; non-UTF-8 resources remain
   byte-exact.
7. Generated drift, source formatting, lint, tests, and the spec tether pass in Base itself.

## 9. Non-goals

Model routing or headless agent dispatch · hosted coordination · accounts/auth · centralized policy
service · external tracker synchronization · secrets distribution · telemetry aggregation ·
marketplace/registry protocol · interpreting arbitrary workflow expressions.

## 10. Residual decisions

1. Pack publication remains a filesystem/git distribution concern; a signed remote registry needs a
   demonstrated team bottleneck and separate threat model.
2. Remote harness-image bootstrap is outside Base v0.2. Teams must install a `requires-base`-
   compatible binary in any Claude, Codex, or Copilot environment expected to execute hooks.
3. Adapter surfaces are volatile. Each mapping carries an as-of date and must be re-verified before
   a target contract changes.
4. The shipped proof is local Windows. Path validation rejects known Windows-incompatible names and
   case-colliding pack/skill-resource paths, but Windows+Linux CI remains required before claiming
   full cross-platform runtime proof.
