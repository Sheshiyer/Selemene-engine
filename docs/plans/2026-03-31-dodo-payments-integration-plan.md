# Dodo Payments Integration Plan

Date: 2026-03-31
Repo: `Sheshiyer/Selemene-engine`
Primary surfaces:
- `crates/noesis-api`
- `crates/noesis-data`
- `crates/noesis-auth`
- `supabase/migrations`
- future authenticated user-facing billing surface

## Discovery Summary

- Planning depth: deeply detailed
- Delivery mode: production
- Release model: phased rollout
- CI/CD expectation: production-grade
- Quality bar:
  - hosted checkout and portal must use server-side Dodo API credentials only
  - internal entitlements must remain database-backed and auditable
  - webhook handling must be idempotent, replay-safe, and observable
  - plan/price/meter contracts must be frozen before parallel implementation
  - rollout must support test mode before live mode
- Team / agent topology:
  - planner / orchestrator: overall sequencing, contract freeze, review
  - backend / data: Dodo client, database schema, webhook processing, entitlement sync
  - UI / app: billing page, upgrade CTA, checkout launch, portal launch
  - validation: contract tests, webhook replay tests, smoke checks, rollout gates

## Inputs Reviewed

- Existing billing model:
  - `plan_catalog`
  - `billing_subscriptions`
  - `user_active_plan_resolutions`
- Existing auth/tier behavior:
  - JWT and API key flows still rely on `user.tier`
  - plan resolution exists in `noesis-data`, but entitlement reads are not yet fully Dodo-driven
- Existing billing integration state:
  - `crates/noesis-api/src/billing.rs` is still Stripe-shaped debug payload formatting, not a real provider integration
- Dodo Payments docs reviewed:
  - API introduction and API-key usage
  - checkout session creation
  - subscription integration guide
  - usage-based billing guide

## Assumptions And Constraints

- Dodo Payments is the target billing provider for paid subscriptions going forward.
- Hosted checkout and Dodo customer portal are acceptable; no custom PCI surface is desired.
- Internal authorization must not trust checkout redirects alone; webhooks and provider fetches remain the source of truth for billing state transitions.
- The current schema is close, but not sufficient for production Dodo sync:
  - it lacks first-class idempotent webhook event storage,
  - it likely needs a durable customer mapping outside the subscription row,
  - and its status vocabulary needs to align to Dodo lifecycle states.
- The repository currently exposes admin UI only. A user-facing billing surface may need to be added or attached to an existing authenticated user flow outside `apps/admin-web`.
- Existing tier semantics (`free`, `basic`, `premium`, `enterprise`) should be preserved as the canonical internal plan codes unless product decides otherwise.

## Contract Freeze

Before implementation branches diverge, freeze these contracts:

### Billing provider contract

- Provider identifier in local storage: `dodo_payments`
- Dodo secrets live server-side only:
  - `DODO_PAYMENTS_API_KEY`
  - `DODO_PAYMENTS_WEBHOOK_SECRET` or signing-key equivalent
  - `DODO_PAYMENTS_ENV` (`test` or `live`)
- All product, price, and meter identifiers are stored in database metadata, never hard-coded in UI.

### Internal data contract

- `plan_catalog.code` remains the canonical entitlement key used by product logic.
- `plan_catalog.metadata` stores Dodo mapping data:
  - `product_id`
  - `price_id`
  - `meter_id` where applicable
  - `billing_mode` (`flat` or `metered`)
- Add a dedicated customer mapping surface, preferably `billing_customers`, with:
  - `user_id`
  - `provider`
  - `provider_customer_id`
  - `email`
  - `mode`
  - timestamps
- Add a dedicated idempotency/audit surface, preferably `billing_webhook_events`, with:
  - provider event id
  - event type
  - payload snapshot
  - processed status
  - processed at
  - error metadata

### Subscription state contract

Freeze a canonical local status vocabulary before code splits:

- `trialing`
- `active`
- `past_due`
- `on_hold`
- `canceled`
- `expired`
- `incomplete`
- `failed`

