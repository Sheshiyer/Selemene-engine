# Transits — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

Current planetary positions vs. the natal chart: where the 12 grahas sit *now*, the angular aspects they make to their natal placements (conjunction/sextile/square/trine/opposition with orbs), Sade Sati status, and an overall period quality. The orbital constellation engine.

## 1. Identity
| | |
|---|---|
| `engine_id` | `transits` (verified — `crates/engine-transits/src/engine.rs:36`, registry `crates/noesis-orchestrator/src/workflow/registry.rs:52`) |
| Domain crate | `crates/engine-transits/src/models.rs` (`TransitAnalysisResult`, L321); engine + serializer in `src/engine.rs` (`serialize_result`, L112) |
| Runtime source | **`engine-transits` via the orchestrator** — `EngineOutput.result` = serialized `TransitAnalysisResult` (`engine.rs:219`, `engine.rs:163`). Driven from `crates/noesis-api/src/handlers/witness.rs:106` (`run_engine(&orch, "transits", …)`). A second native facade `crates/noesis-vedic-api/src/transits/api.rs` (`TransitApiResponse`, L61) exists with a **different** wire shape (see §7) |
| Renderer | `apps/noesis-web/src/components/engines/Transits.tsx` |
| Fixture (runtime `result`) | **`tests/fixtures/expected_outputs/transits/*.json`** (e.g. `user_mumbai_1992.json`) — full `EngineOutput`, authoritative |
| Fixtures (raw ephemeris) | `crates/noesis-vedic-api/tests/fixtures/captures/planets_bangalore_1991-08-13.json`, `…/western_houses_bangalore_1991-08-13.json` — vendor-capture position/house format (input side, **not** the engine `result` shape) |
| OpenAPI stub | `crates/noesis-core/src/types.rs:302` (`TransitsResultSchema` — `dominant_transit`/`intensity`/`next_peak_date`; **none of these fields exist at runtime** — see §7) |

## 2. Output schema

**Runtime JSON (authoritative — `expected_outputs/transits/user_mumbai_1992.json`, produced by `serialize_result` `engine.rs:112`):**
```jsonc
{
  "natal_positions": [        // 12 entries, fixed graha order (see §3)
    { "planet": "Sun", "longitude": 107.876, "sign": "Cancer",
      "degree_in_sign": 17.88, "is_retrograde": false }
    // … Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Rahu, Ketu
  ],
  "transit_positions": [      // 12 entries — note: carries `speed`, natal does NOT
    { "planet": "Sun", "longitude": 295.234, "sign": "Capricorn",
      "degree_in_sign": 25.23, "speed": 1.019, "is_retrograde": false }
  ],
  "aspects": [                // transit-planet → natal-planet hits, variable length (0..N)
    { "transiting_planet": "Pluto", "natal_planet": "Saturn",
      "aspect_type": "Conjunction", "orb": 4.78,
      "is_applying": true, "nature": "Neutral" }
  ],
  "sade_sati": { "is_active": false, "phase": null,           // phase string only when active
                 "saturn_sign": "Pisces", "moon_sign": "Libra" },
  "period_quality": "Favorable",                               // enum-as-string (5 values)
  "retrograde_planets": ["Rahu", "Ketu"]                       // names of currently-retro transit grahas
}
```

**Domain struct `TransitAnalysisResult` (`models.rs:321`) — matches runtime 1:1:**
`natal_positions: Vec<PlanetaryPosition>` · `transit_positions: Vec<PlanetaryPosition>` · `aspects: Vec<TransitAspect>` · `sade_sati: SadeSatiStatus` · `period_quality: PeriodQuality` · `retrograde_planets: Vec<TransitPlanet>`.
`PlanetaryPosition` (`models.rs:157`): `planet, longitude:f64, latitude:f64, speed:f64, sign:ZodiacSign, degree_in_sign:f64, is_retrograde:bool` — **`latitude` is dropped by the serializer; `speed` is emitted only on `transit_positions`** (`engine.rs:117` vs `engine.rs:131`).
`TransitAspect` (`models.rs:256`): `transiting_planet, natal_planet, aspect_type:AspectType, orb:f64, is_applying:bool, nature:AspectNature`.
`SadeSatiStatus` (`models.rs:290`): `is_active:bool, phase:Option<SadeSatiPhase>, saturn_sign, moon_sign`.

