# Vimshottari Engine - Data Structures & Wisdom Loading Implementation

**Tasks Completed: W1-S6-01 & W1-S6-02**

## Files Created/Modified

### 1. Data Structures (`src/models.rs`)
Complete 3-level Vimshottari dasha hierarchy:

#### Core Structures:
- **VimshottariChart** - Complete timeline with birth date, all mahadashas, current period, upcoming transitions
- **Mahadasha** - Major period (9 total) with planet, dates, duration, antardashas, qualities
- **Antardasha** - Sub-period (9 per mahadasha) with pratyantardashas
- **Pratyantardasha** - Sub-sub-period (9 per antardasha) with duration in days
- **CurrentPeriod** - Active periods at all 3 levels with end dates
- **Transition** - Period change events with level indicator

#### Vedic Planet System:
- **VedicPlanet** enum - 9 planets (Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury, Ketu, Venus)
- Methods:
  - `period_years()` → Returns duration (6-20 years)
  - `next_planet()` → Returns next in cycle
  - `as_str()` → String representation
  - `from_str()` → Parse from string

#### Nakshatra System:
- **Nakshatra** struct - 27 lunar mansions with:
  - number (1-27), name, ruling planet
  - start/end degrees (0-360°)
  - deity, symbol, qualities, description

#### Planetary Qualities:
- **PlanetaryQualities** struct - Complete wisdom data:
  - themes (e.g., "expansion", "wisdom")
  - qualities (e.g., "teaching", "spirituality")
  - element (Fire, Water, Earth, Air, Ether)
  - description
  - consciousness_lessons (5-7 per planet)
  - optimal_practices (5-7 per planet)
  - challenges (3-4 per planet)

### 2. Wisdom Data Structures (`src/wisdom.rs`)
JSON deserialization structures for 3 wisdom files:

- **VimshottariPeriodsData** - Parses `vimshottari_periods.json`
  - periods → PeriodInfo (years, element, qualities, themes)
  - planetary_qualities → QualityDetails (lessons, practices, challenges)

- **NakshatrasData** - Parses `nakshatras.json`
  - nakshatras_info → metadata
  - nakshatras → HashMap of NakshatraEntry

- **DashaPeriodsData** - Parses `dasha_periods.json`
  - mahadasha_periods → PeriodDuration
  - planetary_order → sequence
  - nakshatra_rulers → 1-27 mapping

### 3. Static Wisdom Loading (`src/wisdom_data.rs`)
Lazy-loaded static references using `lazy_static!`:

#### Static Refs:
```rust
VIMSHOTTARI_PERIODS: HashMap<VedicPlanet, u8>     // 9 planets → years
NAKSHATRAS: Vec<Nakshatra>                        // 27 nakshatras sorted
PLANETARY_QUALITIES: HashMap<VedicPlanet, PlanetaryQualities>  // 9 planets
NAKSHATRA_RULERS: HashMap<u8, VedicPlanet>        // 1-27 → ruling planet
PLANETARY_ORDER: Vec<VedicPlanet>                  // Cycle sequence
```

#### Load Functions:
- `load_periods()` → Reads dasha_periods.json, extracts planet durations
- `load_nakshatras()` → Reads nakshatras.json, sorts by number
- `load_qualities()` → Reads vimshottari_periods.json, merges themes + detailed qualities
- `load_nakshatra_rulers()` → Extracts nakshatra → planet mapping

#### Helper Functions:
- `init_wisdom()` → Forces lazy_static initialization
- `get_nakshatra_from_longitude(f64)` → Moon longitude → Nakshatra
- `get_nakshatra_by_number(u8)` → Number → Nakshatra

### 4. Module Exports (`src/lib.rs`)
```rust
pub mod models;
pub mod wisdom;
pub mod wisdom_data;
pub use models::*;
pub use wisdom_data::*;
```

## Wisdom Data Files Loaded

### 1. `/data/vimshottari/dasha_periods.json`
- 9 planet periods (Sun: 6y, Moon: 10y, ... Venus: 20y)
- Total: 120 years
- Planetary sequence order
- 27 nakshatra ruler mappings

