# Biorhythm — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

Classic biorhythm: sine cycles seeded at birth, sampled at a target date. Six cycles (physical/emotional/intellectual + intuitive/aesthetic/spiritual), three composites (mastery/passion/wisdom), critical-day detection, an optional forecast window, and an optional two-person compatibility block.

## 1. Identity
| | |
|---|---|
| `engine_id` | `biorhythm` (`crates/engine-biorhythm/src/lib.rs:150-151`; asserted in tests L860) |
| Domain crate | `crates/engine-biorhythm/src/calculator.rs` (math + `BiorhythmResult` L30, `CycleResult` L50, `ForecastDay` L62); `lib.rs` (engine impl, `CompatibilityResult` L63, `CycleCompatibility` L51) |
| Runtime source | **`engine-biorhythm` crate** — pure native Rust, `backend: "native-rust"` (`lib.rs:279`). No vedic-api path. Output is `serde_json::to_value(BiorhythmResult)` (`lib.rs:252`) with `compatibility` spliced in when requested (L256-268). |
| Renderer | `apps/noesis-web/src/components/engines/Biorhythm.tsx` |
| Fixture (real values) | **none found** — no `crates/engine-biorhythm/tests/` dir; values below are derived from the math in `calculator.rs`. |
| OpenAPI stub | `crates/noesis-core/src/types.rs:214` (`BiorhythmResultSchema` — flat `physical`/`emotional`/`intellectual: f64`, examples only) |

> ⚠️ **Note the duplicate `BiorhythmResult` / `CycleResult` / `ForecastDay` definitions.** `src/types.rs` (the file the task pointed at) declares its own copies, but `lib.rs` re-exports the **`calculator.rs`** versions (`lib.rs:9-13`) and serializes those. `src/types.rs` also omits the `aesthetic`/`spiritual` fields that the live struct carries — it appears to be **stale/unused**. The runtime contract is **`calculator.rs`**. Flagged in §7.

## 2. Output schema

**Runtime JSON (authoritative — `serde` of `calculator::BiorhythmResult`, `lib.rs:233-248`). Illustrative values from the math, not a captured fixture:**
```jsonc
{
  "days_alive": 12950,
  "target_date": "2026-06-04",                 // YYYY-MM-DD, = input.current_time.date_naive() (lib.rs:173)
  "physical":     { "value": 0.824, "percentage": 91.2, "phase": "Rising",   "days_until_peak": 3, "days_until_critical": 8,  "is_critical": false, "cycle_day": 4 },
  "emotional":    { "value": 0.417, "percentage": 70.9, "phase": "Rising",   "days_until_peak": 5, "days_until_critical": 12, "is_critical": false, "cycle_day": 5 },
  "intellectual": { "value": -0.152,"percentage": 42.4, "phase": "Falling",  "days_until_peak": 25,"days_until_critical": 3,  "is_critical": false, "cycle_day": 17 },
  "intuitive":    { "value": 0.0,   "percentage": 50.0, "phase": "Critical", "days_until_peak": 9, "days_until_critical": 1,  "is_critical": true,  "cycle_day": 0 },
  "aesthetic":    { "value": 0.6,   "percentage": 80.0, "phase": "Rising",   "days_until_peak": 4, "days_until_critical": 10, "is_critical": false, "cycle_day": 8 },
  "spiritual":    { "value": -0.7,  "percentage": 15.0, "phase": "Low",      "days_until_peak": 31,"days_until_critical": 6,  "is_critical": false, "cycle_day": 30 },
  "mastery":  66.8,                             // (physical.percentage + intellectual.percentage) / 2   (lib.rs:191)
  "passion":  81.1,                             // (physical.percentage + emotional.percentage)   / 2   (lib.rs:192)
  "wisdom":   56.7,                             // (emotional.percentage + intellectual.percentage)/ 2   (lib.rs:193)
  "critical_days": ["2026-06-05", "2026-06-09"],// upcoming dates any PRIMARY cycle crosses zero (calculator.rs:185)
  "overall_energy": 58.2,                       // equal-weighted mean of all SIX .percentage values (lib.rs:196-202)
  "forecast": [                                 // present unless forecast_days <= 0; default 7 (lib.rs:205-219)
    { "date": "2026-06-05", "days_alive": 12951, "physical": 96.0, "emotional": 78.1,
      "intellectual": 38.7, "intuitive": 41.2, "aesthetic": 83.0, "spiritual": 12.0,
      "overall_energy": 58.2 }                  // ForecastDay: per-cycle values are PERCENTAGES 0..100 (calculator.rs:217-233)
  ],
  "compatibility": {                            // present ONLY if options.partner_birth_date supplied (lib.rs:222-230)
    "birth_date_a": "1990-01-01", "birth_date_b": "1992-08-14", "target_date": "2026-06-04",
    "physical":     { "score": 73.4, "period": 23.0, "days_diff": 956 },
    "emotional":    { "score": 12.1, "period": 28.0, "days_diff": 956 },
    "intellectual": { "score": 88.0, "period": 33.0, "days_diff": 956 },
    "intuitive":    { "score": 45.0, "period": 38.0, "days_diff": 956 },
    "overall": 57.8                             // mean of physical+emotional+intellectual ONLY (lib.rs:120)
  }
}
```

