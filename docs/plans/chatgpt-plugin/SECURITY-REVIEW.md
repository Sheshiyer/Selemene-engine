# MCP security review — final bounded local review

Date: 2026-09-30. Reviewer: phase_repair, independent of the MCP implementation.

Current verdict: **PASS for the bounded source and synthetic local review.**
The earlier checkpoint required changes; its findings and provenance remain
below as history. The final section records repairs and the independent rerun
of 16 passing tests. No remaining source blocker was identified in the reviewed
scope. This is not production or ChatGPT connection acceptance, nor approval
of later edits.

## Scope and method

Read-only review of `workers/selemene-mcp/src/{index,consent,mcp,upstream,env}.ts`,
tool schemas/handlers, and the installed pinned OAuth provider 1.2.1 and MCP SDK
1.31.0 implementation/types. No credentials, production user data or live
authorization grants were accessed. No requests were sent to upstream calculation
or account-linking endpoints. Only this review receipt was written during the
review assignments; a separate authorized assignment added the focused
synthetic protocol test identified in the final section.

Initial reviewed source fingerprints:

| File | SHA-256 |
| --- | --- |
| `src/index.ts` | `71d9feeefecbe023a22aabaea10d8e9b5be533d3219bdbbd1dd931f6bbae431c` |
| `src/consent.ts` | `0235d7c5f892fac52d801f2f03a205ff8144b3ab62cc6c11c005a18e737d72ca` |
| `src/mcp.ts` | `df3b1ca8f60e1206757a29b698ada96fa934d18ea73fa4a06571c11d1291ed48` |
| `src/upstream.ts` | `3edf89b857fbd9066d75489b3a095e7d2c62b5fc642ced6b4e53a1835b82fc79` |

Files were actively being repaired after the snapshot; hashes are provenance for
the findings, not claims that the current working files retain those defects.

## Findings handed to builder and parent

| ID | Severity | Observed issue | Required closure |
| --- | --- | --- | --- |
| SEC-01 | High | Consent POST reparses the bare form-action request as an OAuth authorization request, falls back to empty scopes, and thereby drops requested calculation permission. Caller-controlled POST query parameters can also differ from the stored consent transaction. | Call `approveConsent(request, handle)` without a replacement scope and derive granted scopes only from `approved.request.scope`. Never trust POST query scope as the approved transaction. Test normal read+calculate approval and a forged alternate query. |
| SEC-02 | Medium | Missing resource is not rejected: the check only runs when `parsed.resource` is truthy, and the provider normalizes omission to the configured resource. This contradicts the adapter's explicit required-resource contract. | Validate exactly one raw authorization `resource` parameter equal to the canonical MCP URL before parsing; cover missing, duplicate and wrong resource. Provider audience validation must remain enabled. |
| SEC-03 | High | Consent request size checks trust `Content-Length` before unbounded `formData()`. A missing, understated or chunked header bypasses the intended 8 KiB bound. | Read a bounded UTF-8 byte stream before parsing and reject overflow regardless of header. Test missing and dishonest headers. |
| SEC-04 | High | Upstream-401 challenge constructs `/mcp/.well-known/oauth-protected-resource` instead of `/.well-known/oauth-protected-resource/mcp`, and splits one Bearer challenge across three array entries. | Return one syntactically complete Bearer challenge string with the actual provider metadata URL. Exercise discovery from the returned challenge; preserve 403 as permission denial. |
| SEC-05 | Medium | MCP source claims batches are rejected, but SDK 1.31.0 explicitly accepts arrays up to `MAX_BATCH_SIZE`. Multiple state-changing calls could therefore occur in one request under the current wrapper. | If the intended adapter contract remains one operation per request, reject arrays using a bounded parser before SDK dispatch; prove no upstream write for rejected batches or call-shaped notifications. |
| SEC-06 | Medium | Consent validates `meData.phase`, but the actual account field is `consciousness_level`; the copied phase may bypass intended integer/range checks. | Validate the actual field 0–5 before grant creation and fail closed for malformed authenticated identity. |
| SEC-07 | Medium | Workflow options remained an unrestricted record and were copied directly to the upstream body; empty/malformed live engine composition still passed the allowed-engine filter. | Strictly constrain workflow options and require exact nonempty reviewed composition before POST. Exercise unknown/internal keys and absent/malformed/drifted composition. |
| SEC-08 | Medium | Write tool callback tests `calculate` alone although its comment/contract says all tools require `read`. Provider `requiredScopes` does not itself enforce per-tool callback authorization in the inspected `handleApiRequest` path. | Explicitly require the intended scope combination in the adapter. Test read-only, calculate-only, neither and read+calculate tokens, including scope narrowing on refresh. |
| SEC-09 | Low / conditional | Consent discards `describeConsent().clientDomain` and reconstructs redirect hostname instead of using the helper's `redirectHost`. | Show verified client domain when available and use the library display value. Current config leaves CIMD disabled (the library default); do not claim CIMD support. If enabled later, require `global_fetch_strictly_public` and dedicated client metadata/redirect tests. |

