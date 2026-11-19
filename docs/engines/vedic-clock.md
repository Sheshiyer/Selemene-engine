# Vedic Clock — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

A TCM organ clock (12 × 2-hour windows) fused with Ayurvedic dosha periods (3 × two daily cycles) plus hora, choghadiya and the day periods from local sunrise/sunset, and optional Panchanga overlays — temporal recommendations for *now* at a place. Replay semantics are in §8.

## 1. Identity
| | |
|---|---|
| `engine_id` | `vedic-clock` (verified: `engine.rs:38`, asserted in tests `vedic_clock_tests.rs:339`) |
| Domain crate | `crates/engine-vedic-clock/src/models.rs` (`VedicClockResult`, L256) |
| Runtime source | **`crates/engine-vedic-clock/src/engine.rs`** (`build_result`, L126 — the authoritative serializer) |
| Renderer | `apps/noesis-web/src/components/engines/VedicClock.tsx` |
| Fixture (real values) | **none found** (`crates/engine-vedic-clock/tests/vedic_clock_tests.rs` asserts shape, not a JHora reference fixture) |
| OpenAPI schema | `noesis-core/src/types.rs` `VedicClockResultSchema`: the top-level fields of `build_result`, pinned by `noesis-api` test `vedic_clock_result_schema_matches_engine_output` |

Unlike the other Vedic engines (panchanga/vimshottari/transits), this one is **not** served by `noesis-vedic-api` — it's a self-contained `engine-*` crate. There is **one** computation path: `engine.rs`. The struct, stub, and renderer disagree (§2/§7); **`build_result` is the runtime truth**.

## 2. Output schema

**Runtime JSON (authoritative — exactly what `build_result` emits, `engine.rs:134-188`):**
```jsonc
{
  "current_organ": {                       // engine.rs:135
    "organ": "Heart",                      // Organ enum → variant string (12 values, models.rs:14)
    "element": "Fire",                     // Element enum → variant string (models.rs:70)
    "time_window": "11 AM - 1 PM",         // time_range_display() — NOTE: key is time_window, not time_range
    "peak_energy": "…",
    "associated_emotion": "…",
    "recommended_activities": ["…"]
  },
  "current_dosha": { "dosha": "Pitta", "qualities": ["Transformation","Digestion","…"] }, // engine.rs:143
  "recommendation": {                      // engine.rs:147
    "time_window": "11 AM - 1 PM",
    "organ": "Heart", "dosha": "Pitta",
    "activities": [ { "activity": "…", "quality": "optimal", "reason": "…" } ],
    "panchanga_quality": null              // string ONLY if tithi_index/nakshatra_index passed (integration.rs:35)
  },
  "synthesis": "Heart (Fire) time during Pitta period - harmonious energy. …", // string, integration.rs:159
  "calculated_for": "2026-03-08T12:43:16+00:00",   // rfc3339 of input.current_time
  "timezone": { "offset_minutes": 330, "source": "birth_data.timezone", "local_hour": 12 }, // engine.rs:156
  "activity_timing": {                     // OPTIONAL — only when options.activity set (engine.rs:164)
    "activity": "Meditation", "is_favorable_now": true, "reason": "…",
    "optimal_windows": [ { "time_window": "…", "quality": 0.87, "reason": "…" } ]
  },
  "upcoming_transitions": [                 // OPTIONAL — JSON key, NOT the struct's `upcoming` (engine.rs:185)
    { "time": "13:00", "description": "Small Intestine time begins", "new_organ": "SmallIntestine", "new_dosha": null }
  ],
  "vedic_day": {                           // ALWAYS present (#1523); engine.rs `add_vedic_time`
    "status": "computed",                  // or "not_computed" (+ not_computed_reason, detail, omitted)
    "date": "2026-03-08", "weekday": "Sunday",       // local date whose sunrise starts this Vedic day
    "sunrise": "2026-03-08T06:31:37+05:30", "sunset": "2026-03-08T18:29:23+05:30",
    "next_sunrise": "2026-03-09T06:31:00+05:30",
    "location": { "latitude": 12.9716, "longitude": 77.5946, "source": "options" }, // options | location | birth_data
    "sun_event_method": "noaa-solar-position", "sun_event_altitude_deg": -0.833
  },
  "hora": {                                // ONLY when vedic_day.status == "computed"
    "number": 6, "ruler": "Jupiter", "period": "day",
    "start": "2026-03-08T11:30:41+05:30", "end": "2026-03-08T12:30:30+05:30",
    "is_favorable": true, "recommendations": [ { "activity": "…", "quality": "favorable", "reason": "Jupiter Hora" } ]
  },
  "choghadiya": {                          // ONLY when vedic_day.status == "computed"
    "number": 4, "name": "Amrit", "meaning": "Nectar", "nature": "good", "ruler": "Moon", "period": "day",
    "start": "2026-03-08T11:00:47+05:30", "end": "2026-03-08T12:30:30+05:30",
    "suitable_for": ["General", "Starting new ventures", "Medical"], "avoid": [], "recommendations": [ … ]
  },
  "replay": {                              // ALWAYS present (#1250); engine.rs `ReplayInputs`
    "deterministic": true,
    "fingerprint": "ffb115669386ee11",     // FNV-1a of the inputs + engine version; seeds the witness prompt
    "inputs": {
      "current_time": "2026-03-08T06:43:16Z",
      "timezone": { "offset_minutes": 330, "source": "options.timezone_offset", "zone": null },
      "location": { "latitude": 12.9716, "longitude": 77.5946, "source": "options" },
      "activity": null, "tithi_index": null, "nakshatra_index": null, "consciousness_level": 2
    },
    "excluded": ["metadata.timestamp", "metadata.calculation_time_ms", "metadata.cached"]
  },
  "provenance_detail": {                   // ALWAYS present (#1250); engine.rs `provenance_detail`
    "current_organ": { "origin": "table_lookup", "keyed_by": "local_hour" },
    "sunrise_sunset": { "origin": "calculated", "method": "noaa-solar-position" },  // or not_computed + reason
    "hora": { "origin": "calculated", "method": "sunrise_to_sunset_in_12" },
    "panchanga_quality": { "origin": "not_supplied" },          // or table_lookup + inputs: caller_supplied
    "recommendation": { "origin": "derived_interpretation" },
    "witness_prompt": { "origin": "templated_narrative", "selection": "seeded_by_replay_fingerprint" },
    "fallback": { "used": false, "reason": "…" }, "mock": { "used": false },
    "generated_artifacts": { "status": "not_applicable", "reason": "the engine renders no media" }
    // also current_dosha, upcoming_transitions, choghadiya, day_periods, synthesis, activity_timing
  }
}
```