**Domain struct (`calculator::CycleResult`, L50-58)** — each cycle carries **two** scale fields plus phase metadata:
`value:f64` (raw sine −1..1) · `percentage:f64` (0..100) · `phase:String` · `days_until_peak:i64` · `days_until_critical:i64` · `is_critical:bool` · `cycle_day:i64`.

**OpenAPI stub (`types.rs:214`):** `{ physical:f64=82.4, emotional:f64=41.7, intellectual:f64=-15.2 }` — flat, 3-of-6 cycles, no `CycleResult` nesting, examples on a **signed −100..100** scale that the live struct never emits. Docs only; **do not build against it.**

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `<cycle>.value` | **−1.0 … +1.0** | raw `sin(2π·days_alive/period)` (calculator.rs:79-81); `validate()` rejects outside [−1,1] (lib.rs:315) |
| `<cycle>.percentage` | **0.0 … 100.0** | `(value+1)/2·100` (calculator.rs:84-86) — **50 = zero crossing, 0 = trough, 100 = peak.** NOT a signed %. `validate()` rejects outside [0,100] (lib.rs:322) |
| `<cycle>.phase` | `Peak`\|`Low`\|`Rising`\|`Falling`\|`Critical` | `Critical` wins if within 1 day of a crossing; else Peak `value>0.95` / Low `<−0.95` / sign of cosine (calculator.rs:89-106) |
| `<cycle>.days_until_peak` | **1 … period** | next sin=1 (phase 0.25); never 0 (wraps to `period`) (calculator.rs:110-124) |
| `<cycle>.days_until_critical` | **1 … period** | next zero crossing (phase 0.0 or 0.5) (calculator.rs:128-151) |
| `<cycle>.is_critical` | bool | `|value| < |sin(2π·1/period)|` — within ~1 day of a crossing (calculator.rs:154-161) |
| `<cycle>.cycle_day` | **0 … period−1** | `days_alive.rem_euclid(period as i64)` (calculator.rs:171) |
| Cycle periods (days) | P **23** · E **28** · I **33** · intuitive **38** · aesthetic **43** · spiritual **53** | `calculator.rs:14-19` (constants). All six are computed (lib.rs:183-188). |
| `mastery`/`passion`/`wisdom` | **0 … 100** | composites of `.percentage` (lib.rs:191-193); validate enforces [0,100] (lib.rs:338) |
| `overall_energy` | **0 … 100** | mean of all six `.percentage` (lib.rs:196-202) |
| `days_alive` | **≥ 0** | `target − birth` in days; negative ⇒ hard error (lib.rs:175-180) |
| `critical_days` | array of `YYYY-MM-DD`, len ≤ `forecast_days` | only **physical/emotional/intellectual** crossings counted (calculator.rs:195-197) |
| `forecast[*]` cycle fields | **0 … 100** (percentages) | `ForecastDay.physical…spiritual` are `to_percentage(...)` (calculator.rs:217-222) — different scale from `CycleResult.value` |
| `compatibility.<cycle>.score` | **0 … 100** | `50·(1+cos(2π·days_diff_mod_period/period))`; 100 = same phase, 0 = half-period apart (lib.rs:81-94) |
| `compatibility.overall` | **0 … 100** | mean of physical+emotional+intellectual **only** (intuitive excluded despite being present) (lib.rs:120) |

