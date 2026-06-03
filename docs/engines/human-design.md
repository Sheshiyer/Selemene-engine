# Human Design — Data Reference

The BodyGraph for a moment of birth: Type, Strategy, Authority, Profile, plus the 9 centers (defined/undefined), 36 channels, and 64 gates derived from two activation sets (personality at birth, design ~88° of solar arc before birth).

## 1. Identity
| | |
|---|---|
| `engine_id` | `human-design` (`crates/engine-human-design/src/engine.rs:26`, asserted `engine.rs:288`) |
| Domain crate | `crates/engine-human-design/src/lib.rs` (chart structs in `models.rs:7`, wisdom in `wisdom.rs`) |
| Runtime source | **`engine-human-design`** — `serialize_chart()` (`engine.rs:60`) produces `EngineOutput.result` |
| Renderer | `apps/noesis-web/src/components/engines/HumanDesign.tsx` |
| Fixture (real values) | `crates/engine-human-design/tests/reference_charts.json` (16 charts; synthetic, see §7) |
| OpenAPI stub | `noesis-core/src/types.rs:225` (`HumanDesignResultSchema` — `type_name`/`authority`/`strategy`) |

## 2. Output schema

**Runtime JSON (authoritative — exactly the 8 keys `serialize_chart` emits, `engine.rs:105`):**
```jsonc
{
  "hd_type":    "ManifestingGenerator",   // Debug-formatted enum (PascalCase, no spaces)
  "authority":  "Emotional",              // Debug-formatted enum
  "profile":    "2/5",                    // "conscious/unconscious" line, e.g. 2/5
  "definition": "Single",                 // Debug-formatted enum
  "defined_centers": ["Ajna","Root","Sacral","SolarPlexus","Throat"],  // Debug-formatted Center names
  "active_channels": ["39-55","43-23","59-6"],                         // "gate1-gate2" strings
  "personality_activations": {            // keyed by planet name (lowercase), 13 planets
    "sun":   { "gate": 48, "line": 2, "longitude": 191.6 },
    "earth": { "gate": 21, "line": 2, "longitude": 11.6  }
    // … moon, northnode, southnode, mercury, venus, mars, jupiter, saturn, uranus, neptune, pluto
  },
  "design_activations": { "sun": { "gate": 39, "line": 5, "longitude": … }, "earth": { … }, … }
}
```

**Domain struct `HDChart` (`models.rs:7`) — richer than the serialized output:** `personality_activations: Vec<Activation>` · `design_activations: Vec<Activation>` · `centers: HashMap<Center, CenterState>` (`CenterState{defined:bool, gates:Vec<u8>}`, `models.rs:44`) · `channels: Vec<Channel>` (`Channel{gate1,gate2,name,circuitry}`, `models.rs:63`) · `hd_type: HDType` · `authority: Authority` · `profile: Profile{conscious_line,unconscious_line}` (`models.rs:91`) · `definition: Definition`. `Activation{planet,gate,line,longitude}` (`models.rs:19`). **`serialize_chart` drops** per-center gate lists, channel `name`/`circuitry`, and collapses `profile` to a string.

**OpenAPI stub (`types.rs:225`):** `{ type_name:String, authority:String, strategy:String }` — examples only; field name is `type_name`, not `hd_type` or `type`.

