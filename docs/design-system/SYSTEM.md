# Noesis Design System — Hybrid Tri-Voice Architecture

**Status:** v0.1 (post-moodboard direction freeze, pre-component-library build)
**Authority:** `brand-docs-final/tryambakam-noesis-aleph/06-visual-identity.md`
**Locked direction:** Hybrid — three voices, one system.

---

## The Three Voices

The product surface is not monolithic. Different cognitive tasks demand different visual registers. The brand's three voices are not "themes" — they are **functionally distinct surface modes** that share tokens but diverge in expression.

| Voice | Source moodboard | Where it lives | What it does |
|---|---|---|---|
| **Instrument** | C — Living Instrument | Engine surfaces, dashboards, the active product UI, the 17-engine grid, biofield-web capture UI | Sacred geometry as load-bearing HUD chrome. Information-dense. Operator-facing. |
| **Folio** | B — Codex Folio | Long-form reading detail, synthesis pages, about, settings, narrative content, exported PDFs | Illuminated-manuscript register. Calm, lineage-aware, paragraph-friendly. |
| **Mandala** | A — Sacred Mandala | Brand mark, splash, loading states, app shell, marketing site, empty states | The still center. Lowest density. Highest reverence. |

**Rule of voice selection:** Pick the voice that matches the cognitive mode the surface is asking the user to enter.
- *Operating* → Instrument
- *Reading* → Folio
- *Beholding* → Mandala

Never mix two voices on the same screen. Transition between voices is a navigation event, not a styling choice.

---

## Shared Tokens (all three voices)

### Color — Goethe Spectrum

```
--void-black:        #070B1D    /* Instrument BG, Mandala BG */
--witness-violet:    #2D0050    /* depth, threshold, contemplation */
--flow-indigo:       #0B50FB    /* primary action, active state */
--sacred-gold:       #C5A017    /* emphasis, gilding, the gilt-edge */
--coherence-emerald: #10B5A7    /* signal, alive, coherence */
--parchment:         #F0EDE3    /* Folio BG, light surface */

--ink-bronze:        #5A4A2E    /* Folio body type only */
--ink-iron:          #1A1A24    /* high-contrast type on parchment */
```

**Light/dark inversion:**
- Instrument & Mandala = void-black canvas, light glyphs
- Folio = parchment canvas, ink-bronze body, ink-iron headings

### Typography

```
--font-display-instrument: 'Orbitron', 'Eurostile', system-ui   /* HUD labels */
--font-display-folio:      'Cinzel', 'Trajan Pro', serif        /* illuminated heads */
--font-display-mandala:    'Cormorant Garamond', serif          /* reverent display */

--font-body:    'Inter', system-ui                              /* universal body */
--font-serif:   'Crimson Pro', 'Iowan Old Style', serif         /* Folio body */
--font-mono:    'JetBrains Mono', 'IBM Plex Mono', monospace    /* code, data, HUD ticks */
```

### Sacred Geometry — Load-Bearing Architecture

The 12 imported sigils (`/icons/raw/`, `/icons/cropped/`) are not decoration. They are **structural elements** with assigned semantic roles:

| Sigil | Slot | Used as |
|---|---|---|
| `cover-trinity-hex` | brand mark | logomark, loader |
| `cover-alt-1-vesica` | brand alt | favicon, social cards |
| `cover-alt-2-eye` | brand alt | error / 404 still-eye |
| `part-1-convergence` | STABILIZE axis | engine card frame |
| `part-2-sri-yantra` | HEAL axis | engine card frame |
| `part-3-metatron` | CREATE axis | engine card frame |
| `part-4-compass` | MUTATE axis | engine card frame |
| `part-5-wealth-mandala` | STABILIZE | section divider |
| `part-6-vesica-piscis` | HEAL | section divider |
| `part-7-chakra-column` | CREATE | spinal nav |
| `part-8-root-tree` | MUTATE | lineage/history view |
| `part-9-dna-helix` | STABILIZE | continuity ribbon |

**Hard rule:** Every sigil placement must answer "what does this glyph *do*?" If the answer is "looks nice" — remove it.

