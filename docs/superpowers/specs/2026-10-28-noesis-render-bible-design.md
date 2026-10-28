# Noesis Render Bible

Design for regenerating the Tryambakam Noesis visual library as one world. Written 2026-10-28 after the asset audit (https://claude.ai/artifact/1fBB1v2ScLXN94dCR11xAk). Decision taken by Sheshnarayan Iyer: **void is the ground, bronze and gold line are the material.**

## Intent

The library holds six visual worlds and seven marks because each generation pass invented its own ground, light, type and mark. The tokens and brand book already fix the ground and the palette. This document fixes what the tokens cannot: material, light, camera, container, the object for every organ and flow, the prompt grammar that produces them, the motion rule, and where the files land. Every render from here is built from this page, so two renders made months apart sit next to each other as one set.

## Goals

- One rendered object per engine (17), workflow (6), compass direction (4) and voice (3), plus the brand mark rebuilt once in this material, so the seven marks collapse to one.
- A motion version of every still: a short calm loop in which one mechanism moves.
- Anatomy figures (interactive isometric SVG) for the organs a still cannot explain: the engine mandala, the dyad chamber, the witness pipeline, the compass.
- Everything lands in the design system as its own group with a README, and the superseded families stay in Generated v3 as history.

## Non-goals

- Re-cutting the tokens or the three voices. They stand.
- Lifestyle photography. The conservatory scenes stay as Folio and marketing reference and are not regenerated.
- UI mockups. Anchor mockups are a separate track.
- Deleting anything.

## Anti-goals (what would make this a failure even if it technically works)

- A new set that is internally consistent but still disagrees with the tokens (a fourth dark world).
- Any neon, purple haze, blue glow, HUD rectangle, deco frame, daylight or terrazzo in a brand render.
- Words baked into renders. Type is set in code, never painted.
- An engine count that disagrees with the live registry.

## Constraints

- Palette and type are the tokens in the design system (`project/tokens.json`). Hex values below are quoted from there.
- Taxonomy is the live repo: `docs/ENGINES.md` lists 17 engines (11 Rust, 6 TypeScript); `crates/noesis-orchestrator/src/workflow/registry.rs` lists 6 workflows.
- Stills through Higgsfield, model Nano Banana Pro (`nano_banana_pro`) with image references for material lock; fall back to GPT Image 2 for anything that needs a transparent background. Motion through Seedance 2.0 (`seedance_2_0`) from the still as start image, audio off.
- Figures through the anatomy skill (`github.com/wheresryan22/anatomy`), dark theme first, light theme for Folio.
- Stills are 2:3 at 2k for engines and workflows, 1:1 for marks and direction frames, 16:9 for voice plates.

## The world

Every prompt is assembled from these eight lines. They never change between renders.

| Element | Rule | Token |
|---|---|---|
| Ground | Pure void. No gradient, no vignette brighter than the ground, no floor plane except the plinth. | `void` #070B1D |
| Light | One warm key from upper left, about 40° elevation. A thin gold rim from behind right. A soft gold halo under the plinth, nothing else lit. | `gold-warm` #E6B84D |
| Material | Cast bronze with verdigris only in the recesses. Gold as line: filigree edges, engraved rules, ring edges, never a gold mass. | `ink-bronze` #5A4A2E, `sacred-gold` #C5A017 |
| Glass | One colour of glass per object, carrying the object's evidence state: emerald for computed, violet for witnessed, terracotta for unresolved. The default is emerald. | `coherence-emerald` #10B5A7, `violet-glow` #7B68EE, `terracotta` #C65D3B |
| Plinth | A thin bronze hexagonal plinth, flat-to-flat, gold-edged. The only floor. | `hex-cell` |
| Camera | Three-quarter, 20° above, object centred and filling about 60% of the frame. The same lens across the set. | |
| Scale | Engines read as desk instruments, 30 to 60 cm. Workflows read as benches or stations, 1 to 2 m. Marks read as medallions, a hand wide. | |
| Container | Instrument voice: the hexagon (the plinth). Threshold and direction: a vesica or ring of gold line behind the object. Mandala: one ring. Never a rectangle. | |

**Never in frame:** text, numbers, labels, a frame or border, a second object, neon, purple or blue haze, lens flare, daylight, sky, terrazzo, stone, wood, fabric, a human, a rectangle panel, a HUD, a gradient background.

## The one mark

The trinity-hex: three interlocking rings inside a hexagon. In this world it is a bronze hexagonal medallion with the three rings as gold line and a single emerald glass lens where the three overlap. Rebuilt once as a still and once as a loop (the lens brightens as the rings settle). The diamond star favicon stays as the code-drawn glyph. The other five marks retire to history.

## The organs: 17 engines

One object each. The mechanism is the meaning. The object is what the still shows and what the loop moves.

| # | Engine | Object | What moves in the loop |
|---|---|---|---|
| 1 | Panchanga | Five nested bronze rings on one spindle (tithi, nakshatra, yoga, karana, vara), a sun and a moon pointer | The five rings turn at five rates and align once |
| 2 | Human Design | A nine-socket patch loom: nine hexagonal bronze sockets joined by gold wire channels, gate pins along each wire | Channels light along the wire as pins seat |
| 3 | Gene Keys | A cipher drum with 64 teeth and three depths of engraving (shadow, gift, siddhi) | The drum turns one tooth, the deeper engraving catches gold |
| 4 | Vimshottari Dasha | A nine-ring armillary, each ring a planet, a single time index through all nine | The rings precess at nested rates around the index |
| 5 | Numerology | A reduction abacus: columns of gold beads that fall through gates until one bead remains | Beads descend column by column to one |
| 6 | Biorhythm | A three-pen harmonograph: three swinging arms writing three sine traces on a bronze drum | The arms swing, the traces cross |
| 7 | Vedic Clock | A single-hand 24-hour clock face with a hora ring and an organ-meridian ring | The hand sweeps one hour, the hora ring steps |
| 8 | Biofield | A resonance capacitor: a bronze hand plate under an emerald glass dome, coil below | The dome breathes once, brighter then dim |
| 9 | Face Reading | A craniometric armature: bronze calipers on a ring around a featureless bronze mask, twelve zone marks | The calipers close to a reading and open |
| 10 | Nadabrahman | A monochord over a bronze resonator with a sand plate beneath | The string is plucked, the sand forms a figure |
| 11 | Transits | An orrery: planets on bronze arms around a central lens | The arms advance, two planets align |
| 12 | Tarot | A card cabinet of 78 shallow drawers (22 above, 56 below) with one drawer out | One drawer slides open, a blank card lifts |
| 13 | I Ching | A six-bar flip board: six hinged bronze bars, each whole or broken, on a frame with a yarrow counter | Bars flip from one hexagram to the next |
| 14 | Enneagram | A nine-point escapement: a wheel with nine stations and the inner lines as gold wire, a pointer | The pointer moves along the inner lines |
| 15 | Sacred Geometry | A pantograph with a compass arm drawing a flower of life onto a bronze plate | The arm draws one more circle |
| 16 | Sigil Forge | A screw press that strikes a sigil into a bronze blank | The press descends and lifts, the sigil remains |
| 17 | Raaga | A raga clock: a bronze disc with 72 spokes and a sun pointer, seven small bells on its rim | The pointer crosses a spoke, one bell rings |

## The flows: 6 workflows

Workflows are stations where several engines meet. They read larger than engines and show more than one mechanism.

| Workflow | Station | What moves |
|---|---|---|
| birth-blueprint | A natal press: a bronze plate under a glass lens, three engine dies above it (dasha, bodygraph, keys) | The dies descend together and the plate shows one impression |
| daily-practice | A tide bench: a water glass column with a float, a bell, and the vedic clock hand on one shaft | The float rises with the hour, the bell tips once |
| decision-support | A four-pan balance on a hexagonal base, each pan a compass direction | Weights settle, one pan lowers |
| self-inquiry | A dyad mirror: two facing bronze mirrors, one hexagonal (Pichet, gold), one vesica (Aletheios, emerald), a lamp between | The lamp swings, one mirror lights while the other dims |
| creative-expression | The forge bench: the sigil press beside the pantograph, a blank between them | The pantograph draws, the press strikes |
| full-spectrum | Seventeen small mirrors in a ring around one lens | The mirrors turn in sequence to face the lens |

## The compass: 4 directions

The four existing sigils carry the directions and stay: convergence (STABILIZE), sri yantra (HEAL), metatron (CREATE), compass (MUTATE). They are re-rendered once in this material as vesica frames on void so they match the set. Each loop is a slow gold-line draw of the geometry.

## The voices: 3 plates

Three 16:9 plates that stand for the voices in the brand book and decks. Instrument: the armillary on its hexagon on void. Folio: the same armillary as a sepia engraving on parchment, the one render allowed off void. Mandala: the trinity medallion alone, ring behind it, nothing else.

## Witnesses and the triad

Pichet and Aletheios appear only in the dyad mirror station. Kha, Ba and La are not rendered as objects; they are the three rings of the mark.

## Prompt grammar

Every still prompt is the base block, the object line, and the closing block. Nothing else.

```
Base:    A single {object}, a desk instrument cast in patinated bronze, verdigris only in the recesses, fine gold filigree along every edge and engraved gold rule lines, {glass colour} glass {where the glass is}, standing on a thin bronze hexagonal plinth with a gold edge.
Object:  {one sentence naming the parts and the mechanism, from the table}
Closing: Pure void black background #070B1D, one warm key light from upper left, a thin gold rim light from behind right, a soft gold halo under the plinth, nothing else lit. Three-quarter view from slightly above, the object centred and filling 60% of the frame. Photoreal product render, sharp, no text, no labels, no frame, no second object, no neon, no purple or blue light, no daylight, no stone, no fabric.
```

References passed with every still: the trinity-hex sigil (material and finish) and one pass-2 poster (object craft only). The prompt says what to take from each.

Loop prompts are one sentence from the "what moves" column plus: `Camera locked, no zoom, no pan. The light does not change. The motion is slow and settles. Seamless loop.`

## Motion rule

- 5 seconds, 1080p, audio off, camera locked.
- One mechanism moves. Nothing else. No particles, no flares, no colour change.
- Start and end on the still, so the loop is seamless.
- Under reduced motion the still is shown.

## Anatomy figures

Four organs get an interactive isometric figure built with the anatomy kit, dark theme on void, tones mapped to the tokens (base `ink-bronze`, structure bronze, subject gold, the lit accent `gold-warm`), light theme on parchment for the Folio.

| Organ | Figure | Size |
|---|---|---|
| Engine mandala | A honeycomb of seventeen hex cells on a bench with Witness as the centre cell; hover lifts a cell and shows its engine id | Figure |
| Dyad chamber | Two mirrors on a rail with a lamp between; drag the lamp and the speaking mirror lights | Figure |
| Witness pipeline | A bench of four stations (source, inference, conclusion, folio) joined by a belt; a plate moves through and gains an impression at each | Hero |
| Compass | A four-pan balance whose pans are the directions; the readout names where it settles | Figure |

Each figure's readout is true to the model: the mandala shows the real 17 ids, the pipeline shows the real stage names.

## Files and where they land

Design system group **Instruments**, placed after Icons, tile `l`:

```
assets/Instruments/engines/{nn}-{engine-id}.jpg          2:3 2k still
assets/Instruments/engines/{nn}-{engine-id}.mp4          5 s loop
assets/Instruments/workflows/{workflow-id}.jpg|.mp4
assets/Instruments/directions/{direction}.jpg|.mp4
assets/Instruments/voices/{voice}.jpg
assets/Instruments/mark/trinity-hex.jpg|.mp4
assets/Instruments/README.md
```

Engine numbering follows `docs/ENGINES.md`. Anatomy figures are code and go to `components/<Figure>/preview.html` in the design system and to the Selemene repo under `docs/design/figures/`.

## Pilot

Three stills and one loop before the set runs:

1. Engine: Vimshottari Dasha (the armillary; the hardest object and the one with the most prior versions to compare against).
2. Workflow: daily-practice (a station, to test scale).
3. Mark: the trinity-hex medallion (to test the mark in this material).
4. Loop: the armillary.

Pass criteria, judged side by side with the trinity sigil and moodboard A on void: same ground, same light direction, bronze reads as the same alloy across all three, gold is line not mass, one glass colour, no banned element, and the mark at 48 px still reads as three rings in a hexagon.

## Left to the builder

- Exact wording of each object line within the grammar.
- Which pass-2 poster is the craft reference for each engine.
- Retries: up to three per still before changing the object line; a changed object line is recorded in this document.
- The part lists and camera for the four anatomy figures.
- Order of the set run (suggested: engines in registry order, then workflows, directions, voices, mark).

## Calls I made

- Raaga is the seventeenth engine, per `docs/ENGINES.md`; the audit's guess of nakshatra was wrong and is corrected.
- Emerald is the default glass colour because computed evidence is the engines' native state.
- The four direction sigils are re-rendered rather than kept, so the set has one finish.
- The Folio voice plate is the only render allowed off void.

Settled after the builder check (2026-10-28), each one a builder call unless marked:

- **Mandala figure layout.** A 19-cell honeycomb (1 + 6 + 12): Witness at the centre, the 17 engines in registry order spiralling outward from the inner ring, and the last cell empty, labelled "the question". Witness is not an engine; it is the dyad service behind `/api/v1/witness/interpret`, and the empty cell is the user's open question. The readout shows the engine id under the pointer.
- **Witness pipeline stages.** Verified against `crates/noesis-witness/src/*.rs` during the Hero build: the crate's sequence is `generate_witness_prompt` (engine output) → `routing_for_engine` (A / P / A+P) → `interpret_with_llm` with Aletheios and Pichet answering in parallel → a third `synthesis` completion. Folio is not in the crate (it lives in the TypeScript `packages/witness-pipeline`). The figure's four stations are therefore **engine output → routing → interpret → synthesis**; the plate's impression at each is the engine's index, the chosen route, two witnessed lines written at once, and the folded synthesis page.
- **Figure tones and glass.** Figures carry glass only where the still does. The kit's one lit accent is `gold-warm`. The dyad figure is the one exception to "colour only when colour is the subject": Pichet lights gold and Aletheios lights emerald, because the colour is the subject there. Violet and terracotta never appear in a figure.
- **The trinity-hex reference** is the Icons group file `cover-trinity-hex.png` (asset id 697481bebf7533946f8612ad8e75379a), used for finish only. The pilot medallion was generated from it and is now the material reference for everything else; the old sigil retires to history once the set is done.
- **Workflow stations follow the registry membership.** Each station's dies, mirrors or pans are the engines the registry lists for that workflow, no more: birth-blueprint has five dies (numerology, human-design, vimshottari, biofield, face-reading); daily-practice carries panchanga rings, the vedic clock hand, the biorhythm float, a transits arm and the nadabrahman bell; decision-support's four pans are tarot, i-ching, human-design and enneagram with gene-keys as the balance beam; self-inquiry's two mirrors carry gene-keys and enneagram on one, face-reading and biofield on the other; creative-expression's bench holds sigil-forge, sacred-geometry, nadabrahman, numerology and raaga; full-spectrum's ring is the seventeen. The tables above are superseded where they disagree.
- **Loops.** Seedance 2.0 accepts both a start and an end image; every loop passes the still as both, which is how the pilot was made. A loop fails when the mechanism does not complete, when the camera moves, or when the last frame visibly drifts from the still; then retry once, then ping-pong the clip in post. Five seconds is the default; raaga and tarot may take ten.
- **References and paths.** The library lives at `/Volumes/madara/2026/Projects/tryambakam-noesis/brand-docs-final/tryambakam-noesis-aleph/generated-v3/` and in the design system's Generated v3 group; the pass-2 poster for each engine is the one with the matching slug, and engines without one use the vimshottari poster. Higgsfield takes two references per call here; it uploads from a local path.
- **Compass directions** are done in code, not rendered: each sigil is drawn as a gold-line SVG with a stroke animation over a vesica, on void, and ships as the direction's loop. The decision-support pans and the compass figure assign STABILIZE to the left pan, HEAL to the near pan, CREATE to the right and MUTATE to the far pan, reading clockwise from the viewer.
- **Transparent fallback** is used only for the mark, which also ships as a transparent PNG for UI through GPT Image 2 with the pilot medallion as reference.
- **Pass judgment** belongs to the owner, at the pilot and once more after the first five engines. The 48 px mark test is done on the exported file. Budget: one pass per still, up to two retries, which is at most 93 stills and 31 loops; a failed pilot stops the run.

## Pilot result (2026-10-28)

Three stills and one loop were generated with Nano Banana Pro and Seedance 2.0 from the grammar above, with the trinity sigil and a pass-2 poster as references. All three share the ground, light, alloy, gold line, glass colour and plinth. The mark's filigree edge reads heavier than "line"; the next pass of the mark thins the hexagon border to a single gold rule. The loop keeps the camera locked and returns to the still. The files are in the design system under Instruments, marked pilot.
