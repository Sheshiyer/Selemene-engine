# Gene Keys — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

The 64 Gene Keys derived from the Human Design gates: each gate becomes a contemplative archetype with a **Shadow → Gift → Siddhi** frequency spectrum and a **Line (1–6)**, organised into sequences. This engine emits the **Activation Sequence** (the four prime gifts) plus every active key enriched with frequency data.

## 1. Identity
| | |
|---|---|
| `engine_id` | `gene-keys` (verified — `engine.rs:36`, and dispatcher `app/readings/[id]/page.tsx:38` `case "gene-keys"`) |
| Domain crate | `crates/engine-gene-keys/src/` (`GeneKeysChart`, `models.rs:172`) |
| Runtime source | **`crates/engine-gene-keys/src/engine.rs`** — `serialize_chart` (L222) is the literal `result` JSON; depends on `engine-human-design` to derive gates |
| Renderer | `apps/noesis-web/src/components/engines/GeneKeys.tsx` |
| Fixture (real values) | `crates/engine-gene-keys/tests/reference_charts.json` (8 charts; HD gates → expected sequence pairs) |
| OpenAPI stub | `noesis-core/src/types.rs:236` (`GeneKeysResultSchema` — `activation_sequence`/`venus_sequence`/`pearl_sequence`, all `String`) |