### Spacing & Geometry

- Base unit: **8px**
- Sacred ratios: **1, φ (1.618), √2 (1.414), 3, 5, 8, 13, 21** — composition uses Fibonacci, not arbitrary multiples
- Hex grid (Instrument): 120px flat-to-flat at 1× zoom
- Folio column: 680px max, 1.5 leading, 1em paragraph indent

### Motion

- Instrument: 180ms `cubic-bezier(0.2, 0.8, 0.2, 1)` — instrument-precise
- Folio: 320ms `cubic-bezier(0.4, 0, 0.2, 1)` — page-turn cadence
- Mandala: 720ms `cubic-bezier(0.4, 0, 0.4, 1)` — breath
- Respect `prefers-reduced-motion` everywhere

---

## Voice-Specific Surface Rules

### Instrument Voice (C)

- Canvas: void-black with subtle hex grid at 4% opacity
- Primary chrome color: coherence-emerald + flow-indigo
- Accent: sacred-gold for active/selected state only
- Type stack: Orbitron (display) + Inter (body) + JetBrains Mono (data)
- Sigil treatment: line-art, glowing edge, never filled
- Information density: high (operator UI, not marketing)
- Interactive feedback: pulse + edge-glow, not scale
- Forbidden: bento-box rectangular cards. All containers are hex, vesica, or radial.

### Folio Voice (B)

- Canvas: parchment with 2% paper-grain texture
- Primary chrome: ink-bronze
- Accent: sacred-gold (as actual gold leaf, not yellow)
- Type stack: Cinzel (heads) + Crimson Pro (body) + JetBrains Mono (sidenotes)
- Sigil treatment: filled ink illustration, illuminated-capital style
- Information density: paragraph-led, generous margins
- Drop caps mandatory on long-form section openings
- Forbidden: any neon glow, any HUD chrome, any sci-fi typography

### Mandala Voice (A)

- Canvas: void-black, near-empty
- Primary chrome: line-art sigil + Goethe palette strip
- Type stack: Cormorant Garamond (display) + JetBrains Mono (labels)
- One central glyph per surface. No competing geometry.
- Used for: splash, loading, 404, empty states, brand identity, marketing hero
- Forbidden: high information density. If you need >40 words on screen, you've picked the wrong voice.

---

## Canonical Pattern · DyadChamber (guided flows)

**Status:** Canonical. Use for every guided flow in the product.

The DyadChamber pattern replaces every "generic form header" surface
in Noesis. It manifests the framework's central thesis — that
interpretation happens at the meeting of **two witnesses** — as
literal UI architecture. Two character portraits flank the form
content: **Pichet** on the left (embodied / structure / "the bone"),
**Aletheios** on the right (witness / flow / "names you into being").
The character owning the current cognitive step lights forward; the
other dims back to watching. At dyad-fork moments (joined steps),
both are equally lit.

### Where it lives
- React: `src/components/dyad/` — `DyadChamber`, `StepIndicator`,
  `WitnessFigure`, `SigilToken`
- Portrait assets: `public/depth-reading/characters/{pichet,aletheios}-front.png`
- Keyframes: `app/globals.css` under the DYAD-CHAMBER section
  (`thresholdPulse`, `thresholdRingDrift`, `thresholdStepFade`)
- Reference implementation: `app/get-reading/page.tsx`

### When to use
- Any multi-step form (birth-data intake, biofield session intake,
  settings save, profile setup)
- Any moment where the user is being asked to commit to a depth
  rather than just pick a value
- Onboarding flows where the framework's voice should be present
- Anywhere we would otherwise default to a generic form header
  with a logo and a step counter

### When NOT to use
- The 17-engines honeycomb (Engine Mandala) — that surface is its
  own architectural primitive, not a guided flow
- Long-form reading detail (Folio voice — different cognitive mode)
- Empty / loading / 404 states (Mandala voice — too quiet for a dyad)

### API at a glance

