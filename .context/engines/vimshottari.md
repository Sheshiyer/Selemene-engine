# Vimshottari Dasha Consciousness Engine

## Overview

### What is Vimshottari Dasha?

Vimshottari Dasha is the most widely used planetary period system in Vedic astrology. It divides the human lifespan into a 120-year cycle of planetary periods, each governed by one of 9 Vedic planets. The system provides a timeline of consciousness themes, life experiences, and karmic patterns based on the Moon's position at birth.

The word "Vimshottari" means "120" in Sanskrit, referring to the total cycle duration. The system is based on the relationship between the 27 Nakshatras (lunar mansions) and 9 Vedic planets.

### What Does This Engine Calculate?

The Vimshottari engine generates a complete 120-year planetary timeline with 3 levels of nested periods:

**3-Level Hierarchy:**
- **Mahadasha** (Major Period): 9 periods, 6-20 years each, totaling 120 years
- **Antardasha** (Sub-Period): 81 sub-periods (9 per Mahadasha)
- **Pratyantardasha** (Sub-Sub-Period): 729 micro-periods (9 per Antardasha)

**Key Outputs:**
- Birth nakshatra determination from Moon longitude
- Complete 120-year timeline with exact dates
- Current active period detection (all 3 levels)
- Upcoming transitions at all hierarchy levels
- Planetary quality enrichment (themes, life areas, challenges, opportunities)
- Consciousness-adapted witness prompts

---

## Architecture

### Module Breakdown

```
engine-vimshottari/src/
|-- lib.rs              # Public API exports and re-exports (29 lines)
|-- models.rs           # Data structures: Mahadasha, Antardasha, Pratyantardasha, etc. (250 lines)
|-- calculator.rs       # Core calculation engine: nakshatras, dashas, timeline (1,304 lines)
|-- wisdom.rs           # Wisdom data deserialization structures (82 lines)
|-- wisdom_data.rs      # Static wisdom data loader + planetary qualities (486 lines)
|-- witness.rs          # Consciousness-level-adaptive witness prompts (289 lines)
```

**Total**: 2,440 lines of Rust source code

### Data Flow

```
Birth Data (via EngineInput)
        |
        v
+----------------------------+
| VimshottariEngine          |
| (ConsciousnessEngine trait)|
+----------------------------+
        |
        |-- 1. Get Moon longitude via Swiss Ephemeris (from HD engine)
        |
        v
+----------------------------+
| Nakshatra Determination    |
| (calculator.rs)            |
+----------------------------+
        |
        |-- Moon longitude / 13.333 = nakshatra index (0-26)
        |-- Ruling planet determines starting Mahadasha
        |
        v
+----------------------------+
| Dasha Balance Calculation  |
| (calculator.rs)            |
+----------------------------+
        |
        |-- Remaining degrees in nakshatra -> fraction remaining
        |-- fraction * planet_period_years = balance_years
        |
        v
+----------------------------+
| Timeline Generation        |
| (calculator.rs)            |
+----------------------------+
        |
        |-- 9 Mahadashas (first uses balance, rest use full periods)
        |-- 81 Antardashas (9 per Mahadasha)
        |-- 729 Pratyantardashas (9 per Antardasha)
        |
        v
+----------------------------+      +----------------------------+
| Current Period Detection   |      | Upcoming Transitions       |
| Binary search O(log 729)  |      | Forward iteration          |
+----------------------------+      +----------------------------+
        |                                  |
        v                                  v
+----------------------------+      +----------------------------+
| Period Enrichment          |      | Witness Prompt Generation  |
| (wisdom_data.rs)           |      | (witness.rs)               |
+----------------------------+      +----------------------------+
        |                                  |
        v                                  v
+--------------------------------------------------------------+
| EngineOutput (JSON)                                           |
| - mahadashas: [{planet, start, end, duration, antardashas}]   |
| - current_period: {mahadasha, antardasha, pratyantardasha}    |
| - upcoming_transitions: [{type, from, to, date, days_until}]  |
| - witness_prompt: "consciousness-level-adapted question..."    |
+--------------------------------------------------------------+
```