If Dodo emits more granular states, map them explicitly at the adapter boundary.

### API boundary contract

Backend-owned endpoints:

- `POST /api/billing/checkout-session`
- `POST /api/billing/customer-portal-session`
- `POST /api/webhooks/dodo`
- `GET /api/billing/subscription`
- `GET /api/billing/plans`

UI never calls Dodo directly.

### Validation contract

Minimum required evidence before wave close:

- migration tests or schema assertions
- adapter unit tests for Dodo request/response mapping
- webhook signature verification tests
- idempotency/replay tests
- plan entitlement resolution tests
- end-to-end smoke for checkout session creation in test mode

## Agent Ownership Model

| Concern | Primary owner | Secondary reviewer | Notes |
|---|---|---|---|
| Planning / orchestration | Planner / orchestrator | Human lead | Owns phase boundaries and contract freeze |
| Dodo provider adapter | Backend / data | Planner / orchestrator | Owns HTTP client, event mapping, retries |
| Schema and persistence | Backend / data | Planner / orchestrator | Owns migrations, repository methods, idempotency |
| User-facing billing UI | UI / app | Planner / orchestrator | Owns billing page, upgrade CTA, portal handoff |
| Validation / release gates | Validation | Planner / orchestrator | Owns replay tests, smoke tests, rollout checklist |

## Phase Map

### Phase P1 — Contract And Foundation Freeze

Goal:
- freeze Dodo resource mapping, internal schema updates, endpoint contracts, and rollout boundaries

Exit criteria:
- Dodo plan matrix agreed
- local status mapping frozen
- endpoint shapes frozen
- migration set specified
- validation plan approved

### Phase P2 — Provider Plumbing And Persistence

Goal:
- add Dodo client integration, schema support, webhook ingestion, and authoritative subscription sync

Exit criteria:
- checkout and portal sessions can be created from backend
- webhook events are verified, persisted, replay-safe, and processed
- subscription/customer records sync cleanly into local tables

### Phase P3 — Product Surface And Entitlement Wiring

Goal:
- connect billing state to user-facing upgrade flows and internal entitlement resolution

Exit criteria:
- authenticated user can see plans, launch checkout, open portal, and see current subscription state
- auth and product gating resolve from canonical plan state

### Phase P4 — Metering, Hardening, And Launch

Goal:
- send usage signals when required, complete rollout verification, and prepare live launch

Exit criteria:
- metered flows are tested in Dodo test mode
- observability and replay tooling exist
- runbook and rollback plan are documented

## Detailed Phase 1 Wave Layout

### Wave W1 — Vendor And Plan Contract Freeze

#### Swarm S1 — Dodo resource mapping

- Goal:
  - define how `free`, `basic`, `premium`, and `enterprise` map to Dodo products, prices, and optional meters
- Owner:
  - backend / data
- Inputs:
  - current `plan_catalog`
  - Dodo dashboard product design
  - pricing strategy
- Outputs:
  - canonical plan mapping table
  - `plan_catalog.metadata` shape
- Validation:
  - mapping doc reviewed against Dodo objects in test mode

#### Swarm S2 — Internal entitlement contract

- Goal:
  - freeze local status vocabulary, active-plan resolution rules, and fallback behavior
- Owner:
  - planner / orchestrator
- Inputs:
  - `billing_subscriptions`
  - `user_active_plan_resolutions`
  - auth tier reads
- Outputs:
  - entitlement resolution spec
  - status mapping spec
- Validation:
  - repository query paths enumerated and approved

### Wave W2 — API And Webhook Boundary Freeze

#### Swarm S1 — Billing API surface

- Goal:
  - define the authenticated backend endpoints for plan listing, checkout, portal, and status reads
- Owner:
  - backend / data
- Inputs:
  - auth model
  - intended user journey
- Outputs:
  - request/response contracts
  - auth requirements
  - metadata and redirect contract
