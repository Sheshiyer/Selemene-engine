# Selemene Engine → Tryambakam Noesis

A multi-engine consciousness platform integrating 14 wisdom systems for self-inquiry and consciousness evolution.

## Overview

Tryambakam Noesis is the evolution of the Selemene Vedic astrology engine into a comprehensive consciousness platform. It combines ancient wisdom traditions (Vedic astrology, I Ching, Tarot, etc.) with modern systems (Human Design, Gene Keys, Enneagram) to generate personalized self-inquiry prompts based on consciousness level (0-5).

## Architecture

The platform uses a **multi-engine architecture** with:

- **9 Rust Engines**: High-performance engines for computational wisdom systems
- **5 TypeScript Engines**: Engines for generative and symbolic systems
- **Unified API**: Single gateway for all engine calculations
- **Multi-Layer Caching**: L1 (memory), L2 (Redis), L3 (disk)
- **Consciousness-Based Prompts**: Adaptive self-inquiry questions based on user level

### Core Components

```
crates/
├── noesis-core/           # Shared traits and types
├── noesis-cache/          # Multi-layer caching system
├── noesis-auth/           # Authentication
├── noesis-metrics/        # Monitoring
├── noesis-witness/        # Prompt generation
├── noesis-orchestrator/   # Multi-engine workflow coordinator
├── noesis-bridge/         # TypeScript ↔ Rust bridge
├── noesis-api/            # Unified API gateway
│
└── Engines (9 Rust):
    ├── engine-panchanga/       # Vedic calendar
    ├── engine-numerology/      # Number patterns
    ├── engine-biorhythm/       # Life cycles
    ├── engine-human-design/    # 64 gates, 9 centers
    ├── engine-vimshottari/     # Planetary periods
    ├── engine-gene-keys/       # Shadow-Gift-Siddhi
    ├── engine-vedic-clock/     # Time cycles
    ├── engine-biofield/        # Energy field
    └── engine-face-reading/    # Facial analysis

ts-engines/ (5 TypeScript):
├── enneagram/
├── i-ching/
├── sacred-geometry/
├── sigil-forge/
└── tarot/
```

## Wisdom Data Layer

The platform includes a comprehensive wisdom data layer with **36 JSON files** covering 13 wisdom systems:

- **Human Design**: 12 files - 64 gates, 9 centers, 36 channels, 5 types
- **Gene Keys**: 1 file - 64 keys with Shadow/Gift/Siddhi spectrum
- **Vimshottari**: 4 files - Vedic planetary periods (120-year cycle)
- **I Ching**: 2 files - 64 hexagrams with line interpretations
- **Tarot**: 2 files - 78-card deck (22 Major + 56 Minor Arcana)
- **Enneagram**: 1 file - 9 personality types
- **Sacred Geometry**: 2 files - Universal patterns and constructions
- **Biofield**: 2 files - 7-layer aura algorithms
- **Biorhythm**: 1 file - Life cycle age points
- **Vedic Clock**: 5 files - Panchanga + TCM organ clock
- **Face Reading**: 3 files - 468-point mesh + diagnostics
- **Cross-System**: 1 file - Vedic-TCM correspondences

**Documentation**: 4 comprehensive MD files (3,707 lines) covering schemas, integration patterns, and code examples.

📖 **[Read the Data Layer Documentation →](./data/README.md)**

## Consciousness Levels

The system adapts prompts based on user consciousness level:

| Level | Name | Description | Prompt Type |
|-------|------|-------------|-------------|
| **0** | Dormant | Unaware of system | Observational |
| **1** | Glimpsing | First exposure | Reflective |
| **2** | Practicing | Active engagement | Inquiry |
| **3** | Integrated | Living the wisdom | Authorship |
| **4-5** | Embodied | Full integration | Open |

## Tech Stack

### Rust Backend
- **Language**: Rust 2021 edition
- **Web Framework**: Axum (async HTTP)
- **Serialization**: Serde (JSON)
- **Async Runtime**: Tokio
- **Planned DB**: PostgreSQL with SQLx
- **Planned Cache**: Redis

### TypeScript Engines
- 5 engines for symbolic and generative systems
- Connected to Rust via `noesis-bridge`
- JSON-based data exchange

## Getting Started

### Prerequisites
- Rust 1.70+ with Cargo
- Node.js 18+ with npm/yarn
- (Optional) PostgreSQL for persistence
- (Optional) Redis for caching

### Build

```bash
# Build all Rust crates
cargo build --release

# Install TypeScript dependencies
npm install

# Build TypeScript engines
npm run build
```

### Run Tests

```bash
# Test all Rust crates
cargo test

# Test specific engine
cd crates/engine-human-design
cargo test

# Test TypeScript engines
npm test
```

## Project Status

**Current State**: Architecture designed but mostly unimplemented (~80% architecture, ~20% implementation)

**Completed**:
- ✅ Core trait definitions (`ConsciousnessEngine`)
- ✅ Workspace crate structure (17 crates)
- ✅ Wisdom data layer (36 JSON files)
- ✅ Gate sequence algorithms (Human Design)
- ✅ Comprehensive documentation

**In Progress**:
- 🔄 Engine implementations
- 🔄 Data loading modules
- 🔄 API gateway
- 🔄 Caching layer
- 🔄 Witness prompt generation

**Planned**:
- 📋 PostgreSQL integration
- 📋 Redis caching
- 📋 User authentication
- 📋 Multi-engine orchestration
- 📋 Production deployment

## Documentation

- **[Data Layer](./data/README.md)** - Wisdom data organization and usage
- **[Architecture](./selemene_architecture.md)** - Original Selemene design (superseded)
- **[Codebase Summary](./CODEBASE_SUMMARY.md)** - Current implementation status
- **[Project Summary](./PROJECT_SUMMARY.md)** - Project goals and vision
- **[Memory](./memory.md)** - Project history and completed tasks

## Contributing

This is a personal consciousness platform under active development. While not currently open for external contributions, the architecture and wisdom data layer may serve as reference for similar systems.

## License

Proprietary - All Rights Reserved

---

*From Selemene (Vedic astrology) to Tryambakam Noesis (multi-engine consciousness) - bridging ancient wisdom with modern computation.*
