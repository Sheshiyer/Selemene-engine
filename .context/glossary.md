# Glossary - Tryambakam Noesis

> **Terminology reference** - Technical and mystical terms for the dual-paradigm system

**Purpose**: Define terms used across codebase and documentation  
**Audience**: Developers, AI assistants, researchers  
**Organization**: Technical terms first, then mystical/archetypal terms

---

## 🔧 Technical Terms

### Backend & Infrastructure

**Axum**  
Rust web framework built on Tokio, used for HTTP API layer. Provides routing, middleware, and async request handling.

**Backend Routing Strategy**  
Algorithm that selects between Native engines (VSOP87/ELP-2000) and Swiss Ephemeris based on calculation requirements, precision level, and validation mode.

**Cache Cascade**  
Three-tier lookup sequence: L1 (in-memory LRU) → L2 (Redis distributed) → L3 (disk precomputed). Requests check each tier in order, returning on first hit.

**Cache Hit Rate**  
Percentage of requests served from cache vs. computed fresh. Target: 85%+ for birth data calculations.

**Cache Key**  
SHA-256 hash of normalized input (date, time, coordinates, precision, engine ID). Deterministic: same input always generates same key.

**Cargo Workspace**  
Monorepo structure with multiple related crates sharing `Cargo.toml` dependencies. Enables modular development with shared types.

**ConsciousnessEngine Trait**  
Core trait implemented by all 13 engines. Defines `calculate()`, `validate()`, and `cache_key()` methods. Enables uniform orchestration and API routing.

**EngineError Enum**  
Structured error type for all engine operations. Variants include `CalculationError`, `ValidationError`, `CacheError`, `AuthenticationError`. Never use `anyhow` in public APIs.

**EngineInput Struct**  
Standard input container for all engine calculations. Contains `birth_data`, `current_time`, `location`, `precision`, and `options` HashMap.

**EngineOutput Struct**  
Standard output container from all engines. Contains `engine_id`, `result` (JSON), `witness_prompt`, `consciousness_level`, and `metadata`.

**Hybrid Backend**  
System that routes calculations between Native engines (Rust implementations) and Swiss Ephemeris (C library) based on strategy.

**L1 Cache (In-Memory)**  
First cache tier using `dashmap::DashMap` for thread-safe LRU caching. ~256MB capacity, <1ms access time.

**L2 Cache (Redis)**  
Second cache tier using Redis for distributed caching across instances. ~1GB capacity, <10ms access time.

**L3 Cache (Disk)**  
Third cache tier with precomputed results for common queries (e.g., daily Panchanga for major cities). Slowest but largest capacity.

**Orchestrator Pattern**  
Architectural pattern where all calculations flow through `CalculationOrchestrator`. Coordinates backend selection, caching, validation, and metrics.

**Precision Level**  
Enum with variants: `Standard` (fast, real-time), `High` (increased accuracy), `Extreme` (research-grade). Affects calculation method and cache key.

**Tokio Runtime**  
Async executor for Rust. All I/O-bound operations use `async fn` with `.await`. Powers the Axum server.

**TypeScript Bridge**  
HTTP adapter pattern where `BridgeEngine` implements `ConsciousnessEngine` by proxying to Bun-powered TypeScript engines (Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge).

---

### Time Systems

**Ghati**  
Vedic time unit. 1 day = 60 ghatis, 1 ghati = 24 minutes. Used in Panchanga and auspicious timing calculations.

**Ghati Hybrid System**  
Implementation approach: Fixed 24-minute intervals as base + solar time adjustments based on longitude. Balances simplicity with astronomical accuracy.

**Julian Day (JD)**  
Continuous count of days since noon UTC on January 1, 4713 BCE. Used internally for astronomical calculations.

**Panchanga**  
Vedic calendar system with 5 limbs (pancha = five, anga = limb): Tithi, Vara (weekday), Nakshatra, Yoga, Karana. Determines auspicious timing.

**Tithi**  
Lunar day. One-thirtieth of the lunar month (~23.6 hours). Used for religious observances and timing.

