# Vimshottari Dasha — Data Reference

The 120-year planetary-period timeline of Vedic time: a fixed cycle of 9 lords, each ruling a major period (Mahadasha) nested into sub-periods (Antardasha/Bhukti → Pratyantardasha …), with the period active *now* and its upcoming transitions.

## 1. Identity
| | |
|---|---|
| `engine_id` | `vimshottari` (verified `crates/engine-vimshottari/src/engine.rs:37`) |
| Domain crate | `crates/engine-vimshottari/src/models.rs` (`VimshottariChart`, L8); `result` built in `engine.rs:202` |
| Runtime source | **`crates/noesis-vedic-api/src/vimshottari/`** (types.rs, mappers.rs, current.rs) — JHora-verified, like Panchanga |
| Renderer | `apps/noesis-web/src/components/engines/Vimshottari.tsx` |
| Fixture (real values) | `crates/noesis-vedic-api/tests/fixtures/reference_data/dasha_reference.json` (validation charts, **not** the API envelope) |
| OpenAPI stub | `noesis-core/src/types.rs:247` (`VimshottariResultSchema` — current_mahadasha/current_antardasha/days_remaining only) |

⚠️ **Unlike Panchanga, no single Rust serializer emits exactly what the renderer reads.** Five shapes disagree (see §2 + §7). The renderer's tolerant `??` fallbacks are the only confirmed web contract; treat it as the spec and the engine `result` as the closest producer.

## 2. Output schema

**Renderer contract (authoritative — what the web actually reads, `Vimshottari.tsx:79-123`):**
```jsonc
{
  "mahadasha":  { "planet": "Venus", "years_remaining": 12.4,            // OR top-level result.mahadasha (string)
                  "start": "2023-07-15", "end": "2043-07-15" },          // OR result.current_mahadasha
  "antardasha": { "planet": "Sun", "start": "...", "end": "..." },       // OR result.antardasha / result.current_antardasha
  "upcoming_periods": [                                                   // OR result.periods ; renders first 8
    { "planet": "Sun", "start": "...", "end": "...", "level": "antardasha" } // planet|name, level|type
  ]
}
```
Renderer reads only: `mahadasha.{planet,years_remaining,start,end}`, `antardasha.{planet,start,end}`, `upcoming_periods[].{planet|name, start, end, level|type}`. No rings/dates parsed today — all rendered as text (see §4).

**engine-vimshottari `result` (domain path — nested, the closest real producer, `engine.rs:202-216`):**
```jsonc
{
  "birth_nakshatra": { "name": "Ashwini", "number": 1, "moon_longitude": 8.5 },
  "timeline": { "birth_date": "<rfc3339>", "total_years": 120,
    "mahadashas": [ { "planet": "Sun", "start_date": "<rfc3339>", "end_date": "<rfc3339>",
                      "duration_years": 6.0, "antardasha_count": 9 } ] },     // top-level only; nested AD count, not list
  "current_period": {                                                          // ⚠ not "current_mahadasha"
    "mahadasha":       { "planet": "...", "start": "<rfc3339>", "end": "<rfc3339>", "years": 6.0 },
    "antardasha":      { "planet": "...", "start": "...", "end": "...", "years": 1.0 },
    "pratyantardasha": { "planet": "...", "start": "...", "end": "...", "days": 73.0 } },
  "upcoming_transitions": [ { "type": "Mahadasha", "from_planet": "...", "to_planet": "...",
                             "date": "<rfc3339>", "days_until": 1460 } ],      // ⚠ not "upcoming_periods"
  "period_enrichment": { "mahadasha_themes": [..], "combined_description": "...", "life_areas": [..],
                         "opportunities": [..], "challenges": [..] }            // null if no current period
}
```

**vedic-api `VimshottariTimeline` (runtime crate, `vimshottari/types.rs:165`):** `birth_datetime:NaiveDateTime` · `birth_nakshatra:String` · `birth_dasha_balance:DashaBalance{lord, years:u32, months:u32, days:u32, total_days:i64}` · `mahadashas:Vec<DashaPeriod>`. **`DashaPeriod`** (`types.rs:119`): `lord:DashaLord` · `level:DashaLevel` · `start_date:NaiveDate` · `end_date:NaiveDate` · `duration_days:i64` · `sub_periods:Vec<DashaPeriod>` (recursive; omitted when empty). Note: `lord` (not `planet`), `start_date`/`end_date` (not `start`/`end`), `duration_days` (not `years`).

**Domain struct `VimshottariChart` (`models.rs:8`):** `birth_date:DateTime<Utc>` · `mahadashas:Vec<Mahadasha>` · `current_period:CurrentPeriod` · `upcoming_transitions:Vec<Transition>`. `Mahadasha`(L17): `planet, start_date, end_date, duration_years:f64, antardashas:Vec<Antardasha>, qualities`. `Antardasha`(L28)→`pratyantardashas:Vec<Pratyantardasha>`; `Pratyantardasha`(L38) carries `duration_days:f64`. `CurrentPeriod`(L139): `{mahadasha:CurrentMahadasha, antardasha:CurrentAntardasha, pratyantardasha:CurrentPratyantardasha, current_time}` — Maha/Antar carry `years:f64`, Pratyantar carries `days:f64`.

