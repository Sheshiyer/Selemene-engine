# Engine Data References

> Per-engine data contracts for the Noesis calculation engines: **what each engine outputs, the value ranges/constraints, which component renders it, and how the data maps to (and changes) the visual.**
> These ground the Wave-2 engine-renderer work (`apps/noesis-web/src/components/engines/*`) the same way `docs/design/biofield-web/` grounds biofield-web.

Source of truth: the Rust engine crates (`crates/engine-*`), the Vedic API (`crates/noesis-vedic-api`), the **TypeScript engines** (`ts-engines/src/engines/*`), the schema registry (`crates/noesis-core/src/types.rs`), the renderers (`apps/noesis-web/src/components/engines/*.tsx`), and the JHora/reference fixtures (`crates/**/tests/**`). **Nothing here is invented** — every field/range is cited to a file, and unknowns are flagged.

---

## The `EngineOutput` envelope (every engine)

All engines return the same envelope (`crates/noesis-core/src/types.rs:39`, TS mirror `packages/noesis-sdk-ts/src/index.ts:52`):

```rust
pub struct EngineOutput {
    pub engine_id: String,           // e.g. "panchanga", "gene-keys", "i-ching"
    pub result: Value,               // engine-specific JSON — the per-engine schema (free-form)
    pub witness_prompt: String,      // one self-inquiry question generated from the calc
    pub consciousness_level: u8,     // 0–5, user's development level (gates depth of interpretation)
    pub metadata: CalculationMetadata, // timing, backend, precision
}
```
TS adds optional `witness_prompts?: string[]` and treats `result` as `Record<string, unknown>`. **`result` is free-form JSON** — the per-engine schema below describes its shape.

---

## ⚠️ Three schema layers + three computation paths

A given engine's output appears in **up to three shapes** that do **not** always agree. Each engine doc records all that apply and names the **runtime truth**.

| Layer | Where | What it is | Trust for |
|---|---|---|---|
| **1. OpenAPI stub** | `noesis-core/src/types.rs` `*ResultSchema` (`#[cfg(feature="openapi")]`) | 3 hand-picked example fields, docs only | example values, canonical engine naming — **not** the full field list (often pure fiction, see audit) |
| **2. Domain struct / producer** | `crates/engine-*/src/{lib,models,types}.rs` (Rust) **or** `ts-engines/src/engines/*/engine.ts` (TS) | the engine's real output type | field names/types/ranges of the engine itself — but check its **serializer**, not just the struct |
| **3. Runtime JSON** | what the API actually returns → what the **renderer** reads + the **fixtures** | the shape the web consumes | **the real contract** — build against this |

**Three computation paths produce `result`:**
1. **Rust `engine-*` crates** — 11 engines, wired into the `EngineResultData` enum. ⚠️ The runtime shape is the crate's **`serialize_*` function**, which often differs from the bare struct.
2. **`noesis-vedic-api`** — the Vedic engines (panchanga, vimshottari, transits, vedic-clock) are *also* served here (JHora-verified, nested/rich JSON). When the vedic-api shape and the `engine-*` struct disagree, the **fixture / vedic-api shape is what ships**.
3. **`ts-engines` (Bun sidecar, `:3001`)** — 6 engines (Enneagram, Tarot, I Ching, Sacred Geometry, Sigil Forge, Raaga) are TypeScript, reached from Rust via **`noesis-bridge`** (`BridgeEngine::*` → HTTP `{TS_ENGINES_URL}/engines/<id>/calculate`), which passes `result` through verbatim. Their `types.rs` OpenAPI stubs are illustration only and share **no** field names with the real JSON.

---

## ⚠️ Systemic finding — renderers are broken against runtime

The biggest outcome of this audit: **almost every engine renderer reads keys the engine never emits.** This is a static (code-level) analysis — high confidence, but the multi-path Vedic engines (panchanga/vimshottari/transits/vedic-clock) should be confirmed with a **live runtime capture** before fixing, since which of the two paths actually ships isn't provable from code alone. Detail + citations live in each doc's §7.

