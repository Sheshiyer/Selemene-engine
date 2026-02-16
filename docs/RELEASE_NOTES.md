# Selemene Engine v3.0.0 Release Notes

**Release Date**: 2026-02-16  
**Codename**: Wave 3 — SDK, TUI & OpenClaw

---

## Overview

Selemene Engine v3.0.0 delivers two major new crates — the **Rust SDK** (`noesis-sdk`) and an interactive **Terminal TUI** (`noesis-tui`) — alongside OpenClaw integration and agent-ready documentation. Users can now run all 16 engines and 6 workflows from a polished terminal interface without writing any code.

This release expands access to reflective computation — mirrors for inquiry, not prescriptive outputs.

It also includes a full philosophy-first documentation realignment: README narrative rewrite, docs-wide messaging normalization, canonical link policy enforcement, vocabulary linting, and final QA sign-off artifacts.

---

## What's New

### Philosophy-First Documentation Realignment (36-task closeout)
- `README.md` reframed around authorship, reflection, inquiry, witness
- Kha-Ba-La model and self-consciousness calibration integrated in core narrative
- Engines/workflows language normalized from pipeline framing to mirror/synthesis framing
- Tiered docs consistency pass across user-facing, platform, and ops docs (light-touch for technical runbooks)
- Canonical domain policy enforced across docs:
   - `https://selemene.tryambakam.space` (API runtime)
   - `https://tryambakam.space` (ecosystem context)
   - `https://1319.tryambakam.space` (Somatic Canticles)
- QA artifacts published:
   - `docs/DOCS_CONSISTENCY_CHECKLIST_2026-02-16.md`
   - `docs/DOCS_LINK_MAP_2026-02-16.md`
   - `docs/DOCS_VOCAB_LINT_REPORT_2026-02-16.md`
   - `docs/IMAGE_PLACEMENT_STRATEGY_2026-02-16.md`
   - `docs/DOCS_EXTERNAL_LINK_AUDIT_2026-02-16.md`
   - `docs/DOCS_CONSISTENCY_SIGNOFF_2026-02-16.md`

### Rust SDK (`noesis-sdk`)
- `NoesisClient` — HTTP client for all 16 engines and 6 workflows
- `LocalProfile` — JSON profile persistence (`~/.noesis/profile.json`)
- `KeychainStore` — macOS Keychain API key storage
- `MarkdownRenderer` — Report rendering (Markdown/JSON/Text) with phase indicators
- `Config` — TOML + environment variable configuration with builder pattern
- 27 unit tests

### Terminal TUI (`noesis-tui`)
- Full Ratatui 0.29 interactive interface — 3,300+ lines across 18 files
- **Welcome screen** — ASCII header, version display, connection status (● Connected / ● Offline), 5-item menu with vim keys
- **Onboarding wizard** — 8-step profile setup (name, birth date/time, coordinates, timezone, API key) with validation and progress gauge
- **Engine picker** — 16 engines with `/` filter mode, category tags, loading state
- **Workflow picker** — 6 workflows with filter, engine detail panel
- **Result display** — Styled Markdown rendering, scroll position indicator (Line X/Y), export MD (`e`) / JSON (`J`), re-run (`r`)
- **History browser** — Browse past readings, Enter to view, refresh with `r`
- **Profile editor** — Edit 6 birth data fields + API key (keychain-backed), inline validation
- **Help overlay** — `?` toggle, complete keyboard shortcut reference
- **Error bar** — Transient error messages, properly positioned above footer

### UX Gap Analysis (18 Fixes)
All critical and major UX issues resolved via 5-agent parallel dispatch:
- Help menu, quit shortcut, error propagation, null guards, history navigation, filter mode, API key editing, workflow filter, export labels, scroll indicator, re-run keybind, connection indicator, version display, shared utilities, error positioning, dead code cleanup

### Agent-Ready Documentation
- OpenAPI spec refreshed to match live routes, headers, and schemas
- Unified API docs aligned to the shared `EngineInput` format
- OpenClaw integration guide for agent workflows

### Identity & Profile Auto-Population
- Each API key is a unique user identity
- If `birth_data` is provided, the user profile is auto-populated

### History & Reporting
- Readings history endpoints:
   - `GET /api/v1/readings`
   - `GET /api/v1/readings/{reading_id}`
   - `GET /api/v1/readings/stats`

## Roadmap

**P0 — SDK Foundation** ✅ Complete

**P1 — Terminal TUI** ✅ Complete

**P2 — Desktop Surfaces** *(planned)*
- Raycast extension for quick readings
- macOS menu bar applet
- Tauri wrapper app