**OpenAPI stub `TransitsResultSchema` (`types.rs:302`):** `{ dominant_transit:String, intensity:String, next_peak_date:String }` — **fictional**; no overlap with the real schema (§7).

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `*_positions[].planet` | enum, **12 grahas** | `Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, Rahu, Ketu` (`models.rs:9`, order from `TransitPlanet::all()` L28). Serialized as PascalCase string |
| `*_positions[].longitude` | **0–360°** | ecliptic longitude, sidereal (Lahiri). Normalized `((lon%360)+360)%360` (`models.rs:90`). Rounded to 3 dp (`engine.rs:119`) |
| `*_positions[].sign` | 12 signs `Aries…Pisces` | derived `floor(longitude/30)` (`models.rs:89`). String name |
| `*_positions[].degree_in_sign` | **0–30°** | `longitude % 30` (`models.rs:108`). Rounded to 2 dp |
| `transit_positions[].speed` | deg/day, **signed** | negative ⇒ retrograde (`models.rs:163`). ~13°/day Moon … ~0.01°/day Pluto (fixture L19–28). Rounded 4 dp. **Absent on `natal_positions`** |
| `*_positions[].is_retrograde` | bool | `speed < 0`. Sun & Moon never retrograde |
| `aspects[].aspect_type` | 5 enum strings | `Conjunction`(0°) `Sextile`(60°) `Square`(90°) `Trine`(120°) `Opposition`(180°) (`models.rs:175`, angles L190) |
| `aspects[].orb` | **0–8°** | actual angular deviation from exact. Max = that aspect's allowed orb: Conj/Opp **8°**, Trine/Square **6°**, Sextile **4°** (`default_orb` `models.rs:201`). Rounded 2 dp |
| `aspects[].is_applying` | bool | true = separation shrinking (planet moving toward exact) |
| `aspects[].nature` | 3 enum strings | `Harmonious` (trine/sextile), `Challenging` (square/opposition), `Neutral` (conjunction) (`models.rs:212`) |
| `sade_sati.phase` | 3 strings \| null | `"Rising (12th from Moon)"`, `"Peak (conjunct Moon)"`, `"Setting (2nd from Moon)"` (Display, `models.rs:278`). Null ⇔ `is_active=false` |
| `sade_sati.{saturn,moon}_sign` | sign name | comparison basis for the 3-sign Saturn-over-Moon window |
| `period_quality` | **5 enum strings** | `HighlyFavorable, Favorable, Mixed, Challenging, Difficult` → Display adds spaces: `"Highly Favorable"` (`models.rs:298`/L307) |
| `retrograde_planets` | subset of the 12, by name | mirrors `transit_positions[].is_retrograde==true` |

Ayanamsa: **Lahiri sidereal**, Swiss Ephemeris (`engine.rs:229` backend `"swiss-ephemeris"`; vedic-api facade defaults `ayanamsa: "lahiri"`, `api.rs:54`). **Invariants:** both arrays are length **12** in graha order (engine `validate()` asserts exactly 12, `engine.rs:278`/L290); `sign`/`degree_in_sign`/`is_retrograde` are all *derived from* `longitude`+`speed` (not independent); each aspect's `orb ≤ default_orb(aspect_type)` by construction.

## 4. Component & brand archetype
**Today** (`Transits.tsx`): an SVG **zodiac wheel** (`viewBox 0 0 300 300`, center 150,150) — a 12-segment outer ring (`ZodiacWheel`, L202) tinted by element (`ELEMENT_COLORS`: fire/earth/air/water, L22), 30° spoke lines, a dark inner disc labelled "TRANSITS", **planet dots** placed at `sign*30 + degree` on radius 104 with per-planet glyph+color (`PLANET_META`, L34) and a red "R" retrograde marker, plus a **Sade Sati arc** (±20° gold sweep around Saturn, L279). Below: text cells for Planetary Positions, a Significant Aspects list, and a Sade Sati status/phase block. Already partly on-brand (gold `#C5A017` Sun, emerald Venus, Void inner disc) — **aspects are text-only; there are no aspect lines on the wheel, and the wheel only ever plots one position set** (see §5/§7).

**Wave-2 target — orbital constellation:** the 12 grahas as luminous nodes on the zodiac wheel (angle = ecliptic longitude), with **aspect lines drawn chord-to-chord between the planets they connect** (`aspects[]` → line from `transiting_planet` node to `natal_planet` node), the line **hue by `nature`** (emerald harmonious / terracotta challenging / gold neutral) and **weight/opacity by tightness** (`1 − orb/maxOrb`). Node **glow ∝ involvement** (count/closeness of aspects). Show **two concentric position rings** (natal inner, transit outer) so aspect chords visibly bridge the two, with the wheel itself the constellation field; `period_quality` sets the ambient core color, Sade Sati keeps the gold arc.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `transit_positions[].longitude` | planet node angle on the wheel (`zodiacToXY`, L168); currently `sign*30+degree`, equivalent to longitude |
| natal vs transit ring | (target) inner ring = `natal_positions`, outer ring = `transit_positions`; today only one set is plotted (§7) |
| `*_positions[].sign` | which 30° element-tinted segment the node falls in |
| `is_retrograde` / `retrograde_planets` | red "R" glyph beside the node (`Transits.tsx:314`); (target) reversed/dimmed node |
| `transit_positions[].speed` | (target) node drift/animation rate (fast Moon vs near-static Pluto) |
| `aspects[]` (`transiting_planet`→`natal_planet`) | **(target) chord line between the two nodes** — the defining constellation geometry; today rendered only as a text row |
| `aspects[].nature` | line color: emerald harmonious / terracotta challenging / gold neutral |
| `aspects[].orb` | line weight + opacity (tighter orb ⇒ brighter/thicker) |
| `aspects[].aspect_type` | line style per angle family (e.g. solid trine, dashed square) + badge label |
| `sade_sati.is_active` + Saturn position | gold ±20° arc around Saturn on the rim (`Transits.tsx:279`) |
| `period_quality` | (target) ambient/core hue (HighlyFavorable→emerald … Difficult→terracotta) |

