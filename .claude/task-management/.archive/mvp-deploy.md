# Selemene Engine — MVP Deployment Plan

## Railway + Supabase + Observability Stack

**Document Version:** 1.0
**Date:** February 2026
**Scope:** Deploy Tryambakam Noesis API (v2.1.0) to production-grade MVP infrastructure for initial test group (<50 users)
**Monthly Budget Target:** $31–41/month

---

## 1. Current State of the Codebase

The Selemene Engine (internally: Tryambakam Noesis) is a Rust-based consciousness calculation platform running 14 engines across 6 synthesis workflows. The API layer (`noesis-api` crate) is built on Axum and already includes production-grade middleware, but all infrastructure currently assumes `docker-compose` on a single host.

### What Already Exists and Works

**Compute layer (`crates/noesis-api`):**
The Axum HTTP server is fully operational. `main.rs` loads `ApiConfig::from_env()`, validates the config, initializes structured logging (pretty for dev, JSON for prod via `LOG_FORMAT` env var), builds application state containing the `WorkflowOrchestrator`, `CacheManager`, `AuthService`, and `NoesisMetrics`, then binds a TCP listener on the configured `HOST:PORT`. The router exposes health endpoints (`/health`, `/health/live`, `/health/ready`, `/ready`), a Prometheus metrics endpoint (`/metrics`), Swagger UI at `/api/docs`, versioned API routes under `/api/v1/`, and legacy backward-compatible routes under `/api/legacy/`.

The middleware stack is layered correctly: CORS (environment-based allowlist via `ALLOWED_ORIGINS`) → TraceLayer → request logging middleware (structured spans with `trace_id`, `span_id`, method, path, user_id) → auth middleware (JWT via `Authorization: Bearer` or API key via `X-API-Key` header) → rate limiting middleware (per-user sliding window using `DashMap`, configurable via `RATE_LIMIT_REQUESTS` and `RATE_LIMIT_WINDOW_SECS`). Rate limit responses include `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` headers.

**Authentication (`crates/noesis-auth`):**
The `AuthService` struct supports both JWT tokens (HS256, 24-hour expiry) and API keys stored in an in-memory `HashMap` behind a `RwLock`. The `Claims` struct includes `sub` (user ID), `exp`, `iat`, `tier` (free/premium/enterprise), `permissions` (vector of permission strings), and `consciousness_level` (0–5). The `ApiKey` struct mirrors this with additional fields for `created_at`, `expires_at`, `last_used`, and per-key `rate_limit`. Tier-based rate limits are hardcoded: free=60/min, premium=1000/min, enterprise=10000/min. Endpoint-level permissions exist for `/api/v1/panchanga`, `/api/v1/admin/*` routes.

**Critical gap:** API keys are currently stored only in-memory (`HashMap`). On restart, all keys are lost. There is no persistence layer for API keys or user accounts. There is no registration endpoint, no key generation API accessible to users, and no admin interface. Keys can only be added programmatically via `AuthService::add_api_key()`.

**Caching (`crates/noesis-cache`):**
Three-layer caching is implemented: L1 is an in-process LRU cache (configurable size via `CACHE_L1_SIZE`, default 256MB, TTL via `CACHE_L1_TTL`), L2 is Redis (optional, enabled when `REDIS_URL` is set, TTL via `CACHE_L2_TTL`), L3 is disk-based archive. The `CacheManager` exposes `health_check()` for the readiness probe.

**Metrics (`crates/noesis-metrics`):**
Prometheus metrics are fully registered: `noesis_requests_total`, `noesis_request_duration_seconds`, `noesis_active_connections`, `noesis_calculations_total`, `noesis_calculation_duration_seconds`, `noesis_calculation_errors_total`, `noesis_swiss_ephemeris_usage_total`, `noesis_native_engine_usage_total`, `noesis_cache_hits_total`, `noesis_cache_misses_total`, plus per-engine breakdowns via `IntCounterVec` and `HistogramVec` keyed by `engine_id` and `status`. OpenTelemetry tracing is wired to export to an OTLP endpoint (default: `http://jaeger:4317`). The `MetricsCollector` runs a background task every 30 seconds, though system metrics (memory, CPU) are still stubbed with `0.0` placeholders.

