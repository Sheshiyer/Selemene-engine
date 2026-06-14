# Biofield — Data Reference

The energetic field around a person, read **two different ways**: a server-side **birth-data analysis** (`engine-biofield`, currently mock) and a client **live camera capture** (`biofield-web` app + `engine-biofield-capture` + a Python CV sidecar). They share vocabulary (coherence, symmetry, fractal dimension, chakras) but are **separate engines with separate `engine_id`s, separate metric sets, and separate runtime producers** — do not conflate them.

> ⚠️ Read [`README.md` §"Three schema layers"](README.md) first. Biofield additionally has **two engines**, so it has *two* domain structs to keep straight.

## 1. Identity

| | (A) Server analysis | (B) Client live capture |
|---|---|---|
| `engine_id` | **`biofield`** (`engine-biofield/src/engine.rs:36`) | **`biofield-capture`** (`engine-biofield-capture/src/lib.rs:20`; `biofield-domain.ts:1`) |
| Domain struct | `BiofieldAnalysis` (`engine-biofield/src/models.rs:98`) wrapping `BiofieldMetrics` (`models.rs:11`, **5 metrics**) | `BiofieldMetrics` (`biofield-domain.ts:63`, **11 metrics**) + `CompositeScores` (`pip/types.ts:27`, **5 live**) |
| Runtime producer | **mock** today — `generate_mock_metrics` (`engine-biofield/src/mock.rs`); `is_mock_data:true` always (`engine.rs:133,137`). A Vedic path (`VedicBiofieldAnalyzer`, `engine.rs:18`) exists but the standalone engine still returns mock. | **Python CV sidecar** via `noesis-api/src/biofield_client.rs:15` (`BiofieldClient`, `python_biofield_url`) → persisted Reading → `engine-biofield-capture` re-exposes it by `reading_id` (`lib.rs:138`). Live in-browser metrics computed by `MetricsCalculator` (`pip/MetricsCalculator.ts:12`). |
| Renderer | `apps/noesis-web/src/components/engines/Biofield.tsx` | `apps/biofield-web/src/components/BiofieldCosmogram.tsx` + `BiofieldLiveMetrics.tsx` |
| OpenAPI stub | `noesis-core/src/types.rs:258` (`BiofieldResultSchema` — `dominant_element`, `coherence_score 0.72`, `recommended_practice`) — **describes neither struct accurately** | — |
| Brand / design (frozen) | [`docs/design/biofield-web/README.md`](../design/biofield-web/README.md) + sheets `02`,`04`,`05`,`06`,`07`,`08`,`09`,`10` | same — biofield-web is the design's primary target |

**How (A) and (B) relate (the bridge):** in the live app, the WebGL viewer runs `MetricsCalculator.compute()` every 100 ms (`MetricsCalculator.ts:7`) producing per-frame `FrameMetrics` → mapped to 5 `CompositeScores` (`MetricsCalculator.ts:152-172`) that drive the on-screen cosmogram in real time. On **capture**, a frame is uploaded; the **Python sidecar** computes the authoritative **11-field `BiofieldMetrics`** + `QualityAssessment`, persisted as a `biofield-capture` reading. `engine-biofield-capture` later resolves that reading by `reading_id` into the standard `EngineOutput` envelope (`lib.rs:75-88`). The **server `biofield` engine (A) is a wholly separate birth-data path** that never sees the camera — it emits its own 5-metric mock `BiofieldAnalysis`. So: **live PIP `CompositeScores` (5, in-browser, approximate) → captured `BiofieldMetrics` (11, sidecar, authoritative) → `biofield-capture` EngineOutput**; and **`biofield` (A) is parallel, not downstream.**

## 2. Output schema

### 2A. Server `BiofieldAnalysis` (`engine_id: "biofield"`)