**Domain struct `VedicClockResult` (models.rs:256) — field names differ from the JSON:** `current_organ:OrganWindow` · `current_dosha:DoshaTime` · `recommendation:TemporalRecommendation` · `upcoming:Option<Vec<UpcomingTransition>>` · `calculated_for:String`. The struct has **no** `synthesis`, `timezone`, or `activity_timing` (those are added ad-hoc in `build_result`), and its `upcoming` is **renamed to `upcoming_transitions`** in the JSON. `OrganWindow` (L134) carries `start_hour`/`end_hour:u8` which are **not** serialized — only the formatted `time_window` string is.

**OpenAPI schema (`VedicClockResultSchema`):** until #1250 it described a different clock (`current_segment`/`guna_bias`/`next_transition_local`) that the engine never emitted. It now lists exactly the top-level keys above, with `activity_timing`, `hora`, `choghadiya` and `day_periods` optional; nested objects are typed `object` and described here.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `current_organ.organ` | 12 enum values | `Lung, LargeIntestine, Stomach, Spleen, Heart, SmallIntestine, Bladder, Kidney, Pericardium, TripleWarmer, Gallbladder, Liver` (models.rs:14); cycle starts 3 AM = Lung (models.rs:31) |
| `current_organ.element` | `Wood\|Fire\|Earth\|Metal\|Water` | models.rs:70. **Mismatch vs renderer** — see §7 (Pericardium/TripleWarmer are Fire in the engine, Wood in the renderer) |
| organ window length | **2 h**, fixed | every window spans 2 hours (test `test_each_organ_window_is_2_hours`); transitions on **odd** local hours 1,3,5…23 (calculator.rs:60) |
| `current_dosha.dosha` | `Vata\|Pitta\|Kapha` | models.rs:105 |
| dosha period length | **4 h**, fixed, 6 periods/day | Vata 2–6 & 14–18; Kapha 6–10 & 18–22; Pitta 10–14 & 22–2 (dosha.rs:14-92); transitions on hours 2,6,10,14,18,22 (engine.rs:215) |
| `timezone.offset_minutes` | integer minutes, **−720…840** | `options.timezone_offset` (must be an integer in range) → `birth_data.timezone` (IANA name, resolved with daylight saving **at the requested instant**, or `±HH:MM`/`±HH`, or `PST`/`CST`/`AEST`) → `default_utc` (`resolve_timezone`). An unknown zone, a malformed offset or a non-integer/out-of-range `timezone_offset` is a `ValidationError`; it never falls through to UTC |
| `options.tithi_index` | **0–29** | 15–29 (Krishna) fold onto the same 15 tithi qualities as 0–14; anything else → `ValidationError` |
| `options.nakshatra_index` | **0–26** | anything else → `ValidationError` |
| `timezone.local_hour` | **0–23** | `get_local_hour` wraps mod 24 (calculator.rs:31) |
| `recommendation.activities[].quality` | `optimal\|favorable\|neutral\|use caution\|panchanga-favored\|avoid` | string enum from Panchanga rating (integration.rs:63-101) |
| `activity_timing.optimal_windows[].quality` | **0.0–1.0** f64 | favorability score, sorted descending (`TimeWindow.quality`, models.rs:210; test L193) |
| `activity` (input + echoed) | `Meditation\|Exercise\|Work\|Eating\|Sleep\|Creative\|Social` | 7 values (models.rs:177); parsed lowercase from `options.activity` (engine.rs:98) |
| `upcoming_transitions[]` | look-ahead **6 h** | `new_organ` set on organ hops, `new_dosha` on dosha hops; never both (engine.rs:200-224) |
| `recommendation.panchanga_quality` | `null` or `"Rating: description"` | non-null only when `tithi_index` and/or `nakshatra_index` supplied (integration.rs:35) |
| `vedic_day.not_computed_reason` | `location_missing\|no_sunrise_or_sunset` | no location supplied, or polar day/night; `hora`/`choghadiya` are then omitted, never guessed |
| location | lat −90…90, lon −180…180 | `options.latitude`+`options.longitude` → `location` → `birth_data`; out of range, non-numeric, or only one of the pair → `ValidationError` |
| `hora.number` | **1–24** | 1–12 divide sunrise→sunset, 13–24 sunset→next sunrise; first hora = weekday lord, then Sun→Venus→Mercury→Moon→Saturn→Jupiter→Mars (vedic_time.rs) |
| `choghadiya.number` | **1–8** per `period` | day/night halves divided by 8; day starts with the weekday lord's choghadiya, night five places on (vedic_time.rs) |
| sunrise/sunset accuracy | within ~10 s of Swiss Ephemeris `swe_rise_trans` below ±60° latitude | apparent upper limb with refraction, offline (sun.rs) |
| `consciousness_level` | **0–6** | input `options.consciousness_level`, default **2**; the API route sets it from the caller's token. Picks the witness-prompt depth (0–2 awareness, 3–4 observation, 5–6 integration) |

