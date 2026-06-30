# I Ching — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

Hexagram divination: a primary hexagram (1–64, six yin/yang lines), optional changing lines, and a relating (transformed) hexagram, with judgment + image text.

## 1. Identity
| | |
|---|---|
| `engine_id` | `i-ching` (confirmed: `ts-engines/src/engines/i-ching/engine.ts:14,147`; `crates/noesis-bridge/src/lib.rs:263`) — **kebab-case, not `iching`** |
| Crate | **— (no Rust engine; stub + renderer)** |
| Runtime source | **`ts-engines/src/engines/i-ching/engine.ts`** (`IChingEngine`, TS) — served via the Bun TS-engines server (`http://localhost:3001`) behind `crates/noesis-bridge/src/lib.rs:257-264` (`BridgeEngine::i_ching`) |
| Data | `ts-engines/src/engines/i-ching/wisdom.ts` (`HEXAGRAMS[]`, `getHexagramByNumber`) — **only 8 of 64 hexagrams have full data; rest are stubs** (`wisdom.ts:16`) |
| Witness prompts | `ts-engines/src/engines/i-ching/witness.ts` (`generateWitnessPrompts`) |
| Renderer | `apps/noesis-web/src/components/engines/IChing.tsx` |
| OpenAPI stub | `crates/noesis-core/src/types.rs:335` (`IChingResultSchema` — `hexagram`/`hexagram_name`/`guidance` only) |
| Fixture (real values) | none found for the web path; nearest sample is the orchestrator test shape `result.hexagram.{number,name}` (`crates/noesis-orchestrator/tests/synthesis_tests.rs:523`) — a **third** shape, see §7 |

## 2. Output schema

⚠️ **The producer and the renderer disagree on shape.** The runtime producer is the TS engine; the renderer was written against a different (flat) contract. Both are recorded; the mismatch is the headline open question (§7).

**Runtime JSON — what the TS producer actually emits (`engine.ts:119-145`):**
```jsonc
{
  "primary_hexagram": {
    "number": 24,                 // 1–64
    "name": "Return",
    "chinese_name": "復 (Fù)",
    "meaning": "...",
    "judgment": "...",
    "image": "..."
  },
  "changing_lines": [1, 4] ,      // 1-based line positions, or null if none (engine.ts:128)
  "relating_hexagram": {          // present only when changing_lines non-empty; else null
    "number": 2, "name": "...", "chinese_name": "...",
    "meaning": "...", "judgment": "...", "image": "..."
  },
  "casting": {
    "method": "three_coins",      // "three_coins" | "yarrow_stalks"
    "line_values": [7,8,6,9,7,8]  // 6 values, one per line bottom→top; 6/7/8/9
  },
  "seed": 123456
}
```

**Runtime JSON — what the renderer reads (`IChing.tsx:228-235`):** all **flat / top-level**, none nested:
```jsonc
{
  "hexagram_number": 24,          // OR "number" OR "hexagram"  (IChing.tsx:228)
  "hexagram_name": "Return",      // OR "name"                  (IChing.tsx:229)
  "judgment": "...",              // OR "judgement"             (IChing.tsx:230)
  "image": "...",                 //                            (IChing.tsx:231)
  "lines": [true,false,...],      // 6 booleans bottom→top, true=yang; optional (IChing.tsx:26-31)
  "changing_lines": [1,4]         // numbers, or [{line:n}] objects (IChing.tsx:49-63)
}
```

**OpenAPI stub (`types.rs:335` — examples only):** `{ hexagram: i32 = 24, hexagram_name: String = "Return", guidance: String = "Return to the center…" }`. Note `guidance` is in neither the producer nor the renderer.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| hexagram number | **1–64** | producer `primary_hexagram.number`; renderer `hexagram_number\|number\|hexagram`. Validated 1–64 in producer (`engine.ts:59`) |
| `lines` | 6 × boolean | bottom→top, `true`=yang/solid, `false`=yin/broken (`wisdom.ts:13`, `IChing.tsx:25`) |
| `casting.line_values[i]` | **6,7,8,9** | 6=old yin, 7=young yang, 8=young yin, 9=old yang (`engine.ts:91`) |
| `changing_lines` | subset of **1–6** | a line changes iff its value is 6 or 9 (`engine.ts:93`); `null`/`[]` when none |
| `casting.method` | `three_coins` \| `yarrow_stalks` | default `three_coins` (`engine.ts:140`) |
| `relating_hexagram` | hexagram 1–64 or `null` | present only when `changing_lines` non-empty (`engine.ts:99-112`) |
| changing-line probability | ~1/4 per line | each line uniform over {6,7,8,9} → P(change)=2/4 in this RNG, *not* the canonical three-coin 1/4 (`engine.ts:91`) — see §7 |

**Invariants.** (a) Lines bottom→top, line 1 = bottom. (b) The relating hexagram *should* equal the primary with every changing line flipped (`engine.ts:102-107` computes `relatingLines` correctly) — **but the emitted `relating_hexagram` is a random hexagram, not that flip** (`engine.ts:109-111`); `relatingLines` is computed and discarded. (c) Renderer, when `result.lines` is absent, reconstructs the 6 lines from its own `KING_WEN` bit table indexed by hexagram number (`IChing.tsx:5-46`) — so given only a number it still draws lines. No trigram data exists anywhere (see §7).