## Controls confirmed in the inspected source

- OAuth provider 1.2.1 defaults to S256 only and rejects authorization-code
  requests without PKCE for public clients. The adapter also explicitly demands
  a challenge and S256 method on the browser authorization path.
- Browser consent rewrite uses the provider's `beginConsent`, `approveConsent`
  and `denyConsent` helpers. Installed source shows browser cookie binding,
  stored-request recovery, transaction expiry handling and handle consumption.
  The provider defaults to a `__Host-oauth-` cookie prefix. These are source
  observations; replay/cross-browser acceptance tests still need to pass.
- The consent POST checks exact Origin, rejects unknown actions and delegates
  redirects to the stored validated authorization request via library helpers.
- MCP context obtains the effective token scopes from `ctx.auth.scope` rather
  than assuming the original grant scopes. Provider code checks token expiry and
  canonical resource audience before invoking the protected handler.
- Each MCP request creates a separate SDK server carrying that authenticated
  grant's props. The adapter sends that grant's Selemene credential only as
  `X-API-Key` to a fixed allowlisted HTTPS upstream, with redirects disabled.
  No shared operator credential path was observed.
- Customer credentials are assigned to encrypted grant `props`, not grant
  metadata. No adapter console logging or intentional tool output of raw keys,
  OAuth tokens or full auth props was observed. This is a bounded source scan,
  not proof covering provider observability or every future exception path.
- Upstream calls have no retry loop; the repair keeps timeout active while
  reading the response. Synthetic tests must still prove stalled/oversized
  response behavior and sanitized failures.

## Evidence requested at the initial checkpoint

The builder/parent should record exact final source and passing tests covering:

1. Successful real provider authorization-code/PKCE flow with synthetic upstream
   identity, then an SDK client initialize/list/call sequence.
2. Missing/wrong/duplicate resource, absent/plain/wrong PKCE, tampered redirect,
   expired/replayed/cross-browser consent, unknown action and wrong Origin.
3. Original approved scopes preserved; POST query cannot widen them; narrowed
   token scopes deny calculations; revoked and expired tokens fail.
4. Two independent users retain separate upstream credentials and results;
   failure bodies and protocol errors do not expose either user's credentials.
5. Valid 401 reauthorization metadata, distinct 403 handling, bounded chunked
   forms, bounded/stalled upstream bodies and zero writes for rejected batches.
6. Strict engine/workflow input validation and live workflow composition checks.

No automated test pass or final UI acceptance was claimed by that initial
read-only checkpoint. No deployment or public connection was attempted.

## Bounded source follow-up after direct repair

The final bounded read observed these newer fingerprints:

| File | SHA-256 |
| --- | --- |
| `src/mcp.ts` | `06ff2b25389c209d9cf6cb2590a844395b4b0358e1d631b209bc99a72f51b01e` |
| `src/consent.ts` | `a1c7f85518fc51d5472879a05b9822c5c4acda6c3a8100346df9dfb6443e15b5` |

At these source revisions, SEC-01, SEC-02, SEC-03, SEC-05, SEC-06,
SEC-08 and the display portion of SEC-09 are addressed in source. Approval
uses the stored transaction's scopes, the raw resource parameter must occur
exactly once, form bytes are counted during streaming, batches and tool-call
notifications are rejected, authenticated `consciousness_level` is validated,
write tools require both scopes, and the verified consent display fields are
used. SEC-04's challenge now also uses one complete Bearer string and the
correct resource metadata path in the inspected upstream helper.

An additional compatibility issue was identified in the earlier MCP wrapper:
SDK `McpServer.registerTool` 1.31.0 destructures a fixed set of descriptor
properties and does not retain an arbitrary top-level `securitySchemes`.
The newer source uses the official lower-level SDK `Server` and an explicit
`ListToolsRequestSchema` handler. It now emits OAuth scope arrays at both
top-level `securitySchemes` and `_meta.securitySchemes`. Scope denial also
now includes `_meta["mcp/www_authenticate"]`, with one complete Bearer
challenge that identifies the scopes required for that tool.

