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
Sections 3 to 5 below use the flat fields.

## 3. Ranges, constraints & invariants
All indices are 0-based. Fields marked *derived* are not in the result; compute
them from the fields shown.

| Field | Range / domain | Notes |
|---|---|---|
| `tithi_index` | **0–29** | 0–14 Shukla (waxing; 14 = Purnima), 15–29 Krishna (waning; 29 = Amavasya) |
| `tithi_value` | 0 ≤ v < 30 | continuous: (Moon − Sun) / 12°; `tithi_index = floor(tithi_value)` |
| *paksha* (derived) | `Shukla` \| `Krishna` | `tithi_index < 15` → Shukla; also the suffix of `tithi_name` except Purnima/Amavasya |
| *tithi elapsed* (derived) | 0 ≤ f < 1 | `tithi_value − tithi_index` |
| `nakshatra_index` | **0–26** | 0 = Ashwini … 26 = Revati |
| `nakshatra_value` | 0 ≤ v < 27 | continuous: sidereal Moon / 13°20′ |
| *pada* (derived) | **1–4** | `floor(fract(nakshatra_value) × 4) + 1` |
| `yoga_index` | **0–26** | 0 = Vishkambha … 26 = Vaidhriti |
| `yoga_value` | 0 ≤ v < 27 | continuous: (Sun + Moon) / 13°20′, mod 27 |
| `karana_index` | **0–10** | 0–6 movable (Bava … Vishti), 7–10 fixed (Shakuni, Chatushpada, Naga, Kimstughna) |
| `karana_value` | **1–60** | half-tithi **slot**, not a continuous value; slot 1 = Kimstughna, 58–60 = Shakuni, Chatushpada, Naga |
| `vara_index` | **0–6** | 0 = Sunday; local civil date, see `vara_basis` |
| `vara_basis` | `local_date` | midnight-based weekday, not sunrise-based; absent on old results |
| `solar_longitude`, `lunar_longitude` | 0 ≤ λ < 360° | sidereal: tropical minus `ayanamsa_degrees` |
| `ayanamsa_degrees` | ≈ 23.86° at J2000, rising ≈ 0.014° a year | dated Lahiri value from Swiss Ephemeris; absent on old results |
| `julian_day` | Julian Day (UT) | the instant the positions were computed for |
| `position_source` | `swiss_ephemeris_files` \| `swiss_ephemeris_moshier` \| `mean_longitude` | only `swiss_ephemeris_files` is a non-fallback source; absent on old results |
| `birth_time_assumed` | bool | `true` when birth time was missing and local noon was used |

**Not emitted:** nakshatra ruler/lord, yoga nature (auspicious/inauspicious) and
muhurta. The ruler and karana type follow from the indices; muhurta lives in
`noesis-vedic-api`. `engine-vedic-clock`'s `panchanga_qualities` can rate a
tithi, but its nakshatra categories are a placeholder (`index % 6`), so do not
present them as classical classes.

**Boundaries:** names change at fixed longitude boundaries, so a time a few
minutes either side of a transition can give the neighbouring tithi, nakshatra,
yoga or karana. Engine tests pin exact names for synthetic references (for
example the 2026-03-10 Vishti→Bava karana transition).

Ayanamsa: **Lahiri (Chitrapaksha)**, dated. Invariant: tithi and yoga derive
from Moon − Sun and Sun + Moon, nakshatra from the Moon alone, so they co-vary.
An ayanamsa-only change leaves tithi and karana unchanged (it cancels in
Moon − Sun) but shifts nakshatra and yoga.

## 4. Component & brand archetype
**Renderer:** none in this repository. The retired noesis-web `Panchanga.tsx`
(being ported to Sankalpa) read the nested adapter shape. Its design, kept as
reference for the port:

**Previous design:** 6 text cells (tithi/nakshatra/yoga/karana/vara/day quality) + a **27-segment nakshatra donut ring** (`SEG_DEG=360/27`, active segment emerald-filled + glow) with a **tithi arc** (30 tithis → 0–360°, gold) and center label. On-brand (gold `#C5A017` + emerald `rgba(16,181,167)`, sacred-geometry ring, glow filter). Only nakshatra and tithi are geometric; yoga/karana/vara are text.

**Wave-2 target:** full **5-limb mandala**: concentric rings for tithi (30), nakshatra (27), yoga (27), karana (11 names; 60 slots), vara (7), each active segment lit on the Ba-Arc. Center = day quality as the bioluminescent core, once a quality source exists (see §3). Missing geometry drawn in (Anime.js stroke-dashoffset) on load.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `nakshatra_index` (0–26) | active segment `nakshatra_index` of the 27-ring (emerald fill + glow); center label = `nakshatra_name` + `#(nakshatra_index+1) · Pada p` |
| *pada* (derived, 1–4) | pada subdivision tick within the active segment |
| `tithi_value` (0–30) | gold arc sweep 0 → `tithi_value / 30 × 360°` (continuous, so the arc shows elapsed tithi) |
| *paksha* (derived from `tithi_index`) | arc direction / hue (Shukla waxing vs Krishna waning) |
| `yoga_index` (0–26) | (target) outer 27-ring active segment |
| `karana_index` (0–10) | (target) inner 11-ring active segment; or `karana_value` (1–60) on a 60-slot ring |
| `vara_index` (0–6) | (target) 7-spoke compass; weekday spoke lit |
| `position_source` | provenance badge: no badge for `swiss_ephemeris_files`; "approximate" marker for the two fallbacks |
| `birth_time_assumed` | "time unknown: computed for local noon" note beside the chart |
| day quality | core colour (emerald / gold / terracotta). Not in this result: needs a quality source first (see §3) |

## 6. Dynamics
**One-shot per (date, time, lat/long, tz).** Not live. Recompute when the moment or place changes. No baseline/delta semantics (unlike biofield). On render, the active segments + tithi arc should animate in (line-draw) once; no perpetual loop except an optional slow core breath (4:7:8) if quality is shown as the core. `consciousness_level` (0–5) may gate how much interpretive text (lords, nature, muhurta guidance) is surfaced.

## 7. Open questions / assumptions
- **Shape:** `/api/v1/engines/panchanga/calculate` returns the flat result (§2–5). The nested shape (`tithi.number`, `nakshatra.pada`, `nakshatra.ruler`, `tithi.percentage`, `muhurta.quality`) belongs to the historical `noesis-vedic-api` adapter and the retired renderer. A port reading the nested names will find nothing on this route; build Wave-2 against the flat fields.
- **Day quality:** no source emits a classical quality for this result. Decide whether the mandala's core needs one before porting it.
- **Nakshatra ruler and yoga nature:** not emitted. Add them to the engine if the port needs them, rather than inferring them in the renderer.
