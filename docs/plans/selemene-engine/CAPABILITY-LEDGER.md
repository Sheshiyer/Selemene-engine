# Selemene Engine — Capability Ledger

**Evidence cut:** 2026-08-25  
**Scope:** Repository-local implementation, declared infrastructure, and fresh Selemene probes

## Status language

| Code | Meaning |
|---|---|
| `Y` | Directly evidenced in the current repository or a named fresh probe |
| `P` | Partial, conditional, fallback-dependent, or only some paths are real |
| `N` | Absent from the reviewed surface |
| `?` | Claimed or externally configured, but not freshly verified here |
| `NA` | Not applicable |

Axes are: `Dcl` declared, `Imp` substantive implementation, `Exe` executable evidence, `Int` repository integration, `Dep` deployed evidence, and `Ops` operational evidence. “Complete” is not used without naming the axes it covers.

## Primary evidence anchors

| Area | Current source anchors |
|---|---|
| Runtime catalogue and native registration | `crates/noesis-orchestrator/src/lib.rs` |
| Conditional capture and API catalogue | `crates/noesis-api/src/lib.rs` |
| TypeScript routing and readiness | `crates/noesis-bridge/src/lib.rs`, `ts-engines/src/server/app.ts` |
| Workflow execution and synthesis | `crates/noesis-orchestrator/src/workflow/`, `crates/noesis-api/src/lib.rs` |
| Witness and asset generation | `crates/noesis-api/src/handlers/witness.rs`, `crates/noesis-api/src/handlers/assets.rs`, `packages/witness-pipeline/` |
| Billing, publication, and onboarding | `crates/noesis-api/src/billing.rs`, `crates/noesis-api/src/handlers/billing.rs`, `crates/noesis-api/src/handlers/living_reading_invites.rs`, `crates/noesis-api/src/handlers/onboarding.rs` |
| SDK and client contracts | `crates/noesis-sdk/`, `packages/noesis-sdk-ts/`, `packages/noesis-engine-sdk/`, `bridges/cli/` |
| Persistence and cache | `migrations/`, `crates/noesis-cache/`, `crates/noesis-api/src/handlers/biofield.rs` |
| CI, release, and images | `.github/workflows/`, `Dockerfile.prod`, `docker-compose.yml` |
| Deployment topology | `railway.toml`, `ts-engines/railway.*`, `python-services/railway.toml`, `k8s/`, `workers/` |
| Monitoring and assets | `monitoring/`, `runbooks/`, `data/`, `docs/assets/` |

## Engine runtime ledger

