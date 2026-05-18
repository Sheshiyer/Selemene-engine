// ─── Depth-Reading section data model ──────────────────────────────────
// 15 sections that form the full chapter arc of a Vedic reading,
// adapted for the codrops depth-gallery's plane-per-section paradigm.
//
// Each section is a plane in 3D space with:
//   - Position (x ∈ [-1, 1] offset; y stays 0; depth z derived by index)
//   - Goethe-spectrum colors (background, accent, blob1, blob2)
//   - Short label (Roman numeral + title + 1-2 line summary)
//   - Optional textureSrc (sigil / yantra svg → png export later)
//
// On click: opens a modal with the full prose for that section
// (witness-agents output, loaded server-side, hydrated as 3-4 sentence
// scroll-highlighted text per design v2 directive).

export type SectionKind =
  | "cover"
  | "witness-layer"
  | "compendium"
  | "part" // I–XI
  | "closing"
  | "quine";

export interface SectionData {
  /** Stable id used to look up prose content (witness-agents output) */
  id: string;
  /** What kind of plane this is */
  kind: SectionKind;
  /** Display number — Roman for parts, special tokens for cover/closing */
  numeral: string;
  /** Section title (3-6 words) */
  title: string;
  /** 1-2 sentence summary shown on the plane label (not the full prose) */
  summary: string;
  /** Position of the plane in 3D space */
  position: { x: number; y: number };
  /** Depth multiplier — controls how far back in Z the plane sits */
  depth: number;
  /** Goethe-palette derived colors */
  backgroundColor: string;
  accentColor: string;
  blob1Color: string;
  blob2Color: string;
  /** Cardinal direction tone — drives accent tint */
  direction?: "STABILIZE" | "HEAL" | "CREATE" | "MUTATE" | "WITNESS" | "QUINE";
  /** Optional texture for the plane (sigil / yantra image) */
  textureSrc?: string;
  /** Specs shown in the RIGHT-column data card (codrops Label.js pattern) */
  specs?: Array<{ label: string; value: string }>;
  /** Path under /depth-reading/meshes/ for the Meshy-generated GLB. When
   *  present, the DepthScene loader fetches the GLB and renders it in
   *  place of the flat colored plane. Falls back to the plane otherwise. */
  meshPath?: string;
  /** Per-section transform overrides applied to the loaded GLB so each
   *  section's mesh sits correctly in the gallery. Meshy outputs are at
   *  arbitrary world scale + facing; tune these per-section after first
   *  load. */
  meshTransform?: {
    scale?: number;
    rotation?: { x?: number; y?: number; z?: number };
    position?: { x?: number; y?: number; z?: number };
  };
}

// Goethe Consciousness Spectrum (from apps/noesis-web/app/globals.css)
const VOID = "#070B1D";
const VIOLET = "#2D0050";
const INDIGO = "#0B50FB";
const GOLD = "#C5A017";
const EMERALD = "#10B5A7";
const PARCHMENT = "#F0EDE3";

// Cycle yields one of 4 cardinal tones per Part i (1-indexed in [1..11])
const CARDINAL_ORDER: SectionData["direction"][] = [
  "STABILIZE",
  "HEAL",
  "CREATE",
  "MUTATE",
];
const cardinalForPart = (n: number): SectionData["direction"] =>
  CARDINAL_ORDER[(n - 1) % CARDINAL_ORDER.length];

const accentForCardinal: Record<NonNullable<SectionData["direction"]>, string> = {
  STABILIZE: VIOLET,
  HEAL: INDIGO,
  CREATE: EMERALD,
  MUTATE: GOLD,
  WITNESS: VOID,
  QUINE: VOID,
};

