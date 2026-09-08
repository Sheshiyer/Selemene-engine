# External release candidate checklist

Status: prepared, not released  
Candidate line: post-`3.3.1` patch candidate  
Branch: `codex/dodo-free-mode-20260908`  
Review: [PR #1489](https://github.com/Sheshiyer/Selemene-engine/pull/1489)

This document is the external-release handoff for the Dodo-free billing and
Tarot truth work. It records what can be published and what remains blocked.
It does not authorize tag creation, package publication, deployment,
production promotion, or merge.

## Public contract

- The public API retains the billing routes and SDK methods.
- `BILLING_MODE=free` is the safe default when the variable is absent.
- `BILLING_MODE=disabled` is an operator hard ceiling.
- Paid Dodo behavior requires explicit mode, complete credentials, and current
  provider evidence.
- Historical Dodo subscription and promotion rows remain audit data; they do
  not prove current entitlement or extend the prior promotion duration.
- Tarot has a deterministic local fixture. Generated, fallback, and
  unavailable Tarot states are not claimed because this implementation does
  not expose those paths.

## Distribution surfaces

| Surface | Prepared state | Release gate |
| --- | --- | --- |
| `noesis-core` | Manifest and README carry repository metadata | Registry receipt at candidate version |
| `noesis-sdk` | Manifest and README carry repository metadata | `noesis-core` receipt, then SDK receipt |
| `noesis-tui` | Manifest and README added; source install documented | Core and SDK receipts, then TUI receipt |
| `@noesis/sdk` | README and billing caveat updated | Build, pack, registry receipt |
| `@selemene/bridge` | Package provenance metadata updated | Build, pack, registry receipt |
| `@selemene/engine-sdk` | Package provenance metadata updated | Build, pack, registry receipt |

The current local evidence still holds Rust SDK/TUI publication because the
registry does not expose the required `noesis-core` dependency line. No package
has been published or installed from a candidate registry version.

## Required external receipts

- [ ] Exact PR head has a fresh green canonical CI Gate.
- [ ] Package dry-runs pass from a clean candidate commit.
- [ ] Registry pages show the dependency chain in publish order.
- [ ] Railway attests source SHA, schema revision, effective `BILLING_MODE`,
      image digest, and post-deploy health.
- [ ] Cloudflare source/account bindings are read back using the explicitly
      scoped account session.
- [ ] Vercel production protection and same-origin admin proxy checks pass.
- [ ] Rollback target and schema rollback path are recorded and exercised.
- [ ] Only after the above: tag, draft release, package publication, deploy,
      promotion, and merge are separately approved.

## Current holds

- Advisor verification is unavailable because the local OAuth session expired.
- The latest completed PR CI run is
  [34244891890](https://github.com/Sheshiyer/Selemene-engine/actions/runs/34244891890)
  on release head `00bb9e59`. Security, workspace, TypeScript, Python,
  workflow-parity, test, and build lanes passed. Integration Tests and the
  aggregate CI Gate failed only because the capability-route test returned
  `500 INTERNAL_ERROR` instead of `200`; billing E2E **4/4**, replay **5/5**,
  and hooks **3/3** passed. A fresh green run is required after that blocker
  is resolved.
- Local Postgres lacks the configured `noesis_user` role; local DB-backed billing
  replay is not a production authorization receipt.
- Railway, Cloudflare, Vercel, image provenance, and rollback attestations are
  not claimed by this candidate.

## Rollback posture

Do not delete tags, revert deployments, alter production schema, or publish a
replacement package as an unplanned rollback. If a candidate is released and
fails, record the prior healthy source/image/schema tuple first, then use the
scoped provider runbook and a new patch version for immutable registries.
