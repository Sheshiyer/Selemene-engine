# Railway infrastructure audit for the Selemene customer plugin

Observed 2026-09-29 22:03 UTC / 2026-09-30 Europe/Paris. Read-only investigation; no Railway linkage, service, deployment, variable, schema, or provider mutation. This document is the only file created by this audit. Existing dirty files were preserved.

## Identity and exact target

- Railway CLI: `/opt/homebrew/bin/railway`, version **5.62.1**.
- `railway whoami`: authenticated as **Mage Narayan**.
- This checkout is **not linked**: `railway status` returned “No linked project found.” Authentication and local project linkage are separate states.
- `railway list --json` located **robust-adventure**, project `11eedde4-41e6-4f51-b86b-cf77111cf592`, workspace `871e554f-2c1b-4a0e-850a-d09019b4036d`.
- Its accessible environment is **production**, `702b945e-2c66-4d5a-bae1-4c67ea14c3bb`. All later reads used explicit project/environment selectors; no `railway link` was run.
- Local checkout HEAD: `3e9ecb61e3f4a4b40ac624a5ecefa5753876b26f`. Parent audit verified remote main `ebd97fe940a3454b0425ff60d56003b07337564e`; neither is automatically the running Rust API revision.

## Current deployment inventory

All seven service deployments returned `SUCCESS`, with one `RUNNING` instance each, in `asia-southeast1-eqsg3a`. This is provider deployment state, not proof of every engine's functional behavior.

| Service | Service ID | Active deployment / created UTC | Source evidence |
| --- | --- | --- | --- |
| Selemene-engine | `48b3bd23-5620-4f7b-8e5d-96bc5c5d7fc4` | `ee9cb602-d3d3-4db3-a8d6-d06fc75090a8` / Sep 8 18:10 | Provider `commitHash` absent; historical release receipt links this deployment to `561c3e476e134eeac424c77b092b95b9749e0215` |
| ts-engines | `94419a41-9003-4a31-8bfe-d55b39ca4cb2` | `ef0033cf-6ffe-4229-9f70-1d6564d03afe` / Sep 8 18:43 | Provider SHA `ebd97fe940a3454b0425ff60d56003b07337564e`, branch main |
| biofield-cv-service | `f596e31b-e190-409c-993d-a3b618d29a73` | `11bcc687-9c4b-4682-b5ff-bbba49f824d3` / Jul 11 09:24 | Provider SHA `0efc3d8ae939f1c793537a700d0c3e37c2cdd38e` |
| Redis | `37fc197c-19ef-4f5e-a371-6a4abdbee907` | `8e29af18-7ba9-46e2-b92e-916ee6eae308` / Sep 5 10:58 | `redis:8.2.9` |
| Postgres | `1ba4a991-ca9a-4831-981e-aa3470101ef1` | `922eb502-80a7-4027-af98-5bce1e53c714` / Aug 23 11:44 | `ghcr.io/railwayapp-templates/postgres-ssl:18` |
| witness-agents | `f62adfac-9f81-4a0a-8cf5-8e5c212eddbb` | `377d2553-848f-4072-b4e0-c6986e8d3a6c` / Jun 1 08:08 | No repository or commit recorded in returned source metadata |
| suno-bridge | `eb888f5b-0ce0-4db0-8f8e-8c08dbc7841d` | `6199dcb8-80b3-4c35-b1b2-96c39c89c924` / Jun 1 08:08 | No repository or commit recorded in returned source metadata |

The root image digest is `sha256:ba3baabf749bce30c9543f2049410b2da8df6d574067f6dba7ef3c0d4680ec82`. The TS digest is `sha256:8c4e24e167a654d4e507f046959cb63381d94e295863d14b8306f8ace638339d`.

