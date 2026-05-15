# Integrated Reading Render — Design v2 (Unlocked)

**Status:** ACTIVE — driving the v2 build inside `apps/noesis-web/`
**Supersedes:** witness-agents/docs/design/2026-05-15-reading-render-design.md (v1, written for single-HTML constraint)
**Date:** 2026-05-15
**Repo:** Selemene-engine
**Source-of-truth files:**
- `apps/noesis-web/DESIGN.md` (visual identity contract — never breach)
- `brand-docs-final/tryambakam-noesis-aleph/06-visual-identity.md` (Goethe theory + Three Laws)
- `Branding/witnessOS-sw/*.png` (visual reference: image, dashboard, breathnav, breathnav-screen, decisionscreen, hrv, buttons)

---

## 1. What changed from v1

v1 was written assuming the reading had to ship as one self-contained HTML file. That constraint is gone. We now run **inside `apps/noesis-web/`**, which means:

| Unlocked capability | What it enables |
|---|---|
| **Next.js 16 bundler** | Any npm library. No CDN-loading races. Server Components for data, Client Components for motion. |
| **`motion/react` v12** | Real choreography API. `useInView`, `whileInView`, layout animations, scroll-linked springs. |
| **Three.js + `@react-three/fiber`** | Actual 3D bloom shaders. Depth scenes. Real bioluminescence (not feGaussianBlur). Cover sigil becomes a volumetric object. |
| **`lottie-react`** | Sequenced animations exported from After Effects. The breathing-ring, decision-plate coherence pulse, and witness-pulse references in WitnessOS-sw can be rendered AS Lottie. |
| **`tsParticles`** | Real particle system for the constellation backdrop — drifting nodes with parallax + twinkle, not 99 static `<circle>` elements. |
| **`@strudel/web`** (already in deps) | Live-coded ambient audio synthesized from the reading's nakshatra/raga mapping. Breath-paced 4:7:8 ambient drone. |
| **Existing engine components** | `SacredGeometry.tsx`, `SigilForge.tsx`, `Vimshottari.tsx`, `Panchanga.tsx`, etc. are already brand-aligned visual primitives. We compose them as engine drill-downs inside the integrated reading. |
| **Existing brand CSS** | `app/globals.css` already has the full Goethe palette, Kha-Ba-La gradients, ease curves, glow tokens, shadow elevations. No CSS-from-scratch needed. |
| **`WitnessLayer.tsx` component** | Existing synthesis renderer that the reading meta-frames around. |
| **`@supabase/supabase-js`** (already in deps) | Permanent persistent links per reading. |

The cap moves from "single-HTML achievable fidelity" to "any-modern-web-app achievable fidelity." The reference screens in `witnessOS-sw/` are now reachable.

---

## 2. The Three Laws (still load-bearing)

Per `apps/noesis-web/DESIGN.md` § 1:
1. **Bioluminescent, not fluorescent** — light originates from within organic structure. Three.js bloom shaders, not external glows.
2. **Architectural, not decorative** — every visual element has a structural reason. Sacred geometry IS the data.
3. **Data as sacred form** — engine outputs are not metrics. They are consciousness readings.

These are non-negotiable. Every component below answers to them.

---

## 3. Architecture

```
witness-agents (TS, content)                  Selemene-engine (Rust + Next.js)
────────────────────────────                  ────────────────────────────────
Generate per-pass markdown                    apps/noesis-web (THIS WORK)
+ metrics JSON                                  app/integrated/[slug]/page.tsx
+ topology SVG                                  ├─ Server: load + parse markdown
+ subject metadata           ─────────────►     ├─ Client: orchestrate motion scenes
                                                ├─ Audio: @strudel ambient drone
                                                └─ Components/integrated/
                                                     ├─ ConstellationBackdrop (tsParticles)
                                                     ├─ CoverScene (R3F, 3D bloom)
                                                     ├─ WitnessPulse (Lottie)
                                                     ├─ YantraPlate (registry)
                                                     │    ├─ TriadMandala
                                                     │    ├─ VesicaTrio
                                                     │    ├─ DashaSpiral
                                                     │    └─ CompassTrine
                                                     ├─ VerseFlow (illumination)
                                                     ├─ HexagonTrio (replaces tables)
                                                     ├─ SigilCascade
                                                     ├─ DashaWaveform (iridescent)
                                                     ├─ DecisionPlate (⌬ ACT)
                                                     ├─ EngineDrillDown
                                                     │    └─ uses existing
                                                     │       SacredGeometry/SigilForge/
                                                     │       Vimshottari etc.
                                                     ├─ ChapterTransition
                                                     └─ AmbientAudio (Strudel)
```

