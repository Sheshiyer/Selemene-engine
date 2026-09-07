---
phase: 03-capability-and-contract-closure
plan: "14"
subsystem: cli-generators
tags: [bun, typescript, cli, openapi, langchain, authentication]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "13"
    provides: typed CLI API-key/bearer transport and bounded HTTP errors
provides:
  - protected Rust-only OpenAPI merge for public generation
  - deterministic 17-mirror LangChain source projection
  - sidecar, validate and mixed-auth exclusion guarantees
affects: [03-15, 03-16, 03-21]

tech-stack:
  added: []
  patterns:
    - public generators consume only allowlisted protected Rust paths
    - generated clients use mutually exclusive X-API-Key or bearer branches
    - unsupported validation and direct sidecar paths are filtered before emission

key-decisions:
  - "The CLI keeps tsUrl as legacy configuration but never merges direct sidecar OpenAPI paths into public generated tools."
  - "Rust path-derived names retain the engines_ segment so generated names remain traceable to the protected route."
  - "The generator test inspects returned source strings without writing artifacts or contacting a service."

requirements-completed: [CON-02]
requirements-progressed: []

duration: 8min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 14 Summary

**CLI and LangChain generation now projects only the protected Rust public operation contract.**

## Accomplishments

- Restricted OpenAPI merging to the protected Rust status, engine and workflow path families.
- Removed direct TypeScript sidecar spec fetching and `/ts` re-export from public generation.
- Added explicit filtering for unsupported `/validate` operations in Claude, OpenAI and LangChain generators.
- Updated generated LangChain clients to use `SELEMENE_RUST_URL` and mutually exclusive `X-API-Key`/bearer credentials.
- Added deterministic snapshots covering the exact 17 public mirrors, including `raaga`, and excluding non-public identities.

## Verification Evidence

- `cd bridges/cli && bun test src/generators/langchain.test.ts src/core/http.test.ts` — 7 passed.
- `cd bridges/cli && bun run typecheck` — passed.
- No provider, database, Railway, Cloudflare, deployment, merge or release mutation occurred.

## Deviations from Plan

- The existing path-derived naming convention retains `engines_` in generated function and tool names; the snapshot records that canonical route-derived form while explicitly covering `raaga`.
- The generator accepts `tsUrl` for backward-compatible configuration parsing, but the value is not fetched or emitted into generated public source.

## User Setup Required

None.

## Next Phase Readiness

Plan 15 can align the remaining public SDK and universal-tool boundaries with the canonical protected Rust contract.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 14*
