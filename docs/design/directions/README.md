# Compass directions — gold-line draws

The four compass directions of Tryambakam Noesis as code, per the Render Bible (`docs/superpowers/specs/2026-10-28-noesis-render-bible-design.md`): gold line (`sacred-gold` #C5A017) on void (#070B1D) inside a vesica frame, one emerald lens (`coherence-emerald`) at the centre. Each SVG draws itself over about five seconds with a stroke-dash animation, holds, and loops; under `prefers-reduced-motion` it shows complete.

| File | Direction | Geometry |
|---|---|---|
| `convergence.svg` | STABILIZE | six logarithmic spiral arms converging on the lens |
| `sri-yantra.svg` | HEAL | nine interlocking triangles, two rings, 8 and 16 petal rows |
| `metatron.svg` | CREATE | the fruit of life, thirteen circles with every centre joined, in a hexagon |
| `compass.svg` | MUTATE | eight-point rose, three rings, 32 ticks |

Use them with `<img>` or inline. Static PNG renders of the completed state live in the design system under `assets/Instruments/directions/`. The geometry is generated, not hand-drawn; regenerate with `generate.py` in this folder if the tokens change.