---

## Calculations

### Nakshatra Determination

The Moon's ecliptic longitude (0-360 degrees) maps to one of 27 Nakshatras:

**Formula:**
```
nakshatra_index = floor(moon_longitude / 13.333333)
nakshatra_number = nakshatra_index + 1  (1-27)
```

Each nakshatra spans exactly 13.333333 degrees (360 / 27).

**The 27 Nakshatras and Their Rulers:**

| # | Nakshatra | Degrees | Ruler | Symbol |
|---|-----------|---------|-------|--------|
| 1 | Ashwini | 0.000 - 13.333 | Ketu | Horse's Head |
| 2 | Bharani | 13.333 - 26.667 | Venus | Yoni |
| 3 | Krittika | 26.667 - 40.000 | Sun | Razor |
| 4 | Rohini | 40.000 - 53.333 | Moon | Cart |
| 5 | Mrigashira | 53.333 - 66.667 | Mars | Deer's Head |
| 6 | Ardra | 66.667 - 80.000 | Rahu | Teardrop |
| 7 | Punarvasu | 80.000 - 93.333 | Jupiter | Bow and Quiver |
| 8 | Pushya | 93.333 - 106.667 | Saturn | Cow's Udder |
| 9 | Ashlesha | 106.667 - 120.000 | Mercury | Serpent |
| 10 | Magha | 120.000 - 133.333 | Ketu | Throne |
| 11 | Purva Phalguni | 133.333 - 146.667 | Venus | Hammock |
| 12 | Uttara Phalguni | 146.667 - 160.000 | Sun | Bed |
| 13 | Hasta | 160.000 - 173.333 | Moon | Hand |
| 14 | Chitra | 173.333 - 186.667 | Mars | Pearl |
| 15 | Swati | 186.667 - 200.000 | Rahu | Coral |
| 16 | Vishakha | 200.000 - 213.333 | Jupiter | Triumphal Arch |
| 17 | Anuradha | 213.333 - 226.667 | Saturn | Lotus |
| 18 | Jyeshtha | 226.667 - 240.000 | Mercury | Earring |
| 19 | Mula | 240.000 - 253.333 | Ketu | Root |
| 20 | Purva Ashadha | 253.333 - 266.667 | Venus | Elephant Tusk |
| 21 | Uttara Ashadha | 266.667 - 280.000 | Sun | Planks |
| 22 | Shravana | 280.000 - 293.333 | Moon | Ear |
| 23 | Dhanishta | 293.333 - 306.667 | Mars | Drum |
| 24 | Shatabhisha | 306.667 - 320.000 | Rahu | Empty Circle |
| 25 | Purva Bhadrapada | 320.000 - 333.333 | Jupiter | Sword |
| 26 | Uttara Bhadrapada | 333.333 - 346.667 | Saturn | Twin |
| 27 | Revati | 346.667 - 360.000 | Mercury | Fish |

Note: The ruler pattern repeats 3 times: Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury.

### Dasha Sequence (Vimshottari Order)

The 9 planets cycle in this fixed order, with period durations:

| Planet | Period (Years) | Themes |
|--------|---------------|--------|
| Ketu | 7 | Spirituality, detachment, liberation |
| Venus | 20 | Love, beauty, relationships, luxury |
| Sun | 6 | Self-expression, authority, vitality |
| Moon | 10 | Emotions, nurturing, intuition |
| Mars | 7 | Action, courage, conflict, energy |
| Rahu | 18 | Ambition, innovation, obsession |
| Jupiter | 16 | Growth, wisdom, expansion, teaching |
| Saturn | 19 | Discipline, karma, structure, maturity |
| Mercury | 17 | Communication, learning, intellect |

**Total: 7 + 20 + 6 + 10 + 7 + 18 + 16 + 19 + 17 = 120 years**

### Balance Calculation (First Dasha)

The first Mahadasha is typically partial - the person is born partway through it. The balance (remaining portion) is calculated from the Moon's position within its nakshatra:

**Formula:**
```
position_in_nakshatra = moon_longitude - nakshatra_start_degree
remaining_degrees = nakshatra_end_degree - moon_longitude
fraction_remaining = remaining_degrees / 13.333333
balance_years = fraction_remaining * planet_period_years
```

**Example:**
- Moon at 125.0 degrees (in Magha nakshatra, 120-133.333)
- Remaining: (133.333 - 125.0) / 13.333 = 0.625
- Magha ruler = Ketu (7 years)
- Balance = 0.625 * 7 = 4.375 years of Ketu Mahadasha remaining

### Mahadasha Generation

Starting from the birth nakshatra's ruler with the balance period:

1. First Mahadasha: `balance_years` duration (partial)
2. Subsequent 8 Mahadashas: Full `period_years` duration each
3. Planet sequence: Starting planet -> next_planet() -> ... (cycles through all 9)
4. Date calculation: `end_date = start_date + (duration_years * 365.25) days`

### Antardasha Subdivision

Each Mahadasha is divided into 9 Antardashas:

**Duration Formula:**
```
antardasha_duration_years = (mahadasha_duration_years * antardasha_planet_period_years) / 120
```

**Sequence Rule:** Antardasha sequence starts with the Mahadasha lord, then cycles through the Vimshottari order.

**Example (Jupiter Mahadasha, 16 years):**
- Jupiter Antardasha: (16 * 16) / 120 = 2.133 years
- Saturn Antardasha: (16 * 19) / 120 = 2.533 years
- Mercury Antardasha: (16 * 17) / 120 = 2.267 years
- ... and so on for all 9 planets

**Verification:** Sum of all 9 Antardashas = Mahadasha duration (within rounding tolerance).

### Pratyantardasha Subdivision

Each Antardasha is divided into 9 Pratyantardashas using the same formula:

**Duration Formula:**
```
pratyantardasha_duration_years = (antardasha_duration_years * pratyantardasha_planet_period_years) / 120
pratyantardasha_duration_days = pratyantardasha_duration_years * 365.25
```

**Sequence Rule:** Pratyantardasha sequence starts with the Antardasha lord.

**Complete Structure:**
- 9 Mahadashas (120 years)
- 81 Antardashas (9 per Mahadasha)
- 729 Pratyantardashas (9 per Antardasha)

### Current Period Detection

Finding the active period at any point in time uses a binary search algorithm:

**Algorithm (O(log 729)):**
1. Flatten 3-level hierarchy into linear array of 729 Pratyantardashas (with parent references)
2. Binary search for the period containing `current_time`
3. Walk up to find parent Antardasha and Mahadasha
4. Return `CurrentPeriod` with all 3 levels

**Performance:** ~10 comparisons (log2(729) = 9.5)

### Upcoming Transitions

Calculate next N transitions at all hierarchy levels:

**Algorithm:**
1. Find current position via `find_current_period()`
2. Iterate forward through flattened periods
3. Detect level changes:
   - Mahadasha transition: Parent Mahadasha planet changed (highest priority)
   - Antardasha transition: Parent Antardasha planet changed
   - Pratyantardasha transition: Period planet changed
4. Return first N transitions, chronologically ordered

**Hierarchy guarantee:** Pratyantardasha transitions are most frequent, Mahadasha transitions are rarest.

---

## Testing

### Test Coverage Summary

**42+ tests across all modules:**

**Nakshatra Tests (4):**
- 27 nakshatras loaded correctly
- Full 0-360 degree coverage
- Ruling planet pattern (3x repeat of 9 planets)
- Specific longitude lookups (Magha at 125, Ashwini at 5, Revati at 355)

**Balance Calculation Tests (3):**
- Standard case: Moon at 125 in Magha = 4.375 years
- At start of nakshatra: Full period remaining
- At end of nakshatra: Near-zero remaining

**Mahadasha Tests (4):**
- 120-year cycle total verification
- Planet sequence correctness
- Date progression continuity
- Balance period for first Mahadasha

