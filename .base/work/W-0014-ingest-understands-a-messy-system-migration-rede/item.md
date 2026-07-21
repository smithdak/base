---
id: W-0014
title: Ingest understands a messy system; migration redesigns it
status: review
verdict: pending
created: 2026-07-21
tags:
- ingest
- migration
- architecture
---

# Ingest understands a messy system; migration redesigns it

## Acceptance Criteria

- [ ] base ingest de-floods permissions (one ConfigSummary, not per-entry) and classifies the whole tree into knowledge/state/tooling/generated raw-material rollups
- [ ] reader surfaces capability clusters + redundancy signals (name-affix + tool-set similarity) so over-fragmentation is visible; smart root detection accepts the .claude dir directly
- [ ] migration doctrine (migration-architecture.md) + reframed /migrate pipeline drive a capability-first redesign (consolidate/rename/drop), human-gated; not a 1:1 mirror
- [ ] cargo test --all-targets + clippy clean; base check && base sync --check green; D-029 recorded
