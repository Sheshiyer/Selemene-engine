# Changelog

All notable changes to the Tryambakam Noesis Engine project.

## [3.2.0] - 2026-05-05

### Added

**WitnessOS Biofield Viewer — Geometry-First UI (`feat/viewer-metrics-ui`)**

*Biofield cosmogram v4*
- Borderless floating geometry panels — 6 metrics (ENERGY / SYMMETRY / COHERENCE / COMPLEXITY / REGULATION / COLOR FIELD) with correct PIP spec names (Light Quanta, Bilateral, Hurst·Pattern, Fractal Dim, OFA·Lyapunov, GLCM·Spectral)
- SVG 800×800 viewBox, arc progress rings + geometric icons per metric, click-to-collapse panels
- Responsive layout: `flex: 1 1 0` container (min 42dvh, max 72dvh)

*Viewer page — WitnessOS void-field redesign*
- Full `#070B1D` void canvas — no gradients, no card backgrounds, no borders
- Right panel divider replaced with thin gold gradient geometry rule (absolute positioned SVG-style div)
- Witness Dyad: all card borders/backgrounds removed → floating text with diamond SVG glyphs as agent markers
- Agent text floats with left-padding indent; synthesis separated by thin emerald geometry line
- Capture result: no card — inline separator + mono metric baseline rows
- Session strip: thin gold geometry gradient line replaces solid border-top; account/session/tier as single baseline row

*PIP viewer panel — circular portal*
- Camera feed clipped to circle via `borderRadius: 50%` + `overflow: hidden`
- Radial vignette blends circle edges into void field
- `PortalRings` SVG overlay: concentric dot rings, 12 radial spokes, 4 metric arc sectors in ring halo band
  - COH / SYM / LUM / REG arcs sweep proportional to live metric values, colored per consciousness spectrum
- Status pills replaced with 3 floating geometry dots (top-center of portal ring)
- Action bar replaced with geometry node buttons: diamond glyph + spaced-caps labels (OPEN FIELD / CAPTURE / PAUSE / CLOSE)
- Idle state: animated flower-of-life + concentric dot rings geometry placeholder
- Calibrating state: spinning gold arc ring + pulsing emerald dot overlay
- MetricRow progress bars replaced with `LiveMetricArcs` SVG arcs embedded in portal ring

*NVIDIA NIM tier-aware LLM witness*
- `crates/noesis-witness/src/llm.rs` — tier-aware client using NVIDIA NIM / OpenRouter
- Model mapping: free → kimi-k2/minimax, standard → gpt-oss-120b, enterprise → nemotron-49b
- Fallback chain: NVIDIA NIM → OpenRouter → rule-based witness
- `POST /api/v1/witness/interpret` multi-engine LLM endpoint: takes birth data + live biofield scores, invokes all available engines, synthesizes Aletheios/Pichet dyad via LLM

### Fixed

- GLSL shader `scale` constant undeclared — was referenced as `scale` in fragment shader but only documented in a comment; added `const float scale = 1.0 / PERIOD;` (≈ 16.7) to constants block, fixing `ERROR: 0:163: 'scale' : undeclared identifier` that prevented WebGL render loop from starting

### Changed

- `PIPViewerPanel` no longer uses `biofield-panel` / `biofield-form-panel` CSS classes — pure void-field container
- Viewer page right panel background: pure `#070B1D` (was blue gradient `#070B1D → #0a0e20 → #0E1428`)
- All inline `border` properties in viewer removed — geometry SVG lines define visual separation

---

## [3.1.0] - 2026-04-30

### Added