**Antardasha Tests (5):**
- 9 subdivisions per Mahadasha
- Starts with Mahadasha lord
- Duration formula: (M * A) / 120
- Durations sum to Mahadasha
- Date continuity (no gaps)

**Pratyantardasha Tests (4):**
- 9 subdivisions per Antardasha
- Starts with Antardasha lord
- Duration formula correctness
- Date continuity

**Complete Timeline Tests (4):**
- 9 * 9 * 9 = 729 total Pratyantardashas
- Nested continuity verification
- Partial first Mahadasha subdivisions
- Full timeline structure validation

**Current Period Detection Tests (4):**
- Basic period lookup
- Boundary detection
- Binary search across 729 periods
- Time containment at all levels

**Upcoming Transitions Tests (5):**
- Chronological order
- Transition hierarchy (Pratyantardasha > Antardasha > Mahadasha)
- Days-until accuracy
- Count limit enforcement
- Planet accuracy

**Witness Prompt Tests (7):**
- Planet references in prompts
- Consciousness level adaptation
- Upcoming transition integration
- Beginner prompt concrete details
- Intermediate prompt questions
- Advanced prompt consciousness language
- All 9 planets have enrichment data

**Wisdom Data Tests (6):**
- Period loading (9 planets)
- Nakshatra loading (27)
- Quality loading (9 planets)
- Nakshatra ruler mapping (27)
- Planetary order (9)
- Vimshottari total = 120

### Performance

- Nakshatra lookup: O(1) via index calculation
- Balance calculation: O(1) arithmetic
- Full timeline generation (729 periods): <5ms
- Current period detection: O(log 729) = ~10 comparisons
- Upcoming transitions: O(N) from current position
- Complete calculation with enrichment: <10ms

---

## API Endpoints

### Calculate Vimshottari Dasha

```
POST /api/v1/engines/vimshottari/calculate
```

### Authentication

Requires JWT token or API key with `consciousness_level >= 2` (Vimshottari is Phase 2 engine).

### Request Format

```json
{
  "birth_data": {
    "date": "1985-06-15",
    "time": "14:30",
    "timezone": "America/Los_Angeles",
    "latitude": 34.0522,
    "longitude": -118.2437
  },
  "precision": "Standard",
  "options": {
    "consciousness_level": 3,
    "upcoming_transition_count": 5
  }
}
```

### Response Format

```json
{
  "engine_id": "vimshottari",
  "result": {
    "birth_nakshatra": {
      "number": 10,
      "name": "Magha",
      "ruling_planet": "Ketu",
      "deity": "Pitris",
      "symbol": "Throne"
    },
    "dasha_balance_years": 4.375,
    "mahadashas": [
      {
        "planet": "Ketu",
        "start_date": "1985-06-15T14:30:00Z",
        "end_date": "1989-11-22T00:00:00Z",
        "duration_years": 4.375,
        "antardashas": [
          {
            "planet": "Ketu",
            "start_date": "1985-06-15T14:30:00Z",
            "end_date": "1985-11-10T00:00:00Z",
            "duration_years": 0.255
          }
        ]
      }
    ],
    "current_period": {
      "mahadasha": {
        "planet": "Jupiter",
        "start": "2020-01-01T00:00:00Z",
        "end": "2036-01-01T00:00:00Z",
        "years": 16.0
      },
      "antardasha": {
        "planet": "Saturn",
        "start": "2024-03-15T00:00:00Z",
        "end": "2026-10-20T00:00:00Z",
        "years": 2.533
      },
      "pratyantardasha": {
        "planet": "Mercury",
        "start": "2026-01-10T00:00:00Z",
        "end": "2026-04-15T00:00:00Z",
        "days": 95.2
      }
    },
    "upcoming_transitions": [
      {
        "transition_type": "Pratyantardasha",
        "from_planet": "Mercury",
        "to_planet": "Ketu",
        "transition_date": "2026-04-15T00:00:00Z",
        "days_until": 74
      }
    ]
  },
  "witness_prompt": "What is this Jupiter-Saturn-Mercury period revealing about your journey?",
  "consciousness_level": 3,
  "metadata": {
    "calculation_time_ms": 8.5,
    "backend": "swiss-ephemeris",
    "precision_achieved": "Standard",
    "cached": false,
    "timestamp": "2026-01-31T12:00:00Z"
  }
}
```