## 6. Dynamics
**One-shot per (natal birth datetime+place, transit date).** Computed server-side; not a live client stream. The natal half is fixed for a person; the transit half advances with the chosen `transit_date` (the vedic-api facade defaults transit date to *today*, `api.rs:223`), so re-rendering for "now vs next week" is the natural recompute trigger — fast grahas (Moon ~13°/day) move visibly day-to-day, slow ones (Saturn/outer planets) barely. On load, nodes and aspect chords should draw in once (stroke-dashoffset); an optional slow breath on the core (`period_quality`) is the only perpetual motion. `consciousness_level` (0–5, envelope) gates interpretive depth — the witness prompt itself names the dominant aspect (fixture: *"As Pluto deeply transforms… Saturn…"*). No baseline/delta semantics.

## 7. Open questions / assumptions
- **Renderer ⇄ runtime key mismatch (confirmed, likely a real defect).** The renderer reads `result.planetary_positions ?? result.positions ?? result.transits` for the wheel/positions and `result.significant_aspects ?? result.aspects` for aspects (`Transits.tsx:392`/L393), but the runtime emits **`transit_positions`** + **`natal_positions`** + **`aspects`** (`engine.rs:163`, confirmed by `expected_outputs` fixture). `aspects` is the only key that lines up. Result against today's real payload: **the wheel plots zero planets** (no `planetary_positions`/`positions`/`transits` key) and the **Planetary Positions** text grid is empty; only the aspects list and Sade Sati block render. Flag for Wave-2: renderer must read `transit_positions`/`natal_positions`.
- **Aspect field-name mismatch (confirmed).** Renderer reads `aspect.planet1 ?? aspect.from`, `aspect.planet2 ?? aspect.to`, `aspect.type ?? aspect.aspect` (`Transits.tsx:445`/L454); runtime emits `transiting_planet`/`natal_planet`/`aspect_type`. So even the aspects list shows `"— —"` / `"—"` today. Building the constellation chords requires reading `transiting_planet`/`natal_planet`.
- **`sade_sati.active` vs `is_active` (confirmed).** Renderer keys off `sade_sati.active` (`Transits.tsx:400`/L462); runtime emits `is_active`. The Sade Sati block and the gold arc are therefore inert against real data.
- **OpenAPI stub is fictional (confirmed).** `TransitsResultSchema` (`dominant_transit`/`intensity`/`next_peak_date`, `types.rs:302`) shares **no field** with the real output. Ignore it except as a naming placeholder; it is not the contract.
- **Two engine paths, two shapes (confirmed).** The orchestrator path (this doc's authoritative shape) emits `natal_positions`+`transit_positions`+`aspects[]`. The separate `noesis-vedic-api` facade `TransitApiResponse` (`api.rs:61`) emits a flat `transits: [{planet, sign, degree, is_retrograde, natal_aspects:[{natal_planet, aspect_type, orb}]}]` + `sade_sati{is_active,…}` + `jupiter_transit{…}` — note it **nests aspects under each transit planet** and has **no natal-position array**. Confirm which path actually backs the web `EngineOutput` for transits before wiring the renderer; the `expected_outputs` fixture says it's the orchestrator/`engine-transits` shape.
- **Raw capture fixtures are input-side, not `result`.** `planets_bangalore_1991-08-13.json` / `western_houses_…json` are the vendor ephemeris/house capture format (`fullDegree` 0–360, `normDegree` 0–30, `current_sign` **1-indexed** 1=Aries…12=Pisces, `isRetro` as the **string** `"true"`/`"false"`, houses 1–12 with `zodiac_sign.number`). They feed position computation but do **not** match the engine `result` schema (which is 0-indexed enums, real `bool`, no house field). Don't render against them directly.
- **No house data in the transit `result`.** `PlanetaryPosition` has no house field; houses 1–12 appear only in the raw capture (`house_number`, `western_houses` fixture). If the constellation needs house overlays, that data isn't in the current engine output — flag as a gap.
- **`latitude` dropped, `speed` natal-absent (confirmed, minor).** `serialize_result` omits `latitude` entirely and emits `speed` only on transit positions (`engine.rs:117` vs L131); fine for a 2-D wheel but note natal retrograde is still carried via `is_retrograde`.