| Runtime ID | Runtime path | Dcl | Imp | Exe | Int | Dep | Ops | Current truth |
|---|---|:---:|:---:|:---:|:---:|:---:|:---:|---|
| `panchanga` | native Rust | Y | P | Y | Y | P | P | Substantive calculation silently substitutes mean longitude when ephemeris lookup fails; output lacks fallback provenance |
| `numerology` | native Rust | Y | Y | Y | Y | P | P | Substantive deterministic engine; live catalogue requires authentication |
| `biorhythm` | native Rust | Y | Y | Y | Y | P | P | Substantive deterministic engine; no revision-bound production result probe |
| `human-design` | native Rust | Y | Y | Y | Y | P | P | Ephemeris-backed implementation; asset and result integrity remain release gates |
| `gene-keys` | native Rust | Y | P | Y | Y | P | P | Human Design gates are real, but calculated line data is discarded and replaced with fixed line 3 |
| `vimshottari` | native Rust | Y | Y | Y | Y | P | P | Native runtime is registered; production result accuracy was not freshly sampled |
| `biofield` | native Rust + direct Python analysis path | Y | P | Y | P | P | P | Native calculation, capture metrics, and Biofield CV are distinct paths; fallback provenance matters |
| `vedic-clock` | native Rust | Y | Y | Y | Y | P | P | Substantive time engine; live health proves a loaded count, not this engine's result correctness |
| `face-reading` | native Rust + MediaPipe sidecar | Y | P | Y | P | P | P | Sidecar path exists, but MediaPipe has no repository-proven live deployment and byte-heuristic fallback must remain explicit |
| `nadabrahman` | native Rust | Y | Y | Y | Y | P | P | Registered native runtime; narrower evidence than its narrative claims |
| `transits` | native Rust | Y | Y | Y | Y | P | P | Ephemeris-backed native runtime with focused tests |
| `financial-biosensor` | native composed engine | Y | P | Y | Y | P | P | Real composite, but it reruns Human Design through Gene Keys, silently omits failed sources, and renormalizes remaining weights |
| `biofield-capture` | database-conditional native engine | Y | Y | Y | P | P | P | Registration depends on a database pool and represents persisted capture lookup, not live CV analysis |
| `tarot` | TypeScript bridge | Y | P | Y | Y | P | P | Engine executes; replay seed, requested modes, and result/renderer semantics are not fully closed |
| `i-ching` | TypeScript bridge | Y | P | Y | Y | P | P | Changing lines are computed and then ignored; the relating hexagram is randomly selected |
| `enneagram` | TypeScript bridge | Y | Y | Y | Y | P | P | Registered and tested; authenticated production calculation not freshly probed |
| `sacred-geometry` | TypeScript bridge | Y | P | Y | Y | P | P | Symbolic output exists; promised generated geometry artifact remains partial |
| `sigil-forge` | TypeScript bridge + image providers | Y | P | Y | Y | P | P | Provider adapters and fallbacks exist; availability can describe mock/placeholder paths as usable |
| `raaga` | TypeScript bridge + optional audio service | Y | P | Y | Y | P | P | Theory engine and local WAV path work; hosted clip mode defaults off and richer audio remains partial |

Fresh `bun test` evidence on 2026-08-25: 88 TypeScript engine tests passed. Fresh production health reported `engines_loaded: 19` and all six TypeScript engines healthy. That probe proves registry and bridge reachability only. The TypeScript server's default self-check reports engines healthy when no engine implements a self-check, so this is module-loading evidence rather than calculation or provider evidence.

## Platform and contract ledger

