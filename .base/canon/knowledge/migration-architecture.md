# Designing a better system from an ingested one

How to turn another agent system into a base pack that is **better than the source**, not a copy of
it. Assume every ingested system is messy and over-built: too many agents, one capability split
across a family of near-duplicates, runtime state and generated output tangled in with definitions,
vendor-specific naming. `base ingest` understands the source and surfaces its fragmentation; this is
the doctrine for designing the target. Claude Code source formats verified against
code.claude.com/docs on 2026-07-21; re-verify before changing this doctrine (D-025).

## Design from capabilities, not files

List the outcomes the system actually delivers (plan a change, implement it, review it, scan for
security issues, integrate with a work-tracker). Design the minimal base-native structure that
delivers those outcomes. Never preserve the source's file layout or fragmentation — a `security-*`
family of eight near-identical scanners is **one** capability, not eight agents.

## Minimal role-based agents

Prefer a small, stable role set: **analyst / implementer / reviewer**, plus only the specialists that
are genuinely a distinct role. Fold per-technology "experts" (`backend-expert`, `frontend-expert`,
`solr-expert`, …) into the implementer + **domain knowledge**, unless the role truly differs. N
near-identical scanners → **one** `security-review` pipeline with a single auditor agent. A source
with 20+ agents is a signal of over-fragmentation, not richness.

## Capabilities are pipelines; techniques are skills; facts are knowledge

- **Pipeline** — a repeatable capability with stages, gates, and a verifier (implement, security
  review, release). Name it for the outcome.
- **Skill** — a reusable technique invoked on demand (write an SPE script, format an ADO work item).
- **Knowledge** — durable domain facts (glossary, architecture patterns), authored as summaries.
- **Policy / gate** — lifecycle guardrails (block default-branch push) and approval checkpoints.

## Naming

Outcome-named pipelines (`security-review`, not `snyk-gate`), role-named agents (`reviewer`, not
`critic`), kebab-case, no vendor or client cruft in the canonical names.

## Carry, rebuild, or drop the raw material

- **knowledge** dirs (memory, learnings, investigations, specs) → carry the still-true parts as
  authored knowledge (D-019). Do not copy dumps verbatim.
- **state/runtime** dirs (work mirrors, audit logs, handoffs, run outputs) → rebuild in base's
  work / runs / state model; never copy bytes.
- **tooling** dirs (scripts, integrations) → out of canon (D-015); reproduce only where a script *is*
  a capability worth a skill or verifier.
- **generated** dirs (reports, scans, dashboards) → out of scope; regenerate.
- **harness config** (permissions, MCP) → not canon; a standing-denial deny rule may become a gate.

## What base adds

A faithful set of capabilities is the floor. The redesign also gains: work items + kanban with human
verdicts; stage-approval gates recorded as artifacts; runs + an append-only history ledger; typed
verifiers (pass | fail | inconclusive); durable handoff + pickup; cross-harness output (Codex +
Copilot for free); drift-protected generated surfaces.