- Validation:
  - endpoint shapes reviewed against UI needs and Dodo API inputs

#### Swarm S2 — Webhook processing contract

- Goal:
  - define signature verification, event persistence, idempotency, and state transitions
- Owner:
  - validation
- Inputs:
  - Dodo webhook lifecycle
  - current subscription schema
- Outputs:
  - event handling matrix
  - replay behavior
  - dead-letter / manual recovery notes
- Validation:
  - event-type matrix approved before implementation

### Wave W3 — Execution Scaffold And Rollout Rules

#### Swarm S1 — Environment and secret discipline

- Goal:
  - define test/live env shape, local/dev behavior, and secret propagation
- Owner:
  - backend / data
- Inputs:
  - deployment environments
  - current env management
- Outputs:
  - env var contract
  - test-mode rollout plan
- Validation:
  - env matrix checked against deploy targets

#### Swarm S2 — Verification and launch guardrails

- Goal:
  - freeze the evidence needed for each later wave
- Owner:
  - validation
- Inputs:
  - CI shape
  - risk profile
- Outputs:
  - wave-level gates
  - smoke checklist
  - rollback expectations
- Validation:
  - gates attached to plan before parallel implementation begins

## Full Task List

The plan is intentionally granular so execution can be parallelized safely.

| ID | Phase | Wave | Swarm | Area | Owner | Est Hrs | Title | Dependencies | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---:|---|---|---|---|---|
| DODO-001 | P1 | W1 | S1 | product | planner | 1.5 | Inventory existing tier and entitlement surfaces | none | tier inventory doc | all tier reads identified | code search review attached |
| DODO-002 | P1 | W1 | S1 | product | planner | 1.0 | Inventory current billing schema and repo touchpoints | DODO-001 | schema touchpoint inventory | plan/data/auth surfaces mapped | inventory cross-checked against repo |
| DODO-003 | P1 | W1 | S1 | product | planner | 1.0 | Capture Dodo API capabilities relevant to hosted billing | none | vendor capability summary | checkout, portal, usage, webhook flows listed | doc citation review |
| DODO-004 | P1 | W1 | S1 | product | planner | 1.0 | Define canonical internal plan codes | DODO-001,DODO-003 | plan code contract | free/basic/premium/enterprise confirmed | product review note |
| DODO-005 | P1 | W1 | S1 | product | planner | 1.5 | Map each internal plan to Dodo product and price IDs | DODO-004 | plan mapping matrix | every paid plan has test-mode Dodo objects | dashboard verification |
| DODO-006 | P1 | W1 | S1 | product | planner | 1.0 | Decide which plans are flat-rate vs metered | DODO-005 | billing mode matrix | each plan tagged with billing mode | reviewed against Dodo setup |
| DODO-007 | P1 | W1 | S1 | data | backend | 1.0 | Define `plan_catalog.metadata` schema for Dodo identifiers | DODO-005,DODO-006 | metadata contract | metadata fields frozen | schema review |
| DODO-008 | P1 | W1 | S2 | data | backend | 1.0 | Define canonical subscription status vocabulary | DODO-002,DODO-003 | status contract | local statuses frozen | review note attached |
| DODO-009 | P1 | W1 | S2 | data | backend | 1.0 | Map Dodo lifecycle events/statuses into local statuses | DODO-008 | mapping table | every relevant Dodo state mapped | event matrix review |
| DODO-010 | P1 | W1 | S2 | data | backend | 1.0 | Define active-plan resolution precedence rules | DODO-008,DODO-009 | entitlement rules | ambiguous multi-subscription behavior specified | repository query review |
| DODO-011 | P1 | W1 | S2 | data | backend | 1.0 | Define fallback behavior when webhook lag exists | DODO-010 | consistency policy | no entitlement path depends only on redirect | failure-mode review |
| DODO-012 | P1 | W1 | S2 | product | planner | 1.0 | Freeze customer identity strategy between users and Dodo customers | DODO-003,DODO-010 | customer mapping spec | customer creation and reuse rules documented | review signoff |
| DODO-013 | P1 | W2 | S1 | backend | backend | 1.0 | Define `GET /api/billing/plans` contract | DODO-007 | endpoint contract | plan listing shape frozen | request/response doc review |
| DODO-014 | P1 | W2 | S1 | backend | backend | 1.0 | Define `GET /api/billing/subscription` contract | DODO-010,DODO-013 | endpoint contract | current subscription shape frozen | contract review |
| DODO-015 | P1 | W2 | S1 | backend | backend | 1.5 | Define `POST /api/billing/checkout-session` contract | DODO-005,DODO-012 | endpoint contract | input/output and auth defined | UI/backend review |
| DODO-016 | P1 | W2 | S1 | backend | backend | 1.0 | Define redirect and metadata contract for checkout launches | DODO-015 | redirect contract | success/cancel/return behavior frozen | threat-model review |
| DODO-017 | P1 | W2 | S1 | backend | backend | 1.0 | Define `POST /api/billing/customer-portal-session` contract | DODO-012 | endpoint contract | portal launch contract frozen | contract review |
| DODO-018 | P1 | W2 | S1 | backend | backend | 1.0 | Define API auth rules for all billing endpoints | DODO-013,DODO-014,DODO-015,DODO-017 | auth matrix | auth scope is explicit | security review |
| DODO-019 | P1 | W2 | S2 | backend | backend | 1.5 | Enumerate webhook event types required for MVP | DODO-003,DODO-009 | event matrix | subscription and payment events selected | docs cross-check |
| DODO-020 | P1 | W2 | S2 | backend | backend | 1.5 | Define webhook signature verification contract | DODO-019 | verification spec | secret/header/body requirements frozen | security review |
| DODO-021 | P1 | W2 | S2 | data | backend | 1.0 | Define idempotent event storage model | DODO-019,DODO-020 | event storage spec | duplicate and replay behavior specified | replay scenario review |
| DODO-022 | P1 | W2 | S2 | data | backend | 1.0 | Define event-to-state transition matrix | DODO-009,DODO-019 | transition matrix | every accepted event has one state action | transition review |
| DODO-023 | P1 | W2 | S2 | qa | validation | 1.0 | Define webhook failure and retry policy | DODO-021,DODO-022 | failure policy | manual recovery path exists | ops review |
| DODO-024 | P1 | W2 | S2 | qa | validation | 1.0 | Define audit logging requirements for billing changes | DODO-021,DODO-022 | audit contract | subscription mutations are traceable | logging review |
| DODO-025 | P1 | W3 | S1 | infra | backend | 1.0 | Define environment variable contract for Dodo integration | DODO-015,DODO-020 | env matrix | test/live vars and required secrets listed | deploy review |
| DODO-026 | P1 | W3 | S1 | infra | backend | 1.0 | Define local development behavior for Dodo test mode | DODO-025 | local-dev spec | developers can exercise checkout and webhook flow | runbook review |
| DODO-027 | P1 | W3 | S1 | infra | backend | 1.0 | Define deployment target and webhook endpoint exposure rules | DODO-025 | deploy ingress rules | public webhook route ownership clear | platform review |
| DODO-028 | P1 | W3 | S1 | infra | backend | 1.0 | Define secret rotation and revocation procedure | DODO-020,DODO-025 | secret ops note | key rotation path documented | security review |
| DODO-029 | P1 | W3 | S2 | qa | validation | 1.0 | Define unit-test baseline for billing adapter work | DODO-015,DODO-022 | unit test plan | adapter test coverage expectations frozen | validation review |
| DODO-030 | P1 | W3 | S2 | qa | validation | 1.0 | Define integration-test baseline for webhook processing | DODO-021,DODO-022 | integration test plan | replay and state-sync tests specified | validation review |
| DODO-031 | P1 | W3 | S2 | qa | validation | 1.0 | Define smoke-test baseline for checkout and portal launch | DODO-015,DODO-017 | smoke checklist | manual/automated smoke steps exist | checklist review |
| DODO-032 | P1 | W3 | S2 | qa | validation | 1.0 | Define rollback decision gates for launch | DODO-023,DODO-031 | rollback gate doc | rollback triggers and owners explicit | ops review |
| DODO-033 | P2 | W1 | S1 | data | backend | 2.0 | Add migration for `plan_catalog.metadata` Dodo fields if needed | DODO-007 | migration | metadata supports Dodo mapping | migration test |
| DODO-034 | P2 | W1 | S1 | data | backend | 3.0 | Add `billing_customers` table and indexes | DODO-012 | migration | one customer mapping per user/provider supported | migration test |
| DODO-035 | P2 | W1 | S1 | data | backend | 3.0 | Add `billing_webhook_events` table and indexes | DODO-021 | migration | webhook events can be persisted idempotently | migration test |
| DODO-036 | P2 | W1 | S1 | data | backend | 2.0 | Extend `billing_subscriptions` status constraint for Dodo lifecycle | DODO-008,DODO-009 | migration | local schema matches frozen status contract | migration test |
| DODO-037 | P2 | W1 | S1 | data | backend | 2.0 | Add provider metadata columns needed for Dodo sync | DODO-034,DODO-036 | migration | subscription rows can store Dodo identifiers cleanly | migration test |
| DODO-038 | P2 | W1 | S2 | backend | backend | 3.0 | Introduce Dodo API client module in `noesis-api` | DODO-015,DODO-025 | provider client | authenticated API wrapper exists | unit tests |
| DODO-039 | P2 | W1 | S2 | backend | backend | 2.0 | Add config loading for Dodo env and secrets | DODO-025,DODO-038 | config wiring | runtime loads Dodo config safely | config tests |
| DODO-040 | P2 | W1 | S2 | backend | backend | 2.0 | Add customer upsert helper using Dodo and local mapping | DODO-034,DODO-038,DODO-039 | customer sync helper | users map deterministically to Dodo customers | unit tests |
| DODO-041 | P2 | W2 | S1 | backend | backend | 2.0 | Implement `GET /api/billing/plans` endpoint | DODO-013,DODO-033,DODO-039 | plans endpoint | plans can be listed from canonical catalog | endpoint tests |
| DODO-042 | P2 | W2 | S1 | backend | backend | 2.0 | Implement `GET /api/billing/subscription` endpoint | DODO-014,DODO-037 | subscription endpoint | current subscription state resolves cleanly | endpoint tests |
| DODO-043 | P2 | W2 | S1 | backend | backend | 3.0 | Implement checkout session creation endpoint | DODO-015,DODO-016,DODO-038,DODO-040 | checkout endpoint | authenticated user receives Dodo checkout URL | endpoint tests |
| DODO-044 | P2 | W2 | S1 | backend | backend | 2.0 | Implement customer portal session endpoint | DODO-017,DODO-038,DODO-040 | portal endpoint | authenticated user receives portal URL | endpoint tests |
| DODO-045 | P2 | W2 | S1 | backend | backend | 2.0 | Attach billing metadata to checkout session creation | DODO-016,DODO-043 | metadata propagation | checkout session carries internal correlation data | endpoint tests |
| DODO-046 | P2 | W2 | S2 | backend | backend | 2.0 | Implement webhook route skeleton | DODO-019,DODO-020,DODO-039 | webhook endpoint | route accepts raw payload and headers | smoke test |
| DODO-047 | P2 | W2 | S2 | backend | backend | 2.5 | Implement webhook signature verification | DODO-020,DODO-046 | verifier | invalid signatures are rejected | verifier tests |
| DODO-048 | P2 | W2 | S2 | data | backend | 2.5 | Persist webhook events before mutation | DODO-035,DODO-046,DODO-047 | event persistence path | each accepted event is durably stored | integration test |
| DODO-049 | P2 | W2 | S2 | data | backend | 2.5 | Add idempotent duplicate-event short circuit | DODO-048 | idempotency guard | repeated event IDs do not reapply mutations | replay test |
| DODO-050 | P2 | W2 | S2 | data | backend | 3.0 | Implement subscription state projection from webhook events | DODO-022,DODO-037,DODO-048,DODO-049 | state projector | local subscription rows track Dodo lifecycle | integration test |
| DODO-051 | P2 | W3 | S1 | data | backend | 2.0 | Update repositories to resolve plan from canonical subscription rows | DODO-010,DODO-050 | repository updates | entitlement reads no longer depend on manual tier drift | repository tests |
| DODO-052 | P2 | W3 | S1 | backend | backend | 2.0 | Replace Stripe-shaped billing emitter with provider-neutral usage abstraction | DODO-003,DODO-051 | billing abstraction | no Stripe-specific naming remains in runtime path | unit tests |
| DODO-053 | P2 | W3 | S1 | backend | backend | 2.0 | Add admin/debug endpoints or logs for billing state inspection | DODO-048,DODO-050 | inspection surface | operators can inspect customer/subscription/event state | smoke test |
| DODO-054 | P2 | W3 | S2 | qa | validation | 2.0 | Add webhook replay integration tests | DODO-049,DODO-050 | replay tests | duplicate and out-of-order cases are covered | CI pass |
| DODO-055 | P2 | W3 | S2 | qa | validation | 2.0 | Add checkout-session contract tests | DODO-043,DODO-045 | contract tests | session payloads match frozen contract | CI pass |
| DODO-056 | P2 | W3 | S2 | qa | validation | 1.5 | Add portal-session contract tests | DODO-044 | contract tests | portal launch contract is stable | CI pass |
| DODO-057 | P3 | W1 | S1 | frontend | ui/app | 2.0 | Design minimal authenticated billing information architecture | DODO-013,DODO-014,DODO-031 | billing IA note | user journey from upgrade CTA to portal is explicit | design review |
| DODO-058 | P3 | W1 | S1 | frontend | ui/app | 3.0 | Create billing page shell for current plan and available upgrades | DODO-057,DODO-041,DODO-042 | billing page shell | page can render plan catalog and current plan state | UI smoke test |
| DODO-059 | P3 | W1 | S1 | frontend | ui/app | 2.0 | Add upgrade CTA entry points from gated product surfaces | DODO-057,DODO-058 | upgrade entry points | paid features route user into billing flow | UI smoke test |
| DODO-060 | P3 | W1 | S2 | frontend | ui/app | 2.0 | Wire checkout launch from UI to backend endpoint | DODO-043,DODO-058 | checkout launch flow | user can open Dodo checkout from billing page | browser smoke test |
| DODO-061 | P3 | W1 | S2 | frontend | ui/app | 2.0 | Wire customer portal launch from UI to backend endpoint | DODO-044,DODO-058 | portal launch flow | user can open Dodo portal from billing page | browser smoke test |
| DODO-062 | P3 | W1 | S2 | frontend | ui/app | 1.5 | Add pending/success/error states for billing actions | DODO-060,DODO-061 | billing action states | failure/success UX is explicit | UI state review |
| DODO-063 | P3 | W2 | S1 | backend | backend | 2.5 | Update auth/session creation to prefer resolved active plan over legacy tier when safe | DODO-051 | auth entitlement patch | JWT/session tier comes from canonical resolution path | auth tests |
| DODO-064 | P3 | W2 | S1 | backend | backend | 2.0 | Add compatibility layer to keep legacy `user.tier` in sync during migration window | DODO-051,DODO-063 | compatibility sync | old readers still behave during rollout | integration test |
| DODO-065 | P3 | W2 | S1 | data | backend | 2.0 | Add backfill routine for existing paid users into Dodo-aware metadata model | DODO-033,DODO-034,DODO-037 | backfill script/runbook | existing users can be migrated without losing access | dry-run evidence |
| DODO-066 | P3 | W2 | S2 | qa | validation | 2.0 | Add end-to-end entitlement tests for free vs paid access | DODO-059,DODO-063,DODO-064 | entitlement e2e tests | feature gating matches subscription state | CI pass |
| DODO-067 | P3 | W2 | S2 | qa | validation | 1.5 | Add browser-level smoke for checkout and portal redirects | DODO-060,DODO-061 | browser smoke suite | launch URLs behave in test mode | smoke evidence |
| DODO-068 | P3 | W3 | S1 | frontend | ui/app | 2.0 | Add billing history/status messaging sourced from local records | DODO-050,DODO-058 | billing status UI | user can see meaningful billing state | UI review |
| DODO-069 | P3 | W3 | S1 | frontend | ui/app | 1.5 | Add support/help copy for billing failures and plan changes | DODO-062,DODO-068 | help copy | support path is clear for failed/on-hold states | copy review |
| DODO-070 | P3 | W3 | S2 | docs | planner | 1.5 | Document internal billing architecture and runbooks | DODO-053,DODO-064 | architecture doc | future maintainers can operate the system | doc review |
| DODO-071 | P4 | W1 | S1 | backend | backend | 2.5 | Design provider-neutral usage event payload for metered plans | DODO-006,DODO-052 | usage event contract | event shape supports Dodo ingest needs | contract review |
| DODO-072 | P4 | W1 | S1 | backend | backend | 3.0 | Implement usage-event queue or send path to Dodo for metered plans | DODO-071 | metering sender | metered usage can be reported from runtime | integration test |
| DODO-073 | P4 | W1 | S1 | data | backend | 2.0 | Persist outbound usage-event attempts and failures | DODO-072 | usage audit log | metering failures are traceable | integration test |
| DODO-074 | P4 | W1 | S2 | qa | validation | 2.0 | Add metered usage tests against provider adapter layer | DODO-072,DODO-073 | usage adapter tests | quantity aggregation and error handling are covered | CI pass |
| DODO-075 | P4 | W1 | S2 | qa | validation | 1.5 | Add quota-vs-billing boundary tests | DODO-052,DODO-072,DODO-074 | boundary tests | runtime quota behavior matches billing design | CI pass |
| DODO-076 | P4 | W2 | S1 | infra | backend | 1.5 | Add observability for billing endpoint latency and webhook failures | DODO-043,DODO-047,DODO-072 | metrics/logging | operators can detect billing regressions | metrics evidence |
| DODO-077 | P4 | W2 | S1 | infra | backend | 1.5 | Add alert thresholds for webhook verification or projection failures | DODO-076 | alerting config | critical billing failures page on-call path | alert review |
| DODO-078 | P4 | W2 | S1 | docs | planner | 1.5 | Write Dodo test-mode verification runbook | DODO-067,DODO-074,DODO-076 | runbook | test-mode acceptance is repeatable | dry run |
| DODO-079 | P4 | W2 | S2 | qa | validation | 2.0 | Execute full test-mode subscription lifecycle rehearsal | DODO-050,DODO-067,DODO-078 | rehearsal evidence | create, pay, renew, cancel, fail flows are proven | rehearsal report |
| DODO-080 | P4 | W2 | S2 | qa | validation | 1.5 | Execute replay and rollback drills | DODO-054,DODO-078 | drill evidence | rollback and replay paths are proven | drill report |
| DODO-081 | P4 | W3 | S1 | docs | planner | 1.0 | Write live-launch checklist and freeze window plan | DODO-079,DODO-080 | launch checklist | launch gate is explicit | review signoff |
| DODO-082 | P4 | W3 | S1 | infra | backend | 1.0 | Prepare live-mode secret and webhook cutover checklist | DODO-028,DODO-081 | cutover checklist | secrets and endpoints can be switched safely | ops review |
| DODO-083 | P4 | W3 | S2 | qa | validation | 1.5 | Run post-cutover smoke checks | DODO-082 | smoke evidence | live checkout and portal basic flows are healthy | smoke report |
| DODO-084 | P4 | W3 | S2 | docs | planner | 1.0 | Capture residual risks and follow-on billing roadmap | DODO-083 | launch review note | known gaps are documented with owners | review signoff |

