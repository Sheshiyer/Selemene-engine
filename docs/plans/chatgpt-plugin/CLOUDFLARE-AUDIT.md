# Selemene Cloudflare audit — 2026-09-30 Europe/Paris

Read-only infrastructure pass, observed at approximately 2026-09-29 22:00–22:06 UTC. Local source HEAD: `3e9ecb61e3f4a4b40ac624a5ecefa5753876b26f`, with unrelated working-tree changes retained. This document is the only file created by this audit. No deployment, DNS, binding, secret, login, logout, profile activation, or resource-data operation was performed. Worker source was read into process memory to compare public code indicators; credential values were never emitted or saved.

## Identity and scope

The user-specified **Wrangler auth profile `9d9d` works**. Installed Wrangler is `4.124.0`; its help advertises `--profile` for resource commands. `wrangler auth list` returned `9d9d | -`: the profile exists but has no directory binding. Therefore explicit profile selection matters. No unscoped `whoami` result was used to judge authentication.

Live scoped API `GET /accounts/9d9d23b27f32e70ae3afb6a1aa2c0f10` returned success and account name `Sheshnarayan.iyer@gmail.com's Account`. A scoped zone query returned active zone `tryambakam.space`, ID `3c1066df55d4e99464c8bcf1f850894b`, owned by the same account. Scoped Workers and Pages reads succeeded. The account Workers subdomain is `sheshnarayan-iyer`.

DNS record reads through this OAuth profile returned HTTP 403 for all three exact requested names: `selemene.tryambakam.space`, `144.tryambakam.space`, and `atlas.tryambakam.space`. This is a DNS-read permission limitation, **not** a failed account login. No alternate credential was substituted.

## Live resource map

| Resource | Live deployment / version | Bindings and routing | Role |
|---|---|---|---|
| `selemene-gw` | Deployment `8942c715-0cc0-4b35-9208-2ba552193504`; version `fc97ef70-02e8-4ede-95fc-23a0f88b1752` at 100%; 2026-07-27T20:33:16Z | `SELEMENE_SECRETS` KV `7d11ab631a5145bfa8076546da6a9e27`; compatibility date 2026-07-01; workers.dev and previews enabled | Owner-token authenticated API-key proxy to `https://selemene.tryambakam.space` |
| `selemene-admin-api-proxy` | Deployment `3e8d0783-0c4e-4395-9069-8f941afc8e65`; version `2772fda7-f2ab-471c-9eb0-23516f36cfae` at 100%; 2026-07-13T09:51:24Z | No bindings reported; compatibility date 2026-07-06; workers.dev and previews disabled; live zone route `144.tryambakam.space/api/*`, route ID `d452a59105e04865875f67b3e62a5a4e`, fail-open false | Same-origin administrator proxy to Railway, preserving Cloudflare Access identity |
| `selemene-llm-proxy` | Deployment `a15cb736-851a-4442-b6d4-9a1df7e60ad1`; version `a2ecb33d-4f08-4d08-9092-e4b06aaf6051` at 100%; 2026-07-23T20:04:57Z | `LLM_SECRETS` KV `310ee556e91d465eb54d55586a541e49`; secret **name only** `CHAT_PROXY_TOKEN`; compatibility date 2025-04-01; workers.dev and previews enabled | LLM provider fallback proxy |
| `selemene-pattern-memory` | Scoped deployments read returns Cloudflare code `10007`: Worker does not exist in this account | Source config contains placeholder KV, Vectorize and D1 IDs plus an R2 bucket name | Source-only proposal; not a deployed dependency |
| Pages `selemene-atlas` | Project `eabff6e8-db1c-42f2-a999-411ba60625a6`; latest production deployment `f6ed2a82-0942-49f5-abda-e95b1da7b8d3`, 2026-09-12T02:28:05.59Z; branch `main`, reported source label `9a05f5c` | Domains `selemene-atlas.pages.dev`, `atlas.tryambakam.space`; deployment URL `https://f6ed2a82.selemene-atlas.pages.dev` | Static source-grounded visual atlas, not the engine API or MCP runtime |

