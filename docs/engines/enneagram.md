# Enneagram — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

A typology of nine interconnected personality patterns. Given an assessment (45 answers) **or** a direct type number, returns the primary type + adjacent wing + integration/disintegration lines + center/group taxonomy. The Enneagram is framed throughout as *patterns of perception*, **not** fixed identity.

## 1. Identity
| | |
|---|---|
| `engine_id` | `enneagram` (verified — `ts-engines/src/engines/enneagram/engine.ts:87`, bridge `crates/noesis-bridge/src/lib.rs:273`, renderer router `apps/noesis-web/app/readings/[id]/page.tsx:49`) |
| Crate | **— (no Rust engine; OpenAPI stub + renderer)** |
| Runtime source | **TypeScript engine** `ts-engines/src/engines/enneagram/engine.ts` (`EnneagramEngine`), registered `ts-engines/src/index.ts:25`, served `POST /engines/enneagram/calculate` (`ts-engines/src/server/app.ts:145`). Reached from Rust via `noesis-bridge` `BridgeEngine::enneagram()` (`crates/noesis-bridge/src/lib.rs:267-273`) → HTTP `{TS_ENGINES_URL}/engines/enneagram/calculate` (`lib.rs:368`, default `http://localhost:3001`, `lib.rs:42`). **Not** "unconfirmed" — README.md:58 predates this trace; correct it. |
| Renderer | `apps/noesis-web/src/components/engines/Enneagram.tsx` |
| Wisdom data (all 9 types) | `ts-engines/src/engines/enneagram/wisdom.ts:38` (`ENNEAGRAM_TYPES`), connections `wisdom.ts:419` (`ENNEAGRAM_CONNECTIONS`) |
| Assessment (45 Q + scoring) | `ts-engines/src/engines/enneagram/assessment.ts:39` |
| OpenAPI stub | `crates/noesis-core/src/types.rs:313` (`EnneagramResultSchema` — `core_type:i32` / `wing:String` / `archetype:String`) |
| Fixture (real values) | **none found** — no JSON fixture; the wisdom data IS the source of truth |

## 2. Output schema

**Runtime JSON (authoritative — the shape `Enneagram.tsx` reads).** The TS engine emits a `result` object with a `mode` discriminant and up to three sub-objects (`engine.ts:32-37`). The renderer consumes `result.assessment` and `result.typeAnalysis`:

```jsonc
{
  "mode": "assessment",                  // "assessment" | "lookup" | "questions"  (engine.ts:33)
  "assessment": {                        // present in assessment mode (answers provided)
    "scores": [ { "type": 4, "normalizedScore": 82 }, … ],   // 9 entries, sorted desc
    "primaryType": { "number": 4, "name": "The Individualist", "description": "…" },
    "wing":        { "number": 5, "name": "The Investigator",  "description": "…" },
    "confidence":  0.73,                 // 0–1  (renderer shows Math.round(c*100)%)
    "tritype":     [4, 8, 6],            // optional; one per center, only when confidence ≥ 0.3
    "note": "These results indicate pattern tendencies, not fixed identity…"
  },
  "typeAnalysis": {                      // present in BOTH assessment & lookup modes
    "type": {
      "number": 4, "name": "The Individualist",
      "coreFear": "Having no identity…", "coreDesire": "To find themselves…",
      "coreWeakness": "Envy—…",
      "description": "…", "keyMotivations": ["…"],
      "healthyTraits": ["…"], "averageTraits": ["…"], "unhealthyTraits": ["…"]
    },
    "wings":          [ { "number": 3, "name": "The Achiever" }, { "number": 5, "name": "The Investigator" } ],
    "integration":    { "type": 1, "name": "The Reformer",  "description": "Fours access objectivity…" },
    "disintegration": { "type": 2, "name": "The Helper",    "description": "Fours become clingy…" },
    "center":         "heart",           // gut | heart | head
    "hornevianGroup": "withdrawn",       // assertive | compliant | withdrawn
    "harmonicGroup":  "reactive"         // positive | competency | reactive
  }
}
```
In **lookup mode** (a `type` number was passed, no answers) only `typeAnalysis` is present — no `assessment`, so the renderer's Wing/Confidence/Core-Desire/Core-Fear cells (which read `assessment.*` / `primaryType.*`) go blank; see §7. In **questions mode** `result` is `{ mode, questions: [{id,text,scale}] }` — the 45 assessment items, which the renderer does not handle (falls through to `GenericEngineView`).

**Envelope.** The TS engine returns its own envelope `{ engine_id, result, witness_prompts[], calculated_at, processing_time_ms }` (`engine.ts:207-213`). The bridge maps it into the standard Rust `EngineOutput` (`crates/noesis-bridge/src/lib.rs:439-456`): `witness_prompts[0].prompt` → `witness_prompt`, `consciousness_level` ← `required_phase` (=**1**), `metadata.backend="typescript"`.

