Tryambakam Noesis is a witness instrument: clinical precision at visionary scale — "the anatomist who sees fractals". The system has **three voices that share one token set**. Pick the voice from the cognitive mode the surface asks for, never mix two on one screen, and treat a voice change as navigation.

| Voice | Mode | Theme | Where |
|---|---|---|---|
| **Instrument** | Operating | `instrument` | Engine mandala, rooms, graph, dashboards, biofield capture, guided flows |
| **Folio** | Reading | `folio` | Long-form readings, synthesis, exported PDFs, about/settings prose |
| **Mandala** | Beholding | `instrument` | Brand mark, splash, loading, empty/404, marketing hero |

## Content fundamentals

- Alternate **3–8 word declarations** with **25–45 word technical traverses**: "One field. 7 rooms." then "Each room opens a lens — a bounded surface over the Selemene engine pool. The graph is the interface at every depth."
- Second person, calm, no exclamation marks, no emoji. Never predict; witness.
- Lexicon — say *field* (not platform/app), *room* (not page/tab), *witness* (not prediction), *Folio* (not archive/library/history), *threshold* (not onboarding/signup), *instrument* (not dashboard), *engine* (not model/algorithm), *the question is active* (not "get started").
- Banned: journey, path, healing, manifesting, "AI" as a feature, artificial intelligence, Archive, Library.
- Load-bearing lines, verbatim: **"See the pattern. Keep the authority."** · **"Source before model. Consent before relation. Privacy before convenience."** · "Insight without overclaim." · "Enter when the question is active." · "The system succeeds when you no longer need it."
- Labels and eyebrows are uppercase and tracked; prose is sentence case.

## Color

- Instrument and Mandala sit on `void`; panels on `surface` (`bg-raised`). There is no light mode — the `folio` theme is the reading room, applied to the content area only.
- `sacred-gold` is structure: line, type, glow, the active diamond. Never a large fill. On parchment it is a mark only; Folio's gold text is `accent` (folio-rule `#76652A`).
- `flow-indigo` is flow and the single primary action fill (text on it: `on-flow`). It is 3.3:1 on void — never small text there.
- Evidence colors carry analytical truth: `evidence-computed` (emerald), `evidence-witness` (violet-glow), `evidence-unresolved` (terracotta); label text uses `evidence-copy-*`. Interaction colors (`accent`, `selected`, `focus`) describe navigation, never truth. Always pair evidence color with its word.
- `witness-violet` is a depth ground (Union Mirror, Bridge Query), not a text color.
- Text: `text-primary` → `text-secondary` → `text-metadata`; all hold ≥6.4:1 on their grounds in both themes. Hard-code no hex outside tokens.

## Typography

- **Engraving — Cinzel**: wordmark, page titles, hero, Folio chapter heads. Engraved Roman caps, wide tracking (`page-title` 0.28em).
- **Display — Panchang**: section heads (`h2`), node and hex labels, eyebrows, epithets, buttons.
- **Sans — Satoshi**: prose (`body`, 70ch), UI, metadata.
- **Folio serif — Crimson Pro** for `folio-body`; **Mandala — Cormorant Garamond** for `mandala-wordmark`; **Mono — JetBrains Mono** for `data`, sidenotes and HUD ticks.
- Seven size stops only: 0.75 · 0.875 · 1.0625 · 1.1875 · 1.5 · 2.25 · 3.25rem. Reaching for "the next size" lands on a stop.
- Panchang and Satoshi are Fontshare faces; where they cannot load, Unbounded and Manrope stand in.

## Space, shape, depth

- 8px base on Fibonacci steps: `space-1` 8 · `space-2` 16 · `space-3` 24 · `space-5` 40 · `space-8` 64 · `space-13` 104 · `space-21` 168. Every target ≥ `target-min` 44px.
- Three radii: `radius-tile` (reading tiles), `radius-card` (panels, dialogs), `radius-pill` (every button, chip, input, tab). Nothing renders as a hard block.
- In the Instrument grid, containers are hexagons (`hex-cell` 120px), vesicas or rings — never bento rectangles. A hex grid underlays the field at `hex-grid` opacity.
- Depth is edge-glow and layered geometry first; shadows are rare, directional and cast from the void (`shadow-capsule`, `shadow-popover`, `shadow-dialog`, `shadow-dock`).
- Folio column `measure-folio` 680px; dividers are hairline–diamond–hairline.

## Motion

- One curve: **`cubic-bezier(0.19, 1, 0.22, 1)`** (out-expo). Reveal 1000ms, snap 300ms, route crossfade 180/220ms.
- Nodes condense out of the hub (scale 0.55 → 1); spokes draw outward; the starfield breathes in once.
- Voice changes cross through a 160ms void breath (Instrument → Mandala → Folio).
- Everything collapses to static under `prefers-reduced-motion`.

## Sacred geometry and iconography

- Sigils are load-bearing: every placement must answer "what does this glyph do?" — if the answer is "looks nice", remove it.
- Brand mark: the **trinity-hex** (three interlocking circles in a hexagon; medallion `assets/Instruments Codex Dark/mark/trinity-hex.jpg`, set with the wordmark by MarkLockup; earlier emblem in `assets/Icons`, retired to history). Alternates: vesica (favicon, social), still eye (404). The shipped favicon is the gold **diamond star** (`assets/Marks/noesis-diamond-mark.svg`).
- The 12 sigils live in `assets/Icons` with their slots; they are rendered emblems for Mandala and hero use on void, while small Instrument chrome uses line-art glyphs drawn in code.
- The brand library is `assets/Instruments Codex Dark`: one dark bronze instrument per engine, one station per workflow, three voice plates and the mark, all in one world (void, one key light, patinated bronze with gold as line, one emerald glass, a hexagonal plinth). The four direction draws are the gold-line SVGs in `assets/Instruments/directions`. The rules that made them are the Render Bible section. `assets/Instruments` (the Higgsfield set) and `assets/Instruments Codex` (the first Codex pass) are kept for reference and are not the brand look. Earlier renders in `assets/Icons` and `assets/Generated v3` are history.
- Four organs have interactive figures under Components → Figures: EngineMandala, DyadChamber, WitnessPipeline, CompassBalance. Their readouts come from the live registry.
- Motion for stills is code. InstrumentMotion animates light and tilt on a still and never moves parts; DirectionFrame draws the four directions; HexCell has a still form for the honeycomb; MarkLockup sets the mark with the wordmark.
- Axis frames: convergence (STABILIZE), sri yantra (HEAL), metatron (CREATE), compass (MUTATE).
- Instrument: line art, glowing edge, never filled, 1.1px stroke. Folio: filled ink illustration, illuminated-capital style. Mandala: one central glyph, nothing competing.
- The diamond (`nx-diamond`) is the smallest sigil: nav separator, active marker, dialog corners.
- No emoji; no generic icon fonts in brand surfaces.

## Patterns

- **DyadChamber** — every guided flow is held between two witnesses: Pichet (gold, hex, structure) and Aletheios (emerald, vesica, naming). The speaking one lights; the other dims. See DyadSpeaker.
- **Engine Mandala** — the honeycomb of engines with Witness at the centre. See HexCell.
- **Mandala surfaces** hold one glyph and ≤40 words.
