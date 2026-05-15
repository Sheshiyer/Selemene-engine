"use client";

// ─── IntegratedReadingView — client wrapper for animated rendering ─────
// Composes: ConstellationGrid backdrop, CoverScene hero (W1), then per Part:
//   - WitnessPulse (breathing-ring opener, cardinal direction per Part)
//   - YantraPlate (Part-signature mandala — α triad, β vesica, γ dasha, δ compass)
//   - DashaWaveform between Part III header and verses
//   - VerseFlow (illuminated prose body)
//   - LaArcFade between Parts
//
// Per design MD § 4 (page composition), § 5.3 (WitnessPulse), § 5.4 (YantraPlate),
// § 5.8 (DashaWaveform).

import { ConstellationGrid } from "@/components/integrated/ConstellationGrid";
import { CoverScene } from "@/components/integrated/cover/CoverScene";
import { VerseFlow } from "@/components/integrated/VerseFlow";
import { LaArcFade } from "@/components/integrated/LaArcFade";
import {
  WitnessPulse,
  type WitnessDirection,
} from "@/components/integrated/yantras/WitnessPulse";
import {
  YantraPlate,
  type YantraKind,
  type YantraData,
} from "@/components/integrated/yantras/YantraPlate";
import {
  DashaWaveform,
  type DashaSegment,
} from "@/components/integrated/yantras/DashaWaveform";
import type { IntegratedReading, PassMetric } from "@/lib/integrated/loader";
import type { Block } from "@/lib/integrated/parseBlocks";

type PassWithBlocks = PassMetric & { markdown: string; blocks: Block[] };

interface ViewProps {
  reading: Omit<IntegratedReading, "passes"> & { passes: PassWithBlocks[] };
}

function toRoman(n: number): string {
  const map: Array<[number, string]> = [
    [1000, "M"], [900, "CM"], [500, "D"], [400, "CD"], [100, "C"],
    [90, "XC"], [50, "L"], [40, "XL"], [10, "X"], [9, "IX"],
    [5, "V"], [4, "IV"], [1, "I"],
  ];
  let s = "";
  let r = n;
  for (const [v, sym] of map) {
    while (r >= v) { s += sym; r -= v; }
  }
  return s;
}

// ─── Per-Part yantra plan ───────────────────────────────────────────────
// Pass α / opening    → triad-mandala (uses witness-agents topology SVG)
// Pass β / resonance  → vesica-trio
// Pass γ / phase-lock → dasha-spiral
// Pass δ / anti-dep   → compass-trine
const PART_YANTRAS: YantraKind[] = [
  "triad-mandala",
  "vesica-trio",
  "dasha-spiral",
  "compass-trine",
];

const PART_DIRECTIONS: WitnessDirection[] = [
  "STABILIZE",
  "HEAL",
  "CREATE",
  "MUTATE",
];

// ─── W×H×Mohan mock dasha (current: Rahu MD → Jupiter pivot 2026-09-14) ──
const VIMSHOTTARI_YEARS: Record<string, number> = {
  sun: 6, moon: 10, mars: 7, rahu: 18, jupiter: 16,
  saturn: 19, mercury: 17, ketu: 7, venus: 20,
};
const VIMSHOTTARI_ORDER = [
  "ketu", "venus", "sun", "moon", "mars",
  "rahu", "jupiter", "saturn", "mercury",
] as const;

function isoYearsBefore(iso: string, years: number): string {
  const d = new Date(iso);
  d.setUTCFullYear(d.getUTCFullYear() - years);
  return d.toISOString().slice(0, 10);
}
function isoYearsAfter(iso: string, years: number): string {
  const d = new Date(iso);
  d.setUTCFullYear(d.getUTCFullYear() + years);
  return d.toISOString().slice(0, 10);
}

