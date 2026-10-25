# Selemene MCP deployment receipt

Observed 2026-09-30 local session date (provider UTC 2026-09-29T23:05Z).

## Dedicated deployment

- Source commit: `a1ddbb65` on `codex/selemene-chatgpt-plugin`.
- Account: `9d9d23b27f32e70ae3afb6a1aa2c0f10`; explicit Wrangler profile `9d9d`.
- New dedicated namespace: `selemene-mcp-OAUTH_KV`, ID `7f42c45d5e6843f892c5d2af84c7e12f`.
- Worker: `selemene-mcp`.
- Origin: https://selemene-mcp.sheshnarayan-iyer.workers.dev
- MCP resource: https://selemene-mcp.sheshnarayan-iyer.workers.dev/mcp
- Initial version: `9e80e849-b54b-422a-b45a-9efd0598c895`.
- CLI deployment succeeded; 1072.38 KiB upload, gzip207.64 KiB, startup73ms.
- Only this new Worker and dedicated namespace were created. Existing owner gateway and Railway services were not modified. No shared key or customer credential was installed.

## Public checks

`live-endpoint-check.json` records curl checks: health200, authorization metadata200, canonical protected-resource metadata200, unauthenticated MCP401 with canonical challenge. DCR returned201 for a synthetic public client with a loopback callback and no client secret. No customer key or OAuth access token was used.

Python urllib default user-agent received Cloudflare1010 browser-signature rejection. Default curl and the Codex in-app browser reach the service. No Cloudflare security setting was changed. Actual ChatGPT client compatibility remains a separate acceptance check.

The actual in-app browser rendered client identity, callback host, both scopes, persistence disclosure, password field and Approve/Deny controls. A live Deny click exposed a consent-page policy defect: browser POST sent Origin:null under no-referrer and was correctly rejected by the strict Origin guard. A focused repair is in progress; this first deployment is not connected-customer acceptance.

## Rollback and limits

This was the first version, so there is no earlier working connector to roll back to. To contain access, disable this Worker's public endpoint in Cloudflare settings while preserving its OAuth namespace. For later updates, record the prior version and use the scoped Wrangler rollback command documented in workers/selemene-mcp/README.md. Never alter the owner gateway as rollback.

No real customer connection, personal reading, provider purchase, recorded demo, plugin upload, review submission or publication is claimed.

## Browser repair verified

Commit `c79806c0` fixes the consent GET policy to same-origin; approval/denial redirects retain no-referrer and strict Origin validation remains. Full local suite expanded to18 passing tests, including scope metadata/step-up and consent policy regressions. Deployed version `e8d38136-faa5-426f-abcd-63a5e56d76b7`, upload1072.81KiB/gzip207.78KiB, startup58ms.

Fresh IAB browser form test observed POST Origin exactly the canonical Worker origin. The response was302 with `error=access_denied`, original synthetic state, canonical issuer, and `Referrer-Policy: no-referrer`. The loopback callback navigation was aborted locally because no callback server was running; response headers prove the server denial completed. No API key was entered and no account grant or reading was created. This verifies deployed rendering and keyless denial, not a successful real customer authorization.
