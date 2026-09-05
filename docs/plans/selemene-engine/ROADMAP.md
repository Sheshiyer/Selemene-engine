# Selemene Engine — Dependency-Ordered Roadmap

**Baseline:** 2026-08-25  
**Boundary:** Work in `Selemene-engine` only

This roadmap closes missing internal edges in dependency order. External products are compatibility contexts, never implementation owners in this plan. A wave exits only when its named evidence changes the corresponding row in the [capability ledger](./CAPABILITY-LEDGER.md).

## Wave 0 — Authority, evidence, and inventory

**Purpose:** Stop status drift before changing runtime behavior.

1. Make the 19-ID engine registry machine-readable and distinguish 17 mirrors from the composed and operational IDs.
2. Generate or contract-test every internal catalogue: API schema, bridge, Rust SDK, TS SDK, CLI/tool server, TUI, admin, and documentation.
3. Adopt the six completion axes from the ledger in issue, release, and project status templates.
4. Add a deployment receipt containing commit SHA, image digest, build timestamp, service role, schema revision, and enabled optional dependencies.
5. Classify historical plans, generated architecture files, reports, fixtures, media, and top-level legacy assets as current, generated, archived, or runtime-critical.

**Exit:** One registry fixture fails on catalogue drift; the target deployment receipt schema is accepted; every active subsystem and asset class has an evidence row and owner; no active document uses an unqualified “all engines complete” claim. Wave 0 creates authority only—Wave 1 constructs and then enforces the gates.

## Wave 1 — Reproducible build and fail-closed gates

**Purpose:** A green build must mean the required product was actually validated.

1. Correct `Dockerfile.prod` manifest caching for every workspace member and remove broad `cargo build ... || true` suppression.
2. Make migration application fail closed and validate ordering, duplicate numbers, gaps, upgrade, and rollback policy.
3. Package and run a deployable migrator; include migrations in production artifacts and Railway watch paths before stateful release claims.
4. Make TypeScript typecheck, admin smoke, and required service smokes fail when their release target is in scope; use explicit opt-out jobs for intentionally absent targets.
5. Turn the verification package into a real consumable runner with valid default fixtures, then define a single release gate spanning Rust, TypeScript, Python, packages, migrations, and image builds.
6. Pin Rust/Bun versions and third-party actions to reviewed versions or immutable commits.
7. Add admin lint/typecheck/build/audit and both Python sidecars to the required matrix; orchestrate bridge integration tests with their service dependencies.
8. Clear current critical security debt: both reported Rust `h2` advisories and the 18 high production Node advisories.
9. After the new checks are reproducibly green, protect `main` and require them for human and agent merges.
10. Then stop independent source deployment from unverified pushes; promote one immutable tested image digest with deployment concurrency and automatic rollback on failed smoke.

**Exit:** A clean checkout produces every declared artifact; injected failures in migrations, typecheck, security, admin, and required smokes make CI fail; protected checks prevent a deliberately red commit from deploying; only the verified digest is promoted and failed smoke rolls it back; the receipt records toolchains and digests.

## Wave 2 — Contract and routing convergence

**Purpose:** One request and result model must survive every internal boundary.

1. Define canonical versioned request, result, error, consent, provenance, and capability-discovery schemas.
2. Freeze list envelopes, engine fields, HTTP methods, API-key headers, witness input, validation behavior, and a universal JSON error body in generated contract tests.
3. Generate or validate Rust, TypeScript, OpenAPI, SDK, CLI, TUI, admin, and bridge types against those schemas; replace isolated SDK mocks that reproduce incorrect server behavior.
4. Encode runtime kind and dependency requirements in the engine registry so unavailable TypeScript bridges, database-conditional engines, and Python paths cannot appear silently executable.
5. Add contract suites for authentication, native/TS/Python routing, timeout, partial dependency failure, version skew, and unknown engine IDs.
6. Implement or remove the Rust bridge's unsupported `/engines/:id/validate` call and replace default healthy self-checks with meaningful per-engine checks.
7. Connect the public workflow path to the intended synthesis executor, complete or narrow generic synthesis types, and expose partial engine failures.
8. Remove stale five-engine comments and duplicated hand-maintained catalogues.

**Exit:** Cross-language fixtures round-trip byte-for-byte where required; runtime discovery reports declared, available, and degraded states separately; registry drift fails CI.

## Wave 3 — Engine and media truth

**Purpose:** Complete means substantive semantics with explicit provenance, not a plausible fallback.

1. Close Tarot input/mode/replay and producer-renderer semantics.
2. Correct and reference-test I Ching casting and relating-hexagram behavior.
3. Define the Sacred Geometry artifact contract and implement or explicitly retire generated-geometry claims.
4. Make Sigil provider capability reflect real configured/reachable behavior; label mock and placeholder outputs as non-production provenance.
5. Define Raaga tiers—deterministic theory, local synthesis, hosted clip—and report each independently; retain consent boundaries.
6. Separate Biofield calculation, capture persistence, and Biofield CV analysis in names, routes, health, and documentation.
7. Make Face Reading's MediaPipe and heuristic paths distinct, testable provenance classes; decide whether MediaPipe is a supported production service.
8. Preserve calculated Gene Keys line data; remove Financial Biosensor's duplicate Human Design work and surface omitted-source failures.
9. Add explicit Panchanga ephemeris/fallback provenance to every affected result.

**Exit:** Every partial row has either real-path fixtures and operational checks or an explicitly documented reduced scope; no fallback can be mistaken for provider output.

## Wave 4 — Persistence, auth, and operational integrity

