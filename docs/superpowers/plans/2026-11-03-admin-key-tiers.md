# Simple API Key Creation and Tier Entitlements Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task by task. Preserve existing work and verify each deliverable before integration.

**Goal:** Make creating an API key reliable and simple: select Basic, Pro, or Enterprise and let the server assign the correct engine scopes, access level, and rate limit.

**Architecture:** One server-owned entitlement policy drives API key creation and tier changes. The dashboard chooses a tier and displays the returned effective access; it never constructs engine permission strings. Product access tiers remain separate from human administrator roles.

**Tech stack:** Rust/Axum, SQLx/PostgreSQL, Next.js/React, Railway, Cloudflare Access.

**Spec:** User request in this chat on 3 November 2026: repair the existing enterprise key, fix new-key creation, and plan three simple tiers with internally mapped scopes. Implementation completed and deployed on 3 November 2026. See `docs/release/admin-api-key-tiers-2026-11-03.md` for actual release evidence and deviations.

## Current evidence and completed production repair

- Production Railway project: `robust-adventure` (`11eedde4-41e6-4f51-b86b-cf77111cf592`), environment `production`, service `Selemene-engine`.
- Cloudflare account beginning `9d9d` routes `144.tryambakam.space/api/*` through `selemene-admin-api-proxy` to the production Railway API.
- Existing key ID `f882a3b6-2731-46e9-98f7-ec9544fc044c`, name `witness-agents-service`, was already enterprise but held only `basic:access` and access level 0. `basic:access` is a baseline permission, not a product-tier label.
- Completed: merged the eight requested engine read scopes, raised this key to level 2, and independently read back the committed row. Its secret, owner, enterprise tier, rate limit, expiry, and admin roles were preserved. Scope enforcement remains enabled.
- Live verification: Human Design, Gene Keys, Panchanga, Vimshottari, Transits, Numerology, Biorhythm, and Vedic Clock each returned HTTP 200 with results. Tarot returned HTTP 403 `ENGINE_SCOPE_FORBIDDEN`. Admin session and key listing returned 200; liveness returned 200.
- The secret remains in `/Users/sheshnarayaniyer/.claude/.env`, variable `SELEMENE_API_KEY`. No secret is included here. Private rollback and sanitized verification records are under `/Users/sheshnarayaniyer/.codex/private/selemene-key-repair-2026-11-03/`.
- Source diagnosis: `create_api_key_with_events` names 12 insert columns but originally provided only 11 values, omitting the bind for `rotated_from_key_id`. Existing uncommitted work supplies `$11` and adds diagnostics/tests. Preserve and review that work rather than overwrite it. This source diagnosis is not a claim that a deployed creation repair has been verified.
- The current form exposes manual permissions with obsolete `engines:<id>` values; enforced calculation routes require `<id>:read`. Creation defaults omitted permissions to `basic:access` and omitted access level to 0, regardless of enterprise tier.
- This checkout differs from current `origin/main`. Use an isolated current-source branch for implementation and deliberately carry over the existing key-creation fix; do not reset, rebase, or overwrite the working checkout.

## Global constraints and planning defaults

- Preserve existing active keys and all concurrent local work. Do not rotate, delete, or disclose the existing secret.
- No Cloudflare authentication weakening, scope-enforcement bypass, billing-provider change, or pricing change.
- UI labels are exactly **Basic**, **Pro**, **Enterprise**. Keep persisted tier codes `free`, `premium`, `enterprise` for compatibility with current billing/quota code. Normalize incoming `basic` to `free` and `pro` to `premium`; accept existing canonical codes. Do not offer `standard` or a fourth tier.
- Proposed entitlement defaults reuse the existing free/premium/enterprise model and rate-limit defaults. They are planning defaults, not additional production grants made by this repair:

| UI tier | Stored code | Default engine scopes | Access level | Requests/minute |
| --- | --- | --- | --- | --- |
| Basic | free | `panchanga:read`, `numerology:read` | 0 | 60 |
| Pro | premium | Explicit read scopes for all supported registered calculation engines | 3 | 1,000 |
| Enterprise | enterprise | Explicit read scopes for all supported registered calculation engines | 5 | 10,000 |

- Include `basic:access` in every bundle for compatibility. Render it as baseline API access, never as the key's product tier. Retain existing Basic/free quotas; do not invent new prices or quota values.
- Freeze the supported engine IDs in the reviewed policy at implementation time, using actual API registration. Unknown or newly registered engines must not automatically gain access before policy review. Do not reuse the stale historical generator list blindly.
- Enterprise grants engine access, not `admin:*`. Preserve administrator authorization through the existing owner/Cloudflare roles. Do not grant human admin powers to newly created customer keys.

## Review focus

- New-key SQL and lifecycle event inserts must commit together, with correct placeholder binding.
- A tier label must agree with the stored scopes, phase access, and rate limit.
- Browser requests or legacy API overrides must not increase privileges above the selected tier.
- Role assignment, rotation, and tier changes must not accidentally erase engine scopes or add administrator permissions.
- Secret reveal, retries, and error handling must not disclose credentials or create duplicate keys accidentally.

## Task 1: Repair and prove key creation

