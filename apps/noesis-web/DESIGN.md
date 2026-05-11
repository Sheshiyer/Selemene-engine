# Noesis-Web Design System

> **Single source of truth** for every visual and interaction decision in `apps/noesis-web`.
>
> Source authorities:
> - Brand identity: `brand-docs-final/tryambakam-noesis-aleph/06-visual-identity.md`
> - Engine-to-geometry mapping: `brand-docs-final/launch-beta-branding/brandmint-run/brandmint/config/engine-visual-mapping.md`
> - CSS implementation reference: `apps/biofield-web/app/globals.css`

---

## 1. The Three Laws (Non-Negotiable)

1. **Bioluminescent, not fluorescent.** Light originates from within organic structure. No external spotlights. No neon glows. No wellness gradients. Soft, internal, warm-to-cool.
2. **Architectural, not decorative.** Every visual element has a structural reason. Sacred geometry is not ornament — it IS the data rendered visible.
3. **Data as sacred form.** Engine outputs are not dashboard metrics. They are consciousness readings. Render them accordingly.

---

## 2. Anti-Patterns (Hard Stops)

| ❌ Never Do This | ✅ Do This Instead |
|---|---|
| Pill status badges ("Ready", "Loading") | Bioluminescent border color states |
| Horizontal scrolling tab bars for engine selection | 4×5 icon grid with sacred geometry sigils |
| Flat Void Black canvas with no depth | Kha Arc gradient + constellation mesh background |
| SaaS card grids with icons and descriptions | Compass-oriented direction cards |
| Generic grid cells for engine output | Engine-specific geometric visualization (SVG) |
| `'Exo 2'` font | `var(--font-display)` (Panchang) |
| `'Space Grotesk'` or `'IBM Plex Mono'` as body | `var(--font-body)` (Satoshi) / `var(--font-mono)` (SF Mono) |
| `#c4873b` (wrong gold) | `var(--c-gold)` = `#C5A017` (Sacred Gold) |
| Inline `style={{ color: '#...' }}` hardcodes | CSS custom properties from `:root` |
| Spinner or loading bar | Sacred geometry SVG builder animation |
| "No data available" placeholder | Empty state with constellation grid + sigil |
| Rounded buttons | Square or low-radius (≤6px) CTA buttons |
| Light mode | Void Black only. No light mode. Ever. |

---

## 3. Color Tokens

```css
/* Complete token set — must be present in globals.css */

/* Consciousness Spectrum (Goethe's Zur Farbenlehre) */
--c-void:        #070B1D;   /* La — Inertia/Source */
--c-violet:      #2D0050;   /* Kha — Spirit/Observer */
--c-indigo:      #0B50FB;   /* Kha→Ba — Flow state */
--c-gold:        #C5A017;   /* Ba — Body/Activation */
--c-emerald:     #10B5A7;   /* Ba↔La — Coherence */
--c-parchment:   #F0EDE3;   /* Text — Satoshi's ground */

/* Surface layers */
--bg:            #070B1D;
--surface:       #0E1428;
--surface-2:     #121932;

/* Text */
--text:          #F0EDE3;
--text-2:        rgba(240,237,227,0.72);
--muted:         rgba(240,237,227,0.44);

/* Lines (Flow Indigo tinted) */
--line-faint:    rgba(11, 80, 251, 0.08);
--line-mid:      rgba(11, 80, 251, 0.18);
--line-strong:   rgba(11, 80, 251, 0.35);

/* Accents */
--signal:        #C5A017;   /* Gold — live/active */
--error:         #C65D3B;   /* Terracotta — warm, not clinical */
--success:       #10B5A7;   /* Coherence Emerald */

/* Gradients — the Kha-Ba-La triad */
--grad-kha: linear-gradient(135deg, #070B1D 0%, #2D0050 50%, #0B50FB 100%);
--grad-ba:  linear-gradient(90deg,  #10B5A7 0%, #C5A017 100%);
--grad-la:  linear-gradient(135deg, #C5A017 0%, #2D0050 50%, #070B1D 100%);

/* Glows */
--glow-indigo:  0 0 16px rgba(11, 80, 251, 0.5);
--glow-gold:    0 0 16px rgba(197, 160, 23, 0.5);
--glow-emerald: 0 0 16px rgba(16, 181, 167, 0.5);
--glow-violet:  0 0 16px rgba(45, 0, 80, 0.7);
```

