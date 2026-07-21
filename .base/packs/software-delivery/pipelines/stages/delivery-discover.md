---
id: delivery-discover
description: Ground the requested outcome in repository evidence and current state.
---

Run `base state context`, inspect repository guidance and the owning code, and bound the requested
outcome. Before any gate or implementation, atomically create
`.base/runs/YYYY-MM-DD-<short-kebab-slug>/`; if it already exists, retry with the next numeric
suffix (`-2`, `-3`, and so on). Reserve its empty `evidence/` directory. Record assumptions,
constraints, acceptance checks, and unresolved proof obligations in that run's `task.md`. Use this
same run slug for every later `base approve` and `base verify --run` command.