**Invariants:** organ + dosha are **pure functions of `local_hour`** (which derives from `current_time` + `timezone.offset_minutes`) — they co-vary, never independent. `local_hour` fully determines `current_organ`, `current_dosha`, `synthesis`, and `upcoming_transitions`. No ephemeris / sidereal longitudes are involved in the organ/dosha part. `vedic_day`/`hora`/`choghadiya` are functions of `current_time`, location, and `timezone.offset_minutes` (the offset picks the local date, hence the weekday); sunrise/sunset come from offline solar-position arithmetic (sun.rs), with no network call. Replays must pin all three inputs; §8 lists every input and how it is recorded.

## 4. Component & brand archetype
**Today** (`VedicClock.tsx`): a **12-segment radial organ clock** (240×240 SVG donut, `SEG_DEG=30`, `R_OUTER=100`/`R_INNER=60`) — each segment colored by TCM element, the active organ filled emerald (`rgba(16,181,167,0.5)`) + bright stroke, organ abbreviations (LU, HT…) rotated to read outward, a live **clock hand** (`new Date()`, 3 AM = top/−90°, 15°/hr), a small center dot, and a center label (organ name + time range). Below the ring: text cells for organ/element, emotion/virtue, recommendation, dosha/period/rasa, and "Peak Organs Today" tags. Palette is already on-brand (gold `#C5A017`, emerald, water-indigo `rgba(11,80,251)`).