---

## Dependencies

### Internal Dependencies

- **engine-human-design** (for Moon position): Uses `EphemerisCalculator` and `HDPlanet::Moon` from HD engine's ephemeris module to get Moon longitude at birth time.
- **noesis-core**: Provides ConsciousnessEngine trait, EngineInput, EngineOutput, EngineError types.

### External Dependencies

- **chrono**: DateTime handling with UTC timezone support
- **lazy_static**: Static initialization of nakshatra data, planetary qualities, and wisdom
- **serde/serde_json**: JSON deserialization for wisdom data files
- **async-trait**: Async trait for ConsciousnessEngine
- **tokio**: Async runtime (dev dependency)

### Data Files

- **data/vimshottari/dasha_periods.json**: Mahadasha durations and nakshatra-ruler mappings
- **data/vimshottari/nakshatras.json**: 27 nakshatras with deities, symbols, qualities
- **data/vimshottari/vimshottari_periods.json**: Planetary period themes and qualities
- All loaded at compile time via `include_str!`

### Swiss Ephemeris (Indirect)

Vimshottari uses Swiss Ephemeris indirectly through the HD engine's `EphemerisCalculator` to get the Moon's ecliptic longitude at birth time. This is the single astronomical input required for the entire Vimshottari calculation.

---

## Witness Integration

### Consciousness Levels 0-6

**Level 0-2 (Beginner): Concrete Timing and Life Areas**

Focus: Practical information about current planetary period and upcoming changes.

Format: Informational with specific dates and themes.

Example: "You are currently in Venus's Mahadasha, Mercury's Antardasha, and Jupiter's Pratyantardasha (until May 31, 2026). This Jupiter period emphasizes Growth, Wisdom, Teaching, Expansion. Life areas to focus on: Higher education, Long-distance travel, Teaching and mentorship. In 120 days, you'll transition to Saturn's period, bringing a shift toward Discipline."

**Level 3-4 (Intermediate): Opportunities and Challenges Awareness**

Focus: Inquiry-based exploration of how planetary themes manifest in life.

Format: Questions that invite self-reflection on current period dynamics.

Example: "What is this Venus-Mercury-Jupiter period revealing about your journey? Notice how Venus (Mahadasha) provides the backdrop, Mercury (Antardasha) colors the experience, and Jupiter (Pratyantardasha) brings immediate focus to Growth, Wisdom. Opportunities present: Pursue higher learning, Expand horizons. Challenges to navigate: Over-optimism, Idealistic expectations. How are you meeting these themes in your life right now?"

**Level 5-6 (Advanced): Witnessing Karmic Patterns**

Focus: Deep consciousness awareness, karmic pattern recognition, preparing for transitions.

Format: Invitations to witness the impersonal forces moving through personal experience.

Example: "You are in the conscious field of Jupiter's influence, nested within Mercury's container, illuminated by Venus's immediate presence. What karmic patterns are ripening? What is seeking release through Over-optimism and Idealistic expectations? Beyond the themes of expansion and learning, what wants to be witnessed in pure awareness? How does the approaching transition to Saturn (in 120 days) invite you to prepare your consciousness?"

### Prompt Structure

Every Vimshottari witness prompt includes:
1. All 3 current planets named (Mahadasha, Antardasha, Pratyantardasha)
2. Thematic content from planetary qualities
3. At least one question mark (inquiry format)
4. Consciousness-level-appropriate language
5. Optional: Upcoming transition awareness (if transitions provided)

### Integration with Other Engines