**Purpose:** Stateful and protected behavior must be reproducible under failure.

1. Inventory all 36 migrations and reconcile duplicate/gapped numbering with the production schema.
2. Test clean install, upgrade from supported revisions, concurrent migration protection, and backup/restore.
3. Verify cache invalidation, Redis degradation, database-conditional registration, auth tiers, rate limiting, API-key lifecycle, and sensitive-field redaction.
4. Wire the cache manager into intended calculation/workflow paths or relabel it as infrastructure-only; fix L1 replacement accounting and replace Redis `KEYS` with bounded scanning.
5. Resolve the password-versus-Cloudflare auth model, distributed token revocation, API-key logout semantics, and plain-text error exceptions.
6. Make billing, entitlement, credit, and usage state fail explicitly rather than installing silent no-op behavior; prove the Dodo webhook signature, replay/idempotency, mutation, and retry chain.
7. Define durable Living Reading publication/invitation storage, authorization, expiry, revocation, and recovery semantics.
8. Replace OpenClaw's in-memory invitation state for any multi-instance/production claim and test restart, expiry, replay, and tenant boundaries.
9. Add bounded dependency health for Python sidecars and configured media providers without making paid generation calls or exposing secrets.
10. Move Biofield capture artifacts to declared durable storage and establish retention/deletion rules for readings, captures, witness assets, usage logs, and generated media.

**Exit:** A disposable environment reaches the expected schema and passes authenticated stateful journeys; degraded dependencies produce typed, privacy-safe states.

## Wave 5 — Repository-owned distribution surfaces

**Purpose:** SDK, CLI/TUI, admin, and packages become release products rather than source directories.

1. Define compatibility and versioning policy across API, Rust SDK, TypeScript SDK, bridge, CLI/tool server, TUI, witness pipeline, and admin.
2. Pack/install every publishable package in an isolated consumer fixture; validate exports, declarations, binaries, license, provenance, and registry metadata.
3. Make Biofield domain/client packages independently buildable with declared dependencies and compiled exports.
4. Correct CLI API-key transport, secret storage and ignore rules, schema acquisition, workspace inclusion, and generated-client tests.
5. Replace TUI/admin hard-coded catalogues and connection assumptions with dynamic capability and health data.
6. Drive Witness Dyad and witness-pipeline engine selection from the canonical manifest and make provider/model/fallback receipts truthful.
7. Either wire `/assets/generate` to the rich witness pipeline or name and document it as deterministic preview generation.
8. Produce signed checksums or attestations and a release manifest linking packages and images to one source revision.
9. Test only Selemene's side of compatibility fixtures for Sankalpa, Urania, FalseEarth, and Raycast; file downstream work in those repositories separately when authorized.

**Exit:** Each supported distribution surface has an owner, version, artifact, installation smoke, compatibility range, and deprecation path.

## Wave 6 — Deployment, observability, and asset governance

**Purpose:** Make the declared topology and the running topology the same inspectable system.

1. Publish a service manifest for Rust API, TypeScript engines, Biofield CV, optional MediaPipe, admin, workers, PostgreSQL, and Redis with target, owner, config source, and revision.
2. Declare Railway the sole production topology unless another target is independently validated; migrate all services to one current-schema configuration authority before 2026-12-01.
3. Switch deployment activation to readiness, deploy/version TS and selected Python services explicitly, and bind the running artifact to its verified digest.
4. Quarantine dormant Kubernetes until image names, secrets, namespaces, policies, ephemeris mounts, tags, and a real cluster smoke are valid.
5. Repair Compose health tooling, ephemeris mounts, loopback bindings, credentials, and Docker-socket exposure; prove a clean local stack start.
6. Make deployment workflows cover or explicitly exclude each service; remove ambiguous dependence on unrecorded Railway/Vercel/Cloudflare integrations.
7. Fix metrics endpoints, names, labels, PromQL, canary targets, webhook receivers, and human escalation; run continuous synthetic probes and recovery drills.
8. Require authentication, tenant/body/rate limits, secret rotation, dry-runs, and deployment receipts at every Worker edge.
9. Create an asset manifest for ephemeris, wisdom, fixtures, generated media, diagrams, and brand files including provenance, license, checksum, size, consumer, and release inclusion.
10. Move large tracked media to an intentional LFS/object-storage policy and prune local caches/worktrees only through recoverable, user-approved operations.
11. Regenerate overview, service inventory, OpenAPI, engine tables, and deployment docs from the accepted registries/manifests.

**Exit:** An operator can map any live response to source and schema revisions, identify every required service and asset, and execute a tested recovery runbook.

## Sequencing rule

Do not begin broad feature expansion before Waves 0 and 1 are closed. Waves 2 and 3 may run in parallel only after registry and build truth are hard gates. Wave 4 precedes claims about authenticated production journeys. Waves 5 and 6 turn the verified core into releaseable, observable platform delivery.

## Continuation overlay — 2026-09-05

The original 2026-08-25 baseline above is preserved. Current recovery evidence is [RECOVERY-2026-09-05.md](./RECOVERY-2026-09-05.md), with the [infrastructure map](./INFRASTRUCTURE-MAP.json), [dependency audit](./DEPENDENCY-AUDIT-2026-09-05.md), [CodeGraph receipt](./CODEGRAPH-2026-09-05.md), and [570-issue index](./ENGINE-ISSUE-INDEX.json). `ISA.md` remains the acceptance ledger; `.planning/ROADMAP.md` maps GSD phases to these original waves. Recovery and local passing checks do not close a wave or establish deployment.