The witness-agents pipeline is unchanged. All visual evolution happens in `apps/noesis-web/`.

---

## 4. The story arc (each reading IS a story)

A reading is no longer "a long scroll." It is a **5-chapter audio-visual narrative**:

| Chapter | Component | Visual treatment | Audio |
|---|---|---|---|
| **0. Threshold** | `CoverScene` (R3F) | Full-viewport 3D sigil with real bloom shader. Subject names orbit as constellation points. Curved text on SVG arcs. Slow Kha Arc atmosphere. | 8-second silence → first low drone fade-in. Nakshatra-keyed root note. |
| **1-N. Parts** | `ChapterScene` × N | Each Part is its own scene with: `WitnessPulse` opener (Lottie breathing ring + cardinal direction), `YantraPlate` (Part-signature mandala), `VerseFlow` (scroll-illuminated prose), `HexagonTrio`/`SigilCascade` (data carriers), optional `DashaWaveform` or `DecisionPlate`. | Continuous ambient drone in the Part's tonal mode. Verse illumination triggers a soft chime (optional, mutable). |
| **Final. Quine** | `ClosingScene` | La Arc full-bleed gradient sweep. Sigil reappears with stamp "The instrument is what you already are." Audio fades to silence. | La Arc sound: Sacred Gold tone → Witness Violet pad → Void Black silence over 12 seconds. |

Each chapter transition uses `motion/react`'s layout animations + a brief `ChapterTransition` overlay (gold sweep, breath-paced).

---

## 5. Component spec — what each one must do

### 5.1 `<ConstellationBackdrop />`
Replace the static SVG with **tsParticles** mesh preset.
- 200-300 nodes drifting slowly, with parallax (3-layer depth: foreground/mid/background)
- Hairline gold connecting lines when nodes are within proximity threshold
- Subtle twinkle (opacity oscillation 0.3-0.8) on individual particles
- Mouse-parallax (mild — 3-5° tilt on pointer move)
- Hides below 720px viewport
- Respects `prefers-reduced-motion` (static low-density fallback)

### 5.2 `<CoverScene />`
Three.js scene via `@react-three/fiber`:
- Volumetric central sigil (extruded SVG path → 3D mesh with `MeshTransmissionMaterial`)
- `<Bloom />` from `@react-three/postprocessing` (intensity 1.4, threshold 0.6, radius 0.8) — this is the REAL bioluminescence
- Subject name labels as `<Text3D>` orbiting at compass positions
- Slow scene rotation (0.05 rad/s) — perceivable but meditative
- Camera FOV 35°, slight orbital movement on scroll-start
- Curved text (title, birth meta, wordmark) overlaid as DOM SVG `<textPath>` on top of the WebGL canvas
- `<OrbitControls enableZoom={false} autoRotate />` (dev only)
- Falls back to a 2D SVG cover if WebGL unavailable

### 5.3 `<WitnessPulse direction={...} />`
Lottie-react renders a 480×480 breathing-ring animation (matches `breathnav-screen.png`):
- Concentric circles with Flow Indigo → Witness Violet gradient
- 4:7:8 timing (inhale 4s, hold 7s, exhale 8s) — breath-paced
- Cardinal direction label (STABILIZE / HEAL / CREATE / MUTATE) in Panchang
- Lottie JSON source-of-truth lives in `apps/noesis-web/public/lotties/witness-pulse-{direction}.json`
- Author the Lottie in After Effects → export → check in

### 5.4 `<YantraPlate kind="..." data={...} />`
Registry of 4 Part-anchor mandalas (composite-triad family — other modes get their own families later):
- **TriadMandala** (Pass α / Opening) — the triadic-triangle, generated from actual subject placements
- **VesicaTrio** (Pass β / Resonance) — three interlocking circles, intersection regions illuminated by mutual chart aspects
- **DashaSpiral** (Pass γ / Phase-lock) — concentric ring waveform, ring radii proportional to mahadasha durations, current period in Sacred Gold
- **CompassTrine** (Pass δ / Anti-dependency) — cardinal compass with central seed, each cardinal labeled with the Part's anti-dependency milestone