Vimshottari prompts provide temporal context that enriches other engine prompts:
- **HD**: "Your Sacral authority says respond. During Saturn's period, responses may come slower. Trust the timing."
- **Gene Keys**: "Gene Key 17's gift of Far-Sightedness meets Jupiter's expansion. What wider vision is emerging?"

---

## Planetary Quality Enrichment

### PeriodEnrichment Structure

The engine enriches the current 3-level period with combined qualities:

```rust
pub struct PeriodEnrichment {
    pub mahadasha_themes: Vec<String>,       // Overarching life themes
    pub antardasha_themes: Vec<String>,      // Coloring themes
    pub pratyantardasha_themes: Vec<String>, // Immediate focus themes
    pub combined_description: String,         // Narrative combining all 3 levels
    pub life_areas: Vec<String>,             // Areas of life most active
    pub opportunities: Vec<String>,          // What to leverage
    pub challenges: Vec<String>,             // What to navigate
}
```

### Planetary Qualities (All 9 Planets)

Each planet carries distinct consciousness themes:

**Sun (6 years):** Self-expression, authority, vitality, recognition. Career advancement, leadership roles, public visibility. Challenges: ego inflation, conflicts with authority.

**Moon (10 years):** Emotions, nurturing, home/family, intuition. Domestic life, emotional security, creative expression. Challenges: emotional overwhelm, mood swings, dependency.

**Mars (7 years):** Action, courage, conflict, energy. Physical activity, competition, property. Challenges: anger, impulsiveness, accidents.

**Rahu (18 years):** Ambition, innovation, foreign connections, obsession. Unconventional paths, technology, material success. Challenges: obsessive desire, deception, ethical compromises.

**Jupiter (16 years):** Growth, wisdom, teaching, expansion. Higher education, travel, philosophy, children. Challenges: over-optimism, excessive indulgence, lack of boundaries.

**Saturn (19 years):** Discipline, structure, karma, maturity. Career responsibility, long-term goals, service. Challenges: depression, loneliness, physical limitations.

**Mercury (17 years):** Communication, learning, business, intellect. Education, writing, commerce, technology. Challenges: mental restlessness, overthinking, superficiality.

**Ketu (7 years):** Spirituality, detachment, past-life themes, liberation. Spiritual practice, solitude, psychic abilities. Challenges: isolation, confusion, loss of motivation.

**Venus (20 years):** Love, beauty, luxury, relationships. Romance, marriage, arts, social life. Challenges: overindulgence, vanity, materialism.

---

## Data Model Reference

### VedicPlanet

```rust
pub enum VedicPlanet {
    Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury, Ketu, Venus
}

impl VedicPlanet {
    pub fn period_years(&self) -> u8;   // 6, 10, 7, 18, 16, 19, 17, 7, 20
    pub fn next_planet(&self) -> Self;  // Vimshottari cycle order
    pub fn as_str(&self) -> &'static str;
    pub fn from_str(s: &str) -> Option<Self>;
}
```

### Nakshatra

```rust
pub struct Nakshatra {
    pub number: u8,              // 1-27
    pub name: String,            // e.g., "Ashwini"
    pub ruling_planet: VedicPlanet,
    pub start_degree: f64,       // e.g., 0.0
    pub end_degree: f64,         // e.g., 13.333333
    pub deity: String,           // Presiding deity
    pub symbol: String,          // Visual symbol
    pub qualities: Vec<String>,  // Characteristic qualities
    pub description: String,     // Brief description
}
```

### Mahadasha

```rust
pub struct Mahadasha {
    pub planet: VedicPlanet,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_years: f64,
    pub antardashas: Vec<Antardasha>,
    pub qualities: PlanetaryQualities,
}
```

### CurrentPeriod

```rust
pub struct CurrentPeriod {
    pub mahadasha: CurrentMahadasha,
    pub antardasha: CurrentAntardasha,
    pub pratyantardasha: CurrentPratyantardasha,
    pub current_time: DateTime<Utc>,
}
```

### UpcomingTransition