| Capability | Dcl | Imp | Exe | Int | Dep | Ops | Evidence and correction |
|---|:---:|:---:|:---:|:---:|:---:|:---:|---|
| Rust core contracts | Y | Y | Y | Y | P | P | Widely consumed; duplicate client types still permit drift and the live revision is unidentified |
| Orchestrator and six workflows | Y | P | Y | P | P | P | Live health reports 19 engines and six workflows, but the API path returns `synthesis: None`; the richer executor is disconnected and four synthesis types remain generic |
| Axum API | Y | Y | Y | Y | P | P | `/health/live` and `/health/ready` returned 200 on 2026-08-25, but the running source/image revision is unidentified |
| Authenticated engine API | Y | P | Y | Y | P | P | Unauthenticated protected routes correctly return 401; password login is retired while registration/reset/change-password and clients still expose it, and API-key logout is a no-op |
| Rust HTTP bridge | Y | P | Y | P | P | P | Routes six calculation IDs, but Rust calls `/engines/:id/validate` while the TypeScript server exposes no matching route or interface method |
| Rust SDK | Y | P | Y | N | ? | ? | List envelopes and engine fields disagree with API, list requests omit required auth, and its test server reproduces the wrong contract |
| General TypeScript SDK | Y | P | Y | N | ? | ? | List envelopes/fields, `updateMe` method, witness request, validation semantics, and engine catalogue disagree with the live API |
| Focus engine SDK | Y | P | Y | P | ? | ? | Build/typecheck/33 tests pass, but default routing sends Biofield and Face to a TS service that documents only Raaga and Sigil live |
| Universal tool server / CLI bridge | Y | P | P | N | ? | ? | Sends API keys as bearer JWTs instead of `X-API-Key`, stores plaintext `.selemenerc.json` without an ignore rule, and depends on optional Swagger output |
| TUI | Y | P | P | P | ? | ? | Hard-coded 16-ID catalogue omits three runtime IDs and declares connectivity when client construction succeeds without a health probe |
| Admin web | Y | P | P | P | ? | ? | Substantial UI typechecks, but retains retired password login, legacy localStorage bearer auth, and “all 16” messaging; delivery is external-state dependent |
| Witness pipeline | Y | P | P | P | ? | ? | Package registry has 16 IDs and omits Raaga, Financial Biosensor, and Biofield Capture; fresh package verification was not fully green |
| Witness Dyad API | Y | P | P | P | P | P | Executes seven hard-coded engines, advertises Biofield without executing it, and can label empty failed synthesis as LLM-powered |
| Deterministic asset generation | Y | Y | Y | Y | P | P | Rust `/assets/generate` creates deterministic seed artifacts and does not invoke the richer TypeScript witness pipeline |
| Verification package | Y | P | Y | N | NA | NA | 36 isolated tests pass, but the exported entrypoint is empty, default fixtures path is absent, and root verification does not integrate API/SDK/admin/bridge surfaces |
| Biofield domain/client packages | Y | P | N | N | NA | NA | Private source-only packages point `main` at TypeScript source; client omits its domain dependency and neither is independently buildable in the audited environment |
| Error contract | Y | P | P | P | P | P | Canonical trace-bearing JSON exists, but some middleware/handlers return plain text that the TS SDK attempts to JSON-parse unconditionally |
| Billing, entitlements, credits, and usage | Y | P | P | P | P | ? | Substantial conditional routes exist, but absent database/credentials can install a no-op emitter and no authenticated live journey was probed |
| Dodo webhook ingestion | Y | P | P | P | P | ? | Internal ingestion exists; signature, replay/idempotency, entitlement mutation, and production delivery need one durable contract receipt |
| Living Reading publication and invitations | Y | P | P | P | P | ? | Publication/invitation routes and safety tests exist; storage durability, lifecycle, authorization, and live delivery were not freshly verified |
| OpenClaw onboarding | Y | P | P | P | ? | ? | Onboarding routes exist, but invitation state is explicitly in-memory and restart/multi-instance behavior is not durable |

## Data, infrastructure, and operations ledger

