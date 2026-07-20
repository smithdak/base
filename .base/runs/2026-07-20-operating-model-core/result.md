# Result

## Outcome

Base is now the vendor-neutral development operating-model core. Repository canon, ordered
versioned packs, durable work/run state, approval and policy contracts, verifier evidence, and
generated-surface ownership live in Base; Claude Code, Codex, and GitHub Copilot remain execution
clients.

## Delivered

- Schema 2 and `requires-base` fail before mutation under incompatible v0.1 runtimes.
- Immutable repository-vendored packs compose global, ordered pack, and project layers with hashes,
  upgrade preflight, and override reporting. The generic `software-delivery` pack is version 1.2.0.
- Rules, agents, skills/resources, pipelines/stages, policies, verifiers, and knowledge are canonical
  first-class definitions.
- Native Claude, Codex, and Copilot instructions, agents, skills, hooks, and product-specific pipeline
  profiles compile from one repository model. Target-specific migration inputs compose through the
  allowlisted `.base/native/` mirror.
- Work IDs use committed atomic reservations; repository commands coordinate through bounded shared
  and exclusive locks; handoff state binds an existing work item and run.
- Approval verdict creation is race-safe but deliberately described as self-asserted workflow state,
  not authenticated authorization. Default-branch hooks cover common Git pushes and known GitHub MCP
  write namespaces.
- Verification records typed `pass | fail | inconclusive` evidence with command, duration, byte count,
  and hashes. UTF-8 generated identity is CRLF/LF invariant; non-UTF-8 resources remain byte-exact.

## Verification

- Canonical verifier passed all four checks at
  `evidence/verifications/base-20260720T211559.971Z-p25712.json`.
- `cargo fmt --all -- --check`, Clippy with `-D warnings`, 39 unit tests, 45 CLI integration tests,
  1 specification test, and `cargo build --release` passed.
- Installed `base 0.2.0` passed hook protocol 1 capability checks, `base check --json` with no warnings,
  and `base sync --check --json` with no drift.
- Installed-runtime protocol probes passed for default-branch denial and session context on all three
  targets. Copilot's observed `github-mcp-server-*` namespace is mapped and regression-tested.
- Global pack-only refresh retained the existing Sitecore pack and installed `software-delivery`
  1.2.0.

## Falsification outcome

An independent review found and drove fixes for retained work-state invariants, unsafe agent tool
metadata, CRLF false drift, approval MCP read overreach, and Copilot's live MCP naming mismatch. No
source-level P0-P2 release blocker remained after the final pass.

## Residual proof gaps

Linux execution, a live Claude hook session, Codex hook trust/execution, and post-fix Copilot cloud
execution were not proven. These are explicit runtime/CI cells; they do not change the local source
verdict. Hook controls remain bypassable local workflow guardrails, with authoritative default-branch
protection belonging at the Git server.
