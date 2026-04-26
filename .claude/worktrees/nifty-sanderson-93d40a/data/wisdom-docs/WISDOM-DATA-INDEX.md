# WitnessOS Wisdom Data Index

> **Extracted from**: WitnessOS `/docs/api/engines/data/` & `/docs/engines/data/`  
> **Date**: 2026-01-26  
> **Status**: ✅ Complete Extraction (35 JSON files + 1 Python module)

---

## Overview

This directory contains the complete wisdom data repository extracted from WitnessOS, consolidating all 16 engine systems and their associated data files. These JSON datasets power the archetypal, symbolic, and wisdom-based calculations within WitnessOS engines.

---

## 📊 Data Architecture

### Total Assets
- **35 JSON Data Files**
- **1 Python Module** (gate_sequence.py)
- **13 Wisdom Systems**
- **7 Category Directories**

---

## 🔮 Wisdom System Catalog

### 1. **ASTROLOGY / VIMSHOTTARI** 
**Directory**: `astrology/`  
**Files**: 4 files

| File | Records | Purpose |
|------|---------|---------|
| `dasha_periods.json` | Vimshottari planetary periods | Vedic astrology timing system |
| `nakshatras.json` | 27 lunar mansions | Nakshatra qualities & attributes |
| `planets.json` | Planetary bodies data | Celestial influences & meanings |
| `vimshottari_periods.json` | Period calculations | Age-based planetary periods |

**Engine**: `vimshottari.py`  
**Data Usage**: Calculates planetary periods based on birth time and current age

---

### 2. **BIOFIELD**
**Directory**: Root level  
**Files**: 2 files

| File | Records | Purpose |
|------|---------|---------|
| `biofield_spatial_algorithms.json` | Spatial geometry algorithms | 7-layer aura field calculations |
| `biofield_temporal_algorithms.json` | Temporal algorithms | Time-based field dynamics |

**Engine**: `biofield.py`  
**Data Usage**: Generates multi-layered biofield visualizations with spatial-temporal coherence

---

### 3. **BIORHYTHM**
**Directory**: Root level  
**Files**: 1 file

| File | Records | Purpose |
|------|---------|---------|
| `age_points_mapping.json` | Age progression points | Life cycle biorhythm mapping |

**Engine**: `biorhythm.py`  
**Data Usage**: Calculates physical, emotional, intellectual, and intuitive cycles

---

### 4. **ENNEAGRAM**
**Directory**: `enneagram/`  
**Files**: 1 file

| File | Records | Purpose |
|------|---------|---------|
| `types.json` | 9 personality types + wings + centers | Complete Enneagram system |

**Engine**: `enneagram.py`  
**Data Usage**: Personality type assessment and integration/disintegration patterns

**Schema Structure**:
```json
{
  "enneagram_info": { metadata },
  "centers": [ "Instinctive", "Feeling", "Thinking" ],
  "assessment_questions": [ ... ],
  "types": {
    "1": { core_motivation, fear, desire, virtue, passion, ... }
  }
}
```

---

### 5. **FACE READING**
**Directory**: Root level  
**Files**: 4 files

| File | Records | Purpose |
|------|---------|---------|
| `facial_landmark_mappings.json` | 68 facial landmarks | Face geometry coordinate system |
| `tcm_face_correlations.json` | TCM diagnostic zones | Traditional Chinese Medicine face reading |
| `vedic_face_correlations.json` | Vedic face zones | Samudrika Shastra correlations |
| `vedic_tcm_correspondences.json` | Cross-system mapping | Integration between Vedic & TCM |

**Engine**: `face_reading.py`  
**Data Usage**: Analyzes facial features for health, personality, and destiny indicators

---

### 6. **GENE KEYS**
**Directory**: `gene_keys/`  
**Files**: 1 file

| File | Records | Purpose |
|------|---------|---------|
| `archetypes.json` | 64 Gene Keys with Shadow/Gift/Siddhi | Complete Gene Keys transmission |

**Engine**: `gene_keys.py`  
**Data Usage**: Maps I Ching hexagrams to consciousness frequencies (Shadow → Gift → Siddhi)

---

### 7. **HUMAN DESIGN**
**Directory**: `human_design/`  
**Files**: 12 files + 1 Python module

