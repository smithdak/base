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

## D-014: Repo-stamped outputs compile from repo-resident canon

- **Status:** accepted (2026-07-15)
- **Context:** The fix pipeline could have been authored in the freshly initialized global canon.
  Generated files are hash-stamped into the project manifest, so a surface sourced only from
  `~/.base` makes `sync --check` fail on every environment lacking that global canon (verified:
  a fresh clone with `BASE_HOME` pointed at a nonexistent directory).
- **Decision:** Definitions that shape a repo's generated surfaces live in that repo's
  `.base/canon/`. The global canon seeds new projects and holds personal defaults; it is never the
  sole source of a committed surface. Promotion global→project means copying into the repo.
- **Commits us to:** environment-independent drift checks (CI-safe, clone-safe), and accepting
  duplication between layers as the price of reproducibility.

## D-015: MCP servers are harness config, admitted by rule

- **Status:** accepted (2026-07-15)
- **Context:** MCP registration has no canon kind; each harness has a native surface (`.mcp.json`,
  `~/.codex/config.toml`, `.vscode/mcp.json`). Every active server costs context tokens each turn
  and widens the prompt-injection surface.
- **Decision:** MCP stays outside canon until cross-harness duplication demands an adapter cell.
  Admission rule: a server earns a slot only for capability a CLI cannot replicate more cheaply —
  discoverable capability such as schema introspection, semantic search, OAuth-gated APIs without a
  good CLI, or live interactive surfaces. Registrations default to read-only / least privilege;
  write access is granted deliberately per server, and the active set is re-audited periodically.
  Applied: the GitHub remote MCP server is registered in `.mcp.json` with full access granted
  explicitly (2026-07-15), accepted knowing write-capable MCP tools bypass the Bash-bound push
  gate; closing that side door is tracked as W-0002.
- **Commits us to:** weighing every proposed server against its CLI alternative, and treating an
  idle server as a cost, not a neutral.

## D-016: Descriptive docs are generated, tethered, or deleted

- **Status:** accepted (2026-07-15)
- **Context:** The canon → `sync` pipeline already makes generated surfaces drift-proof
  (`sync --check`), and DECISIONS.md is append-only so it cannot go stale. That leaves
  hand-written prose describing current code behavior as the one artifact class that rots
  silently — nothing breaks when it diverges. SPEC §7's CLI table was the first instance.
- **Decision:** Every doc claim about current behavior gets one of three treatments:
  **generate** it from the source of truth, **tether** it with a mechanical check that fails
  the build on drift, or **delete** it and let the intent doc stay silent. Intent docs
  (SPEC, DECISIONS) stay upstream of code: diverging from them is a decision, recorded here
  first. Applied: `tests/spec.rs` tethers SPEC §7 — verb set, `work` subcommand set,
  documented flags, the global `--json` promise, and the verb-count prose — to the clap
  definition; drift in either direction fails `cargo test`.
- **Commits us to:** never adding a hand-maintained behavior mirror without an attached
  check, and treating a failing tether as "update the doc or the code deliberately," never
  as a test to silence.

## D-017: Global knowledge is a library, adopted by copy

- **Status:** accepted (2026-07-15)
- **Context:** Rendering global-canon knowledge into instruction files put pointers in committed
  output whose sources live outside the repo; one promoted lesson made `sync --check` fail on
  every machine lacking that global canon (W-0004, reproduced). SPEC §5's "global knowledge
  reaches every project on next sync" and D-014 could not both hold via render-time merging.
- **Decision:** Committed surfaces render **repo-resident knowledge only**. The global canon is a
  personal library: it seeds new projects, and an existing project adopts a lesson by copying it
  into its own `.base/canon/knowledge/`. `base check` warns about global-only entries awaiting
  adoption — excluded honestly, never silently. Vendoring on sync (manifest-owned copies written
  by `base sync`) is the recorded upgrade path if multi-project use makes copy-adoption tedious.
- **Open residual (W-0005):** global-only rules, agents, and pipelines still render into
  committed output — the same hazard with a larger payload; they need the same treatment.
- **Commits us to:** nothing outside the repo is ever the sole source of committed bytes, and
  every exclusion the renderer makes is visible in `base check`.

## D-018: The whole global layer is seed-and-adopt

- **Status:** accepted (2026-07-15)
- **Context:** W-0005 reproduced D-017's hazard for the remaining canon kinds, with a larger
  payload: one global-only rule/agent/stage/pipeline made a single `sync` write seven files —
  full rule bodies into all three instruction files plus four brand-new generated files — all
  irreproducible without that global canon. Stages compound it: they inline into pipeline skills,
  so per-kind render filters alone would still let a project pipeline smuggle global stage bytes
  into committed output.
- **Decision:** D-017's rule covers every canon kind. Committed surfaces compile from
  repo-resident definitions only; the global layer seeds new projects and serves as a library
  adopted by copy. `base check` warns per excluded global-only definition (kind, id, adoption
  path). A project pipeline referencing a global-only stage is a **validation error**, not a
  warning, because the composition would commit foreign bytes. This narrows D-007's "sync compiles
  global + overlay" to seed-and-adopt semantics — said here explicitly rather than left ambiguous.