| Engine | Renderer reads | Runtime emits | Effect today |
|---|---|---|---|
| Panchanga | `tithi.name` (nested) | nested via vedic-api ✓ / flat `tithi_name` (crate) | **OK** via vedic-api; crate path differs |
| Vimshottari | `current_mahadasha`, `upcoming_periods`, `years_remaining` | `current_period`, `upcoming_transitions`, `years` | empty cells + empty table |
| Transits | `planetary_positions`, `significant_aspects`, `sade_sati.active` | `transit_positions`, `aspects`, `sade_sati.is_active` | 0 planets plotted; empty aspects/Sade-Sati |
| Vedic Clock | `tcm_organ_clock.*`, `ayurvedic_timing.*` | `current_organ`, `current_dosha` | clock never renders → `GenericEngineView` |
| Human Design | `result.type`, `strategy`, `not_self_theme` | `hd_type` (others not serialized) | Type cell `—`; several fields blank |
| Gene Keys | `activation_sequence.spheres[]` | four `[gate,gate]` arrays + `active_keys[]` | every sphere card `—` |
| Numerology | `result[key].number` | `result[key].value` | numbers render `—` |
| Biorhythm | `value` as ±100 | `value` ±1.0 (+ `percentage` 0–100) | **all waves flat on the zero line** |
| Biofield | `coherence`, `chakras[].value` (0–100) | `metrics.coherence`, `chakra_readings[].activity_level` (0–1) | empty/zero panels |
| Face Reading | `facial_features`, `vitality_score`, `constitution.dosha` | `analysis.constitution.primary_dosha`, `elemental_balance` | near-empty grid |
| Nadabrahman | `chakra_frequency.mantra/.note/.frequency_hz` | `{chakra_name, solfeggio_hz, binaural_target_hz}` | mantra/Hz/note/element blank |
| Enneagram | `triad` (+ lookup-mode cells) | `center` | partial blanks |
| Tarot | `result.cards`, `result.spread` | `{spread, question, positions[], seed}` | cards don't render |
| I Ching | flat `hexagram_number`, `lines`, `changing_lines` | nested `primary_hexagram.*`, `casting.line_values` | hexagram doesn't render |
| Sacred Geometry | `meditation_guidance`, `svg_preview` (string) | `meditation.prompt`, `svg_preview{status}` | unwired / empty |
| Sigil Forge | `vector_path` glyph | method/process text + optional base64 **PNG** (no `vector_path`) | glyph never renders; `dangerouslySetInnerHTML` XSS risk |
| Raaga | `mood`, `rasa`, `vadi`, `samvadi` | producer emits none of these | those cards always hidden |

**Implication for Wave-2:** the engine renderers can't just be restyled to the brand archetypes — most must first be **re-keyed to the real runtime JSON** (and a few engine serializers enriched). Each doc's §4 (data→visual mapping) is written against the *correct* runtime shape, so it doubles as the fix spec.

---

## Engines

**17 engines.** **11** have dedicated **Rust** `engine-*` crates; **6** (Enneagram, Tarot, I Ching, Sacred Geometry, Sigil Forge, Raaga) are **TypeScript** engines in the `ts-engines` Bun sidecar, proxied via `noesis-bridge`. The first 16 are in the Rust `EngineResultData` enum (`types.rs:369`); **Raaga is not** (TS-only, no stub).

