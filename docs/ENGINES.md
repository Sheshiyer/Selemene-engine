<p align="center">
  <img src="assets/images/engines/2C-stained-glass-logo-nanobananapro-v2.png" alt="Selemene Engine" width="400">
</p>

<h1 align="center">The 16 Engines</h1>

<p align="center">
  <em>Sixteen implemented engines acting as mirrors for inquiry and synthesis.</em><br>
  <sub>16 implemented engines across Rust and TypeScript surfaces</sub>
</p>

<br>

---

<br>

# Rust Engines (11)

High-performance native calculation engines built in Rust with sub-millisecond performance.

For a body-paced narrative companion to these timing and pattern systems, see [Somatic Canticles](https://1319.tryambakam.space).

<br>

## 1. Panchanga Engine

**Vedic calendar system** — Calculates the five limbs of time: Tithi, Nakshatra, Yoga, Karana, Vara.

**Input:** `date, lat/lng, timezone`  
**Endpoint:** `POST /api/v1/engines/panchanga/calculate`

Foundation for all Vedic timing calculations.

<br>

## 2. Human Design Engine

**Complete bodygraph** — Type, Strategy, Authority, 9 Centers, 26 Gates, Profile, Definition.

**Input:** `date, time, lat/lng, timezone`  
**Endpoint:** `POST /api/v1/engines/human-design/calculate`

Synthesizes I Ching, Kabbalah, Chakras, and Astrology.

<br>

## 3. Gene Keys Engine

**64 Keys** — Shadow → Gift → Siddhi progression across 4 sequences (Activation, Venus, Pearl, Life's Work).

**Input:** `date, time, lat/lng, timezone`  
**Endpoint:** `POST /api/v1/engines/gene-keys/calculate`

Maps consciousness evolution through 64 genetic pathways.

<br>

## 4. Vimshottari Dasha Engine

**120-year timeline** — Nested planetary periods (Mahadasha → Antardasha → 729 total periods).

**Input:** `date, time, lat/lng, timezone`  
**Endpoint:** `POST /api/v1/engines/vimshottari/calculate`

Foundation for Vedic predictive timing.

<br>

## 5. Numerology Engine

**Pythagorean & Chaldean systems** — Life Path, Expression, Soul Urge, Birthday, Personality, Master Numbers.

**Input:** `date, name`  
**Endpoint:** `POST /api/v1/engines/numerology/calculate`

### Key Calculations
- Life Path Number (birth date reduction)
- Expression Number (full name)
- Soul Urge (vowels)
- Personality (consonants)
- Birthday Number
- Master Numbers (11, 22, 33)

<br>

## 6. Biorhythm Engine

**3 biological cycles** — Physical (23-day), Emotional (28-day), Intellectual (33-day).

**Input:** `date`  
**Endpoint:** `POST /api/v1/engines/biorhythm/calculate`

### Calculations
- Current phase for all 3 cycles
- Critical day detection (zero crossings)
- Peak and valley predictions
- Cycle intersection points
- Energy trend forecasting

<br>

## 7. Vedic Clock Engine

**TCM Organ Clock + Ayurvedic Doshas** — Maps 24-hour cycle to organ energy peaks and dosha dominance.

**Input:** `current_time` plus either `options.timezone_offset` or `birth_data.timezone`  
**Endpoint:** `POST /api/v1/engines/vedic-clock/calculate`

### Returns
- Current organ energy peak (12 meridians)
- Dominant dosha (Vata/Pitta/Kapha)
- Optimal activity recommendations
- Energy quality of current hour
- Resolved timezone metadata (`offset_minutes`, `source`, `local_hour`)

### Timezone Semantics
- Priority order: `options.timezone_offset` -> `birth_data.timezone` -> UTC
- `birth_data.timezone` accepts supported IANA values such as `Asia/Kolkata` and explicit offsets such as `+05:45`
- The result payload includes the resolved timezone basis so API consumers can verify which local clock the organ-hour calculation used

<br>

## 8. Biofield Engine

**Vedic Birth-Data Biofield Analysis** — Real planetary position-driven chakra readings using Navagraha dignity, aspects, and temporal transit modulation.

**Input:** `date, time, lat/lng`
**Endpoint:** `POST /api/v1/engines/biofield/calculate`

### Calculations
- 7 chakra readings via planet-chakra correspondences (Vedic Jyotish)
- Fractal dimension, entropy, coherence, symmetry metrics
- Planetary dignity (exaltation → debilitation) + aspect scoring
- Temporal modulation: Moon transit, lunar phase, day-of-week Vara
- Falls back to mock data when no birth_data provided

<br>

## 9. Face Reading Engine

**Physiognomy analysis** — Extracts facial features and maps to personality traits, health indicators, and life patterns.

**Input:** `image_data`  
**Endpoint:** `POST /api/v1/engines/face-reading/calculate`

Combines Chinese Mian Xiang with Western physiognomy traditions.

<br>

## 10. Nadabrahman Engine

**Sound consciousness** — Analyzes vocal patterns, frequency signatures, and maps to Vedic sound cosmology (Nada Brahma).

**Input:** `audio_data`  
**Endpoint:** `POST /api/v1/engines/nadabrahman/calculate`

Maps voice characteristics to raaga resonance and chakra activations.

<br>

## 11. Transits Engine

**Planetary transits & Sade Sati** — Current sidereal planetary positions, natal-to-transit aspects, and Saturn's 7.5-year Sade Sati cycle detection.

**Input:** `date, time, lat/lng, timezone`
**Endpoint:** `POST /api/v1/engines/transits/calculate`

### Returns
- Current sidereal positions for 12 tracked bodies (Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Rahu, Ketu)
- Transit-to-natal aspects (conjunction, trine, square, sextile, opposition)
- Sade Sati phase detection (rising, peak, setting)
- Retrograde status for all planets
- Witness prompts calibrated to active transits

<br>

---

<br>

# TypeScript Engines (5)

Esoteric calculation engines built in TypeScript, bridged to main Rust API.

<br>

## 12. Tarot Engine

**78-card system** — Birth cards, yearly cycles, archetypal progression through Major Arcana.

**Input:** `date, question`  
**Endpoint:** `POST /api/v1/engines/tarot/calculate`

### Calculations
- Birth card(s) from numerology
- Current year card
- Life path card sequence
- Archetypal themes
- Symbolic inquiry prompts

<br>

## 13. I Ching Engine

**64 hexagrams** — Oracle casting via yarrow stalk, coin, or digital methods. Interprets changing lines and transformations.

**Input:** `question, casting_method`  
**Endpoint:** `POST /api/v1/engines/i-ching/calculate`

Traditional Book of Changes divination system.

<br>

## 14. Enneagram Engine

**9 personality types** — Core type, wing, tritype, integration/disintegration arrows, instinctual variants.

**Input:** `date, name`  
**Endpoint:** `POST /api/v1/engines/enneagram/calculate`

### Returns
- Core type (1-9)
- Wing tendency
- Tritype combination
- Integration/disintegration paths
- Instinctual stacking (sp/sx/so)
- Centers of intelligence

<br>

## 15. Sacred Geometry Engine

**Platonic solids & geometric patterns** — Generates sacred geometry constructions, calculates proportions (phi, pi, √2), and maps to energetic resonances.

**Input:** `parameters (shape, size, proportions)`  
**Endpoint:** `POST /api/v1/engines/sacred-geometry/calculate`

Includes Flower of Life, Metatron's Cube, Sri Yantra, Golden Spiral.

<br>

## 16. Sigil Forge Engine

**Intent manifestation** — Transforms text intentions into visual sigils using letter reduction, geometric encoding, and symbolic transformation methods.

**Input:** `intent_text, method (chaos/planetary/angelic)`  
**Endpoint:** `POST /api/v1/engines/sigil-forge/calculate`

Combines Austin Osman Spare's method with planetary and angelic signatures.

<br>

---

<br>

# Additional Engines (In Development)

## Astrocartography Engine

**Planetary line mapping** — Projects birth chart onto world map, identifies power spots and relocation themes.

**Status:** Partially implemented  
**Input:** `date, time, birth_location`

### Calculations
- Planetary line crossings (ASC/MC/DSC/IC)
- Power spot coordinates
- Relocation chart analysis
- Travel timing recommendations

<br>

## HRV (Heart Rate Variability) Engine

**Autonomic nervous system mapping** — Predicts sympathetic/parasympathetic balance patterns and recovery windows from birth data.

**Status:** Experimental  
**Input:** `date, optional_hrv_data`

### Returns
- Baseline ANS tendency
- Optimal recovery times
- Stress resilience patterns
- Circadian HRV rhythm

<br>

## Nakshatra Engine

**27 lunar mansions** — Deep dive into Vedic lunar mansion system. Each nakshatra's deity, qualities, and karmic themes.

**Status:** Integrated with Panchanga  
**Input:** `date, lat/lng`

### Returns
- Birth nakshatra
- Pada (quarter within nakshatra)
- Ruling deity and planet
- Karmic themes
- Compatible nakshatras

<br>

## Biofield-Raaga Engine

**Sound frequency + energy field integration** — Maps birth patterns to Hindustani raaga resonance and biofield layer activations.

**Status:** In development  
**Input:** `date, time`

### Calculations
- Dominant raaga resonance
- Biofield layer activations (7 layers)
- Frequency signatures
- Time-of-day raaga cycles
- Sound healing recommendations

<br>

---

<br>

## Common Response Format

All engines return this unified structure:

```json
{
  "engine": "engine_name",
  "result": { /* engine-specific data */ },
  "witness_prompt": "Calibrated question for self-inquiry",
  "consciousness_level": 0,
  "metadata": {
    "calculation_time_ms": 0.042,
    "backend": "native",
    "version": "3.0.0"
  }
}
```

<br>

---

<br>

<p align="center">
  <a href="../README.md">← Back to Main README</a>
  &nbsp;&nbsp;|&nbsp;&nbsp;
  <a href="api/README.md">API Documentation →</a>
</p>

<br>