function buildMockDashaSegments(): {
  periods: DashaSegment[];
  pivots: Array<{ iso: string; label: string }>;
} {
  // Anchor: Rahu ends 2026-09-14. Today is 2026-05-15 → Rahu is current.
  const RAHU_END = "2026-09-14";
  const todayIso = "2026-05-15";

  const rahuIdx = VIMSHOTTARI_ORDER.indexOf("rahu");
  const chain: Array<{ lord: string; start: string; end: string }> = [];

  // Backfill four periods before Rahu.
  let cursorEnd = isoYearsBefore(RAHU_END, VIMSHOTTARI_YEARS.rahu);
  for (let k = 1; k <= 4; k++) {
    const lordIdx =
      (rahuIdx - k + VIMSHOTTARI_ORDER.length) % VIMSHOTTARI_ORDER.length;
    const lord = VIMSHOTTARI_ORDER[lordIdx];
    const start = isoYearsBefore(cursorEnd, VIMSHOTTARI_YEARS[lord]);
    chain.unshift({ lord, start, end: cursorEnd });
    cursorEnd = start;
  }
  // Rahu itself.
  chain.push({
    lord: "rahu",
    start: isoYearsBefore(RAHU_END, VIMSHOTTARI_YEARS.rahu),
    end: RAHU_END,
  });
  // Forward fill four after Rahu.
  let cursorStart = RAHU_END;
  for (let k = 1; k <= 4; k++) {
    const lordIdx = (rahuIdx + k) % VIMSHOTTARI_ORDER.length;
    const lord = VIMSHOTTARI_ORDER[lordIdx];
    const end = isoYearsAfter(cursorStart, VIMSHOTTARI_YEARS[lord]);
    chain.push({ lord, start: cursorStart, end });
    cursorStart = end;
  }

  const today = new Date(todayIso).getTime();
  const periods: DashaSegment[] = chain.map((c) => {
    const s = new Date(c.start).getTime();
    const e = new Date(c.end).getTime();
    const state: DashaSegment["state"] =
      e < today ? "past" : s > today ? "future" : "current";
    return {
      lord: c.lord,
      start_iso: c.start,
      end_iso: c.end,
      duration_years: VIMSHOTTARI_YEARS[c.lord],
      state,
    };
  });

  return {
    periods,
    pivots: [{ iso: RAHU_END, label: "RAHU → JUPITER · 2026-09-14" }],
  };
}

