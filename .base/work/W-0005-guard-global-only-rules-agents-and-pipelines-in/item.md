---
id: W-0005
title: Guard global-only rules, agents, and pipelines in committed output
status: todo
verdict: pending
created: 2026-07-15
tags:
- design
- gates
---

# Guard global-only rules, agents, and pipelines in committed output

## Acceptance Criteria

- [ ] Decision extends the D-017 treatment (render guard or vendoring) to rules, agents, and pipelines
- [ ] Fresh-clone sync --check stays green when the global canon holds definitions the project lacks
- [ ] base check reports every excluded global-only definition