## 4. Component & brand archetype
**Today** (`IChing.tsx`): an **SVG 6-line hexagram stack** (bottom→top; yang = one solid gold bar `#C5A017`, yin = two gold bars with a gap `rgba(197,160,23,0.4)`; a small gold dot to the right marks a changing line — `IChing.tsx:86-152`) beside a header (`#<number>` + name), then text sections for **Judgment**, **Image**, and a **Changing Lines** list (`IChing.tsx:237-271`). Already partly on-brand (gold lines, display font). No trigram split, no 64-grid, no transformed-hexagram rendering yet.

**Wave-2 target:** the **6-line hexagram stack** (broken/solid bottom→top, changing lines marked) as the hero, with an **optional 64-grid** locating the cast hexagram, and a **primary → transformed** pairing (draw the relating hexagram beside the primary, animate the changing lines flipping). Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `lines[i]` / King Wen of number (1–64) | i-th bar bottom→top: solid (yang) vs split (yin) gold bar (`IChing.tsx:104-137`) |
| hexagram number | large `#n` label in the header (gold, display font) |
| hexagram name | name label under the number |
| `changing_lines` (1–6) | gold dot to the right of each changing bar (`IChing.tsx:140-147`); also a text list |
| `judgment` | "Judgment" text section |
| `image` | "Image" text section |
| `relating_hexagram` | (target) second hexagram stack; flipped lines highlighted → the transformation |
| hexagram number (1–64) | (target) lit cell in the optional 8×8 / 64-grid |

## 6. Dynamics
**One-shot per cast.** A reading is generated once from `(hexagram?, method, seed)` (`engine.ts:37-152`); not live, no recompute cadence. **Casting is RNG-driven** (`SeededRandom`, `engine.ts:40,72,91,110`): omit `hexagram` and the primary, the six line values, and the relating hexagram are all drawn from the seed — so a fixed `seed` reproduces a reading, and a new seed yields a new one (`result.seed` echoes it). The natural one-time animation is the line-stack drawing in (stroke/scale) and, for Wave-2, the changing lines flipping into the relating hexagram. `consciousness_level` (envelope, 0–5) may gate how much interpretive text (meaning/judgment/image) is shown.

## 7. Open questions / assumptions
- **⚠️ NO Rust engine.** I Ching has no `engine-*` crate — only the OpenAPI stub (`types.rs:335`) + the renderer. The real producer is the **TypeScript** `IChingEngine` (`ts-engines/src/engines/i-ching/engine.ts`), reached through `noesis-bridge` → Bun TS server (`:3001`). The README's "runtime source: unconfirmed" is now **confirmed = the TS engine**. (Hexagram 24 "Return" in the stub is just the schema example, matching the King Wen entry.)
- **⚠️ Producer ↔ renderer schema mismatch (confirmed, likely a real defect).** Producer emits **nested** `result.primary_hexagram.{number,name,judgment,image}`, `relating_hexagram`, `casting.line_values`. Renderer reads **flat** `result.hexagram_number|number|hexagram`, `hexagram_name|name`, `judgment`, `image`, `lines`, `changing_lines`. Against raw producer output the renderer would find none of name/number/judgment/image (they're one level down) and would fall back to all-yang lines + empty text. **Something must reshape the bridge output before the renderer, or the renderer is mis-wired.** No such reshaper was found under `apps/noesis-web/src`; no file imports `IChing.tsx` except itself. Resolve before building Wave-2: either flatten in the producer/bridge, or read the nested shape in the renderer. Build against whichever the API actually returns — **unverified here** (no web-path fixture).
- **A third shape exists.** Orchestrator/synthesis code reads `result.hexagram.{number,name}`, `result.relating_hexagram`, `result.changing_lines` (`crates/noesis-orchestrator/src/workflow/synthesis/decision_support.rs:230-246`; test data `synthesis_tests.rs:523`) — i.e. `hexagram.*` (singular), not `primary_hexagram.*`. So three contracts are in play: producer (`primary_hexagram`), orchestrator (`hexagram`), renderer (flat). Confirm which the live API emits.
- **No trigrams.** The task asked to verify upper/lower trigrams; **they do not exist** in the producer (`Hexagram` has only `number,name,chineseName,meaning,judgment,image,lines` — `wisdom.ts:6-14`) nor the renderer. Lines are stored/derived directly, never decomposed into the two trigrams. Trigrams would be new work for Wave-2.
- **Data is mostly stub.** `wisdom.ts:16` states only the first 8 hexagrams have full meaning/judgment/image; the other 56 carry placeholder text. Real interpretive copy is needed before this is production-grade.
- **`relating_hexagram` is wrong.** It is a *random* hexagram, not the primary with changing lines flipped (`engine.ts:109-111` discards the correctly-computed `relatingLines`). The transformed-hexagram visual (§4/§5 target) will be meaningless until this is fixed.
- **Changing-line odds are off.** Each line is uniform over {6,7,8,9} (`engine.ts:91`), giving P(changing)=1/2, not the canonical three-coin 1/4 — and `method` is recorded but never affects the cast (`yarrow_stalks` behaves identically).
- **`guidance` (stub) is vestigial** — present in `IChingResultSchema` only; neither producer nor renderer uses it. Treat as a stale example field.