**External API integration (`crates/noesis-vedic-api`):**
The FreeAstrologyAPI.com client has a robust resilience layer: circuit breaker (`circuit_breaker.rs`), retry logic (`retry.rs`), rate limiter (`rate_limiter.rs`), and response caching (`cached_client.rs`). The free tier allows 50 requests/day with 1 request/second throttling. Configuration is fully environment-driven via `FREE_ASTROLOGY_API_KEY`, `FREE_ASTROLOGY_API_BASE_URL`, timeout, retry count, cache TTLs, and provider fallback settings. When `VEDIC_ENGINE_FALLBACK_ENABLED=true`, the system falls back to native Rust calculations when the API is unavailable or rate-limited.

**Docker:**
`Dockerfile` uses a multi-stage build: `rust:1.75` builder → `debian:bookworm-slim` runtime. The production `Dockerfile.prod` adds LTO and binary stripping. Swiss Ephemeris data, wisdom docs, and constants data are baked into the image at `/app/data/`. The `docker-compose.yml` orchestrates three services: `noesis-api` (port 8080), `redis` (Redis 7 Alpine, port 6379), and `postgres` (PostgreSQL 16 Alpine, port 5432) with health checks, persistent volumes, and a bridge network.

**CI/CD (`.github/workflows/`):**
`test.yml` runs lint (rustfmt + clippy), unit tests, integration tests (with Redis service container), security audit (rustsec), release build check, and TypeScript engine lint/tests. `release.yml` triggers on version tags, builds Docker images to GHCR, compiles cross-platform binaries (Linux x86_64, macOS x86_64, macOS ARM64), creates GitHub releases, and updates CHANGELOG.

**OpenAPI Documentation:**
Already generated via `utoipa` with SwaggerUI served at `/api/docs` and the OpenAPI JSON spec at `/api/openapi.json`. Security schemes for both Bearer JWT and API Key (`X-API-Key`) are defined. All engine and workflow endpoints are documented with request/response schemas.

### What Does NOT Exist Yet

1. **Persistent API key storage** — keys live in-memory and vanish on restart
2. **User registration or key management API** — no way for test users to self-serve
3. **Database schema or migrations** — Postgres is in docker-compose but not used by the application
4. **Sentry or any error tracking integration** — errors go to stdout only
5. **Product analytics** — no Posthog or equivalent event capture
6. **Uptime monitoring** — no external health check service
7. **Custom domain or DNS configuration** — only `localhost` access
8. **Railway-specific deployment configuration** — `railway.toml`, `Procfile`, or equivalent
9. **Supabase connection pooling configuration** — `DATABASE_URL` currently points at local Postgres
10. **API key seeding script** — no way to bootstrap initial test user keys
11. **CORS configuration for production domain** — `ALLOWED_ORIGINS` defaults to localhost:3000/5173

---

## 2. Target Architecture

```
┌─ RAILWAY (Starter Plan) ───────────────────────────────────┐
│                                                             │
│  ┌─────────────────────────────────┐                       │
│  │ noesis-api (Docker)             │                       │
│  │ ├── Axum HTTP :8080             │                       │
│  │ ├── 14 engines (9 Rust + bridge)│                       │
│  │ ├── 6 synthesis workflows       │                       │
│  │ ├── Auth (JWT + API Key)        │                       │
│  │ ├── Rate limiter (DashMap)      │                       │
│  │ ├── L1 cache (in-memory LRU)    │                       │
│  │ ├── Prometheus /metrics         │                       │
│  │ ├── SwaggerUI /api/docs         │                       │
│  │ └── Ephemeris data (baked in)   │                       │
│  └──────────────┬──────────────────┘                       │
│                 │                                           │
│  ┌──────────────▼──────────────────┐                       │
│  │ Redis (Railway Add-on)          │                       │
│  │ └── L2 cache layer              │                       │
│  └─────────────────────────────────┘                       │
│                                                             │
└─────────────────────────┬───────────────────────────────────┘
                          │
              ┌───────────▼────────────┐
              │ SUPABASE (Pro Plan)    │
              │ ├── PostgreSQL 15      │
              │ │   ├── api_keys table │
              │ │   ├── users table    │
              │ │   └── usage_logs     │
              │ ├── Supavisor pooling  │
              │ └── Dashboard + SQL    │
              └────────────────────────┘
                          │
┌─────────────────────────┴───────────────────────────────────┐
│ OBSERVABILITY                                                │
│  Sentry (Free)      → Error tracking, Rust stack traces     │
│  Posthog (Free)     → Product analytics, engine usage events│
│  BetterStack (Free) → Uptime monitoring, /health pings      │
│  Railway Logs       → Structured JSON request logs           │
│  Prometheus /metrics→ Engine latency, cache hit rate, errors │
└──────────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────┴───────────────────────────────────┐
│ INFRASTRUCTURE                                               │
│  Cloudflare (Free)  → DNS, SSL termination, DDoS protection │
│  GitHub Actions     → CI/CD (existing test.yml + release.yml)│
│  Custom Domain      → api.tryambakam.com (or similar)        │
└──────────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────┴───────────────────────────────────┐
│ EXTERNAL DEPENDENCY                                          │
│  FreeAstrologyAPI.com → 50 req/day (free tier)              │
│  └── Circuit breaker + native fallback already implemented   │
└──────────────────────────────────────────────────────────────┘
```