**Wisdom layer (NOT in `result`):** `wisdom.rs` defines `GateWisdom` (`:7`), `CenterWisdom`, `ChannelWisdom`, `TypeWisdom` (carries `strategy`, `signature`, `not_self_theme`, `aura`), `AuthorityWisdom`, `ProfileWisdom` (`:74`), `IncarnationCrossWisdom`, etc., loaded from `data/human-design/*.json`. These are interpretive lookups, not part of the engine's calculated output.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `hd_type` | `Generator` \| `ManifestingGenerator` \| `Projector` \| `Manifestor` \| `Reflector` | `HDType`, `models.rs:71`. Fixture covers only MG/Manifestor/Projector (`reference_charts.json:689`) |
| `authority` | `Sacral` \| `Emotional` \| `Splenic` \| `Heart` \| `GCenter` \| `Mental` \| `Lunar` | `Authority`, `models.rs:80`. Fixture covers only Emotional/Sacral/Splenic (`:682`) |
| `definition` | `Single` \| `Split` \| `TripleSplit` \| `QuadrupleSplit` \| `NoDefinition` | `Definition`, `models.rs:97`. **Not present in fixture `expected`** — coverage unverified |
| `profile` | `c/u`, c,u ∈ **1–6** | `Profile.conscious_line`/`unconscious_line` are `u8` (`models.rs:91`); 12 canonical pairs (1/3,1/4,2/4,2/5,3/5,3/6,4/6,4/1,5/1,5/2,6/2,6/3). Fixture `profiles: 11` (`:687`) |
| `defined_centers[]` | subset of the **9** Centers | `Head, Ajna, Throat, G, Heart, Spleen, SolarPlexus, Sacral, Root` (`Center`, `models.rs:50`). Length 0–9; Debug names (`SolarPlexus`, `G`) |
| `active_channels[]` | each `"g1-g2"`, g ∈ **1–64** | up to **36** total channels. Order/direction is the struct's `(gate1,gate2)` — **not normalized** (e.g. fixture has both `"10-34"` and `"34-20"`, `"20-10"`) |
| `*_activations.<planet>` | 13 planets each set | keys: `sun, earth, moon, northnode, southnode, mercury, venus, mars, jupiter, saturn, uranus, neptune, pluto` (`Planet`, `models.rs:27`, lowercased Debug) |
| `…gate` | **1–64** | the hexagram/gate |
| `…line` | **1–6** | line within the gate |
| `…longitude` | **0–360°** | ecliptic longitude (tropical; Swiss Ephemeris, `engine.rs:192`) |

**Invariants.** A center is *defined* iff a complete channel touching it is active (so `defined_centers` and `active_channels` co-vary — not independent). `profile` lines = the *lines* of personality-Sun (conscious) and design-Sun (unconscious); the incarnation cross = the four gates of personality Sun/Earth + design Sun/Earth. Personality vs design = same chart at birth vs ~88° solar arc earlier (`design_time.rs`). Sacral defined ⟹ Type ∈ {Generator, ManifestingGenerator}.

## 4. Component & brand archetype
**Today** (`HumanDesign.tsx`): a real **BodyGraph SVG** (`viewBox 0 0 200 270`, `:204`) — 9 center polygons with correct shapes (Head/G diamond, Ajna/SP tri-down, Spleen/Ego tri-up, Throat/Sacral/Root rect; `BODY_CENTERS`, `:114`), lit emerald `rgba(16,185,129,0.6)` when defined else faint (`:194`), plus **10 hardcoded channel lines** between center *pairs* (`CHANNELS`, `:127`) thickened/emerald when both ends defined. Definition is read from `result.defined_centers` (array path, `isCenterDefined`/`definedList`, `:151`) with broad center aliases (`CENTER_ALIASES`, `:76`). Below: a 6-cell stat grid (Type/Strategy/Authority/Profile/Definition/Not-Self Theme) + an Incarnation-Cross block + a 9-row Centers list. **Already substantially on-brand** (sacred-geometry BodyGraph, emerald fills) — but four readouts and the gate/channel detail are placeholders fed by absent fields (see §5/§7).

**Wave-2 target:** the BodyGraph is **load-bearing sacred geometry, not decoration.** Render the *real* 36 channels as composed gate-pairs (each channel = two gates, one per center) and the **64 gates as nodes** on the center perimeters (lit when activated, colored by personality=black / design=red per HD convention). Light a channel only when present in `active_channels`; light a center only when defined. Type/Strategy/Authority/Profile become the readout around the graph; center color = bioluminescent on-brand emerald `#10B5A7` defined vs Void `#070B1D` open. Gates draw-in (Anime.js stroke-dashoffset) on load. Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `defined_centers[]` | the 9 center polygons; named center filled emerald + glow when present, else open outline |
| `active_channels[]` (`"g1-g2"`) | (target) the specific channel line drawn lit between its two centers; gate nodes at each end lit |
| `…gate` (1–64) per activation | (target) 64 perimeter gate-nodes; personality set vs design set colored distinctly |
| `…line` (1–6) | gate-node sub-tick / tooltip; feeds profile |
| `hd_type` | central readout (Type) — **renderer reads `result.type`, which the engine does not emit (§7)** |
| `authority` | readout cell (Authority) |
| `profile` (`"2/5"`) | readout cell; renderer prefers `profile.line1/line2` then falls back to the string |
| `definition` | readout cell (Single/Split/…) — geometry could tint connected sub-graphs |
| *strategy* | readout cell — **no runtime field; from `TypeWisdom.strategy` lookup (§7)** |
| *not_self_theme* | readout cell — **no runtime field; from `TypeWisdom.not_self_theme` (§7)** |
| *incarnation_cross / sun_gate / earth_gate* | "Incarnation Cross Seed" block — **no runtime field; derivable from `personality_activations.{sun,earth}` (§7)** |