- **Commits us to:** render output being a pure function of the repo alone; the global canon
  never silently changing any committed surface; adoption always being a visible copy in the
  project's history.

## D-019: skillsmith stays separate; skills arrive as harness-level plugins

- **Status:** accepted (2026-07-15)
- **Context:** Dakota also maintains skillsmith (github.com/smithdak/skillsmith), a Claude Code
  skills monorepo — 14 skills in 4 installable plugins, with its own validate→generate→check
  pipeline, trigger-hit-rate evals, and security rules. The question was whether to fold that
  functionality into base and seed the system with those skills. Absorbing it would reverse
  SPEC §9 (plugin/marketplace runtime is a non-goal), split the toolchain (Rust vs. Bun/TS,
  against D-005), and pull a Claude Code-only artifact format into a vendor-neutral canon
  (against D-003/D-009).
- **Decision:** The repos stay separate with a defined division of labor. **base** is the
  per-project operating system: pipelines, gates, rules, work, runs, knowledge. **skillsmith**
  is the personal skill library and its distribution channel. Skillsmith capability reaches
  projects via user-level plugin install (`/plugin marketplace add smithdak/skillsmith`) —
  treated the way D-015 treats MCP servers: harness config outside canon, never committed
  bytes. Routing rule for new additions: a project's way of working → base canon; a portable
  personal technique → skillsmith. Know-how worth keeping is distilled into canon knowledge
  per D-017 (authored rewrite, not a copy of plugin bytes); a skill that becomes part of a
  standard gated delivery flow gets promoted the same way, as a base pipeline.
- **Commits us to:** base never growing plugin/marketplace machinery; skill installs staying
  user-scoped and out of committed surfaces; promotion into canon always being an authored
  rewrite so no committed byte's sole source is the skillsmith repo.

## D-020: Project-type packs live in the global library, adopted by copy

- **Status:** accepted (2026-07-15)
- **Context:** W-0007 recovered a planning-era idea: curated canon bundles for project types we
  build repeatedly, Sitecore first. The open questions were where pack sources live and how
  adoption works. Research run `2026-07-15-research-sitecore-pack-contents` weighed the options
  against D-001/D-002 (one user, lean core) and D-017/D-018 (seed-and-adopt, repo-pure surfaces).
- **Decision:** A pack is a folder in the global library — `~/.base/canon/packs/<pack-id>/` —
  mirroring the canon kind subfolders (`rules/`, `knowledge/`, `pipelines/`, `agents/`) plus a
  `pack.md` manifest carrying inventory, adoption instructions, and provenance. `packs/` sits
  outside the kind folders the compiler reads, so pack contents are invisible to `sync` until a
  project adopts them by copying files into its own `.base/canon/` — the same visible-copy
  semantics as D-018, grouped by project type instead of kind. Zero core changes. Pack content
  follows the D-019 boundary: project-resident ways of working only; portable personal technique
  stays in user-level skills. Applied: the Sitecore pack (one rules file, three knowledge files,
  deliberately no pipelines or agents) is the first instance, drafted via writing run
  `2026-07-15-write-sitecore-pack-draft`.
- **Commits us to:** packs never being a compiler concept until real multi-project use demands
  one; adoption always being a visible copy in the adopting repo's history. A `base adopt
  <pack>` helper is the recorded upgrade path when copy-adoption proves tedious.

## D-021: Stage-gate approval is an artifact, not an utterance

- **Status:** accepted (2026-07-15)
- **Context:** Stage approval was `assisted` on every target — compiled STOP prose the agent is
  trusted to obey — and the W-0006/W-0007 session showed the failure mode: a session directive
  stood in for plan approval with no record beyond prose the agent wrote about its own judgment
  call (W-0008). Consent was the one state the system still kept only in conversation.
- **Decision:** A stage-approval gate may declare `satisfied-by`, a run-folder-relative artifact
  path (declarations stay data — a path, a flag; no conditions, per D-008). Protocol: the agent
  writes `<satisfied-by>.request` describing what needs approval and stops; the human records
  the verdict from outside the session via `base approve <run> <gate> [--deny]`, which writes an
  immutable stamped record (who, when, verdict, note). The Claude adapter compiles hooks that
  deny all mutating tools (Bash, Edit/Write/NotebookEdit, GitHub MCP) while any request lacks
  its response — which also blocks the agent from self-approving or forging the record. Either
  verdict lifts the mechanical block; prose routes `denied` to `record aborted`. Standing
  directives satisfy a gate only as recorded approvals citing their source (`--note`). Matrix:
  plan-approval becomes `enforced` on Claude, stays `assisted` on Codex, `advisory` on Copilot.
- **Accepted trade-offs:** any pending request blocks the whole session's mutating tools, not
  just the gated run (one user, v1 — recorded, not solved); the gate scan fails open on IO
  errors so a filesystem oddity cannot brick a session (the push denial keeps fail-closed); in
  the unlikely config with artifact gates but no default-branch denial gate, the push check
  still rides along in the shared hook binary — over-enforcement accepted over new machinery.
- **Commits us to:** gate satisfaction never living only in conversation; approval records
  being immutable (a changed decision is a new run, not an edited file); the enforcement matrix
  reporting the upgrade honestly per target.