### 2. `/data/vimshottari/nakshatras.json`
- 27 nakshatras (Ashwini → Revati)
- Each with: number, name, degrees, ruling planet
- Deity, symbol, nature, gana, qualities, description
- Complete 0-360° zodiac coverage

### 3. `/data/vimshottari/vimshottari_periods.json`
- 9 planet period info (years, element, qualities, themes)
- Planetary qualities details for 5 planets:
  - Jupiter: expansion, teaching, higher learning
  - Saturn: discipline, structure, karmic lessons
  - Mercury: communication, intelligence, adaptability
  - Venus: love, beauty, creativity
  - Mars: courage, action, energy
- Each with consciousness_lessons, optimal_practices, challenges

## Data Verification

### Planetary Periods (9 total):
- Ketu: 7 years
- Venus: 20 years
- Sun: 6 years
- Moon: 10 years
- Mars: 7 years
- Rahu: 18 years
- Jupiter: 16 years
- Saturn: 19 years
- Mercury: 17 years
**Total: 120 years ✓**

### Nakshatras (27 total):
1. Ashwini (0°-13.33°) → Ketu
2. Bharani (13.33°-26.67°) → Venus
...
27. Revati (346.67°-360°) → Mercury
**Coverage: 0-360° ✓**

### Planetary Qualities (9 total):
- All 9 planets have themes and element
- 5 planets (Jupiter, Saturn, Mercury, Venus, Mars) have detailed qualities:
  - Consciousness lessons: 5-7 items each
  - Optimal practices: 5-7 items each
  - Challenges: 3-4 items each

## Schema Discoveries

1. **Nakshatras JSON**: Uses string keys ("1", "2", ...) for nakshatras, not array
2. **Planetary Qualities**: Split into 2 sections in vimshottari_periods.json:
   - "periods" → basic info (all 9 planets)
   - "planetary_qualities" → detailed info (5 planets: Jupiter, Saturn, Mercury, Venus, Mars)
3. **Nakshatra Degrees**: Some precision loss in JSON (e.g., 13.333333 vs exact 40/3)
4. **Ruling Planet Mapping**: Duplicated in 2 files (nakshatras.json and dasha_periods.json)
5. **Planetary Order**: Standard Vimshottari sequence (not alphabetical)

## Implementation Notes

### Serde Compatibility:
- All structs use `#[derive(Serialize, Deserialize)]`
- DateTime<Utc> from chrono for timestamps
- VedicPlanet implements Hash + Eq for HashMap keys

### Pattern Matching HD Engine:
- Same lazy_static pattern as engine-human-design
- Similar wisdom_data.rs structure
- init_wisdom() function for startup loading
- Test coverage in each module

### Compilation:
- Dependencies: chrono, serde, serde_json, lazy_static
- All modules use `use crate::models::*` imports
- include_str! for compile-time JSON embedding

## Next Steps (Not in Scope)

1. Implement calculation engine
2. Add Mahadasha/Antardasha/Pratyantardasha subdivision logic
3. Create CurrentPeriod determination from birth date + Moon position
4. Add transition date calculations
5. Wire to API endpoints

## Build Command
```bash
cargo build -p engine-vimshottari
```

## Usage Example
```rust
use engine_vimshottari::*;

// Initialize wisdom data
wisdom_data::init_wisdom();

// Access static data
let periods = &wisdom_data::VIMSHOTTARI_PERIODS;
let nakshatras = &wisdom_data::NAKSHATRAS;
let qualities = &wisdom_data::PLANETARY_QUALITIES;

// Get nakshatra from Moon longitude
if let Some(nak) = wisdom_data::get_nakshatra_from_longitude(125.0) {
    println!("{} ruled by {}", nak.name, nak.ruling_planet.as_str());
}

// Get planet period
let jupiter_years = VedicPlanet::Jupiter.period_years();  // 16
```

## Acceptance Criteria Met ✓

✅ Serde-compatible structs for 3-level dasha nesting  
✅ VedicPlanet enum with period_years() and next_planet()  
✅ All 3 wisdom files loaded (dasha_periods, nakshatras, vimshottari_periods)  
✅ 27 nakshatras loaded with ruling planets  
✅ 9 planetary qualities loaded  
✅ Static refs available (lazy_static pattern)  
✅ Code structure matches requirements