---

## 3. Task Breakdown by Phase

### Phase 1: Database Layer — Supabase Integration

The most foundational change. Right now, Postgres exists in docker-compose but the Rust application doesn't connect to it for anything. The auth crate stores API keys in a `HashMap`. This phase wires Supabase Postgres into the auth system.

**Task 1.1: Create Supabase project and database schema**

Provision a Supabase project. Create the following tables:

- `api_keys` — stores hashed API keys with `key_hash` (SHA-256), `user_id`, `tier`, `permissions` (JSONB), `consciousness_level`, `rate_limit`, `created_at`, `expires_at`, `last_used`, `is_active`. Index on `key_hash`.
- `users` — stores `user_id` (UUID), `email` (optional for MVP), `tier`, `consciousness_level`, `created_at`. The `api_keys` table references this via foreign key.
- `usage_logs` — stores `user_id`, `engine_id`, `workflow_id` (nullable), `status`, `duration_ms`, `created_at`. Partitioned by month for future scale. This replaces in-memory metrics for historical analysis.

No ORM. Use `sqlx` with compile-time checked queries against the Supabase connection string. Add `sqlx` and `sqlx-postgres` to `noesis-auth/Cargo.toml`. Use `sqlx::migrate!()` macro to run migrations on application startup — this ensures schema is always up to date without a separate migration step.

The `api_keys` table design deserves attention. The current in-memory `ApiKey` struct stores the plaintext key. In Postgres, store only the SHA-256 hash. When validating, hash the incoming `X-API-Key` header value and query against `key_hash`. This means even a database breach doesn't expose usable keys. The `permissions` column should be JSONB rather than a text array — it gives you flexibility to add structured permission objects later without schema migration.

The `usage_logs` table serves double duty: it replaces the need for a separate analytics database for API usage patterns, and it provides an audit trail for debugging. Insert asynchronously (Tokio spawn) so logging never blocks the request path. Include `request_path`, `engine_id`, `response_status`, and `duration_ms`. Partition by month from day one — it costs nothing to set up and prevents the table from becoming a performance bottleneck as usage grows.

**Task 1.2: Migrate `AuthService` from in-memory to Postgres-backed**