## Dependency Rationale

- P1 must finish before parallel implementation because the largest risk is contract drift:
  - plan codes,
  - status mapping,
  - endpoint shapes,
  - and event semantics.
- P2 is the backend/data foundation and should land before heavy UI work.
- P3 can split safely once checkout/portal contracts and subscription reads are stable.
- P4 depends on P2 for provider plumbing and on P3 for end-to-end billing UX.

Serialized lock zones:

- `crates/noesis-api/src/lib.rs`
- `crates/noesis-api/src/billing.rs`
- shared config/env loading
- `crates/noesis-data/src/repositories/user_repository.rs`
- `crates/noesis-data/src/repositories/admin_repository.rs`
- `supabase/migrations/*billing*`

Parallel-safe split after P1:

- backend adapter + migrations
- UI billing surface
- validation harness

## Verification Strategy

Per-wave proof:

- P1:
  - written contract docs reviewed
  - task boundaries frozen
- P2:
  - migration tests
  - endpoint tests
  - webhook signature and replay tests
- P3:
  - entitlement regression tests
  - browser smoke for checkout and portal launch
- P4:
  - metering tests
  - rehearsal evidence
  - live smoke and rollback drills

Required command-level validation before calling implementation done:

- targeted `cargo test` for `noesis-api`, `noesis-data`, and `noesis-auth`
- migration verification against a disposable Postgres instance
- manual or automated Dodo test-mode smoke
- log review for webhook ingest and subscription projection