**Invariants.** All cycles seeded at `days_alive=0` ⇒ every `value=0`, `percentage=50` at birth (tests L415, L431). Cosine derivative drives Rising/Falling. The six cycles are mutually independent (different coprime-ish periods); only `days_alive` couples them. `consciousness_level` is hard-coded `0` (lib.rs:276) and `required_phase=0` (lib.rs:158-160) — ungated.

## 4. Component & brand archetype
**Today** (`Biorhythm.tsx`): a stat-cell grid (one cell per cycle, value + optional peak/trough sub-labels) above a **600×180 SVG line chart** — a 60-day window (30 past / 30 future, `TOTAL_DAYS`/`PAST_DAYS` L43-44), horizontal zero axis, day gridlines at −30…+30, ±100 amplitude labels, a dashed **"today" vertical marker** at x=300 (L262-270), per-cycle dots at today's value (L273-288), and one `<polyline>` sine per cycle (L248-259). Colors: physical `#ef6b73` (off-brand coral), emotional `var(--c-indigo,#0B50FB)`, intellectual `var(--c-gold,#C5A017)` (`CYCLES` L32-36). **Only 3 of the 6 cycles are drawn** (intuitive/aesthetic/spiritual ignored). The wave is reconstructed client-side from a single current value via `asin` phase inference (L60-89) — the engine's `cycle_day`/`phase` are not used.