> ⚠️ This engine is the strongest example in the repo of the **three-layer mismatch** from the [README](README.md#️-three-schema-layers-read-this-before-trusting-any-one-source). Runtime, renderer, and stub describe three *incompatible* shapes. Read §7 before building.

## 2. Output schema

**Runtime JSON (authoritative — exactly what `serialize_chart` emits, `engine.rs:248-257`):**
```jsonc
{
  "activation_sequence": {                 // an OBJECT of four gate-pairs (NOT a string, NOT spheres)
    "lifes_work": [17, 18],                // [personality_sun, personality_earth]
    "evolution":  [45, 26],                // [design_sun,      design_earth]
    "radiance":   [17, 45],                // [personality_sun, design_sun]
    "purpose":    [18, 26]                 // [personality_earth, design_earth]
  },
  "active_keys": [                         // one per HD gate activation (models.rs:54 GeneKeyActivation)
    {
      "key_number": 17,                    // 1–64
      "line": 4,                           // 1–6
      "source": "PersonalitySun",          // Debug-formatted enum, e.g. "DesignEarth" (engine.rs:231)
      "name": "The Eye",                   // present only if gene_key_data was attached
      "shadow": "Opinion", "gift": "Far-Sightedness", "siddhi": "Omniscience"
    }
    // …
  ],
  "frequency_assessments": [               // assess_frequencies(), frequency.rs:82 — per active key
    {
      "gene_key": 17, "name": "The Eye",
      "shadow": "Opinion", "gift": "Far-Sightedness", "siddhi": "Omniscience",
      "shadow_description": "…", "gift_description": "…", "siddhi_description": "…",
      "suggested_frequency": "Gift",       // "Shadow"|"Gift"|"Siddhi"|null — gated by consciousness_level
      "recognition_prompts": { "shadow": ["…"], "gift": ["…"], "siddhi": ["…"] }
    }
  ]
}
```
The fixture only asserts the **`activation_sequence` pairs** (`reference_charts.json` `expected.{lifes_work,evolution,radiance,purpose}`); it does not capture `active_keys`/`frequency_assessments` (the engine attaches those at runtime from `wisdom.rs`).

**Domain structs (`engine-gene-keys/src/models.rs`):**
- `GeneKeysChart` (L172) = `{ activation_sequence: ActivationSequence, active_keys: Vec<GeneKeyActivation> }`. **Note: the struct has no `frequency_assessments` field** — it is added only by `serialize_chart` (engine.rs:246,256), so the runtime JSON is a superset of the struct.
- `ActivationSequence` (L142) = four `(u8, u8)` tuples `lifes_work / evolution / radiance / purpose`, built by `from_activations(personality_sun, personality_earth, design_sun, design_earth)` (L155).
- `GeneKeyActivation` (L54) = `{ key_number:u8, line:u8, source:ActivationSource, gene_key_data:Option<GeneKey> }`.
- `GeneKey` (L8) = `{ number:u8, name, shadow, gift, siddhi, shadow_description, gift_description, siddhi_description, programming_partner:Option<u8>, codon:Option<String>, amino_acid:Option<String>, physiology:Option<String>, keywords:Vec<String>, life_theme:Option<String> }` — but `serialize_chart` only forwards `name/shadow/gift/siddhi` into `active_keys`; the descriptions reach the wire **only** via `frequency_assessments`.
- `ActivationSource` (L67) = enum of 26 named planet×{Personality,Design} variants + `Other(String)`; serialized via Rust `Debug` (`"PersonalitySun"`, …), **not** snake_case.
- `FrequencyAssessment` (`frequency.rs:25`) and `RecognitionPrompts` (`frequency.rs:60`) — shapes shown above.

**OpenAPI stub (`types.rs:236`, docs only):** `{ activation_sequence:String, venus_sequence:String, pearl_sequence:String }`. **All three fields are wrong/absent at runtime:** `activation_sequence` is an *object*, not a string; `venus_sequence` and `pearl_sequence` **are never produced** (see §7).

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `key_number` / `gene_key` / sequence pair members | **1–64** | 1:1 with HD gates (`models.rs:9,57`); fixture covers boundaries 1, 2, 63, 64 (Ref 3) |
| `line` | **1–6** | hexagram line (`models.rs:58`); derived from the gate's positional fraction in the HD engine |
| `activation_sequence.*` | `[u8; 2]` | each is a **pair of gate numbers**, not a single key; see derivation below |
| `source` | one of 26 variants + `Other` | `Personality*` / `Design*` × {Sun,Earth,Moon,N/S Node,Mercury,Venus,Mars,Jupiter,Saturn,Uranus,Neptune,Pluto} (`models.rs:68-95`) |
| `suggested_frequency` | `Shadow` \| `Gift` \| `Siddhi` \| `null` | from `consciousness_level`: 0–2→Shadow, 3–4→Gift, 5–6→Siddhi, else null (`frequency.rs:116-123`) |
| `frequency_assessments[].*_description` | non-empty strings | archetypal copy from `wisdom.rs` (`get_gene_key`) |
| `recognition_prompts.{shadow,gift,siddhi}` | `string[]` | self-inquiry prompts, not predictions (`frequency.rs:5`) |

**Sequence derivation invariant** (`ActivationSequence::from_activations`, `models.rs:155-167`, confirmed by every fixture row):
```
lifes_work = (personality_sun,   personality_earth)
evolution  = (design_sun,        design_earth)
radiance   = (personality_sun,   design_sun)
purpose    = (personality_earth, design_earth)
```
So the four sequences are the **2×2 grid** of {Personality,Design}×{Sun,Earth}: rows = the two activation pairs, columns = Sun-axis (Radiance) and Earth-axis (Purpose). The four pairs reuse exactly four distinct gates — they are not independent.

**Frequency-band semantics** (the spectrum, not a numeric range): every key has three named bands — **Shadow** (reactive/unconscious), **Gift** (conscious/constructive), **Siddhi** (transcendent) — `models.rs:15-23`, mirrored by `Frequency` enum `frequency.rs:12`. There is **no birth-determined frequency**: the module is explicit that frequency depends on lived consciousness and must be self-identified (`frequency.rs:3-5`). `required_phase()` for this engine is **2** (`engine.rs:278`) — deeper than HD.

## 4. Component & brand archetype
**Today** (`GeneKeys.tsx`): a responsive **grid of "Sphere" cards** (`sphereGrid`, `auto-fit minmax(240px,1fr)`). Each card shows a name, an optional `Gate {n}`, and a **`<Spectrum>`** — a fixed three-stop gradient bar (Shadow `#C65D3B` terracotta → Gift `#10B5A7` emerald → Siddhi `#C5A017` gold, `SPECTRUM_GRADIENT` L5) with three labelled tick columns. It renders the four named spheres `["Life's Work","Evolution","Radiance","Purpose"]` (L109) and an optional **Programming Partner** line. **It is purely text + a static bar — no rings, no per-key geometry, no line (1–6) shown.** Colours already match brand (terracotta/emerald/gold).

**Critical:** the renderer reads `activation.spheres` / per-sphere objects with `{gate, shadow, gift, siddhi}` **strings** — a shape the engine **does not emit** (see §7). Against real runtime JSON every card currently shows gate `—` and Shadow/Gift/Siddhi `—`.

**Wave-2 target — hologenetic sequence rings.** The three sequences as **constellation paths** between spheres on a circle: each Gene Key a **node** plotted by gate (1–64 → angle), with edges drawn as the sequence paths. Activation Sequence = the four prime nodes joined into the 2×2 grid (Life's Work–Evolution rows, Radiance–Purpose columns). Each node renders its **Shadow→Gift→Siddhi** as a short radial gradient spoke (reuse the existing terracotta→emerald→gold ramp), with the **line 1–6** as a 6-tick subdivision on the node. Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`. Paths drawn-in on load (Anime.js `stroke-dashoffset`). **Venus and Pearl sequences are aspirational** (named in the stub) — do not render rings for them until the engine emits them (§7).

## 5. Data → visual mapping
| Field | Visual (today → Wave-2) |
|---|---|
| `activation_sequence.lifes_work[0]` (= personality_sun gate) | sphere/node "Life's Work"; gate label → node angle on the 64-ring |
| `activation_sequence.evolution` pair | "Evolution" node(s); edge along the Design row |
| `activation_sequence.radiance` pair | "Radiance" node(s); edge along the Sun column |
| `activation_sequence.purpose` pair | "Purpose" node(s); edge along the Earth column |
| `frequency_assessments[].{shadow,gift,siddhi}` | the three labels under each spectrum bar (today the renderer reads sphere-level strings instead — broken; should read these) |
| `frequency_assessments[].suggested_frequency` | (target) which band of the node's spectrum spoke is lit |
| `active_keys[].line` (1–6) | **not rendered today**; Wave-2: 6-tick subdivision on the node |
| `active_keys[].source` | (target) node grouping / label (Personality vs Design ring) |
| `GeneKey.programming_partner` | `Programming Partner` text line (`partner`, GeneKeys.tsx:206,245) — **note:** engine `serialize_chart` does **not** emit `programming_partner`, so this line is always hidden at runtime (§7) |

## 6. Dynamics
**One-shot per birth chart.** Computed from `birth_data` (via the HD engine, `engine.rs:284-295`) or directly from supplied `hd_gates` options (`engine.rs:473`). Not live; recompute only if birth data changes. The **only** dynamic input is envelope `consciousness_level` (0–5; engine validates ≤6, `engine.rs:386`): it sets `suggested_frequency` per key (`frequency.rs:116`) — i.e. raising the user's level shifts highlighted bands Shadow→Gift→Siddhi without recomputing gates. `witness_prompt` is generated from the chart + level (`witness.rs`, via `generate_witness_prompt`, engine.rs:322). On render, sequence paths/nodes should animate in once; an optional slow core breath only.

## 7. Open questions / assumptions
- **Renderer ⟷ runtime mismatch (confirmed, highest priority).** `GeneKeys.tsx` expects `activation_sequence.spheres[]` or per-sphere **objects** `{name,gate,shadow,gift,siddhi}` (L205-242). The engine emits `activation_sequence` as four **gate-pair arrays** (`engine.rs:248-254`) and puts the human-readable shadow/gift/siddhi under **`active_keys[]`** and **`frequency_assessments[]`** — which the renderer **never reads**. Net effect against real output: `spheres` is empty → fallback path reads `activation.lifes_work` = `[17,18]` → `obj()` coerces the array to `{}` → every card shows gate `—` and Shadow/Gift/Siddhi `—`. **Build decision needed:** either (a) renderer maps gate pairs → look up names from `frequency_assessments`, or (b) engine adds a `spheres`/hologenetic block. Recommend (a) — runtime is the contract; do not invent (b) without an engine change.
- **`venus_sequence` / `pearl_sequence` are stub-only — never produced.** The OpenAPI example (`types.rs:239-242`) and the domain prompt both name three sequences (Activation, Venus, Pearl), but the engine computes **only the Activation Sequence**. No Venus/Pearl code exists in the crate (grep: only `Venus` *planet* references). The `GeneKeysInfo.sequences: Vec<String>` field (`models.rs:194`) is metadata loaded from `archetypes.json` and is **not** part of `GeneKeysChart` output. **Do not render Venus/Pearl rings** — they are aspirational.
- **Stub `activation_sequence: String` is doubly wrong** (`types.rs:238`): runtime type is an object, and the example value `"Gift of Patience"` matches nothing emitted. Treat the whole stub as non-authoritative for Gene Keys.
- **No `hologenetic`/`spheres` anywhere in the engine** (grep confirms). The "hologenetic profile spheres" from the domain brief are a Gene Keys *concept* but are **not** in this codebase's output — Wave-2 must synthesise spheres from the gate pairs + `frequency_assessments`, or the engine must grow them.
- **`programming_partner` not on the wire.** `GeneKey.programming_partner` exists in the struct (`models.rs:34`) and the renderer reads `result.programming_partner` (GeneKeys.tsx:206), but `serialize_chart` never emits it → the Programming Partner line is dead today. Confirm whether to surface it (add to serializer) or drop the renderer branch.
- **`source` casing.** `active_keys[].source` is Rust `Debug` output (`"PersonalitySun"`, `engine.rs:231`), not `snake_case`/`Personality Sun`. Any renderer label must format it client-side.
- **`active_keys` length is unverified by fixture.** The number of activations (how many planets × Personality/Design map to keys) isn't asserted in `reference_charts.json`; it comes from the HD engine. Expect up to 26 (the `ActivationSource` variant count) but verify against a live HD output before sizing the ring.
- **`line` source.** Lines (1–6) live on `GeneKeyActivation.line` from the HD engine; this crate does not compute them — `reference_charts.json` only carries gates, so line values aren't fixture-verified here.
