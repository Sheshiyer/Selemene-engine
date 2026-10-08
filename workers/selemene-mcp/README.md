# Selemene MCP Worker

A dedicated customer connector for Selemene Engine. It uses the official MCP SDK's stateless Streamable HTTP server and Cloudflare's OAuth provider. It does not reuse the owner gateway's key.

## What is implemented

Seven tools: selemene_list_engines, selemene_engine_info, selemene_prepare_reading, selemene_calculate, selemene_list_workflows, selemene_workflow_info, selemene_run_workflow.

The reviewed engine allowlist is numerology, human-design, gene-keys, vimshottari, panchanga (natal only), vedic-clock, biorhythm, transits, enneagram, tarot, i-ching. This is an adapter allowlist, not a claim that every engine is operational or authorized for every account. Authenticated catalog membership is separate from runtime readiness. A missing capabilities route reports readiness unknown.

Only decision-support is eligible as a workflow, and its live engine composition must exactly match the reviewed set before execution. Results report missing engine IDs and partial status without inventing omission reasons.

Input preparation is local and uses the same strict per-engine validation as calculation. It never calls the backend output-validation route. Missing or invalid fields are reported without invoking an engine. Full requirements are returned by engine_info. Numerology requires a nonempty birth name and date; the upstream BirthData contract also requires coordinates and an IANA timezone. Time can be omitted for numerology/biorhythm; exact natal time is required for chart engines and natal Panchanga. The adapter never fabricates a birth time or substitutes a location.

Calculations/workflows can save birth inputs/results, populate account profiles, and affect usage/XP/phase. Tools disclose these side effects and are neither read-only nor idempotent. There is no automatic retry. Persistence happens asynchronously upstream: a successful result does not confirm storage.

## Identity and storage

- Authorization-code flow with mandatory S256 PKCE and exact resource PUBLIC_ORIGIN/mcp.
- DCR at /oauth/register; public clients use token_endpoint_auth_method=none. CIMD is not enabled or claimed.
- Consent uses the provider's beginConsent/approveConsent/denyConsent browser-bound transaction. POST requires same Origin, explicit approve/deny, and a bounded form.
- The customer enters their own nk_ API key into a password field on the HTTPS consent page. Never put keys in chat, tools, URLs, source files, or command arguments.
- GET /api/v1/users/me with X-API-Key establishes the account ID. The API key is retained only in the OAuth provider's encrypted grant props; it is not placed in plaintext KV metadata.
- Access tokens expire after one hour; refresh tokens after 30 days. Refresh can narrow scopes. OAuth revocation is supported at the token endpoint.
- All tools require mcp:read; writes additionally require mcp:calculate. Per-tool security schemes are mirrored in _meta. Rejected upstream credentials return a complete MCP authentication challenge.
- Backend access uses the linked user's credential on every request. No shared owner key, administrative route, password-login exchange, direct sidecar call, paid generation, or report route is exposed.
- Public origin and upstream origins are checked. Upstream is restricted to the verified Railway API and selemene.tryambakam.space. Redirects are refused. Requests/responses/time are bounded.

Cloudflare documents encrypted props in its pinned provider package's storage-schema.md. This protects stored credential confidentiality; it does not replace access controls on the KV namespace or account.

## Local checks

Run from workers/selemene-mcp:

    npm ci
    npm run typecheck
    npm test
    npm run deploy:dry-run

The integration suite uses actual OAuthProvider logic, actual WebCrypto, and an actual MCP SDK Client/HTTP transport with in-memory KV and a synthetic backend. A minimal cloudflare:workers WorkerEntrypoint shim permits Node execution; these tests do not establish live Cloudflare behavior or ChatGPT connection acceptance. No test uses real customer credentials.

## Deployment prerequisites and commands

The checked-in wrangler.jsonc binds the dedicated selemene-mcp-OAUTH_KV namespace provisioned on 2026-09-30 and the canonical workers.dev origin. Live deployment evidence is recorded in docs/plans/chatgpt-plugin/DEPLOYMENT-RECEIPT.md at the repository root. The account is pinned to 9d9d23b27f32e70ae3afb6a1aa2c0f10. The authenticated Wrangler profile is 9d9d; select it explicitly.

After review and release authorization:

    npx wrangler kv namespace create OAUTH_KV --profile 9d9d

Record the returned namespace ID in wrangler.jsonc and choose the exact HTTPS public origin. The resource will be that origin plus /mcp. Keep the same origin through authorization, token exchange, resource metadata, and plugin connection. Do not change Railway services, the owner gateway, or admin proxies to deploy this Worker.

    npm run typecheck
    npm test
    npx wrangler deploy --dry-run
    npx wrangler deployments list --name selemene-mcp --profile 9d9d
    npx wrangler deploy --profile 9d9d

Record source commit/digest, deployed version ID, account, namespace binding, origin, and any previous version before claiming deployment.

After deployment, verify /health, /.well-known/oauth-authorization-server, /.well-known/oauth-protected-resource/mcp, and an unauthenticated /mcp challenge. Then connect through ChatGPT developer mode using a designated reviewer/customer account. Verify two distinct accounts, discovery, scope handling, disconnect/revoke, and one explicitly authorized synthetic reading. Do not put a real API key in a curl command or saved receipt.

Public privacy/terms/support pages, publisher identity/domain verification, reviewer account access, directory metadata and final submission remain separate work. This Worker does not fabricate those facts.

## Rollback

For an update, record the existing deployment version first. Restore that exact version using:

    npx wrangler rollback PREVIOUS_VERSION_ID --name selemene-mcp --profile 9d9d

Confirm the installed Wrangler command help before release. For the first deployment there is no previous version: disable public access through the Cloudflare Worker settings if needed rather than deleting the OAuth namespace. Preserve the namespace until grants have expired/revoked and retention decisions are made. Do not remove or modify another Selemene Worker as rollback.

## Known evidence limits

The production API's /engines/capabilities route was absent during the preceding audit; fallback is intentional. Native engine semantics and upstream role/phase permissions remain enforced by Railway. Synthetic tests do not prove production engine readiness, account onboarding, billing behavior, browser UX in ChatGPT, or submission approval. Cloudflare KV's consistency characteristics still apply to provider state; the tests use deterministic in-memory storage.