## GitHub Sync And Dispatch Strategy

- one issue per task ID
- titles encoded as:
  - `[P?][W?][S?] DODO-### — task title`
- labels:
  - `billing`
  - `dodo-payments`
  - `phase:p1|p2|p3|p4`
  - `area:backend|data|frontend|infra|qa|product`
- branch/worktree naming:
  - `feat/dodo-0xx-short-slug`
- no overlapping ownership inside a wave unless explicitly marked as integration work

## Worker Bootstrap Packet Strategy

If this plan is executed with multiple agents, use this default split:

- Planner / orchestrator
  - owns P1 contract freeze
  - owns issue graph
  - owns integration gate review
- Backend / data worker
  - owns migrations
  - Dodo client
  - webhook pipeline
  - entitlement sync
- UI / app worker
  - owns billing page
  - upgrade CTA
  - portal/checkout launch UX
- Validation worker
  - owns replay harness
  - smoke tests
  - entitlement regressions
  - launch evidence

Fresh worker packets must include:

- this plan
- the frozen endpoint contract
- the status mapping table
- the env var contract
- the validation gates relevant to the owned tasks

## Risks And Fallback Plan

- Risk: Dodo lifecycle events do not map cleanly onto the current local status schema.
  - Trigger: webhook event handling needs ad hoc conditional logic.
  - Fallback: widen local canonical status set once, then freeze again before more code lands.

- Risk: existing product logic still reads `user.tier` directly in too many places.
  - Trigger: subscription changes do not update access consistently.
  - Fallback: add a temporary compatibility sync layer while migrating entitlement reads.

- Risk: metered billing adds more complexity than the first release needs.
  - Trigger: usage-event contract remains unstable after P2.
  - Fallback: ship flat-rate subscriptions first, then unlock P4 metering separately.

- Risk: there is no ready user-facing app surface for self-service billing.
  - Trigger: no authenticated customer UI is available to launch checkout and portal flows.
  - Fallback: ship backend + webhook foundation first and expose an interim billing route in the nearest authenticated surface.

- Risk: webhook delivery or signature verification differs between local assumptions and real Dodo behavior.
  - Trigger: test-mode events cannot be replayed reliably.
  - Fallback: keep event persistence and manual reconciliation tooling in scope before live cutover.
