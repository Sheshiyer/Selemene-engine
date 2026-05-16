# Depth-Reading Asset Pipeline

Each of the **15 sections** in the depth-reading view can have:
- A **reference image** (PNG, the ChatGPT-generated artwork after your cleanup)
- A **3D model** (GLB, the Meshy AI output from that image)

## Folder convention

```
public/depth-reading/
├── images/
│   ├── cover.png
│   ├── witness-layer.png
│   ├── compendium.png
│   ├── part-1.png    ... part-11.png
│   ├── closing.png
│   └── quine.png
└── meshes/
    ├── cover.glb
    ├── witness-layer.glb
    ├── compendium.glb
    ├── part-1.glb    ... part-11.glb
    ├── closing.glb
    └── quine.glb
```

Filename = `{section.id}.{ext}`. Section ids are defined in `src/depth-reading/data/sections.ts`.

## How the depth-reading consumes these

- When a GLB exists at `/depth-reading/meshes/{section-id}.glb`, the DepthScene loads it via Three.js GLTFLoader and replaces that section's flat colored plane with the loaded 3D mesh.
- When no GLB exists, the section falls back to the current flat colored plane (visual placeholder).
- Loader scaffolding is added once the **first GLB** lands here — until then this folder just reserves the path convention.

## Per-section mesh transform overrides

Once a GLB is dropped in, you'll likely want to tune its `scale`, `rotation`, `position` per section because Meshy outputs at arbitrary world scale. Those overrides go in `src/depth-reading/data/sections.ts` on each section's `meshTransform` field (will be added when the loader is scaffolded).

---

## The 15 sections — archetype starting points

You can refine each per-section as you generate. These are the visual archetypes I'd recommend feeding into ChatGPT:

| id | numeral | Archetype |
|---|---|---|
| `cover` | ∴ | Bioluminescent sigil — three interlocking circles inside a hexagonal frame |
| `witness-layer` | 0 | A single open eye / lotus aperture / threshold gateway |
| `compendium` | · | A folded star chart / cartographic disk / engraved bronze tablet |
| `part-1` | I | Spiral of converging filaments around a central node (Convergence Map) |
| `part-2` | II | A Sri Yantra in carved bronze relief (Vedic Foundation) |
| `part-3` | III | A nested polyhedral cage / Metatron's cube fragment (Karmic Architecture) |
| `part-4` | IV | A compass with four cardinals + central pillar (Career & Dharma) |
| `part-5` | V | A coin or mandala-disc with concentric rings (Wealth & Money) |
| `part-6` | VI | A vesica piscis with two interlocking forms (Love & Marriage) |
| `part-7` | VII | A chakra column / spinal axis with light points (Health & Energy) |
| `part-8` | VIII | A branching tree / root system in 3D (Family & Lineage) |
| `part-9` | IX | A spiral helix / DNA-like double-helix (Master Timeline) |
| `part-10` | X | An open hand / mudra / receiving gesture (Practices & Anti-Dependency) |
| `part-11` | XI | A crystallizing geode / cracked-open form (Final Synthesis) |
| `closing` | → | A dissolving sigil / unwinding spiral |
| `quine` | ∞ | A perfect torus / Ouroboros ring |

---

## ChatGPT prompt template (consistent visual style across all 15)

Paste this template into ChatGPT-4o image (or DALL-E), filling the `{ARCHETYPE}` line per section:

```
A single bioluminescent sacred-geometry object suspended in deep void-black space.

OBJECT: {ARCHETYPE}

STYLE:
- Bioluminescent, not fluorescent — light originates from within the object's organic structure, not projected onto it
- Sacred geometry as load-bearing structure, never wallpaper
- Sacred Gold (#C5A017) wireframe lines visible
- Coherence Emerald (#10B5A7) core glow at the center
- Witness Violet (#2D0050) ambient atmosphere at the edges
- Void Black (#070B1D) background — pre-chromatic Ur-ground
- Bronze, brass, mineral materials — not plastic, not chrome
- Anatomical precision at visionary scale — PubMed precision meets Alex Grey

FRAMING:
- Single centered object, full silhouette visible
- Plain dark background, no decorative elements behind
- Clean edges (will be used as input for image-to-3D, so silhouette must be unambiguous)
- Front-facing or 3/4 angle — no extreme perspective
- 1024×1024 square format

AVOID:
- Photo-realistic flesh / skin / faces
- Lifestyle / aspirational imagery
- Stock-photo warmth
- Cosmic / ethereal cliché (third eye, lotuses-as-cliche, mandalas-as-wallpaper)
- Wellness gradients (pink→purple)
- Neon glow, fluorescent edges
- Chrome / metallic-paint highlights
```

After ChatGPT generates the image:
1. Clean it up in Photoshop / Krita / Affinity (remove background artifacts, ensure crisp silhouette, optionally add subtle edge contrast)
2. Save as `images/{section-id}.png` here
3. Upload the same image to Meshy AI → Image to 3D → "PBR textured", polycount 5-10k tris, texture 2048px → export GLB
4. Save the GLB as `meshes/{section-id}.glb` here

The DepthScene loader (added once the first GLB lands) will pick it up automatically.

---

## Meshy AI generation parameters

When running each cleaned image through Meshy:

- **Mode:** Image to 3D
- **Topology:** Quad-dominant (for clean edges + better deformation if we later add motion)
- **Polycount:** 5,000–10,000 triangles (web performance budget — 15 models × 10k = 150k tris, well within Three.js single-scene budget)
- **Texture:** PBR (albedo + normal + roughness + metallic), 2048×2048
- **Output:** GLB (single-file, easy to load in Three.js)
- **Symmetry hint:** Yes for symmetric archetypes (sigil, compass, vesica, torus), No for asymmetric (chart, hand, tree)

Result: 15 GLB files, each 500KB–2MB, total ~15–30MB. Lazy-loaded per section so the initial scene is light.
