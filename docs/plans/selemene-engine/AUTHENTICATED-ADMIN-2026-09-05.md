# Authenticated admin and DNS evidence — 2026-09-05

The user supplied an authenticated in-app browser session. These were read-only production UI observations; no session credentials or personal records are retained in this receipt.

- `/admin/dashboard` grants route access and reports the bridge healthy, with six healthy TS sidecar engines.
- `/admin/engines` displays 19 distinct runtime route links, including `biofield-capture` and `financial-biosensor`. Both headings incorrectly said “16 engines”; the candidate replaces the fixed count with runtime-registry wording. Listing a route is not semantic/capability proof.
- `/admin/sidecars` reports six healthy TypeScript engines at `ts-engines.railway.internal:3001`, default health checks passed and closed circuit breakers. The page does not display Python CV health even though its description claimed it did. Candidate wording now describes the TS-only data actually shown; Python coverage remains GSD 03.
- The authenticated Cloudflare zone view confirms account `9d9d23b27f32e70ae3afb6a1aa2c0f10`, zone `3c1066df55d4e99464c8bcf1f850894b`, and admin Worker route `144.tryambakam.space/api/*`.
- The DNS table confirms `selemene` -> `9apgcwm8.up.railway.app` (proxied), `144` -> `805d39b3f845b26f.vercel-dns-017.com` (proxied), and `48` -> `6636vvzi.up.railway.app` (DNS only). All three use Auto TTL. This is targeted DNS verification, not an export of all 36 zone records.

The earlier Wrangler-authenticated API returned DNS error10000; that does not invalidate the later authenticated UI evidence. No new DNS access grant is required for this read-only mapping pass. No DNS values, Worker routes, cloud configuration, secrets or credentials were changed.

Browser evidence applies to the currently deployed admin. Candidate UI changes are separately linted/typechecked/built and require their own deployment acceptance. This production session does not prove the unreleased candidate is deployed.

`/admin/system` reports API, PostgreSQL, Redis and workflow orchestrator healthy. Its six workflow cards are idle in the selected24h window. No source commit, image digest or schema revision is shown; operational source/schema attribution remains unverified.