These are source closures, pending final synthetic protocol evidence. An
actual SDK `tools/list` test must assert both serialized security-scheme
arrays; insufficient-scope calls must assert the returned challenge and zero
upstream calculation requests. This reviewer did not execute the builder's
tests and does not promote this follow-up to final acceptance. SEC-07 and
CIMD opt-in behavior were not re-audited in this final bounded read. The
parent/builder owns the final test receipt and review of subsequent edits.

## Final review and independently executed tests

Final review covered the completed adapter and installed OAuth provider 1.2.1,
including workflow constraints (SEC-07), DCR/CIMD configuration, token resource
binding, encrypted grants, effective scopes, and upstream networking. The
independent reviewer ran `npm test` in `workers/selemene-mcp`: **2 files,
16 tests passed**, total duration **5.16 seconds** (Vitest start 01:02:28).
This includes the builder's nine full-provider integration tests and the
reviewer's seven protocol boundary tests. Earlier focused TypeScript checking
also passed; this final review did not rerun a deployment build.

Final observed fingerprints, relative to `workers/selemene-mcp`:

| File | SHA-256 |
| --- | --- |
| `src/index.ts` | `b91557f223944cf17403270783f81b241e2661475636f4554296a9bd71071f7d` |
| `src/consent.ts` | `0236019a6e7f87a04151d70895d695a4ebd7abdbc841387bc2b043b68d7302ec` |
| `src/mcp.ts` | `074028026ff44467d4c17328c15180bf715bbb25ee6dcf4139c8913d4aa6d3a5` |
| `src/upstream.ts` | `dc89793e9ae665320e457093ce0e5b130e1b2ab0aa0ec3fe796cd08f98775759` |
| `src/tools/index.ts` | `b2a40a89057177db88ba3be8b94b36505a843bba669e8744d1add079a7aba3e2` |
| `test/integration.test.js` | `575b2b56cfed6cef80b2dbead9dbf2788f46f13e7988452727d96dcc172a9c4e` |
| `test/security-protocol.test.ts` | `f88d31d728013f6122d6dca214c2f93c855e3c0083054117553effa9bed00f8f` |

### Final controls and closure

- **SEC-07 closed in source and partly exercised end to end.** Workflow options
  are a strict object with reviewed keys and bounded values. Live composition
  must be nonempty, all strings, distinct, and an exact match for the reviewed
  workflow before any calculation POST. Each participating engine's input is
  validated. Integration tests reject empty, incomplete, extra-engine and
  malformed composition, prove zero POSTs for these cases, and exercise the
  valid partial-output path. Strict unknown workflow-option rejection is
  established by the source schema, not a dedicated integration assertion.
- **CIMD is disabled, consistently with the README.** The provider option is
  absent and the installed implementation advertises false unless explicitly
  enabled together with `global_fetch_strictly_public`. The Wrangler flag is
  absent. The provider does not fetch arbitrary URL client IDs in this mode.
  No CIMD support or CIMD runtime validation is claimed.
- **DCR is enabled.** The full-provider test registers synthetic public clients
  with `token_endpoint_auth_method=none`. Provider source validates listed
  redirect URIs, forbids dangerous schemes, fragments and userinfo, requires
  at least one supported callback, then revalidates the selected callback
  against registration and the HTTPS/loopback policy at authorization and
  completion. Consent identifies unverified client names and shows the
  helper's redirect host. Malicious DCR/redirect variants are source-reviewed,
  not exhaustively covered by the local test suite.
- **Resource binding is enforced at two boundaries.** Authorization requires
  exactly one raw canonical resource; the wrapper now also requires exactly
  one canonical resource for code and refresh exchanges. Installed provider
  code checks the stored grant resource when issuing tokens and exact stored
  access-token audience before decrypting props or invoking MCP. Tests cover
  wrong/missing authorization resource, missing/wrong code-exchange resource,
  and a successful resource-bound refresh. Direct corruption of stored token
  audience and duplicate resource variants were not runtime-tested.
- **Grant confidentiality and effective scopes are evidenced.** The provider
  creates a fresh AES-GCM-256 key for props, stores encrypted props, and wraps
  the encryption key using the authorization code/access/refresh token paths.
  The adapter stores customer keys only in props, not public grant metadata.
  Tests link two customers, find neither plaintext key in synthetic KV values,
  and verify calculations use the respective customer's credential. Refresh
  narrowing to read-only prevents POSTs; independent tests deny read-only,
  calculate-only and absent scope combinations with complete challenges.