---

## 4. Typography

| Role | Font | Weight | Size | Use |
|------|------|--------|------|-----|
| Display / Engine names | **Panchang** | 700–800 | 26–68px | Section headers, engine card titles, hero text |
| Body / UI labels | **Satoshi** | 400–600 | 14–18px | All prose, form labels, descriptions |
| Data / Code / Metrics | **SF Mono** | 400–600 | 10–14px | Engine output numbers, timestamps, badges, coordinates |

**Loading:** Add to `app/layout.tsx` via FontShare CDN:
```html
<link rel="stylesheet" href="https://api.fontshare.com/v2/css?f[]=panchang@700,800&f[]=satoshi@400,500,600,700&display=swap">
```

**Rules:**
- Never use `'Exo 2'`, `'Space Grotesk'`, or `'IBM Plex Mono'` — all replaced
- `h1, h2, h3` → `var(--font-display)`
- Body, buttons, labels → `var(--font-body)`
- Numbers, data, metadata → `var(--font-mono)`

---

## 5. Compass Framework (Engine Orientation)

The four compass directions are the UX navigation system, not generic filters:

| Direction | Intention | Gradient | Engines |
|---|---|---|---|
| **STABILIZE** (North) | Ground, root, anchor | Void → Violet (Kha onset) | Panchanga, Vimshottari, Transits |
| **HEAL** (East) | Restore, integrate, clear | Violet → Indigo (Kha full) | Biofield, Biorhythm, Vedic Clock |
| **CREATE** (South) | Activate, express, generate | Emerald → Gold (Ba arc) | Nadabrahman, Raaga, Sacred Geometry, Sigil Forge |
| **MUTATE** (West) | Transform, see, dissolve | Gold → Void (La arc) | Tarot, I-Ching, Human Design, Gene Keys, Enneagram, Numerology, Face Reading |

---

## 6. Engine-to-Visual Mapping (The Contract)

Each engine has a **canonical visual form** derived from its domain. Never render engine output as raw key-value text.

### VEDIC ASTRONOMY

**Panchanga** — *Five-Limb Mandala*
- Form: Concentric ring SVG. Outer ring = 27 nakshatra segments. Inner ring = 30 tithi segments. Center = current hora
- Active element: Sacred Gold fill at 40% opacity + Emerald 2px border
- Center text: Current nakshatra name in Panchang at 26px
- Data below: Yoga / Karana / Vara in 3-col SF Mono grid
- Color domain: Void (60%) · Violet (15%) · Indigo (10%) · Gold (10%) · Emerald (5%)

**Vimshottari** — *Golden Spiral Timeline*
- Form: Horizontal timeline with 9 planetary nodes. Spacing proportional to MahaDasha length
- Current period: Sacred Gold glow `var(--glow-gold)`. Past: Muted. Future: Indigo
- Planet glyphs: SF Mono, positioned at node points
- Color gradient: Gold (Sun/start) → Deep Violet (Rahu/Saturn) → Gold (Venus/end)

**Transits** — *Orbital Wheel*
- Form: 360° zodiac wheel SVG. 12 equal sign segments, labeled with glyphs
- Transit planets: Plotted at longitudinal degree positions on outer ring
- Aspect lines: Trine=Emerald, Square=Terracotta, Sextile=Indigo, Conjunction=Gold
- Sade Sati: Prominent Terracotta banner above wheel if present
- Style: "NASA precision meets Vedic tradition"

### ENERGETIC DESIGN