**Orchestrator reader (a 2nd runtime consumer, `noesis-orchestrator/.../birth_blueprint.rs:195`):** expects `result.current_dasha.{mahadasha, antardasha, years_remaining}` + `result.upcoming_transitions[]` **as strings** — a THIRD shape (`current_dasha`), only seen in test mocks (`birth_blueprint.rs:370`, `synthesis/birth_blueprint.rs:468`), no real serializer.

**OpenAPI stub:** `{ current_mahadasha:String, current_antardasha:String, days_remaining:i64 }` — examples only.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| lord / planet | 9 grahas | `Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury` (`types.rs:11`). vedic-api serializes **lowercase** (`#[serde(rename_all="lowercase")]`, `types.rs:10`); engine path Title-Case (`as_str`, `models.rs:91`). ⚠ case mismatch |
| Mahadasha `duration_years` | fixed per lord | Ketu 7 · Venus 20 · Sun 6 · Moon 10 · Mars 7 · Rahu 18 · Jupiter 16 · Saturn 19 · Mercury 17 (`models.rs:61`, `types.rs:25`, fixture L229) |
| cycle total | **120 years** | Σ of the 9 durations = 120 (asserted `types.rs:268-272`; `total_years:120` hard-coded `engine.rs:210`) |
| sequence order | fixed wheel | `Ketu→Venus→Sun→Moon→Mars→Rahu→Jupiter→Saturn→Mercury→(Ketu)` (`next()` `types.rs:55`, fixture `correct_order` L227) |
| birth (1st) Mahadasha | **partial** | shortened by `birth_dasha_balance` — Moon's nakshatra longitude sets the entry lord + remaining fraction (fixture `moon_data`, e.g. Sun 3.46 yr L40; `is_birth_dasha:true`); only the first period is < full |
| `mahadashas` length | **9** | one full wheel from birth lord; `antardasha_count` = 9 per Maha (`engine.rs:149`) |
| sub-period depth | up to 5 levels | `DashaLevel` = Mahadasha(9) · Antardasha(81) · Pratyantardasha(729) · Sookshma(6561) · Prana (`types.rs:103`). Default = **Antardasha** (`types.rs:107`) |
| `pratyantardasha.days` / `duration_days` | days | sub-period durations in days, not years (`models.rs:42`, `types.rs:129`) |
| `days_until` / `days_remaining` | i64 ≥ 0 | days to a future transition (`models.rs:209`); clamped to 0 once past (`types.rs:156`) |
| `years_remaining` | f64 ≥ 0 | **renderer-only field — no Rust producer emits it** (engine gives `current_period.mahadasha.years` = total, not remaining). ⚠ see §7 |
| dates | start ≤ end, monotonic | periods are contiguous & non-overlapping; `start_date` of N+1 = `end_date` of N (fixture chains exactly) |
| `birth_dasha_balance` | y<lord-yrs, m 0–11, d 0–29 | remaining at birth (`types.rs:217`); `total_days` ≈ y·365+m·30+d (`mappers.rs:44`) |

**Ayanamsa: Lahiri** (`dasha_reference.json` `_meta.ayanamsa`). Tolerance: dates drift ±15–30 days across software (fixture `tolerance.date_days`); historical charts ±15 min birth-time uncertainty. **Invariant:** the entire timeline is a deterministic function of birth Moon longitude (entry lord + balance) — all 9 Mahadashas and every sub-period derive from it; nothing is independent.

## 4. Component & brand archetype
**Today** (`Vimshottari.tsx`): a **2-cell text grid** — Mahadasha (planet · "N years remaining" · From/Until) + Antardasha (planet · From/Until) — plus an **8-row "Upcoming Periods" table** (Planet · Start · End · Level). **Zero geometry, zero brand palette, no SVG** — the least-visual of the Vedic renderers; pure `var(--text)`/`var(--gold)` section title. Furthest of the Wave-2 set from the archetype.

