# Dodo-free billing release and promotion gate

This gate describes the release state after Dodo Payments access is lost. It
preserves the billing API and route shapes while preventing an unavailable
provider from becoming an accidental paid-access path.

## Runtime contract

- `BILLING_MODE=free` is the safe default when the variable is absent.
- `BILLING_MODE=disabled` is a hard ceiling: payment actions and balance
  reads return `503 BILLING_DISABLED`.
- In `free`, the checkout, portal, and webhook routes remain registered and
  return the stable disabled response; balance returns the deterministic free
  allowance.
- The admin control can persist only `free` or `disabled`. It cannot enable
  Dodo, and it cannot override a release-level `disabled` ceiling.
- Existing subscription and promotion rows are retained as audit data. They
  are not treated as newly verified provider entitlements while Dodo is
  unavailable.

## Required evidence before production promotion

Promotion remains held until every item has a receipt tied to the exact
release commit and environment:

1. Fresh remote CI is green on the pushed release commit, including the
   Tarot generated, fallback, and unavailable fixtures.
2. Rust SDK and TUI packages have publish receipts. The current source
   compile/test gate is green, but packaging requires `noesis-core` (and then
   the dependent SDK/TUI packages) to be available at version `3.3.1`.
3. The deployed Railway service attests the exact source revision, schema
   revision, effective billing mode, and post-deploy health.
4. The image digest has a verifiable build/provenance attestation.
5. Cloudflare source bindings and the intended account are verified through
   an explicitly scoped session; local Wrangler OAuth account identity is not
   sufficient evidence.
6. Vercel protection and same-origin admin proxy checks are verified.
7. A rollback receipt names the prior healthy deployment, schema rollback
   strategy, and operator.
8. Payment promotion is not enabled without durable one-use receipts and
   current Dodo provider credentials/health. Free access may ship without
   those credentials.

## Current branch disposition

The current review branch is
`codex/dodo-free-mode-20260908`, with code closure commit `cf2aa01a`,
follow-on evidence receipts, and this release-preparation pass in draft PR
[#1489](https://github.com/Sheshiyer/Selemene-engine/pull/1489). The PR is
mergeable but remains draft. The latest completed CI run on the prior remote
head, [34242496013](https://github.com/Sheshiyer/Selemene-engine/actions/runs/34242496013),
passed every lane except Integration Tests and the aggregate CI Gate:
billing E2E **4/4**, replay **5/5**, and hooks **3/3** passed, while
`test_capability_route_returns_contract_v1_shape_for_all_ts_engines` returned
`500 INTERNAL_ERROR` instead of `200`. A fresh run is required on the pushed
release-preparation head.

No production migration, Railway/Cloudflare/Vercel deployment, or production
promotion was executed. Live health is not a source, schema, image, or
effective billing-mode attestation. Those external receipts remain explicit
holds rather than inferred from local tests or a healthy endpoint.