| File | Records | Purpose |
|------|---------|---------|
| `authorities.json` | 7 decision-making authorities | Inner authority types |
| `centers.json` | 9 energy centers | Defined/undefined centers |
| `channels.json` | 36 channels | Energy pathways between centers |
| `circuitry.json` | Circuit groupings | Individual, Tribal, Collective circuits |
| `definitions.json` | Definition types | Split definitions & configurations |
| `gates.json` | 64 gates | I Ching gates with HD interpretations |
| `incarnation_crosses.json` | Life purpose crosses | 4-gate incarnation crosses |
| `lines.json` | 6 line themes | Line-by-line meanings |
| `planetary_activations.json` | Planetary influences | Personality & Design calculations |
| `profiles.json` | 12 profiles | Role archetypes (1/3, 4/6, etc.) |
| `types.json` | 5 types + strategies | Generator, Projector, Manifestor, Reflector, Manifesting Generator |
| `variables.json` | 16 variables | Cognition & Environment arrows |
| `gate_sequence.py` | Python module | Gate ordering logic |

**Engine**: `human_design.py`  
**Data Usage**: Complete Human Design chart generation from birth data

**Schema Example** (`gates.json`):
```json
{
  "1": {
    "name": "The Creative",
    "keynote": "Self-Expression",
    "center": "G",
    "circuit": "Individual - Knowing",
    "shadow": "...",
    "gift": "...",
    "siddhi": "..."
  }
}
```

---

### 8. **I CHING**
**Directory**: `iching/`  
**Files**: 2 files

| File | Records | Purpose |
|------|---------|---------|
| `hexagrams.json` | 64 hexagrams (compact) | Core hexagram data |
| `hexagrams_complete.json` | 64 hexagrams (full) | Complete with line interpretations |

**Engine**: `iching.py`  
**Data Usage**: Generates hexagram readings with changing lines and transformations

**Schema Structure**:
```json
{
  "hexagram_info": { "total": 64, "version": "..." },
  "methods": [ "three_coin", "yarrow_stalks", "date_time" ],
  "hexagrams": {
    "1": {
      "number": 1,
      "chinese_name": "乾",
      "english_name": "The Creative",
      "trigrams": { "upper": "☰", "lower": "☰" },
      "judgment": "...",
      "image": "...",
      "lines": { ... }
    }
  }
}
```

---

### 9. **NUMEROLOGY**
**Directory**: Root level  
**Files**: 0 dedicated files *(calculated algorithmically)*

**Engine**: `numerology.py`  
**Data Usage**: Calculates Life Path, Destiny, Soul Urge numbers from birth data and names

---

### 10. **SACRED GEOMETRY**
**Directory**: `sacred_geometry/`  
**Files**: 2 files

| File | Records | Purpose |
|------|---------|---------|
| `symbols.json` | Sacred geometric symbols | Flower of Life, Metatron's Cube, Platonic Solids, etc. |
| `templates.json` | Geometric construction templates | Mathematical formulas for generation |

**Engine**: `sacred_geometry.py`  
**Data Usage**: Generates sacred geometric visualizations with meaning overlays

---

### 11. **SIGIL FORGE**
**Directory**: Root level  
**Files**: 0 dedicated files *(generative system)*

**Engine**: `sigil_forge.py`  
**Data Usage**: Creates personalized sigils from intention statements using letter-to-geometry mapping

---

### 12. **TAROT**
**Directory**: `tarot/`  
**Files**: 2 files

| File | Records | Purpose |
|------|---------|---------|
| `major_arcana.json` | 22 Major Arcana cards | Fool's Journey archetypes |
| `rider_waite.json` | 78-card deck (Major + Minor) | Complete Rider-Waite-Smith system |

**Engine**: `tarot.py`  
**Data Usage**: Performs multi-card spreads with positional meanings and elemental dignities

**Schema Example** (`major_arcana.json`):
```json
{
  "cards": [
    {
      "number": 0,
      "name": "The Fool",
      "archetype": "The Innocent",
      "element": "Air",
      "astrological_correspondence": "Uranus",
      "hebrew_letter": "Aleph",
      "keywords": ["Beginnings", "Innocence", "Spontaneity"],
      "upright_meaning": "...",
      "reversed_meaning": "...",
      "journey_stage": "Beginning"
    }
  ]
}
```