**Runtime JSON** the API emits (`engine-biofield/src/engine.rs:165-178`, `serialize_result`):
```jsonc
{
  "metrics": {                       // NESTED — note the renderer reads these at top level (mismatch, §7)
    "fractal_dimension": 1.52,       // 1.0–2.0
    "entropy": 0.55,                 // 0.0–1.0
    "coherence": 0.65,               // 0.0–1.0
    "symmetry": 0.75,                // 0.0–1.0
    "vitality_index": 0.62,          // 0.0–1.0 composite
    "timestamp": "2026-…Z"
  },
  "chakra_readings": [               // 7, Root→Crown (models.rs:70)
    { "chakra": "Root", "chakra_name": "Root", "activity_level": 0.6,  // 0.0–1.0
      "balance": -0.1,              // -1.0..1.0 (neg=left, pos=right; models.rs:48)
      "color_intensity": "…", "location": "…", "element": "Earth" }    // location/element from wisdom.rs
  ],
  "interpretation": "…",            // human-readable string
  "areas_of_attention": ["…"],
  "is_mock_data": true,             // ALWAYS true today (engine.rs:309 validates presence)
  "witness_layer": { … }            // dyad, injected at engine.rs:267-268 (not in BiofieldAnalysis struct)
}
```
`consciousness_level` (0–5) is read from `input.options.consciousness_level` (`engine.rs:236-238`) and rides the **envelope**, not `result`.

**Domain struct** `BiofieldMetrics` (`models.rs:11`) — the 5 fields above + `chakra_readings: Vec<ChakraReading>` + `timestamp`. `ChakraReading` (`models.rs:40`): `chakra`, `activity_level`, `balance`, `color_intensity`. Interpretation thresholds live in `MetricInterpretation` (`wisdom.rs:36`, `optimal_range:(f64,f64)`).

**OpenAPI stub** (`types.rs:258`): `{dominant_element, coherence_score, recommended_practice}` — **none of these fields exist** in `BiofieldAnalysis`. Secondary/illustrative only; ignore for the contract.

### 2B. Client live capture (`engine_id: "biofield-capture"`)

**Authoritative captured metrics** — `BiofieldMetrics` (11 fields, `biofield-domain.ts:63`), produced by the Python sidecar:
```jsonc
{
  "light_quanta_density": 1.2e8,    // ~1e8 scale — NOT 0–1; needs log-normalize (§3, §7)
  "normalized_area": 0.41,          // 0–1
  "average_intensity": 0.58,        // 0–1
  "inner_noise": 0.12,              // 0–1 (lower better)
  "energy_analysis": { "low": …, "medium": …, "high": …, "total": … },  // EnergyBands (biofield-domain.ts:48)
  "entropy_form_coefficient": 0.5,  // 0–1
  "fractal_dimension": 1.5,         // ~1.0–2.0
  "correlation_dimension": 1.3,     // ~1.0–2.0
  "body_symmetry": 0.74,            // 0–1
  "contour_complexity": 0.4,        // 0–1 (scale unverified, §7)
  "pattern_regularity": 0.6         // 0–1
}
```
Wrapped as `BiofieldCaptureResult` (`biofield-domain.ts:126`): `{reading_id, session_id, analysis_version, metrics, quality_assessment, artifacts}`.

**`QualityAssessment`** (`biofield-domain.ts:55`): `sharpness, contrast, noise_level, exposure` (all 0–1, scale unverified) + `sufficient_quality:boolean` (the gate; surfaced in `BiofieldReadingSummary.quality`, `:90`).

**`CompositeScores`** — the **5 live, in-browser** scores (`pip/types.ts:27`), all clamped 0–1 by `MetricsCalculator.ts:152-172`:
| Field | Computed from (`MetricsCalculator.ts`) |
|---|---|
| `lightQuantaDensity` | `avgLum*0.6 + pixelVariance*0.4` (L154) — **0–1 proxy**, not the sidecar's 1e8 value |
| `normalizedArea` | `maskWeight/n` (L157), else 0.5 |
| `bodySymmetry` | combined pixel + face-landmark symmetry (L139,160) |
| `patternRegularity` | `1 - entropyScore` (L163) |
| `overallCoherence` | `lum*0.2 + sym*0.3 + (1-entropy)*0.25 + variance*0.25` (L166-171) |

`FrameMetrics` (`pip/types.ts:19`): raw per-frame `averageLuminance, pixelVariance, symmetryScore, entropyScore, timestamp`.

