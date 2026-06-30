# Sacred Geometry — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

A contemplative engine: pick (or randomly seed) one sacred geometric **form** — Flower of Life, Metatron's Cube, Sri Yantra, a Platonic solid… — and return its symbolism, elements, a numerology number, and a meditation prompt. The brand's core motif: the whole design system *is* sacred geometry, so this renderer is its most literal expression.

> ⚠️ **The README lists this engine as "stub only / runtime source unconfirmed." That is wrong.** There is no Rust `engine-*` crate, but there **is** a fully-implemented **TypeScript** producer (`ts-engines/src/engines/sacred-geometry/`). The Rust side is an HTTP proxy. See §1 and §7.

## 1. Identity
| | |
|---|---|
| `engine_id` | `sacred-geometry` (confirmed: TS `metadata().id` engine.ts:34; Rust bridge lib.rs:283; SDK list client.rs:30) |
| Rust crate | **— (no Rust engine; OpenAPI stub + HTTP-proxy bridge + renderer)** |
| **Runtime source** | **`ts-engines/src/engines/sacred-geometry/engine.ts`** (`SacredGeometryEngine.calculate`, L56) — the real producer, a Bun HTTP service |
| Producer catalog | `ts-engines/src/engines/sacred-geometry/wisdom.ts` (`SACRED_FORMS[]`, L16 — 13 forms) · `witness.ts` (prompts) |
| Bridge (Rust→TS) | `crates/noesis-bridge/src/lib.rs:277` (`BridgeEngine::sacred_geometry`, `required_phase=0`) → POST `…/engines/sacred-geometry/calculate` at `DEFAULT_TS_SERVER_URL` `http://localhost:3001` (lib.rs:42; env `TS_ENGINES_URL` lib.rs:558) |
| Renderer | `apps/noesis-web/src/components/engines/SacredGeometry.tsx` (⚠️ not wired into any grid/router yet — no importer found in `apps/noesis-web/src`) |
| Fixture (real values) | **none found** — no JHora/reference fixture; nearest sample is the orchestrator mock `mock_geometry_output()` (synthesis/creative_expression.rs:645) |
| OpenAPI stub | `crates/noesis-core/src/types.rs:346` (`SacredGeometryResultSchema` — `pattern`/`point_count`/`symbolic_theme`) |

**Two name spaces, one engine.** The TS metadata `id` is `sacred-geometry` (engine.ts:34) — the same string the Rust bridge registers. (Some grep tooling renders the class/path as `ln`; that is a display artifact, the source reads `SacredGeometryEngine` / `sacred-geometry`.)

## 2. Output schema

**Runtime JSON (authoritative — what the TS producer emits, engine.ts:89–105):**
```jsonc
{
  "form": {
    "id":          "flower-of-life",            // enum, see §3 (13 ids)
    "name":        "Flower of Life",
    "description": "An ancient symbol composed of …",
    "symbolism":   "Unity, creation, and the interconnectedness …",
    "elements":    ["circle", "hexagon", "vesica piscis"],  // string[]
    "numerology":  6                            // int 0–20 (per-form constant; NOT point_count)
  },
  "meditation": { "prompt": "Allow your awareness to rest …", "duration_suggestion": "5-15 minutes" }, // OBJECT
  "intention":   null,                          // string | null (echo of question/parameters.intention)
  "svg_preview": { "status": "absent" },        // OBJECT {status:'absent'|'accepted'|'rejected', reason?} — NOT a string
  "seed":        123456
}
```
This `result` is wrapped in the standard `EngineOutput` envelope (engine_id, result, witness_prompts[], calculated_at, processing_time_ms — TS types `ts-engines/src/types/engine.ts:35`; Rust mirror `TsEngineResponse` ts_client.rs:28 → mapped to core `EngineOutput` lib.rs:439).

**OpenAPI stub `SacredGeometryResultSchema` (types.rs:346 — docs only, ⚠️ does NOT match the producer):**
`{ pattern:String="flower-of-life", point_count:i32=144, symbolic_theme:String="Harmonic expansion" }`. **None of these three fields exist in the runtime JSON.** `pattern`≈`form.id`; `point_count` and `symbolic_theme` are invented examples — do not build against them.

