---
id: migrate-inventory
description: Model the source system deterministically and retain its inventory as run evidence.
---

Run `base ingest <source-path> --run <run-slug>` against the system to migrate. This models a Claude
Code source — a `.claude-plugin/plugin.json` bundle first, else loose `.claude/` directories — into a
portable inventory plus a canon mapping/fidelity report, retained under the run's
`evidence/migration/`. Read the whole report. Account for every artifact and every entry in the
`unmapped`, `claude-only`, and out-of-canon buckets — nothing is dropped silently. Do not author any
canon yet; ingest reports, it never writes canon.
