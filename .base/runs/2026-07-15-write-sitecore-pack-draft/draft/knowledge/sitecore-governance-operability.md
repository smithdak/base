---
id: sitecore-governance-operability
description: Governance, security, CI/CD, and operability expectations for Sitecore platforms.
---

## Content governance

Workflow on every editable template, insert-option discipline everywhere, and an explicit
shared-vs-local content boundary (shared templates/renderings/themes in collection-level or
foundation modules; site-specific content under each site). Multi-site at scale means SXA site
collections in one solution — separate instances only when legal or data isolation demands.
Without these, large platforms rot within a year.

## Security

Least-privilege roles; no `/sitecore` admin surface on CD; Experience Edge tokens scoped
narrowly; secrets in a vault, never serialized; a patch cadence that actually exists. On
XM Cloud, review environment/role mapping and deployment credentials explicitly.

## CI/CD

Serialization (SCS) is the pipeline's source of truth: automated deploys, environment
promotion, and publish/index-rebuild steps as pipeline stages rather than manual rituals.

## Observability and DR

Settle the log story (Application Insights / platform logs / front-end APM) before go-live.
Define RPO/RTO. On XM Cloud know the ownership split: Sitecore manages the platform; the
serialized items and the head application are yours to protect and restore.