**Capture lifecycle enums** (`biofield-domain.ts`): session status `active|closed|abandoned` (`:3`); capture state `requested|uploaded|analyzed|persisted|rejected|reprocessed` (`:9`). The `biofield-capture` engine validates output has `reading_id`+`analysis` (`lib.rs:181-201`).

**`biofield-capture` EngineOutput** (`lib.rs:75-88`) wraps the persisted reading: `{available, reading_id, engine_id, created_at, session_id, analysis_version, contract_version, quality_assessment, input, analysis}`. `consciousness_level` comes from the stored reading (`lib.rs:169`).

## 3. Ranges, constraints & invariants

| Field | Range / domain | Source | Notes |
|---|---|---|---|
| `fractal_dimension` (both) | **1.0–2.0**; optimal 1.4–1.7 | `models.rs:13-14`, `wisdom.rs:347`, `mock.rs:49` | <1.3 depleted, >1.8 chaotic (`wisdom.rs:338-345`) |
| `correlation_dimension` (B) | **~1.0–2.0** | `biofield-domain.ts:71` | bound inferred from sibling fractal dim; unverified |
| `entropy` / `entropy_form_coefficient` | **0.0–1.0**; optimal 0.4–0.7 | `wisdom.rs:367` / `biofield-domain.ts:68` | Shannon, normalized |
| `coherence` (A) / `overallCoherence` (B) | **0.0–1.0**; optimal 0.5–0.8 | `wisdom.rs:387` / `pip/types.ts:31` | >0.9 rare (`wisdom.rs:384`) |
| `symmetry` (A) / `body_symmetry` / `bodySymmetry` | **0.0–1.0**; optimal 0.6–0.9 | `wisdom.rs:408` | 1.0 rare; slight asymmetry healthy |
| `vitality_index` (A) | **0.0–1.0**; optimal 0.5–0.8 | `wisdom.rs:428` | composite of the other 4 (`mock.rs:32`) |
| `light_quanta_density` (B) | **~1e8** (NOT 0–1) | `biofield-domain.ts:64`; test uses `42.0` (`engine-biofield-capture/src/lib.rs:320`) | **needs log-normalize before display** (§7) |
| `normalized_area`, `average_intensity`, `inner_noise`, `contour_complexity`, `pattern_regularity` (B) | **0–1** (assumed) | `biofield-domain.ts:65-74` | exact bounds not asserted in TS; `inner_noise` lower=better |
| `energy_analysis` (B) | low/medium/high **bands + total** | `biofield-domain.ts:48` | `EnergyBands`; low/med/high should sum ≈ total (unverified) |
| `chakra.activity_level` (A) | **0.0–1.0** | `models.rs:46` | renderer treats as 0–100 % (§7 mismatch) |
| `chakra.balance` (A) | **-1.0…1.0** | `models.rs:48` | neg=left-dominant, pos=right-dominant |
| 7 chakras (A) | Root→Crown, fixed order | `models.rs:57-66`, `:70` | `Chakra::all()` len 7 (test `models.rs:118`) |
| `QualityAssessment.*` (B) | 0–1 (assumed) + `sufficient_quality:bool` | `biofield-domain.ts:55` | the capture gate |
| `consciousness_level` | **0–5** | envelope (README) | (A) from `options` (`engine.rs:236`); (B) from reading (`lib.rs:169`) |

**Invariants:** (A) `vitality_index` is derived, not independent (`mock.rs:32`). (A) `is_mock_data` always `true` and **must be present** (`engine.rs:309`). (B) capture only persists when `sufficient_quality` passes the gate. `MetricInterpretation.optimal_range.0 < .1` (test `wisdom.rs:539`).

## 4. Component & brand archetype

**Today (A) — `Biofield.tsx`:** one "Overall Coherence" number + 7 horizontal **progress bars** (one per chakra, `CHAKRA_COLORS`/`CHAKRA_NAMES` L80-88). This is **off-brand SaaS dashboard chrome** — progress bars + cards are on the design's *master negative* (`docs/design/biofield-web/README.md` §"Banned"). Wave-2 must replace it.

