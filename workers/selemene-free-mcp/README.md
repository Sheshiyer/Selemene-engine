# Selemene free MCP connector

This is a separate Cloudflare Worker for people using Selemene through ChatGPT. It needs no customer account, OAuth approval, or customer API key. The sponsor key is installed as a Worker secret; it is never embedded in the distributable plugin, tool inputs, source, or results. The existing `selemene-mcp` OAuth Worker and its namespace are independent.

Free engine readings use a **shared Selemene administrator account**. Birth inputs and calculated results may be stored in that account; usage, profile, XP and phase may change. There is no separate private customer history. Persistence is asynchronous and reported as `not_confirmed`. Input preparation is local and makes no API call.

The connector returns engine calculations, local Tarot/I Ching symbolic draws, and static engine meanings. The user's ChatGPT session supplies conversational interpretation using that user's ChatGPT limits. It calls no hosted inference model, witness pipeline, asset generator, media analyzer, or paid generation endpoint.

## Tools

- `selemene_list_engines`, `selemene_engine_info`, `selemene_prepare_reading`
- `selemene_calculate`
- `selemene_list_workflows`, `selemene_workflow_info`, `selemene_run_workflow`
- `selemene_vedic_chart`: a fixed Swiss Ephemeris natal D1/D9 chart calculation; its route does not save personal readings. Placeholder combustion flags and uncomputed dignities/yogas/Atmakaraka/Darakaraka are excluded or marked unavailable.

The eleven reviewed engines are numerology, human-design, gene-keys, vimshottari, panchanga (natal), vedic-clock, biorhythm, transits, enneagram, tarot, and i-ching. Only decision-support is exposed as a workflow, and its live composition must exactly match the reviewed engine set. Natal calculations require the necessary birth date, exact time, coordinates and timezone; the assistant must request missing fields rather than invent them.

The eighth chart tool is a distinct route, not a new `vedic-kundali` engine ID. Full multi-pass witness reports are outside this connector.

## Boundaries

Only two verified HTTPS API origins are accepted, and each tool fixes its method and path. Caller credentials, account identity, consciousness level, roles, arbitrary upstream URLs, internal auth context and unknown fields are rejected or ignored where MCP metadata requires acceptance. The trusted account phase is derived internally from `users/me` for sponsored calculation attempts; account ID, email and phase are not shown.

Requests are streamed with a 32 KiB cap. Upstream responses and final MCP responses have a 256 KiB cap. Metadata requests time out after 15 seconds; calculations after 45 seconds. There are no automatic calculation retries. Errors, credential echoes, auth metadata and credential-bearing URLs are sanitized. SDK lifecycle initialized/cancelled notifications have no tool/upstream effect; calculation notifications, batches and unsupported operations are rejected.

## Distributed quotas

One SQLite Durable Object named `sponsor-global` atomically enforces the service ceiling across all Worker instances. It persists timestamps, counters and salted HMAC hashes of Cloudflare's trusted connecting IP; no raw IPs, birth input, result or credential is stored in this Worker. A bounded record holds at most 512 IP hashes, pruned after two minutes. The global daily counter remains through IP cleanup and resets at UTC midnight.

Default fixed-window quotas:

| Limit | Default |
| --- | --- |
| Requests per connecting IP per minute | 120 |
| Calculation attempts per connecting IP per minute | 30 |
| Requests for the entire service per minute | 600 |
| Calculation attempts for the entire service per minute | 60 |
| Calculation attempts for the entire service per UTC day | 1,000 |

Calculation attempts are charged after a known tool and validated outer input. Later account unavailability or workflow preflight failure is not refunded. The request quota and calculation quota are separate; a calculation does not charge the request counter twice. Optional corresponding `IP_*`/`GLOBAL_*` variables are validated positive integers.

Connecting IP is an abuse-control signal, not customer identity or per-user accounting. NATs and ChatGPT MCP gateways may share an IP. Fairness is best effort; the atomic service ceiling is enforced. Missing trusted IP, salt, namespace or available Durable Object fails closed. Quota denial returns 429 and Retry-After; limiter unavailability returns 503.

## Local checks

Imports reuse the reviewed contracts in the sibling OAuth package, so install both packages:

```sh
npm --prefix ../selemene-mcp ci
npm ci
npm run typecheck
npm test
npm run deploy:dry-run
```

Tests use the official MCP client/transport and mocked backend/transactional storage. They perform no real reading, paid generation, credential transfer or deployment.

## Scoped deployment

The configuration fixes account `9d9d23b27f32e70ae3afb6a1aa2c0f10`, Worker `selemene-free-mcp`, a new SQLite Durable Object migration, and the public resource:

`https://selemene-free-mcp.sheshnarayan-iyer.workers.dev/mcp`

After source review and local checks, deploy this new Worker using the verified scoped profile:

```sh
wrangler deploy --config wrangler.jsonc --profile 9d9d
python3 scripts/install-secrets.py
```

The secret installer reads only `SELEMENE_API_KEY` from the operator's Claude credential file, passes it privately over stdin to Wrangler, and installs a cryptographically random `RATE_LIMIT_SALT` only when missing. It never prints secret values, sources a credential file into the shell, places secrets in command arguments, or overwrites an existing salt. It requires that this Worker already exists. Review its source before execution; creation of the file is not evidence of a performed transfer.

Before secrets are installed, public requests fail closed. A health response reports configuration only; it is not proof that the credential, engines, metadata route or all dependencies are operational. Verify the actual SDK connection, eight noauth descriptors, locally prepared input, and an explicitly authorized reading separately.

Record the installed version before any future update. Roll back only this Worker's recorded version; preserve the Durable Object and existing OAuth connector. Public directory upload, review and publication are separate plugin states.