**Wave-2 target:** **nested concentric time-arc rings** — outer ring = the 9 Mahadashas (each lord an arc sized by its years, full 120-yr wheel), middle = the current Maha's Antardashas (Bhukti), inner = Pratyantardashas. The **current period is lit on the Ba-Arc** at each level (Maha/Antar/Pratyantar segment glowing), with **`days_remaining` (or % elapsed) drawn as arc-fill progress** sweeping the active segment. Lord color per graha; transitions animate the boundary crossing (Anime.js stroke-dashoffset). Center = current `Maha-Antar-Pratyantar` display string as the bioluminescent core.

Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `timeline.mahadashas[].duration_years` (9, Σ=120) | outer ring segment arc-length (years → degrees: `yrs/120·360°`) |
| `mahadashas[].planet` / `lord` | segment hue (per-graha color); label on hover |
| current `mahadasha.planet` | outer active segment lit on the Ba-Arc + glow |
| `mahadasha.years_remaining` / `days_remaining` | arc-fill progress sweeping the active Maha segment |
| current `antardasha` (Bhukti) | middle ring active segment (subdivides the current Maha) |
| current `pratyantardasha` | inner ring active segment (`days`-scaled) |
| `upcoming_transitions[].date` + `days_until` | boundary tick on the ring; nearest transition pulses; animate on crossing |
| `current_period` display string | center core label (`Venus-Sun-Moon`) |
| `period_enrichment.*` (engine path) | (target) interpretive text panel, gated by `consciousness_level` |

## 6. Dynamics
**Hybrid: one-shot timeline + time-sensitive "current".** The full 9-Mahadasha wheel is computed once per (birth date/time, lat/long, tz) and is stable for life. But *which* period is **current** — and `years_remaining`/`days_remaining`/% elapsed — advances with **today's date** (`find_current_period`/`get_current_dashas` take `current_time`/`date`, `current.rs:11`, `engine.rs:296`). Re-resolve the active segment + progress on each render (date-granular; no sub-day ticking needed — a Pratyantardasha lasts weeks). On load, rings draw in once (line-draw) and the active segments light; a slow progress sweep is optional. Period boundaries (Maha every 6–20 yr, Antar months, Pratyantar weeks) are the natural transition events. `consciousness_level` (0–5) gates how much `period_enrichment` (themes, lessons, practices, challenges) is surfaced.

## 7. Open questions / assumptions
- **Schema mismatch (CRITICAL — renderer ↔ producer key names diverge):** the renderer reads `result.current_mahadasha` / `upcoming_periods` / `mahadasha.years_remaining`, but the engine `result` emits `current_period` / `upcoming_transitions` / `mahadasha.years` (= *total*, not remaining) (`engine.rs:154,177,160`). **As written, none of the renderer's primary keys match any known producer** → the cells fall through to `result.mahadasha`/`result.antardasha` (a bare string) and the Upcoming table renders empty. This is the top Wave-2 blocker: the producer JSON and `Vimshottari.tsx` must be reconciled before any ring work. Verify the live `noesis-vedic-api` engine endpoint's actual `result` shape (none of the 5 shapes here is confirmed as the deployed one).
- **No `years_remaining` producer anywhere** — renderer-only field; only `years` (total) and `days_until`/`days_remaining` (i64 days) exist in Rust. Either compute it client-side from `end - now`, or add it server-side.
- **Three+ competing runtime shapes:** (a) engine `result` nested `current_period`; (b) orchestrator expects `result.current_dasha.{...}` as strings (`birth_blueprint.rs:195`) — only in test mocks, no serializer; (c) vedic-api `VimshottariTimeline.mahadashas[].{lord,start_date,...}`; (d) OpenAPI flat strings. Which one `noesis-vedic-api` actually returns to the web is **unconfirmed** — I found no `*ResponseSchema`/handler that serializes `VimshottariTimeline` into the `EngineOutput.result`. Flag for runtime capture.
- **Lord case mismatch:** vedic-api serializes lords **lowercase** (`types.rs:10`); engine path Title-Case (`engine.rs:145`). Renderer does no normalization → if fed lowercase, color/label lookups keyed on `"Venus"` will miss. Verify casing before keying visuals.
- **`models.rs` carries dead/duplicate types:** `VimshottariChart` (L8, `current_period`+`upcoming_transitions`) vs the `engine.rs` JSON (same field names — this IS the source) — but `models.rs` also defines `Nakshatra` (L124), `UpcomingTransition` (L204, distinct from `Transition` L184), `PlanetaryPeriodQualities` (L214), `PeriodEnrichment` (L224, mirrors enrichment). `TransitionType` is an alias of `TransitionLevel` (L200). Confirm which are live before relying on them.
- **Sub-period lists are not in the engine `result`:** `timeline.mahadashas[]` carries only `antardasha_count` (a number), not the nested Antardasha array (`engine.rs:149`) — so a ring drawing Bhukti arcs for *non-current* Mahadashas has **no data** from this path; only the *current* Maha's Antar/Pratyantar are emitted (under `current_period`). Wave-2 nested rings need the server to expose `sub_periods` (the vedic-api `DashaPeriod.sub_periods` recursion supports it; the engine `result` truncates it).
- **Fixture is a validation reference, not the envelope:** `dasha_reference.json` holds `reference_charts[].expected_mahadashas[]` (`{planet, start_date, end_date, duration_years, is_birth_dasha}`) for cross-checking JHora — useful for ranges/sequence, but it is **not** the API `result` JSON. Don't build the renderer against it.
