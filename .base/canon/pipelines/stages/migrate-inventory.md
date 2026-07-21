---
id: migrate-inventory
description: Understand the source system and retain the understanding as run evidence.
---

Run `base ingest <source-path> --run <run-slug>`. It resolves the source (a project root or the
`.claude` directory itself), summarizes harness config instead of enumerating it, classifies the
whole tree into knowledge / state / tooling / generated raw material, and surfaces capability
clusters and redundancy signals. Read the whole understanding report. Note the fragmentation signals
(families of near-duplicate agents, over-large agent counts), the real capabilities behind them, and
which raw material is durable knowledge versus runtime state versus tooling versus generated output.
Do not design or author yet — understand first.