**Wave-2 target (brand archetype):** *radial day-clock compass* — the 24h/segment ring exists; finish it by (a) lighting the **current segment** from real data, (b) showing **guna/dosha as the core color** (bioluminescent center), and (c) a **next-transition tick** on the rim driven by `minutes_until_next_transition` (calculator.rs:56, currently unused by the web). Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `current_organ.organ` | active segment of the 12-ring (emerald fill + glow); matched case-insensitively by name (`findActiveIndex`, tsx:94) |
| `current_organ.element` | inactive segment fill color (Metal grey / Earth gold / Fire coral / Water indigo / Wood emerald, tsx:34) |
| `current_organ.time_window` | center sub-label + organ cell `time_range` line |
| `timezone.local_hour` (via `new Date()` today) | clock-hand angle; **target:** drive hand from `local_hour` not browser clock (see §7) |
| `current_dosha.dosha` | (target) core color of the ring center = guna/dosha bias |
| `upcoming_transitions[0].time` | (target) next-transition tick on the rim |
| `recommendation.activities[]` | recommendation text cell |
| `current_organ.peak_organs`/peaks | "Peak Organs Today" tag row (tsx:310) — **source key unconfirmed**, see §7 |

## 6. Dynamics
**Time-of-day engine — recompute as the clock advances**, not one-shot. The natural cadence is the **2-hour organ window** (the cache key is the exact instant plus every other replay input, `ReplayInputs::canonical`) with finer **4-hour** dosha and minute-level transition ticks. `minutes_until_next_transition` (calculator.rs:56) and `get_window_progress` (calculator.rs:97) exist in the crate to support a live progress sweep / countdown, but the **web does not call them** — today the only live element is the SVG clock hand, which reads the **browser's** `new Date()` (tsx:152) and is therefore **decoupled from the engine's `timezone`/`local_hour`** (wrong hand position for any non-local tz). For Wave-2: poll/recompute on window boundaries, animate the active segment + a rim countdown to `upcoming_transitions[0]`, and optionally a slow core breath on the dosha color. `consciousness_level` (0–5, default 2) gates witness-prompt depth only.

## 7. Open questions / assumptions
- **Renderer reads keys the engine never emits (BLOCKER):** `VedicClock.tsx:259-260` reads `result.tcm_organ_clock.*` and `result.ayurvedic_timing.*`, but `build_result` emits **top-level** `current_organ`/`current_dosha` with **no** `tcm_organ_clock`/`ayurvedic_timing` wrapper (grep finds these keys only in the renderer + an old `wave2-task-plan.json` note). Net effect: the guard at `tsx:264` (`if (!tcm && !ayurvedic) return <GenericEngineView/>`) is **always true for this engine's output → the radial clock never renders; it silently falls back to GenericEngineView.** The renderer was built against a planned static `tcm_organ_clock.json` shape that the crate doesn't produce. ⚠️ Must reconcile before Wave-2 — pick one shape (recommend: nest the engine output, or flatten the renderer reads).
- **Schema disagreement (resolved for OpenAPI in #1250):** the OpenAPI schema now matches the runtime JSON. The Rust struct `VedicClockResult` still differs (`upcoming` vs `upcoming_transitions`; no `synthesis`/`timezone`/`activity_timing`/`vedic_day`/`replay`), so `build_result` stays the serializer of record. The Brahma Muhurta / Sattva / 06:12 fields of the old stub describe a clock this engine does **not** implement.
- **Element mismatch (confirmed):** engine maps **Pericardium & TripleWarmer → Fire** (models.rs / `wisdom.rs` per test L88-89), but the renderer's `ORGANS[]` tags Pericardium/TripleWarmer (and Gallbladder) as **Wood** (tsx:28-30). Inactive-segment colors will be wrong for those organs. Verify against `wisdom.rs::get_organ_element`.
- **"Peak Organs Today" source unconfirmed:** renderer reads `tcm_organ_clock.peak_organs[]` (tsx:262) — no such field is produced by `build_result`. Dead UI under the current contract.
- **No reference fixture:** there is no JHora/golden-value fixture for this engine (only behavioral tests). Ranges above are from the struct + dosha/calculator tables + integration tests; "correct values" for a given timestamp can be reproduced from `engine.rs` but are not pinned to an external authority.
- **`guna` / `Sattva-Rajas-Tamas` is absent from the implementation** — the brand archetype's "guna as core color" has **no backing field** today; it would map most naturally onto `current_dosha.dosha` (the engine's actual triad) unless a guna calc is added.

