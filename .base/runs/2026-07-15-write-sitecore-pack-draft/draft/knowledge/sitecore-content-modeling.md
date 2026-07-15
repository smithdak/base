---
id: sitecore-content-modeling
description: Template hierarchy, Standard Values checklist, and field conventions for Sitecore.
---

## Template inheritance

Model in three layers; items are only ever created from the final layer:

```
base templates (single concern, underscore-prefixed: _HasTitle, _HasImage, _HasMetadata)
  └── feature templates (compose base templates; a component's data shape)
        └── page templates (inherit feature + page-level bases; assigned to content items)
```

Base templates define one small set of related fields each. Composition happens through
inheritance at the feature layer, never by re-declaring fields.

## Standard Values checklist

For every template, on `__Standard Values`: insert options (allowed child types), default field
values, default presentation (layout + renderings so new items render immediately), and initial
workflow with its starting state. This is the governance mechanism, not an optional polish step.

## Field conventions

| Rule | Example |
|---|---|
| PascalCase, descriptive, unambiguous | `HeroImage`, `ArticlePublishDate` |
| Booleans prefixed `Is` / `Has` | `IsFeatured`, `HasSidebar` |
| Related fields grouped in sections | `Content`, `Navigation` |

Pick field types by intent: single-line for labels/titles, multi-line for unformatted text,
rich text only where editors genuinely format, General Link for anything linkable, Droptree/
Multilist for references — and keep datasource locations and search scopes on the rendering,
not hardcoded.
