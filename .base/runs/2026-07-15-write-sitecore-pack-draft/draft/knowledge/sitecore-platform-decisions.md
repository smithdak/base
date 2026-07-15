---
id: sitecore-platform-decisions
description: XM Cloud vs. XM/XP signals, self-hosted topology, and migration phasing.
---

## Platform choice

XM Cloud (headless, Content SDK/Next.js) is the default for new work. Choose self-hosted XM/XP
only on a named, concrete constraint: data residency or no-SaaS mandate, an irreplaceable
xDB/legacy dependency, or a frozen re-platform budget. Team familiarity with MVC is a training
cost, not a constraint. Classic xDB personalization does not port — it re-architects onto
Personalize/CDP.

## Self-hosted topology (when XM/XP is the answer)

- CM/CD separation is mandatory for anything author-facing in production.
- Solr backs search and indexing — size it deliberately and plan index rebuild time at scale.
- Scale CD horizontally behind the load balancer; CM stays single-active; session state moves
  to Redis with multiple CDs; add a dedicated publishing instance when content volume justifies.
- Know the cache layers before tuning: rendering/output cache, item/data cache, prefetch, CDN.
  Most "Sitecore is slow" reports are missing or mis-keyed rendering caching.

## Migration (XP → XM Cloud)

Phases: assess (inventory templates, renderings, custom pipelines, xDB dependencies,
integrations) → re-platform the front end → carry content model over via SCS → re-architect
personalization on Personalize/CDP → re-point integrations to Experience Edge → parallel-run
and cut over with redirects planned.

Two standard under-scopings to flag every time: the head is a **rebuild**, not a port (MVC
views/controllers do not migrate), and personalization is a **re-think**, not a migration.