| Capability | Dcl | Imp | Exe | Int | Dep | Ops | Evidence and correction |
|---|:---:|:---:|:---:|:---:|:---:|:---:|---|
| PostgreSQL persistence | Y | P | P | P | P | P | Readiness reports healthy, but runtime does not apply the migration set, the production image omits it, and migration-only changes are outside Railway watch paths |
| Migration discipline | Y | P | P | P | NA | P | Duplicate `007`, gaps, and CI `psql ... || true` make migration success a fail-open gate |
| Redis cache | Y | P | P | N | P | P | Cache manager exists and readiness is healthy, but calculation/workflow paths do not call it; L1 accounting and L2 `KEYS` behavior contradict stronger claims |
| Rust production image | Y | Y | P | Y | P | P | An image is built, but the live target is not tied to its digest; cache stage omits workspace manifests and suppresses failure with `|| true` |
| TypeScript image | Y | Y | Y | Y | ? | P | CI builds and pushes it; workflow does not deploy it, so live target is external-state dependent |
| Biofield CV image | Y | Y | P | P | ? | ? | Dedicated Railway configuration exists under `python-services` |
| MediaPipe image | Y | Y | P | P | N | N | Compose declares it; repository Railway config explicitly excludes it |
| Railway Rust deploy | Y | P | Y | P | P | P | Live service is ready, but deploy is independent of full CI, switches on liveness, has no automatic smoke-failure rollback, and cannot identify its source/image digest |
| Railway configuration | Y | P | P | P | P | P | All three TOMLs fail the current official schema (`deploy.watch`/`resources`); TS has competing TOML/JSON authority and Config-as-Code has a time-bound migration |
| Docker Compose | Y | P | P | P | NA | N | Config renders, but API image omits curl used by healthcheck, fresh ephemeris volume masks bundled data, and data/monitoring services bind unsafe development defaults |
| Kubernetes | Y | P | N | N | N | N | Dormant opt-in manifests have divergent image names, `latest` tags, placeholder Secret values, and optional ephemeris mount masking risk; not a valid fallback topology |
| Cloudflare workers | Y | P | ? | P | ? | ? | No uniform CI/deploy receipt; Pattern Memory is undeployed and unauthenticated, LLM proxy fails open when its token is unset, and rate limits are per-isolate |
| Monitoring and runbooks | Y | P | N | P | ? | N | Prometheus scrapes JSON as metrics, alerts target absent API webhooks, canary queries labels the counter lacks, and no active human receiver/continuous canary is evidenced |
| Main branch controls | Y | N | Y | N | NA | N | GitHub reported unprotected main; agent merge automation can merge without checking required status checks |
| Main Rust CI gates | Y | P | Y | P | NA | N | Latest main CI was red while deploy proceeded; toolchains/actions are mutable and security failure does not prevent production mutation |
| TypeScript CI typecheck | Y | Y | P | P | NA | N | Current workflow converts typecheck failure into success output |
| Python sidecar CI | Y | Y | P | P | NA | P | Tests/smokes exist; deployed parity is not proved |
| Secret and dependency security | Y | P | N | N | NA | N | Latest Rust audit reported two `h2` advisories; production pnpm audit reported 18 high advisories; admin is absent from CI and currently fails lint |
| Release workflow | Y | P | P | N | ? | N | Two tag release paths have different gates; one can publish after unit tests without security, integration, deployment, or smoke success; SemVer lineage is inconsistent |
| Runtime-critical assets | Y | Y | P | Y | P | P | Ephemeris and wisdom data are in-repo; ownership, provenance, size, and release inclusion need a manifest |
| Biofield capture artifacts | Y | P | P | P | ? | ? | Default storage is local `.runtime/biofield-artifacts`; production image creates `/app/data`, so durable persisted-media storage is unproved |
| Repository hygiene | Y | P | NA | P | NA | P | Historical reports, generated architecture files, large docs/media, and legacy top-level artifacts need classification rather than silent authority |

## Fresh probe receipt

On 2026-08-25, `https://selemene.tryambakam.space` returned:

- `/health/live`: HTTP 200, version `3.3.1`, 19 engines loaded, six workflows loaded.
- `/health/ready`: HTTP 200 with PostgreSQL, Redis, orchestrator, bridge, and six TypeScript engines healthy.
- `/api/v1/status` and `/api/v1/engines`: HTTP 401 with structured authentication errors.

The health payload does not contain a commit SHA, image digest, configuration revision, or deployment timestamp. It therefore cannot prove which source revision is running.

## Contradiction register