The account Workers custom-domain listing contained no matching Selemene or `144.tryambakam.space` entries. The administrator hostname is connected by the verified zone route, rather than a Worker custom-domain entry. The Pages source label alone does not establish a clean Git-triggered deployment; repository publication notes describe direct upload from a dirty checkout.

## Bounded public observations

All requests were unauthenticated GETs, with no personal inputs or inference requests.

| Request | Observed response | What it proves |
|---|---|---|
| `https://selemene.tryambakam.space/health/live` using curl | HTTP/2 200, JSON, `server: cloudflare` | Branded API liveness route is reachable from this client |
| `https://atlas.tryambakam.space/` | HTTP/2 200 HTML | Published atlas is reachable |
| `https://144.tryambakam.space/api/v1/admin/session` | HTTP/2 302 to `red-queen-4dfa.cloudflareaccess.com` login; redirect query omitted | Administrator route is protected by Access for this anonymous client |
| `https://selemene-gw.sheshnarayan-iyer.workers.dev/health` | 200; `status: ok`, version `0.1.0`, key-presence booleans `nk_current: true`, `owner_token: true` | Worker executes and its expected KV keys exist; values and upstream key validity are untested |
| Gateway `/api/v1/engines` | 401 `unauthorized` | Anonymous requests are rejected by the owner-token gate |
| Gateway `/mcp` | 404 `not_found` | No MCP route on this deployed gateway |
| `https://selemene-llm-proxy.sheshnarayan-iyer.workers.dev/` | 405 `POST only` | Proxy executes; provider credentials, billing, response quality and fallback success remain untested |

A parallel audit saw HTTP 403 through Python urllib for the branded API health route. The bounded curl GET above succeeded immediately. This client-dependent difference does not establish an API outage or universal Access restriction; its exact cause was not determined and no bypass rule was changed.

## Source versus deployment findings

1. **Existing gateway is not the customer plugin's authentication boundary.** `workers/selemene-gw/src/index.js:101` checks one `owner_token`, then line 118 substitutes one upstream `X-API-Key` from `nk_current`, removing the original Authorization and Cookie headers. It forwards all `/api/v1/*` methods. This loses customer-specific identity if reused as the universal public customer bridge. Live script inspection confirms the same credential-key names and upstream host; there are no `initialize` or `tools/list` indicators and `/mcp` returns 404. The rate limit is explicitly per-isolate in-memory, 60/min/IP, not a shared customer quota.

2. **The administrator Worker is a distinct protected surface.** Local code and live source both target `selemene-engine-production.up.railway.app`. Source extracts an Access cookie into an identity header when necessary. The deployed route and anonymous Access redirect are verified; no authenticated administrator or end-user session was exercised. It should not be repurposed as the public customer plugin endpoint.

3. **The LLM proxy has real source/deployment drift.** Live script's default provider models are `deepseek/deepseek-v4-pro`, `deepseek-ai/DeepSeek-V4-Pro`, `nvidia/llama-3.3-nemotron-super-49b-v1.5`, and `kimi-k2.6`. Current local `workers/llm-proxy/src/index.ts:80` instead uses `nvidia/nemotron-3-nano-omni-30b-a3b-reasoning` for NVIDIA. Thus the current NVIDIA change is not in the inspected deployed script. The source comment claims the older model reached EOL, but this audit did not independently invoke the provider or verify that lifecycle claim. Both live and local scripts contain `CHAT_PROXY_TOKEN`; version metadata confirms the secret name is bound. A binding's presence does not prove a nonblank value or successful protected inference.

4. **Pattern memory is not ready infrastructure.** Besides the absent live Worker, source `workers/pattern-memory/wrangler.toml` contains symbolic placeholder IDs. This feature is unnecessary for discovery and one-shot personal readings and cannot be claimed as an available MCP persistence layer.

5. **Account selection is inconsistent in source configs.** Only `workers/selemene-gw/wrangler.toml` pins the account ID. Admin and LLM configs omit it; Pages must use explicit account environment/profile selection. All audit commands explicitly selected both the account and profile. This is an operational reproducibility issue, not an authorization failure.

