---
id: base
description: Full source, lint, test, specification, and generated-surface contract for Base.
checks:
  - id: format
    run:
      - cargo
      - fmt
      - --check
    timeout-seconds: 120
  - id: clippy
    run:
      - cargo
      - clippy
      - --all-targets
      - --all-features
      - --
      - -D
      - warnings
    timeout-seconds: 300
  - id: tests
    run:
      - cargo
      - test
      - --all-targets
      - --all-features
    timeout-seconds: 600
  - id: generated-surfaces
    run:
      - cargo
      - run
      - --quiet
      - --
      - sync
      - --check
    timeout-seconds: 300
---

This suite is the canonical completion contract for Base itself. A local pass does not prove target
runtime behavior outside the rendered contract tests or installation in a remote agent environment.