Root recent deployment history also contains two later **SKIPPED** entries: `789ea854-5ba1-4644-9458-2e812fcb6928` at Sep 8 18:43 for main `ebd97fe`, and `238f61d5-95b8-4341-bfde-260e26e4dc20` at 18:59 without SHA. Neither replaced the active deployment. `git show ebd97fe:docs/release/external-release-candidate.md` records workflow [34258440332](https://github.com/Sheshiyer/Selemene-engine/actions/runs/34258440332) validating source `561c3e47` and producing active deployment `ee9cb602`. This is documentary provenance corroborated by the still-active deployment ID, not a newly returned provider commit attestation. The same historical document explicitly leaves deployed-image/GHCR equivalence and Railway schema revision unproven.

## Routing, data, and sidecars

- Rust API: repo `Sheshiyer/Selemene-engine`, root config `/railway.toml`, `Dockerfile.prod`; Railway health gate `/health/live`. Both `selemene.tryambakam.space` and `selemene-engine-production.up.railway.app` route to port **8163**. Dockerfile's baked port is 8080; runtime configuration therefore matters.
- TS: same repo, root `/ts-engines`, config `/ts-engines/railway.toml`, resolved builder **DOCKERFILE**, `Dockerfile`; `/health` gate. Two public domains route to port **3001**: `ts-engines-production.up.railway.app` and `ts-engines-production-56f0.up.railway.app`.
- Biofield: same repo, root `python-services`, config `/python-services/railway.toml`, `Dockerfile.biofield`. It has **no public domain** and its active deployment manifest has **healthcheckPath null**. Local source now declares `/health`; that source setting is not evidence it is active on the July image.
- No MediaPipe service appears in this project's seven-service inventory. Root variable-name inventory includes `PYTHON_BIOFIELD_URL`, not a MediaPipe URL name. This does not rule out an external service, but no operational face-reading sidecar was established here.
- Witness service domains: `48.tryambakam.space` and `witness-agents-production.up.railway.app`, port 3333. Suno bridge domain: `suno-bridge-production.up.railway.app`, port 3000. Neither was exercised; paid generation was outside this audit.
- Postgres volume `postgres-volume` mounts `/var/lib/postgresql/data`; Redis `redis-volume-h8mJ` mounts `/data`. Two other READY Redis-named volumes have no serviceId in the response. No cleanup or volume reads were performed.
- Root variable **names only** confirm configuration hooks for `DATABASE_URL`, `REDIS_URL`, `TS_ENGINES_URL`, `PYTHON_BIOFIELD_URL`, Cloudflare Access, JWT, internal service key, provider keys, R2, and billing. Presence does not prove values, validity, authorization, or provider behavior. A separate `S_ENGINES_URL` name also exists; no usage or effect was established. Secret values were neither output nor saved.

## Live, unauthenticated HTTP observations

All probes were GET requests with no customer data and no calculation/generation calls.

| Surface / path | Result | Meaning |
| --- | --- | --- |
| Custom domain `/health/live`, `/health/ready` | 403 from Python urllib | Current client path through the custom domain is blocked; reconcile with Cloudflare audit before selecting the plugin upstream |
| Direct Railway root `/health/live` | 200 | `status=ok`, version 3.3.1, 19 engines loaded, 6 workflows loaded |
| Direct Railway root `/health/ready` | 200 | Redis/Postgres ok, orchestrator ready, TS bridge available, six bridge health checks passed |
| Direct Railway root `/api/v1/engines` and `/api/v1/workflows` | 401 | Contract-v1 UNAUTHORIZED; API requests require JWT bearer or `X-API-Key` |
| Direct Railway root `/api/v1/engines/capabilities` | 404 | The canonical local public capability route is not present on the running root image |
| Direct Railway root `/mcp` | 404 | No GET MCP endpoint established at this path |
| Direct Railway root `/.well-known/oauth-authorization-server` and `/.well-known/oauth-protected-resource` | 404 | No OAuth discovery established at these paths |
| TS `/health` and `/health/ready` | 200 | Six registered engines healthy; health detail says “default health check passed” |
| TS `/engines/capabilities` | 200 | Six contract-v1 rows, each availability `available`; current source-reported metadata is live here |

TS rows are tarot, i-ching, enneagram, sacred-geometry, sigil-forge, and raaga. Enneagram and sigil-forge declare required_phase=1; the rest declare phase 0. All report TypeScript runtime, empty dependency arrays, and version 1.0.0 except sigil-forge 2.0.0. Registry availability and default health checks are insufficient evidence of full generation/provider paths or a caller's phase access. The earlier repository note saying live TS capabilities returns 404 is now stale.

## Deployment-contract gaps relevant to the plugin

1. **Public root capability discovery is ahead of production.** Local `crates/noesis-api/src/lib.rs:1025` and handler at line 3233 define `/api/v1/engines/capabilities`; live returns 404. The plugin must not assume all local consumer contracts are deployed.
2. **Sidecars have different revision ages.** TS is current remote main, root maps to the September release via historical receipt, and Python is a July SHA. Any plugin promise involving newer Python provenance/capture contracts requires its own deployed evidence.
3. **No customer MCP/OAuth service exists in this inventory.** `bridges/universal-tool-server` has a FastAPI Dockerfile but is not among live Railway services. Its current source exposes `/tools` and `/tools/execute`, keeps one server-wide `SELEMENE_API_KEY`, and has no inbound customer-auth dependency at execution. Deploying it unchanged would not provide customer isolation or MCP/OAuth.
4. **Config intent differs from resolved config.** TS TOML says NIXPACKS and JSON says DOCKERFILE; active resolved manifest says DOCKERFILE. TS and Python `[deploy.watch]` include/exclude intent does not appear as effective build.watchPatterns (both empty). Root `[deploy.resources]` declares 1024 MB / 1 CPU but active manifest has limitOverride null; explicit resource enforcement was not verified.
5. **Migration and identity proof need a release plan.** Active root manifest has no preDeployCommand. CI `.github/workflows/deploy.yaml:244` validates an exact source checkout, builds both images, deploys only the named root service with project-token `railway up --service ... --ci`, and then health/smoke checks; it does not itself make all sidecars atomic or attest schema/image equivalence. Root Dockerfile uses source builds rather than deploying the prebuilt GHCR digest.
6. **Config format maintenance is approaching.** Installed Railway CLI schema inspection warns `railway.toml` / `railway.json` Config as Code is deprecated, existing files work until **2026-12-01**, and names `.railway/railway.ts` as the migration destination. This is an observed CLI warning; no migration was attempted.
7. **The deployment guide is stale.** `docs/deployment/RAILWAY.md` still describes three services, `noesis-redis`, root port 8080, and nonexistent `railway.ts-engines.toml`; live inventory has seven services, Redis named `Redis`, root public target 8163, and `/ts-engines/railway.toml`.

## Hosting implications

Railway is authenticated and operational; missing Plugin Creator guides do not mean the existing infrastructure is unavailable. A new customer MCP adapter could technically be hosted as a separate Railway service beside this API and use private networking, but creating/deploying it remains a separate action and does not restore Plugin Creator's missing publishing dependency. The adapter needs inbound customer identity, scoped access, safe per-user token handling, a capability contract compatible with the actually deployed API, and a health route that does not imply every provider is functional. Direct TS calculation routes should not become an ungoverned shortcut around Rust user/phase authorization. Use ordinary engine readings for the initial scope; keep image capture, biometrics, paid audio generation, and other sidecars out until their runtime contracts are proven.

## Reproducible read-only commands

```sh
railway --version
railway whoami
railway status
railway list --json
railway status --project 11eedde4-41e6-4f51-b86b-cf77111cf592 --environment production --json
railway deployment list --project 11eedde4-41e6-4f51-b86b-cf77111cf592 --environment production --service 48b3bd23-5620-4f7b-8e5d-96bc5c5d7fc4 --limit 5 --json
git show ebd97fe:docs/release/external-release-candidate.md
```

Status/deployment JSON was projected to service IDs, deployment state, source metadata, domains, builder/health settings, image digests, and volume mount metadata before synthesis. Do not use `railway variable list --json` bare: it includes raw values. Variable-name verification used an in-memory subprocess and printed only sorted keys:

```python
import subprocess, json
for service in ['Selemene-engine', 'ts-engines', 'biofield-cv-service']:
    result = subprocess.run([
        'railway', 'variable', 'list',
        '--project', '11eedde4-41e6-4f51-b86b-cf77111cf592',
        '--environment', 'production', '--service', service, '--json'
    ], capture_output=True, text=True, check=True)
    print(service, sorted(json.loads(result.stdout).keys()))
```

HTTP probes used Python urllib with a 20-second timeout against the paths in the observation table; responses retained only public health/catalogue data or short authentication/status errors. No application logs, personal readings, database contents, or secret values were requested for output.