**Today (B) — `BiofieldCosmogram.tsx`:** already on-brand — a **central mandala** (lotus petals, 8-point compass, spinning dot ring) surrounded by **6 floating geometry panels**, one per axis, each an arc-gauge + bespoke geometric icon, no boxes/cards. Brand tokens hard-coded (`BiofieldCosmogram.tsx:23-27`: Gold `#C5A017`, Emerald `#10B5A7`, Violet `#6B3FA0`, Indigo `#0B50FB`, Parchment `#F0EDE3`). Center shows a state word (`ATTUNING`/`BUILDING`/`OPTIMAL`) keyed off coherence (`:308-309`).

**The 6 cosmogram axes** (`BiofieldCosmogram.tsx:10-16`, `300-307`): **ENERGY, COHERENCE, SYMMETRY, COMPLEXITY, REGULATION, COLOR FIELD** — note these are richer than the frozen sheet's `02-cosmogram-spec.png` label "COH/SYM/LUM/REG" (4 axes); the built component went to 6 (§7).

**Brand archetype (target, both):** **cosmogram + mandala** as the unifying form (frozen in `docs/design/biofield-web`). Per the sheet map (`README.md` §Sheets): `CompositeScores`→Metric Node arc-rings (`04`); the 11 `BiofieldMetrics`→a **Biofield Mandala** of Energy/Geometry/Chaos rings (`05`); fractal/correlation dim + entropy→Fractal/Chaos signature (`06`); `QualityAssessment`→radial quality gauge (`07`); `consciousness_level`→5-state Goethe spectrum (`08`); capture flow→capture compass + state stepper (`09`); reading + baseline deltas→result mandala (`10`). Palette is Goethe polarity (Void `#070B1D`, Witness Violet `#2D0050`, Flow Indigo `#0B50FB`, Sacred Gold `#C5A017`, Coherence Emerald `#10B5A7`, Parchment `#F0EDE3`), Kha/Ba/La arc gradients, bioluminescent (light from within, Void canvas). For **(A)**, Wave-2 should bring `Biofield.tsx` onto the same mandala system (chakras as a 7-ring/7-spoke mandala, not bars).

## 5. Data → visual mapping

**(B) live cosmogram** (`BiofieldCosmogram.tsx`):
| Axis (panel) | Driven by | Visual |
|---|---|---|
| ENERGY | `lightQuantaDensity` (`:302`) | gold hexagon icon scales with value + arc gauge |
| COHERENCE | `overallCoherence` (`:301`) | indigo concentric rings; also sets center state word/color + HRV-style sine wave |
| SYMMETRY | `bodySymmetry` (`:303`) | emerald nested-diamond icon |
| COMPLEXITY | `patternRegularity` (`:304`, **proxy** "until fractal dim lands") | violet 5-point star icon |
| REGULATION | `regulationScore ?? coherence*0.8` (`:305`, **fallback — field absent**, §7) | emerald sine wave |
| COLOR FIELD | `colorBalance ?? normalizedArea` (`:306`, **fallback — field absent**, §7) | 8-petal multi-color lotus |
| center | `overallCoherence` | mandala core: emerald≥0.75 OPTIMAL / indigo≥0.5 BUILDING / gold ATTUNING (`:308-309`) |

**(A) server renderer** (`Biofield.tsx`) — *as currently wired* (see §7 mismatch):
| Field read | Visual |
|---|---|
| `result.coherence ?? result.overall_coherence` (`:95`) | "Overall Coherence" number |
| `result.chakras[].{value\|activation\|energy}` (`:111`) clamped 0–100 | per-chakra horizontal bar % |
| fallback: `result.{root,sacral,…}` top-level keys (`:136-137`) | bar % when `chakras[]` absent |

## 6. Dynamics

