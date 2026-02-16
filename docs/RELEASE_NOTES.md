# Selemene Engine v3.0.0 Release Notes

**Release Date**: 2026-02-16  
**Codename**: Wave 3 - OpenClaw Integration

---

## Overview

Selemene Engine v3.0.0 focuses on product readiness: unified API documentation, OpenClaw integration, and a clean agent-first onboarding path. This release makes the platform easier to integrate into agent workflows while keeping the core engine and workflow APIs stable.

---

## What's New

### Agent-Ready Documentation
- ✅ OpenAPI spec refreshed to match live routes, headers, and schemas
- ✅ Unified API docs aligned to the shared `EngineInput` format
- ✅ OpenClaw integration guide for agent workflows

### Identity & Profile Auto-Population
- ✅ Each API key is a unique user identity
- ✅ If `birth_data` is provided, the user profile is auto-populated

### History & Reporting
- ✅ Readings history endpoints documented and supported:
   - `GET /api/v1/readings`
   - `GET /api/v1/readings/{reading_id}`
   - `GET /api/v1/readings/stats`

## Roadmap (Phases)

**Phase 1 — Onboarding + TUI**
- ✅ Terminal TUI for non-frontend onboarding and first reading
- ✅ Profile capture + reuse (birth_data saved locally)
- ✅ Report export (Markdown) for Somatic Canticles workflows

**Phase 2 — Desktop Surfaces**
- ✅ Raycast extension for quick readings (numerology, panchanga, daily-practice)
- ✅ macOS menu bar applet (daily timing + quick prompts)
- ✅ Tauri wrapper app for non-technical users

**Phase 3 — Apple Ecosystem**
- ✅ Apple Shortcuts actions (run engine/workflow via HTTP)
- ✅ Apple Watch faces/complications (daily-practice, biorhythm snapshot)
- ✅ On-device caching for low-connectivity reads

**Integration Notes & Delivery Constraints**
- All surfaces call the same API endpoints and share `EngineInput`.
- API keys must be stored securely (Keychain/Shortcuts secure storage).
- Watch faces are limited to cached summaries and small payloads.
- Menu bar + Raycast must avoid long-running calls; use workflows where possible.

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