```tsx
import {
  DyadChamber,
  StepIndicator,
  type DyadStep,
} from "@/components/dyad";

const STEPS: ReadonlyArray<DyadStep> = [
  { speaker: "aletheios", symbol: "vesica" },
  { speaker: "pichet",    symbol: "hex" },
  { speaker: "both",      symbol: "trinity" },
];

<div style={{ position: "relative", minHeight: "100vh" }}>
  <DyadChamber
    speaker={STEPS[stepIndex].speaker}
    submitting={isSubmitting}
  />
  <StepIndicator
    steps={STEPS}
    currentIndex={stepIndex}
    onJump={setStepIndex}
  />
  {/* your form content here, above z:0 chamber */}
</div>
```

### Speaker semantics (do not break these mappings)

| Speaker | Color | Symbol | Owns |
|---|---|---|---|
| `pichet` | Sacred Gold `#C5A017` | `hex` | Coordinates, structure, anchor moments — "the bone" |
| `aletheios` | Coherence Emerald `#10B5A7` | `vesica` | Naming, attunement, flow, witness — "names you into being" |
| `both` | Parchment `#F0EDE3` | `trinity` | Joined fields, choice moments, dyad forks |

Voice attribution eyebrows (`SPEAKER_LABEL[speaker] + ":"`) above each
prompt name the speaking witness. This is non-negotiable — it teaches
the user that two distinct intelligences are reading them.

### What this pattern is NOT
- Not a wrapper component you drop in to "make a form look nice"
- Not an aesthetic option — it's a framework commitment
- Not optional decoration — the dyad IS the navigation chrome,
  the same way sacred geometry IS the engine grid (not an icon)

---

## Transition Choreography

Navigation between surfaces of different voices uses a 320ms cross-fade through a `mandala` intermediate state. The transition is not chrome — it is **the act of switching cognitive mode**.

```
Instrument → Mandala (160ms void breath) → Folio
```

This is a feature, not a perf cost. It teaches the user that the voices are different rooms in the same temple.

---

## Component Library Tiers (mockup generation plan)

### Tier 1 — Anchor (8 mockups, validate voice first)

1. `T1-01_brand-mark-mandala` — the logomark + loader sequence (Mandala voice)
2. `T1-02_splash-mandala` — first-paint splash screen (Mandala)
3. `T1-03_engines-index-instrument` — the 17-engine grid (Instrument, replaces bento)
4. `T1-04_engine-detail-instrument` — single engine surface (Instrument)
5. `T1-05_reading-detail-folio` — long-form reading page (Folio)
6. `T1-06_navigation-system` — primary nav + voice-switch behavior
7. `T1-07_empty-state-mandala` — empty/loading/404 (Mandala)
8. `T1-08_biofield-capture-instrument` — biofield-web PIP capture UI (Instrument)

→ **STOP. User reviews. Sign off on voice / proportion / sigil placement.**

### Tier 2 — Engine surfaces (17 mockups, one per engine)

One Instrument-voice mockup for each of the 17 engines, demonstrating how sacred geometry expresses that engine's semantics. Generated after Tier 1 sign-off.

### Tier 3 — Component vocabulary (~20 mockups)

Buttons (5 states × 3 voices), cards/containers, modals, forms, charts/data viz (HUD style), CTAs, status pills, tooltips, scrollbars, focus rings, nav patterns, breadcrumbs.

### Tier 4 — Cross-surface flows (~10 mockups)

User journey storyboards: onboarding, daily reading, biofield session, reading export, profile, settings. Shows how Instrument↔Folio↔Mandala compose into a real session.

**Total: ~55 mockups. Budget: ~3 hours of background gen, in 4 tranches with sign-off between each.**

---

## What this system replaces

- The current bento-box engine grid (~5% of brand vision)
- All decorative-only sigil usage anywhere in the app
- Generic Tailwind card / rectangle UI patterns
- "Just a dark theme" — this is not a dark theme; it's a tri-voice system

## What this system does NOT introduce

- Skeuomorphic textures pretending to be paper (Folio uses *real* parchment-feeling typography & spacing, not photo-textured CSS)
- Anime mecha aesthetic (Instrument is HUD-precise, not gamer-flashy)
- Drop shadows as primary depth tool (depth comes from edge-glow and layered geometry)
