# Noesis anatomy figures — shared brief for builders

You are building one interactive isometric figure with the **anatomy** skill. Read, in this order, and follow them exactly:

1. `docs/design/figures/.references/../kit` is the kit (copied from github.com/wheresryan22/anatomy). The skill text is at `/private/tmp/claude-501/-Volumes-madara-2026-Projects-tryambakam-noesis-Selemene-engine/c0dd7e0d-d0da-42a2-8673-72251f695d90/scratchpad/anatomy-repo/SKILL.md`; its references are in `docs/design/figures/.references/` (craft.md, motion.md, verify.md, kit.md); its check scripts are in `docs/design/figures/.scripts/` (capture.mjs, drive.mjs, lines.mjs). Google Chrome is installed at /Applications/Google Chrome.app. Node is v26.
2. The closest example to copy the structure from: `/private/tmp/claude-501/-Volumes-madara-2026-Projects-tryambakam-noesis-Selemene-engine/c0dd7e0d-d0da-42a2-8673-72251f695d90/scratchpad/anatomy-repo/examples/dial-indicator/build.mjs` (framework-free; imports the kit, writes a standalone page). For a Hero with a belt and several stations, also read `examples/arcade-cabinet/build.mjs` there.
3. The Render Bible: `docs/superpowers/specs/2026-10-28-noesis-render-bible-design.md`, sections "The world", "Anatomy figures" and the "Settled after the builder check" calls. Those calls win over the tables where they disagree.

## Brand rules that bind every figure

- **Themes.** Dark theme first, on void. Override the kit's CSS variables so the page uses the brand tokens: page/background `#070B1D`, paper (part fill) a dark bronze `#2A2418`, the four side tones from `#1A1611` (lo) through `#3A3222` and `#5A4A2E` (ink-bronze) to `#7A6840` (hi), top faces `#8A7850`, lines `#C5A017` (sacred-gold) at the kit's 0.6px, the single **lit** accent `#E6B84D` (gold-warm), muted/halo `rgba(197,160,23,.22)`, text `#F0EDE3`, metadata `#AEBBC4`. Light theme on parchment `#F0EDE3` with ink `#5A4A2E` and `#1A1A24`, lit accent `#76652A`. Build both (`--light`).
- **Material.** Patinated bronze with gold as line. No neon, no blue or purple, no rectangles as containers where a hexagon, vesica or ring can hold the thing. Glass appears only where the Render Bible says.
- **Type in the card.** Card title and readout in the kit's defaults are fine; if you set a font, use `Cinzel` for the title (Google Fonts link is allowed) and a monospace readout.
- **Truth.** Every id, count and stage name in the readout comes from the repo, not from memory: engines from `docs/ENGINES.md` (17, in that order), workflows from `crates/noesis-orchestrator/src/workflow/registry.rs`, witness stages per the Render Bible call (verify against `crates/noesis-witness/src/*.rs`).
- **Motion.** Calm, time-based, eased; idle tour after 1.4 s; still under reduced motion; stops offscreen. See `.references/motion.md`.
- **Verify like a critic.** Capture desktop and 390px phone, hover, and two moments of motion with `.scripts/capture.mjs`; run `.scripts/lines.mjs` on the dark and light pages to zero line ends; zoom to 4x on the subject and add the missing real parts. Read every PNG you make.

## Deliverables (write only inside your own folder `docs/design/figures/<name>/`)

- `build.mjs` — imports `../kit/iso-kit.mjs` (relative), builds the page(s). `node build.mjs` writes `<name>.html`; `node build.mjs --light` writes `<name>-light.html`.
- `<name>.html`, `<name>-light.html` — standalone pages, committed outputs.
- `preview.html` — the dark standalone page again, but with this exact first line prepended: `<!-- @dsCard group="Figures" height=520 -->` (the design system reads it).
- `shots/` — the capture PNGs.
- `README.md` — one paragraph: the object, what moves and what the motion means, the truth sources of the readout, the size tier, and the audit/line-check results with their numbers.

Do not commit. Do not touch any file outside your folder. Do not edit the kit. If the kit's API surprises you, read `.references/kit.md` rather than guessing.
