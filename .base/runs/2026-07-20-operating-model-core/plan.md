# Plan

1. Record the decisions that expand Base's boundary: repo-vendored versioned packs, first-class
   skills/policies/verifiers, native multi-target agents, deterministic verification, and session state.
2. Extend config and canon loading with a schema-2 compatibility fence, an explicit v0.1 migration,
   ordered pack layers, source provenance, override reporting, and cross-definition validation.
3. Replace copy-flattened pack adoption with managed `.base/packs/<id>/` adoption and guarded upgrades.
4. Render skills, agents, pipelines, and lifecycle policies to the currently verified native
   surfaces. Bind Codex through trusted `.codex/hooks.json`; keep only its non-equivalent
   `session-end` lifecycle assisted. Report Copilot VS Code versus CLI/cloud profiles explicitly.
5. Add `base verify` and `base state` commands with typed reports and durable evidence/state contracts.
6. Ship and dogfood a generic `software-delivery` pack derived from rezidnt's maker/checker loop.
7. Update tethered documentation, regenerate Base's own harness outputs, and run the complete gate.

## Material risks

- Vendor schemas are volatile. Keep mappings isolated in adapters and stamp a verified-as-of date.
- Pack upgrades can destroy local edits. Hash every adopted file and preflight the complete change.
- Canon kinds can collide in shared skill output directories. Reject output-path collisions before sync.
- Hook semantics differ. Normalize input and output through Base's internal hook protocol.
