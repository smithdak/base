# Software delivery operating model

Base is the canonical system of record. Packs provide ordered, immutable, repository-vendored
defaults. `.base/canon/` is the final project-specific overlay. Generated Claude Code, Codex, and
GitHub Copilot files are adapters and must never become independent sources of truth.

The lifecycle is intentionally split into work items (what is being delivered), runs (how one
attempt proceeded), state (where to resume), decisions (why durable constraints exist), and
verifier evidence (what was actually proved).
