# Face Reading — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

> **🧊 P1 W1 Frozen (2026-07-17):** Contracts per [P1W1-CONTRACTS-FROZEN.md](../plans/engine-integration/P1W1-CONTRACTS-FROZEN.md) (worktree `.worktrees/T-002-copilot`). EngineInput/Output media, capture lifecycle, image provider, raaga audio locked (T-002+). Cite extraction: goal-understanding.md, resources-and-assets.md, gaps-and-improvements.md. See updated engine-matrix.json. Do not mutate w/o re-freeze. Handoff W2.

Constitutional analysis from facial features, fusing **Ayurvedic dosha** typing, **TCM Five Elements (Mian Xiang)**, and **Western physiognomy** — into a constitution, an elemental balance, personality traits, and per-zone health indicators. **Stub engine:** no image processing yet; output is deterministic-heuristic / mock (`crates/engine-face-reading/src/lib.rs:8-11`).

## 1. Identity
| | |
|---|---|
| `engine_id` | `face-reading` (verified `engine.rs:40`; asserted `engine.rs:412-413`) |
| Domain crate | `crates/engine-face-reading/src/models.rs` (`FaceAnalysis`, L12) + `src/wisdom.rs` (`FaceZoneWisdom`, L12) |
| Runtime source | **`crates/engine-face-reading/src/engine.rs`** — `serialize_analysis()` L143-199 builds the JSON; values from `mock.rs` (`generate_mock_analysis` L15) or `engine.rs` heuristics (`heuristic_from_seed` L45) |
| Renderer | `apps/noesis-web/src/components/engines/FaceReading.tsx` |
| Fixture (real values) | **none** — no `tests/` dir in the crate; runtime values come from the mock/heuristic generators (see §3) |
| OpenAPI stub | `noesis-core/src/types.rs:280` (`FaceReadingResultSchema` — `constitutional_type`/`confidence`/`key_observation`) |

> ⚠️ The renderer is **not yet wired into any router** (no file imports `FaceReading.tsx` except itself; same for the other Wave-2 renderers). It is a draft to build against, not yet mounted.

## 2. Output schema

**Runtime JSON (authoritative — exactly what `serialize_analysis` emits, `engine.rs:147-198`):**
```jsonc
{
  "analysis": {
    "constitution": {
      "primary_dosha":   "vata",          // enum, snake_case (models.rs:168)
      "secondary_dosha": "pitta",         // enum | null  (Option<Dosha>, models.rs:31)
      "tcm_element":     "metal",          // enum, snake_case (models.rs:197)
      "body_type":       "ectomorph",      // enum, snake_case (models.rs:243)
      "descriptions": { "dosha": "...", "element": "...", "body_type": "..." } // prose, engine.rs:154-158
    },
    "personality_indicators": [            // 3–5 items (mock.rs:160) | exactly 2 (heuristic, engine.rs:85-99)
      { "trait_name": "Analytical Thinker", "facial_indicator": "high forehead", "description": "..." }
    ],
    "elemental_balance": {                 // five f64 proportions that SUM TO 1.0 (see §3)
      "wood": 0.21, "fire": 0.18, "earth": 0.23, "metal": 0.19, "water": 0.19,
      "dominant": "earth"                  // ⚠️ injected at serialize (engine.rs:173) — NOT a struct field
    },
    "health_indicators": [                 // 2–3 items (mock.rs:265) | exactly 2 (heuristic, engine.rs:101-115)
      { "zone": "forehead", "associated_organ": "Bladder/Small Intestine", "observation": "..." }
    ],
    "is_mock_data": true                   // true for mock path; false for birth/image heuristic (engine.rs:116, mock.rs:26)
  },
  "notice":       "This is simulated analysis...",   // varies by path (engine.rs:184-188)
  "traditions":   ["Chinese Mian Xiang", "Ayurvedic Face Analysis", "Western Physiognomy"],
  "future_capabilities": ["Real-time facial landmark detection", "Photo-based analysis", ...],
  "disclaimer":   "...not...for medical diagnosis..."  // engine.rs:197
}
```

**Domain struct `FaceAnalysis` (models.rs:12):** `constitution: ConstitutionAnalysis` (L27) · `personality_indicators: Vec<PersonalityTrait>` (L40) · `elemental_balance: ElementalBalance` (L51) · `health_indicators: Vec<HealthIndicator>` (L107) · `is_mock_data: bool`. The struct has **no `confidence`, no `vitality`, no `facial_features`/`face_shape`/`eyes`** — see §7.

