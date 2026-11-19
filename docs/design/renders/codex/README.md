# Codex render prompts

`prompts.json` holds the 27 prompts used for the first Codex render set (`Instruments Codex` in the design system): 17 engines, 6 workflows, 3 voice plates and the mark. Each entry has an `id`, a `size` (aspect), a `px` size and the `prompt`.

Generation ran through the Codex CLI's image generation, one `codex exec` per item, with the Vimshottari still attached with `-i` as the style reference. The prompt goes after `--` so the image flag does not swallow it. Output was saved as PNG, then converted to 1400 px JPEG for publishing.

This first pass is superseded. The brand look is the darker redo in `../codex-dark/`: the same subjects, darker metal, gold as line.

The 27 PNG originals are in this folder too (copied from the session scratch folder on 2026-10-29, checksums verified against the source). They are about 60 MB in total. Decide whether they belong in git before committing.
