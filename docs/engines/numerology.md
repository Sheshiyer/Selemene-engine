# Numerology — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

Six core numbers derived from a person's birth date + full name, each reduced to a single digit (1–9) or a preserved master number (11/22/33), with the full reduction chain and a short meaning.

## 1. Identity
| | |
|---|---|
| `engine_id` | `numerology` (`engine-numerology/src/lib.rs:317`, `engine_id() -> "numerology"`) |
| Domain crate | `crates/engine-numerology/src/lib.rs` (`NumerologyEngine`, L266; compute L273) |
| Domain struct | `crates/engine-numerology/src/types.rs:19` (`NumerologyResult`) + `NumerologyNumber` (L10) |
| Runtime source | **`engine-numerology`** — pure-Rust, no external API; serialized into `EngineOutput.result` (`lib.rs:334`) |
| Renderer | `apps/noesis-web/src/components/engines/Numerology.tsx` |
| Fixtures (real values) | `crates/engine-numerology/tests/reference_validation_tests.rs` (well-known birthdate charts, L81+); inline unit tests `lib.rs:441–688` |
| OpenAPI stub | `noesis-core/src/types.rs:203` (`NumerologyResultSchema` — `life_path_number`/`expression_number`/`core_theme`) |

## 2. Output schema

**Runtime JSON (authoritative — what the engine serializes via `serde_json::to_value(&NumerologyResult)`, `lib.rs:334`):**
```jsonc
{
  // Each of the 6 keys is a NumerologyNumber object (types.rs:10):
  "life_path":     { "value": 7, "is_master": false, "reduction_chain": [16, 7], "meaning": "Analysis, wisdom, introspection" },
  "expression":    { "value": 2, "is_master": false, "reduction_chain": [20, 2], "meaning": "Partnership, diplomacy, sensitivity" },
  "soul_urge":     { "value": 6, "is_master": false, "reduction_chain": [6],     "meaning": "Responsibility, nurturing, harmony" },
  "personality":   { "value": 5, "is_master": false, "reduction_chain": [14, 5], "meaning": "Freedom, change, adventure" },
  "birthday":      { "value": 6, "is_master": false, "reduction_chain": [15, 6], "meaning": "Responsibility, nurturing, harmony" },
  "chaldean_name": { "value": 9, "is_master": false, "reduction_chain": [18, 9], "meaning": "Compassion, completion, universal love" }
}
```
(Values above are the verified `"John"` / day-15 unit-test results — Expression `lib.rs:555`, Soul Urge L562, Personality L568, Birthday L576, Chaldean L583; `reduction_chain` semantics from `reduce_to_core` L109.)

**Domain struct `NumerologyResult` (`types.rs:19`):** six fields, each a `NumerologyNumber` —
`life_path` · `expression` · `soul_urge` · `personality` · `birthday` · `chaldean_name`.
**`NumerologyNumber` (`types.rs:10`):** `value:u32` · `is_master:bool` · `reduction_chain:Vec<u32>` · `meaning:String`.

**OpenAPI stub (`noesis-core/src/types.rs:203`):** `{ life_path_number:i32=7, expression_number:i32=3, core_theme:String="Reflective and analytical" }` — examples only; flat scalars, **none of these field names exist in the real output** (see Open questions).

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `*.value` | **1–9, or 11 / 22 / 33** | enforced by `validate()` (`lib.rs:385–390`); masters via `is_master` (`lib.rs:92`) |
| `*.is_master` | bool | true ⟺ value ∈ {11,22,33}; consistency checked at `lib.rs:394` |
| `*.reduction_chain` | non-empty `Vec<u32>`, length ≥ 1 | first element = raw sum, last = `value`; monotone-decreasing after head until single/master (`reduce_to_core` L109–118) |
| `life_path` | 1–9 / 11 / 22 / 33 | `reduce(year)+reduce(month)+reduce(day)`, then reduce (`lib.rs:186–191`) |
| `expression` | 1–9 / master | full name, Pythagorean A=1…I=9, J=1…R=9, S=1…Z=8 (`lib.rs:20`, `:195`) |
| `soul_urge` | 1–9 / master | vowels only (A/E/I/O/U), Pythagorean (`lib.rs:201`) |
| `personality` | 1–9 / master | consonants only, Pythagorean (`lib.rs:211`) |
| `birthday` | 1–9 / master | day-of-month reduced (`lib.rs:221`); a day like 29→11 stays master |
| `chaldean_name` | 1–9 / master | full name, **Chaldean** mapping (1–8, no 9; `lib.rs:55`) |

