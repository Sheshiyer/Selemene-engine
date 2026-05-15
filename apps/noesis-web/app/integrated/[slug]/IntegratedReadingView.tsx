"use client";

// ─── IntegratedReadingView — client wrapper for animated rendering ─────
// Composes: ConstellationGrid backdrop, CoverScene hero (W1), then each
// Part as a full ChapterScene (W7) with ChapterTransition (W7) between
// consecutive Parts. ChapterProgress + ChapterNavigator (W7) mounted
// globally. W5 audio + W6 drilldown also live here as global providers.
//
// Per integrated-reading-design-v2.md § 4 (story arc — each reading is
// a chapter narrative), § 5.3 (WitnessPulse), § 5.4 (YantraPlate),
// § 5.8 (DashaWaveform), § 5.12 (ChapterTransition).

import React, { useState, useMemo, useCallback } from "react";

import { ConstellationBackdrop } from "@/components/integrated/backdrop/ConstellationBackdrop";
import { CoverScene } from "@/components/integrated/cover/CoverScene";
import { VerseFlow } from "@/components/integrated/VerseFlow";
import { LaArcFade } from "@/components/integrated/LaArcFade";
import { EngineDrillDown } from "@/components/integrated/drilldown/EngineDrillDown";
import { WitnessLayerOpener } from "@/components/integrated/drilldown/WitnessLayerOpener";
import {
  DrilldownContext,
  type DrilldownContextValue,
  type DrilldownTarget,
} from "@/components/integrated/drilldown/DrilldownContext";
import {
  AudioStateProvider,
  AmbientAudio,
  CoherenceBreath,
  AudioControlPanel,
  CursorProximityScene,
} from "@/components/integrated/audio";
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
import {
  ChapterScene,
  ChapterTransition,
  ChapterProgress,
  ChapterNavigator,
  type ChapterDirection,
} from "@/components/integrated/chapters";
import {
  SubjectCompendium,
  PartBento,
} from "@/components/integrated/bento";
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

