# Shared identity API v1

Root migrations 045 and 046 are additive. Apply with the existing journaled migration runner after comparing the deployed ledger. No personal seeds are included. Existing `/desktop/*` routes retain their original native loopback contract.

## Registered sessions

Configure `NOESIS_DAILY_REDIRECT_URIS` and `NOESIS_RAYCAST_REDIRECT_URIS` as comma-separated **exact** HTTPS redirects. Daily paths must be `/auth/callback` without query or fragment. Raycast's emitted callback (including `packageName` query) must match exactly. `antahkarana` alone permits `http://127.0.0.1:<1024–65535>/callback`.

Configure `CF_END_USER_ISSUER` and `CF_END_USER_AUDIENCE` for a separate end-user Access application. Only shared-account routes consult that validator, and its identities receive viewer permissions without changing administrative roles. Administrative Access audience configuration is unchanged. When dedicated configuration is absent, owner Access remains the fallback; this does not make enrollment available to other people. The edge must pass the verified Access assertion on `/sessions/authorize` and `/account/email-links/verify`, and permit PKCE exchanges and refresh/revoke POSTs without imposing an owner-only Access challenge.

- `GET /api/v1/sessions/authorize`: `client_id`, exact `redirect_uri`, `state` (16–128 unreserved characters), `code_challenge`, `code_challenge_method=S256`. Fresh verified CF identity required; API keys, password JWTs and development bypass are insufficient.
- `POST /api/v1/sessions/token`: authorization-code or refresh grant matching HTTP-CONTRACT v1. Tokens are client-bound, code digests single-use, refresh families locked through successor insertion. Reuse revokes the family. Every registered access JWT rechecks current family/account/provider state server-side.
- `POST /api/v1/sessions/revoke`: `{client_id,refresh_token}`. Idempotent; immediately invalidates the matching access/refresh family.

## Context and profiles

`GET /api/v1/users/me/context` returns `noesis.account-context.v1`. Subject fields include the additive `can_calculate` permission. Read, edit and calculation are independent; explicit confirmed self enrollment grants all three. A resource owner or platform role alone grants no read/edit/calculation access. Relationship grants independently expose only relationship metadata, never an individual profile.

`PATCH /api/v1/users/me/context` accepts `{default_subject_id:uuid|null}` and checks the current read grant. Preferred email is only returned if backed by a live verified identity.

`GET /api/v1/subjects/:id/profile` returns the immutable current accepted revision. A subject with no admitted revision returns `PROFILE_NOT_ADMITTED`; revision 0 in context means no accepted profile. `PATCH` accepts `{expected_revision,patch}`; revision 0 creates the initial profile and a stale value returns 409. Whitelisted values are full_name, birth_date, birth_time, birth_location, timezone, preferences, birth_time_confidence and source_hashes. Email and authority fields are rejected. Updates to an explicitly linked self also project the accepted defaults into legacy `user_profiles` in the same transaction. Historical revisions remain immutable.

`POST /api/v1/subjects/:id/engines/:engine/calculate` accepts `{expected_revision,options?,current_time?}`. It requires `can_calculate` in addition to read. Birth inputs come from the accepted revision; caller auth context and birth overrides are rejected. Existing orchestrator phase, scope, input, quota and consent boundaries apply. The reader grant is checked again before storing/returning the result. The result preserves the normal provenance envelope, and persisted rows record actor account and subject separately. This path never auto-populates the actor's defaults.

`GET /api/v1/subjects/:id/readings` returns the latest 100 run descriptors. `GET /api/v1/subjects/:id/readings/:reading` returns the scoped result. Both recheck current grants; no permanent object links are returned.

## Enrollment

A fresh CF identity or a current registered human session is required to create, claim or confirm enrollment. Registered identity is recovered through the live access JTI, family and provider link; it never grants interactive/browser authority. Password JWTs and API keys do not qualify.

- `POST /account/enrollments`: `{subject_id,intended_email?,sharing_reader_id?,expires_in_seconds?,relationship_ids?}`. Only that corpus resource owner can create; default TTL one day, maximum seven days. Returns `{id,token,subject_id,sharing_reader_id,expires_in}` once. Store the returned token privately; PostgreSQL stores only its digest.
- `POST /account/enrollments/preview`: `{token}` returns the selected subject name, disclosed sharing reader and pinned `relationships:[{id,label}]`. Requires authentication. The recipient must see this before acceptance.
- `POST /account/enrollments/claim`: `{token,accept_sharing:true}`. A pre-bound intended email must match fresh verified proof. Unknown emails consume the claim and return `pending_owner_confirmation`, granting no profile access.
- `GET /account/enrollments`: owner's pending claims and verified claim email, for explicit confirmation.
- `POST /account/enrollments/:id/confirm`: owner confirms an unexpired live pending claim. Confirmation atomically grants read to only the pinned relationships after rechecking they are active. Creation requires each relationship to be owned by the invitation owner and contain the selected subject; composite references preserve that exact membership. Duplicate subject/self mappings fail; names and birth dates are never account selectors.
- `DELETE /account/enrollments/:id`: owner revokes an unconfirmed enrollment. Accepted grants are separately revoked through the owner-managed grant records.

All account paths above have prefix `/api/v1`.

## Preferred email verification

`POST /api/v1/account/email-links` accepts `{email}` only from fresh CF proof or a live registered human session. Password JWTs/API keys do not establish verified email authority. Returns `{status:"pending_verification",challenge,expires_in:600,complete_path,verification_path}`. Open `verification_path` at the trusted API origin while signed in to the initiating account first. The challenge is high entropy state bound to the existing account and exact requested address.

`verification_path` now points to `/api/v1/account/email-links/begin?challenge=...`. GET only renders a confirmation page. POST `/begin` requires explicit `confirm:true`, exact `NOESIS_ACCOUNT_ORIGIN`, and fresh CF proof mapping to the initiating account. It sets a Secure HttpOnly browser nonce and returns the `/verify` page link. After switching provider email, GET `/verify` renders only; POST `/complete` with `{challenge,confirm:true}` requires the source-bound browser cookie, exact Origin and fresh requested-email proof. Challenge possession alone grants nothing. Successful completion preserves the stable UUID/original login email, revokes registered families, and returns `reauthentication_required:true`. See [the rollout runbook](ROLLOUT.md) for browser and database authority setup.

Private responses are no-store. Profile/run readers use transaction-local account context plus explicit current-grant queries and FORCE RLS, including table-owner enforcement. Superuser/BYPASSRLS connections remain privileged; production runtime-role verification is a separate rollout gate.

## Verification limits

Local PostgreSQL/router tests establish source behavior using synthetic identities. They do not prove external Access application policy, actual owner sign-in, registered callback configuration, production runtime role/RLS, delivery to a real device, or cross-surface hosted acceptance. Those remain rollout checks with separate receipts.
