# Vedic Clock — Data Reference

A TCM organ clock (12 × 2-hour windows) fused with Ayurvedic dosha periods (3 × two daily cycles) plus optional Panchanga overlays — temporal recommendations for *now* at a place. **Note the naming gap:** the OpenAPI stub describes a *Vedic muhurta / guna* clock (Brahma Muhurta, Sattva…), but the actual engine ships a *TCM-organ + dosha* clock. They are different systems under one `engine_id` (see §2, §7).

## 1. Identity
| | |
|---|---|
| `engine_id` | `vedic-clock` (verified: `engine.rs:38`, asserted in tests `vedic_clock_tests.rs:339`) |
| Domain crate | `crates/engine-vedic-clock/src/models.rs` (`VedicClockResult`, L256) |
| Runtime source | **`crates/engine-vedic-clock/src/engine.rs`** (`build_result`, L126 — the authoritative serializer) |
| Renderer | `apps/noesis-web/src/components/engines/VedicClock.tsx` |
| Fixture (real values) | **none found** (`crates/engine-vedic-clock/tests/vedic_clock_tests.rs` asserts shape, not a JHora reference fixture) |
| OpenAPI stub | `noesis-core/src/types.rs:269` (`VedicClockResultSchema` — `current_segment`/`guna_bias`/`next_transition_local`); enum variant at `types.rs:377` |

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
  ]
}
```

**Domain struct `VedicClockResult` (models.rs:256) — field names differ from the JSON:** `current_organ:OrganWindow` · `current_dosha:DoshaTime` · `recommendation:TemporalRecommendation` · `upcoming:Option<Vec<UpcomingTransition>>` · `calculated_for:String`. The struct has **no** `synthesis`, `timezone`, or `activity_timing` (those are added ad-hoc in `build_result`), and its `upcoming` is **renamed to `upcoming_transitions`** in the JSON. `OrganWindow` (L134) carries `start_hour`/`end_hour:u8` which are **not** serialized — only the formatted `time_window` string is.

**OpenAPI stub (types.rs:269) — a DIFFERENT clock entirely:** `{ current_segment:"Brahma Muhurta", guna_bias:"Sattva", next_transition_local:"06:12" }`. None of these three fields exists in the runtime JSON. This is the SDK/OpenAPI contract (via `EngineResultData::VedicClock`, types.rs:377) — it does not match what ships.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `current_organ.organ` | 12 enum values | `Lung, LargeIntestine, Stomach, Spleen, Heart, SmallIntestine, Bladder, Kidney, Pericardium, TripleWarmer, Gallbladder, Liver` (models.rs:14); cycle starts 3 AM = Lung (models.rs:31) |
| `current_organ.element` | `Wood\|Fire\|Earth\|Metal\|Water` | models.rs:70. **Mismatch vs renderer** — see §7 (Pericardium/TripleWarmer are Fire in the engine, Wood in the renderer) |
| organ window length | **2 h**, fixed | every window spans 2 hours (test `test_each_organ_window_is_2_hours`); transitions on **odd** local hours 1,3,5…23 (calculator.rs:60) |
| `current_dosha.dosha` | `Vata\|Pitta\|Kapha` | models.rs:105 |
| dosha period length | **4 h**, fixed, 6 periods/day | Vata 2–6 & 14–18; Kapha 6–10 & 18–22; Pitta 10–14 & 22–2 (dosha.rs:14-92); transitions on hours 2,6,10,14,18,22 (engine.rs:215) |
| `timezone.offset_minutes` | integer minutes, e.g. `0, 330, 345, -480` | resolved options→birth_data→UTC (engine.rs:65); `+05:45` Nepal = 345 (test L595) |
| `timezone.local_hour` | **0–23** | `get_local_hour` wraps mod 24 (calculator.rs:31) |
| `recommendation.activities[].quality` | `optimal\|favorable\|neutral\|use caution\|panchanga-favored\|avoid` | string enum from Panchanga rating (integration.rs:63-101) |
| `activity_timing.optimal_windows[].quality` | **0.0–1.0** f64 | favorability score, sorted descending (`TimeWindow.quality`, models.rs:210; test L193) |
| `activity` (input + echoed) | `Meditation\|Exercise\|Work\|Eating\|Sleep\|Creative\|Social` | 7 values (models.rs:177); parsed lowercase from `options.activity` (engine.rs:98) |
| `upcoming_transitions[]` | look-ahead **6 h** | `new_organ` set on organ hops, `new_dosha` on dosha hops; never both (engine.rs:200-224) |
| `recommendation.panchanga_quality` | `null` or `"Rating: description"` | non-null only when `tithi_index` and/or `nakshatra_index` supplied (integration.rs:35) |
| `consciousness_level` | **0–5** (envelope) | input `options.consciousness_level`, default **2** (engine.rs:291); gates witness-prompt depth |

**Invariants:** organ + dosha are **pure functions of `local_hour`** (which derives from `current_time` + `timezone.offset_minutes`) — they co-vary, never independent. `local_hour` fully determines `current_organ`, `current_dosha`, `synthesis`, and `upcoming_transitions`. No ephemeris / sidereal longitudes are involved (this is a wall-clock engine, not an astronomical one).

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
**Time-of-day engine — recompute as the clock advances**, not one-shot. The natural cadence is the **2-hour organ window** (the cache key buckets by `hour/2`, engine.rs:376) with finer **4-hour** dosha and minute-level transition ticks. `minutes_until_next_transition` (calculator.rs:56) and `get_window_progress` (calculator.rs:97) exist in the crate to support a live progress sweep / countdown, but the **web does not call them** — today the only live element is the SVG clock hand, which reads the **browser's** `new Date()` (tsx:152) and is therefore **decoupled from the engine's `timezone`/`local_hour`** (wrong hand position for any non-local tz). For Wave-2: poll/recompute on window boundaries, animate the active segment + a rim countdown to `upcoming_transitions[0]`, and optionally a slow core breath on the dosha color. `consciousness_level` (0–5, default 2) gates witness-prompt depth only.

## 7. Open questions / assumptions
- **Renderer reads keys the engine never emits (BLOCKER):** `VedicClock.tsx:259-260` reads `result.tcm_organ_clock.*` and `result.ayurvedic_timing.*`, but `build_result` emits **top-level** `current_organ`/`current_dosha` with **no** `tcm_organ_clock`/`ayurvedic_timing` wrapper (grep finds these keys only in the renderer + an old `wave2-task-plan.json` note). Net effect: the guard at `tsx:264` (`if (!tcm && !ayurvedic) return <GenericEngineView/>`) is **always true for this engine's output → the radial clock never renders; it silently falls back to GenericEngineView.** The renderer was built against a planned static `tcm_organ_clock.json` shape that the crate doesn't produce. ⚠️ Must reconcile before Wave-2 — pick one shape (recommend: nest the engine output, or flatten the renderer reads).
- **Three-way schema disagreement (confirmed):** (1) OpenAPI stub = muhurta/guna (`current_segment`/`guna_bias`/`next_transition_local`); (2) Rust struct `VedicClockResult` = TCM/dosha with field `upcoming`; (3) runtime JSON = TCM/dosha but renames `upcoming`→`upcoming_transitions` and adds `synthesis`/`timezone`/`activity_timing`. The **SDK/OpenAPI contract (`EngineResultData::VedicClock`, types.rs:377) advertises shape (1) while the engine ships shape (3)** — any client typed off the SDK will mis-parse. ⚠️ The brief's stub fields (Brahma Muhurta / Sattva / 06:12) describe a clock this engine does **not** implement.
- **Element mismatch (confirmed):** engine maps **Pericardium & TripleWarmer → Fire** (models.rs / `wisdom.rs` per test L88-89), but the renderer's `ORGANS[]` tags Pericardium/TripleWarmer (and Gallbladder) as **Wood** (tsx:28-30). Inactive-segment colors will be wrong for those organs. Verify against `wisdom.rs::get_organ_element`.
- **"Peak Organs Today" source unconfirmed:** renderer reads `tcm_organ_clock.peak_organs[]` (tsx:262) — no such field is produced by `build_result`. Dead UI under the current contract.
- **No reference fixture:** there is no JHora/golden-value fixture for this engine (only behavioral tests). Ranges above are from the struct + dosha/calculator tables + integration tests; "correct values" for a given timestamp can be reproduced from `engine.rs` but are not pinned to an external authority.
- **`guna` / `Sattva-Rajas-Tamas` is absent from the implementation** — the brand archetype's "guna as core color" has **no backing field** today; it would map most naturally onto `current_dosha.dosha` (the engine's actual triad) unless a guna calc is added.
