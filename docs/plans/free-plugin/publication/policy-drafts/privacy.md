**DRAFT FOR REVIEW — NOT EFFECTIVE OR PUBLISHED.**

# Selemene privacy — implementation facts for review

- Proposed route: https://www.tryambakam.space/selemene/privacy/
- Website destination: `public/selemene/privacy/index.html`
- Prepared: 2026-10-07 for the free 0.2.0 connector. Effective date: **PENDING**.

This draft describes the reviewed free connector's current behavior. It is not an effective privacy notice and does not provide a complete processor inventory, legal basis, retention schedule or request-handling commitment. Publisher/controller identity, address, jurisdiction and private privacy contact are **PENDING**.

## Information submitted for a reading

Depending on the chosen engine, supported calculation fields can include name, birth date, exact birth time, location coordinates, timezone, selected parameters and a calculation date. Not every engine requires every field. Missing required details are identified during preparation instead of guessed. The assistant sends only fields supported by the tool schema; optional interpretive context can remain in ChatGPT.

The service receives the supported fields actually submitted to its tools and returns calculated results, local symbolic draws, static meanings or reviewed rule-based synthesis. Discovery and local input preparation are distinct from an execution request.

## Shared upstream account and persistence

**Free readings use a shared Selemene service/admin account. Submitted birth inputs and calculated results may be stored there; usage, profile, history, XP and phase may change. No personal linked account or separate private user history is created.** This disclosure is presented before an ordinary calculation or workflow is requested.

The ordinary reading handlers attempt asynchronous, best-effort persistence. The inspected store can hold normalized inputs, results, engine/workflow and account associations, and timestamps. A successful calculation is not confirmation that storage succeeded. The plugin exposes no personal account history, account administration or deletion operation.

The separate fixed Vedic D1/D9 chart handler does not save a personal reading/history by that handler. This limited route fact does not establish end-to-end ephemeral processing or a general no-storage mode. ChatGPT conversation data, provider logs and other infrastructure remain distinct.

## Processing surfaces

Requests pass through ChatGPT/OpenAI, a Cloudflare sponsored MCP Worker and the Selemene Rust API hosted on Railway. Upstream database/cache infrastructure can process or retain calculation data. The connecting ChatGPT account/conversation is governed by its provider's own arrangements; this draft does not promise control or deletion of that data through Selemene.

The user's current ChatGPT model writes interpretation, Aletheios/Pichet perspectives and final prose using the user's applicable ChatGPT limits. The reviewed connector does not invoke another hosted inference model, a witness pipeline, media analysis, assets generation or paid generation.

**PENDING/UNVERIFIED:** the complete processor/subprocessor inventory, hosting regions, telemetry destinations, cross-border arrangements and applicable safeguards. The known request path is not proof that only the named infrastructure providers process data.

## Credentials and edge abuse controls

The free connector requires no personal Selemene login, API key or customer OAuth approval. Its sponsor credential is installed as a Worker server secret, excluded from the distributable plugin and customer inputs/results. Do not place credentials in chat or public support channels.

For abuse controls, the Worker processes Cloudflare's trusted connecting IP and derives a salted HMAC hash. Its Durable Object stores only hash entries, counters and timestamps, with a maximum of 512 active entries. Inactive entries use a two-minute TTL and are pruned during requests and alarms. The service-wide daily counter is independent and resets by UTC day. This hash is an abuse-control signal, not a personal identifier for user-account accounting; gateways and shared networks may share an IP.

The reviewed Worker source does not log or persist raw IPs, reading inputs/results or sponsor credentials. This narrow source-level finding does not establish Cloudflare edge logs or any other provider's logging/retention practices. The configured Worker observability is disabled; provider operational practices remain separately unverified.

The scoped backend audit identifies request metadata logging such as method, path, trace/account identifier, response status and duration. It also records configured error monitoring whose live payload/redaction, delivery and retention have not been established. Do not infer a system-wide no-logging promise from the Worker implementation.

## Retention, access and deletion

**PENDING:** justified retention periods for shared reading inputs/results, profile/usage data, security/operational logs and backups; the private contact, request owner, access/deletion procedure and response timing; implementation and verification of any promised deletion.

The existing bounded lifecycle audit did not find a general customer/reading deletion endpoint, general account-data export function or scheduled reading/profile/usage purge. That finding is not an erasure guarantee or a statement that operational deletion can never be implemented. Cache settings and historical backup metadata do not establish a reading retention policy. Removing a plugin connection is not a deletion receipt for data already processed or retained upstream or by ChatGPT.

The current public [technical issue tracker](https://github.com/Sheshiyer/Selemene-engine/issues) is not a private privacy/deletion channel. Do not post birth details, full readings, credentials or account requests there. The actual private privacy channel remains **PENDING**.

## Decisions required before this becomes an effective notice

- **PENDING:** controller/publisher identity, address, jurisdiction, privacy contact and request owner.
- **PENDING:** applicable processing purposes/bases, supported audience/countries and minimum age.
- **PENDING/UNVERIFIED:** complete processors, regions, telemetry/backups and access arrangements.
- **PENDING:** implemented retention, access/export/deletion procedure and any justified timing commitments.
- **PENDING:** effective date, versioning, final review and approval against the deployed service.

[Home](https://www.tryambakam.space/selemene/) · [Reading flow](https://www.tryambakam.space/selemene/readings/) · [Terms](https://www.tryambakam.space/selemene/terms/) · [Support](https://www.tryambakam.space/selemene/support/)