---

### 13. **VEDIC CLOCK & TCM**
**Directory**: Root level  
**Files**: 5 files

| File | Records | Purpose |
|------|---------|---------|
| `tcm_organ_clock.json` | 24-hour organ energy cycle | Traditional Chinese Medicine clock |
| `panchanga_qualities.json` | Vedic time qualities | Tithi, Vara, Nakshatra, Yoga, Karana |
| `twelve_houses.json` | Astrological houses | House meanings & life areas |
| `five_elements_constitution.json` | 5 Element constitutions | Wood, Fire, Earth, Metal, Water |
| `consciousness_practices.json` | Meditation & practices | Time-based spiritual practices |

**Engine**: `vedicclock_tcm.py`  
**Data Usage**: Real-time organ energy and auspicious timing calculations

**Schema Example** (`tcm_organ_clock.json`):
```json
{
  "organ_clock": [
    {
      "organ": "Liver",
      "element": "Wood",
      "time_range": "01:00-03:00",
      "peak_energy": "Detoxification and regeneration",
      "practices": ["Deep sleep", "Dreaming"],
      "emotional_aspect": "Anger / Planning"
    }
  ]
}
```

---

## 🗂️ File Organization Structure

```
wisdom-references/
├── WISDOM-DATA-INDEX.md              (this file)
├── DATA-SCHEMAS.md                   (detailed schema documentation)
├── QUICK-REFERENCE-GUIDE.md          (usage examples)
├── ENGINE-DATA-MAPPINGS.md           (how engines consume data)
│
├── astrology/
│   ├── dasha_periods.json
│   ├── nakshatras.json
│   ├── planets.json
│   └── vimshottari_periods.json
│
├── enneagram/
│   └── types.json
│
├── gene_keys/
│   └── archetypes.json
│
├── human_design/
│   ├── authorities.json
│   ├── centers.json
│   ├── channels.json
│   ├── circuitry.json
│   ├── definitions.json
│   ├── gates.json
│   ├── gate_sequence.py
│   ├── incarnation_crosses.json
│   ├── lines.json
│   ├── planetary_activations.json
│   ├── profiles.json
│   ├── types.json
│   └── variables.json
│
├── iching/
│   ├── hexagrams.json
│   └── hexagrams_complete.json
│
├── sacred_geometry/
│   ├── symbols.json
│   └── templates.json
│
├── tarot/
│   ├── major_arcana.json
│   └── rider_waite.json
│
└── (root level files)
    ├── age_points_mapping.json
    ├── biofield_spatial_algorithms.json
    ├── biofield_temporal_algorithms.json
    ├── consciousness_practices.json
    ├── facial_landmark_mappings.json
    ├── five_elements_constitution.json
    ├── panchanga_qualities.json
    ├── tcm_face_correlations.json
    ├── tcm_organ_clock.json
    ├── twelve_houses.json
    ├── vedic_face_correlations.json
    ├── vedic_tcm_correspondences.json
    └── vimshottari_periods.json
```

---

## 📈 Data Statistics

| System | JSON Files | Total Records | Complexity |
|--------|-----------|---------------|------------|
| Human Design | 12 | ~500+ | ★★★★★ |
| Astrology/Vimshottari | 4 | ~100 | ★★★★☆ |
| Face Reading | 4 | ~200+ | ★★★★☆ |
| I Ching | 2 | 64 hexagrams | ★★★★☆ |
| Tarot | 2 | 78 cards | ★★★☆☆ |
| TCM/Vedic Clock | 5 | ~150 | ★★★★☆ |
| Sacred Geometry | 2 | ~50 symbols | ★★★☆☆ |
| Biofield | 2 | Algorithmic | ★★★★☆ |
| Gene Keys | 1 | 64 keys | ★★★☆☆ |
| Enneagram | 1 | 9 types | ★★★☆☆ |
| Biorhythm | 1 | Algorithmic | ★★☆☆☆ |
| Numerology | 0 | Algorithmic | ★★☆☆☆ |
| Sigil Forge | 0 | Generative | ★★★☆☆ |

---

## 🔗 Related Documentation

