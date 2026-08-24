---
id: W-0016
title: Start preserves existing harness surfaces
status: review
verdict: pending
created: 2026-08-24
tags:
- cli
- onboarding
- brownfield
---

# Start preserves existing harness surfaces

## Acceptance Criteria

- [ ] start detects existing harness files that generated output would overwrite and fails with guidance
- [ ] start --migrate-native moves recognized files byte-preserving into .base/native and the first sync composes them
- [ ] existing overlay destinations are never overwritten by migration
