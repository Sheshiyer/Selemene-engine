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

This branch has not pushed a remote branch, created a new review, executed a
production migration, deployed to Railway/Cloudflare/Vercel, or promoted
production. Those actions require the remote release operator and remain
explicit holds rather than inferred from local tests.
