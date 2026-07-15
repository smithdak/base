---
id: W-0008
title: Approval-as-artifact gate satisfaction
status: done
verdict: pass
created: 2026-07-15
tags:
- gates
- design
---

# Approval-as-artifact gate satisfaction

Motivated by the 2026-07-15 W-0006/W-0007 session: the writing and automation runs' plan-approval
gates were satisfied by a session `/goal` directive, and the only record was prose the agent wrote
about its own judgment call. Stage approval is currently *assisted* everywhere — a STOP paragraph
the agent is trusted to obey. This item makes gate satisfaction a **file**: the gate declares a
`satisfied-by` artifact path (e.g. `approvals/plan.md` in the run folder), a CLI verb writes the
stamped record, and the Claude adapter denies mutating tools while the artifact is missing —
upgrading stage approval from assisted to enforced on the reference harness, same jump the
default-branch denial already made. Files-are-the-substrate (D-004) applied to consent.

Keep D-008's line: gate declarations stay data (paths, flags); no conditions or expressions.

**Follow-on axes** (same run-state + compiled-hook machinery; separate items when real use
demands them):
1. *Gate delegation policy* — gates declare `per-run` vs `standing-allowed`, turning the /goal
   judgment call into declared policy.
2. *Tool posture per stage* — `posture: read-only` on intake/investigate/diagnose/synthesize,
   enforced by hook; the research pipeline currently promises read-only without enforcement.
3. *Stage parameters* — the caps/flags slot SPEC §3 reserves (`evidence: required`,
   `write-scope`), compiled to prose and, where possible, hook config.

## Acceptance Criteria

- [x] A decision records approval-as-artifact semantics: stage gates declare a satisfied-by artifact path relative to the run folder, and gate satisfaction is a durable file, never only conversation
- [x] base approve <run> <gate> writes the stamped approval record (who, when, what was approved)
- [x] The Claude adapter compiles an enforced mechanism: mutating tools are denied while an active run's gated stage lacks its approval artifact; the enforcement matrix reports the per-target fidelity honestly (enforced on claude)
- [x] Standing approvals (e.g. a session goal directive) are recorded as explicit approval artifacts citing their source, not inferred in prose
