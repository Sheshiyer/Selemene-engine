# Admin API key release — 3 November 2026

Status: **deployed and verified on production**.

## Released source and infrastructure

- Code commit: `156c08efc66f20e6bd3f3c5371b228d26a085913`, branch `codex/admin-key-tiers-20261103`.
- Railway project `robust-adventure`, service `Selemene-engine`: staging deployment `b53ca655-84c7-4e76-ab89-052246743d3a` and production deployment `7172d160-49ad-4baa-9f40-e1df4573d5ad` both report SUCCESS.
- Vercel project `enantiodromia-engine-dashboard`: production deployment `dpl_8g2u1gKsFSj92gvmWua8LvW355ai` reports READY, names the code commit above, and owns the `144.tryambakam.space` alias.
- Actual browser acceptance used `https://144.tryambakam.space/admin/api-keys`, through the existing Cloudflare Access session and Worker route. Cloudflare policies and Worker routing were preserved.

## Resulting behavior

Creating a key now requires choosing Basic, Pro, or Enterprise. The server maps permissions, access level, and request limits; manual permission and rate controls were removed from the creation form. Owner defaults to the signed-in administrator and remains editable for managed customer keys. Optional expiry uses a proper UTC timestamp.

| Label | Stored code | Default engine access | Access level | Requests/minute |
| --- | --- | --- | --- | --- |
| Basic | free | Panchanga and Numerology | 0 | 60 |
| Pro | premium | Reviewed catalog of 19 engines | 3 | 1,000 |
| Enterprise | enterprise | Reviewed catalog of 19 engines | 5 | 10,000 |

`basic:access` remains a baseline permission for all tiers. Administrator authorization is owned by the existing roles; the product tier does not add `admin:*` to newly created customer keys. Existing quota/billing behavior is retained.

The existing `witness-agents-service` key was explicitly reconciled to the full Enterprise bundle at the user's request: 19 engine read scopes plus baseline access, level 5, rate 10,000/minute. Its raw secret, owner, expiry, and identity were preserved. The sole active production key after QA is still that original key. Its secret remains at `/Users/sheshnarayaniyer/.claude/.env`, variable `SELEMENE_API_KEY`; no secret is recorded here.

## Implementation and review

- Repaired the creation INSERT's missing eleventh bind for `rotated_from_key_id`; the key and lifecycle audit event commit atomically.
- Added server-owned tier aliases, bundles, and a protected `GET /api/v1/admin/api-key-tiers` metadata endpoint. Both key-list and key-create administrators can read its nonsecret metadata.
- Legacy optional scope, phase, and rate overrides can only restrict a selected tier. Invalid tiers and privilege expansion are rejected.
- Default tier-managed intent is persisted in existing creation-event JSON and retained on rotation. Explicitly restricted and legacy keys are not inferred to be managed merely because a bundle happens to match a tier.
- Tier updates and rotations use the same owner-first lock order. Role changes preserve engine permissions rather than replacing the whole array with administrator permissions.
- Added submission guards, inline creation errors, server-derived entitlement previews, and friendly tier labels in the key list and filters.
- Preserved the recent `f3066115` minimal-input Numerology/Biorhythm changes and embedded-mode upload exceptions so this release does not regress that existing work.
- Independent review identified three actionable issues (restricted-key round-trip expansion, rotation/downgrade concurrency, and read-only tier metadata). All were addressed with failing-then-passing regression tests.

## Verification evidence

- Local: four tier-policy tests, two API integration/catalog tests, one real PostgreSQL lifecycle/concurrency test, ten existing scope tests, and two retained minimal-input compatibility tests passed. API compilation, dashboard typecheck, and Next.js production build passed.
- Staging: 34 expected-status checks passed, covering all three tiers, actual key authentication/calculations, denied Basic Tarot access, rotation, old-key rejection, revocation, role changes, and Basic/Pro/Enterprise tier transitions. Staging used a bounded signed admin fixture through its Railway API; no staging equivalent of the Cloudflare admin browser alias was configured.
- Production API: 27 expected-status lifecycle checks passed across all three tiers. Disposable keys were revoked; revoked and rotated-out secrets returned 401.
- Production browser: verified the three choices and their server-derived previews; created an Enterprise key through the real form, copied its secret without displaying it in chat, used that copied secret for a successful Tarot calculation, refreshed the registry, and revoked it through the UI. The copied secret then returned 401 and its temporary private file was removed.
- Additional browser checks: keyboard navigation and Escape handling; invalid-owner error displayed inline with the owner input retained; double-click submission created exactly one Basic key with its expected bundle. That disposable Basic key was revoked too.
- Original key: all eight originally requested engines plus Tarot returned 200 with results after the production release. Daily-practice workflow and the engine capability, skill status, system health, and Hermes status routes also accepted the expanded Enterprise key.
- Final state: exactly one active production key, the original Enterprise key with all 19 engine scopes and access level 5. Staging synthetic owner/history were cleaned up. Production QA records remain revoked for audit; ordinary synthetic reading/history records may remain.

Sanitized detailed receipts and rollback records are private under `/Users/sheshnarayaniyer/.codex/private/selemene-admin-key-tiers-2026-11-03/`. Original repair rollback evidence also remains under `/Users/sheshnarayaniyer/.codex/private/selemene-key-repair-2026-11-03/`. No raw keys or connection credentials are included in the tracked release documentation.

## Recovery

Previous production deployments remain Railway `d21dc4fc-fd4b-48c9-a7d5-000927ac80c5` and Vercel `dpl_CTbVRgodk1n7vF3ceQchGVgDoFbV`. Restore a deployment only to reverse a release failure; restore authorization fields only through the saved exact-key rollback record while checking for later concurrent changes. Existing working checkouts and their uncommitted API-key work were preserved.