**Files:** `crates/noesis-data/src/repositories/admin_repository.rs`, `crates/noesis-api/src/handlers/admin.rs`, and focused API/data integration tests.

- [x] Carry the existing 12-column/11-bind repair into an isolated branch based on current remote source, preserving unrelated diagnostics and tests in the original checkout.
- [x] Add a real PostgreSQL test covering successful insertion with nullable `rotated_from_key_id`, matching lifecycle event, and transaction rollback if the event insert fails. A skipped database test is not acceptance.
- [x] Verify the create API returns 201 with a usable one-time secret, list returns the new row without the secret/hash, and a failed request returns a bounded actionable error without SQL or credentials.
- [x] Run focused repository and API tests against a disposable database. Never point test suites at production.

## Task 2: Centralize tier entitlement mapping

**Files:** `crates/noesis-auth/src/lib.rs` (or a small sibling tier-policy module), the admin handlers, and role/tier update methods in the existing admin repository.

**Interfaces:** Introduce `KeyTier` with Basic/Pro/Enterprise variants and `KeyEntitlements { permissions: Vec<String>, consciousness_level: u8, rate_limit: u32 }`. Parse aliases once and return the canonical stored code. Creation, tier changes, and response labels consume this policy.

- [x] Test all three bundles, aliases, invalid tiers, deduplicated canonical `<engine-id>:read` scopes, and every supported engine's phase requirement.
- [x] Derive all defaults server-side. Keep `POST /api/v1/admin/api-keys` and its existing one-time-secret response contract; require no raw scopes from the dashboard. If tier is omitted by an old client, normalize the owner's existing supported tier and apply its bundle.
- [x] Retain legacy optional scope/rate/access-level fields only as restrictions within the tier bundle. Reject attempts to exceed it with 422. The new dashboard does not send these fields. Unknown tiers fail closed.
- [x] Ensure tier changes recompute tier-owned entitlements atomically and rotation preserves the effective bundle. Role updates must preserve engine permissions instead of replacing every active key's permission array with admin-role permissions.
- [x] Test Basic allowed/denied calls, Pro/Enterprise phase access, admin-role separation, rejected privilege escalation, tier downgrade, role updates, and rotation.
- [x] Prepare a preview of existing keys whose stored bundle differs from their tier. Preserve explicitly scoped service keys by default; do not run a broad entitlement backfill. Identify the repaired existing key separately and record the proposed Enterprise bundle transition before rollout.

## Task 3: Simplify the admin key form

**Files:** `apps/admin-web/app/(protected)/api-keys/page.tsx`, `apps/admin-web/src/lib/api.ts`, and `apps/admin-web/src/types/admin.ts`.

- [x] Replace permission checkboxes and access-level/rate-limit controls with a tier selector containing exactly Basic, Pro, and Enterprise. Keep key name; default the owner to the signed-in administrator, allowing existing owner selection for managed customer keys. Keep expiry optional.
- [x] Send the selected canonical tier and ordinary key metadata only. Render entitlement summaries from server-derived values. Do not maintain a second scope policy in the browser.
- [x] Display Enterprise prominently for the repaired key; use friendly tier labels in filters, detail panels, and list rows. Keep baseline permission details secondary.
- [x] Disable submission while pending, retain form values on error, prevent blind automatic create retries, and keep the existing one-time secret/copy experience without logging or persistent browser storage.
- [x] Run dashboard typecheck/build and meaningful form tests for all three tiers, keyboard use, server errors, duplicate-click prevention, and one-time secret handling.

## Task 4: Rollout and end-to-end acceptance

- [x] Deploy the reviewed API and dashboard together through their existing Railway/Vercel paths, leaving the Cloudflare Worker and Access policies intact.
- [x] Use staging first: through `144`'s equivalent staging/browser path, create one disposable key per tier, verify real API access against the tier policy, exercise rotation/revocation, then clean up only those disposable keys. Record IDs and sanitized results, never raw secrets.
- [x] Verify the actual production browser flow at `https://144.tryambakam.space/admin/api-keys`: create a named disposable key, copy its secret once, use it successfully, refresh the listing, and revoke it. Use existing browser tooling and complete the browser QA gate before claiming the dashboard is fixed.
- [x] Recheck the original enterprise key and its eight previously verified calculations. Any wider Enterprise bundle transition must be explicit in the release receipt; do not describe the current eight-scope repair as full-tier reconciliation.
- [x] Record API/browser acceptance separately from database tests and deployment health. Roll back deployments if creation fails; use targeted saved authorization fields for a key-access reversal, preserving later concurrent changes.

## Status

- [x] Existing enterprise key repaired for the eight requested calculations and verified live.
- [x] Three-tier creation and server-side mapping work added to project planning.
- [x] Key-creation code integrated, tested, and deployed.
- [x] Basic / Pro / Enterprise form and entitlement policy implemented.
- [x] Production browser creation accepted.

## Release outcome

All four tasks are complete. The later user instruction expanded the original Enterprise key to all 19 engine scopes and access level 5. Staging API acceptance and actual production Cloudflare browser acceptance passed; full evidence and recovery pointers are in `docs/release/admin-api-key-tiers-2026-11-03.md`.