const partTitles: Array<{ title: string; summary: string }> = [
  {
    title: "The Convergence Map",
    summary: "Where every engine the reading uses meets — the synthesis floor.",
  },
  {
    title: "The Vedic Foundation",
    summary: "Lagna, Rashi, Nakshatra — the placements that anchor the chart.",
  },
  {
    title: "The Karmic Architecture",
    summary: "What carried over from before — the structural inheritance.",
  },
  {
    title: "Career & Dharma",
    summary: "The 10th-bhava signal — work as direction-of-being.",
  },
  {
    title: "Wealth & Money",
    summary: "The 2nd and 11th — what gathers, what flows, what compounds.",
  },
  {
    title: "Love, Marriage, Spouse",
    summary: "The 7th-bhava read — partnership as discipline.",
  },
  {
    title: "Health & Energy Body",
    summary: "Pranic flow, constitutional balance, the body as instrument.",
  },
  {
    title: "Family, Roots, Soul Lineage",
    summary: "The 4th-bhava + Atmakaraka — what you inherited and what you carry.",
  },
  {
    title: "The Master Timeline",
    summary: "Mahadasha + Antardasha sequence — when each chapter opens.",
  },
  {
    title: "Practices & Anti-Dependency",
    summary: "What to do so the reading makes itself unnecessary.",
  },
  {
    title: "Final Synthesis",
    summary: "The whole compressed into a single recognizable shape.",
  },
];

/**
 * Build the 15-section depth-gallery data for a given subject.
 * Positions alternate slightly left/right and depth increases with index so
 * the reader scrolls forward into successive planes.
 */
