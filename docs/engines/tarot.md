# Tarot — Data Reference

A seeded card draw laid on a named spread: per-position card (name, arcana, suit, element, upright/reversed) with position meaning + keywords, plus reflective witness prompts. No "synthesis" field is produced at runtime (that's stub-only).

## 1. Identity
| | |
|---|---|
| `engine_id` | `tarot` (confirmed — `ts-engines/src/engines/tarot/engine.ts:108`; renderer dispatch `apps/noesis-web/app/readings/[id]/page.tsx:45`) |
| Crate | **— (no Rust engine; stub + renderer)** |
| Runtime source | **`ts-engines/src/engines/tarot/` (TypeScript engine)** — `engine.ts` (`TarotEngine.calculate`, L38) registered `ts-engines/src/index.ts:23`, served `POST /engines/tarot/calculate` (`ts-engines/src/server/app.ts:146`) |
| Renderer | `apps/noesis-web/src/components/engines/Tarot.tsx` |
| Deck data (in-code, authoritative) | `ts-engines/src/engines/tarot/wisdom.ts` (`MAJOR_ARCANA` L30, `createMinorArcana` L579, `loadTarotDeck` L648) |
| Reference data (NOT consumed by engine) | `data/tarot/rider_waite.json`, `data/tarot/major_arcana.json` — see Open questions |
| OpenAPI stub | `crates/noesis-core/src/types.rs:324` (`TarotResultSchema` — `spread_type`/`focal_card`/`synthesis`) |

## 2. Output schema

**Runtime JSON (authoritative — emitted by `engine.ts:79–105`, inside the `EngineOutput.result` envelope):**
```jsonc
{
  "spread": { "type": "three_card", "name": "Three Card Spread",      // type ∈ SpreadType enum (snake_case)
              "description": "Past, Present, and Future…" },
  "question": "Should I change careers?",                              // optional; echoes input, may be absent
  "positions": [                                                       // length = spread's position count
    {
      "position": 0,                                                   // NUMBER (0-indexed), not a label
      "name": "Past",                                                  // human label for the slot
      "meaning": "What has led to this moment; influences from the past",
      "card": {
        "id": "major-9", "name": "The Hermit",
        "arcana": "major",                                            // "major" | "minor"
        "suit": "wands",                                              // present only for minor arcana
        "number": 9, "element": "earth",
        "isReversed": false,
        "interpretation": { "meaning": "…upright or reversed text…",  // orientation-resolved
                            "keywords": ["introspection","solitude","guidance","wisdom"] }
      }
    }
    // … one entry per position
  ],
  "seed": 12345                                                       // optional; undefined ⇒ non-deterministic draw
}
```
Envelope also carries `witness_prompts` (see §3 note), `calculated_at`, `processing_time_ms` (`engine.ts:107–113`).

**OpenAPI stub (`types.rs:324`) — examples only, does NOT match runtime:** `{ spread_type:"three-card", focal_card:"The Star", synthesis:"Renewed trust in unfolding" }`. None of these three keys are produced by the engine: runtime uses `spread.type` (`three_card`, underscore), has **no** `focal_card`, and emits **no** `synthesis`. Treat the stub purely as canonical-naming/example bait.

**No Rust domain struct exists** (crateless engine) — the TS `result` object above is the only real contract.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `spread.type` | `single_card` \| `three_card` \| `celtic_cross` \| `relationship` \| `career` | `SpreadType` enum, `spreads.ts:5`; default `three_card` (`engine.ts:42`) |
| `positions.length` | **1 / 3 / 10 / 7 / 5** | per spread: single=1, three=3, celtic_cross=10, relationship=7, career=5 (`spreads.ts:196`, counts via `getSpreadPositionCount` L208) |
| `positions[].position` | **0 … n−1** | integer, 0-indexed slot index (`spreads.ts` position fields) |
| `card.arcana` | `major` \| `minor` | `wisdom.ts:13` |
| `card.suit` | `wands` \| `cups` \| `swords` \| `pentacles` | minor only; **absent for major** (`wisdom.ts:6`; major cards omit `suit`) |
| `card.number` | major **0–21**; minor **1–14** | minor: 1=Ace…10, 11=Page,12=Knight,13=Queen,14=King (`wisdom.ts:260–261,605,622`) |
| `card.element` | `fire` \| `water` \| `air` \| `earth` ( \| `spirit`) | minor element fixed by suit (`SUIT_ELEMENTS`, `wisdom.ts:253`); major element hand-assigned; `spirit` declared in type but unused by current deck |
| `card.isReversed` | bool, **p=0.5** per card | `shuffle.ts:35` (`rng.nextBool(0.5)`) |
| `interpretation.meaning` | string | `card.uprightMeaning`/`reversedMeaning` selected by orientation (`reading.ts:36`) |
| `interpretation.keywords` | string[] (≈4) | from the drawn card (`reading.ts:62`) |
| `seed` | uint32 \| absent | when absent ⇒ `getDefaultSeed()` = `Date.now() ^ Math.random()` (`utils/random.ts:61`) — **non-deterministic** |

**Invariants:** deck = **78 cards** (22 major + 56 minor = 4 suits × [10 pip + 4 court]); `loadTarotDeck()` builds `allCards = [...major, ...minor]` (`wisdom.ts:653`). Cards are drawn **without replacement** via Fisher-Yates over a copy (`shuffle.ts:17,29`), so all `card.id` in a reading are distinct. Same `seed` ⇒ identical spread + orientations + prompts (deterministic when seeded).

> **⚠️ Renderer/producer field mismatch (confirmed — see Open questions):** the engine emits `result.positions[]` with `{position:number, name, meaning, card:{…isReversed…}}`, but `Tarot.tsx` reads `result.cards ?? result.spread` (L371), and per-slot reads `raw.card ?? raw`, `raw.position` (as the **label**), `raw.reversed/is_reversed`, `reading.present`, `card.position_description`. The current runtime shape does **not** populate these. No adapter remaps `positions`→`cards` anywhere in `apps/noesis-web` / `packages/noesis-sdk-ts` / `crates/noesis-bridge`.

## 4. Component & brand archetype
**Today** (`Tarot.tsx`): one `CardSlot` per entry. Reads `result.cards ?? result.spread` (array) else falls back to single-card from the root object. Each slot draws a **bioluminescent card face** (`CardFace`, L188) — 140×233px (`CARD_W`, 5:3, L100), gold border, violet→void radial gradient, inset gold + emerald glow — with a central **suit sigil** (`suitSymbol`: wands `⟁`, cups `◯`, swords `✦`, pentacles `⬡`, else `◈`, L19), card number (top-left), a rotated "Reversed" badge (top-right, terracotta, L160), card name (bottom), and an **element dot** colored by element (fire terracotta / water indigo / air white / earth emerald, `elementColor` L29). Below the face: keyword pills, an italic interpretation blockquote, and Description / Position / Meaning / Reversed-Meaning sections. Position labels come from `raw.position` or the hard-coded `POSITIONS=["Past","Present","Future"]` fallback (L364). Already palette-correct (gold `#C5A017`, emerald `#10B5A7`, violet, indigo) but each card is a **rectangular face**, not yet a geometric glyph, and the **spread has no layout geometry** (cards stack in a flex column).

**Wave-2 target — card-glyph spread compass:** positions laid out on the **actual spread geometry** (three-card row; Celtic Cross's cross+staff; relationship/career bespoke layouts) rather than a vertical list. Each card becomes a **sacred-geometry sigil** (suit/arcana-derived glyph, not photoreal art), drawn-in on load (Anime.js stroke-dashoffset). **Upright/reversed = literal 180° glyph rotation** (not just a badge). The **focal card** (e.g. the Major Arcana key card / `getKeyCards`, `reading.ts:108`) emphasized — larger, brighter bioluminescent core, others dimmed. Element drives glyph accent color; keywords radiate as a small label ring.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `spread.type` | overall layout geometry (row / cross+staff / bespoke) — *target* |
| `positions[].position` + `.name` | glyph placement on the spread + slot label |
| `positions[].meaning` | slot caption / "Position" section |
| `card.arcana` + `card.suit` | which sacred-geometry sigil is drawn (`suitSymbol` today) |
| `card.element` | accent / element-dot color (`elementColor`) |
| `card.isReversed` | 180° glyph rotation (today: rotated "Reversed" badge) |
| `card.number` | corner numeral on the face |
| `interpretation.keywords` | keyword pills / radial label ring |
| `interpretation.meaning` | interpretation blockquote |
| focal / key card | size + glow emphasis — *target* |

## 6. Dynamics
**One-shot per call.** Inputs: `{ spread, question, seed }` (`engine.ts:42–43`). Unlike biofield/vedic-clock there is no live recompute and no time-of-day dependency — but it is **not idempotent unless a `seed` is supplied**: without `seed`, `getDefaultSeed()` reseeds from wall-clock + `Math.random()` each call, so re-running yields a different spread. With a fixed `seed`, the entire reading (cards, reversals, witness prompts) is reproducible. On render the cards/glyphs should animate in once (line-draw); no perpetual loop except an optional focal-card breath. `consciousness_level` (0–5, envelope) may gate how much meaning text is surfaced; it does **not** change the draw.

## 7. Open questions / assumptions
- **⚠️ NO dedicated engine crate — producer is the TypeScript `ts-engines` service, not Rust.** The only Rust artifacts are the OpenAPI stub (`types.rs:324`) and the orchestrator's spread-type enum/synthesis helpers (`crates/noesis-orchestrator/src/workflow/decision_support.rs:37`, `…/synthesis/decision_support.rs`). The runtime `result` is produced by `ts-engines/src/engines/tarot/engine.ts`. **Confirm this TS service is the deployed producer for the web** (vs. some other path) — the renderer is wired (`page.tsx:45`) and the TS server exposes `/engines/tarot/calculate`, but the request path from `noesis-api`/`get-reading` to `ts-engines` was not traced here.
- **⚠️ Renderer ≠ producer schema (confirmed defect, not just naming).** Renderer reads `result.cards`/`result.spread` (array of cards) + `raw.card`, `raw.reversed`, `reading.present`, `position_description`, `key_meaning`, `arcana_number`; producer emits `result.positions[]` + `positions[].card.isReversed` + `interpretation.{meaning,keywords}` and a `spread` **object** (not an array). As written, `Tarot.tsx` would find `result.cards` undefined and `result.spread` an object → `arr()` returns `[]` → it falls into the **single-card branch** and renders one card from the root (mostly empty). **This must be reconciled in Wave-2**: either the renderer reads `positions[]`/`card.isReversed`, or a thin adapter maps producer→renderer. Build Wave-2 against the **producer** shape (it's what actually ships) and fix the renderer.
- **Spread enum naming drift:** producer uses snake_case (`three_card`); OpenAPI stub example and orchestrator `TarotSpread::as_str()` use other forms (`"three-card"`, `"THREE_CARD"`). The web should normalize via `parseSpreadType` (`spreads.ts:212`, strips `-`/spaces, lowercases) before relying on the value.
- **`data/tarot/*.json` is dead relative to the engine.** `loadTarotDeck()` builds the deck from in-code constants (`wisdom.ts`); the richer `rider_waite.json` / `major_arcana.json` (astrological correspondences, Hebrew letters, Tree-of-Life paths, `spreads`) are **not loaded** by the TS engine. They're a candidate data source for Wave-2 glyph/correspondence detail — verify before citing any field from them as runtime.
- **`witness_prompts` shape:** `engine.ts:110` sets `witness_prompts = generateQuestionBasedPrompts(...)`, which returns **`WitnessPrompt[]` objects** `{prompt, context, themes}` (`witness.ts:170–187`), not the `string[]` the envelope's TS `witness_prompts?: string[]` implies (README envelope). Confirm whether the API serializes these objects as-is or flattens to strings before the web reads them.
- **`focal_card` / `synthesis` (stub) have no runtime source.** If the brand "focal card emphasis" needs a server-chosen focal card, it currently must be derived client-side (e.g. first Major Arcana via `getKeyCards`); the engine does not flag one.