**Nakshatra**  
Lunar mansion. One of 27 (or 28) divisions of the zodiac (~13°20' each). Associated with specific deities, qualities, and activities.

**Yoga**  
One of 27 combinations of Sun and Moon positions. Calculated from sum of longitudes. Associated with specific effects.

**Karana**  
Half of a Tithi. 11 total Karanas (7 movable, 4 fixed). Used for detailed timing precision.

**Vara**  
Sanskrit for weekday. Seven varas ruled by seven classical planets (Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn).

---

### Astronomical Terms

**88-Day Solar Arc**  
In Human Design, the Design calculation is 88 days (approximately 88° of solar arc) before birth. Critical for accurate chart generation.

**VSOP87**  
Variations Séculaires des Orbites Planétaires (Secular Variations of Planetary Orbits). High-precision planetary theory used in native solar engine.

**ELP-2000**  
Ephemeride Lunaire Parisienne. Analytical lunar theory with 2000+ terms. Used in native lunar engine for high-accuracy Moon positions.

**Swiss Ephemeris**  
High-precision astronomical library. Used as fallback when native engines don't support required calculations (e.g., outer planets, asteroids).

**Sidereal Zodiac**  
Zodiac aligned with fixed stars (as opposed to tropical zodiac aligned with seasons). Used in Vedic astrology and Panchanga.

**Tropical Zodiac**  
Zodiac aligned with seasonal equinoxes/solstices. Used in Western astrology and Human Design.

**Ayanamsa**  
Difference between tropical and sidereal zodiacs (~24° currently). Critical for converting between systems.

---

### Calculation Terms

**Dasha**  
Planetary period system in Vedic astrology. Vimshottari Dasha is 120-year cycle divided among 9 planets.

**Mahadasha**  
Major planetary period (years-long). Example: Jupiter Mahadasha lasts 16 years.

**Antardasha**  
Sub-period within Mahadasha (months-long). Example: Jupiter-Saturn Antardasha.

**Pratyantardasha**  
Sub-sub-period (days/weeks-long). Third level of dasha subdivision.

**Ephemeris Data**  
Astronomical tables showing planetary positions at specific times. Required for accurate chart calculations.

---

## 🔮 Mystical & Archetypal Terms

### Self-Consciousness Framework

**Consciousness (Universal)**  
The field itself. Awareness as ground of being. Not personal, not individual. Cannot be developed (it already IS).

**Self-Consciousness (Individual)**  
The witness. The observer. "I am aware that I am aware." The capacity to step back and watch yourself. CAN be developed.

**Witness / Observer**  
The part of awareness that watches thoughts, emotions, and patterns without identifying with them. Core capacity trained by Noesis.

**Witness Prompt**  
Self-inquiry question generated from calculation results. Trains observer capacity. Example: "Notice: When have you felt this energy before?"

**Consciousness Level (0-5)**  
Scale of self-consciousness development:
- **0 - Dormant**: Fully identified with patterns
- **1 - Glimpsing**: Occasional witnessing after the fact
- **2 - Practicing**: Regular witnessing, some conscious choice
- **3 - Integrated**: Stable observer, active authorship
- **4 - Embodied**: Seamless witness-response integration
- **5 - Mature**: Spontaneous wisdom, no internal conflict

**Authorship / Self-Authorship**  
Conscious creation of responses vs. unconscious reactivity. The capacity to choose meaning and action rather than being driven by patterns.

**Decision Mirror**  
Core metaphor: Engines show patterns, they don't prescribe actions. Reflect unconscious patterns so witness can see them clearly.

---

### Human Design Terms

**Type**  
One of 5 energetic blueprints: Manifestor, Generator, Manifesting Generator, Projector, Reflector. Determines strategy for decision-making.

**Strategy**  
How each type is designed to make decisions aligned with their energy. Example: Generators "respond" to life rather than initiate.

**Authority**  
Where wisdom lives in YOUR body. Seven types: Emotional, Sacral, Splenic, Ego, Self-Projected, Environmental, Lunar. Not all decisions are mental.

**Gates**  
64 specific themes based on I-Ching hexagrams. Numbered 1-64 sequentially around the wheel (NOT King Wen sequence).

**Lines**  
Six sub-variations within each gate. Example: Gate 17, Line 3 = different expression than Gate 17, Line 5.

**Channels**  
Connections between two gates activating specific life themes. 36 total channels. Require both gates defined.

**Centers**  
Nine energy hubs (like chakras but different): Head, Ajna, Throat, G, Heart, Spleen, Sacral, Solar Plexus, Root. Can be defined (consistent) or undefined (influenced).

**Profile**  
Life theme based on Personality and Design Sun/Earth line numbers. Example: 6/2 Profile = "Role Model/Hermit" archetype.

**Personality**  
Conscious aspects of your design. What you're aware of. Calculated at birth time.

**Design**  
Unconscious aspects. What others see in you before you do. Calculated 88 days before birth.

**Defined**  
Centers/channels that are consistent in your energy. Your "fixed" nature.

**Undefined**  
Centers that are open to influence. Where you absorb and amplify others' energy.

---

### Gene Keys Terms

**Shadow**  
Unconscious, reactive frequency of a Gene Key. The pattern when you're asleep to it. Not "bad" - just unconscious.

**Gift**  
Present, creative frequency. The pattern when witnessed consciously. Natural expression of the Key.

**Siddhi**  
Transcendent, enlightened frequency. Beyond personal self. Rare glimpses of the Key's highest potential.

**Transformation Pathway**  
Shadow → Gift → Siddhi progression. Not linear - spiral movement as consciousness develops.

**Codon Ring**  
Genetic/biological correlation for each Gene Key. Connects I-Ching to DNA structure.

**Amino Acid**  
Molecular correlation. Each Gene Key maps to one of 20 amino acids (some have multiple Keys).

---

### Tarot Terms

**Major Arcana**  
22 trump cards (0-21). Archetypal journey from Fool to World. Primary consciousness themes.

**Minor Arcana**  
56 cards in 4 suits (Wands, Cups, Swords, Pentacles). Daily life situations and energies.

**Upright**  
Card in standard position. Generally positive or integrated expression of archetype.

**Reversed**  
Card inverted. Often indicates blocked, shadow, or internalized aspect of archetype.

**Archetypal Mirror**  
Tarot as reflection of unconscious patterns, not fortune-telling. Shows what IS, not what WILL BE.

---

### I-Ching Terms

**Hexagram**  
Six-line symbol representing specific situation or energy. 64 total hexagrams.

**Trigram**  
Three-line component. Hexagrams combine upper and lower trigrams. 8 trigrams total.

**Judgment**  
Traditional interpretation/wisdom for the hexagram as a whole.

**Image**  
Symbolic representation offering guidance on how to work with the energy.

**Changing Lines**  
Lines transitioning from yin to yang or vice versa. Indicate dynamic transformation.

**King Wen Sequence**  
Traditional ordering of I-Ching hexagrams. NOT used in Human Design gate sequencing.

---

### Enneagram Terms

**Type**  
One of 9 core personality patterns. Defined by core fear, core desire, and defense mechanism.

**Wing**  
Adjacent type influencing your core type. Example: Type 5 with 4 wing (5w4) or 6 wing (5w6).

**Integration**  
Direction of growth. Line pointing to type you move toward when healthy.

**Disintegration**  
Direction of stress. Line pointing to type you move toward when unhealthy.

**Center**  
Three centers: Body/Instinctive (8,9,1), Heart/Feeling (2,3,4), Head/Thinking (5,6,7).

---

### VedicClock-TCM Terms

**TCM Organ Clock**  
24-hour cycle where each 2-hour window corresponds to peak energy in a specific organ/meridian.

**Five Elements**  
Wood, Fire, Earth, Metal, Water. Each associated with organs, seasons, emotions, and qualities.

**Dosha**  
Ayurvedic constitutional type: Vata (air/ether), Pitta (fire/water), Kapha (earth/water).

**Brahma Muhurta**  
96 minutes before sunrise. Considered most auspicious time for spiritual practice in Vedic tradition.

**Hora**  
Planetary hour. One of 24 hours ruled by a specific planet in Vedic time reckoning.

---

### Sacred Geometry Terms

**Golden Ratio (φ)**  
~1.618... Proportion appearing throughout nature. Used in sacred geometric constructions.

**Flower of Life**  
Geometric pattern of overlapping circles. Foundation symbol in sacred geometry.

**Metatron's Cube**  
Contains all 5 Platonic solids. Derived from Fruit of Life pattern.

**Platonic Solids**  
Five perfect 3D shapes: Tetrahedron (fire), Hexahedron/Cube (earth), Octahedron (air), Dodecahedron (ether), Icosahedron (water).

**Vesica Piscis**  
Almond shape formed by two overlapping circles. Symbol of creation and intersection.

---

### Biofield Terms

**Biofield**  
Energetic field surrounding the body. Measured through various techniques (Kirlian, GDV, PIP).

**Fractal Dimension**  
Measure of self-similarity across scales. Higher = more complex/coherent field.

**Entropy Form Coefficient**  
Measure of spatial order vs. chaos in biofield patterns.

**Coherence**  
Degree of organized, harmonious patterns in the field. High coherence = balanced state.

---

### Face Reading Terms

**Samudrika Shastra**  
Vedic science of physiognomy. Reading character, destiny, and constitution from facial features.

**TCM Face Zones**  
Traditional Chinese Medicine maps specific face areas to internal organs and systems.

**Facial Landmarks**  
68-point coordinate system for analyzing facial geometry (MediaPipe standard).

---

## 🧬 Wisdom Data Terms

**Archetypal Data**  
JSON files containing symbolic/mystical wisdom (gates, hexagrams, cards, etc.). Not "content" - crystallized millennia of knowledge.

**Wisdom Corpus**  
Complete collection of 35 JSON files from WitnessOS extraction. Located in `data/wisdom-docs/`.

**Crystallized Knowledge**  
Information that has been validated, structured, and solidified into permanent reference form. Opposite of "fluid knowledge" (Slack threads, tribal knowledge).

**Read-Only Reference**  
Wisdom data files are immutable. Never modified at runtime. User customizations stored separately.

**Schema Validation**  
Using Serde structs to ensure JSON matches expected structure at compile-time.

---

## 🔄 Workflow Terms

**Multi-Engine Workflow**  
Calculation combining multiple engines to reveal core patterns through cross-validation. Example: birth-blueprint uses Numerology + HD + Vimshottari.

**Synthesis Pattern**  
How multiple engine results are combined to reveal deeper insights. Example: Life Path (Numerology) + Type (HD) + Current Dasha (Vimshottari) = present moment guidance.

**Cross-Validation**  
When 3+ engines independently point to same pattern. Increases confidence in pattern recognition.

---

## 🎭 Project-Specific Terms

**Selemene Engine**  
Current state: Single-crate Rust project providing Panchanga calculations. Named after Dota character (Moon goddess).

**Tryambakam Noesis**  
Target state: 13-engine consciousness computing platform. Tryambakam = "three-eyed one" (Shiva). Noesis = direct knowing.

**WitnessOS**  
Source project from which wisdom data was extracted. Broader consciousness operating system concept.

**Transformation / Evolution**  
Current phase: Migrating from Selemene (single engine) to Tryambakam (13 engines). Documented in `.claude/crystalline-giggling-trinket.md`.

---

## 📚 Documentation Terms

**.context Method**  
"Documentation as Code as Context" system. Structured markdown files that give AI tools deep project understanding.

**Substrate**  
Foundation documentation layer. Entry point file (`substrate.md`) provides project overview.

**AI Rules**  
Hard constraints that MUST be followed (`.context/ai-rules.md`). Overrides convenience.

**Anti-Patterns**  
Forbidden approaches documented in `.context/anti-patterns.md`. What NOT to do.

**Crystalline-Giggling-Trinket**  
Active transformation plan document. Records phase-by-phase migration strategy.

---

## 🔗 Cross-Reference Notes

Many terms have **dual meanings** - technical AND symbolic:

- **Engine**: Both "calculation engine" AND "consciousness mirror"
- **Cache**: Both "performance optimization" AND "preserving calculated patterns"
- **Validation**: Both "data integrity" AND "cross-checking truth claims"
- **Level**: Both "API access tier" AND "consciousness development stage"

**When in doubt**: Check context. Technical docs use technical meaning. Consciousness docs use symbolic meaning.

---

**Last Updated**: 2026-01-30  
**Maintenance**: Add terms as new engines/features are implemented  
**Style**: Technical terms (plain), Mystical terms (with rich context)
