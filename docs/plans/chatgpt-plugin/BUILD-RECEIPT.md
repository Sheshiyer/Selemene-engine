# Selemene MCP implementation receipt

Observed 2026-09-30. Source location: workers/selemene-mcp in the isolated codex/selemene-chatgpt-plugin worktree. This receipt records local implementation and synthetic verification; it does not assert a deployment, real customer connection, or submitted directory entry.

## Implementation

- Official @modelcontextprotocol/sdk 1.31.0 Server + WebStandardStreamableHTTPServerTransport, stateless JSON mode.
- @cloudflare/workers-oauth-provider 1.2.1, browser-bound library consent, DCR, S256, exact resource binding, encrypted grant credential props, expiry/refresh/revocation.
- Seven tools, eleven reviewed non-media engine adapters; exact decision-support composition checking; strict input preflight; per-user X-API-Key; readiness unknown when capability route absent.
- Per-tool OAuth metadata, bounded private-account annotations, complete authentication challenges, strict scopes, no batch/notification writes or automatic retry.
- Stream byte limits, response/time limits, fixed upstream origins, secret redaction, sanitized errors, persistence disclosure and partial workflow reporting.
- Independent package and pinned package-lock; placeholder KV binding/public origin; no shared owner key.

## Routed implementation provenance

Initial heavy work used noesis-build via omniroute-codex.sh with session tag selemene-mcp-build. Parent call_logs receipts reported successful claude/claude-opus-4-7 and cheaperinference/claude-sonnet-5 paths. The first pass was stopped and reaped after review found protocol/contract blockers.

A focused repair used the same noesis-build rail, session tag selemene-mcp-repair. Parent call_logs reported successful claude/claude-opus-4-7 and command-code/xiaomi/mimo-v2.5-pro paths. It was stopped and reaped after source generation. Neither canceled subprocess is represented as a successful completed build. Parent-authorized bounded direct repairs closed concrete findings and added integration tests.

## Verification

- npm run typecheck: passed.
- npm test: 16 tests passed in two files (9 integration, 7 independently authored protocol tests).
- Real OAuthProvider with actual WebCrypto and actual MCP SDK Client against synthetic backend/in-memory KV: two-user credential isolation, encrypted KV props, discovery, personal calculation request translation, exact requested scopes, missing/plain PKCE and wrong/missing resource rejection, Origin/cookie/action/deny, consent/code replay, access expiry/revocation, refresh downscoping, upstream-key revocation challenge.
- Strict preflight missing fields/calendar/timezone/provider-option rejection, workflow malformed/drift rejection before POST and partial result detection.
- SDK wire tools/list securitySchemes in both locations, correct annotations, zero-fetch scope failures, batch/notification rejection, multibyte request cap, stalled upstream response timeout with no retries.
- wrangler deploy --dry-run: passed; final local measured bundle 1070.49 KiB / gzip 207.57 KiB, with explicit placeholder bindings.
- npm audit --omit=dev --json after final dependency changes: zero vulnerabilities, 96 production dependencies.

The test runtime has a minimal cloudflare:workers WorkerEntrypoint shim for Node; this is not native workerd or deployed browser proof. No remote deployment, infrastructure mutation, real reading, paid provider call, or credentials were performed by this implementation lane. Deployment, policy/publisher identity, ChatGPT E2E and submission approval remain separate evidence.