**OpenAPI stub:** `{ constitutional_type:"vata-pitta", confidence:0.81, key_observation:"soft jawline with high brow curvature" }` — examples only. **None of these three field names exist in the runtime JSON** (§7).

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `constitution.primary_dosha` | `vata` \| `pitta` \| `kapha` | Ayurvedic dosha, single (models.rs:169-173) |
| `constitution.secondary_dosha` | same enum **\| `null`** | `Option` — present ~60% in mock (`gen_bool(0.6)`, mock.rs:38); never equals primary (mock.rs:39) |
| `constitution.tcm_element` | `wood`\|`fire`\|`earth`\|`metal`\|`water` | mock correlates element to dosha (mock.rs:46-50) |
| `constitution.body_type` | `ectomorph`\|`mesomorph`\|`endomorph` | Western somatotype (models.rs:244-248) |
| `elemental_balance.{wood,fire,earth,metal,water}` | `f64`, **proportions in (0,1)** | **sum = 1.0** (invariant) — *not* 0–100, *not* integers |
| `elemental_balance` per-value | ≈ **0.07–0.57** in practice | mock draws each in `0.1..0.4` then normalizes (mock.rs:176-188); heuristic draws `(seed%97)/100` floored at 0.1 then normalizes (engine.rs:69-76) |
| `elemental_balance.dominant` | element enum | `argmax` of the five (models.rs:77-90); ties → first-max |
| `personality_indicators[]` | 3–5 (mock) or 2 (heuristic) | each `{trait_name, facial_indicator, description}` strings (models.rs:40-47) |
| `health_indicators[].zone` | one of **10** `FaceZone` | `forehead, eyebrows, eyes, nose, cheeks, mouth, chin, ears, jawline, temples` (models.rs:119-130) |
| `health_indicators[]` | 2–3 (mock) or 2 (heuristic) | each `{zone, associated_organ, observation}` strings (models.rs:107-114) |
| `is_mock_data` | `bool` | ⚠️ doc-comment says "Always true" (models.rs:21) but is **false** on birth/image paths (engine.rs:116) |
| `consciousness_level` | **0–5** envelope; engine default **2** | read from `options` (engine.rs:255-260); witness buckets span 0–6 (witness.rs:28-33), exceeding the envelope's 0–5 |
| ~~`confidence`~~ | — | **does not exist** in runtime (stub-only, types.rs:283) |

**`ElementalBalance` sums to 1.0 — confirmed three ways:** `normalize()` divides by the sum (models.rs:93-102); both generators normalize (mock.rs:182-188, engine.rs:76); test `test_elemental_balance_sums_to_one` asserts `(sum-1.0).abs() < 0.001` (mock.rs:324-329). **Render as proportions (0–1), scale ×100 for display.**

**Determinism:** same `seed` → same constitution (engine.rs:472-483; mock.rs:291-303). Three input paths pick the generator (engine.rs:234-252): `image_data` → image heuristic; else `birth_data` → birth heuristic; else → seeded/entropy mock. The two heuristic paths emit fixed 2-trait / 2-indicator lists (engine.rs:85-115); only the mock path uses the rich 14-trait / 13-indicator pools (mock.rs:87-263).

## 4. Component & brand archetype
**Today** (`FaceReading.tsx`): a flat **card grid** — Primary Dosha (+secondary sub), Element, Vitality, Face Shape, Eyes — plus a "Facial Features" key/value list (first 8 entries). Pure text, brand gold (`var(--gold)`) on `var(--field)` cards; **no geometry**. Critically, it reads a *different schema* than the engine emits (§7), so against real output most cells render `—` and the feature list is empty → it falls through to `GenericEngineView` (the guard at `FaceReading.tsx:26` fires only if `analysis`/`constitution`/`features` are all absent; `analysis` **is** present, so it renders the near-empty grid instead).

