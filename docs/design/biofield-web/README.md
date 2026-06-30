# Noesis Biofield — Visual System (frozen)

> **Note (2026-06-30):** `apps/biofield-web` has been retired from this repo.
> This design reference is preserved; production UI is being implemented in
> [Sankalpa](../../sankalpa/). Paths referencing `apps/biofield-web` are
> historical.

Brand-grounded design reference for biofield-web. **Build against these before writing UI.**

## Source of truth
`brand-docs-final/tryambakam-noesis-aleph/06-visual-identity.md` + `14-visual-prompt-cookbook.md`.
Generated with the gpt-image-2 skill, grounded in the real API contracts (historically `apps/biofield-web/src/components/pip/types.ts`, `lib/selemene/biofield-domain.ts`) and the brand's Goethe/Kha-Ba-La system.

## Palette (Goethe's Zur Farbenlehre — polarity, not spectrum)
| Color | Hex | State / Kha-Ba-La |
|---|---|---|
| Void Black | `#070B1D` | Source (La) — primary canvas |
| Witness Violet | `#2D0050` | Observer (Kha) |
| Flow Indigo | `#0B50FB` | Flow (Kha→Ba) — data streams |
| Sacred Gold | `#C5A017` | Activation (Ba) — accents/CTAs (antique, NOT bright `#FFB347`) |
| Coherence Emerald | `#10B5A7` | Coherence (Ba↔La) — success/health |
| Parchment | `#F0EDE3` | Primary text (not white) |
| Muted Silver | `#8A9BA8` | 1px hairlines, secondary text |
| Terracotta | `#C65D3B` | Errors only (rare) |

Gradients: **Kha Arc** `#070B1D→#2D0050→#0B50FB` (canvas) · **Ba Arc** `#10B5A7→#C5A017` (interactive) · **La Arc** `#C5A017→#2D0050→#070B1D` (completion).
Type: **Panchang** (display) · **Satoshi** (body) · **SF Mono** (data/breath). φ=1.618 scale.

## Non-negotiable principles
1. **Sacred geometry AS data visualization** — compass / mandala / waveform / constellation grid. Load-bearing, not decoration.
2. **Data as sacred form** — metrics render as arcs/rings/mandalas; HRV as a waveform overlay; breath at 4:7:8. No dashboard cards.
3. **Bioluminescent** — light from within; Void Black canvas; no external glows.
4. **Banned (master negative):** SaaS dashboard chrome, boxy cards, progress bars, pill buttons, rounded corners, neon excess, wellness/pastel, third-eye/lotus cliché, stock photos, faces, clinical white.

## Sheets
| File | Component |
|---|---|
| `00-moodboard.png` | Brand system board (palette / type / sigil / data-as-sacred-form) |
| `02-cosmogram-spec.png` | Cosmogram ring (COH/SYM/LUM/REG) |
| `03-witness-dyad-spec.png` | Witness Dyad (Aletheios / Pichet / synthesis) — `witness_layer` |
| `04-scorecard-spec.png` | Metric Node (arc-ring, not a card) — `CompositeScores` |
| `05-metrics-panel-spec.png` | Biofield Mandala (Energy/Geometry/Chaos rings) — 11 `BiofieldMetrics` |
| `06-fractal-chaos-spec.png` | Fractal/Chaos signature — fractal/correlation dim + entropy |
| `07-quality-gauge-spec.png` | Capture Quality radial — `QualityAssessment` |
| `08-consciousness-spectrum-spec.png` | Consciousness level — 5-state Goethe spectrum |
| `09-capture-compass-spec.png` | Capture controls + capture-state stepper |
| `10-result-mandala-spec.png` | Reading result + baseline deltas — `BiofieldMetricDelta` |
| `11-foundations-spec.png` | Buttons / HRV waveform graph / session strip |

(`01` intentionally omitted — an off-brand SaaS draft, discarded.)
