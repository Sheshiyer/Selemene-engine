---
task: Implement Cloudflare auth and retire Supabase login
slug: 2026-07-02-cf-auth-supabase-retirement-tasks-5-12
project: selemene-supabase-retirement
effort: e3
effort_source: explicit
phase: verify
progress: 0/36
mode: interactive
started: 2026-07-02T00:00:00Z
updated: 2026-07-02T00:00:00Z
---

## Problem

The Supabase retirement plan requires replacing human/admin authentication with Cloudflare Zero Trust while keeping internal machine carve-outs intact. Tasks 5-12 are unimplemented: Cloudflare auth types/config, JWT validation, JIT user resolution, role sync, middleware wiring, Discord OAuth route retirement, password login deprecation, and admin-web CF login simplification.

## Vision

The Rust API accepts Cloudflare Access identity headers, resolves human admins to local users on first access, synchronizes CF groups to `user_roles`, and falls back to a secure dev bypass only in development. Discord OAuth and password login are retired from the API surface; the admin web app shows a single Cloudflare Access continuation screen.

## Out of Scope

- Database migration script changes (Task 1-4 already delivered).
- Raga payload alignment (Task 13) and R2/Suno engine work (Tasks 14-15).
- Removing the `oauth.rs` handler file entirely in this task; only its route registration is retired.
- Changing internal carve-out routes `/internal/billing/events` and `/internal/raga/clip`.
- Live Cloudflare JWT verification against real JWKS endpoints in unit tests.

## Principles

- Human auth has one source of truth: Cloudflare Access.
- Internal machine routes remain narrow shared-secret carve-outs.
- Legacy JWT/API-key validation stays available after CF checks until explicitly removed.
- Dev bypass is explicit, token-gated, and production-disabled.

## Constraints

- Work only in `/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/selemene-supabase-retirement`.
- Do not modify `scripts/supabase-to-railway-migrate.sh`, TS engine scripts, or docs.
- Keep legacy JWT and API key validation after CF checks in middleware.
- `selemene-admin` CF group maps to `platform-admin`; other groups map directly.
- Commit after each passing task.

## Goal

Implement Tasks 5-12 of the Supabase retirement plan in the target workspace, verify each with the exact commands in the plan, and commit after each passing task, leaving internal machine routes and legacy auth fallbacks intact.

## Criteria

- [ ] ISC-1: Task 5 Step 2 — `crates/noesis-api/src/cf_access.rs` exists with `CfIdentity`, `map_cf_group_to_role`, `roles_from_cf_groups`, and `development_auth_user`.
- [ ] ISC-2: Task 5 Step 4 — `cargo test -p noesis-api cf_access -- --nocapture` passes.
- [ ] ISC-3: Task 5 Step 3 — `ApiConfig` includes `cf_access_issuer`, `cf_access_audience`, and `cf_dev_bypass_token`.
- [ ] ISC-4: Task 5 Step 3 — `pub mod cf_access;` is declared in `crates/noesis-api/src/lib.rs`.
- [ ] ISC-5: Task 6 Step 1 — `CfAccessClaims` and `identity_from_claims` exist in `cf_access.rs`.
- [ ] ISC-6: Task 6 Step 4 — `cargo test -p noesis-api cf_access -- --nocapture` passes (claim extraction tests).
- [ ] ISC-7: Task 6 Step 3 — `jsonwebtoken = "9.3"` dependency exists in `crates/noesis-api/Cargo.toml`.
- [ ] ISC-8: Task 6 Step 3 — `CfAccessValidator` stub exists with `new` and `validate_token`.
- [ ] ISC-9: Task 7 Step 3 — `auth_user_from_parts` exists and builds `AuthUser` from local user + roles.
- [ ] ISC-10: Task 7 Step 4 — `cargo test -p noesis-api auth_user_from_parts -- --nocapture` passes.
- [ ] ISC-11: Task 7 Step 3 — `UserRepository::find_or_create_cloudflare_user` exists in `crates/noesis-data/src/repositories/user_repository.rs`.
- [ ] ISC-12: Task 7 Step 4 — `cargo check -p noesis-data` passes.
- [ ] ISC-13: Task 8 Step 3 — `role_values_for_sql` is deterministic and uses `roles_from_cf_groups`.
- [ ] ISC-14: Task 8 Step 4 — `cargo test -p noesis-api role_values_for_sql -- --nocapture` passes.
- [ ] ISC-15: Task 8 Step 3 — `AdminRepository::replace_user_roles_from_cloudflare` exists.
- [ ] ISC-16: Task 8 Step 4 — `cargo check -p noesis-data` passes.
- [ ] ISC-17: Task 9 Step 3 — `AppState` has `cf_access_validator` and `cf_dev_bypass_token` fields.
- [ ] ISC-18: Task 9 Step 3 — `auth_middleware` signature uses `State<AppState>` and runs dev bypass + CF checks before legacy JWT/API-key.
- [ ] ISC-19: Task 9 Step 4 — `cargo check -p noesis-api` passes.
- [ ] ISC-20: Task 9 Step 4 — `cargo test -p noesis-api cf_auth_tests -- --nocapture` passes.
- [ ] ISC-21: Task 10 Step 1 — Route inventory test no longer expects `/api/v1/auth/discord/authorize` and `/api/v1/auth/discord/callback`.
- [ ] ISC-22: Task 10 Step 4 — `cargo test -p noesis-api --test route_inventory_tests -- --nocapture` passes.
- [ ] ISC-23: Task 10 Step 3 — Discord OAuth routes are removed from `lib.rs` route registration.
- [ ] ISC-24: Task 10 Step 4 — `cargo check -p noesis-api` passes.
- [ ] ISC-25: Task 11 Step 1 — `user_management_tests.rs` expects `410 GONE` with `AUTH_RETIRED` for login.
- [ ] ISC-26: Task 11 Step 4 — `cargo test -p noesis-api --test user_management_tests login -- --nocapture` passes.
- [ ] ISC-27: Task 11 Step 3 — `login` handler returns `410 GONE` with `AUTH_RETIRED`.
- [ ] ISC-28: Task 11 Step 4 — `cargo check -p noesis-api` passes.
- [ ] ISC-29: Task 12 Step 1 — `pnpm --dir apps/admin-web typecheck` baseline recorded.
- [ ] ISC-30: Task 12 Step 2 — `getDiscordAuthUrl` and `discordCallback` removed from `apps/admin-web/src/lib/api.ts`.
- [ ] ISC-31: Task 12 Step 3 — `LoginClient` simplified to Cloudflare Access message with continue link.
- [ ] ISC-32: Task 12 Step 4 — Discord helper and callback UI files deleted.
- [ ] ISC-33: Task 12 Step 5 — `pnpm --dir apps/admin-web typecheck` passes.
- [ ] ISC-34: Task 12 Step 5 — `pnpm --dir apps/admin-web lint` passes.
- [ ] ISC-35: Anti: legacy JWT/API-key validation is removed or bypassed before CF checks.
- [ ] ISC-36: Anti: internal carve-out routes `/internal/billing/events` or `/internal/raga/clip` are modified to require CF auth.