**Renderer's read shape (`SacredGeometry.tsx`)** — tolerant, with fallbacks for a *flatter* shape that the producer does not emit (mismatches in §7):
`result.form.{name,category,elements,symbolism,golden_ratio_present,description}` (preferred) **OR** flat `result.{form_name|primary_form, form_id, category, elements, symbolism, golden_ratio, description}`; plus `result.intention`, `result.svg_preview` (read as **string** → `dangerouslySetInnerHTML`), `result.meditation_guidance` (read as **string**). Pattern art is chosen by `detectForm(name)` (SacredGeometry.tsx:27) string-matching the form name.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `form.id` | **13-value enum** (wisdom.ts:16) | `flower-of-life`, `seed-of-life`, `metatrons-cube`, `sri-yantra`, `vesica-piscis`, `tetrahedron`, `cube`, `octahedron`, `dodecahedron`, `icosahedron`, `golden-spiral`, `torus` — **12 listed; README "platonic solids" = the 5 solids** (tetra/cube/octa/dodeca/icosa). Input `form` validated against this set or 400 `INVALID_SACRED_FORM` (engine.ts:69) |
| `form.numerology` | **int 0–20** | per-form constant, not a count: torus=0, golden-spiral=1, vesica-piscis=2, tetra=4, flower/cube=6, seed=7, octa=8, sri-yantra=9, dodeca=12, metatron=13, icosa=20 (wisdom.ts) |
| `form.elements` | `string[]` (1–7) | building blocks, e.g. metatron = circle/line + 5 Platonic solids (wisdom.ts:50) |
| `meditation.duration_suggestion` | constant `"5-15 minutes"` | engine.ts:100 |
| `svg_preview.status` | `absent`\|`accepted`\|`rejected` | `absent` unless caller passes `parameters.svg_template`; rejected if not a complete `<svg>…</svg>` or contains `<script`/`onload=` (XSS guard, engine.ts:12–30) |
| `intention` | string \| `null` | from `input.question` ?? `parameters.intention` (engine.ts:61) |
| `seed` | u64 | echoed; drives random form pick + witness template choice (engine.ts:63,77) |
| `consciousness_level` | required_phase **0** | lowest gate — always accessible (engine.ts:39; bridge lib.rs:283) |

**Stub-only fields** (`point_count`, `symbolic_theme` — types.rs:349-352): **invented**, no producer emits them. Renderer-only fields (`category`, `golden_ratio_present`/`golden_ratio`): **not emitted** by the producer → those renderer cells are always hidden today.

**Orchestrator vocabulary divergence:** the Creative Expression workflow's Rust `SacredForm` enum (creative_expression.rs:30) uses **underscore** ids (`flower_of_life`, `metatrons_cube`, `platonic_solids`) and adds `circle`/`merkaba`/`fibonacci_spiral`/`platonic_solids` not in the TS catalog — that enum feeds *workflow options/synthesis*, not the engine's own id set (hyphenated). When sending `parameters.form` to the engine, use the **hyphenated** TS ids.

## 4. Component & brand archetype
**Today** (`SacredGeometry.tsx`): renders a **200×200 SVG** of the detected form (emerald stroke `rgba(16,181,167)` on a Void `#070B1D` field) — hand-built generators for flower-of-life (center + hex ring), Metatron's cube (13 centers, all interconnecting lines), Sri Yantra (3 up + 3 down nested triangles + bindu), Fibonacci (golden-spiral arcs + φ-rectangle), seed-of-life, vesica-piscis, merkaba (two tetrahedra), torus (tilted ellipses), and a default mandala — plus a text grid (Form, Symbolism, Golden Ratio, Elements), held-intention line, description, and a gold meditation-guidance card. If `result.svg_preview` is a (truthy) string it is injected raw and replaces the generator. **Already on-brand** (emerald sacred-geometry line-art on Void) but **static** — no draw-in animation, and several reads don't match the producer (§7).