**API Input Validation Hardening (#462)**
- `validate_engine_input()` helper called at the API boundary in both `calculate_handler` and `execute_workflow_by_id` before any engine compute
- Validates `birth_data` coordinate bounds (latitude ∈ [-90, 90], longitude ∈ [-180, 180]) → 422 on violation
- Caps `options` map at 64 keys to prevent unbounded memory usage → 422 on violation
- Uses the standard `ErrorMapper::response` path so errors carry `trace_id`

**Security Audits (#459, #460)**
- `cargo audit` clean: 0 vulnerabilities; 6 unmaintained-crate warnings are pre-existing and noted
- JWT implementation confirmed OWASP-compliant: HS256 enforced, required claims `exp`/`iat`/`sub` validated, 24-hour expiry, no algorithm=none path

**API Migration Guide (#472)**
- New doc: `docs/api/migration-v2-to-v3.md` — covers base URL, auth changes, `EngineInput` schema, engine/workflow renames, response envelope changes, error format, and new endpoints

### Changed
- All crate versions bumped from `3.0.0` → `3.1.0` (#474)
- `health` endpoint now reports `version: "3.1.0"`

---

## [2.2.0] - 2026-04-30

### Added

**Biorhythm — Secondary Cycles & Compatibility**
- `spiritual` (53-day) and `aesthetic` (43-day) cycles now fully integrated as `CycleResult` fields in all engine outputs (#371, #372)
- `spiritual` cycle included in critical-day detection across 6 cycles (#372)
- 7-day `forecast` array includes `aesthetic` and `spiritual` percentage values for every day
- `partner_birth_date` option: pass a partner's birth date in `EngineInput.options` to receive a `compatibility` block with per-cycle scores (physical, emotional, intellectual, intuitive) and an equal-weighted `overall` score (#394)

**Daily Practice Workflow**
- `transits` engine added as the 4th participant in the `daily-practice` workflow (#416)
- `DailyPracticeSynthesizer` now references aesthetic and spiritual cycle levels in the synthesis summary when they are at notable values (> 80% or < 20%) (#393)
- Secondary rhythm signal included in synthesis summary text

**Full Spectrum Workflow**
- `partner_birth_date` option forwarded transparently through full-spectrum execution to the biorhythm engine (#394)

### Tests
- 14 new biorhythm validation tests covering:
  - Aesthetic cycle (43-day) at birth, quarter-period, half-period, and full-period (#407)
  - Spiritual cycle (53-day) at birth and quarter-period (#407)
  - Percentage mapping at known sine values (#407)
  - Compatibility cosine formula, same-birthday, full-period, overall mean, and range (#408)
  - `ConsciousnessEngine` contract: `engine_id`, `required_phase=0`, `validate` accepts own output (#410)
- 6 new E2E integration tests in `noesis-integration`:
  - `daily-practice` biorhythm output with all 4 engines (#416)
  - `full-spectrum` biorhythm output fields and compatibility block (#417)

### Benchmarks
- New criterion benchmark groups added to `engine-biorhythm`:
  - `biorhythm_7day_forecast` — full calculate with 7-day forecast (target: < 1 ms) (#409)
  - `biorhythm_compatibility` — two-person compatibility calculation (target: < 500 µs) (#409)

### Documentation
- `docs/portal/docs/engines/biorhythm.md` updated with:
  - `options` table covering `forecast_days` and `partner_birth_date` (#395)
  - Full response example with secondary cycles and compatibility block (#395)
  - Per-field descriptions for primary cycles, secondary cycles, composite scores, and compatibility (#395)
  - Compatibility score formula and interpretation guide (#395)



### Added
- Consciousness level auto-promotion docs with reading-count thresholds and XP accrual notes in [authentication.md](docs/api/authentication.md).
- API surface docs for `GET /api/v1/status` in [docs/api/README.md](docs/api/README.md).

### Fixed
- Corrected workflow docs to match runtime: `birth-blueprint` uses `numerology + human-design + gene-keys`.
- Corrected engine `required_phase` values in [docs/api/engines.md](docs/api/engines.md) to match current engine implementations.
- Updated stale `noesis-bridge` unit tests for TS engine phase values (`i-ching`, `sacred-geometry`, `sigil-forge`).

## [3.0.0] - 2026-02-16

### Added

**noesis-sdk Crate (P0 Foundation)**
- `NoesisClient` — HTTP client for all 16 engines and 6 workflows
- `LocalProfile` — JSON profile persistence at `~/.noesis/profile.json`
- `KeychainStore` — macOS Keychain API key storage via `keyring` crate
- `MarkdownRenderer` — Markdown/JSON/Text report rendering with phase indicators
- `Config` — TOML + environment variable configuration with builder pattern
- 27 unit tests passing

**noesis-tui Crate (P1 Terminal TUI)**
- Full Ratatui 0.29 interactive terminal interface (3,300+ lines, 18 files)
- 7 screens: Welcome, Onboarding (8-step wizard), Engine Picker (16 engines), Workflow Picker (6 workflows), Result Display, History Browser, Profile Editor
- 3 widgets: Help overlay, error bar, spinner
- Type-to-filter with `/` activation mode for engine and workflow pickers
- Styled Markdown rendering with phase dots, consciousness levels, witness prompts
- Export to Markdown (`e`) and JSON (`J`) from result display
- Scroll position indicator and re-run keybind (`r`)
- API key management via keychain in profile editor
- Connection status indicator and version display on welcome screen
- Tracing logs to `~/.noesis/logs/tui.log`

**UX Gap Analysis Fixes (18 issues resolved)**
- C1: Help menu item now toggles help overlay (was navigating to self)
- C2: `q` shortcut now properly quits via `should_quit` flag
- C3: Engine/workflow errors surfaced to user via `ShowError` action
- M1: Profile/client null guards show helpful error messages
- M2: History Enter key opens selected reading in result display
- M3: `/` enters explicit filter mode, j/k safe for navigation
- M4: API key field added to profile editor with keychain integration
- M5: Workflow picker now has filter support matching engine picker
- M6: JSON export footer label corrected to show `J` (Shift+J)
- Plus 8 minor fixes: shared `format_name` utility, error bar positioning, spinner dead-code suppression, version display, connection indicator, scroll position, re-run keybind, edit buffer cleanup

- OpenClaw integration guide and LLM/agent guide for Noesis APIs
- OpenAPI spec refreshed to match current routes, headers, and schemas

### Changed
- API docs aligned with unified `EngineInput` schema and live endpoints
- API keys treated as unique user identities; profile auto-population from `birth_data`
- Philosophy-first messaging realignment across README + docs tiers (user-facing, platform, and operational intros)
- Engine/workflow framing normalized to mirrors/synthesis language where conceptual
- Canonical domain policy standardized (`selemene.tryambakam.space`, `tryambakam.space`, `1319.tryambakam.space`)

### Documentation QA Artifacts
- `docs/DOCS_CONSISTENCY_CHECKLIST_2026-02-16.md`
- `docs/DOCS_LINK_MAP_2026-02-16.md`
- `docs/DOCS_VOCAB_LINT_REPORT_2026-02-16.md`
- `docs/IMAGE_PLACEMENT_STRATEGY_2026-02-16.md`
- `docs/DOCS_EXTERNAL_LINK_AUDIT_2026-02-16.md`
- `docs/DOCS_CONSISTENCY_SIGNOFF_2026-02-16.md`

### Fixed
- Documentation references to legacy endpoints and auth headers

## [2.3.0] - 2026-02-10

### Added
- Universal Agent Bridge — Python generators for Claude, OpenAI, LangChain tool definitions from OpenAPI specs
- `@selemene/bridge` CLI — interactive TypeScript CLI (`npx @selemene/bridge init`) for zero-Python tool generation
- Universal Tool Server (FastAPI/MCP-compatible) scaffold
- Cron job scaffolds (daily witness, hourly panchanga)
- `llms.txt` and `.well-known/agent.json` for AI agent discovery (llms.txt spec + Google A2A protocol)

### Fixed
- Sprint 1 Aleph Launch — 5 P0 bug fixes + assessment tasks

### Changed
- Official domain `selemene.tryambakam.space` patched into all docs and scripts
- MVP task tracking (22 tasks marked done)

---

## [2.1.0] - 2026-02-09

### Added

#### Phase 10: FreeAstrologyAPI Integration (18 tasks completed)

**noesis-vedic-api Crate**
- FreeAstrologyAPI.com integration for high-accuracy Vedic astrology calculations
- `VedicApiService` as the unified entry point with metrics, fallback, and resilience
- `CachedVedicClient` with LRU caching and daily rate limit tracking (50/day)
- Complete Panchang support: Tithi, Nakshatra, Yoga, Karana, Vara, Muhurtas, Hora, Choghadiya
- Vimshottari Dasha at all 4 levels (Maha, Antar, Pratyantar, Sookshma)
- Birth Chart (Rashi D1) and Navamsa Chart (D9)
- Advanced modules: Yogas, Shadbala, Ashtakavarga, Transits, Eclipses, Festivals

**Test Infrastructure (174 tests)**
- Comprehensive test mocks (37 tests) for CI/CD without API keys
- Validation tests (51 tests) against JHora, Swiss Ephemeris, and Shesh's birth profile
- Integration tests (86 tests) covering end-to-end flows, error handling, and resilience

**Fallback and Resilience (FAPI-098, FAPI-105)**
- Automatic native calculation fallback when API is unavailable
- `FallbackCalculator` for approximate Panchang, Dasha, and Birth Chart
- `RateLimitHandler` for HTTP 429 responses with exponential backoff
- Retry-After header support with configurable delay cap (60s max)
- Circuit breaker pattern to prevent cascading failures

**Metrics and Monitoring (FAPI-099)**
- Prometheus-compatible metrics export (11 metric families)
- API call counts, cache hit/miss ratios, response time histograms
- Error counts by type, fallback trigger counts by reason
- JSON summary export for logging and health checks

**Rate Limit Handling (FAPI-105)**
- Client-side daily quota tracking (50 calls/day with 5-request safety buffer)
- Server-side 429 handling with exponential backoff (1s, 2s, 4s... up to 60s)
- Retry-After header respected when present

**Docker and Deployment (FAPI-103, FAPI-104)**
- Docker configuration for Railway deployment (`Dockerfile.prod`, `railway.toml`)
- Health check endpoint support (`/health/live`)
- Environment variable configuration for all API settings

**Batch and Versioning (FAPI-106, FAPI-107)**
- Batch Panchang request support via `batch_panchang()`
- API version routing with deprecation headers (`X-API-Version`, `Sunset-Notice`)

**Documentation**
- Comprehensive README for noesis-vedic-api crate
- Migration guide: Native engines to FreeAstrologyAPI (`docs/MIGRATION_TO_FREE_ASTROLOGY_API.md`)
- Internal migration guide: v1 (VedicApiClient) to v2 (VedicApiService) (`MIGRATION.md`)

### Changed
- `noesis-vedic-api` version bumped to 0.1.0 (initial release of crate)

### Migration
- See [Migration Guide](docs/MIGRATION_TO_FREE_ASTROLOGY_API.md) for migrating from native engines
- See [crates/noesis-vedic-api/MIGRATION.md](crates/noesis-vedic-api/MIGRATION.md) for v1-to-v2 internal migration

---

## [2.0.0] - 2026-02-01

### Added

#### Wave 2 - Consciousness Engines Complete

**New Rust Engines (3)**
- `engine-vedic-clock` - TCM organ clock + Ayurvedic dosha timing
- `engine-biofield` - Chakra energy readings (stub with mock data)
- `engine-face-reading` - Physiognomy analysis (stub with mock data)

**New TypeScript Engines (5)**
- `tarot` - 78-card Rider-Waite-Smith deck, 5 spread types
- `i-ching` - 64 hexagrams with changing lines and nuclear hexagrams
- `enneagram` - 9 types with wings, integration/disintegration paths
- `sacred-geometry` - Geometric form meditation prompts (stub)
- `sigil-forge` - Intent-based sigil generation (stub)

**New Workflows (6)**
- `birth-blueprint` - Natal life analysis (Numerology + HD + Vimshottari)
- `daily-practice` - Daily timing optimization (Panchanga + VedicClock + Biorhythm)
- `decision-support` - Multi-perspective guidance (Tarot + I-Ching + HD)
- `self-inquiry` - Shadow work (Gene Keys + Enneagram)
- `creative-expression` - Generative guidance (Sigil Forge + Sacred Geometry)
- `full-spectrum` - Complete portrait (all 14 engines)

**Infrastructure**
- `noesis-orchestrator` - Parallel workflow execution with theme detection
- `noesis-bridge` - HTTP bridge for Rust↔TypeScript communication
- Docker production image (`Dockerfile.prod`)
- Kubernetes manifests (`k8s/`)
- GitHub Actions CI/CD (`test.yml`, `deploy.yaml`)
- Prometheus alerts and Grafana dashboards
- E2E, load (k6), chaos, and security test suites

### Changed

**Human Design Engine**
- Fixed gate sequence to use Rave I-Ching Mandala (was sequential 1→64)
- Fixed design time calculation to use 88° solar arc (was 88-day offset)
- Personality Sun/Earth gates now accurate

**API**
- Added `/api/v1/engines/{engine_id}/calculate` unified endpoint
- Added `/api/v1/workflows/{workflow_id}/execute` workflow endpoint
- Response structure includes `engine_id`, `result`, `witness_prompts`

### Fixed
- Gate sequence mapping for Human Design
- Design time solar arc calculation
- Swiss Ephemeris data path auto-discovery

### Known Issues
- HD Design Sun ~6° off expected (calibration needed)
- HD Profile lines need adjustment
- Biofield and Face Reading are stub implementations

---

## [1.1.0] - 2026-01-31

### Added

#### Wave 1 Complete

**Core Engines (6)**
- `engine-panchanga` - Vedic calendar (tithi, nakshatra, yoga, karana, vara)
- `engine-numerology` - Pythagorean + Chaldean name/date reduction
- `engine-biorhythm` - Physical (23d), Emotional (28d), Intellectual (33d) cycles
- `engine-human-design` - 26 planetary activations, type/authority/profile
- `engine-gene-keys` - Shadow-Gift-Siddhi activation sequences
- `engine-vimshottari` - 120-year dasha period calculations

**Infrastructure**
- 3-layer cache (L1 memory, L2 Redis, L3 disk)
- JWT + API key authentication
- Rate limiting by tier
- Prometheus metrics

### Performance
- HD calculation: 1.31ms (76x faster than target)
- Gene Keys: 0.012ms (4166x faster than target)
- Vimshottari: <1ms (200x faster than target)

---

## [1.0.0] - 2025-08-13

### Added
- Initial Selemene Engine release
- Panchanga calculation API
- Swiss Ephemeris integration
- Basic caching

---

## Version History Summary

| Version | Date | Engines | Tests | Highlights |
|---------|------|---------|-------|------------|
| 3.0.0 | 2026-02-16 | 16 | 429+ | SDK + TUI, OpenClaw, UX gap analysis (18 fixes) |
| 2.3.0 | 2026-02-10 | 14 + API | 402+ | Agent Bridge CLI, AI discovery files, custom domain |
| 2.1.0 | 2026-02-09 | 14 + API | 402+ | FreeAstrologyAPI integration, metrics, resilience |
| 2.0.0 | 2026-02-01 | 14 | 228+ | Wave 2 complete, workflows, production ready |
| 1.1.0 | 2026-01-31 | 6 | 100+ | Wave 1 complete, all core engines |
| 1.0.0 | 2025-08-13 | 1 | 20+ | Initial release, Panchanga only |
