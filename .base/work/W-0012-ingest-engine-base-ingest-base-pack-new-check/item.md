---
id: W-0012
title: 'Ingest engine: base ingest + base pack new/check'
status: review
verdict: pending
created: 2026-07-21
tags:
- cli
- ingest
- migration
---

# Ingest engine: base ingest + base pack new/check

## Acceptance Criteria

- [ ] base ingest <path> reads a Claude Code project (plugin manifest or loose .claude/) into a normalized Inventory + MappingReport with native/partial/claude-only/unmapped fidelity buckets
- [ ] base pack new <id> scaffolds a valid library pack skeleton; base pack check <path> validates a drafted pack before adoption
- [ ] ingest never writes canon (D-019); SPEC section 7 verb table + count updated and tests/spec.rs green
- [ ] cargo test --all-targets passes including round-trip and fidelity-bucket tests