**Human Design** — *Circuit Bodygraph*
- Form: SVG bodygraph (340×500px). 9 centers as geometric shapes at canonical positions
- Defined centers: `var(--c-gold)` fill. Undefined: transparent + `var(--line-mid)` border
- Active channels: 2px Sacred Gold lines. Gate numbers: SF Mono 9px at midpoint
- Below: Type / Profile / Strategy / Authority / NOT-Self Theme in Satoshi + Panchang header

**Gene Keys** — *Shadow-Gift-Siddhi Spectrum*
- Form: Horizontal spectrum bar per activated key. 3 zones: Shadow → Gift → Siddhi
- Colors: Shadow=Terracotta (La arc) · Gift=Emerald (Ba arc) · Siddhi=Gold (Ba arc peak)
- Key number: Panchang 700 at 42px left-aligned
- Codon / programming partner: SF Mono 10px Muted below bar

**Biofield** — *(handled by biofield-web; noesis-web shows summary data only)*
- Show: Coherence score as radial gauge. Chakra states as 7-point column with color dots
- Colors: Coherent=Emerald · Incoherent=Violet · Heart=Gold

### BIOLOGICAL RHYTHM

**Biorhythm** — *Triple Sine Wave*
- Form: SVG chart 600×160px. 3 overlaid sine waves, ±15 days from today
- Physical: `var(--c-gold)` · Emotional: `var(--c-violet)` · Intellectual: `var(--c-indigo)`
- Today: Coherence Emerald vertical dashed line
- Values: SF Mono labels at right edge. Phase name + % in 3-col summary below
- Critical days (wave intersections): Terracotta dot marker

**Vedic Clock** — *TCM Organ-Hour Ring*
- Form: SVG 24-hour organ clock (360px). 12 × 2-hour arc segments for organ systems
- Current segment: Sacred Gold fill 60% + Emerald `var(--glow-emerald)`
- Center: Current organ name in Panchang (English + Sanskrit below in SF Mono 10px)
- Clock hand: 1px Sacred Gold line from center, like a real clock hand

### SOMATIC PERCEPTION

**Nadabrahman** — *Cymatic Frequency Spheres*
- Form: Concentric circles radiating from center. Frequency layers as ring thickness
- Sub-bass: Deep Violet · Mid-tone: Indigo · Overtones: Gold · Silence: Void
- Center: OM symbol or recommended raga name in Panchang
- Data below: Frequency, recommended time, dosha affinity in SF Mono grid

**Raaga** — *Swara Grid with Avaroha* *(already implemented)*
- See current `Raaga.tsx` — this is the reference implementation for V2 audio controls

**Face Reading** — *Physiognomic Zone Map*
- Form: Simplified face outline SVG with 7 labeled zones (Forehead, Brows, Eyes, Nose, Cheeks, Mouth, Chin)
- Active zones: Sacred Gold outline + subtle gold fill
- Analysis text: Satoshi 400, max-width 320px, right column
- Style: Medical illustration meets sacred mapping

### DIVINATION

**Tarot** — *Bioluminescent Card Face*
- Form: 200×340px card frame. Kha Arc gradient border (animated 4s pulse)
- Card name: Panchang 700 at 42px, Sacred Gold
- Arcana badge: SF Mono 10px "Major Arcana" / "Minor · Cups" etc.
- Reversed: 180° rotation + Terracotta border
- Three-card spread: Render all three side-by-side when present in output

**I-Ching** — *Binary Hexagram SVG*
- Form: 6-line hexagram. Yang=solid 4px `var(--c-gold)` bar. Yin=4px gold bar with 8px center gap
- Moving lines: `var(--c-emerald)` color
- Trigram glyphs (☰☱☲☳☴☵☶☷): Panchang 40px above and below hexagram
- Hexagram number: SF Mono 48px Muted behind figure
- Changing hexagram: Second hexagram to the right with La Arc arrow between

### PERSONALITY ARCHITECTURE

