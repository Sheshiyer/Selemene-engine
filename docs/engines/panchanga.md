# Panchanga — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

The five limbs of Vedic time (pañcāṅga) for a moment + place: tithi, nakshatra, yoga, karana, vara.

## Calculation contract from engine-panchanga 3.3.2

`POST /api/v1/engines/panchanga/calculate` returns the flat native result
described below. The historical nested Vedic adapter shape is a separate API
surface. The native calculation now follows these rules:

- `options.mode: "daily"` calculates positions at the exact UTC `current_time`,
  including seconds. A supplied `birth_data.timezone` selects the local date
  for vara without shifting that instant. Without birth data, vara uses UTC.
- Birth mode interprets `birth_data.date` and `time` in its timezone; a missing
  time uses local noon and sets `birth_time_assumed: true`.
- IANA zones use their rules at the calculation instant. Fixed `+HH:MM`,
  `-HH:MM` and whole-hour offsets are supported, along with legacy fixed
  `PST`, `CST` and `AEST`. Unknown or malformed zones return a validation error.
- A repeated DST wall time uses its earlier occurrence. A skipped wall time
  uses the offset before the transition. Callers requiring a different
  occurrence should supply an explicit fixed offset for that birth time.
- Vara uses the local civil date, with `vara_basis: "local_date"`. This is a
  midnight-based weekday, not a sunrise-based convention.
- Sidereal longitudes subtract the Swiss Ephemeris dated Lahiri ayanamsa.
  `ayanamsa_degrees` reports the value actually applied, and the native Vedic
  adapter reports the same value.

These corrections can change DST/previously unrecognized-zone positions,
local-midnight vara, and sidereal values previously based on a fixed 23.72
degrees. Tithi and karana are unaffected by an ayanamsa-only change because
the same correction cancels in Moon minus Sun. Time corrections can change
them too. New outputs identify this calculation as `metadata.engine_version:
"3.3.2"`.

The cache identity now distinguishes daily/birth mode, daily instant and
timezone, and missing versus explicit birth time. It differs from the old
identity, so old cached calculations are not reused. No cache flush, database
migration or rewrite of stored readings is required. Existing result JSON
remains readable: absent `vara_basis`, `ayanamsa_degrees` and `position_source`
deserialize as unknown rather than being relabelled as newly calculated.

Saved readings keep their original values and provenance. The separate
noesis-daily repository retains its accepted readings and its 06:00 IST
refresh anchor; changing or recalculating those is outside this correction.

The OpenAPI Panchanga schema and `POST /api/legacy/panchanga/calculate`
carry `position_source`, `birth_time_assumed`, `vara_basis` and
`ayanamsa_degrees`. Source/basis/ayanamsa are nullable for historical
compatibility; current native calculations report their actual method values.
The schema mirrors the full flat result (§2) and a drift test keeps them in step.

## 1. Identity
| | |
|---|---|
| `engine_id` | `panchanga` |
| Domain crate | `crates/engine-panchanga/src/lib.rs` (`PanchangaResult`) |
| Runtime source | `engine_panchanga::PanchangaEngine`, registered under `panchanga` in `noesis-orchestrator/src/lib.rs` (`register_native_runtime_engines`) |
| Renderer | None in this repository. `apps/noesis-web` (`Panchanga.tsx`) was retired; the renderer is being ported to Sankalpa, outside Selemene |
| Tests | `crates/engine-panchanga/src/lib.rs` (`probe_*`), `crates/engine-panchanga/tests/` |
| OpenAPI schema | `noesis-core/src/types.rs` `PanchangaResultSchema`: mirrors `PanchangaResult`; drift test `noesis-api/tests/openapi_schema_tests.rs::panchanga_result_schema_matches_engine_output` |
| SDK round-trip tests | `crates/noesis-sdk/src/client.rs::panchanga_calculate_keeps_provenance_fields`, `packages/noesis-sdk-ts/src/index.test.ts` ("panchanga provenance") |
| Persistence test | `crates/noesis-api/tests/panchanga_reading_persistence_tests.rs` (Postgres via `scripts/cloud/test-db.sh`): a saved reading keeps the provenance fields; older readings are returned as stored |

## 2. Output schema

**Runtime JSON (authoritative):** `/api/v1/engines/panchanga/calculate` returns
the flat `PanchangaResult` as `result`. Synthetic example, 2000-01-01 12:00 UTC:
```jsonc
{
  "tithi_index": 25,      "tithi_name": "Ekadashi (Krishna)",   "tithi_value": 25.246,   // 0..30; 0-14 Shukla, 15-29 Krishna
  "nakshatra_index": 14,  "nakshatra_name": "Swati",            "nakshatra_value": 14.96, // 0..27
  "yoga_index": 7,        "yoga_name": "Dhriti",                "yoga_value": 7.198,      // 0..27
  "karana_index": 0,      "karana_name": "Bava",                "karana_value": 51.0,     // value = half-tithi slot 1..=60
  "vara_index": 6,        "vara_name": "Shanivara (Saturday)",  "vara_basis": "local_date",
  "solar_longitude": 256.512, "lunar_longitude": 199.467,       // sidereal (Lahiri), 0..360
  "julian_day": 2451545.0,    "ayanamsa_degrees": 23.857,       // dated Lahiri value applied
  "position_source": "swiss_ephemeris_files",                   // or swiss_ephemeris_moshier / mean_longitude (fallbacks)
  "birth_time_assumed": false                                   // true when birth time was missing and noon was used
}
```
`vara_basis`, `ayanamsa_degrees` and `position_source` are absent only on results
stored before those fields existed. All indices are 0-based. Paksha is not a
field: it follows from `tithi_index` (below 15 is Shukla).

**Not this route:** the nested shape (`tithi.name`, `tithi.paksha`,
`nakshatra.pada`, `nakshatra.ruler`, `muhurta.quality`, ...) belongs to the
historical `crates/noesis-vedic-api/src/panchang/` adapter and the retired
noesis-web renderer. Its JHora reference fixture is
`crates/noesis-vedic-api/tests/fixtures/reference_data/panchang_jhora_reference.json`.
Sections 3 to 5 below still describe fields in that nested shape.

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