export function buildSectionsForSubject(
  _subject: string,
  _engineOutputs?: Record<string, unknown>,
): SectionData[] {
  // X positions zig-zag mildly so planes don't stack directly on the same axis
  const xPattern = [0, -0.7, 0.7, -0.5, 0.5, -0.8, 0.8, -0.4, 0.4, -0.6, 0.6, -0.3, 0.3, 0, 0];

  const sections: SectionData[] = [];

  // 0. Cover (front of the gallery — z=0, dead-center)
  sections.push({
    id: "cover",
    kind: "cover",
    numeral: "∴",
    title: "Integrated Reading",
    summary: "Subject as field. Scroll to descend.",
    position: { x: 0, y: 0 },
    depth: 0,
    backgroundColor: VOID,
    accentColor: GOLD,
    blob1Color: VIOLET,
    blob2Color: INDIGO,
    direction: "WITNESS",
    specs: [
      { label: "MODE", value: "SOLO" },
      { label: "REGISTER", value: "L4 · L5" },
      { label: "ARC", value: "15 SECTIONS" },
      { label: "LINEAGE", value: "TRYAMBAKAM" },
    ],
    meshPath: "/depth-reading/meshes-fal/cover.glb",
    meshTransform: { scale: 1.5, rotation: { y: 0 } },
  });

  // 1. Witness Layer (chapter 0 — the threshold question)
  sections.push({
    id: "witness-layer",
    kind: "witness-layer",
    numeral: "0",
    title: "What This Reading Is About",
    summary: "The threshold question. Read this before the chapters open.",
    position: { x: xPattern[1], y: 0 },
    depth: 1,
    backgroundColor: VOID,
    accentColor: EMERALD,
    blob1Color: INDIGO,
    blob2Color: VIOLET,
    direction: "WITNESS",
    specs: [
      { label: "TYPE", value: "OPENER" },
      { label: "FRAME", value: "WITNESS LAYER" },
      { label: "PROMPT", value: "READ FIRST" },
    ],
  });

  // 2. Compendium (subject snapshot — Lagna / AK / Nakshatra / Dasha)
  sections.push({
    id: "compendium",
    kind: "compendium",
    numeral: "·",
    title: "The Native",
    summary: "The chart at a glance — Lagna, Atmakaraka, Nakshatra, current dasha.",
    position: { x: xPattern[2], y: 0 },
    depth: 2,
    backgroundColor: VOID,
    accentColor: GOLD,
    blob1Color: VIOLET,
    blob2Color: EMERALD,
    direction: "WITNESS",
    specs: [
      { label: "TYPE", value: "SNAPSHOT" },
      { label: "AXIS", value: "BIRTH DATA" },
      { label: "ENGINES", value: "16" },
    ],
  });

  // 3-13. 11 reading parts
  const romanNumerals = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI"];
  // Per-part theme/yantra label that goes in the RIGHT specs column
  const partThemes = [
    { theme: "BEDROCK", yantra: "TRIAD MANDALA" },
    { theme: "FOUNDATION", yantra: "VESICA TRIO" },
    { theme: "INHERITANCE", yantra: "DASHA SPIRAL" },
    { theme: "DHARMA", yantra: "COMPASS TRINE" },
    { theme: "FLOW", yantra: "TRIAD MANDALA" },
    { theme: "PARTNERSHIP", yantra: "VESICA TRIO" },
    { theme: "EMBODIMENT", yantra: "DASHA SPIRAL" },
    { theme: "LINEAGE", yantra: "COMPASS TRINE" },
    { theme: "TIMELINE", yantra: "TRIAD MANDALA" },
    { theme: "PRACTICE", yantra: "VESICA TRIO" },
    { theme: "SYNTHESIS", yantra: "DASHA SPIRAL" },
  ];
  for (let i = 0; i < 11; i++) {
    const direction = cardinalForPart(i + 1);
    const accent = accentForCardinal[direction!];
    // Parts 1-9 have Era3D+Hunyuan multi-view GLBs ready (volumetric,
    // depth-ratio 0.11-0.88, mean 0.49). Parts 10-11 await image gen.
    const meshPath = i < 9
      ? `/depth-reading/meshes-fal/part-${i + 1}.glb`
      : undefined;
    sections.push({
      id: `part-${i + 1}`,
      kind: "part",
      numeral: romanNumerals[i],
      title: partTitles[i].title,
      summary: partTitles[i].summary,
      position: { x: xPattern[3 + i] ?? 0, y: 0 },
      depth: 3 + i,
      backgroundColor: VOID,
      accentColor: accent,
      blob1Color: GOLD,
      blob2Color: EMERALD,
      direction,
      specs: [
        { label: "CARDINAL", value: direction ?? "WITNESS" },
        { label: "THEME", value: partThemes[i].theme },
        { label: "YANTRA", value: partThemes[i].yantra },
        { label: "DEPTH", value: `${i + 1} / 11` },
      ],
      meshPath,
      meshTransform: meshPath ? { scale: 1.4 } : undefined,
    });
  }

  // 14. Closing (final synthesis-of-synthesis)
  sections.push({
    id: "closing",
    kind: "closing",
    numeral: "→",
    title: "Beyond the Chart",
    summary: "What the reading became as it crystallized.",
    position: { x: xPattern[14], y: 0 },
    depth: 14,
    backgroundColor: VOID,
    accentColor: GOLD,
    blob1Color: VIOLET,
    blob2Color: GOLD,
    direction: "MUTATE",
    specs: [
      { label: "TYPE", value: "CRYSTALLIZATION" },
      { label: "BEAT", value: "FINAL" },
      { label: "FRAME", value: "ANTI-DEPENDENCY" },
    ],
  });

  // 15. Quine (the system succeeds when you no longer need it)
  sections.push({
    id: "quine",
    kind: "quine",
    numeral: "∞",
    title: "The Quine",
    summary: "The system succeeds when you no longer need it.",
    position: { x: 0, y: 0 },
    depth: 15,
    backgroundColor: VOID,
    accentColor: PARCHMENT,
    blob1Color: VIOLET,
    blob2Color: VOID,
    direction: "QUINE",
    specs: [
      { label: "PRINCIPLE", value: "ANTI-DEPENDENCY" },
      { label: "TELOS", value: "SUCCESS = UNNECESSITY" },
      { label: "LOOP", value: "RETURN TO START" },
    ],
  });

  return sections;
}

export const PALETTE = { VOID, VIOLET, INDIGO, GOLD, EMERALD, PARCHMENT };
