# Codex Dark render set

This set contains the 27 darker generated PNG originals selected as the brand direction on 29 October 2026: 17 engines, six workflows, three voice plates and the trinity-hex mark. The metal is dark matte patinated bronze, with gold held to fine lines and emerald glass accents on a dark ground. The brighter [first pass](../codex/) remains a historical reference.

`prompts.json` preserves each subject, requested aspect, dimensions and prompt. Engines and workflows are 1024 × 1536 pixels, voice plates are 1536 × 1024, and the mark is 1024 × 1024. The PNG originals are committed in ordinary Git. They retain their original metadata and are not the JPEG derivatives mentioned in earlier design-session notes.

The historical generation notes describe a Codex CLI image run using the first pass's `engines__04-vimshottari.png` as a style reference. They do not establish a currently available model or provider route.

`measure-lens.py` is an optional offline helper requiring Pillow and NumPy. It estimates the emerald-glass centre as percentages of image width and height:

```sh
python3 measure-lens.py engines__04-vimshottari.png
```

The detector is a heuristic and can return no result. Review a coordinate against the image before using it for animation. Historical component coordinates and browser checks belong to the separately archived design session; this source set does not claim those components are installed or accepted on current main.