// WitnessDirection and ChapterDirection share the same 4-tuple — the
// compass framework (DESIGN.md § 5) maps each Part to one cardinal.
const PART_DIRECTIONS: ChapterDirection[] = [
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

  // Resolve per-Part meta up front so transitions + navigator + progress
  // can share the same source of truth.
  const partMeta = reading.passes.map((pass, i) => ({
    partNum: i + 1,
    romanNumeral: toRoman(i + 1),
    title: pass.title,
    direction: PART_DIRECTIONS[i] ?? ("STABILIZE" as ChapterDirection),
    yantraKind: PART_YANTRAS[i] ?? "triad-mandala",
  }));

  // ─── W6 drill-down state ─────────────────────────────────────────────
  // The IntegratedReading shape doesn't (yet) carry engine_outputs; when
  // the witness-agents pipeline starts emitting them, this hook reads
  // them off the reading. For now we expose an empty map so term-clicks
  // open the panel in its "no data yet" placeholder state.
  const [activeDrilldown, setActiveDrilldown] =
    useState<DrilldownTarget | null>(null);
  const engineOutputs = useMemo<
    Record<string, Record<string, unknown> | undefined>
  >(() => {
    const r = reading as unknown as {
      engineOutputs?: Record<string, { result?: Record<string, unknown> }>;
    };
    const map: Record<string, Record<string, unknown> | undefined> = {};
    if (r.engineOutputs) {
      for (const [id, eo] of Object.entries(r.engineOutputs)) {
        map[id] = eo?.result;
      }
    }
    return map;
  }, [reading]);
  const openDrilldown = useCallback(
    (target: DrilldownTarget) => setActiveDrilldown(target),
    [],
  );
  const closeDrilldown = useCallback(() => setActiveDrilldown(null), []);
  const drilldownValue: DrilldownContextValue = useMemo(
    () => ({ open: openDrilldown, engineOutputs }),
    [openDrilldown, engineOutputs],
  );

  return (
    <AudioStateProvider>
     <DrilldownContext.Provider value={drilldownValue}>
      <ConstellationBackdrop />

      {/* W5 — Audio, atmosphere, cursor proximity. Per integrated-reading-design-v2.md § 5.11. */}
      {/* Placeholder nakshatra="rohini" — W6/future will wire to subject's primary nakshatra. */}
      <AmbientAudio nakshatra="rohini" />
      <CoherenceBreath />
      <AudioControlPanel />
      <CursorProximityScene />

      <CoverScene
        title={coverTitle}
        birthMeta={birthMeta}
        subjects={reading.subjects}
        topologySvg={reading.topologySvg}
      />

      {/* W6 — Chapter 0 / pre-reading synthesis spread. Mock data for now;
          when witness-agents emits a top-level witness_layer for the
          integrated reading, pass it through as the witnessLayer prop. */}
      <WitnessLayerOpener />

      {/* Bento — Subject Compendium. 3-up bento cards (one per native) with
          subject name as massive title + lagna/AK/nakshatra/mahadasha as
          chips inside the featured zone. Mocked from the subject list for
          now; later wire to per-subject metadata from witness-agents. */}
      <SubjectCompendium
        subjects={reading.subjects.map((name, idx) => {
          // Hardcoded W×H×Mohan subject metadata for the demo reading.
          // Later: thread structured per-subject data from witness-agents.
          const wHm = [
            {
              name: "WitnessAlchemist",
              birth_date: "1991-08-13",
              birth_place: "Bangalore, India",
              lagna: "Scorpio",
              atmakaraka: "Jupiter (exalted, 9th)",
              birth_nakshatra: "Uttara Phalguni",
              current_dasha: "Rahu → Jupiter (16 yr)",
            },
            {
              name: "Harshita",
              birth_date: "1987-10-15",
              birth_place: "India",
              lagna: "Aries",
              atmakaraka: "Sun (exalted, 1st)",
              birth_nakshatra: "Pushya",
              current_dasha: "Ketu → Venus (20 yr)",
            },
            {
              name: "Mohan Kumar V",
              birth_date: "1995-11-17",
              birth_place: "Tiruchirappalli, Tamil Nadu",
              lagna: "Capricorn",
              atmakaraka: "Mercury (Raj Yoga, 10th)",
              birth_nakshatra: "Shravana",
              current_dasha: "Mars → Rahu (18 yr)",
            },
          ];
          return wHm[idx] ?? { name };
        })}
      />

      {/* Each Part is a full ChapterScene. ChapterTransition fills the
          gap between consecutive Parts. */}
      {reading.passes.map((pass, i) => {
        const meta = partMeta[i];
        const isLast = i === reading.passes.length - 1;
        const yantraKind = meta.yantraKind;
        const direction = meta.direction;

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
          <React.Fragment key={pass.id}>
            <ChapterScene
              partNum={meta.partNum}
              romanNumeral={meta.romanNumeral}
              title={meta.title}
              direction={direction}
              words={pass.words}
              xrefs={pass.xrefs}
            >
              {/* Bento-shell wrapping the WitnessPulse + YantraPlate +
                  optional DashaWaveform in the brand-design-system layout.
                  The pulse/yantra/waveform render INSIDE bento cards with
                  proper rounded-card framing and chip metadata. */}
              <PartBento
                partNum={meta.partNum}
                romanNumeral={meta.romanNumeral}
                title={meta.title}
                direction={direction as WitnessDirection}
                words={pass.words}
                xrefs={pass.xrefs}
                pulseSlot={
                  <WitnessPulse
                    direction={direction as WitnessDirection}
                    title={pass.title}
                  />
                }
                yantraSlot={
                  <div data-proximity="yantra">
                    <YantraPlate kind={yantraKind} data={yantraData} />
                  </div>
                }
                extraSlot={
                  i === 2 ? (
                    <DashaWaveform
                      periods={mockDashaPeriods}
                      pivots={mockPivots}
                    />
                  ) : undefined
                }
              />

              <VerseFlow blocks={pass.blocks} />
            </ChapterScene>

            {!isLast && (
              <ChapterTransition
                fromPart={meta.partNum}
                toPart={meta.partNum + 1}
                toRomanNumeral={partMeta[i + 1].romanNumeral}
                toTitle={partMeta[i + 1].title}
                toDirection={partMeta[i + 1].direction}
              />
            )}
          </React.Fragment>
        );
      })}

      {/* Closing scene — preserves The Quine footer as a final beat. */}
      <section
        style={{
          position: "relative",
          width: "100%",
          padding:
            "clamp(4rem, 8vh, 7rem) clamp(1rem, 2.4vw, 2.5rem) clamp(4rem, 8vh, 7rem)",
          zIndex: 2,
        }}
      >
        <LaArcFade />
        <footer
          style={{
            margin: "clamp(2rem, 4vh, 4rem) auto 0",
            maxWidth: "60rem",
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
          <div
            style={{
              marginTop: "1rem",
              maxWidth: "48ch",
              margin: "1rem auto 0",
              fontStyle: "italic",
              color: "var(--muted)",
            }}
          >
            This document is documentation of an instrument. The instrument is
            what you already are. The Quine principle: the system succeeds when
            you no longer need it.
          </div>
        </footer>
      </section>

      {/* Global chapter chrome — always visible while reading. */}
      <ChapterProgress
        parts={partMeta.map((m) => ({
          partNum: m.partNum,
          romanNumeral: m.romanNumeral,
          title: m.title,
        }))}
      />
      <ChapterNavigator
        parts={partMeta.map((m) => ({
          partNum: m.partNum,
          romanNumeral: m.romanNumeral,
          title: m.title,
        }))}
      />

      {/* W6 — Engine drill-down overlay. Rendered last so it stacks above
          all article content. AnimatePresence inside handles enter/exit. */}
      <EngineDrillDown
        engineId={activeDrilldown?.engineId ?? null}
        result={activeDrilldown?.result}
        onClose={closeDrilldown}
      />
     </DrilldownContext.Provider>
    </AudioStateProvider>
  );
}