**Wave-2 target (brand archetype):** **face proportion grid + elemental balance** — a **three-zone face map** (forehead / midface / jaw) drawn with measurement ticks (calliper aesthetic), the active zones lit on the Ba-Arc; **vata / pitta / kapha as arc gauges** (radial sweeps, *not* bars) for the constitution; the **five TCM elements as a pentagon / radar** whose vertices read the `elemental_balance` proportions; `dominant` element = bioluminescent core. Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`. Health-zone observations attach as tooltips on the corresponding face-map zone (data the engine already emits but the current renderer drops).

> Note: the three-zone face map (forehead/midface/jaw) is the **brand** model; the engine's `FaceZone` enum has **10** zones (§3) — map the 10 zones onto the 3 bands (e.g. forehead+eyebrows+temples → upper; eyes+nose+cheeks → mid; mouth+chin+jawline+ears → lower).

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `constitution.primary_dosha` | dominant arc gauge (largest sweep) + center label; hue per dosha |
| `constitution.secondary_dosha` | second arc gauge (smaller); hidden when `null` |
| `elemental_balance.{wood…water}` (5 × 0–1) | pentagon/radar vertices, radius = proportion ×100; or 5 arc gauges |
| `elemental_balance.dominant` | lit vertex + bioluminescent core color |
| `constitution.tcm_element` | accent color of the elemental ring (Wood→green, Fire→red, Earth→gold, Metal→white, Water→indigo) |
| `constitution.body_type` | face-map silhouette proportion (ecto=narrow/long, meso=defined, endo=round/soft) — cf. `dosha_facial_signs()` shapes (wisdom.rs:420-444) |
| `health_indicators[].zone` (10→3 bands) | corresponding face-map zone lit; `observation` + `associated_organ` as tooltip |
| `personality_indicators[]` | labelled callouts pinned to `facial_indicator` region (e.g. "high forehead" → upper band) |
| `is_mock_data` / `notice` | "simulated" badge / disclaimer banner — must stay visible while stubbed |

## 6. Dynamics
**One-shot.** Not live. Recompute when the input changes — `image_data`, then `birth_data`, else `seed` (engine.rs:234-252; `cache_key` keys on seed-or-birth-fragment, engine.rs:352-377). No baseline/delta semantics. On render the arc gauges + radar should animate in once (line-draw / sweep); an optional slow core breath on the `dominant` vertex is fine — no perpetual loop. `consciousness_level` (default 2) gates only the **witness prompt** depth (foundational 0-2 / awareness 3-4 / integration 5-6, witness.rs:28-33), not the `result` payload — every level emits the same analysis fields. Engine `required_phase = 1` (engine.rs:218-220): needs self-reflection capacity.

## 7. Open questions / assumptions
- **ElementalBalance normalization — RESOLVED: sums to 1.0** (proportions, not percentages, not integers). Cited three ways in §3. Multiply ×100 only for display.
- **Renderer ↔ engine schema mismatch (CONFIRMED, the big one):** `FaceReading.tsx` reads `analysis.facial_features` / `face_shape` / `eyes` / `eye_type` / `vitality_score` and `constitution.dosha` + top-level `element` (tsx L22-33) — **none of which the engine emits.** The engine emits `constitution.primary_dosha`, nested `constitution.tcm_element` (no top-level `element`), and `personality_indicators` / `health_indicators` / `elemental_balance` (which the renderer **ignores entirely**). The renderer was clearly built against an anticipated MediaPipe shape, not the current stub. **Wave-2 must rebuild the renderer against the §2 runtime JSON** (or the engine must change). Until then it shows a near-empty grid.
- **Stub ↔ runtime field mismatch:** OpenAPI `FaceReadingResultSchema` (types.rs:280-287) advertises `constitutional_type` ("vata-pitta"), `confidence` (0.81), `key_observation` — **all three absent from runtime.** Runtime splits dosha into `primary_dosha` + `secondary_dosha`, has **no confidence score anywhere**, and has no single `key_observation` (closest is `personality_indicators[].facial_indicator`). If the API contract promises `confidence`, the engine does not produce it. Treat the stub as aspirational naming only.
- **`is_mock_data` semantics:** doc-comment "Always true for stub implementation" (models.rs:21) contradicts the code, which sets it **false** for birth-data and image-data paths (engine.rs:116) and `true` only for the pure mock path (mock.rs:26). The "non-mock" paths are still heuristic, not real CV — the `false` flag could mislead the UI into hiding the simulated-data disclaimer. Confirm intended UX.
- **`elemental_balance.dominant` is serialization-only** (engine.rs:173) — present in JSON, absent from the `ElementalBalance` struct and from the OpenAPI stub. Safe to read from runtime JSON; don't expect it in any Rust-typed mirror.
- **`witness.rs` wisdom is under-used:** `FaceZoneWisdom` (wisdom.rs:12) and `dosha_facial_signs()` (wisdom.rs:420) carry rich per-zone TCM/Ayurvedic/emotional correlations and per-dosha facial shapes that **never reach `result`** (only used to phrase witness prompts). A richer renderer could surface these, but it would need the engine to include them in the payload first.
- **No fixture:** unlike Panchanga, there is no reference JSON to diff against; the §2 example is hand-derived from `serialize_analysis`. Generating a `tests/` fixture (e.g. `seed=42`) would give Wave-2 a stable target.
