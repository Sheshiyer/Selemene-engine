# Wisdom Data Layer

This directory contains the wisdom data for all consciousness engines in the Tryambakam Noesis platform. The data represents millennia of spiritual, archetypal, and diagnostic systems, meticulously structured for computational access while preserving depth and nuance.

## Overview

- **36 JSON data files** covering 13 wisdom systems
- **4 comprehensive documentation files** (3,707 lines)
- **13,723 total lines** of wisdom data
- **Full Unicode support** for multilingual content (Sanskrit देवनागरी, Chinese 中文, Hebrew א)

## Directory Structure

```
data/
├── human-design/          # 12 JSON files - 64 gates, 9 centers, 36 channels
├── gene-keys/             # 1 JSON file - 64 keys with Shadow/Gift/Siddhi
├── vimshottari/           # 4 JSON files - Vedic astrology periods
├── i-ching/               # 2 JSON files - 64 hexagrams
├── tarot/                 # 2 JSON files - 78-card deck
├── enneagram/             # 1 JSON file - 9 personality types
├── sacred-geometry/       # 2 JSON files - Sacred symbols and templates
├── biofield/              # 2 JSON files - 7-layer aura algorithms
├── biorhythm/             # 1 JSON file - Life cycle age points
├── vedic-clock/           # 5 JSON files - Time cycles and TCM integration
├── face-reading/          # 3 JSON files - 468-point mesh + diagnostics
├── cross-system/          # 1 JSON file - Vedic-TCM correspondences
└── wisdom-docs/           # 4 MD files - Complete schemas and mappings
```

## Wisdom Systems

### 64-Based Archetypal Systems
- **Human Design** (12 files) - Modern synthesis of I Ching, Kabbalah, Chakras, Astrology
- **Gene Keys** (1 file) - Consciousness evolution framework
- **I Ching** (2 files) - Ancient Chinese divination system

### Personality & Psychology
- **Enneagram** (1 file) - 9 personality types with wings and integration paths
- **Tarot** (2 files) - 78 archetypal cards (22 Major + 56 Minor Arcana)

### Time & Cycles
- **Vimshottari** (4 files) - Vedic planetary periods (120-year cycle)
- **Vedic Clock** (5 files) - Panchanga time qualities + TCM organ clock
- **Biorhythm** (1 file) - Physical, emotional, intellectual cycles

### Body & Energy
- **Face Reading** (3 files) - Vedic + TCM facial diagnostics (468 landmarks)
- **Biofield** (2 files) - 7-layer aura spatial/temporal algorithms
- **Sacred Geometry** (2 files) - Universal patterns and constructions

### Cross-System Integration
- **Cross-System** (1 file) - Vedic-TCM correspondences linking systems

## Documentation

For detailed information about data schemas, integration patterns, and code examples:

- **[WISDOM-DATA-INDEX.md](./wisdom-docs/WISDOM-DATA-INDEX.md)** - Complete catalog of all wisdom systems
- **[DATA-SCHEMAS.md](./wisdom-docs/DATA-SCHEMAS.md)** - Detailed schema reference for all JSON files
- **[QUICK-REFERENCE-GUIDE.md](./wisdom-docs/QUICK-REFERENCE-GUIDE.md)** - Code examples for loading and using data
- **[ENGINE-DATA-MAPPINGS.md](./wisdom-docs/ENGINE-DATA-MAPPINGS.md)** - Engine integration patterns

## Loading Data in Engines

### Rust Engines

Data files are loaded using `serde_json` for parsing:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct GateData {
    pub number: u8,
    pub name: String,
    pub keynote: String,
    pub center: String,
    // ... other fields
}

pub fn load_gates() -> Result<HashMap<u8, GateData>, Box<dyn std::error::Error>> {
    let data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/human-design/gates.json");
    let json_str = std::fs::read_to_string(data_path)?;
    let gates: HashMap<u8, GateData> = serde_json::from_str(&json_str)?;
    Ok(gates)
}
```

### TypeScript Engines

Data files are loaded using Node.js file system:

```typescript
import { readFileSync } from 'fs';
import { join } from 'path';

export interface Hexagram {
  number: number;
  name: string;
  chineseName: string;
  binary: string;
  // ... other fields
}

export function loadHexagrams(): Map<number, Hexagram> {
  const dataPath = join(__dirname, '../../../data/i-ching/hexagrams.json');
  const json = JSON.parse(readFileSync(dataPath, 'utf-8'));
  return new Map(Object.entries(json.hexagrams).map(([k, v]) =>
    [parseInt(k), v as Hexagram]
  ));
}
```

## Data Characteristics

### Archetypal Number Systems
- **64-based**: I Ching, Human Design, Gene Keys (hexagram foundation)
- **12-based**: Zodiac houses, TCM time divisions
- **9-based**: Enneagram personality types
- **7-based**: Biofield layers, chakras
- **6-based**: Lines, colors, tones (Human Design subdivisions)

### Cross-References
Many systems share common numerical foundations:
- Human Design Gate 1 ↔ I Ching Hexagram 1 ↔ Gene Keys Key 1
- Vedic planets ↔ TCM organs (see `cross-system/vedic_tcm_correspondences.json`)
- Face reading zones ↔ Vedic houses + TCM organs

### Metadata Standards
All systems include:
- `system_info` object with name, description, source
- Total count fields (`total_gates`, `total_hexagrams`, etc.)
- UTF-8 encoding for multilingual content

## Integration Architecture

The wisdom data integrates at multiple layers:

```
ARCHETYPAL LAYER (64-System)
├── I Ching Hexagrams (ancient foundation)
├── Human Design Gates (modern synthesis)
└── Gene Keys (consciousness evolution)

PERSONALITY LAYER
├── Enneagram (9 types)
└── Tarot (78 archetypal cards)

TIME LAYER
├── TCM Organ Clock (24-hour)
├── Vedic Panchanga (time qualities)
└── Vimshottari Dasha (life periods)

SPACE/BODY LAYER
├── Face Reading (Vedic + TCM)
├── Biofield (7 layers)
└── Sacred Geometry (patterns)
```

## File Statistics

- **Total JSON Files**: 36
- **Total Lines of JSON**: ~13,723
- **Average File Size**: ~393 lines
- **Largest File**: `i-ching/hexagrams.json` (1,905 lines)
- **Smallest File**: `sacred-geometry/templates.json` (47 lines)
- **Documentation**: 3,707 lines across 4 MD files

## Notes

- **Numerology Engine**: No data files (purely algorithmic calculations)
- **Sigil Forge Engine**: No data files (generative system using letter-to-geometry mapping)
- All data is programmatically accessible with clean JSON key-value structures
- Cross-referenceable using numeric IDs that match across systems
- API-ready structures suitable for RESTful interfaces
- Semantically rich with natural language descriptions

## Validation

To validate JSON integrity:

```bash
# Check file count
find data -name "*.json" -type f | wc -l  # Should be 36

# Validate all JSON syntax
for file in $(find data -name "*.json"); do
  jq empty "$file" || echo "Invalid JSON: $file"
done

# Check UTF-8 encoding
file -i data/**/*.json  # Should show charset=utf-8
```

---

*This wisdom data repository represents a comprehensive digital library of spiritual, archetypal, and diagnostic systems, meticulously structured for computational access while preserving the depth and nuance of ancient wisdom traditions.*