- **(A) `biofield`:** one-shot per birth-data input; not live. Recompute on input change. Returns mock today (`is_mock_data:true`). `consciousness_level` (0–5) gates interpretive depth; a `witness_layer` dyad is injected (`engine.rs:267`).
- **(B) live PIP:** **continuous** — `MetricsCalculator` recomputes every **100 ms** (`MetricsCalculator.ts:7`, ~10 fps), throttled/cached between samples; `useLiveMetrics` (`pip/useLiveMetrics.ts:28`) pushes `CompositeScores` into the cosmogram each frame. The mandala animates perpetually (spin 90 s, pulse 3.5 s, `BiofieldCosmogram.tsx:359-362`).
- **(B) capture → persist:** discrete event; gated by `sufficient_quality`. Lifecycle `requested→uploaded→analyzed→persisted` (or `rejected`/`reprocessed`) (`biofield-domain.ts:9`).
- **Baseline / deltas (B only):** `BiofieldMetricDelta` (`biofield-domain.ts:100`) carries `reading_value`, `baseline_value`, **`absolute_delta`** (always) and **`relative_delta?: number | null`** (optional — null when baseline is 0/undefined). A `BiofieldBaselineComparison` (`:108`) bundles `comparison_version` + baseline summary + `deltas[]`. Baselines aggregate N readings (`CreateBiofieldBaselineRequest.reading_ids`, `:148`). Targets the result-mandala sheet (`10`). **(A) has no baseline/delta concept.**

## 7. Open questions / assumptions

- **Two engines, easily conflated (confirmed):** `biofield` (A, 5 metrics, mock, birth-data) ≠ `biofield-capture` (B, 11 metrics, CV sidecar, camera). Different `engine_id`, struct, producer, renderer. Anyone wiring "biofield" must pick the right one. ✅ flagged.
- **`Biofield.tsx` ↔ server `biofield` shape mismatch (confirmed, highest risk):** renderer reads **top-level** `result.coherence` and `result.chakras[].value` (0–100), but `engine.rs:165-174` emits `result.metrics.coherence` (**nested**, 0–1) and `result.chakra_readings[].activity_level` (0–1). As wired, the renderer shows the empty fallback (no coherence row, chakra bars at 0%) against real server output. Either the renderer must read `result.metrics.*`/`chakra_readings[].activity_level` and ×100, or the engine must flatten. **Verify which producer actually feeds `Biofield.tsx` before Wave-2.**
- **`light_quanta_density` normalization (confirmed gap):** sidecar value is **~1e8** (`biofield-domain.ts:64`) but the live `lightQuantaDensity` composite is a **0–1 proxy** (`MetricsCalculator.ts:154`) and the cosmogram clamps 0–1 (`:302`). There is **no log-normalize step** in the read TS. Displaying the raw sidecar `light_quanta_density` on any 0–1 gauge will peg it. **A log-normalize (e.g. `log10(x)/8`) is required and currently missing** — confirm where it should live (sidecar output vs. client mapper).
- **Cosmogram axes vs `CompositeScores` (confirmed):** `BiofieldCosmogram.tsx:305-306` reads `scores.regulationScore` and `scores.colorBalance` via `as any`, but **neither exists** on `CompositeScores` (only 5 fields, `pip/types.ts:27`). REGULATION and COLOR FIELD therefore always render the fallbacks (`coherence*0.8`, `normalizedArea`). Also COMPLEXITY uses `patternRegularity` as an admitted proxy "until fractal dim lands" (`:304`). The built component has **6 axes**; the frozen sheet `02` names **4** ("COH/SYM/LUM/REG"). Reconcile axis set + add the missing composite fields (or feed the cosmogram the full 11-field `BiofieldMetrics`, which carries real `fractal_dimension`/`pattern_regularity`).
- **OpenAPI stub is wrong for both (confirmed):** `BiofieldResultSchema` (`types.rs:258`) `{dominant_element, coherence_score, recommended_practice}` matches **neither** struct. Treat as illustrative; do not generate clients against it.
- **Unverified scales:** `QualityAssessment.{sharpness,contrast,noise_level,exposure}`, `correlation_dimension`, `contour_complexity`, `inner_noise`, and whether `energy_analysis.{low+medium+high}` sums to `total` — bounds inferred, not asserted in source. Confirm against the Python sidecar contract (`biofield-cv/v1`, referenced `engine-biofield-capture/src/lib.rs:318`).
- **`witness_layer` (A)** is injected post-serialize (`engine.rs:268`) and is **not** part of the `BiofieldAnalysis` struct — consumers must read it off `result`, not the typed struct.
- **`consciousness_level` source differs:** (A) from `input.options.consciousness_level` (`engine.rs:236`); (B) from the persisted reading (`lib.rs:169`). Same envelope field, two provenance paths.
