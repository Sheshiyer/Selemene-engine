# Engine-Data Mappings: How WitnessOS Engines Use Wisdom Data

> Comprehensive documentation of data flow from JSON files to Python engine implementations

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Engine-by-Engine Analysis](#engine-by-engine-analysis)
3. [Data Loading Patterns](#data-loading-patterns)
4. [Computation Flow](#computation-flow)
5. [Cross-Engine Integration](#cross-engine-integration)
6. [API Response Structures](#api-response-structures)

---

## Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    WitnessOS Engine Layer                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   Engine     │    │    Models    │    │  Data Loader │  │
│  │  (Logic)     │◄───┤  (Pydantic)  │◄───┤   (JSON)     │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                    │                    │         │
│         ▼                    ▼                    ▼         │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Wisdom Data (JSON Files)                │  │
│  │  • astrology/   • human_design/   • tarot/          │  │
│  │  • iching/      • enneagram/      • sacred_geo/     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow Pattern

```
User Input → Engine Method → Data Loader → JSON File
                    ↓
            Pydantic Models (validation)
                    ↓
            Business Logic (calculations)
                    ↓
            Response (enriched with wisdom data)
```

---

## Engine-by-Engine Analysis

### 1. Human Design Engine

**Files**: `/docs/engines/human_design.py` + `human_design_models.py`  
**Data**: `human_design/*.json` (12 files)

#### Data Loading

```python
# From human_design.py (simplified)
class HumanDesignEngine:
    def __init__(self):
        self.gates = self._load_json("human_design/gates.json")
        self.centers = self._load_json("human_design/centers.json")
        self.channels = self._load_json("human_design/channels.json")
        self.types = self._load_json("human_design/types.json")
        self.profiles = self._load_json("human_design/profiles.json")
        self.authorities = self._load_json("human_design/authorities.json")
        # ... load remaining files
    
    def _load_json(self, filepath):
        with open(DATA_PATH / filepath) as f:
            return json.load(f)
```

#### Key Methods & Data Usage

| Method | Input | Data Used | Output |
|--------|-------|-----------|--------|
| `calculate_chart()` | Birth data (date, time, location) | All HD data files | Complete HD chart object |
| `get_gates()` | Planetary positions | `gates.json` | Activated gates with meanings |
| `determine_type()` | Defined centers | `types.json`, `centers.json` | Energy type + strategy |
| `calculate_authority()` | Defined centers | `authorities.json` | Decision-making authority |
| `get_profile()` | Sun/Earth gates | `profiles.json`, `gates.json` → `lines.json` | Life theme profile |
| `find_channels()` | Activated gates | `channels.json` | Defined channels |
| `determine_definition()` | Channels | `definitions.json` | Definition type (single/split) |

#### Example Flow: Calculate Authority

```python
def calculate_authority(self, defined_centers):
    """
    Flow:
    1. Check if Solar Plexus defined → Emotional Authority
    2. Else if Sacral defined (& no SP) → Sacral Authority
    3. Else if Spleen defined → Splenic Authority
    4. ... continue through hierarchy
    """
    
    # Load authority hierarchy from authorities.json
    authorities = self.authorities["authorities"]
    
    # Priority order defined in data
    priority_order = [
        "Emotional/Solar Plexus",
        "Sacral", 
        "Splenic",
        "Ego/Heart",
        "G-Center/Self-Projected",
        "Mental/Environmental",
        "Lunar/Moon"
    ]
    
    for authority_name in priority_order:
        authority_data = next(a for a in authorities if a["name"] == authority_name)
        required_center = authority_data["defined_center"]
        
        if required_center in defined_centers:
            return {
                "type": authority_name,
                "description": authority_data["decision_process"],
                "guidance": authority_data["guidance"]
            }
```

#### Data Dependencies

```
Chart Calculation Dependencies:
├── gates.json (primary lookup)
├── centers.json (determine definitions)
├── channels.json (gate pairs)
├── types.json (Generator/Projector/etc.)
├── authorities.json (decision-making)
├── profiles.json (life themes)
├── lines.json (line meanings)
├── incarnation_crosses.json (life purpose)
├── variables.json (advanced - arrows)
├── planetary_activations.json (calculation method)
├── circuitry.json (circuit themes)
└── definitions.json (split types)
```

---

### 2. I Ching Engine

**Files**: `/docs/engines/iching.py` + `iching_models.py`  
**Data**: `iching/hexagrams.json`, `iching/hexagrams_complete.json`

#### Data Loading

```python
class IChingEngine:
    def __init__(self):
        # Use complete version for full line interpretations
        self.hexagrams = self._load_json("iching/hexagrams_complete.json")
        self.methods = self.hexagrams["methods"]
```

#### Key Methods & Data Usage

| Method | Input | Data Used | Output |
|--------|-------|-----------|--------|
| `cast_hexagram()` | Method (coins/yarrow/time) | `methods` | Primary hexagram number |
| `get_hexagram()` | Hexagram number | `hexagrams[number]` | Hexagram object with all data |
| `get_changing_lines()` | Hexagram + changing lines | `hexagrams[num].lines` | Line interpretations |
| `get_transformed()` | Primary hex + changing lines | `hexagrams` + binary math | Transformed hexagram |
| `get_nuclear()` | Hexagram number | Hexagram binary | Nuclear hexagram (inner) |
| `get_relating_hexagrams()` | Hexagram number | `hexagrams` | Opposite, reverse, nuclear |

#### Example Flow: Complete Reading

```python
def perform_reading(self, method="three_coin"):
    """
    Complete I Ching reading flow:
    1. Cast primary hexagram
    2. Determine changing lines
    3. Calculate transformed hexagram
    4. Calculate nuclear hexagram
    5. Return full reading with interpretations
    """
    
    # Step 1: Cast
    if method == "three_coin":
        lines, changing = self._three_coin_method()
    elif method == "yarrow_stalks":
        lines, changing = self._yarrow_stalks_method()
    
    # Convert binary to hexagram number
    primary_num = self._binary_to_hexagram(lines)
    primary_hex = self.hexagrams["hexagrams"][str(primary_num)]
    
    # Step 2: Get line interpretations
    line_meanings = []
    for line_num in changing:
        line_data = primary_hex["lines"][str(line_num)]
        line_meanings.append({
            "line": line_num,
            "text": line_data["text"],
            "interpretation": line_data["interpretation"]
        })
    
    # Step 3: Calculate transformed hexagram
    if changing:
        transformed_lines = self._apply_changes(lines, changing)
        transformed_num = self._binary_to_hexagram(transformed_lines)
        transformed_hex = self.hexagrams["hexagrams"][str(transformed_num)]
    else:
        transformed_hex = None
    
    # Step 4: Nuclear hexagram (lines 2-3-4 + 3-4-5)
    nuclear_num = primary_hex["nuclear_hexagram"]
    nuclear_hex = self.hexagrams["hexagrams"][str(nuclear_num)]
    
    return {
        "primary": {
            "number": primary_num,
            "name": primary_hex["english_name"],
            "chinese": primary_hex["chinese_name"],
            "judgment": primary_hex["judgment"],
            "image": primary_hex["image"]
        },
        "changing_lines": line_meanings,
        "transformed": transformed_hex if changing else None,
        "nuclear": {
            "number": nuclear_num,
            "name": nuclear_hex["english_name"],
            "meaning": "Inner dynamics"
        }
    }
```

#### Binary Conversion Logic

```python
def _binary_to_hexagram(self, lines):
    """
    Convert 6 lines to hexagram number using King Wen sequence
    
    Args:
        lines: List of 6 values (111111, 101010, etc.)
    
    Uses:
        hexagrams.json → binary field for reverse lookup
    """
    binary_str = ''.join(str(l) for l in lines)
    
    for num, hex_data in self.hexagrams["hexagrams"].items():
        if hex_data["binary"] == binary_str:
            return int(num)
    
    return None
```

---

### 3. Tarot Engine

**Files**: `/docs/engines/tarot.py` + `tarot_models.py`  
**Data**: `tarot/major_arcana.json`, `tarot/rider_waite.json`

#### Data Loading

```python
class TarotEngine:
    def __init__(self):
        self.major_arcana = self._load_json("tarot/major_arcana.json")
        self.full_deck = self._load_json("tarot/rider_waite.json")
        
        # Build lookup indices
        self.all_cards = self._build_card_index()
```

#### Key Methods & Data Usage

| Method | Input | Data Used | Output |
|--------|-------|-----------|--------|
| `draw_cards()` | Count, reversed? | `full_deck` | Random cards with orientation |
| `get_card()` | Card name/number | `all_cards` index | Card object |
| `perform_spread()` | Spread type (Celtic Cross, etc.) | `spreads` + drawn cards | Positional reading |
| `calculate_life_card()` | Birth date | `major_arcana` + numerology | Life path tarot card |
| `get_elemental_dignities()` | Card sequence | Card elements | Strengthening/weakening |

#### Example Flow: Celtic Cross Spread

```python
def perform_celtic_cross(self, shuffled_deck=None):
    """
    Celtic Cross: 10-card spread
    
    Uses:
    - tarot/rider_waite.json → full deck
    - Spread position meanings (could be in spreads.json)
    """
    
    # Default spread positions
    positions = [
        {"position": 1, "name": "Present Situation", "meaning": "Current state"},
        {"position": 2, "name": "Challenge", "meaning": "Crossing the situation"},
        {"position": 3, "name": "Distant Past", "meaning": "Foundation"},
        {"position": 4, "name": "Recent Past", "meaning": "Leaving behind"},
        {"position": 5, "name": "Best Outcome", "meaning": "Potential"},
        {"position": 6, "name": "Immediate Future", "meaning": "Coming soon"},
        {"position": 7, "name": "Self", "meaning": "Your role"},
        {"position": 8, "name": "Environment", "meaning": "External influences"},
        {"position": 9, "name": "Hopes/Fears", "meaning": "Inner desires/anxieties"},
        {"position": 10, "name": "Outcome", "meaning": "Final result"}
    ]
    
    # Draw 10 cards
    drawn_cards = self.draw_cards(count=10, allow_reversed=True)
    
    # Map cards to positions
    reading = []
    for i, position in enumerate(positions):
        card_data = drawn_cards[i]
        
        reading.append({
            "position": position,
            "card": card_data["card"],
            "orientation": card_data["orientation"],
            "interpretation": self._interpret_positional(
                card_data["card"], 
                position["meaning"],
                card_data["orientation"]
            )
        })
    
    return {
        "spread_type": "Celtic Cross",
        "cards": reading,
        "synthesis": self._synthesize_reading(reading)
    }
```

#### Elemental Dignities

```python
def calculate_elemental_dignities(self, card_sequence):
    """
    Analyzes how cards strengthen/weaken each other
    
    Uses:
    - Card element data (Fire/Water/Air/Earth)
    - Elemental relationship matrix
    """
    
    element_matrix = {
        "Fire": {"strengthens": ["Air"], "weakens": ["Water"]},
        "Water": {"strengthens": ["Earth"], "weakens": ["Fire"]},
        "Air": {"strengthens": ["Fire"], "weakens": ["Earth"]},
        "Earth": {"strengthens": ["Water"], "weakens": ["Air"]}
    }
    
    dignities = []
    for i in range(len(card_sequence) - 1):
        card1 = card_sequence[i]
        card2 = card_sequence[i + 1]
        
        elem1 = card1.get("element")
        elem2 = card2.get("element")
        
        if elem1 and elem2:
            if elem2 in element_matrix[elem1]["strengthens"]:
                dignities.append(f"{card1['name']} strengthens {card2['name']}")
            elif elem2 in element_matrix[elem1]["weakens"]:
                dignities.append(f"{card1['name']} weakens {card2['name']}")
    
    return dignities
```

---

### 4. Enneagram Engine

**Files**: `/docs/engines/enneagram.py` + `enneagram_models.py`  
**Data**: `enneagram/types.json`

#### Key Methods & Data Usage

| Method | Input | Data Used | Output |
|--------|-------|-----------|--------|
| `assess_type()` | Questionnaire responses | `assessment_questions` | Scored type results |
| `get_type_profile()` | Type number (1-9) | `types[number]` | Complete type description |
| `get_wing()` | Type + wing preference | `types` | Type with wing (e.g., 1w9) |
| `get_integration_path()` | Type number | `types[num].integration_arrow` | Growth direction |
| `get_disintegration_path()` | Type number | `types[num].disintegration_arrow` | Stress pattern |
| `get_instinctual_variant()` | Type + subtype | `types[num].subtypes` | SP/SO/SX variant |

#### Example Flow: Type Assessment

```python
def assess_type(self, responses):
    """
    Multi-question assessment to determine Enneagram type
    
    Uses:
    - enneagram/types.json → assessment_questions
    - Scoring algorithm
    """
    
    questions = self.data["assessment_questions"]
    type_scores = {str(i): 0 for i in range(1, 10)}
    
    # Score each response
    for question_id, answer_value in responses.items():
        question = questions[int(question_id)]
        type_indicators = question["type_indicators"]
        
        # Add weighted scores
        for type_num, weight in type_indicators.items():
            type_scores[type_num] += (answer_value * weight)
    
    # Find highest scoring types
    sorted_scores = sorted(type_scores.items(), key=lambda x: x[1], reverse=True)
    
    primary_type = sorted_scores[0][0]
    secondary_type = sorted_scores[1][0]
    
    return {
        "primary_type": self.get_type_profile(primary_type),
        "secondary_type": self.get_type_profile(secondary_type),
        "scores": type_scores,
        "confidence": sorted_scores[0][1] / sum(type_scores.values())
    }
```

---

### 5. Gene Keys Engine

**Files**: `/docs/engines/gene_keys.py` + `gene_keys_models.py`  
**Data**: `gene_keys/archetypes.json`

#### Key Methods & Data Usage

| Method | Input | Data Used | Output |
|--------|-------|-----------|--------|
| `get_gene_key()` | Key number (1-64) | `keys[number]` | Shadow/Gift/Siddhi data |
| `calculate_profile()` | Birth data | HD chart → gates → Gene Keys | Life's Work, Evolution, Radiance, Purpose |
| `get_codon_ring()` | Key number | `codon_rings` | Related keys in genetic grouping |
| `get_spectrum()` | Key number | `keys[num]` frequencies | Shadow → Gift → Siddhi with Hz |
| `calculate_venus_sequence()` | Birth data | Keys 1-64 in sequence | Relationship path |

#### Example Flow: Golden Path

```python
def calculate_golden_path(self, birth_datetime, birth_location):
    """
    Gene Keys Golden Path: 4 primary keys
    
    1. Life's Work (Sun in Personality)
    2. Evolution (Earth in Personality)
    3. Radiance (Sun in Design)
    4. Purpose (Earth in Design)
    
    Uses:
    - Human Design chart calculation
    - gene_keys/archetypes.json
    """
    
    # First, calculate HD chart to get gates
    hd_chart = self.hd_engine.calculate_chart(birth_datetime, birth_location)
    
    # Extract 4 key gates
    sun_personality = hd_chart["personality"]["sun"]["gate"]
    earth_personality = hd_chart["personality"]["earth"]["gate"]
    sun_design = hd_chart["design"]["sun"]["gate"]
    earth_design = hd_chart["design"]["earth"]["gate"]
    
    # Map gates to Gene Keys (1:1 correspondence)
    gene_keys_data = self.data["keys"]
    
    return {
        "lifes_work": {
            "gate": sun_personality,
            "gene_key": gene_keys_data[str(sun_personality)],
            "meaning": "Your life's purpose and service"
        },
        "evolution": {
            "gate": earth_personality,
            "gene_key": gene_keys_data[str(earth_personality)],
            "meaning": "How you attract purpose"
        },
        "radiance": {
            "gate": sun_design,
            "gene_key": gene_keys_data[str(sun_design)],
            "meaning": "Your creative expression"
        },
        "purpose": {
            "gate": earth_design,
            "gene_key": gene_keys_data[str(earth_design)],
            "meaning": "Your unconscious grounding"
        }
    }
```

---

### 6. Vimshottari/Astrology Engine

**Files**: `/docs/engines/vimshottari.py` + `vimshottari_models.py`  
**Data**: `astrology/dasha_periods.json`, `astrology/nakshatras.json`, `astrology/planets.json`, `vimshottari_periods.json`

#### Key Methods & Data Usage

| Method | Input | Data Used | Output |
|--------|-------|-----------|--------|
| `calculate_moon_nakshatra()` | Moon degree (0-360) | `nakshatras.json` | Nakshatra at birth |
| `get_starting_dasha()` | Moon nakshatra | `planetary_sequence` | Starting planetary period |
| `calculate_current_dasha()` | Birth date + current date | `period_durations` | Current Maha Dasha |
| `get_bhukti()` | Current dasha | `sub_periods` | Current sub-period |
| `get_dasha_effects()` | Planet name | `planets.json` + `dasha_periods` | Effects & themes |

#### Example Flow: Current Period

```python
def get_current_periods(self, birth_date, moon_degree):
    """
    Calculate Maha Dasha, Bhukti, Antara
    
    Uses:
    - astrology/nakshatras.json (starting point)
    - vimshottari_periods.json (durations)
    - astrology/dasha_periods.json (meanings)
    """
    
    # 1. Find birth nakshatra
    nakshatra_index = int(moon_degree / 13.333333)  # 27 nakshatras
    birth_nakshatra = self.nakshatras["nakshatras"][nakshatra_index]
    
    # 2. Determine starting dasha planet
    # Each nakshatra ruled by planet in repeating sequence
    planetary_sequence = self.periods["planetary_sequence"]
    starting_planet = planetary_sequence[nakshatra_index % 9]
    
    # 3. Calculate elapsed years
    today = datetime.date.today()
    years_elapsed = (today - birth_date).days / 365.25
    
    # 4. Find current Maha Dasha
    current_planet_index = planetary_sequence.index(starting_planet)
    accumulated_years = 0
    
    for cycle in range(10):  # Multiple full cycles possible
        for i in range(9):
            planet_index = (current_planet_index + i) % 9
            planet = planetary_sequence[planet_index]
            planet_years = self.periods["period_durations"][planet]
            
            if accumulated_years + planet_years > years_elapsed:
                # Found current Maha Dasha
                years_into_dasha = years_elapsed - accumulated_years
                
                # 5. Get sub-period (Bhukti)
                bhukti = self._calculate_bhukti(planet, years_into_dasha)
                
                # 6. Get meanings
                dasha_data = next(d for d in self.dasha_periods["planetary_periods"] 
                                 if d["planet"] == planet)
                
                return {
                    "maha_dasha": {
                        "planet": planet,
                        "total_years": planet_years,
                        "years_remaining": planet_years - years_into_dasha,
                        "qualities": dasha_data["qualities"],
                        "themes": dasha_data["description"]
                    },
                    "bhukti": bhukti,
                    "birth_nakshatra": birth_nakshatra["name"]
                }
            
            accumulated_years += planet_years
```

---

### 7. TCM/Vedic Clock Engine

**Files**: `/docs/engines/vedicclock_tcm.py` + `vedicclock_tcm_models.py`  
**Data**: `tcm_organ_clock.json`, `panchanga_qualities.json`, `consciousness_practices.json`, `twelve_houses.json`

#### Key Methods & Data Usage

| Method | Input | Data Used | Output |
|--------|-------|-----------|--------|
| `get_current_organ()` | Current time | `tcm_organ_clock` | Active organ + guidance |
| `calculate_panchanga()` | Date/time/location | `panchanga_qualities` | Tithi, Vara, Nakshatra, Yoga, Karana |
| `get_optimal_activities()` | Current time | `consciousness_practices` | Recommended practices |
| `get_house_timing()` | Hour | `twelve_houses` | Active astrological house |

---

### 8. Biofield Engine

**Files**: `/docs/engines/biofield.py` + `biofield_models.py`  
**Data**: `biofield_spatial_algorithms.json`, `biofield_temporal_algorithms.json`

#### Key Methods & Data Usage

| Method | Input | Data Used | Output |
|--------|-------|-----------|--------|
| `generate_field()` | Body position | `spatial_algorithms` | 7-layer aura geometry |
| `calculate_coherence()` | Heart rate, breath rate | `temporal_algorithms` | Field coherence score |
| `modulate_by_time()` | Time cycles | `temporal_algorithms` | Time-based field variations |
| `layer_properties()` | Layer number (1-7) | `layers` in algorithms | Layer attributes |

---

### 9-13. Other Engines

**Biorhythm**: Uses `age_points_mapping.json` for life cycle calculations  
**Numerology**: Mostly algorithmic, no dedicated data files  
**Face Reading**: Uses `facial_landmark_mappings.json`, `tcm_face_correlations.json`, `vedic_face_correlations.json`  
**Sacred Geometry**: Uses `sacred_geometry/symbols.json`, `templates.json`  
**Sigil Forge**: Generative, no data files (uses algorithmic letter-to-shape mapping)

---

## Data Loading Patterns

### Pattern 1: Singleton Data Manager

```python
class WisdomDataManager:
    """Centralized data loading with caching"""
    
    _instance = None
    _cache = {}
    
    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance
    
    def load(self, filepath):
        """Load and cache JSON data"""
        if filepath not in self._cache:
            with open(DATA_PATH / filepath) as f:
                self._cache[filepath] = json.load(f)
        return self._cache[filepath]

# Usage in engines
data_manager = WisdomDataManager()
gates = data_manager.load("human_design/gates.json")
```

### Pattern 2: Lazy Loading Properties

```python
class HumanDesignEngine:
    def __init__(self):
        self._gates = None
        self._centers = None
    
    @property
    def gates(self):
        """Lazy-load gates data"""
        if self._gates is None:
            self._gates = self._load_json("human_design/gates.json")
        return self._gates
    
    @property
    def centers(self):
        """Lazy-load centers data"""
        if self._centers is None:
            self._centers = self._load_json("human_design/centers.json")
        return self._centers
```

### Pattern 3: Pydantic Model Validation

```python
from pydantic import BaseModel, Field

# In *_models.py files
class HumanDesignGate(BaseModel):
    """Validated gate model"""
    number: int
    hd_name: str
    center: str
    shadow: str
    gift: str
    siddhi: str
    circuit: str
    biological_correspondence: Optional[str]

# Load and validate
def load_gates_validated():
    raw_data = load_json("human_design/gates.json")
    return {
        num: HumanDesignGate(**gate_data)
        for num, gate_data in raw_data.items()
    }
```

---

## Computation Flow

### Universal Engine Flow

```
1. DATA LOADING
   ↓
2. INPUT VALIDATION (Pydantic models)
   ↓
3. CALCULATION/LOOKUP
   ├─ Direct lookup (e.g., gate[1])
   ├─ Computed lookup (e.g., nakshatra by degree)
   ├─ Cross-reference (e.g., gate → channel)
   └─ Algorithmic (e.g., biorhythm cycles)
   ↓
4. DATA ENRICHMENT
   ├─ Add contextual meanings
   ├─ Cross-system references
   └─ Interpretive layers
   ↓
5. RESPONSE FORMATTING
   └─ JSON/Pydantic model output
```

### Example: Multi-Stage Lookup

```python
def enriched_gate_lookup(gate_number):
    """
    Multi-stage enrichment example
    
    Stages:
    1. Load gate from gates.json
    2. Find center from centers.json
    3. Find channel from channels.json
    4. Find Gene Key from gene_keys/archetypes.json
    5. Find I Ching from iching/hexagrams.json
    """
    
    # Stage 1: Base gate
    gate = gates_data[str(gate_number)]
    
    # Stage 2: Center details
    center = centers_data["centers"][gate["center"]]
    
    # Stage 3: Channel (if paired gate also active)
    channel_name = gate["channel_pairing"]
    channel = next((c for c in channels_data["channels"] 
                   if c["name"] == channel_name), None)
    
    # Stage 4: Gene Key
    gene_key = gene_keys_data["keys"][str(gate_number)]
    
    # Stage 5: I Ching
    hexagram = iching_data["hexagrams"][str(gate_number)]
    
    return {
        "gate": gate,
        "center": center,
        "channel": channel,
        "gene_key": gene_key,
        "iching": hexagram,
        "synthesis": generate_synthesis(gate, gene_key, hexagram)
    }
```

---

## Cross-Engine Integration

### Integration Point 1: Human Design ↔ Gene Keys

```python
# Both use same 64 gates/keys
hd_chart = hd_engine.calculate_chart(birth_data)
sun_gate = hd_chart["personality"]["sun"]["gate"]

# Use that gate number in Gene Keys
gene_key = gk_engine.get_gene_key(sun_gate)

result = {
    "gate_number": sun_gate,
    "hd_interpretation": hd_chart["gates"][sun_gate],
    "gene_key_interpretation": gene_key
}
```

### Integration Point 2: Astrology → Human Design

```python
# Calculate planetary positions (astrology engine)
planets = astro_engine.calculate_planets(birth_data)

# Use positions to find HD gates
sun_degree = planets["sun"]["longitude"]
sun_gate = hd_engine.degree_to_gate(sun_degree)

# Now have both astrological and HD info for Sun
```

### Integration Point 3: TCM ↔ Face Reading

```python
# Analyze face with facial landmarks
face_analysis = face_engine.analyze_face(image)

# Cross-reference with TCM zones
tcm_correlations = tcm_engine.map_face_to_organs(face_analysis)

# Vedic face reading
vedic_correlations = vedic_engine.map_face_features(face_analysis)

# Synthesize all three
synthesis = {
    "facial_features": face_analysis,
    "tcm_health_indicators": tcm_correlations,
    "vedic_destiny_markers": vedic_correlations,
    "integration": integration_data["vedic_tcm_correspondences"]
}
```

---

## API Response Structures

### Standard Response Format

All engines return JSON with:

```json
{
  "status": "success",
  "engine": "human_design",
  "timestamp": "2026-01-26T03:00:00Z",
  "input": {
    "birth_date": "1990-05-15",
    "birth_time": "14:30",
    "birth_location": "New York, NY"
  },
  "result": {
    // Engine-specific results
  },
  "metadata": {
    "calculation_time_ms": 45,
    "data_version": "1.0.0"
  }
}
```

### Human Design Response Example

```json
{
  "result": {
    "type": {
      "name": "Generator",
      "strategy": "To Respond",
      "authority": "Sacral"
    },
    "profile": {
      "profile": "1/3",
      "name": "Investigator/Martyr"
    },
    "defined_centers": ["Sacral", "G", "Throat"],
    "gates": {
      "personality": {
        "sun": { "gate": 1, "line": 3, "data": {...} },
        "earth": { "gate": 2, "line": 3, "data": {...} }
      },
      "design": { ... }
    },
    "channels": [
      {
        "name": "1-8",
        "title": "Channel of Inspiration"
      }
    ]
  }
}
```

### I Ching Response Example

```json
{
  "result": {
    "primary_hexagram": {
      "number": 1,
      "name": "The Creative",
      "chinese": "乾",
      "judgment": "...",
      "image": "...",
      "trigrams": {
        "upper": "Heaven",
        "lower": "Heaven"
      }
    },
    "changing_lines": [
      {
        "line": 3,
        "text": "...",
        "interpretation": "..."
      }
    ],
    "transformed_hexagram": {
      "number": 44,
      "name": "Coming to Meet",
      "meaning": "..."
    }
  }
}
```

---

## Performance Optimization

### Pre-computed Indices

```python
# Build lookup indices at engine initialization
class OptimizedEngine:
    def __init__(self):
        self.raw_data = self._load_json("data.json")
        
        # Pre-compute frequently used lookups
        self.by_center = self._index_by_center()
        self.by_circuit = self._index_by_circuit()
        self.by_keyword = self._index_by_keyword()
    
    def _index_by_center(self):
        """Build center → gates mapping"""
        index = defaultdict(list)
        for num, gate in self.raw_data.items():
            index[gate["center"]].append(num)
        return dict(index)
```

### Caching Expensive Calculations

```python
from functools import lru_cache

class Engine:
    @lru_cache(maxsize=1000)
    def calculate_chart(self, birth_datetime_tuple, birth_lat, birth_lon):
        """Cached chart calculation"""
        # Convert tuple back to datetime
        birth_datetime = datetime(*birth_datetime_tuple)
        
        # Expensive calculation here
        return self._compute_chart(birth_datetime, birth_lat, birth_lon)
```

---

## Summary: Data → Engine → Output

| Engine | Primary Data | Computation Type | Output |
|--------|--------------|------------------|--------|
| Human Design | 12 HD files | Chart calculation from planetary positions | Complete HD chart |
| I Ching | hexagrams.json | Hexagram casting + transformation | Reading with changing lines |
| Tarot | tarot/*.json | Card drawing + spread | Positional reading |
| Gene Keys | archetypes.json | HD gates → GK mapping | Shadow/Gift/Siddhi spectrum |
| Enneagram | types.json | Assessment + typing | Personality profile |
| Vimshottari | astrology/*.json | Dasha calculation from moon | Current planetary periods |
| TCM Clock | tcm_organ_clock.json | Time → organ mapping | Active organ + guidance |
| Biofield | biofield_*.json | Geometric/temporal algorithms | 7-layer aura field |
| Face Reading | facial_*.json | Landmark → zone mapping | TCM + Vedic correlations |
| Sacred Geometry | symbols.json | Template rendering | Geometric visualizations |
| Biorhythm | age_points.json | Cyclic calculations | Physical/emotional/intellectual cycles |
| Numerology | (algorithmic) | Date/name reduction | Life path, destiny numbers |
| Sigil Forge | (generative) | Letter → geometry mapping | Personalized sigils |

---

**Last Updated**: 2026-01-26  
**Version**: 1.0.0  
**Status**: ✅ Complete Documentation
