# Release v2.1.0 - Integration Layer & Vedic Clock Enhancements

**Release Date**: February 3, 2026  
**Codename**: "Vedic Bridge"

## 🎯 Overview

Version 2.1.0 represents a significant architectural evolution, introducing a comprehensive integration layer that bridges native Rust engines with external Vedic astrology APIs. This release enhances the Vedic Clock engine with four new time-system integrations and establishes a robust foundation for external API composition.

## ✨ New Features

### 🌉 Integration Layer (`noesis-integration`)
- **External API Bridge**: New `noesis-integration` crate for composing external astrology APIs
- **Vedic API Client**: `noesis-vedic-api` crate with typed client for FreeAstrologyAPI.com
- **Smart Caching**: Intelligent caching of external API responses
- **Fallback Strategy**: Graceful degradation when external APIs are unavailable
- **Rate Limiting**: Built-in rate limiting for external API calls

### 🕐 Vedic Clock Engine Enhancements
Four new time-system integrations:
1. **Panchang Integration** (`panchang_integration.rs`)
   - Tithi timing calculations
   - Nakshatra timing calculations
   - Yoga timing calculations
   - Karana timing calculations
   - Cross-references with FreeAstrologyAPI for validation

2. **Hora Integration** (`hora_integration.rs`)
   - Planetary hour calculations
   - Hora lord determination
   - Favorable/unfavorable time identification
   - Day/night hora sequences

3. **Choghadiya Integration** (`choghadiya_integration.rs`)
   - 8 Choghadiya periods calculation
   - Quality determination (Amrit, Shubh, Labh, Chal, Udveg, Rog, Kaal)
   - Auspicious timing recommendations

4. **TCM Organ Clock** (`organ_clock.rs`)
   - 12 traditional Chinese medicine organ times
   - Peak energy period calculations
   - Health optimization timing
   - Cross-cultural time synthesis

### 📊 JSON Task Management
- **Structured Planning**: Task plans now stored as JSON in `.claude/task-management/`
- **Wave Organization**: Separate JSON files for each development wave
- **Validation Testing**: Test plans with explicit validation criteria
- **Traceability**: Clear mapping from specs to implementation to tests

## 🐛 Bug Fixes
- Fixed environment variable handling in `.env.example`
- Corrected Cargo.lock dependency resolutions
- Improved error handling in Vedic Clock calculations

## ⚡ Performance Improvements
- Optimized external API caching strategy
- Reduced redundant calculations in time-system integrations
- Improved memory usage in multi-engine orchestration

## 📚 Documentation

### New Documentation
- `API_INTEGRATION_STATUS.md` - Integration layer status and capabilities
- `FREE_ASTROLOGY_API_INTEGRATION_SUMMARY.md` - External API usage guide
- `INTEGRATION_LAYER_SUMMARY.md` - Architecture and design decisions
- `WAVE3_COMPLETION_REPORT.md` - Development phase summary

### Updated Documentation
- `.env.example` - Added integration layer configuration
- `memory.md` - Updated with integration layer patterns
- `todo.md` - Reflected completed integration work

## 🔧 Infrastructure

### New Crates
- `noesis-integration` - Integration layer framework
- `noesis-vedic-api` - Typed Vedic API client

### Enhanced Crates
- `engine-vedic-clock` - Added 4 new integration modules

### Configuration
- New environment variables:
  - `VEDIC_API_KEY` - FreeAstrologyAPI.com authentication
  - `VEDIC_API_BASE_URL` - External API endpoint
  - `ENABLE_EXTERNAL_APIS` - Toggle external integrations

## 📦 Dependencies

### New Dependencies
- `reqwest` - HTTP client for external APIs
- Additional async runtime support for API calls

### Updated Dependencies
- See `Cargo.lock` for full dependency tree

## 🧪 Testing

### New Test Coverage
- Integration layer unit tests
- Vedic API client tests (mocked)
- Time-system integration validation
- JSON task plan validation tests

### Test Organization
- `tests/integration/` - Integration layer tests
- `tests/validation/` - Cross-system validation

## 📈 Metrics

- **Engine Count**: 9 operational engines (HD, Gene Keys, Vimshottari + 6 others)
- **Integration Points**: 4 new Vedic Clock integrations
- **API Coverage**: FreeAstrologyAPI.com integration (Panchang, Hora, Choghadiya)
- **Crate Count**: 14+ workspace crates
- **Test Coverage**: Enhanced with integration validation

## ⚠️ Breaking Changes

**None** - This release is fully backward compatible with v2.0.0

## 🎨 Architecture Changes

### Integration Pattern
```
┌─────────────────────────────────────────┐
│      Noesis Orchestrator                │
│  (Multi-Engine Coordination)            │
└──────────────┬──────────────────────────┘
               │
    ┌──────────┴────────────┐
    │                       │
┌───▼────────────┐  ┌──────▼────────────────┐
│ Native Engines │  │  Integration Layer    │
│   (Rust)       │  │  (noesis-integration) │
│                │  │                       │
│ • HD           │  │  ┌─────────────────┐  │
│ • Gene Keys    │  │  │ Vedic API Client│  │
│ • Vimshottari  │  │  │ (typed)         │  │
│ • Vedic Clock  │  │  └─────────────────┘  │
│   └─Uses───────┼──┼──► FreeAstrologyAPI   │
└────────────────┘  │                       │
                    │  • Caching            │
                    │  • Rate limiting      │
                    │  • Fallback           │
                    └───────────────────────┘
```

## 🙏 Contributors

- **Lead Engineer**: Implementation and architecture
- **Systems Integration**: External API bridge design

## 📥 Installation

### Docker
```bash
# Pull latest image
docker pull ghcr.io/sheshiyer/selemene-engine:v2.1.0

# Run with environment
docker run -p 8080:8080 \
  -e VEDIC_API_KEY=your_key \
  ghcr.io/sheshiyer/selemene-engine:v2.1.0
```

### From Source
```bash
git clone https://github.com/Sheshiyer/Selemene-engine.git
cd Selemene-engine
git checkout v2.1.0

# Configure environment
cp .env.example .env
# Edit .env with your API keys

# Build and run
cargo build --release
./target/release/selemene-engine
```

### Environment Setup
```bash
# Required for external API integration
export VEDIC_API_KEY="your_freeastrologyapi_key"
export VEDIC_API_BASE_URL="https://json.freeastrologyapi.com"
export ENABLE_EXTERNAL_APIS="true"

# Optional: Redis for caching
export REDIS_URL="redis://localhost:6379"
```

## 🔗 Links

- [Full Changelog](CHANGELOG.md)
- [API Documentation](docs/api-docs.md)
- [Integration Guide](INTEGRATION_LAYER_SUMMARY.md)
- [Wave 3 Report](WAVE3_COMPLETION_REPORT.md)

## ⏭️ What's Next (v2.2.0)

### Planned Features
- **Numerology Engine**: Complete implementation with life path, expression, soul urge
- **Biorhythm Engine**: Physical, emotional, intellectual cycles
- **Biofield Engine**: Energy field analysis (design phase)
- **Advanced Synthesis**: Multi-engine correlation analysis
- **Performance Optimization**: Sub-10ms response times for hot paths

### Future Enhancements
- GraphQL API layer
- WebSocket real-time updates
- Mobile SDK (iOS/Android)
- Web dashboard UI

---

**Full Changelog**: [v2.0.0...v2.1.0](https://github.com/Sheshiyer/Selemene-engine/compare/v2.0.0...v2.1.0)
