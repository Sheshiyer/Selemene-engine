"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/02-cosmogram-spec.png.
 */

import { CosmogramRing } from "@/components/CosmogramRing";
import type { CompositeScores } from "@/components/pip/types";

const coherent: CompositeScores = {
  overallCoherence: 0.78,
  bodySymmetry: 0.64,
  lightQuantaDensity: 0.63,
  patternRegularity: 0.71,
  normalizedArea: 0.55,
};

const low: CompositeScores = {
  overallCoherence: 0.34,
  bodySymmetry: 0.28,
  lightQuantaDensity: 0.31,
  patternRegularity: 0.22,
  normalizedArea: 0.3,
};

const cell: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "0.75rem",
};
const label: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.7rem",
  letterSpacing: "0.12em",
  textTransform: "uppercase",
  color: "#8A9BA8",
};

export default function CosmogramPreview() {
  return (
    <main
      style={{
        minHeight: "100dvh",
        background: "#070B1D",
        padding: "3rem",
        display: "flex",
        flexWrap: "wrap",
        gap: "3rem",
        alignItems: "flex-start",
        justifyContent: "center",
      }}
    >
      <div style={cell}>
        <CosmogramRing scores={coherent} size="large" />
        <span style={label}>coherent · large</span>
      </div>
      <div style={cell}>
        <CosmogramRing scores={low} size="large" />
        <span style={label}>attuning · large</span>
      </div>
      <div style={cell}>
        <CosmogramRing scores={coherent} size="compact" />
        <span style={label}>compact</span>
      </div>
      <div style={cell}>
        <CosmogramRing scores={coherent} size="mini" />
        <span style={label}>mini</span>
      </div>
    </main>
  );
}