- **Upstream network controls are bounded.** Source accepts only the fixed
  validated HTTPS origins; tool paths are fixed or validated/encoded IDs.
  Customer keys go in `X-API-Key`; redirects are refused; there are no retries.
  Request and response stream caps count bytes, and the timeout stays active
  through response-body consumption. The stalled-body test terminates a
  synthetic stream and proves exactly one fetch. Network redirect refusal,
  every status mapping, and response-overflow cases were not each exercised
  in the final suite. Source still labels interrupted body reads with the
  byte-cap error text; that is diagnostic imprecision, not a bound bypass.
- **Raw protocol metadata is verified.** The independent test checks actual
  SDK response serialization for all seven tool names, top-level OAuth
  `securitySchemes`, its `_meta` mirror, and all three safety annotations.
  Batch calls, call-shaped notifications and multibyte oversized requests
  fail before upstream traffic. The full SDK Client test separately exercises
  initialization, listing and calls through the OAuth-protected Worker.

### Evidence limits and release boundary

Tests use Node, an in-memory synthetic KV implementation and mocked upstream
fetch. They prove local provider/SDK integration, sequential consent/code
replay rejection, access expiry/revocation and upstream-key reauthorization;
they do not prove Cloudflare KV consistency under concurrent or multi-region
replay, deployed observability, browser UI behavior or a real ChatGPT account
connection. No production secrets, customer records or live readings were
used. Public metadata discovery, real callback configuration and the customer
disconnect/reconnect lifecycle still require deployed acceptance.

The reviewed `wrangler.jsonc` still has placeholder OAuth KV namespace and
public origin. Those are concrete deployment prerequisites, not source test
failures. The source-review pass must not be represented as submitted,
deployed, publicly connectable or production-security certified.

## Deployment follow-up — public discovery and browser Deny verification

This additive update supersedes the placeholder/deployment statements above
only for the following recorded facts. The earlier local review and hashes
remain historical evidence; they do not automatically approve later changes.

The parent's `DEPLOYMENT-RECEIPT.md` records source `a1ddbb65` deployed as
Worker version `9e80e849-b54b-422a-b45a-9efd0598c895` in the scoped 9d9d
account. The dedicated OAuth namespace is now provisioned and the public
origin is `https://selemene-mcp.sheshnarayan-iyer.workers.dev`. The package's
`mcp.json` points to that origin's `/mcp` endpoint. Thus the previous
placeholder KV/public-origin prerequisites are resolved for this initial
deployment; this review lane did not perform those mutations.

`live-endpoint-check.json`, observed at 2026-09-29T23:06Z, records public
health and both authorization/protected-resource metadata endpoints returning
200, and unauthenticated MCP returning 401 with the canonical metadata
challenge. The deployment receipt additionally records synthetic DCR 201
and an actual in-app-browser consent form rendering. No customer credential
or OAuth access token was used for these public checks. The parent reports
17 local tests passing after the earlier independently verified 16-test run;
that later count is attributed evidence, not a fresh rerun by this reviewer.

**The initial public version had a browser consent defect.** The actual Deny click sent
`Origin: null` under the form's no-referrer policy and the strict Origin
guard rejected it. This is a concrete consent-flow defect in the initial
public version. A focused source repair exists at `c79806c0`, but this
initial checkpoint had not established deployment or a browser retest.
Do not relax the CSRF Origin check to accept null as a substitute for the
policy repair and its browser verification.

No authenticated ChatGPT/customer connection, real reading, live token
lifecycle acceptance, demo recording, plugin submission or publication is
established by the public checks. Publisher/legal facts and listing-policy
readiness remain unresolved in the ISA. The earlier bounded local PASS
remains a local result, not a release authorization or live acceptance pass.

### Subsequent parent-executed repair evidence

Before this receipt closed, the parent reported deployment of repair commit
`c79806c0` as version `e8d38136-faa5-426f-abcd-63a5e56d76b7`. Actual in-app
browser/CDP evidence showed the Deny POST sending the exact canonical Origin
and receiving 302 to the synthetic loopback callback with `error=access_denied`,
the synthetic state and canonical issuer. The redirect response preserved
`Referrer-Policy: no-referrer`. Local callback navigation aborted because no
callback server was running; no receiving-client success is inferred.

This closes the observed browser-Origin/Deny defect for that recorded version.
It does not close authenticated Approve, customer key linkage, token exchange
or real-host acceptance: no key was entered and no token or reading was
created. The parent also reports the builder's final full suite at **18 tests
passing**, superseding the intermediate 17-test report. These later browser,
deployment and test results are explicitly parent/builder-executed evidence,
not fresh executions by this reviewer.
