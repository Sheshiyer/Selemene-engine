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

## Evidence status after production promotion

The API is promoted through main commit `561c3e476e134eeac424c77b092b95b9749e0215`
and workflow [34258440332](https://github.com/Sheshiyer/Selemene-engine/actions/runs/34258440332).
The remaining items below are release follow-ups, not permission to enable Dodo:

1. **Complete:** remote CI, source-SHA validation, Railway deployment, API
   smoke, and admin-web smoke are green. The deployed service reports version
   `3.3.1`, 19 engines, 6 workflows, Postgres/Redis `ok`, and bridge `ready`.
2. **Held:** Rust SDK and TUI packages have no publish receipts. The current source
   compile/test gate is green, but packaging requires `noesis-core` (and then
   the dependent SDK/TUI packages) to be available at version `3.3.1`.
3. **Partial:** the workflow validated exact source `561c3e47`, Railway has
   `BILLING_MODE=free`, and post-deploy health is green. A provider-level schema
   revision and exact Railway source/image linkage are still missing.
4. **Partial:** GHCR produced a provenance-backed image digest
   `sha256:7edf1be9bc49433876e1addcd9763ae2019c84105bca62d1ae19fa5ef5e3f066`;
   Railway reports its separately built deployed digest
   `sha256:ba3baabf749bce30c9543f2049410b2da8df6d574067f6dba7ef3c0d4680ec82`.
5. **Held:** Cloudflare source bindings and the intended account are verified through
   an explicitly scoped session; local Wrangler OAuth account identity is not
   sufficient evidence.
6. **Partial:** same-origin admin-web smoke passes; Vercel production protection
   is not configured and remains a hold.
7. **Held:** a rollback receipt names the prior healthy deployment, schema rollback
   strategy, and operator.
8. **Complete:** payment promotion is not enabled without durable one-use receipts and
   current Dodo provider credentials/health. Free access may ship without
   those credentials.

## Current release disposition

The review branch merged through PR [#1489](https://github.com/Sheshiyer/Selemene-engine/pull/1489).
The release record is
[release-3.3.1-free-mode-20260908](https://github.com/Sheshiyer/Selemene-engine/releases/tag/release-3.3.1-free-mode-20260908).
Production is live in free mode; Dodo credentials remain audit-only and no paid
entitlement is inferred from historical subscription or promotion rows.