export function IntegratedReadingView({ reading }: ViewProps) {
  const coverTitle =
    reading.subjects.length >= 2 ? "COMPOSITE FIELD" : "INTEGRATED READING";
  const birthMeta = `${reading.mode.toUpperCase().replace(/-/g, " · ")}  ·  ${reading.registerBand.toUpperCase().replace("_", "-")}  ·  ${reading.totalWords.toLocaleString()} WORDS`;

  const { periods: mockDashaPeriods, pivots: mockPivots } = buildMockDashaSegments();

  return (
    <>
      <ConstellationGrid />

      <CoverScene
        title={coverTitle}
        birthMeta={birthMeta}
        subjects={reading.subjects}
        topologySvg={reading.topologySvg}
      />

      <article
        style={{
          position: "relative",
          width: "100%",
          background: "linear-gradient(180deg, rgba(7,11,29,0.55) 0%, rgba(7,11,29,0.85) 100%)",
          backdropFilter: "blur(2px)",
          zIndex: 2,
        }}
      >
        <div
          style={{
            width: "clamp(18rem, 72vw, 80rem)",
            margin: "0 auto",
            padding: "clamp(2rem, 4vw, 5rem) clamp(1rem, 2.4vw, 2.5rem) clamp(3rem, 6vw, 6rem)",
            fontSize: "clamp(1rem, 0.85rem + 0.45vw, 1.22rem)",
            lineHeight: 1.65,
            color: "var(--text)",
          }}
        >
          {reading.passes.map((pass, i) => {
            const isLast = i === reading.passes.length - 1;
            const yantraKind = PART_YANTRAS[i] ?? "triad-mandala";
            const direction = PART_DIRECTIONS[i] ?? "STABILIZE";

            const yantraData: YantraData = {
              subjects: reading.subjects,
              topologySvg:
                yantraKind === "triad-mandala" ? reading.topologySvg : undefined,
              dashaPeriods:
                yantraKind === "dasha-spiral"
                  ? mockDashaPeriods.map((p) => ({
                      lord: p.lord,
                      start_iso: p.start_iso,
                      end_iso: p.end_iso,
                      current: p.state === "current",
                    }))
                  : undefined,
              cardinals:
                yantraKind === "compass-trine"
                  ? {
                      stabilize: "Ground · root · anchor",
                      heal: "Restore · integrate",
                      create: "Activate · express",
                      mutate: "Transform · see",
                    }
                  : undefined,
            };

            return (
              <div key={pass.id} style={{ marginBottom: "clamp(2rem, 6vw, 6rem)" }}>
                <header
                  id={`part-${i + 1}`}
                  style={{ margin: "clamp(2rem, 4vw, 4rem) 0 clamp(1.5rem, 3vw, 3rem)" }}
                >
                  <div
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "clamp(0.72rem, 0.65rem + 0.15vw, 0.85rem)",
                      letterSpacing: "0.45em",
                      textTransform: "uppercase",
                      color: "var(--c-gold)",
                      marginBottom: "0.85rem",
                    }}
                  >
                    Part {toRoman(i + 1)}
                  </div>
                  <h1
                    style={{
                      fontFamily: "var(--font-display)",
                      fontWeight: 800,
                      fontSize: "clamp(2rem, 1.4rem + 2.4vw, 4.5rem)",
                      lineHeight: 1.02,
                      letterSpacing: "-0.022em",
                      color: "var(--c-parchment)",
                      margin: 0,
                    }}
                  >
                    {pass.title}
                  </h1>
                  <div
                    style={{
                      marginTop: "0.85rem",
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.85rem",
                      color: "var(--c-emerald)",
                      letterSpacing: "0.12em",
                    }}
                  >
                    {pass.words.toLocaleString()} words · {pass.xrefs} cross-references
                  </div>
                </header>

                {/* W2: WitnessPulse opener for this Part */}
                <WitnessPulse direction={direction} title={pass.title} />

                {/* W2: Part-signature yantra mandala */}
                <YantraPlate kind={yantraKind} data={yantraData} />

                {/* W2: DashaWaveform between Part III header and verses */}
                {i === 2 ? (
                  <DashaWaveform
                    periods={mockDashaPeriods}
                    pivots={mockPivots}
                  />
                ) : null}

                <VerseFlow blocks={pass.blocks} />

                {!isLast && <LaArcFade />}
              </div>
            );
          })}

          <LaArcFade />

          <footer
            style={{
              marginTop: "clamp(3rem, 6vw, 6rem)",
              paddingTop: "clamp(1.5rem, 3vw, 3rem)",
              borderTop: "1px solid var(--line-faint)",
              fontFamily: "var(--font-mono)",
              fontSize: "0.85rem",
              color: "var(--muted)",
              textAlign: "center" as const,
              letterSpacing: "0.12em",
            }}
          >
            <div
              style={{
                fontFamily: "var(--font-display)",
                fontStyle: "italic",
                fontWeight: 500,
                fontSize: "1.05rem",
                color: "var(--c-gold)",
                marginBottom: "0.75rem",
              }}
            >
              The Anatomist Who Sees Fractals
            </div>
            <div>TRYAMBAKAM NOESIS · 1331.TRYAMBAKAM.SPACE</div>
            <div style={{ marginTop: "1rem", maxWidth: "48ch", margin: "1rem auto 0", fontStyle: "italic", color: "var(--muted)" }}>
              This document is documentation of an instrument. The instrument is what
              you already are. The Quine principle: the system succeeds when you no
              longer need it.
            </div>
          </footer>
        </div>
      </article>
    </>
  );
}
