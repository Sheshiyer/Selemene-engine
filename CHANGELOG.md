# Changelog

All notable changes to the Tryambakam Noesis Engine project.

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
| 2.0.0 | 2026-02-01 | 14 | 228+ | Wave 2 complete, workflows, production ready |
| 1.1.0 | 2026-01-31 | 6 | 100+ | Wave 1 complete, all core engines |
| 1.0.0 | 2025-08-13 | 1 | 20+ | Initial release, Panchanga only |