Replace the `HashMap<String, ApiKey>` in `AuthService` with a connection pool (`sqlx::PgPool`). The `validate_api_key()` method becomes a database query: `SELECT * FROM api_keys WHERE key_hash = $1 AND is_active = true AND (expires_at IS NULL OR expires_at > NOW())`. Update `last_used` asynchronously (fire-and-forget, don't block the auth path). Keep the `validate_jwt_token()` path unchanged — JWTs are stateless and don't need the database.

**Task 1.3: Create API key seeding script**

Build a CLI tool or Rust binary (`seed_api_keys.rs`) that generates API keys for your initial test group. Each key is a 32-character random string. Store the SHA-256 hash in the database. Output the plaintext keys once (they cannot be recovered). This replaces the `generate_test_credentials.rs` example.

**Task 1.4: Add `DATABASE_URL` to `ApiConfig`**

Extend `ApiConfig::from_env()` to read `DATABASE_URL`. It should be optional for local development (auth falls back to in-memory when absent) and required when `RUST_ENV=production`. Add validation: connection string must start with `postgresql://` or `postgres://`.

### Phase 2: Railway Deployment

**Task 2.1: Create `railway.toml` configuration**

Railway auto-detects Dockerfiles but explicit configuration prevents surprises. Create a `railway.toml` that specifies the build path (`Dockerfile.prod`), sets `PORT=8080`, configures the health check endpoint (`/health` with 30-second interval, 5-second timeout, 3 retries before marking unhealthy), and sets the restart policy to `always`. Also specify the Railway region — choose the one closest to your primary test user base.

Important detail: Railway injects `PORT` as an environment variable. The existing `ApiConfig::from_env()` already reads `PORT` with a fallback to 8080. However, Railway expects the app to bind to `0.0.0.0:$PORT`. The current `HOST` default is already `0.0.0.0`, so no code change is needed — but verify this explicitly in the Railway deployment logs on first deploy.

**Task 2.2: Configure Railway environment variables**

Set all env vars in Railway's dashboard. This is the production-equivalent of `.env.example`. Critical variables: `RUST_ENV=production`, `JWT_SECRET` (generate 64-char random), `DATABASE_URL` (Supabase connection string with `?sslmode=require`), `REDIS_URL` (auto-injected by Railway Redis add-on), `LOG_FORMAT=json`, `RUST_LOG=info`, `ALLOWED_ORIGINS` (your production domain), `FREE_ASTROLOGY_API_KEY`, `VEDIC_ENGINE_PROVIDER=api`, `VEDIC_ENGINE_FALLBACK_ENABLED=true`.

**Task 2.3: Provision Railway Redis add-on**

Add the Redis add-on to the Railway project. Railway injects `REDIS_URL` automatically. The existing `ApiConfig::from_env()` already reads `REDIS_URL` as optional — when present, L2 cache activates. No code changes required.

**Task 2.4: Deploy and verify**

Push to the GitHub branch connected to Railway. The first deploy will take 10–15 minutes (full Rust compilation). Monitor the Railway deployment logs for compilation errors — the most common issues are missing system libraries in the Docker runtime image (the production Dockerfile uses `debian:bookworm-slim` which may be missing `libssl-dev` or `ca-certificates` depending on your dependency tree).

Verification checklist (run these in order):

1. `curl https://<railway-url>/health` — must return `{"status":"ok", "version":"0.1.0", "uptime_seconds":N, "engines_loaded":9, "workflows_loaded":6}`. If `engines_loaded` is 0, the orchestrator failed to initialize — check logs for engine registration errors.
2. `curl https://<railway-url>/health/ready` — must return `{"redis":"ok", "orchestrator":"ready", "overall_status":"ready"}` with HTTP 200. If Redis shows "down", the `REDIS_URL` env var isn't being injected correctly by the Railway add-on.
3. `curl https://<railway-url>/api/docs` — SwaggerUI should load in browser. If it returns 404, the `utoipa-swagger-ui` feature may not be compiled in the release build.
4. `curl https://<railway-url>/metrics` — should return Prometheus-format text with `noesis_` prefixed metrics.
5. Test authenticated request: `curl -X POST https://<railway-url>/api/v1/engines/numerology/calculate -H "X-API-Key: <seeded-key>" -H "Content-Type: application/json" -d '{"birth_data":{"date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"},"consciousness_level":3}'` — must return a valid numerology calculation, not a 401 or 500.
6. Test rate limiting: send 101 rapid requests and confirm the 101st returns HTTP 429 with `X-RateLimit-Remaining: 0`.

**Task 2.5: Optimize Dockerfile for Railway build cache**

Railway supports Docker layer caching. Restructure `Dockerfile.prod` to maximize cache hits: copy `Cargo.toml` and `Cargo.lock` first, run `cargo build --release` with dummy `src/main.rs` to cache dependency compilation, then copy actual source and rebuild. This drops incremental deploy times from 10–15 minutes to 2–3 minutes.

### Phase 3: DNS and Domain — Cloudflare

**Task 3.1: Register domain and configure Cloudflare DNS**

Point your domain (`tryambakam.space`) at Railway's provided URL via CNAME record. Enable Cloudflare proxy (orange cloud) for DDoS protection and SSL termination. Set SSL mode to "Full (strict)."

**Task 3.2: Update CORS and allowed origins**

Change `ALLOWED_ORIGINS` in Railway env vars to include the production domain. If you'll have a frontend later, include that domain too. The existing `create_cors_layer()` in `lib.rs` handles this automatically.

**Task 3.3: Configure Cloudflare caching rules**

Set a page rule for `/health` — cache for 30 seconds (prevents health check storms). Set a rule for `/api/docs` and `/api/openapi.json` — cache for 1 hour (static content). All `/api/v1/*` routes should bypass cache (dynamic, authenticated).

### Phase 4: Error Tracking — Sentry

**Task 4.1: Create Sentry project and install `sentry-rust`**

Add `sentry = "0.34"` and `sentry-tower = { version = "0.34", features = ["http"] }` to `noesis-api/Cargo.toml`. In `main.rs`, initialize the Sentry guard *before* tracing initialization — Sentry needs to capture panics from the earliest possible point. The initialization belongs right after `ApiConfig::from_env()` and `config.validate()`:

```rust
let _sentry_guard = sentry::init((
    std::env::var("SENTRY_DSN").ok(),
    sentry::ClientOptions {
        release: sentry::release_name!(),
        traces_sample_rate: 0.1,
        environment: Some(std::env::var("RUST_ENV").unwrap_or("development".into()).into()),
        ..Default::default()
    }
));
```

The `_sentry_guard` variable must stay alive for the entire application lifetime. When it drops (on shutdown), it flushes any pending events. Using `Option` for the DSN means Sentry gracefully no-ops in local development when the env var is absent.

**Task 4.2: Integrate Sentry with Axum middleware**

Add `SentryHttpLayer` from `sentry-tower` to the router's layer stack. Position it *outside* the auth and rate limit layers so it captures errors from all middleware, not just handlers. The existing `ErrorResponse` struct in `middleware.rs` already includes `error_code` and `details` — pipe these into Sentry context using `sentry::configure_scope()` within the error handling paths of `calculate_handler` and `workflow_execute_handler`. This enriches Sentry events with the specific engine that failed, the user's tier, and the consciousness level — context that makes debugging meaningful rather than just showing a generic 500.

For the `EngineError` variants already mapped in `lib.rs` (there are 12 of them: `InvalidInput`, `CalculationError`, `EngineNotFound`, `AuthError`, `PhaseAccessDenied`, `CacheError`, `BridgeError`, `ConfigError`, `TimeoutError`, `RateLimitError`, `ExternalApiError`, `InternalError`), add fingerprinting so Sentry groups errors by engine + error type rather than creating a new issue for every occurrence.

**Task 4.3: Add `SENTRY_DSN` to Railway env vars**

Store the DSN as an environment variable. The Sentry client reads it automatically. Disable in development by omitting the env var (Sentry's `init` gracefully no-ops without a DSN).

### Phase 5: Product Analytics — Posthog

**Task 5.1: Create Posthog project and add analytics middleware**

Write a thin Axum middleware layer that fires a Posthog event after each API request completes. Use Posthog's HTTP capture API — it's a single POST to `https://app.posthog.com/capture` with `{event, distinct_id, properties}`. No SDK dependency required, just `reqwest` (already in the dependency tree). Fire asynchronously (spawn a Tokio task) so analytics never block the response path. If the Posthog request fails, log a warning and move on — analytics are never worth degrading user-facing latency.

Design these specific events:

- `engine_calculation` — fires on every `POST /api/v1/engines/:engine_id/calculate`. Properties: `engine_id`, `user_id`, `tier`, `consciousness_level`, `status_code`, `duration_ms`, `cache_hit`. This tells you which engines are popular and where performance bottlenecks emerge.
- `workflow_execution` — fires on every `POST /api/v1/workflows/:workflow_id/execute`. Properties: `workflow_id`, `user_id`, `tier`, `engines_count`, `engines_succeeded`, `engines_failed`, `total_duration_ms`. This reveals whether workflows degrade gracefully or fail catastrophically.
- `auth_failure` — fires on every 401 response from `auth_middleware`. Properties: `auth_method` (jwt or api_key), `error_reason`, `ip_hash` (hashed, not raw IP). This surfaces brute-force attempts and misconfigured clients.
- `rate_limit_hit` — fires on every 429 response. Properties: `user_id`, `tier`, `limit`, `window_seconds`. This tells you which users are bumping against their quotas.

Use `user_id` as the `distinct_id` for Posthog. For unauthenticated requests (health checks, docs), use a static identifier like `anonymous`. This keeps your Posthog user count accurate to real API consumers.

**Task 5.2: Add `POSTHOG_API_KEY` to Railway env vars**

Make analytics conditional — when `POSTHOG_API_KEY` is absent, the middleware becomes a no-op passthrough.

### Phase 6: Uptime Monitoring — BetterStack

**Task 6.1: Configure external health monitors**

Create monitors in BetterStack pointing at:
- `https://api.tryambakam.com/health` — checks every 3 minutes, alert on failure
- `https://api.tryambakam.com/health/ready` — checks every 5 minutes, alert on dependency degradation

Configure alerts to your preferred channel (email, Slack, SMS). Optionally enable the public status page — it builds trust with test users and costs nothing.

### Phase 7: API Key Management and Onboarding

**Task 7.1: Build a minimal admin endpoint for key generation**

Add a `POST /api/v1/admin/keys` endpoint protected by a master admin API key (or a specific `admin:keys` permission). This endpoint generates a new API key, hashes it, inserts into Supabase, and returns the plaintext key once. This lets you onboard test users without running SQL manually.

Request body: `{ "user_id": "...", "tier": "free", "consciousness_level": 3, "expires_in_days": 90 }`.
Response: `{ "api_key": "nsk_...", "expires_at": "...", "note": "Store this key securely. It cannot be retrieved again." }`.

**Task 7.2: Write onboarding documentation for test users**

A single-page markdown doc explaining: here's your API key, here's how to use it (`X-API-Key` header), here are the available engines and workflows, here's the SwaggerUI URL. Include 3 working curl examples: one engine calculation, one workflow execution, and the health check.

### Phase 8: TypeScript Engine Bridge (If Needed for MVP)

**Task 8.1: Assess which TS engines are required for MVP**

The 5 TypeScript engines (Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge) run on a separate Bun server at port 3001. The `noesis-bridge` crate communicates with them via HTTP. Decide whether the MVP needs these engines. If only the 9 Rust engines suffice, skip this phase entirely — it saves a second Railway service.

**Task 8.2: Deploy TS engines as a second Railway service (if needed)**

Create a second service in Railway from the `ts-engines/` directory. Configure the bridge URL in the Rust API's environment: `TS_ENGINES_URL=https://<railway-ts-url>`. The `noesis-bridge` crate's circuit breaker handles TS engine unavailability gracefully — workflows degrade to Rust-only engines if the bridge is down.

---

## 4. Environment Variable Reference (Production)

This is the complete set of env vars needed in Railway for the production deployment. Variables marked `[NEW]` do not exist in the current `.env.example` and must be added.

| Variable | Value | Source |
|---|---|---|
| `RUST_ENV` | `production` | Railway |
| `HOST` | `0.0.0.0` | Railway |
| `PORT` | `8080` | Railway |
| `JWT_SECRET` | 64-char random | Railway (secret) |
| `DATABASE_URL` | `postgresql://...?sslmode=require` | Supabase |
| `REDIS_URL` | Auto-injected | Railway Redis |
| `ALLOWED_ORIGINS` | `https://tryambakam.space` | Railway |
| `RATE_LIMIT_REQUESTS` | `100` | Railway |
| `RATE_LIMIT_WINDOW_SECS` | `60` | Railway |
| `REQUEST_TIMEOUT_SECS` | `30` | Railway |
| `RUST_LOG` | `info` | Railway |
| `LOG_FORMAT` | `json` | Railway |
| `FREE_ASTROLOGY_API_KEY` | Your key | Railway (secret) |
| `FREE_ASTROLOGY_API_BASE_URL` | `https://json.freeastrologyapi.com` | Railway |
| `VEDIC_ENGINE_PROVIDER` | `api` | Railway |
| `VEDIC_ENGINE_FALLBACK_ENABLED` | `true` | Railway |
| `SENTRY_DSN` | `[NEW]` Sentry DSN | Railway (secret) |
| `POSTHOG_API_KEY` | `[NEW]` Posthog key | Railway (secret) |

---

## 5. Deployment Sequence

Order matters. Each phase builds on the previous.

```
Week 1: Phase 1 (Supabase schema + auth migration)
         Phase 2 (Railway deployment + Redis)
         → MILESTONE: API accessible at Railway URL with persistent auth

Week 2: Phase 3 (Cloudflare DNS + domain)
         Phase 4 (Sentry error tracking)
         Phase 5 (Posthog analytics)
         → MILESTONE: Production domain, full observability

Week 3: Phase 6 (BetterStack uptime monitoring)
         Phase 7 (Admin key management + test user onboarding)
         → MILESTONE: First test users onboarded

Week 4: Phase 8 (TS engines if needed)
         Stabilization, bug fixes from test user feedback
         → MILESTONE: MVP complete, feedback loop running
```

---

## 6. Risk Register

**Auth migration risk:** Moving from in-memory to Postgres-backed API keys introduces a database dependency on the critical auth path. Mitigation: add a 5-second connection timeout and a fallback that returns 503 (Service Unavailable) rather than silently failing. The `sqlx::PgPool` handles connection pooling and reconnection automatically.

**Railway cold start risk:** On Railway Starter, containers stay warm as long as there's traffic. After extended idle periods, the first request may take 3–5 seconds while the container restarts. Mitigation: BetterStack's 3-minute health checks double as keep-alive pings.

**FreeAstrologyAPI rate limit risk:** With 50 requests/day on the free tier, a burst of test user requests could exhaust the quota. Mitigation: the existing caching layer (infinite TTL for birth data, 24-hour for daily panchang) and the native fallback engine mean most requests never hit the external API. Monitor with Posthog events and Sentry if the circuit breaker trips repeatedly.

**Supabase connection limit risk:** Supabase Pro allows up to 200 concurrent connections with Supavisor pooling. For <50 users, this is more than sufficient. Ensure `DATABASE_URL` uses the Supavisor pooler endpoint (port 6543) rather than direct connection (port 5432) to avoid exhausting connections during traffic spikes.

**Docker build time risk:** First Rust compilation on Railway takes 10–15 minutes. Task 2.5 (Dockerfile cache optimization) reduces subsequent builds to 2–3 minutes. Plan for the initial deploy to take a coffee break.

---

## 7. Success Criteria

The deployment is complete when:

1. `curl https://tryambakam.space/health` returns `200 OK` with engine and workflow counts
2. `curl https://tryambakam.space/health/ready` shows Redis=ok, orchestrator=ready
3. An API key persists across container restarts (Supabase-backed auth)
4. A full engine calculation (`POST /api/v1/engines/human-design/calculate`) completes with a valid API key
5. A workflow execution (`POST /api/v1/workflows/self-inquiry/execute`) synthesizes results from multiple engines
6. Sentry captures and reports a forced 500 error
7. Posthog shows engine usage events in the dashboard
8. BetterStack shows 100% uptime over 24 hours
9. SwaggerUI at `/api/docs` is accessible from the production domain
10. At least 3 test users have received API keys and successfully made authenticated requests

---

## 8. What This Plan Intentionally Does NOT Cover

- **User-facing frontend** — the MVP is API-only. Frontend comes after API stability is proven.
- **Supabase Auth integration** — unnecessary until there's a signup flow. API keys are sufficient for <50 users.
- **Kubernetes** — Railway's Docker abstraction is the right level for this scale.
- **CI/CD changes** — the existing GitHub Actions pipeline is solid. Railway's auto-deploy from GitHub handles the rest.
- **Performance optimization** — the engine already runs at sub-2ms. There is nothing to optimize at this scale.
- **Multi-region deployment** — single region is fine for MVP. Railway supports multi-region if needed later.
- **Log aggregation beyond Railway** — Railway Starter includes log persistence and search. Combined with Sentry and Posthog, this is sufficient observability for MVP.