**Enneagram** — *Nine-Point Geometric Star*
- Form: SVG circle 300px. 9 points around circumference
- Inner triangle (3-6-9): Flow Indigo lines. Hexad (1-4-2-8-5-7): Coherence Emerald lines
- Active type: Sacred Gold glow, enlarged 12px circle
- Center: Type number in Panchang. Type name + wing in Satoshi below figure
- Passion / Virtue / Holy Idea: SF Mono grid below

**Numerology** — *Vibrational Frequency Spiral*
- Form: Circular number wheel SVG. 9 positions around circumference, Life Path at center
- Each number: Color-coded per spectrum (1=Gold, 2=Indigo, 3=Violet, 4=Emerald, 5=Indigo, 6=Gold, 7=Indigo, 8=Violet, 9=Gold)
- Active numbers (from output): Sacred Gold fill + glow. Others: Muted border only
- Center: Life Path number in Panchang 700 at 48px + descriptor in Satoshi

### CREATIVE ENCODING

**Sacred Geometry** — *Sri Yantra / Sacred Form SVG*
- Form: Predefined SVG per `form_name` from engine output (Sri Yantra, Flower of Life, Metatron's Cube, Fibonacci spiral)
- Colors: Sacred Gold wireframe lines. Void Black background. Emerald bioluminescent fill at center
- Size: 300×300px centered
- No external SVG library — use inline SVG paths

**Sigil Forge** — *Compressed Intent Symbol*
- Form: 300×300px SVG canvas. Overlapping intention letters in Panchang at 30% opacity, rotated/stacked
- Color: Sacred Gold wireframe on Void Black
- If stroke data present in output: trace the strokes instead
- Style: "Austin Osman Spare meets cyberpunk sticker"

---

## 7. Motion Principles

| Principle | Implementation |
|---|---|
| Growth, not transition | Elements grow in with `transform: scale(0.9) → 1` + `opacity 0 → 1`, not slide |
| Breath-synchronized | Primary loops: 4s cycle (inhale 1.6s · hold 2.8s · exhale 1.6s) |
| Sacred geometry builds | SVG `strokeDashoffset` animation — lines draw themselves |
| La Arc on completion | Sacred Gold pulse ring on submit button after results appear |
| Staggered reveals | Engine grid cells: 30ms stagger between each cell's Emerald dot appearing |

**Performance rule:** All animations must be CSS keyframes — no JS animation libraries. Respect `prefers-reduced-motion`.

---

## 8. State-to-Color Mapping

| State | Visual Treatment |
|---|---|
| **Idle** | `var(--line-faint)` border, `var(--muted)` text |
| **Loading** | `var(--glow-indigo)` animated border pulse |
| **Active / Ready** | 1px Ba Arc gradient top border + `var(--line-strong)` border |
| **Error** | `var(--error)` (#C65D3B Terracotta) border + glow |
| **Admin** | `var(--c-violet)` (#2D0050) border + "ADM" badge in Terracotta |
| **Completion** | `var(--glow-gold)` Sacred Gold ring pulse (1s, then fades) |

---

## 9. Layout System

- **Max content width:** 1100px (engines page), 680px (witness/prose sections)
- **Background:** Always `var(--bg)` = Void Black + Kha Arc gradient overlay + constellation mesh
- **No light mode.** Dark only.
- **Breakpoints:**
  - Mobile: `<480px` — 1-col form, 3-col engine grid, no compass (stacked)
  - Tablet: `480px–768px` — 2-col form, 3-col engine grid, 2×2 compact compass
  - Desktop: `>768px` — auto-fit form, 5-col engine grid, 2×2 full compass

---

## 10. Component File Checklist

Before marking any component done, verify:

- [ ] No `'Exo 2'`, `'Space Grotesk'`, `'IBM Plex Mono'` in component
- [ ] No hardcoded hex colors (`#c4873b`, `#070B1D`, etc.) — use CSS vars
- [ ] No status badge pills
- [ ] Engine output rendered as geometry/visualization, not raw key-value text
- [ ] Mobile viewport tested at 375px
- [ ] `prefers-reduced-motion` respected for any animation
- [ ] TypeScript `--noEmit` passes
