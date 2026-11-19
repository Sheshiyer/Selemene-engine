# Codex Dark render set

The darker redo of the Codex render set (`Instruments Codex Dark` in the design system, version 14). It is the brand look: the owner chose it on 2026-10-29 over the first Codex pass, because the metal is pushed darker and matte and gold is held to hairline line, as the Render Bible asks.

`prompts.json` holds the 27 prompts. They are the first Codex pass prompts (`../codex/prompts.json`) with two changes: the base block says the metal is dark matte patinated bronze with gold as line only, and the four plates and the mark (voices and mark) get the same metal line. The reference is the Vimshottari still from the first pass (`../codex/engines__04-vimshottari.png`), used for void ground, light direction, plinth and glass. Its metal is brighter than the brief, so the prompt asks for darker metal than the reference.

Generation ran through `codex exec` with `--sandbox workspace-write`, `-i <reference>` and the prompt after `--`, four at a time, one retry each on a failed file. All 27 succeeded on the first attempt.

The PNG originals are here (27 files, about 60 MB in total). Each is 1024 px on the long side for engines and workflows, 1024 square for the mark, 1536 wide for the voices. The design system carries 1400 px JPEG versions (`sips -Z 1400`, quality 85).

`measure-lens.py` finds the emerald glass in each still and prints its centre as a percentage of width and height, which gives the `--lx` and `--ly` values for InstrumentMotion. It is a detector, not a truth: where a still has no single emerald glass the recorded value is the centre of the object, and every value was checked by eye on a contact sheet. The recorded values are in `project/components/InstrumentMotion/README.md` in the design system.

The 27 originals are large. Decide whether they belong in git (for example with Git LFS) before committing this folder.
