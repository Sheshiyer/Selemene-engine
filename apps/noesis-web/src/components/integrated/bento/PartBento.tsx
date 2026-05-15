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
import { SacredScene } from "../sacred-scene/SacredScene";
import type { SceneKind } from "../sacred-scene/presets";

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

const DIRECTION_SACRED_KIND: Record<PartBentoProps["direction"], SceneKind> = {
  STABILIZE: "stabilize",
  HEAL: "heal",
  CREATE: "create",
  MUTATE: "mutate",
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
  const sceneKind = DIRECTION_SACRED_KIND[direction];

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
        {/* Massive Part title card — spans full row, embeds field-weight stats */}
        <BentoCard
          span={3}
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
              flexDirection: "column",
              gap: "clamp(1rem, 1.6vw, 1.4rem)",
            }}
          >
            {/* Chips row */}
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
            {/* Stats row — words + xrefs inline */}
            <div
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: "clamp(1.5rem, 3vw, 3rem)",
                alignItems: "baseline",
                paddingTop: "clamp(0.5rem, 1vw, 0.8rem)",
                borderTop: "1px solid rgba(197,160,23,0.15)",
              }}
            >
              <BentoStat
                label="Words"
                value={words.toLocaleString()}
                direction="horizontal"
                accent="parchment"
              />
              <BentoStat
                label="Cross-refs"
                value={xrefs.toLocaleString()}
                direction="horizontal"
                accent="emerald"
              />
            </div>
          </div>
        </BentoCard>

        {/* Sacred procedural-shader atmosphere — full-row hero per Part.
            Same GLSL primitive used for the cover, configured per cardinal
            direction (stabilize/heal/create/mutate). Each Part feels
            visually distinct from the same shader codebase. */}
        <BentoCard
          span={3}
          eyebrow="Atmosphere"
          title="The Field Right Now"
          description="The procedural geometry of this chapter rendered through Goethe-spectrum shaders."
          tone={tone}
          hasFeature
        >
          <div
            style={{
              position: "relative",
              width: "100%",
              minHeight: "clamp(16rem, 32vw, 28rem)",
              overflow: "hidden",
              borderRadius: "inherit",
            }}
          >
            <SacredScene kind={sceneKind} intensity={0.95} height="clamp(16rem, 32vw, 28rem)" />
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