**Wave-2 target:** **nested Platonic solids / Flower-of-life construction drawn from the data** — Anime.js **self-drawing** (`stroke-dashoffset`) construction on load: the active `form.id` selects the geometry, `form.elements` drives which sub-figures nest (e.g. Metatron → draw the 13 circles, then the connecting lattice, then the 5 inscribed solids in sequence), `form.numerology` sets the node/petal count, gold `#C5A017` for the φ/golden-ratio elements. This engine should be the **most literal** sacred-geometry renderer in the system.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `form.id` / `form.name` | selects the construction (`detectForm` → generator); center label = name |
| `form.numerology` | (target) node / vertex / petal count of the drawn figure |
| `form.elements` | (target) which sub-figures are nested & drawn in sequence (circle→hexagon→solids) |
| `form.symbolism` | text cell (gold) |
| `form.description` | description panel |
| golden-ratio forms (`golden-spiral`, `dodecahedron`) | (target) φ elements drawn in gold `#C5A017`; renderer's "Golden Ratio ◈" cell — but see §7, field not emitted |
| `meditation.prompt` | (target) gold meditation card — **renderer currently reads `meditation_guidance`, a different path** (§7) |
| `intention` | "Held intention: …" line |
| `svg_preview` (if `accepted`) | server-supplied SVG injected raw, replacing the generator |

## 6. Dynamics
**One-shot per (form, intention, seed).** Not live; no time/place inputs (unlike Vedic engines), no baseline/delta (unlike biofield). Output is **deterministic given `seed`** — same seed + same form ⇒ identical witness prompts (SeededRandom, engine.ts:77,82); omit `form` and the seed also picks the form. On render, the construction should animate in once (Anime.js line-draw); an optional slow core "breath" (4:7:8) suits the bindu/center. `consciousness_level` is `0` (always available); deeper levels could gate how much symbolism/meditation text is surfaced, but the producer returns the full block regardless today.

## 7. Open questions / assumptions
- **⚠️ Missing-engine premise is FALSE / README is stale (highest priority).** README lists Sacred Geometry as crate `—` with runtime source **`unconfirmed`** (README.md:43,61). In fact there is **no Rust crate** but a **fully-implemented TS producer** (`ts-engines/.../engine.ts`) reached via `noesis-bridge` HTTP proxy to `:3001`. **Action:** update README's Sacred Geometry row → runtime source = `ts-engines (bridge)`; same correction almost certainly applies to the other four "stub only" engines (Enneagram, Tarot, I Ching, Sigil Forge — all have `BridgeEngine` factories lib.rs:248–293 and `ts-engines/src/engines/*` dirs).
- **⚠️ Stub fields are fiction.** `point_count: 144` and `symbolic_theme` (types.rs:349-352) appear in **no** producer or renderer. The real per-form scalar is `form.numerology` (0–20). Don't build the Wave-2 "144 nodes" idea on the stub — derive node count from `numerology`/`elements`.
- **⚠️ Renderer ↔ producer schema mismatches (real bugs, confirmed):**
  1. `meditation`: producer emits **object** `{prompt, duration_suggestion}` (engine.ts:98); renderer reads **`result.meditation_guidance`** as a string (SacredGeometry.tsx:361) → meditation card **never shows**. Renderer should read `result.meditation.prompt`.
  2. `svg_preview`: producer emits **object** `{status, reason?}` (engine.ts:103); renderer treats it as a **string** and feeds `dangerouslySetInnerHTML` (SacredGeometry.tsx:309,372) → object is truthy, so it would inject `[object Object]`-ish / break. Renderer should gate on `svg_preview.status==='accepted'` and use the (separately-returned) markup — note the producer **validates** but does not currently return the SVG body, only status.
  3. `form.category` and `form.golden_ratio_present`/`golden_ratio` are read by the renderer (SacredGeometry.tsx:366,369) but **not emitted** → those cells are dead today. Either add them to the producer or drop the cells.
- **No fixture.** Unlike panchanga, there's no reference JSON. The orchestrator mock (`mock_geometry_output`, synthesis/creative_expression.rs:645) is the only sample shape and is **not** authoritative — verify any field against `engine.ts`/`wisdom.ts`, not the mock.
- **Renderer is orphaned.** No file under `apps/noesis-web/src` imports `SacredGeometry.tsx` (grep finds only self-references) — confirm how/whether it's mounted before relying on it as "the renderer in production."
- **Synthesis reads a 3rd shape.** `extract_geometry_themes` reads `result.{available, form (string), qualities[]}` (synthesis/creative_expression.rs:357) — `available`/`qualities` are **not** in the producer's output either; that path likely expects a different envelope. Flag for the orchestrator owners.
