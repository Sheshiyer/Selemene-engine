# Panchanga — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

The five limbs of Vedic time (pañcāṅga) for a moment + place: tithi, nakshatra, yoga, karana, vara.

## 1. Identity
| | |
|---|---|
| `engine_id` | `panchanga` |
| Domain crate | `crates/engine-panchanga/src/lib.rs` (`PanchangaResult`, L179) |
| Runtime source | **`crates/noesis-vedic-api/src/panchang/`** (mod.rs, mappers.rs) — JHora-verified |
| Renderer | `apps/noesis-web/src/components/engines/Panchanga.tsx` |
| Fixture (real values) | `crates/noesis-vedic-api/tests/fixtures/reference_data/panchang_jhora_reference.json` |
| OpenAPI stub | `noesis-core/src/types.rs:192` (`PanchangaResultSchema` — vara/paksha/nakshatra only) |

## 2. Output schema

**Runtime JSON (authoritative — the shape the renderer reads & the fixture verifies):**
```jsonc
{
  "tithi":     { "name": "Chaturthi", "paksha": "Shukla", "number": 4 },   // renderer also reads .percentage
  "nakshatra": { "name": "Revati", "number": 27, "pada": 2, "ruler": "Mercury" }, // renderer reads .lord (alias of ruler?)
  "yoga":      { "name": "Shiva", "number": 20, "nature": "auspicious" },
  "karana":    { "name": "Vanija", "type": "Movable" },
  "vara":      "Monday",                                                    // renderer also accepts result.weekday
  "muhurta":   { "quality": "auspicious" }                                 // renderer reads result.muhurta.quality | result.quality
}
```

**Domain struct `PanchangaResult` (engine-panchanga path — FLAT, different shape):**
`tithi_index:u8` `tithi_name:String` `tithi_value:f64` · `nakshatra_index:u8` `nakshatra_name` `nakshatra_value` · `yoga_index:u8` `yoga_name` `yoga_value` · `karana_index:u8` `karana_name` `karana_value` · `vara_index:u8` `vara_name` · `solar_longitude:f64` `lunar_longitude:f64` `julian_day:f64`.

**OpenAPI stub:** `{ vara:String, paksha:String, nakshatra:String }` — examples only.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `tithi.number` | **1–30** | 1–15 Shukla (waxing), 16–30 Krishna (waning); engine `tithi_value` is continuous 0..30 |
| `tithi.paksha` | `Shukla` \| `Krishna` | waxing / waning fortnight |
| `nakshatra.number` | **1–27** | engine `nakshatra_index` is 0–26; renderer's `NAKSHATRAS[]` is 0-indexed (number = index+1) |
| `nakshatra.pada` | **1–4** | quarter of the nakshatra |
| `nakshatra.ruler` | planet name | renderer reads `.lord` — confirm alias (`ruler` vs `lord`) |
| `yoga.number` | **1–27** | `yoga.nature` ∈ auspicious/inauspicious/mixed |
| `karana.number` | **1–11** | engine `karana_index` 0..11; 7 movable + 4 fixed (60 half-tithi slots) |
| `vara` | Sunday…Saturday | engine `vara_index` 0=Sunday |
| `solar_longitude`, `lunar_longitude` | **0–360°** | engine path only; sidereal (Lahiri ayanamsa) |
| tolerance | ±1 on tithi/nakshatra/yoga number | boundary values shift by minutes across ayanamsa/algorithm — see fixture `tolerance` |

Ayanamsa: **Lahiri (Chitrapaksha)**. Invariant: indices are derived from `(lunar−solar)` (tithi/yoga) and `lunar` (nakshatra) longitudes — they co-vary, not independent.

## 4. Component & brand archetype
**Today** (`Panchanga.tsx`): 6 text cells (tithi/nakshatra/yoga/karana/vara/muhurta-quality) + a **27-segment nakshatra donut ring** (`SEG_DEG=360/27`, active segment emerald-filled + glow) with a **tithi arc** (1–30 → 0–360°, gold) and center label. Already on-brand (gold `#C5A017` + emerald `rgba(16,181,167)`, sacred-geometry ring, glow filter) — only nakshatra+tithi are geometric; yoga/karana/vara are text.

**Wave-2 target:** full **5-limb mandala** — concentric rings for tithi (30), nakshatra (27, exists), yoga (27), karana (11), vara (7), each active segment lit on the Ba-Arc; center = muhurta quality as the bioluminescent core; missing geometry drawn-in (Anime.js stroke-dashoffset) on load.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `nakshatra.number` (1–27) | active segment of the 27-ring (emerald fill + glow); center label = name + `#n · Pada p` |
| `nakshatra.pada` (1–4) | pada subdivision tick within the active segment |
| `tithi.number` (1–30) | gold arc sweep 0→(n/30·360°) inside the donut |
| `tithi.paksha` | arc direction / hue (Shukla waxing vs Krishna waning) |
| `yoga.number` (1–27) | (target) outer 27-ring active segment |
| `karana.number` (1–11) | (target) inner 11-ring active segment |
| `vara` | (target) 7-spoke compass; weekday spoke lit |
| `muhurta.quality` | core color: emerald=auspicious, gold=neutral, terracotta=inauspicious |

## 6. Dynamics
**One-shot per (date, time, lat/long, tz).** Not live. Recompute when the moment or place changes. No baseline/delta semantics (unlike biofield). On render, the active segments + tithi arc should animate in (line-draw) once; no perpetual loop except an optional slow core breath (4:7:8) if quality is shown as the core. `consciousness_level` (0–5) may gate how much interpretive text (lords, nature, muhurta guidance) is surfaced.

## 7. Open questions / assumptions
- **Schema mismatch (confirmed):** renderer + fixture use **nested** objects; `engine-panchanga::PanchangaResult` is **flat**. The web is fed by **noesis-vedic-api** (nested). Build Wave-2 against the nested/fixture shape; treat the flat struct as the alternate engine path. ✅ flagged, not a defect.
- `nakshatra.ruler` vs renderer's `.lord` — likely the same field under two names; verify in `noesis-vedic-api/src/panchang/mappers.rs` before relying on `.lord`.
- `tithi.percentage` is read by the renderer but absent from the fixture — confirm vedic-api emits it (else the "% elapsed" line is always hidden).
- `muhurta` block shape beyond `.quality` (does it carry start/end times?) — not in the sampled fixture; check mappers.
