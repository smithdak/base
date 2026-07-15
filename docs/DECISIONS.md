# Decision Log — base

Founding decisions, recorded 2026-07-14. ADR-style: context, decision, what it commits us
to. Newest entries at the bottom.

---

## D-001: Built for one user first

- **Status:** accepted (2026-07-14)
- **Context:** A system designed for a hypothetical team front-loads governance,
  onboarding, and ecosystem work before anyone uses it daily.
- **Decision:** v1's user is Dakota. Team features (shared parts, connectors, onboarding
  kits, cross-team governance) are explicitly out of scope until the personal system is
  daily-useful.
- **Commits us to:** optimizing for one person's real workflow over hypothetical adopters.

## D-002: Neutral core, dev pipelines first

- **Status:** accepted (2026-07-14)
- **Context:** The system could be a dev tool (assume repos, builds, diffs) or a general
  work OS (pipelines over any work). A dev-only core makes generalizing later a rework; a
  general-first build delays daily usefulness.
- **Decision:** The core's concepts — work, pipelines, runs, gates, knowledge — are
  domain-neutral and never assume code. The first shipped pipeline family is software
  delivery, because that is the daily use.
- **Commits us to:** keeping dev-specific vocabulary out of core types; new pipeline
  families (research, writing, automation) must require zero core changes.

## D-003: Multi-harness via canonical definitions + adapters

- **Status:** accepted (2026-07-14)
- **Context:** The system must work across Claude Code, Codex, and Copilot — not be
  configuration for any single vendor's harness.
- **Decision:** All agents, pipelines, rules, and knowledge are authored once in a
  vendor-neutral **canon**. Per-harness **adapters** compile the canon to each harness's
  native surfaces (`.claude/` + `CLAUDE.md`; `AGENTS.md` + Codex prompts; Copilot
  instructions + prompt files). You work *inside* whichever harness you opened; they all
  see the same brain. An orchestrator-of-harnesses (headless routing/dispatch) is a
  possible later layer, not v1.
- **Commits us to:** never hand-editing compiled output; adapter fidelity being explicit
  (see SPEC — enforcement matrix); the canon format being the real product.

## D-004: Plain files in git are the substrate

- **Status:** accepted (2026-07-14)
- **Context:** State could live in files, SQLite, or a hosted service.
- **Decision:** All state — work items, run artifacts, history, knowledge, config — is
  plain files (Markdown/JSON/TOML) in git. No database, no server, no auth. Files are
  diffable, agent-legible from every harness, and cost zero infrastructure.
- **Commits us to:** every feature being expressible as file reads/writes; anything that
  needs a query engine waits until files demonstrably fail.

## D-005: Rust for the CLI and tooling

- **Status:** accepted (2026-07-14)
- **Decision:** The `base` CLI is Rust — single static binary, cross-platform, fast, no
  runtime dependency on Node/Python.
- **Commits us to:** hand-rolling some agent-ecosystem plumbing that TypeScript gets free;
  accepting slower iteration in exchange for a durable, dependency-free tool.

## D-006: Harness-resident interaction; the CLI is plumbing

- **Status:** accepted (2026-07-14)
- **Decision:** Day-to-day work happens inside Claude Code / Codex / Copilot via compiled
  commands and context. The CLI compiles (`sync`), validates (`check`), and manages state
  (`work`, `log`) — touched occasionally, like `git config`. The CLI never runs the agent
  loop.
- **Commits us to:** the harness being the engine; CLI scope staying lifecycle-only.

## D-007: Global install + per-project overlay

- **Status:** accepted (2026-07-14)
- **Decision:** One home-directory install (`~/.base/`) holds the global canon; each
  project carries a thin `.base/` overlay for its specifics plus its own state. `base sync`
  compiles global + overlay into the project's harness surfaces.
- **Commits us to:** a defined merge order (overlay wins), and drift protection via a
  manifest of generated-file hashes.

## D-008: v1 core concepts — gates, runs + history, knowledge loop

- **Status:** accepted (2026-07-14)
- **Decision:** Four concepts make up the v1 core:
  - **Human approval gates** — declared checkpoints where the system must stop for the
    human, enforced by each harness's strongest native mechanism.
  - **Artifact runs + history** — every pipeline run gets a durable artifact folder plus an
    append-only `history.jsonl` ledger.
  - **Knowledge/learnings loop** — deliberate capture and promotion of lessons into canon.
  - **Pipelines declared as data, compiled to prose** — stage sequences are declarations;
    stage bodies are authored prose; the compiler composes, never interprets.
- **⚠ Deferred, not rejected:** **evidence-gated verification** ("done requires captured
  proof, not an agent's say-so") is not in v1 core. The `runs/` artifact shape reserves an
  `evidence/` subfolder so it can be added later without a breaking change (SPEC §5, §10).
- **Commits us to:** these four being the whole core until real use demands more.

## D-009: All three adapters ship in v1

- **Status:** accepted (2026-07-14)
- **Decision:** v1 ships Claude Code, Codex, and Copilot adapters. Claude Code is the
  reference (richest surface); Codex and Copilot prove the canon isn't secretly shaped
  around any one harness.
- **Commits us to:** designing the canon against three targets from day one, with honest
  per-target fidelity rather than least-common-denominator features.

## D-010: Lean spec, then walking skeleton

- **Status:** accepted (2026-07-14)
- **Decision:** First deliverable is a short architecture spec (`docs/SPEC.md`), then
  immediately a walking skeleton: one pipeline defined once in canon, compiled to at least
  two harnesses, run end-to-end. The spec stays honest because code follows within days.
- **Commits us to:** not speccing beyond what the skeleton will exercise.

## D-011: Compile pipelines to current repo-scoped skill surfaces

- **Status:** accepted (2026-07-14)
- **Context:** The original spec named Claude commands and Codex custom prompts. Claude Code has
  unified custom commands with skills, while Codex custom prompts are deprecated and user-scoped;
  Codex now discovers repository skills under `.agents/skills/`.
- **Decision:** Compile pipelines to `.claude/skills/<id>/SKILL.md` for Claude Code and
  `.agents/skills/<id>/SKILL.md` for Codex. Keep Copilot prompt files under `.github/prompts/`.
- **Commits us to:** tracking harness discovery changes explicitly and preferring durable,
  repository-scoped native surfaces over preserving outdated path names.

## D-012: Walking-skeleton schema and honest gate fidelity

- **Status:** accepted (2026-07-14)
- **Context:** The skeleton needed an exact frontmatter schema and a per-cell answer for which gates
  are mechanically enforced.
- **Decision:** Canonical IDs are lowercase kebab-case; pipelines contain an ordered `stages` list;
  each stage reference has `use` and optional `gate`; every pipeline ends in `record`. Stage approval
  is assisted on Claude and Codex and advisory on Copilot. The default-branch standing denial is
  enforced on Claude, assisted on Codex, and advisory on Copilot.
- **Commits us to:** failing validation on missing stage/gate references, never overstating a prose
  checkpoint as enforcement, and keeping the matrix generated from the same config as adapters.

## D-013: Kanban work items use explicit human verdicts

- **Status:** accepted (2026-07-15)
- **Decision:** Work items live in folders and move through the fixed statuses `todo`, `doing`,
  `review`, and `done`. Every item has acceptance criteria plus an explicit verdict; moving to
  `done` requires a human-selected `pass` or `fail`. Checked criteria are evidence, not the source
  of that verdict.
- **Commits us to:** stable work-item folder paths, a CLI-only four-column board, and preserving the
  distinction between checklist completion and the human outcome call.
