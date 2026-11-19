# Noesis brand work: handoff

Written 2026-10-29 by the first session. Updated 2026-10-29 by the second session, which finished the list below. Read this first, then the Render Bible it points at.

## Where things stand

**Design system**, https://claude.ai/artifact/7sRZQq1obgFTvmEh6kKU6K, now version 14 (it was 12 at the start of this session):

- `Instruments`: the Higgsfield set, 27 stills and 18 loops, 4 direction draws, 3 voice plates, the mark. Eight loops are dropped, not owed. This is now the earlier set, kept for reference.
- `Instruments Codex`: the first Codex pass, 27 stills, brighter gold. Kept for reference. Not the brand look.
- `Instruments Codex Dark` (new): the darker redo, 27 stills, on the Render Bible rule. **This is the brand look.** The owner chose it on 2026-10-29.
- `Components`: EngineMandala, DyadChamber, WitnessPipeline, CompassBalance (anatomy figures, live readouts), InstrumentMotion (a still animated in code, with lens positions per still), DirectionFrame (new), MarkLockup (new), HexCell (now also a still form), Cover (recut with the dark mark).
- Brand book (`project/README.md`) names Instruments Codex Dark as the brand library and the Instruments and Instruments Codex groups as reference. Icons is marked retired to history.

**Repo**, committed on `codex/selemene-capability-consumer-surface` before this session: `7295464c` (bible, directions, figures), `61ae02d0` (plugin audit docs), `ea96569b` (job description PDF).

**Repo**, uncommitted (nothing committed this session; the owner has not asked for a commit):

- `docs/design/renders/codex/`: the 27 first-pass PNG originals (about 57 MB), prompts and README.
- `docs/design/renders/codex-dark/`: the 27 darker PNGs (about 55 MB), `prompts.json`, `README.md` and `measure-lens.py`.
- This file.

The PNGs are large. Decide whether they belong in git (for example with Git LFS) before committing either folder.

**Audit page**, https://claude.ai/artifact/1fBB1v2ScLXN94dCR11xAk, has the family review and the set review.

## Decisions already taken

- The Render Bible sets the world: void ground, patinated bronze, gold as line, one emerald glass per object, a hexagonal plinth. Follow it unless the owner overrides it.
- The look is Instruments Codex Dark (owner decision, 2026-10-29). The Codex set as first published is not the brand look.
- Motion for stills is code, not video. Parts that move are built as anatomy figures. The InstrumentMotion layer is light and tilt only.
- The eight Higgsfield loops are dropped. Do not resume the Higgsfield loop run.
- Icons and Generated v3 are history. Do not delete them.
- The brand mark is the dark trinity-hex medallion in Instruments Codex Dark (`mark/trinity-hex.jpg`). The Codex and Higgsfield medallions are history.

## Finished on 2026-10-29

1. **Originals saved.** The first-pass PNGs are in `docs/design/renders/codex/`, copied from the scratch folder with checksums matched.
2. **Look decided.** The owner chose the darker redo. Regenerated 27 stills through the Codex route into `docs/design/renders/codex-dark/`, same subjects, metal darker and matte, gold as line. Prompts in `prompts.json`. Vimshottari first-pass still used as style reference.
3. **Lens positions.** Measured for all 27 with `measure-lens.py` (an emerald-glass detector), checked by eye on a contact sheet. Recorded in the InstrumentMotion README as a table. Where a still has no single emerald glass, the value is the object's centre. The folio plate has no lens and is not used with InstrumentMotion.
4. **Components published.** DirectionFrame (the four direction SVGs in a vesica, using the existing Instruments direction files), MarkLockup (the dark medallion with the wordmark set in code, three forms), HexCell (still form: the still clipped to the hexagon with a gold frame; the line-art cell is unchanged), Cover recut (dark medallion, wordmark, tagline, no indigo or parchment blocks).
5. **Brand book status lines.** Instruments status now says eight loops dropped, not owed. Instruments Codex now says not the brand look. Brand book names the dark set as the brand library.
6. **Icons retirement note.** Added to the Icons README: sigils are history, the brand mark is the dark medallion, and the sigils are not deleted.
7. **InstrumentMotion verified in a browser.** Checked at 375 px (phone) and the desktop pane. No horizontal scroll. Pointer tilt moves the still by a few degrees. The lens sits at the measured coordinate in the DOM. Reduced motion was checked with a copy of the page that forces the reduce branch: the layers are hidden, there is no tilt, and the still shows. A real OS reduced-motion setting was not toggled.
8. **Phone width.** MarkLockup, DirectionFrame, InstrumentMotion and HexCell fit 375 px. The Cover is a fixed 960 × 300 banner, as the previous cover was.

## Still open

- **EngineCard and VoicePlate** were drafted but never published. They are not in the design system. Not done this session.
- **Render Bible in the design system** (`project/render-bible.md`) still describes the Higgsfield set as the library. Not updated this session.
- **Re-check the anatomy figures' bronze tones** against the dark set. The four figures were darkened after the first builds. Not re-checked.
- **Repo docs and the commit.** Nothing is committed. Decide on git handling for the PNGs first.
- **Mark at 48 px.** The lockup shows the medallion at 48 px in the small form. The 48 px test on the exported file was not run as a separate check.

## Pointers

- Render Bible: `docs/superpowers/specs/2026-10-28-noesis-render-bible-design.md`
- Direction draws and generator: `docs/design/directions/`
- Anatomy figures and kit: `docs/design/figures/` (`BRIEF.md` is the builder brief)
- Codex first-pass prompts and originals: `docs/design/renders/codex/`
- Codex Dark prompts, originals and lens tool: `docs/design/renders/codex-dark/`
- Codex route: `codex exec` with `--sandbox workspace-write`, `-i <style reference>`, prompt after `--`. The gateway's `codex/gpt-image-2` route has no provider credentials, so it does not work.
- Design system groups uploaded this session: `Instruments Codex Dark` (27 JPEGs at 1400 px). Asset ids are in the design system index.
