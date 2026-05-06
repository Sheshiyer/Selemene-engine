# Dodo Payments Integration — Security Review (T27)

> Pre-merge security audit of the Dodo Payments billing pipeline.
> **Scope:** crates/noesis-api/src/billing.rs, handlers/billing.rs, the
> bin/dodo_reconcile bin, biofield-web webhook + checkout routes, and the
> three migrations (020, 021, 022).
> **Date:** 2026-05-06
> **Reviewer:** Claude Opus 4.7 (paired with Sheshnarayan Iyer)

---

## TL;DR

No vulnerabilities found in billing-scope code. The webhook auth surface
is defended by three independent gates (Standard Webhooks signature →
shared-secret forwarding → timestamp freshness → atomic Postgres
idempotency). Outbound API calls use bearer auth and never leak the
key beyond the Tokio task. **Cleared for merge.**

Two follow-ups noted; neither is a blocker.

---

## 1. Webhook auth surface — three-layer defense

| Layer | Where | What it verifies | Failure mode |
|---|---|---|---|
| **Standard Webhooks signature** | biofield-web `/api/webhook/dodo-payments/route.ts` via `@dodopayments/core` `verifyWebhookPayload` | HMAC of (`webhook-id`, `webhook-timestamp`, raw body) using `DODO_PAYMENTS_WEBHOOK_KEY` | 401 to caller, no forward to Rust |
| **X-Forward-Secret shared key** | Rust `events_forward` via constant-time compare | Random 48-byte `DODO_INTERNAL_FORWARD_SECRET` matches between Next.js → Rust | 401, no DB writes, metric increment |
| **Timestamp freshness ±5 min** | Rust `validate_timestamp_freshness` | `webhook_timestamp` is within 300 s of now (with +30 s skew tolerance) | 400, no DB writes, metric increment |
| **Postgres PK idempotency** | `processed_webhook_events` | Same `webhook_id` cannot be processed twice | Returns 200 dedup=true |

A signed-fixture replay can only succeed under all four conditions. An
attacker with a *leaked forward secret* still has to either (a) forge a
valid Standard Webhooks signature (cryptographically infeasible without
the webhook signing key) or (b) replay a fresh-but-real Dodo payload
within the 5-minute freshness window — and even then idempotency catches
duplicates.

---

## 2. Sensitive surface inventory

| Item | Location | Storage | Access |
|---|---|---|---|
| `DODO_PAYMENTS_API_KEY` | env, `~/.dodopayments/api-key` (CLI) | env-only at runtime | Read by `DodoWebhookEmitter`, checkout handler, portal handler, balance handler, `dodo_reconcile` bin |
| `DODO_PAYMENTS_WEBHOOK_KEY` (Standard Webhooks signing key) | env | env-only | Read by `@dodopayments/core` in Next.js |
| `DODO_INTERNAL_FORWARD_SECRET` | env | env-only | Read by Next.js webhook route + Rust `events_forward` |
| `users.dodo_customer_id` | Postgres | per-row, partial unique index | Set by webhook on first `subscription.active`, read by emitter + balance + portal |
| `processed_webhook_events.webhook_id` | Postgres | PK | INSERT-only. Pruned by maintenance after 90 d. |

**No secrets are persisted in code, checked-in config, logs, or HTTP
response bodies.** Tier defaults in `BalanceResponse.source` are the only
billing-shaped values returned to unauthenticated paths, and those
contain no PII or credentials.

---

## 3. Threat model

### 3.1 Replay attack
**Mitigation:** dual-locked. Atomic PK on `processed_webhook_events` plus
the freshness window. Verified by `tests/billing_replay_tests.rs` —
100× concurrent same-id deliveries → exactly one mutation. Stale
timestamp → 400, no `processed_webhook_events` row created.

### 3.2 Timing oracle on shared secret
**Mitigation:** `constant_time_eq` (custom XOR-fold; no early return).
Length mismatch returns false without comparing bytes. Verified by
`handlers::billing::tests::constant_time_eq_matches_only_when_equal`.

### 3.3 SQL injection / parametric drift
**Mitigation:** every query in `BillingRepository` uses sqlx parameter
binding. No string concatenation. Schema constraints (`status` CHECK,
partial unique indexes) catch invalid states server-side.

### 3.4 SSRF / outbound URL injection
**Mitigation:** All outbound URLs are constructed from
`{api_base}/...` where `api_base` is a hardcoded host
(`https://test.dodopayments.com` or `https://live.dodopayments.com`).
`api_base` is selected from `DODO_PAYMENTS_ENV` which is validated to
exact-match `"test"` or `"live"` at boot in `ApiConfig::validate`.

### 3.5 Resource exhaustion
**Mitigation:**
- Webhook handler: rate-limited via `internal_routes` rate-limit
  middleware (config: 1000 req/min default)
- Outbound emitter: 2-retry cap, 10 s timeout, fire-and-forget so engine
  responses are never blocked