**Reduction rule (`reduce_to_core`, `lib.rs:109`):** repeatedly digit-sum until the value is a single digit (≤9) **or** a master (11/22/33), which halts reduction. Masters are **preserved** — never collapsed to 2/4/6. Confirmed: `reduce_to_core(29)=11` (L482), `(22)=22` (L489), `(33)=33` (L496), `(48)=3` via 48→12→3 (L504).
**Life-path master nuance:** components (year/month/day) are each `reduce_to_core`'d *before* summing (`lib.rs:186–188`), so a master month/day (11/22) is kept as 11/22 in the sum — not reduced to 2/4. For `1985-11-22`: year=5, month=11, day=22 → 38 → 11 (master). The engine output is **11** (`reference_validation_tests.rs:93`).

## 4. Component & brand archetype
**Today** (`Numerology.tsx`): a flat **CSS-grid of cells** (`styles.grid`, auto-fit 200px), one per number — label + big gold value (`styles.number`, `var(--gold)`) + optional meaning. **No geometry, no spiral, no SVG.** It reads scalar `number` and text `meaning`/`interpretation`/`description` (L73–74), defaulting to `—`. It also iterates a **hardcoded 5-key list** (`NUMBERS`, L55): `life_path`, `expression`, `soul_urge`, `personality`, `personal_year` — **not** the engine's 6 keys (see Open questions).

**Wave-2 target:** **digit-spiral / compass** — nodes for 1–9 (+ masters 11/22/33) arranged on a spiral or radial compass; the person's core numbers lit (gold `#C5A017` glow), **life-path as the central node** (bioluminescent core). Reduction chains animate as a path drawing inward (Anime.js stroke-dashoffset) from raw sum → final digit. Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `life_path.value` | central node of the spiral/compass; gold core + glow; label = value + meaning |
| `expression.value` | digit node on the spiral; lit at its position 1–9 (or master node) |
| `soul_urge.value` | digit node (inner/vowel ring), lit |
| `personality.value` | digit node (consonant ring), lit |
| `birthday.value` | digit node, lit |
| `chaldean_name.value` | digit node, lit (1–8 range — note: never 9) |
| `*.is_master` | dedicated 11/22/33 node off the 1–9 ring; brighter/double-ring treatment |
| `*.reduction_chain` | draw-in path: raw-sum point → … → final-digit node (line-draw on load) |
| `*.meaning` | tooltip / label text on the active node |

## 6. Dynamics
**One-shot per (name, date).** Pure math, deterministic — `cache_key` is `sha256(name|date)` (`lib.rs:422`), `precision_achieved:"exact"`, `backend:"native-rust"` (`lib.rs:346`). Not live; no baseline/delta. Recompute only when name or birth date changes. On render, lit nodes + reduction paths should animate in once (line-draw); an optional slow core breath on the life-path node is fine. `consciousness_level` is hardcoded **0** here (`lib.rs:344`) — unlike the 0–5 envelope range — so depth-gating of interpretive text is not driven by the engine today.

## 7. Open questions / assumptions
- **Enum variant wraps the STUB, not the real struct (confirmed):** `EngineResultData::Numerology(NumerologyResultSchema)` (`noesis-core/src/types.rs:371`) references the 3-field stub, but `calculate()` emits the full nested `NumerologyResult` into the free-form `EngineOutput.result` (`lib.rs:334`). The stub's `life_path_number`/`expression_number`/`core_theme` field names appear **nowhere** in the real output. Build Wave-2 against `NumerologyResult` (nested `{value,is_master,reduction_chain,meaning}`); treat the stub as docs-only example. ✅ flagged.
- **Renderer reads the wrong shape + wrong keys (confirmed defect-risk):** `Numerology.tsx` expects scalar `result[key].number` (L73) but the engine emits `result[key].value`; so the displayed number is always `—` against real output. It also lists `personal_year` (L60) which the engine **does not produce**, and **omits** `birthday` and `chaldean_name` which it **does**. Renderer needs `.number`→`.value` and the key list realigned to the 6 real keys before it shows anything. ⚠️ likely live bug.
- **Test-comment vs implementation discrepancy (cosmetic):** `reference_validation_tests.rs:96` describes the life-path method as reducing master month/day to 2/4 ("month 1+1=2, day 2+2=4"), but the code keeps masters (month=11, day=22). For `1985-11-22` both arithmetics happen to yield 11, so the assertion passes; the comment's stated *method* is nonetheless inaccurate. No output impact observed.
- **`witness_prompt` is life-path-only:** generated solely from `life_path.value` (`lib.rs:242`); the other five numbers don't influence it. Expected, not a defect.
- **Chaldean range:** `chaldean_name.value` can never be 9 pre-reduction-step (Chaldean map maxes at 8, `lib.rs:55`), but after reduction the *final* value can be 9 (e.g. "John"→18→9, L583). Spiral must still light a 9-node for chaldean.
- **`engine_id` verified** as `"numerology"` (`lib.rs:317`), matching the README table — no inference needed.