**Wave-2 target:** three-wave (extensible to six) sine overlay on Void `#070B1D`, each cycle a luminous sinusoid in its brand hue across the time window, a bright **today marker** at the zero-line intersection, glow on the present-value nodes; brand palette (Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`) — repaint physical to Emerald or Violet (drop the coral) and assign the three secondary cycles their own hues. Waves animate in via stroke-dashoffset on load.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `<cycle>.value` (intended driver) | y-position of the sine + today-dot; **but renderer expects −100..100, struct emits −1..1** — see §7 |
| `<cycle>.percentage` (0..100) | unused today; the correct signed amplitude is `(percentage−50)/50` → −1..1 |
| cycle `period` (23/28/33…) | wavelength of each `<polyline>` (`CYCLES[].period`, L82) |
| `is_critical` / zero crossing | (target) flagged where a wave meets the zero axis; today-marker emphasis |
| `phase` (Rising/Falling/Peak/Low) | (target) node icon / arrow; today inferred from `asin` branch instead (L74) |
| `days_until_peak` / `_critical` | (target) "▲ Peak / ▼ Trough" sub-labels — renderer reads non-existent `next_peak`/`next_trough` (L145-146), so these are **always hidden** today |
| `critical_days[]` | (target) tick marks on the future half of the axis |
| `forecast[*]` | (target) the future half could be drawn from real forecast points instead of pure client-side sine |
| `today` marker | fixed dashed vertical at x=`TODAY_X`=300 (L262-270) |

## 6. Dynamics
**One-shot per (birth_date, target_date).** `target_date = input.current_time.date_naive()` (lib.rs:173) — date-granular, so the result only changes **once per day** (or when birth date / `forecast_days` / `partner_birth_date` options change). Not live/sub-day. `cache_key` hashes engine_id + birth date + target date + `forecast_days` (lib.rs:357-377) — note compatibility/partner is **not** in the cache key. The renderer recomputes the whole 60-day curve from the single current value on each render; nothing animates perpetually today (Wave-2 adds a one-time line-draw). No baseline/delta semantics.

## 7. Open questions / assumptions
- **🔴 AMPLITUDE-SCALE MISMATCH (confirmed defect, the headline risk).** The renderer treats `result.<cycle>.value` as **−100…+100**: it clamps `val/100` (Biorhythm.tsx:66), plots `sin(asin(val/100))·amplitude`, prints `{val}%`, and labels the axis ±100. But the engine emits `value` on **−1.0…+1.0** (calculator.rs:79-86; enforced [−1,1] at lib.rs:315). So in production `val/100 ≈ ±0.008` → `asin ≈ 0` → **all waves render as near-flat lines on the zero axis and the stat cells read "+1% / −0% / +0%"** instead of the intended ±80%. The correct signed amplitude already exists as `(percentage − 50)/50`. **Fix path (renderer-side, do NOT change in this docs task):** either (a) read `percentage` and map `(percentage−50)/50` → −1..1, then plot `·amplitude` directly (drop the `/100` and `asin`); or (b) multiply `value` by 100 before feeding the existing code. Option (a) is cleaner — `percentage` is the field meant for display.
- **🔴 Renderer reads fields the engine never emits.** `next_peak`, `next_trough` (L145-146) and `phase_days`/`phaseDays`/`day_in_cycle` (L147) don't exist on `CycleResult`; the engine provides `days_until_peak`, `days_until_critical`, `cycle_day`, `phase`. Consequence: peak/trough sub-labels are always hidden, and phase is inferred via `asin` (principal/ascending branch only, L74) rather than read from `cycle_day`/`phase`. To draw the *correct* curve direction the renderer should use `cycle_day` as the phase seed: `phase = 2π·cycle_day/period`.
- **🟠 Six cycles emitted, three rendered.** Engine computes physical/emotional/intellectual **+ intuitive(38) + aesthetic(43) + spiritual(53)** (calculator.rs:14-19, lib.rs:183-188) and they're all in `result`; `Biorhythm.tsx` `CYCLES` lists only the first three (L32-36). Also the composites `mastery`/`passion`/`wisdom`, `overall_energy`, `critical_days`, `forecast`, and `compatibility` are all unrendered. README row ("three-wave sine overlay (23/28/33 d)") and the `lib.rs` module doc-comment (L1-5, says "three composite cycles") both undercount the live output — decide whether Wave-2 surfaces all six + composites or stays at three.
- **🟠 OpenAPI stub is triply wrong** (`types.rs:214`): flat (no `CycleResult` nesting), only 3 of 6 cycles, and its example values (82.4 / 41.7 / −15.2) are on the renderer's phantom signed −100..100 scale — matching neither `value` (−1..1) nor `percentage` (0..100). Update the stub to nested `CycleResult` with realistic values, or document it as illustrative-only.
- **🟡 Stale `src/types.rs`.** `crates/engine-biorhythm/src/types.rs` declares `BiorhythmResult`/`CycleResult`/`ForecastDay`/`CompatibilityResult` but `lib.rs` re-exports and serializes the **`calculator.rs`** copies (lib.rs:9-13). The `types.rs` copies lack `aesthetic`/`spiritual` and are not the runtime contract; appears dead. Confirm nothing imports `engine_biorhythm::types::*` before relying on / removing it.
- **🟡 `compatibility.overall` excludes intuitive.** The intuitive score is computed and serialized but the `overall` average uses only the three primary cycles (lib.rs:120); the doc-comment at lib.rs:74-75 says "four available cycles" — comment/code disagree. Cosmetic; flag for the consumer.
- **🟡 No fixture exists** for biorhythm (no `tests/` dir; only in-crate `#[cfg(test)]` unit tests). Example JSON above is derived from `calculator.rs` math, not captured output. A golden fixture would pin the contract for the web build.