1. **[DATA-SCHEMAS.md](./DATA-SCHEMAS.md)** - Detailed schema structures for each JSON file
2. **[QUICK-REFERENCE-GUIDE.md](./QUICK-REFERENCE-GUIDE.md)** - Code examples for accessing wisdom data
3. **[ENGINE-DATA-MAPPINGS.md](./ENGINE-DATA-MAPPINGS.md)** - How each engine consumes and processes data
4. **WitnessOS Engine Source**: `/01-Projects/WitnessOS/docs/engines/`
5. **API Documentation**: `/01-Projects/WitnessOS/docs/api/`

---

## 🛠️ Usage Guidelines

### For Developers

```python
# Example: Loading Human Design gates
import json

with open('wisdom-references/human_design/gates.json') as f:
    gates = json.load(f)
    
gate_1 = gates['1']
print(f"Gate {gate_1['name']}: {gate_1['keynote']}")
```

### For Researchers

Each JSON file is structured for:
- **Direct programmatic access** (key-based lookups)
- **Cross-referencing** (via ID/number keys)
- **Semantic analysis** (rich text descriptions)
- **API integration** (RESTful-friendly structures)

### For Mystics

This data represents:
- Millennia of accumulated wisdom traditions
- Mathematically precise symbolic correspondences
- Archetypal pattern languages
- Consciousness cartography

---

## 📝 Extraction Notes

### Source Integrity
- ✅ All data files preserved in original JSON format
- ✅ Directory structure maintained for organizational clarity
- ✅ No data transformations applied
- ✅ Python module (`gate_sequence.py`) included as-is

### Quality Assurance
- ✅ 35 JSON files validated and parseable
- ✅ Schema consistency verified
- ✅ Cross-references intact (e.g., Gate → Center → Circuit)
- ✅ UTF-8 encoding preserved (Chinese characters, Sanskrit, Hebrew)

### Future Enhancements
- [ ] JSON Schema validation files
- [ ] GraphQL API layer
- [ ] Vector embeddings for semantic search
- [ ] Cross-system integration mappings
- [ ] Multilingual translations

---

## 🔮 Wisdom Integration Matrix

| System A | System B | Integration File | Status |
|----------|----------|------------------|--------|
| Vedic | TCM | `vedic_tcm_correspondences.json` | ✅ |
| Face Reading | TCM | `tcm_face_correlations.json` | ✅ |
| Face Reading | Vedic | `vedic_face_correlations.json` | ✅ |
| Human Design | I Ching | `gates.json` (embedded) | ✅ |
| Gene Keys | I Ching | `archetypes.json` (embedded) | ✅ |
| Astrology | Vimshottari | `dasha_periods.json` | ✅ |
| Tarot | Astrology | `major_arcana.json` (embedded) | ✅ |
| Sacred Geometry | All Systems | Geometric templates | 🔄 |

---

## 💡 Key Insights

### Archetypal Foundations
- **64-based systems**: I Ching, Human Design, Gene Keys (hexagrams)
- **9-based systems**: Enneagram (personality types)
- **12-based systems**: Astrology houses, zodiac
- **7-based systems**: Chakras, biofield layers, Enneagram centers

### Cross-Cultural Synthesis
WitnessOS uniquely synthesizes:
- **Eastern traditions**: Vedic astrology, TCM, I Ching, Samudrika Shastra
- **Western esotericism**: Tarot, Kabbalah, Enneagram, Human Design
- **Modern frameworks**: Gene Keys, biofield science, sacred geometry
- **Indigenous wisdom**: Planetary cycles, natural rhythms

### Computational Mysticism
All wisdom data is:
- **Mathematically structured** (JSON key-value pairs)
- **Algorithmically accessible** (Python engines)
- **Semantically rich** (natural language descriptions)
- **Symbolically precise** (Unicode glyphs, hex codes)

---

## 📞 Contact & Contribution

**Curator**: Tryambakam Noesis Evolution Team  
**Source**: WitnessOS Project  
**License**: Proprietary (see WitnessOS licensing)

For data corrections, enhancements, or integration requests:
- Submit issues to the WitnessOS repository
- Propose schema improvements via pull requests
- Document new cross-system correlations

---

*"The universe is made of information, and wisdom is its structure."*  
— Tryambakam Noesis Axiom #7

---

**Last Updated**: 2026-01-26  
**Document Version**: 1.0.0  
**Status**: 🟢 Complete & Validated