| Engine | `engine_id`¹ | Engine impl | Renderer | Runtime source | Brand archetype (Wave-2 target) |
|---|---|---|---|---|---|
| [Panchanga](panchanga.md) | `panchanga` | engine-panchanga | `Panchanga.tsx` | noesis-vedic-api | 5-limb mandala (nakshatra ring + tithi arc) |
| [Vimshottari](vimshottari.md) | `vimshottari` | engine-vimshottari | `Vimshottari.tsx` | noesis-vedic-api | nested dasha→bhukti time-arc rings |
| [Transits](transits.md) | `transits` | engine-transits | `Transits.tsx` | noesis-vedic-api | orbital constellation (planets + aspects) |
| [Vedic Clock](vedic-clock.md) | `vedic-clock` | engine-vedic-clock | `VedicClock.tsx` | engine-vedic-clock | radial day-clock compass (organ + dosha) |
| [Human Design](human-design.md) | `human-design` | engine-human-design | `HumanDesign.tsx` | engine-human-design | BodyGraph (9 centers · 36 channels · 64 gates) |
| [Gene Keys](gene-keys.md) | `gene-keys` | engine-gene-keys | `GeneKeys.tsx` | engine-gene-keys | hologenetic sequence rings (shadow/gift/siddhi) |
| [Numerology](numerology.md) | `numerology` | engine-numerology | `Numerology.tsx` | engine-numerology | digit-spiral nodes (life-path 1–9 + masters) |
| [Biorhythm](biorhythm.md) | `biorhythm` | engine-biorhythm | `Biorhythm.tsx` | engine-biorhythm | three-wave sine overlay (23/28/33 d) |
| [Biofield](biofield.md) | `biofield` | engine-biofield (+ -capture, Py CV) | `Biofield.tsx` | engine + Py sidecar + client PIP | cosmogram + mandala (see `docs/design/biofield-web`) |
| [Face Reading](face-reading.md) | `face-reading` | engine-face-reading | `FaceReading.tsx` | engine-face-reading | face proportion grid + elemental balance |
| [Nadabrahman](nadabrahman.md) | `nadabrahman` | engine-nadabrahman | `Nadabrahman.tsx` | engine-nadabrahman | harmonic waveform / Solfeggio arc |
| [Enneagram](enneagram.md) | `enneagram` | ts-engines (TS) | `Enneagram.tsx` | ts-engines (Bun :3001) | 9-point enneagram figure (type · wing) |
| [Tarot](tarot.md) | `tarot` | ts-engines (TS) | `Tarot.tsx` | ts-engines (Bun :3001) | card-glyph spread compass |
| [I Ching](iching.md) | `i-ching` | ts-engines (TS) | `IChing.tsx` | ts-engines (Bun :3001) | hexagram line-stack / 64-grid |
| [Sacred Geometry](sacred-geometry.md) | `sacred-geometry` | ts-engines (TS) | `SacredGeometry.tsx` | ts-engines (Bun :3001) | nested platonic solids / flower-of-life |
| [Sigil Forge](sigil-forge.md) | `sigil-forge` | ts-engines (TS) | `SigilForge.tsx` | ts-engines (Bun :3001) | sigil construction diagram |
| [Raaga](raaga.md) | `raaga` | ts-engines (TS) | `Raaga.tsx` | ts-engines (Bun :3001) | swara wheel / tonal arc (not in Rust enum) |

¹ `engine_id` values verified per doc against the producer + `noesis-bridge` + renderer route (note `i-ching`, not `iching`). The fallback renderer `GenericEngineView.tsx` renders any engine whose `result` shape isn't specially handled — which, per the audit above, is the *de-facto* state for most engines today.

---

## Per-engine doc template

Each `<engine>.md` follows this structure:

1. **Identity** — `engine_id`, engine impl path(s), renderer path, fixture path(s), runtime source.
2. **Output schema** — the runtime JSON (authoritative) as an annotated example, then the domain struct/producer + OpenAPI stub; mismatches called out.
3. **Ranges, constraints & invariants** — per-field range/enum/unit, nullability, invariants (sums to 1, monotonic dates, index bounds…).
4. **Component & brand archetype** — what the renderer draws today + the sacred-geometry archetype it should become (brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`). Doubles as the re-key/fix spec.
5. **Data → visual mapping** — table: field → geometry / arc / color / position (against the *correct* runtime shape).
6. **Dynamics** — one-shot vs live; recompute cadence (e.g. vedic-clock is time-of-day); transitions; baseline/deltas; what re-renders.
7. **Open questions / assumptions** — flagged uncertainties (schema mismatches, normalization, unverified runtime shapes).

Consciousness level (`0–5`) and `witness_prompt(s)` are envelope-level (above) — engines note only engine-specific deviations.