**OpenAPI stub** (`types.rs:313`): `{ core_type: 4, wing: "4w5", archetype: "Individualist" }` — examples only. **None of these three field names exist in the runtime JSON** (it uses nested `assessment.primaryType.number` / `wing.number` / `primaryType.name`). The stub is a hand-written illustration, not the contract.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| type `number` | **1–9** integer | validated `engine.ts:216-229`; renderer guards `>=1 && <=9` (`Enneagram.tsx:182`) |
| `wing.number` | **adjacent type only** | must equal `n−1` or `n+1` (wrap 9↔1); enforced `engine.ts:242-255`, `assessment.ts:430-434`. Each type's two wings: `wisdom.ts` `wings:[a,b]` |
| `confidence` | **0–1** | `(top.norm − 2nd.norm)/30`, capped at 1 (`assessment.ts:394-397`); rounded to 2 dp in output (`engine.ts:279`) |
| `scores[].normalizedScore` | **0–100** | `raw/maxPossible*100`, rounded (`assessment.ts:383`); array length 9, sorted desc |
| `tritype` | 3 types, one per center | present only when `confidence ≥ 0.3` (`assessment.ts:405`); centers gut[8,9,1]/heart[2,3,4]/head[5,6,7] (`assessment.ts:440-442`) |
| `center` | `gut` \| `heart` \| `head` | renderer prints `"{center} Center"` (`Enneagram.tsx:200`) |
| `hornevianGroup` | `assertive` \| `compliant` \| `withdrawn` | static per type (`wisdom.ts`) |
| `harmonicGroup` | `positive` \| `competency` \| `reactive` | static per type |
| `integration.type` / `disintegration.type` | **1–9** (or `0`=Unknown fallback) | from `ENNEAGRAM_CONNECTIONS` (`wisdom.ts:419`); `0/"Unknown"` only if lookup fails (`engine.ts:326,333`) |
| assessment input `answers` | **array of 45**, each **1–5** | clamped to 1–5 (`assessment.ts:425`); 5 questions × 9 types; high-signal questions weighted **1.5**, rest **1.0** (`assessment.ts`) |
| `required_phase` (consciousness gate) | **1** | engine refuses if `consciousness_level < 1` (`server/app.ts:160`, `engine.ts:91`) |

**Canonical type table** (`wisdom.ts` — name · center · integration→ · disintegration→ · wings):

| # | Name | Center | Integration → | Disintegration → | Wings |
|---|---|---|---|---|---|
| 1 | The Reformer | gut | 7 | 4 | 9, 2 |
| 2 | The Helper | heart | 4 | 8 | 1, 3 |
| 3 | The Achiever | heart | 6 | 9 | 2, 4 |
| 4 | The Individualist | heart | 1 | 2 | 3, 5 |
| 5 | The Investigator | head | 8 | 7 | 4, 6 |
| 6 | The Loyalist | head | 9 | 3 | 5, 7 |
| 7 | The Enthusiast | head | 5 | 1 | 6, 8 |
| 8 | The Challenger | gut | 2 | 5 | 7, 9 |
| 9 | The Peacemaker | gut | 3 | 6 | 8, 1 |