## 8. Replay & determinism (#1250)

**Contract.** For one engine version, `result` and `witness_prompt` are a pure function of the inputs listed in `result.replay.inputs`. Equal inputs give byte-identical `result` and `witness_prompt`. Only `metadata.timestamp`, `metadata.calculation_time_ms` and `metadata.cached` vary; `result.replay.excluded` lists them.

| Input | How it binds | Evidence |
|---|---|---|
| `current_time` | Echoed exactly (`replay.inputs.current_time`). `EngineInput` defaults it to the server clock when a caller omits it (`noesis_core::default_current_time`), so a replay must send it. | `engine.rs` `ReplayInputs` |
| timezone | Offset in force at `current_time`, with its source and IANA zone name. IANA zones follow daylight saving. Unknown or malformed values are rejected. | `engine.rs` `resolve_timezone`, `offset_for_timezone`; probe `probe_boundary_iana_timezone_follows_daylight_saving` |
| location | `options.latitude`/`longitude` → `location` → `birth_data`, with source. Never guessed; missing → hora, choghadiya and day periods are `not_computed` with a reason. | `engine.rs` `resolve_location`; probe `probe_degraded_missing_location_keeps_organ_clock` |
| sunrise/sunset | Computed offline from date, location and offset (`noesis_vedic_api::sun`, NOAA solar position, −0.833°); echoed in `vedic_day` with the method. No network call or ephemeris file. | `vedic_time.rs` `vedic_day_for`; probe `probe_boundary_sunrise_is_bound_into_replay_and_cache_key` |
| temporal boundaries | Organ windows change on odd local hours, doshas on 2/6/10/14/18/22, hora/choghadiya at sunrise-relative instants. Periods are half-open `[start, end)`; a boundary instant belongs to the period it starts. | `vedic_time.rs` `split`; probe `probe_boundary_organ_window_change_is_not_cached_across` |
| activity, tithi/nakshatra, consciousness level | Echoed; tithi/nakshatra are range-checked and labelled caller-supplied. | `engine.rs` `get_panchanga_indices`; probe `caller_supplied_panchanga_indices_are_labelled` |
| witness prompt | Templates picked by `replay.fingerprint` (FNV-1a of the inputs and engine version), not a random source. | `witness.rs` `generate_witness_prompt_seeded`; probe `probe_happy_path_replays_identically` |

**Cache.** `cache_key` is the canonical text of the same inputs (`ReplayInputs::canonical`), so two requests share a cache entry only when they would produce the same result.

**Provenance.** `result.provenance_detail` labels each part `table_lookup`, `calculated`, `not_computed` (with reason), caller-supplied, `derived_interpretation` or `templated_narrative`. `fallback.used`, `mock.used` and `generated_artifacts` are explicit; this engine has no fallback, mock or rendered-artifact path. `organ_clock::get_temporal_recommendation_with_api` (through `noesis_vedic_api::CachedVedicClient`, with its cache, rate limiter and fallback) and `panchang_integration::recommendation_from_complete_panchang` are library helpers that are not on the `calculate` path, so their cache and fallback behaviour does not reach this engine's output.

**Probes.** `crates/engine-vedic-clock/tests/replay_probes.rs`: happy path, boundaries (sunrise, organ window, consciousness level, daylight saving), degraded (missing location), invalid input, serialization round trip. `crates/noesis-api/tests/vedic_clock_replay_route_tests.rs`: the API route keeps the engine fields, replays identically, and rejects an unknown timezone with 422 `VALIDATION_ERROR`.

**Not covered.** Deployed and operational axes are not evidenced here (no deployed route receipt or health/metric sample). SDK, CLI/TUI, workflows and persistence carry `result` as opaque JSON (`EngineOutput.result: Value`); the engine-level serialization round-trip probe covers that, and there is no surface-specific probe for each.