**P3 — Apple Ecosystem** *(planned)*
- Apple Shortcuts actions
- Apple Watch complications
- On-device caching

---

## Breaking Changes

### API Changes

No breaking API changes in v3.0.0. This release is documentation and integration focused.

### Response Format Changes

No response format changes in v3.0.0.

## Release Checklist

- [ ] Confirm `openapi.yaml` matches current routes and schemas
- [ ] Verify `GET /health/live` and `GET /api/v1/engines`
- [ ] Run a sample calculation (numerology or panchanga)
- [ ] Validate OpenClaw integration doc accuracy
- [ ] Publish docs and announce release

### Configuration Changes

1. **New Required Environment Variables**
   - `JWT_SECRET` (required)
   - `TS_ENGINES_URL` (if using TS engines)

2. **Renamed Variables**
   - `EPHEMERIS_PATH` → `SWISS_EPHEMERIS_PATH`
   - `CACHE_SIZE` → `CACHE_L1_SIZE`

---

## Migration Guide

### From v1.x to v2.0.0

#### Step 1: Update API Calls

```bash
# Old
curl http://localhost:8080/api/panchanga/calculate

# New
curl http://localhost:8080/api/v1/engines/panchanga/calculate \
   -H "X-API-Key: $NOESIS_API_KEY"
```

#### Step 2: Update Response Handling

```javascript
// Old
const tithi = response.tithi;

// New
const tithi = response.result.tithi;
const witnessPrompt = response.witness_prompt;
```

#### Step 3: Set Up Authentication

```bash
# Use API key directly
export NOESIS_API_KEY="nk_your_key_here"
```

#### Step 4: Update Environment

```bash
# Add new required variables
export JWT_SECRET="your-secure-secret"
export TS_ENGINES_URL="http://localhost:3001"  # If using TS engines

# Rename existing variables
export SWISS_EPHEMERIS_PATH="$EPHEMERIS_PATH"
export CACHE_L1_SIZE="$CACHE_SIZE"
```

#### Step 5: Start TypeScript Engines (Optional)

```bash
cd ts-engines
bun install
bun run start
```

#### Step 6: Database Migration

```bash
# Run migrations (if using PostgreSQL features)
cargo sqlx migrate run
```

### SDK/Client Updates

If using SDK or client library:
1. Update to latest version
2. Initialize with authentication credentials
3. Update response parsing for new structure
4. Handle new error response format

---

## Performance Improvements

| Metric | v1.x | v2.0 |
|--------|------|------|
| Single Panchanga | 45ms | 15ms |
| Human Design (full) | - | 1.3ms |
| Gene Keys | - | 0.01ms |
| Full Spectrum (16 engines) | - | 50ms |
| Cache hit rate | 70% | 85%+ |

---

## Known Limitations

1. **Biofield and Face Reading** are stub implementations
2. **Human Design Variable/Arrows** not yet implemented
3. **Gene Keys Venus/Pearl Sequences** not yet implemented
4. **Real-time tracking** endpoints disabled pending completion
5. **Historical dates before -3000** may have reduced accuracy

---

## Deprecations

The following will be removed in v3.0:

1. `GET /status` endpoint (use `/health` and `/ready`)
2. Legacy response format support
3. Unauthenticated access to any endpoint

---

## Security Notes

1. JWT tokens expire after 1 hour by default
2. API keys should be kept secret and rotated periodically
3. Rate limiting is enforced by tier
4. TLS required for production deployments

---

## Upgrade Checklist

- [ ] Review breaking changes above
- [ ] Update environment variables
- [ ] Obtain authentication credentials
- [ ] Update API calls to v1 path
- [ ] Update response parsing
- [ ] Test thoroughly in staging
- [ ] Update monitoring dashboards
- [ ] Update client SDKs if applicable
- [ ] Deploy TypeScript engines if using TS-based engines

---

## Contributors

Wave 2 development involved multiple AI agents and human oversight:
- Architecture and core infrastructure
- Individual engine implementations
- Workflow orchestration
- TypeScript engine bridge
- Documentation and testing

---

## What's Next (v2.1 Preview)

- Complete Biofield and Face Reading implementations
- Human Design Variable analysis
- Gene Keys Venus and Pearl sequences
- Custom workflow creation API
- GraphQL API option
- Mobile SDK

---

**Full Changelog**: See CHANGELOG.md
**Documentation**: See /docs/
**Issues**: Report via GitHub Issues

---

*Selemene Engine v2.0.0 - Witness Your Consciousness*