| Claim pattern | Contradicting evidence | Required correction |
|---|---|---|
| “All engines complete” | Several TypeScript/media engines use partial semantics, disabled services, heuristics, mocks, or placeholders | Report status per axis and path |
| “17 engines” versus “19 engines” | Runtime registry contains 19 IDs; only 17 are mirrors under the stated count policy | Always qualify the count |
| “Bridge has five engines” | Current registry and health expose six, including Raaga | Generate bridge documentation from registry |
| “Three Railway services deploy together” | GitHub deploys Rust directly; TS/Python/admin depend on separate or external delivery paths | Publish one deployment manifest and revision receipt |
| “Four migrations” | Repository contains 36 migration files through `037` | Regenerate persistence documentation |
| “Green CI proves release” | Migration and TS typecheck paths can fail open; root verification package is omitted | Make release-critical gates fail closed |
| “Healthy means operational” | Health does not exercise authenticated results, media providers, Python sidecars, or deployment identity | Add bounded dependency and journey probes |
| “Workflow synthesis is integrated” | Current API execution returns `synthesis: None`; richer executor is disconnected | Wire and test the intended executor or narrow the API claim |
| “TypeScript validation is bridged” | Rust requests `/engines/:id/validate`, but no TypeScript route/interface exists | Implement the route or remove the unsupported bridge method |
| “Witness covers the engine platform” | Dyad and package registries are hard-coded subsets and Biofield metadata overstates execution | Generate them from the canonical capability manifest |
| “SDK tests prove API compatibility” | Both general SDKs pass isolated tests while list envelopes, fields, methods, auth, witness, and validation differ from the API | Add generated contract and mock-server parity tests |
| “CLI authentication is production-ready” | CLI and generated LangChain tools send API keys as bearer tokens; server requires `X-API-Key` | Correct auth transport and add a protected-route smoke |
| “Cache is a calculation cache” | API exposes cache health/stats but calculation and workflow paths never use `get` or `store` | Integrate it or relabel the operational surface |
| “Migrations ship with the service” | Runtime/image omit the migration set and Railway ignores migration-only changes | Create a fail-hard deployable migrator |
| “Failed CI cannot reach production” | Latest main deploy reached Railway despite failed CI and then failed smoke; no rollback followed | Protect main and deploy only a verified immutable artifact |
| “Railway configuration is valid” | All three TOMLs fail current schema validation and TS has two competing configs | Migrate to one validated configuration authority |
| “Compose is an operational local topology” | Healthcheck requires absent curl and ephemeris mount masks bundled data | Repair and add an actual service-start smoke |
| “Kubernetes provides redundancy” | Dormant manifests do not agree on image, use placeholders/latest, and are not deployed | Quarantine or fully validate the topology |
| “Monitoring closes the loop” | Scrapes, PromQL labels, webhook routes, canary target, and receivers do not align | Build and exercise one real alert path |
| “Billing routes imply operational monetization” | Billing/entitlement/credit emitters are conditional and can become no-ops without required state/credentials | Add authenticated, revision-bound webhook-to-entitlement-to-usage receipts |
| “Invitations are durable” | OpenClaw invitation state is in-memory; publication/invitation lifecycle lacks a fresh multi-instance receipt | Define persistence, expiry, replay, authorization, and restart semantics |

## Current delivery incidents and security debt

| Severity | Evidence cut 2026-08-25 | Planning consequence |
|---|---|---|
| Critical | `main` was reported unprotected; CI run `32248132223` failed while CD run `32248132202` deployed Railway before four of seven API smokes failed | Stop independent source deploys; require protected checks, verified digest promotion, concurrency control, and rollback |
| Critical | Current security jobs report two Rust `h2` advisories and 18 high production Node advisories; admin lint has four React effect/state errors and is outside CI | Security and admin gates are Wave 0/1 blockers |
| High | All Railway TOMLs fail current schema; Railway Config-as-Code migration has a 2026-12-01 cutoff | Select and migrate one production configuration authority |
| High | Release workflows have incompatible gates and published version lineage regresses from `v3.4.0` to latest `v3.3.1` | Unify release authority before the next tag |
| High | Worker cost/data edges can fail open or lack authentication; monitoring has no demonstrated human alert path | Fail closed and add continuous operational receipts |

## Continuation overlay — 2026-09-05

The original 2026-08-25 baseline above is preserved. Current recovery evidence is [RECOVERY-2026-09-05.md](./RECOVERY-2026-09-05.md), with the [infrastructure map](./INFRASTRUCTURE-MAP.json), [dependency audit](./DEPENDENCY-AUDIT-2026-09-05.md), [CodeGraph receipt](./CODEGRAPH-2026-09-05.md), and [570-issue index](./ENGINE-ISSUE-INDEX.json). `ISA.md` remains the acceptance ledger; `.planning/ROADMAP.md` maps GSD phases to these original waves. Recovery and local passing checks do not close a wave or establish deployment.