## Test Strategy

```yaml
- isc: ISC-2, ISC-6, ISC-10, ISC-14, ISC-20
  type: cargo unit/integration test
  check: targeted test command from plan
  threshold: exit 0
  tool: cargo test -p noesis-api ...

- isc: ISC-12, ISC-16, ISC-19, ISC-24, ISC-28
  type: compile check
  check: crate compiles without errors
  threshold: exit 0
  tool: cargo check -p noesis-api / cargo check -p noesis-data

- isc: ISC-22
  type: integration test
  check: route inventory excludes retired Discord OAuth routes
  threshold: exit 0
  tool: cargo test -p noesis-api --test route_inventory_tests

- isc: ISC-26
  type: integration test
  check: login returns 410 AUTH_RETIRED
  threshold: exit 0
  tool: cargo test -p noesis-api --test user_management_tests login

- isc: ISC-33, ISC-34
  type: typecheck/lint
  check: admin-web typechecks and lints cleanly
  threshold: exit 0
  tool: pnpm --dir apps/admin-web typecheck / lint
```

## Features

```yaml
- name: CfAuthPrimitives
  description: Cloudflare Access types, role mapping, dev bypass, and config fields.
  satisfies: [ISC-1, ISC-2, ISC-3, ISC-4]
  depends_on: []
  parallelizable: false

- name: CfJwtValidation
  description: Claims parsing, identity extraction, and validator stub with jsonwebtoken dependency.
  satisfies: [ISC-5, ISC-6, ISC-7, ISC-8]
  depends_on: [CfAuthPrimitives]
  parallelizable: false

- name: CfJitUserResolution
  description: Repository method to find or create local users from CF identity and AuthUser builder.
  satisfies: [ISC-9, ISC-10, ISC-11, ISC-12]
  depends_on: [CfJwtValidation]
  parallelizable: false

- name: CfRoleSync
  description: Deterministic role values and repository method to replace user_roles from CF groups.
  satisfies: [ISC-13, ISC-14, ISC-15, ISC-16]
  depends_on: [CfJitUserResolution]
  parallelizable: false

- name: CfMiddleware
  description: Wire dev bypass and CF token validation into auth middleware before legacy checks.
  satisfies: [ISC-17, ISC-18, ISC-19, ISC-20, ISC-35, ISC-36]
  depends_on: [CfRoleSync]
  parallelizable: false

- name: RetireDiscordOAuth
  description: Remove Discord OAuth API route registrations and update route inventory test.
  satisfies: [ISC-21, ISC-22, ISC-23, ISC-24]
  depends_on: [CfMiddleware]
  parallelizable: false

- name: DeprecatePasswordLogin
  description: Return 410 AUTH_RETIRED from password login and update tests.
  satisfies: [ISC-25, ISC-26, ISC-27, ISC-28]
  depends_on: [RetireDiscordOAuth]
  parallelizable: false

- name: AdminWebCfLogin
  description: Simplify admin login UI to Cloudflare Access message and delete Discord callback UI.
  satisfies: [ISC-29, ISC-30, ISC-31, ISC-32, ISC-33, ISC-34]
  depends_on: [DeprecatePasswordLogin]
  parallelizable: false
```

## Decisions

- 2026-07-02: Stopped at Task 5 Step 2 because pre-existing uncommitted changes in `crates/noesis-api/src/handlers/raga.rs` (Task 13 in-progress tests) prevent `noesis-api` lib tests from compiling. `cargo check -p noesis-api` passes, confirming the non-test library code is fine. However, every `cargo test -p noesis-api ...` command required by Tasks 5-12 compiles the lib test cfg and fails. Per user constraint "If compilation or tests fail, stop and report the exact error", execution is halted pending resolution.

## Changelog

## Verification

- ISC-1/ISC-4 partial: `cargo check -p noesis-api` passes with only expected `unused_variable` warnings from the `todo!` stubs in `cf_access.rs`.
- Blocker: `cargo test -p noesis-api cf_access -- --nocapture` fails to compile because `crates/noesis-api/src/handlers/raga.rs:270-271` references `suno_prompt` and `r2_key` fields that do not exist on `UpsertRagaClipRequest`.