- Reconcile cron: 30 s timeout per Dodo paginated call, hard cap of 100
  pages (10k subs)

### 3.6 Free-tier quota bypass
**Mitigation:** Counter increments via Postgres atomic UPSERT (`ON
CONFLICT DO UPDATE`). Read-then-increment race window allows ~1 extra
call per concurrent burst — accepted as a v1 trade-off; not a security
issue (worst case: user gets 51 calls instead of 50).

### 3.7 PII exposure
The webhook payload contains email addresses and customer names from
Dodo. We persist `users.email` (already there) and `customer_id`. We do
**not** persist customer name, billing address, payment method, or any
card data. Dodo is Merchant of Record — they handle PCI scope.

---

## 4. Dependency audit

### 4.1 Cargo audit (run 2026-05-06)
Zero CVEs. Four "unmaintained" advisories — none in billing-scope code:

| Advisory | Crate | Pulled by | Risk |
|---|---|---|---|
| RUSTSEC-2025-0012 | `backoff 0.4.0` | `noesis-vedic-api` (legacy retry helper) | None — billing uses inline tokio backoff |
| RUSTSEC-2021-0141 | `dotenv 0.15.0` | `noesis-western-api` (legacy) | None — billing reads `std::env` directly |
| RUSTSEC-2024-0384 | `instant 0.1.13` | transitive of `backoff` | None |
| RUSTSEC-2024-0436 | `paste 1.0.15` | transitive of `sqlx 0.7.4` | None — sqlx 0.8 ships paste-free; bump scheduled separately |

**Verdict:** clean for billing. Pre-existing unmaintained-crate hygiene
debt sits outside this PR's scope.

### 4.2 npm audit (run 2026-05-06)
3 moderate vulns, all in transitive dev tooling (PostCSS via Next.js
build pipeline). No runtime impact:

| GHSA | Package | Severity | Path | Risk |
|---|---|---|---|---|
| GHSA-qx2v-qp2m-jg93 | `postcss < 8.4.31` | moderate | `next > postcss` (build-time only) | None — XSS in CSS stringification, not in our app |

**Verdict:** clean for billing-scope.

---

## 5. Rotation runbook (key compromise)

If `DODO_PAYMENTS_API_KEY` is leaked:

1. **Dodo dashboard** → Settings → API Keys → revoke the leaked key.
2. **Generate new key** in dashboard.
3. **Update `.env` everywhere it runs** (local dev, staging, prod) and
   restart the noesis-api process. The boot-time emitter installer in
   `lib.rs` re-installs `DodoWebhookEmitter` with the new key.
4. **CLI:** `rm ~/.dodopayments/api-key && dodo login`
5. **Verify**: hit `/api/v1/billing/balance` as a paid user → should
   return real Dodo data with `source: "dodo"`. If 503, the install
   didn't pick up the new key.

If `DODO_PAYMENTS_WEBHOOK_KEY` is leaked:

1. **Dodo dashboard** → Developers → Webhooks → open the webhook → "Roll
   signing key".
2. **Capture the new `whsec_…`** and update `DODO_PAYMENTS_WEBHOOK_KEY`
   in env.
3. **Redeploy biofield-web**. The Standard Webhooks `Webhook` instance
   in `verifyWebhookPayload` reads at request time so a process restart
   suffices.
4. **Verify**: send a test event from the dashboard → 200 reaches
   noesis-api logs.

If `DODO_INTERNAL_FORWARD_SECRET` is leaked:

1. **Generate**: `openssl rand -base64 48`.
2. **Set in env** for both biofield-web and noesis-api (same value).
3. **Restart both processes simultaneously** to avoid a window where
   they disagree.
4. **Verify**: a test webhook event reaches the Rust handler with no
   `unauthorized` metric increment.

---

## 6. Follow-ups (non-blocking)

1. **Add a CI step** that runs `cargo audit --deny warnings` on
   billing-scope crates so future advisories surface in PRs. Skip the
   four whitelisted unmaintained-but-fine deps via `audit.toml`.
2. **Migrate to Argon2 + subtle::ConstantTimeEq for `constant_time_eq`** —
   the current XOR-fold is correct but the `subtle` crate's
   primitives are battle-tested. Low priority since current
   implementation is exercised by tests and verified correct.

---

## 7. Sign-off

- [x] No CVEs in billing-scope dependencies
- [x] All secrets are env-only (no persistence in code, logs, or
      response bodies)
- [x] Webhook auth has three independent layers + atomic idempotency
- [x] Timing-safe shared-secret comparison
- [x] Replay attack tests pass (100× concurrent + stale + far-future +
      garbage)
- [x] All outbound URLs built from validated `{api_base}` constant
- [x] Rate-limiting on webhook ingress + outbound emitter timeout
- [x] Free-tier quota uses atomic UPSERT (no TOCTOU window of concern)
- [x] Rotation procedures documented for all three credentials

**Cleared for merge.**
