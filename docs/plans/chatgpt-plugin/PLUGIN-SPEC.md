# Selemene Engine for ChatGPT — revised implementation specification

Status: audited architecture proposal; no MCP implementation, deployment, plugin creation, or connected reading is claimed. Revised 2026-09-30 after user-authorized parallel source, Cloudflare, and Railway investigation. Audience: people discovering engines and requesting personal readings in ChatGPT.

## Decision

Build on the existing Cloudflare account `9d9d23b27f32e70ae3afb6a1aa2c0f10` and Railway Selemene production stack. A dedicated Cloudflare MCP Worker is the preferred new boundary; keep calculations, account state, history, and engine orchestration in Railway. Preserve the owner-token gateway separately. This recommendation supersedes the initial Sites-first proposal: missing Sites guides are not a blocker for this existing infrastructure path.

Use a stateless Streamable HTTP MCP endpoint with maintained SDK support, explicit OAuth protected-resource metadata, and per-user authorization. Do not replace the owner's gateway with a public multi-user interface. Do not assume either Wrangler operator login or Railway operator login authenticates Selemene customers.

## Evidence and source baseline

Detailed receipts: [code audit](CODE-AUDIT.md), [Cloudflare audit](CLOUDFLARE-AUDIT.md), [Railway audit](RAILWAY-AUDIT.md).

- Local HEAD: `3e9ecb61e3f4a4b40ac624a5ecefa5753876b26f`, branch `codex/selemene-capability-consumer-surface`, plus existing witness-pipeline WIP.
- Remote main: `ebd97fe940a3454b0425ff60d56003b07337564e`. Git ancestry reports 2 main-only / 18 local-only commits. Commit counts do not imply all changes are semantically absent from the other branch.
- Main contains the Dodo-free release changes from `561c3e47`; an implementation branch must preserve those changes while deliberately incorporating relevant capability work. Do not deploy the current dirty feature branch wholesale.
- Production Rust `/api/v1/engines/capabilities` is 404; the newer local route cannot be assumed available. Production TS `/engines/capabilities` returns six entries. Neither is a replacement for per-user execution authorization.
- The existing bridge is plain REST and incorrectly places its global API key in a Bearer JWT header. Fixing the header would still leave a shared-owner identity problem.

## Customer flow

1. Explain available kinds of reflection and distinguish symbolic interpretation from measured or calculated output. Use verified public metadata or authenticated live discovery; never advertise all 19 source IDs as operational.
2. Connect a Selemene account through a supported browser OAuth flow when a personal tool needs it.
3. Ask only for the chosen engine's required inputs. Never fabricate birth time, coordinates, time zone, live scores, or media consent.
4. Prepare a structured request locally and show any missing information. Explain that current calculation APIs save reading inputs/results and may populate the account profile.
5. Execute an explicitly requested calculation once, using the authenticated user's phase and permissions.
6. Preserve provenance and label partial workflows with missing engine IDs. Reflect on returned results without inventing an engine output or treating it as a diagnosis or guaranteed prediction.

## Identity and persistence contract

The MCP OAuth identity must bind to one real Selemene user, with explicit scopes, issuer/audience validation, expiry, refresh, and revocation. Implement or select the authorization server deliberately; the audited repository does not already supply the required authorization-code/PKCE flow. A service-owned key is not an acceptable substitute.

A ChatGPT MCP access token cannot simply be passed to the current Rust Bearer validator, which expects Selemene JWTs. Choose and verify either an API verifier for the intended issuer/audience or a bounded backend token-exchange/delegation mechanism producing a short-lived user-specific credential. Never trust a user-id HTTP header from the public edge. Keep account linking and token storage out of chat, plugin archives, and tool outputs.

Current calculations/workflows can save birth input and results, populate profiles, award XP, promote phase, and record usage. Mark calculation tools as having side effects; do not advertise them as read-only or idempotent. The v1 proposal preserves and discloses account history behavior. If an ephemeral-reading mode is desired, it requires a real backend contract covering every persistence path before being promised.

## Initial tool surface

These names are proposed, not registered or installed tools.