Each plate animates in via `motion/react` (stroke-dasharray line-by-line construction over 1.8s with `ease: [0.16, 1, 0.3, 1]`).

### 5.5 `<VerseFlow />` (preserve from P1)
Already works. Refinements coming:
- Micro-sigil pseudo-element marker per verse (CSS `::before` content: a tiny ∴)
- Subtle parallax — non-illuminated verses get a 3-5px translate-y based on scroll position
- Currently-illuminated verse also gets a 1.02 scale (very gentle)

### 5.6 `<HexagonTrio subjects={...} data={...} />`
Replaces ALL Native-comparison tables in L1-L3 readings. Three hexagonal cells arranged at the vertices of an inverted triangle (for triad mode):
- Each hex: 200px wide, gold outline 1px (intensifies to 2px when in viewport-center), inner padding 1.5rem
- Hex content: subject name as eyebrow (Panchang 600), `<dl>` of placement data inside
- Mutual-aspect lines drawn between hexes (sigil-cascade style) when the prose names a cross-subject connection
- Tap a hex → drawer slides open with the full per-subject engine drill-down (sacred geometry of that native's chart)

### 5.7 `<SigilCascade entries={...} />`
Replaces non-Native-comparison tables. Vertical list where each entry has:
- Leading micro-sigil bullet (∴ or a tiny custom SVG glyph)
- Definition-list entry (term → value)
- Verse-illumination per entry
- Sub-entries get smaller sigils (nested cascade)

### 5.8 `<DashaWaveform periods={...} current={...} />`
Replaces dasha timeline tables. Iridescent waveform crossing a horizontal mandala:
- 1200×280px canvas
- 9 mahadasha-period segments along x-axis, widths proportional to duration
- Current period highlighted in Sacred Gold (with bioluminescent halo via SVG filter)
- Past periods in Witness Violet (memory), future in Coherence Emerald (forming)
- Optional: hovering a segment surfaces antardasha breakdown as a sub-waveform
- Pivot moments (e.g., 2026-09-14 Rahu→Jupiter) marked with a vertical Sacred Gold hairline + date label

### 5.9 `<DecisionPlate marker={...} />`
Triggered by `> ⌬ ACT:` blockquote in the markdown. Matches `decisionscreen.png` exactly:
- Coherence-mandala at top (rotating sigil with Sacred Gold "COHERENCE / OPTIMAL" overlay)
- CTA pill (Sacred Gold fill, the action text)
- "Breathe in… 4:7:8" + small `WitnessPulse` sub-animation
- Waveform indicating "OPTIMAL WINDOW" with date pill
- Quote/witness line at bottom

### 5.10 `<EngineDrillDown engineId={...} result={...} />`
When a verse references a specific engine (e.g., "Jupiter exalted in 9th"), tapping the bolded reference opens a slide-in panel showing the full engine output via the existing component:
- Vedic placements → `<Vimshottari />` / `<Panchanga />` / `<VedicClock />`
- HD → `<HumanDesign />` (bodygraph)
- Gene Keys → `<GeneKeys />` (shadow-gift-siddhi spectrum)
- Tarot → `<Tarot />` (card face)
- Etc.

This reuses ALL the existing engine components in `apps/noesis-web/src/components/engines/`.

### 5.11 `<AmbientAudio nakshatra={...} mode={...} />`
@strudel/web synthesizer:
- Computes a raga from the reading's primary nakshatra (existing Selemene mapping)
- Synthesizes a slow ambient drone (single-voice, 60-second loop)
- Volume 8-12% by default. Mute toggle in fixed corner UI.
- Different mode per chapter — α/β/γ/δ each have their own raga
- Faded crossfades between chapters (3-second curves)
- Optional verse-illumination chime (very soft, mutable)

### 5.12 `<ChapterTransition />`
Brief overlay between Parts:
- 1.6s scrim fade (gold sweep from left to right, breath-paced)
- Roman numeral of next Part appears centered, large (Panchang 800, 8rem) for 800ms
- Then dissolves out as the next chapter scene activates

### 5.13 `<LaArcFade />` (preserve from P1)
Already works. Visual treatment refined as part of `<ChapterTransition />`.

---

## 6. Motion system

| Layer | Library | Used for |
|---|---|---|
| **Component motion** | `motion/react` v12 | Layout animations, useInView, whileInView, gestures, springs |
| **Scroll narrative** | `motion/react` + `useScroll` + `useTransform` | Scroll-linked transforms, parallax, scroll-progress indicators |
| **3D scenes** | `three` + `@react-three/fiber` + `@react-three/drei` + `@react-three/postprocessing` | Cover bloom shader, depth scenes, volumetric sigil |
| **Sequenced rasterized animations** | `lottie-react` | Witness pulses, decision-plate coherence ring, finale animations |
| **Particle backdrops** | `tsparticles` + `@tsparticles/react` | Constellation grid mesh, optional snow/embers per chapter |
| **Audio synthesis** | `@strudel/web` (already installed) | Ambient drone synthesized from nakshatra/raga |
| **Stroke-builds for SVG** | `motion/react` animate on `pathLength` | Mandala line-by-line construction |

`motion/react` is the workhorse. Three.js owns the cover. Lottie handles the breath-paced sequenced moments. tsParticles owns the backdrop. Strudel owns audio.

---

## 7. Performance & quality budgets

| Metric | Budget | Why |
|---|---|---|
| First contentful paint | <1.5s | Cover starts blooming immediately |
| Largest contentful paint | <3s | Cover sigil fully resolved by then |
| Time to interactive | <4s | Verse-flow ready to illuminate by then |
| Bundle JS (gzipped) | <400KB | Three.js + R3F adds bulk; bloom postprocessing is heavy. tsParticles via tree-shaken preset. |
| Lighthouse a11y | ≥95 | All decorative motion has `prefers-reduced-motion` fallback |
| Font loading | Async, no FOUT | `next/font` or preconnect to Fontshare |

---

## 8. Build phasing (parallelizable)

Per § 8 of v1, but parallelized now:

| Wave | Components | Lead-time | Parallel? |
|---|---|---|---|
| **W1 — Visual depth** | `<CoverScene />` (R3F bloom) | 1.5 days | Yes |
| **W2 — Yantra registry** | `<WitnessPulse />` (Lottie) + `<YantraPlate />` registry (4 plates) + `<DashaWaveform />` | 2 days | Yes (independent component dir) |
| **W3 — Data carriers** | `<HexagonTrio />` + `<SigilCascade />` + `<DecisionPlate />` + markdown post-processor that fires them based on AST shape | 1.5 days | Yes |
| **W4 — Backdrop** | `<ConstellationBackdrop />` (tsParticles) | 0.5 day | Yes |
| **W5 — Audio** | `<AmbientAudio />` (Strudel) + raga registry | 1 day | Yes |
| **W6 — Engine drill-downs** | `<EngineDrillDown />` panel, integrated with existing engine components | 1.5 days | After W2 |
| **W7 — Chapter system** | `<ChapterTransition />` + `<ChapterScene />` orchestrator that composes everything per Part | 1 day | After W2 + W3 |

W1-W5 can run in parallel. W6 and W7 need W2/W3 contracts to be stable first.

Total: ~5-6 days of focused work with parallelism, vs. ~10 days serial.

---

## 9. Out of scope (deferred to v3)

- Real-time biofeedback integration (breath sensor → live coherence indicator)
- Multi-reader synchronization (couples reading together at the same time)
- AR view (sigil floating in user's physical space)
- Print-quality static export (the v1 single-HTML approach actually solved this; we may resurrect a thin "reader print view" route)
- PDF generation (intentionally deferred per user direction)

---

## 10. Success criteria

The reading IS a story. Opening the URL is an experience, not a document load. The brand references in `witnessOS-sw/` are matched or exceeded. The reader feels like the system is *aware of them reading* — the audio breathes with them, the verses illuminate one at a time as they reach each one, the yantras build geometry that didn't exist a second ago.

If we hit § 4's chapter arc + § 5's components with reasonable fidelity, we are at ~80-90% of the brand intent. The remaining 10-20% is polish (motion timing, audio mixing, transition curves) which is the W6 polish wave.