## 6. Dynamics
**One-shot per birth moment.** `calculate()` (`engine.rs:138`) takes date/time/timezone and computes once; **latitude/longitude are extracted but unused** for the chart (`_latitude, _longitude`, `engine.rs:142`) — unlike Vedic engines, the BodyGraph is location-independent here (houses aren't computed). Not live; recompute only if birth data changes. No baseline/delta semantics. `consciousness_level` (0–5, default 1; `engine.rs:166`) is envelope-level and may gate interpretive depth (gate/channel/type prose) but does not change the geometry. On render, defined centers + active channels + gate nodes should animate in once (line-draw); no perpetual loop except an optional slow core breath.

## 7. Open questions / assumptions
- **`type` vs `hd_type` (CONFIRMED MISMATCH — highest risk):** the engine emits **`hd_type`** (`engine.rs:106`) and its own `validate()` checks `hd_type` (`engine.rs:212`), but the renderer reads **`result.type`** (`HumanDesign.tsx:308`). The OpenAPI stub uses a *third* name, `type_name` (`types.rs:227`). As written, **the Type cell renders `—`.** Pick one canonical key (recommend `type`) and align engine + stub + renderer.
- **`strategy` and `not_self_theme` are never serialized.** Neither appears in `serialize_chart`/`chart.rs` (grep: 0 hits); they live only in `TypeWisdom` (`wisdom.rs:50`) keyed off type. The renderer's Strategy and Not-Self cells (`:312`,`:332`) always render `—` until the engine joins wisdom into `result` (or the renderer derives them client-side from Type).
- **Incarnation cross / `sun_gate` / `earth_gate` absent.** Renderer reads `result.incarnation_cross` / `result.sun_gate` / `result.earth_gate` (`:284`); none are emitted. They are trivially derivable from `personality_activations.{sun,earth}` + `design_activations.{sun,earth}` (already present) — Wave-2 should either add them server-side or have the renderer read the activation objects.
- **`centers` object vs `defined_centers` array:** renderer supports both (`isCenterDefined`, `:142`) and the engine emits the **array** — fine. But the renderer's per-shape lookup uses key `ego` for the Heart/Will center (`CENTER_ALIASES.ego`, `:84`; display maps "heart"→"ego", `:359`), while the engine's enum + array use **`Heart`** (`models.rs:54`, fixture `"Heart"`). The alias list includes `heart` under `ego`, so it matches — verify this mapping survives any rename.
- **Channels are pair-of-centers, not real HD channels.** `CHANNELS` (`:127`) hardcodes 10 center-to-center lines; the real model has **36 gate-defined channels**. The rich data (`active_channels`, per-gate activations) exists in `result` but the renderer doesn't yet draw individual channels/gates — the core Wave-2 build.
- **Channel direction not normalized:** `active_channels` strings follow the struct's `(gate1,gate2)` order, so the same channel can appear as `"10-34"` or `"34-20"` (fixture L222–224). Any gate→channel matching in the renderer must compare unordered pairs.
- **Fixture is synthetic & partial.** `reference_charts.json` is "Selemene HD Engine (Synthetic Reference Data)… Professional HD software validation pending" (`:696`); covers 3 of 5 types, 3 of 7 authorities, and **omits `definition`** from `expected`. Its `expected` block also uses flat `personality_sun`/`design_sun` keys (`:25`), which is the *test-comparison* shape — the live `result` nests these under `personality_activations.sun` (`engine.rs:112`). Treat `serialize_chart` as the contract, the fixture as value samples. The fixture `name` labels (e.g. "Generator 1/3") frequently disagree with the computed `type`/`profile` (e.g. type `ManifestingGenerator`, profile `2/5`) — labels are descriptive, not assertions.