```rust
pub struct UpcomingTransition {
    pub transition_type: TransitionLevel,  // Mahadasha | Antardasha | Pratyantardasha
    pub from_planet: VedicPlanet,
    pub to_planet: VedicPlanet,
    pub transition_date: DateTime<Utc>,
    pub days_until: i64,
}
```

---

## Common Mistakes to Avoid

### CRITICAL Errors

1. **Using wrong nakshatra span**: Each nakshatra is exactly 13.333333 degrees (360/27), not 13.0 or 13.5
2. **Wrong planet order**: Must follow Ketu -> Venus -> Sun -> Moon -> Mars -> Rahu -> Jupiter -> Saturn -> Mercury
3. **Forgetting balance**: First Mahadasha is almost always partial
4. **Integer division in durations**: Use f64 throughout, never truncate to integers
5. **Missing Pratyantardasha level**: All 3 levels must be calculated for proper current period detection

### Common Pitfalls

- Antardasha sum not equaling Mahadasha duration (check formula: M*A/120)
- Date gaps between consecutive periods (verify continuity)
- Binary search not handling boundary conditions (period start/end overlap)
- Not normalizing Moon longitude to 0-360 before nakshatra lookup
- Using calendar days instead of 365.25 for year-to-day conversion

---

## References

### Source Material

- B.V. Raman: "Hindu Predictive Astrology" (Vimshottari Dasha theory)
- K.N. Rao: "Planets and Children" (practical dasha interpretation)
- Parashara's Brihat Hora Shastra (original source text)
- Swiss Ephemeris: Astronomical data for Moon longitude

### Implementation Notes

This documentation reflects the Selemene Engine Vimshottari implementation as of 2026-01. For source code, see `crates/engine-vimshottari/`.

---

---

## Production Metrics (Wave 1)

### Performance

- Full 120-year timeline generation (729 periods): sub-millisecond
- Nakshatra lookup: O(1) via index calculation
- Balance calculation: O(1) arithmetic
- Current period detection: O(log 729) binary search, approximately 10 comparisons
- Upcoming transitions (next 5): O(N) forward iteration, <0.1ms
- Complete calculation with enrichment and witness prompt: <1ms
- Cache hits (L1): <10ms
- Target was <200ms: achieved 200x faster

### Test Coverage

- 42+ tests across all modules
- 4 nakshatra determination tests
- 3 balance calculation tests
- 4 mahadasha generation tests
- 5 antardasha subdivision tests
- 4 pratyantardasha subdivision tests
- 4 complete timeline tests (729 period validation)
- 4 current period detection tests (binary search)
- 5 upcoming transition tests
- 7 witness prompt tests (consciousness level adaptation)
- 6 wisdom data tests (planets, nakshatras, qualities)

### Bug Fixes During Wave 1

- Fixed 3 pre-existing bugs in dasha calculation logic
- Corrected antardasha sum validation (floating point tolerance)
- Fixed boundary condition in binary search for period detection
- Resolved date gap between consecutive period end/start dates

### Known Limitations

- Moon longitude obtained indirectly via HD engine's EphemerisCalculator
- Sidereal ayanamsa uses Lahiri (most common) - not configurable yet
- Yogini Dasha and other alternative dasha systems not implemented
- Transit overlay (current sky vs natal timeline) not yet available
- Planetary strength (Shadbala) not calculated

### API Usage Example

```bash
curl -X POST http://localhost:8080/api/v1/engines/vimshottari/calculate \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "birth_data": {
      "date": "1985-06-15",
      "time": "14:30",
      "latitude": 34.0522,
      "longitude": -118.2437
    },
    "precision": "Standard",
    "options": {
      "consciousness_level": 3,
      "upcoming_transition_count": 5
    }
  }'
```

---

**Last Updated**: 2026-01-31
**Engine Version**: 0.1.0
**Total Source**: 2,440 lines across 6 modules
**Test Count**: 42+ tests across calculator, wisdom, and witness modules
**Validation Status**: All tests passing
**Performance**: Sub-millisecond for complete 120-year timeline with 729 periods
**Wave 1 Status**: Complete - Production Ready
