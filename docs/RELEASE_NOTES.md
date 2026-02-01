# Selemene Engine v2.0.0 Release Notes

**Release Date**: 2026-01  
**Codename**: Wave 2 - Noesis Complete

---

## Overview

Selemene Engine v2.0.0 represents a major evolution from a Vedic astrology calculation engine to a comprehensive consciousness calculation platform. This release introduces 14 integrated engines, 6 predefined workflows, and a synthesis system for cross-engine insights.

---

## What's New

### 14 Consciousness Engines

#### Rust Engines (9)

| Engine | Description | Status |
|--------|-------------|--------|
| **Human Design** | Complete bodygraph calculation with 26 activations, type/authority/profile determination | Full |
| **Gene Keys** | Shadow-Gift-Siddhi mapping with activation sequences and transformation pathways | Full |
| **Vimshottari** | Vedic dasha system with Mahadasha/Antardasha/Pratyantardasha periods | Full |
| **Panchanga** | Five-limb Vedic calendar (Tithi, Nakshatra, Yoga, Karana, Vara) | Full |
| **Numerology** | Western numerology with Life Path, Expression, Soul Urge, Personal Year | Full |
| **Biorhythm** | Physical/Emotional/Intellectual cycle tracking with forecasting | Full |
| **Vedic Clock** | Dosha periods, Muhurtas, Ghati time calculations | Full |
| **Biofield** | Chakra and aura energy assessment | Stub |
| **Face Reading** | Chinese physiognomy feature analysis | Stub |

#### TypeScript Engines (5)

| Engine | Description | Status |
|--------|-------------|--------|
| **Tarot** | 78-card Rider-Waite-Smith with multiple spread types | Full |
| **I-Ching** | 64 hexagrams with changing lines and transformations | Full |
| **Enneagram** | 9-type personality system with wings and instinctual variants | Full |
| **Sacred Geometry** | Generative patterns (Flower of Life, Metatron's Cube, etc.) | Full |
| **Sigil Forge** | Intention-to-sigil encoding via Rose Cross and chaos magic methods | Full |

### 6 Predefined Workflows

| Workflow | Engines | Purpose |
|----------|---------|---------|
| **Birth Blueprint** | Numerology, Human Design, Gene Keys | Core identity mapping |
| **Daily Practice** | Panchanga, Vedic Clock, Biorhythm | Daily rhythm guidance |
| **Decision Support** | Tarot, I-Ching, Human Design | Multi-perspective decisions |
| **Self-Inquiry** | Gene Keys, Enneagram | Shadow work and growth |
| **Creative Expression** | Sigil Forge, Sacred Geometry | Visual/symbolic creation |
| **Full Spectrum** | All 14 engines | Comprehensive portrait |

### Synthesis System

- **Theme Detection**: Identifies patterns appearing across 3+ engines
- **Alignment Analysis**: Finds reinforcing patterns between engines
- **Tension Analysis**: Identifies productive contradictions
- **Unified Witness Prompts**: Synthesized contemplation questions

### Architecture Improvements

- **Workflow Orchestrator**: Parallel engine execution via `futures::join_all`
- **TypeScript Bridge**: HTTP bridge to Node/Bun runtime engines
- **Multi-tier Caching**: L1 (memory), L2 (Redis), L3 (precomputed)
- **Phase Gating**: Consciousness-level-based engine access control

---

## Breaking Changes

### API Changes

1. **Base Path Changed**
   - Before: `/api/panchanga/calculate`
   - After: `/api/v1/panchanga/calculate`

2. **Engine Endpoint Pattern**
   - Before: Engine-specific endpoints
   - After: `/api/v1/engines/{engine_id}/calculate`

3. **Workflow Endpoints Added**
   - New: `/api/v1/workflows/{workflow_id}/execute`

4. **Authentication Required**
   - All endpoints now require JWT or API key
   - Phase-based access control enforced

### Response Format Changes

1. **Engine Output Structure**
   ```json
   // v1.x
   {
     "tithi": {...},
     "nakshatra": {...}
   }
   
   // v2.0
   {
     "engine_id": "panchanga",
     "result": {
       "tithi": {...},
       "nakshatra": {...}
     },
     "witness_prompt": "...",
     "consciousness_level": 0,
     "metadata": {...}
   }
   ```

2. **Error Response Structure**
   ```json
   // v2.0
   {
     "success": false,
     "error": {
       "code": "ERROR_CODE",
       "message": "Human readable message",
       "details": {...}
     }
   }
   ```

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
curl http://localhost:8080/api/v1/panchanga/calculate \
  -H "Authorization: Bearer $TOKEN"
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
# Get token
curl -X POST http://localhost:8080/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "your-api-key"}'
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
| Full Spectrum (14 engines) | - | 50ms |
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