## Implication for the plugin design

Existing Cloudflare infrastructure and scoped operator access are usable foundations. The missing piece is a dedicated customer MCP interface with customer-specific account linking, explicit allowed tools, and the correct per-user upstream credential/authorization contract. A separate Worker is a reasonable proposed isolation boundary if Cloudflare is the selected supported hosting path; this audit did not create or deploy it. Neither the owner gateway, administrator route, LLM proxy, nor static atlas is already that interface.

If the Plugin Creator workflow mandates its own hosting adapter, its tooling availability still needs resolution. Successful Cloudflare reads do not remove that packaging/workflow requirement, and missing adapter guides do not imply that Selemene's existing hosting or CLI credentials are broken.

## Reproduction commands and read paths

Run from repository root. These commands select the verified profile explicitly:

```sh
wrangler --version
wrangler auth list
wrangler deployments list --help
wrangler auth token --help

CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 wrangler deployments list --config workers/selemene-gw/wrangler.toml --profile 9d9d
CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 wrangler deployments list --name selemene-admin-api-proxy --profile 9d9d
CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 wrangler deployments list --name selemene-llm-proxy --profile 9d9d
CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 wrangler deployments list --name selemene-pattern-memory --profile 9d9d

CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 wrangler versions view fc97ef70-02e8-4ede-95fc-23a0f88b1752 --name selemene-gw --profile 9d9d
CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 wrangler versions view 2772fda7-f2ab-471c-9eb0-23516f36cfae --name selemene-admin-api-proxy --profile 9d9d
CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 wrangler versions view a2ecb33d-4f08-4d08-9092-e4b06aaf6051 --name selemene-llm-proxy --profile 9d9d
CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 wrangler pages deployment list --project-name selemene-atlas --profile 9d9d --json

curl -sS --max-time 20 https://selemene-gw.sheshnarayan-iyer.workers.dev/health
curl -sS --max-time 20 https://selemene-gw.sheshnarayan-iyer.workers.dev/api/v1/engines
curl -sS --max-time 20 https://selemene-gw.sheshnarayan-iyer.workers.dev/mcp
```

Additional Cloudflare API reads used `wrangler auth token --profile 9d9d --json` **only through a capturing subprocess**, with the credential retained in memory and supplied as an Authorization header. Do not print that command's output. Only selected noncredential fields were emitted. GET paths beneath `https://api.cloudflare.com/client/v4`:

```text
/accounts/9d9d23b27f32e70ae3afb6a1aa2c0f10
/accounts/9d9d23b27f32e70ae3afb6a1aa2c0f10/workers/subdomain
/accounts/9d9d23b27f32e70ae3afb6a1aa2c0f10/workers/domains
/accounts/9d9d23b27f32e70ae3afb6a1aa2c0f10/workers/scripts/{selemene-gw,selemene-admin-api-proxy,selemene-llm-proxy}/subdomain
/accounts/9d9d23b27f32e70ae3afb6a1aa2c0f10/workers/scripts/{selemene-gw,selemene-admin-api-proxy,selemene-llm-proxy}/deployments
/accounts/9d9d23b27f32e70ae3afb6a1aa2c0f10/workers/scripts/{selemene-gw,selemene-admin-api-proxy,selemene-llm-proxy}
/accounts/9d9d23b27f32e70ae3afb6a1aa2c0f10/pages/projects/selemene-atlas
/zones?name=tryambakam.space&account.id=9d9d23b27f32e70ae3afb6a1aa2c0f10
/zones/3c1066df55d4e99464c8bcf1f850894b/workers/routes
/zones/3c1066df55d4e99464c8bcf1f850894b/dns_records?name={selemene.tryambakam.space,144.tryambakam.space,atlas.tryambakam.space}
```

Brace notation describes separately executed GETs. Inventory responses were filtered to Selemene resources; unrelated account resources were not included. No KV values, database rows, R2 objects, user records, or provider requests were read.
