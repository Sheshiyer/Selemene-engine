# Wisdom Data Schemas & Structures

> Complete schema documentation for all 35 WitnessOS wisdom data files

---

## Table of Contents

1. [Astrology/Vimshottari](#1-astrologyvimshottari)
2. [Biofield](#2-biofield)
3. [Biorhythm](#3-biorhythm)
4. [Enneagram](#4-enneagram)
5. [Face Reading](#5-face-reading)
6. [Gene Keys](#6-gene-keys)
7. [Human Design](#7-human-design)
8. [I Ching](#8-i-ching)
9. [Sacred Geometry](#9-sacred-geometry)
10. [Tarot](#10-tarot)
11. [TCM & Vedic Clock](#11-tcm--vedic-clock)

---

## 1. Astrology/Vimshottari

### `astrology/dasha_periods.json`

**Purpose**: Vimshottari Dasha system - planetary periods in Vedic astrology

**Schema**:
```json
{
  "dasha_system": {
    "name": "Vimshottari Dasha",
    "total_years": 120,
    "description": "..."
  },
  "planetary_periods": [
    {
      "planet": "Sun",
      "years": 6,
      "qualities": ["..."],
      "sub_periods": {
        "Sun": { "duration_months": "...", "theme": "..." },
        "Moon": { "duration_months": "...", "theme": "..." }
      }
    }
  ]
}
```

**Key Fields**:
- `planet`: Primary planetary ruler
- `years`: Main dasha duration
- `sub_periods`: Bhukti/Antardasha subdivisions
- `qualities`: Archetypal themes

---

### `astrology/nakshatras.json`

**Purpose**: 27 Lunar mansions with detailed attributes

**Schema**:
```json
{
  "nakshatra_info": { "total": 27, "arc": "13°20'" },
  "nakshatras": [
    {
      "number": 1,
      "name": "Ashwini",
      "sanskrit": "अश्विनी",
      "degree_range": "0°00' - 13°20' Aries",
      "ruling_planet": "Ketu",
      "deity": "Ashwini Kumaras",
      "symbol": "Horse's Head",
      "guna": "Rajas",
      "caste": "Vaishya",
      "animal": "Horse (Male)",
      "qualities": ["..."],
      "career_indicators": ["..."],
      "health_indicators": ["..."]
    }
  ]
}
```

**Key Fields**:
- `degree_range`: Zodiacal position
- `ruling_planet`: Dasha lord
- `deity`: Presiding divinity
- `animal`: Yoni (sexual compatibility)
- `guna`: Sattva/Rajas/Tamas

---

### `astrology/planets.json`

**Purpose**: Planetary bodies with significations

**Schema**:
```json
{
  "planets": [
    {
      "name": "Sun",
      "sanskrit": "Surya",
      "symbol": "☉",
      "element": "Fire",
      "gender": "Male",
      "nature": "Malefic (Cruel)",
      "body_parts": ["Heart", "Eyes", "Spine"],
      "significations": ["Soul", "Father", "Authority"],
      "exaltation": "Aries 10°",
      "debilitation": "Libra 10°",
      "friends": ["Moon", "Mars", "Jupiter"],
      "enemies": ["Venus", "Saturn"],
      "neutral": ["Mercury"]
    }
  ]
}
```

---

### `vimshottari_periods.json`

**Purpose**: Age-based planetary period calculations

**Schema**:
```json
{
  "planetary_sequence": ["Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury"],
  "period_durations": {
    "Ketu": 7, "Venus": 20, "Sun": 6,
    "Moon": 10, "Mars": 7, "Rahu": 18,
    "Jupiter": 16, "Saturn": 19, "Mercury": 17
  },
  "calculation_method": "Based on Moon's nakshatra at birth"
}
```

---

## 2. Biofield

### `biofield_spatial_algorithms.json`

**Purpose**: 7-layer aura geometry calculations

**Schema**:
```json
{
  "layers": [
    {
      "layer": 1,
      "name": "Etheric Body",
      "distance_from_skin": "0.5 - 2 inches",
      "color_primary": "Blue-gray",
      "geometric_shape": "Exact body outline",
      "function": "Physical vitality blueprint",
      "chakra_correspondence": "Root",
      "frequency_range": "...",
      "detection_method": "Kirlian photography"
    }
  ],
  "algorithms": {
    "layer_expansion": "distance = base_radius * (1 + phi^layer)",
    "color_blending": "HSV interpolation with harmonic ratios",
    "interference_patterns": "Standing wave calculations"
  }
}
```

---

### `biofield_temporal_algorithms.json`

**Purpose**: Time-based field dynamics

**Schema**:
```json
{
  "cycles": {
    "breath_cycle": {
      "duration": "4-6 seconds",
      "expansion_phase": "Inhalation",
      "contraction_phase": "Exhalation",
      "field_modulation": "±15% radius variation"
    },
    "heart_rate_variability": {
      "coherence_states": ["Low", "Medium", "High"],
      "field_harmonics": "..."
    }
  },
  "temporal_equations": {
    "phase_shift": "θ(t) = ω₀t + φ₀",
    "amplitude_modulation": "A(t) = A₀(1 + m·cos(ωₘt))"
  }
}
```

---

## 3. Biorhythm

### `age_points_mapping.json`

**Purpose**: Life cycle progression points

**Schema**:
```json
{
  "cycles": {
    "physical": { "period_days": 23, "phase_length": 11.5 },
    "emotional": { "period_days": 28, "phase_length": 14 },
    "intellectual": { "period_days": 33, "phase_length": 16.5 },
    "intuitive": { "period_days": 38, "phase_length": 19 }
  },
  "age_points": [
    {
      "age": 0, "phase": "Birth", "significance": "Origin point"
    },
    {
      "age": 7, "phase": "First Saturn Square", "significance": "Individuation begins"
    },
    {
      "age": 29.5, "phase": "Saturn Return", "significance": "Maturity threshold"
    }
  ]
}
```

---

## 4. Enneagram

### `enneagram/types.json`

**Purpose**: 9 personality types with centers, wings, arrows

**Schema**:
```json
{
  "enneagram_info": {
    "total_types": 9,
    "centers": 3,
    "system": "Riso-Hudson levels of development"
  },
  "centers": [
    {
      "name": "Instinctive (Gut)",
      "types": [8, 9, 1],
      "core_emotion": "Anger",
      "function": "Action and doing"
    }
  ],
  "types": {
    "1": {
      "name": "The Reformer",
      "center": "Instinctive",
      "core_motivation": "To be good, balanced, and right",
      "core_fear": "Being corrupt, evil, or defective",
      "core_desire": "Integrity and perfection",
      "passion": "Anger (hidden)",
      "virtue": "Serenity",
      "wings": ["9w1", "1w2"],
      "integration_arrow": 7,
      "disintegration_arrow": 4,
      "healthy_traits": ["..."],
      "average_traits": ["..."],
      "unhealthy_traits": ["..."],
      "subtypes": {
        "self_preservation": "...",
        "social": "...",
        "sexual": "..."
      }
    }
  },
  "assessment_questions": [
    {
      "question": "...",
      "type_indicators": { "1": 2, "2": 1, "8": 3 }
    }
  ]
}
```

**Key Concepts**:
- **Wings**: Adjacent types (e.g., 1w9, 1w2)
- **Arrows**: Integration (growth) and disintegration (stress)
- **Subtypes**: Self-preservation, Social, Sexual (One-to-One)
- **Levels of Development**: Healthy (1-3), Average (4-6), Unhealthy (7-9)

---

## 5. Face Reading

### `facial_landmark_mappings.json`

**Purpose**: 68-point facial coordinate system

**Schema**:
```json
{
  "landmark_system": "Dlib 68-point model",
  "regions": {
    "jaw": { "points": [0, 16], "total": 17 },
    "right_eyebrow": { "points": [17, 21], "total": 5 },
    "left_eyebrow": { "points": [22, 26], "total": 5 },
    "nose": { "points": [27, 35], "total": 9 },
    "right_eye": { "points": [36, 41], "total": 6 },
    "left_eye": { "points": [42, 47], "total": 6 },
    "mouth": { "points": [48, 67], "total": 20 }
  },
  "landmarks": [
    {
      "id": 0,
      "name": "Jaw Line Start",
      "region": "jaw",
      "coordinates": "(x, y)",
      "significance": "..."
    }
  ]
}
```

---

### `tcm_face_correlations.json`

**Purpose**: Traditional Chinese Medicine facial diagnostic zones

**Schema**:
```json
{
  "tcm_face_map": {
    "forehead": {
      "upper": { "organ": "Bladder", "element": "Water" },
      "middle": { "organ": "Small Intestine", "element": "Fire" },
      "lower": { "organ": "Heart", "element": "Fire" }
    },
    "cheeks": {
      "left": { "organ": "Liver", "element": "Wood" },
      "right": { "organ": "Lung", "element": "Metal" }
    },
    "nose": {
      "bridge": { "organ": "Liver", "emotion": "Anger" },
      "tip": { "organ": "Heart", "emotion": "Joy" },
      "nostrils": { "organ": "Lung", "emotion": "Grief" }
    }
  },
  "indicators": {
    "redness": "Excess heat/inflammation",
    "pallor": "Qi deficiency",
    "dark_circles": "Kidney deficiency",
    "lines": "Emotional holding patterns"
  }
}
```

---

### `vedic_face_correlations.json`

**Purpose**: Samudrika Shastra (Vedic physiognomy)

**Schema**:
```json
{
  "face_divisions": {
    "upper_face": {
      "region": "Forehead to eyebrows",
      "life_period": "0-25 years",
      "planets": ["Jupiter", "Sun"]
    },
    "middle_face": {
      "region": "Eyebrows to nose tip",
      "life_period": "26-50 years",
      "planets": ["Mars", "Venus"]
    },
    "lower_face": {
      "region": "Nose tip to chin",
      "life_period": "51+ years",
      "planets": ["Saturn", "Mercury"]
    }
  },
  "features": {
    "forehead": {
      "broad_smooth": "Intelligence, good fortune",
      "narrow_wrinkled": "Struggles, worry",
      "high": "Ambition",
      "low": "Practicality"
    }
  }
}
```

---

### `vedic_tcm_correspondences.json`

**Purpose**: Cross-system integration mapping

**Schema**:
```json
{
  "element_correspondences": {
    "Wood": {
      "tcm_organ": "Liver",
      "vedic_planet": "Jupiter",
      "dosha": "Kapha",
      "face_region": "Left cheek, temples"
    }
  },
  "planetary_organ_map": {
    "Sun": ["Heart", "Right eye"],
    "Moon": ["Stomach", "Left eye"],
    "Mars": ["Gallbladder", "Nose"],
    "Mercury": ["Nervous system", "Ears"],
    "Jupiter": ["Liver", "Forehead"],
    "Venus": ["Kidneys", "Cheeks"],
    "Saturn": ["Bones", "Chin"]
  }
}
```

---

## 6. Gene Keys

### `gene_keys/archetypes.json`

**Purpose**: 64 Gene Keys with Shadow-Gift-Siddhi spectrum

**Schema**:
```json
{
  "gene_keys_info": {
    "total": 64,
    "creator": "Richard Rudd",
    "system": "Synthesis of I Ching, Human Design, genetics"
  },
  "keys": {
    "1": {
      "hexagram": 1,
      "name": "Entropy → Freshness → Beauty",
      "shadow": {
        "name": "Entropy",
        "frequency": "7 Hz",
        "description": "Decay, staleness, repetition"
      },
      "gift": {
        "name": "Freshness",
        "frequency": "528 Hz",
        "description": "Spontaneity, originality, renewal"
      },
      "siddhi": {
        "name": "Beauty",
        "frequency": "1000+ Hz",
        "description": "Divine perfection, primordial creativity"
      },
      "codon_ring": "Ring of Light",
      "physiology": "Pineal gland",
      "amino_acid": "Lysine"
    }
  },
  "codon_rings": [
    {
      "name": "Ring of Light",
      "keys": [1, 7, 13, 49],
      "theme": "Illumination"
    }
  ]
}
```

**Key Concepts**:
- **Shadow**: Victim frequency (fear-based)
- **Gift**: Genius frequency (creative service)
- **Siddhi**: Divine frequency (enlightened state)
- **Codon Rings**: Genetic groupings of 4 keys

---

## 7. Human Design

### `human_design/gates.json`

**Purpose**: 64 I Ching gates with HD interpretations

**Schema**:
```json
{
  "1": {
    "number": 1,
    "hexagram_name": "The Creative",
    "hd_name": "Self-Expression",
    "center": "G (Identity)",
    "quarter": "Initiation",
    "theme": "Purpose fulfilled through Mind",
    "keynote": "Creative self-expression",
    "biological_correspondence": "DNA: Lysine",
    
    "lines": {
      "1": {
        "name": "Modesty",
        "description": "...",
        "exaltation": "...",
        "detriment": "..."
      }
    },
    
    "circuit": "Individual - Knowing",
    "channel_pairing": "Gate 8 (Channel 1-8: Inspiration)",
    
    "shadow": "Entropy - feeling stuck",
    "gift": "Freshness - creative renewal",
    "siddhi": "Beauty - divine perfection",
    
    "planetary_exaltation": "Uranus",
    "planetary_detriment": "Saturn",
    
    "chemical": "Lysine",
    "amino_acid_sequence": "AAA, AAG"
  }
}
```

---

### `human_design/centers.json`

**Purpose**: 9 energy centers (chakras in HD context)

**Schema**:
```json
{
  "centers": [
    {
      "name": "Head (Crown)",
      "type": "Pressure",
      "color": "Yellow-green",
      "biological": "Pineal gland",
      "function": "Inspiration, mental pressure",
      "gates": [64, 61, 63],
      "defined_experience": "Constant mental pressure to know",
      "undefined_experience": "Samples others' mental pressure",
      "open_experience": "Wisdom about mental pressure",
      "not_self_question": "Am I trying to answer everyone's questions?"
    }
  ]
}
```

---

### `human_design/channels.json`

**Purpose**: 36 channels (gate pairs forming definition)

**Schema**:
```json
{
  "channels": [
    {
      "number": "1-8",
      "name": "Channel of Inspiration",
      "type": "Projected",
      "circuit": "Individual - Knowing",
      "gates": {
        "1": { "center": "G", "role": "Self-Expression" },
        "8": { "center": "Throat", "role": "Contribution" }
      },
      "theme": "Creative role model",
      "description": "The channel of the creative genius...",
      "when_defined": "Natural ability to inspire through unique expression",
      "challenge": "Waiting for recognition before sharing"
    }
  ]
}
```

---

### `human_design/types.json`

**Purpose**: 5 energy types with strategies

**Schema**:
```json
{
  "types": [
    {
      "name": "Generator",
      "percentage": "37%",
      "strategy": "To Respond",
      "signature": "Satisfaction",
      "not_self_theme": "Frustration",
      "aura": "Open and enveloping",
      "definition": "Sacral defined",
      "role": "Builders of civilization",
      "sleep_strategy": "Go to bed only when exhausted"
    },
    {
      "name": "Manifesting Generator",
      "percentage": "33%",
      "strategy": "To Respond (then inform)",
      "signature": "Satisfaction + Peace",
      "not_self_theme": "Frustration + Anger",
      "aura": "Open and enveloping",
      "definition": "Sacral + Motor to Throat",
      "role": "Multi-passionate builders",
      "unique_trait": "Can skip steps"
    }
  ]
}
```

---

### `human_design/profiles.json`

**Purpose**: 12 life themes (conscious/unconscious line combinations)

**Schema**:
```json
{
  "profiles": [
    {
      "profile": "1/3",
      "name": "Investigator/Martyr",
      "theme": "Foundation through trial and error",
      "line_1_conscious": {
        "archetype": "Investigator",
        "mode": "Introspection",
        "need": "Solid foundation of knowledge"
      },
      "line_3_unconscious": {
        "archetype": "Martyr",
        "mode": "Experimentation",
        "gift": "Learning what doesn't work"
      },
      "life_purpose": "...",
      "interpersonal_style": "...",
      "career_indicators": "..."
    }
  ]
}
```

---

### `human_design/authorities.json`

**Purpose**: 7 decision-making inner authorities

**Schema**:
```json
{
  "authorities": [
    {
      "name": "Emotional/Solar Plexus",
      "percentage": "47%",
      "defined_center": "Solar Plexus",
      "decision_process": "Wait for emotional clarity (ride the wave)",
      "time_frame": "Days to weeks",
      "key_phrase": "There is no truth in the now",
      "guidance": "Never make decisions in the highs or lows"
    },
    {
      "name": "Sacral",
      "percentage": "30%",
      "defined_center": "Sacral (only Generators without emotional definition)",
      "decision_process": "Gut response (uh-huh / unh-unh)",
      "time_frame": "Immediate",
      "key_phrase": "Follow your gut",
      "guidance": "Pay attention to sacral sounds"
    }
  ]
}
```

---

### Additional HD Files

**`incarnation_crosses.json`**: 4-gate life purpose configurations  
**`circuitry.json`**: Individual, Tribal, Collective circuits  
**`definitions.json`**: Single, Split, Triple Split, Quadruple Split  
**`lines.json`**: 6 line themes (1-Foundation through 6-Role Model)  
**`planetary_activations.json`**: Birth/Design planetary positions  
**`variables.json`**: 16 cognitive/environmental arrows  

---

## 8. I Ching

### `iching/hexagrams.json` (Compact)

**Purpose**: Core 64 hexagram data

**Schema**:
```json
{
  "hexagram_info": {
    "total": 64,
    "source": "Wilhelm/Baynes translation",
    "king_wen_sequence": true
  },
  "methods": [
    "three_coin", "yarrow_stalks", "date_time", "computer_random"
  ],
  "hexagrams": {
    "1": {
      "number": 1,
      "chinese_name": "乾",
      "chinese_pinyin": "Qián",
      "english_name": "The Creative",
      "trigrams": {
        "upper": { "name": "Heaven", "symbol": "☰" },
        "lower": { "name": "Heaven", "symbol": "☰" }
      },
      "binary": "111111",
      "judgment": "The Creative works sublime success...",
      "image": "Heaven moves powerfully...",
      "keywords": ["Strength", "Creativity", "Yang energy"]
    }
  }
}
```

---

### `iching/hexagrams_complete.json` (Full)

**Purpose**: Complete hexagram data with line interpretations

**Additional Fields**:
```json
{
  "1": {
    ...base fields...,
    "lines": {
      "1": {
        "text": "Hidden dragon. Do not act.",
        "interpretation": "Time of preparation...",
        "changing_to": 44
      },
      "2": { ... },
      "3": { ... },
      "4": { ... },
      "5": { ... },
      "6": {
        "text": "Arrogant dragon will have cause to repent.",
        "interpretation": "Overextension leads to fall...",
        "changing_to": 1
      }
    },
    "nuclear_hexagram": 1,
    "opposite_hexagram": 2,
    "reverse_hexagram": 2,
    "related_hexagrams": [...]
  }
}
```

**Key Concepts**:
- **Nuclear hexagram**: Inner hexagram (lines 2-3-4 + 3-4-5)
- **Opposite**: Flip all lines (111111 → 000000)
- **Reverse**: Read bottom to top
- **Changing lines**: Create second hexagram for transformation

---

## 9. Sacred Geometry

### `sacred_geometry/symbols.json`

**Purpose**: Sacred geometric forms with meanings

**Schema**:
```json
{
  "symbols": [
    {
      "name": "Flower of Life",
      "category": "Foundation pattern",
      "geometry": {
        "base_shape": "Circle",
        "construction": "19 overlapping circles in hexagonal pattern",
        "dimensions": "2D (can be 3D as spheres)"
      },
      "meaning": {
        "ancient": "Blueprint of creation",
        "modern": "Fundamental pattern of space-time",
        "spiritual": "Unity of all life"
      },
      "found_in": [
        "Temple of Osiris, Egypt",
        "Forbidden City, China",
        "India, Turkey, various"
      ],
      "mathematical_properties": {
        "symmetry": "6-fold rotational",
        "contains": ["Seed of Life", "Tree of Life", "Platonic Solids"]
      },
      "frequencies": [432, 528, 639],
      "chakra_correspondence": "All chakras"
    }
  ]
}
```

---

### `sacred_geometry/templates.json`

**Purpose**: Construction algorithms for generating sacred geometry

**Schema**:
```json
{
  "templates": [
    {
      "name": "Metatron's Cube",
      "base_symbol": "Fruit of Life",
      "construction_steps": [
        "Draw Fruit of Life (13 circles)",
        "Connect centers of all circles",
        "13 vertices form Metatron's Cube",
        "Contains all 5 Platonic Solids"
      ],
      "svg_path": "...",
      "equations": {
        "vertex_positions": "...",
        "edge_lengths": "..."
      },
      "platonic_solids_encoded": [
        "Tetrahedron (4 faces)",
        "Cube (6 faces)",
        "Octahedron (8 faces)",
        "Dodecahedron (12 faces)",
        "Icosahedron (20 faces)"
      ]
    }
  ]
}
```

---

## 10. Tarot

### `tarot/major_arcana.json`

**Purpose**: 22 Major Arcana with archetypal depths

**Schema**:
```json
{
  "journey_structure": {
    "name": "The Fool's Journey",
    "stages": 3,
    "stage_1": "Cards 0-7 (Material World)",
    "stage_2": "Cards 8-14 (Inner World)",
    "stage_3": "Cards 15-21 (Cosmic Consciousness)"
  },
  "cards": [
    {
      "number": 0,
      "name": "The Fool",
      "archetype": "The Innocent",
      "hebrew_letter": "Aleph (א)",
      "element": "Air",
      "astrological_correspondence": "Uranus",
      "kabbalistic_path": "Aleph (Kether to Chokmah)",
      "numerology": "0 (Infinite potential)",
      
      "symbolism": {
        "dog": "Instinct/loyalty",
        "cliff": "Leap of faith",
        "white_rose": "Purity",
        "mountains": "Challenges ahead"
      },
      
      "upright_meaning": "New beginnings, innocence, spontaneity, free spirit",
      "reversed_meaning": "Recklessness, taken advantage of, inconsideration",
      
      "keywords": ["Beginnings", "Innocence", "Spontaneity", "Adventure"],
      
      "journey_stage": "Beginning",
      "life_lesson": "Trust in the journey",
      
      "chakra": "Crown",
      "color": "Yellow",
      "crystal": "Clear Quartz"
    }
  ],
  "integration_with_other_systems": {
    "kabbalah": "22 paths on Tree of Life",
    "astrology": "Planetary/zodiacal correspondences",
    "numerology": "Root numbers 0-21",
    "hebrew_alphabet": "22 letters"
  }
}
```

---

### `tarot/rider_waite.json`

**Purpose**: Complete 78-card deck (Major + Minor Arcana)

**Additional Content**:
```json
{
  "major_arcana": [ ... 22 cards ... ],
  "minor_arcana": {
    "suits": {
      "Wands": {
        "element": "Fire",
        "season": "Spring",
        "astrological_signs": ["Aries", "Leo", "Sagittarius"],
        "theme": "Action, creativity, passion",
        "cards": [
          {
            "name": "Ace of Wands",
            "number": 1,
            "upright": "Inspiration, new opportunities, growth",
            "reversed": "Delays, lack of motivation",
            "imagery": "Hand emerging from cloud holding flowering wand"
          }
        ]
      },
      "Cups": { "element": "Water", ... },
      "Swords": { "element": "Air", ... },
      "Pentacles": { "element": "Earth", ... }
    },
    "court_cards": {
      "Page": "Messenger, student, youthful energy",
      "Knight": "Action, movement, extremes",
      "Queen": "Nurturing, receptive, mature feminine",
      "King": "Authority, control, mature masculine"
    }
  },
  "spreads": [
    {
      "name": "Celtic Cross",
      "positions": 10,
      "layout": "..."
    }
  ]
}
```

---

## 11. TCM & Vedic Clock

### `tcm_organ_clock.json`

**Purpose**: 24-hour organ energy cycle

**Schema**:
```json
{
  "system": "Traditional Chinese Medicine Organ Clock",
  "cycle_duration": "24 hours",
  "organ_clock": [
    {
      "organ": "Liver",
      "element": "Wood",
      "yin_yang": "Yin",
      "time_range": "01:00-03:00",
      "peak_energy": "Detoxification and blood purification",
      "opposite_organ": "Small Intestine",
      "opposite_time": "13:00-15:00",
      
      "functions": [
        "Blood storage and distribution",
        "Emotional processing",
        "Planning and vision"
      ],
      
      "practices": {
        "optimal": "Deep sleep for regeneration",
        "contraindicated": "Heavy meals, alcohol"
      },
      
      "emotional_aspect": {
        "balanced": "Patience, decisiveness",
        "imbalanced": "Anger, frustration, irritability"
      },
      
      "physical_symptoms_when_imbalanced": [
        "Waking at this time repeatedly",
        "Eye problems",
        "Muscle tension"
      ],
      
      "meridian_points": ["LV-3", "LV-14"],
      "associated_season": "Spring"
    }
  ]
}
```

---

### `twelve_houses.json`

**Purpose**: Astrological houses and life areas

**Schema**:
```json
{
  "houses": [
    {
      "house": 1,
      "name": "House of Self",
      "natural_sign": "Aries",
      "natural_ruler": "Mars",
      "element": "Fire",
      "modality": "Angular",
      "life_areas": [
        "Physical appearance",
        "Personality",
        "First impressions",
        "Approach to life"
      ],
      "body_parts": "Head, face",
      "keyword": "Identity"
    }
  ]
}
```

---

### `five_elements_constitution.json`

**Purpose**: TCM constitutional types

**Schema**:
```json
{
  "elements": [
    {
      "element": "Wood",
      "season": "Spring",
      "direction": "East",
      "organs": {
        "yin": "Liver",
        "yang": "Gallbladder"
      },
      "color": "Green",
      "taste": "Sour",
      "emotion": {
        "balanced": "Kindness",
        "imbalanced": "Anger"
      },
      "physical_characteristics": {
        "body_type": "Lean, athletic",
        "face_shape": "Rectangular",
        "complexion": "Greenish tint when imbalanced"
      },
      "psychological_traits": [
        "Decisive", "Visionary", "Can be rigid"
      ],
      "imbalance_symptoms": [
        "Headaches", "Eye problems", "Tendon issues"
      ],
      "balancing_foods": ["Leafy greens", "Sour foods"],
      "cycle_relationships": {
        "generates": "Fire",
        "controls": "Earth",
        "generated_by": "Water",
        "controlled_by": "Metal"
      }
    }
  ]
}
```

---

### `panchanga_qualities.json`

**Purpose**: Vedic time quality calculations

**Schema**:
```json
{
  "panchanga_elements": {
    "tithi": {
      "name": "Lunar Day",
      "total": 30,
      "calculation": "Moon's longitude relative to Sun",
      "categories": [
        { "name": "Nanda", "numbers": [1, 6, 11, 16, 21, 26], "quality": "Joyful" },
        { "name": "Bhadra", "numbers": [2, 7, 12, 17, 22, 27], "quality": "Auspicious" }
      ]
    },
    "vara": {
      "name": "Weekday",
      "days": [
        { "name": "Sunday", "planet": "Sun", "deity": "Surya" },
        { "name": "Monday", "planet": "Moon", "deity": "Chandra" }
      ]
    },
    "nakshatra": {
      "name": "Lunar Mansion",
      "total": 27,
      "reference": "See nakshatras.json"
    },
    "yoga": {
      "name": "Luni-Solar Angle",
      "total": 27,
      "types": ["Vishkambha", "Priti", "Ayushman", ...]
    },
    "karana": {
      "name": "Half Tithi",
      "total": 11,
      "types": ["Bava", "Balava", "Kaulava", ...]
    }
  }
}
```

---

### `consciousness_practices.json`

**Purpose**: Time-based spiritual practices

**Schema**:
```json
{
  "daily_practices": [
    {
      "time": "03:00-06:00 (Brahma Muhurta)",
      "name": "Ambrosial Hours",
      "quality": "Sattva (Purity) predominant",
      "recommended_practices": [
        "Meditation",
        "Pranayama",
        "Scriptural study",
        "Mantra japa"
      ],
      "veil_thinness": "Thinnest - highest spiritual receptivity",
      "organs_active": ["Lungs", "Large Intestine"]
    }
  ],
  "lunar_practices": {
    "new_moon": "Intention setting, inner work",
    "waxing": "Growth activities, building",
    "full_moon": "Completion, meditation, fasting",
    "waning": "Release, detox, letting go"
  }
}
```

---

## Schema Validation Guidelines

### Common Patterns Across All Files

1. **UTF-8 Encoding**: All files support Unicode (Sanskrit, Chinese, Hebrew)
2. **Key-Based Access**: Primary data uses numeric or string keys
3. **Nested Structures**: 2-4 levels deep for complex systems
4. **Array vs Object**: Arrays for sequences, objects for lookups
5. **Metadata Sections**: Most files include `*_info` sections

### Data Type Conventions

```typescript
// Common TypeScript-style schema patterns
interface BaseWisdomData {
  name: string;
  description?: string;
  keywords?: string[];
  metadata?: object;
}

interface WithCorrespondences {
  element?: string;
  planet?: string;
  chakra?: string;
  color?: string;
  frequency?: number;
}
```

---

## Usage Notes

1. **JSON Parsing**: All files are valid JSON (use `json.load()` in Python)
2. **Cross-References**: Files reference each other (e.g., HD gates → I Ching hexagrams)
3. **Extensibility**: Schemas support additional fields without breaking
4. **Localization**: Unicode support enables multilingual expansions

---

**Last Updated**: 2026-01-26  
**Schema Version**: 1.0.0  
**Validation Status**: ✅ All 35 files structurally sound
