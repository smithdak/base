---
id: sitecore-conventions
description: Always-on constraints for working in a Sitecore repository.
---

- Serialized items (SCS) are the source of truth: schema and template changes land as
  serialization in version control, never hand-made in a shared environment's Content Editor.
- Never place secrets, tokens, or connection strings in serialized items or committed config;
  they belong in the platform's secret store.
- Every template ships with Standard Values: insert options, sensible field defaults, initial
  workflow and state, and default presentation. A template without Standard Values is unfinished.
- Respect Helix layering in code and templates: Foundation never depends on Feature, Feature
  never depends on Project, and cross-feature references go through Foundation.
- Name fields PascalCase and unambiguous (`ArticlePublishDate`, not `Date`); prefix booleans
  with `Is`/`Has`; group related fields into named template sections.
- Treat CM/CD separation as given: nothing author-facing on a public instance, no `/sitecore`
  admin surface on CD, and no publishing from a delivery role.