| Tool | Actual integration | Important constraint |
| --- | --- | --- |
| `selemene_list_engines` | Rust authenticated `/api/v1/engines`; capabilities only after verified route release | Distinguish catalog, readiness and user permission |
| `selemene_engine_info` | Rust `/api/v1/engines/{id}/info` plus reviewed adapter schema registry | Backend info has only ID/name/phase, not full input schema |
| `selemene_prepare_reading` | Local schema/input preflight, no calculation | `/validate` accepts EngineOutput and must not be used for this |
| `selemene_calculate` | Rust `/api/v1/engines/{id}/calculate` | Correct ApiEngineInput translation; authenticated phase; persistent side effects |
| `selemene_list_workflows` | Rust `/api/v1/workflows` | Authenticated metadata |
| `selemene_workflow_info` | Rust `/api/v1/workflows/{id}/info` | Establish expected engine IDs before execution |
| `selemene_run_workflow` | Rust `/api/v1/workflows/{id}/execute` | Legacy EngineInput differs from calculate input; report missing engine outputs |

Begin with a tested allowlist of non-media engines. No administrative tools, billing changes, direct sidecar tools, media capture, or provider generation is implied. A future saved-reading tool must use server-enforced user ownership. The adapter rejects arbitrary upstream URLs, unknown identifiers, malformed inputs, and overlarge bodies; bounds timeouts; sanitizes errors; preserves distinct auth/validation/unavailability states; and does not blindly retry calculations.

## Reports are a separate integration lane

The local TS witness pipeline has meaningful recent mode, factual-anchor and narrative changes, with dirty-tree synthetic tests passing. That does not prove a production HTTP route to that pipeline.

The Rust `/assets/generate` handler is explicitly seed-rendering scaffolding; it must not be labeled the full multi-pass witness report engine. `/witness/interpret` is a separate LLM flow requiring live scores, not a general birth-report shortcut. Both audited report handlers use a request/user phase `max` expression; source analysis shows that a caller-supplied high phase can defeat the intended phase gate. Repair and regression-test this before exposing report tools. No production exploit was attempted.

## Implementation order

1. Establish a clean branch based on current main. Reconcile the relevant capability and contract commits while preserving main's billing/release fixes and leaving unrelated witness WIP untouched.
2. Resolve customer identity issuance/delegation and persistence disclosure. Add regression coverage for report phase authority before any report exposure.
3. Build the isolated MCP Worker and seven tool contracts with synthetic adapters and two-user authorization tests. Reuse existing API hosting; do not modify owner-token behavior.
4. Add truthful capability discovery, explicit partial workflow handling, sanitized error translation, rate/size/time limits and request replay handling appropriate to side-effecting tools.
5. Verify locally through an MCP client; then use a separately identified scoped preview/deployment target and verify exact source revision, bindings and API route parity.
6. Package the verified remote endpoint using Plugin Creator's existing-server path. A saved plugin is not a deployed server or a verified customer connection.
7. Connect in ChatGPT and verify discovery plus an explicitly authorized personal reading. Expand to witness/media flows only after their separate provider, consent, cost and artifact contracts pass.

## Acceptance evidence required before claiming delivery

- Real MCP client initialization, discovery and calls work against the selected protocol version.
- OAuth metadata and user binding work with ChatGPT; token issuer, audience, scopes, expiry and revocation are enforced.
- Two users cannot access one another's credentials, readings or profiles.
- Low-phase users cannot escalate by supplying a higher request phase.
- Input preflight uses input schemas, not output validation.
- Production routes match the packaged tools and advertised capabilities.
- Missing workflow engines are visible; omission reasons are not invented.
- Persistence and side effects match the disclosed customer behavior.
- Errors and logs exclude sensitive input and credential values.
- Timeouts/retries cannot silently duplicate calculation side effects.
- The deployed version and rollback target are recorded independently of green CI.
- Connected ChatGPT discovery and a real authorized reading succeed.

## Current documentation references

- [OpenAI plugin authentication](https://developers.openai.com/plugins/build/auth): OAuth metadata and token verification requirements.
- [Cloudflare remote MCP server guide](https://developers.cloudflare.com/agents/model-context-protocol/guides/remote-mcp-server/): Streamable HTTP and stateless handler architecture. Pin and verify actual SDK versions during implementation.

No infrastructure or application source was changed by this audit. The recommended build is ready to scope against the verified stack; implementation and release evidence remain pending.