**Invariants:** integration lines form the two classic cycles — triangle **3→6→9→3** and hexad **1→4→2→8→5→7→1**; disintegration is the reverse traversal of the same edges. The renderer hard-codes these exact paths (`Enneagram.tsx:46-48` `HEXAD`/`TRIANGLE`), independent of the result — so the geometry is always drawn; only the *lit* nodes are data-driven. Wing ⊂ {type's two neighbours}. Tritype picks the top-scoring type from each of the three centers.

## 4. Component & brand archetype
**Today** (`Enneagram.tsx`): a **9-point enneagram SVG** (220×220, `CX/CY=110`, `R=90`; type 9 at top −90°, clockwise at 40° steps, `typeAngle` L27) drawing the outer circle + gold hexad + gold triangle, with 9 numbered nodes — **active type** node enlarged + emerald fill + glow (`#enn-glow`), **wing** node indigo, others faint gold (`Enneagram.tsx:113-131`). Below: a cell grid (Type number · Archetype+Center · Wing · Triad¹ · Confidence) and two callout bars — **Core Desire** (emerald rule) and **Core Fear** (danger rule). Already on-brand (gold `#C5A017`, emerald `#10B5A7`/`rgba(16,181,167)`, indigo `#0B50FB`, sacred-geometry figure, glow filter).

¹ The renderer reads `primary?.triad` (`Enneagram.tsx:177,208`) but the engine emits **`center`**, not `triad` — the Triad cell is **always blank** (see §7).

**Wave-2 target:** the brand archetype is exactly this figure, elevated — the type node lit as the bioluminescent core, the wing adjacent, and **directional arrows** along the classic geometry: an **integration arrow** (growth) and a **disintegration arrow** (stress) drawn from the type node to `typeAnalysis.integration.type` / `.disintegration.type`, animated in on load (Anime.js stroke-dashoffset). Center (gut/heart/head) tints the three-node arc the type belongs to.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `primaryType.number` (1–9) | active node — emerald fill + 1.6 stroke + glow, radius 10 (`Enneagram.tsx:113-118`); also the big `typeNum` cell |
| `wing.number` | wing node — indigo fill/stroke, radius 9 (`Enneagram.tsx:119-124`); Wing cell `n · name` |
| `primaryType.name` / `typeAnalysis.type.name` | Archetype cell (gold) |
| `center` | "{center} Center" sub-label; (target) tints the gut/heart/head node-triad |
| `confidence` (0–1) | Confidence cell `round(c·100)%` |
| `typeAnalysis.type.coreDesire` | emerald-ruled Core Desire callout |
| `typeAnalysis.type.coreFear` | danger-ruled Core Fear callout |
| `integration.type` → | (target) growth arrow along hexad/triangle edge |
| `disintegration.type` → | (target) stress arrow (reverse edge) |
| hexad `1→4→2→8→5→7→1`, triangle `3→6→9→3` | static gold polylines, drawn regardless of data (`Enneagram.tsx:46-48,84-99`) |
| (unused) `triad` | Triad cell — never populated (engine emits `center`) |
| (unused) `scores[]`, `tritype`, `note`, `keyMotivations`, `healthy/average/unhealthyTraits`, `wings[]`, `hornevianGroup`, `harmonicGroup` | computed + returned but **not rendered** today |

## 6. Dynamics
**One-shot per input.** Two trigger shapes: (a) **assessment** — 45 answers in, full `assessment` + `typeAnalysis` out; (b) **lookup** — a `type` (+ optional `wing`) in, `typeAnalysis` only. Deterministic given inputs (witness-prompt wording uses a seeded RNG, `witness.ts` + `seed` param, `engine.ts:136`; the *result* fields are pure lookups/sums). Not live, no recompute cadence, no baseline/delta. On render the figure should animate in once (line-draw of circle/hexad/triangle, then nodes); no perpetual loop. `consciousness_level` must be **≥1** to call at all (gate, §3); deeper levels could progressively reveal the unused traits/motivations blocks.

## 7. Open questions / assumptions
- **⚠️ No Rust engine — but a real TS engine produces it (CALL-OUT).** Unlike the Vedic engines there is **no `engine-enneagram` crate**; the registry README (README.md:43,58) lists it as "stub only / runtime source unconfirmed." That is now **confirmed**: the producer is the **TypeScript `EnneagramEngine`** (`ts-engines/src/engines/enneagram/`), invoked over HTTP through `noesis-bridge` (`crates/noesis-bridge/src/lib.rs:267-273,368,439-456`). README.md row 58 should be updated `unconfirmed → ts-engines`. (Same pattern applies to Tarot, I-Ching, Sacred-Geometry, Sigil-Forge — all have `ts-engines/src/engines/*` implementations.)
- **Stub ≠ contract.** `EnneagramResultSchema` (`types.rs:313`: `core_type`/`wing`/`archetype`) shares **no field names** with the runtime JSON. It is OpenAPI illustration only. Build Wave-2 against the renderer + `engine.ts` shape, not the stub.
- **Renderer reads `triad`, engine emits `center` (confirmed defect).** `Enneagram.tsx:177,208` → `primary?.triad`; nothing in the result sets `triad`. The Triad cell never shows. Either rename the cell to use `typeAnalysis.center`, or have the engine also emit `triad`. Low-risk doc-flag, not fixed here.
- **Lookup mode hides half the UI.** In `mode:"lookup"` there is no `assessment` block, so `primaryType`/`wing`/`confidence`/`coreDesire`/`coreFear` are undefined for the renderer (it reads them off `assessment` / `primaryType`, `Enneagram.tsx:167-180`) → only the figure + (empty) cells render. The renderer's `primary = assessment?.primaryType ?? typeAnalysis?.type` fallback (L169) recovers the type node + archetype + core fear/desire (those live on `typeAnalysis.type` too), but **Wing** and **Confidence** stay blank in lookup mode. Confirm which mode the reading pipeline actually uses.
- **Instinctual variants (sp/so/sx) are absent.** The domain commonly adds a self-preservation/social/sexual subtype; the engine does **not** compute or emit it. Not a gap in this doc — just noting it's out of scope of the current data model. Health *levels* (1–9) are also not numerically emitted; instead the engine gives `healthy/average/unhealthyTraits` lists (`wisdom.ts`).
- **No fixture.** Unlike the Vedic engines there's no reference JSON; correctness rests on the `wisdom.ts` tables + assessment math. The orchestrator's `mock_enneagram_output` (`crates/noesis-orchestrator/src/workflow/synthesis/self_inquiry.rs:648`) is a **test mock**, not a runtime producer — don't mistake it for the contract.
