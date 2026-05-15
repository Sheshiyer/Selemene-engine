"use client";

// ─── PartBento — bento-shell that wraps each Part's WitnessPulse + YantraPlate ──
// Per-Part scene becomes a BentoGrid:
//   [   Massive Part title card (spans 2)   ] [ Cardinal direction stat ]
//   [        WitnessPulse breathing ring (span 1)       ] [ YantraPlate (span 2)  ]
//   [          Cross-Part meta chips (span 3)                                 ]
//
// Composition stays declarative — caller passes WitnessPulse + YantraPlate +
// any sub-blocks; PartBento arranges them in a bento grid framing.

import { type ReactNode } from "react";
import { BentoCard } from "./BentoCard";
import { BentoGrid } from "./BentoGrid";
import { BentoChip } from "./BentoChip";
import { BentoStat } from "./BentoStat";

interface PartBentoProps {
  partNum: number;
  romanNumeral: string;
  title: string;
  direction: "STABILIZE" | "HEAL" | "CREATE" | "MUTATE";
  words: number;
  xrefs: number;
  /** Slot for the WitnessPulse breathing ring */
  pulseSlot: ReactNode;
  /** Slot for the YantraPlate mandala */
  yantraSlot: ReactNode;
  /** Optional DashaWaveform / DecisionPlate slot below the bento */
  extraSlot?: ReactNode;
}

const DIRECTION_TONE: Record<
  PartBentoProps["direction"],
  "violet" | "indigo" | "emerald" | "gold"
> = {
  STABILIZE: "violet",
  HEAL: "indigo",
  CREATE: "emerald",
  MUTATE: "gold",
};

const DIRECTION_BLURB: Record<PartBentoProps["direction"], string> = {
  STABILIZE: "The shared bedrock — what holds the field together.",
  HEAL: "The resonance — how the field receives + repairs itself.",
  CREATE: "The phase-lock — when the field generates new geometry.",
  MUTATE: "The anti-dependency — what the field releases as it matures.",
};

export function PartBento({
  partNum,
  romanNumeral,
  title,
  direction,
  words,
  xrefs,
  pulseSlot,
  yantraSlot,
  extraSlot,
}: PartBentoProps) {
  const tone = DIRECTION_TONE[direction];
  const blurb = DIRECTION_BLURB[direction];

  return (
    <section
      id={`part-${partNum}`}
      style={{
        position: "relative",
        width: "100%",
        zIndex: 3,
        margin: "clamp(2rem, 4vw, 5rem) 0 clamp(2rem, 4vw, 4rem)",
      }}
    >
      <BentoGrid>
        {/* Massive Part title card — spans 2 columns */}
        <BentoCard
          span={2}
          eyebrow={`Part ${romanNumeral} · ${direction}`}
          title={title}
          description={blurb}
          tone={tone}
          status="CHAPTER"
          hasFeature
        >
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              gap: "clamp(0.6rem, 1vw, 1rem)",
              alignItems: "center",
            }}
          >
            <BentoChip label="Cardinal" variant="gold">
              {direction}
            </BentoChip>
            <BentoChip label="Roman" variant="ghost">
              {romanNumeral}
            </BentoChip>
            <BentoChip label="Part" variant="default">
              {String(partNum).padStart(2, "0")}
            </BentoChip>
          </div>
        </BentoCard>

        {/* Stat card — words + xrefs */}
        <BentoCard
          span={1}
          eyebrow="Field weight"
          title=""
          hasFeature={false}
        >
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "clamp(0.75rem, 1.4vw, 1.4rem)",
              paddingBottom: "clamp(1.25rem, 2vw, 2rem)",
            }}
          >
            <BentoStat
              label="Words"
              value={words.toLocaleString()}
              accent="parchment"
            />
            <BentoStat
              label="Cross-refs"
              value={xrefs.toLocaleString()}
              accent="emerald"
            />
          </div>
        </BentoCard>

        {/* WitnessPulse — span 1 */}
        <BentoCard
          span={1}
          eyebrow="Breath"
          title="4 · 7 · 8"
          description="The breathing cycle of this chapter."
          tone={tone}
          hasFeature
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              width: "100%",
              minHeight: "clamp(12rem, 22vw, 18rem)",
            }}
          >
            {pulseSlot}
          </div>
        </BentoCard>

        {/* YantraPlate — span 2 */}
        <BentoCard
          span={2}
          eyebrow="Yantra"
          title="Signature Geometry"
          description="The mandala that anchors this chapter's reading."
          tone="violet"
          hasFeature
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              width: "100%",
              minHeight: "clamp(14rem, 28vw, 24rem)",
            }}
          >
            {yantraSlot}
          </div>
        </BentoCard>

        {/* Optional extras (DashaWaveform / DecisionPlate) — span 3 (full) */}
        {extraSlot && (
          <BentoCard
            span={3}
            eyebrow="Phase Geometry"
            title="Time Becomes Visible"
            description="The mahadasha sequence rendered as a coherent ribbon."
            tone="emerald"
            hasFeature
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: "100%",
              }}
            >
              {extraSlot}
            </div>
          </BentoCard>
        )}
      </BentoGrid>
    </section>
  );
}
