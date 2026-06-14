"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/04-scorecard-spec.png.
 */

import { ScoreCard } from "@/components/ScoreCard";

const metrics: { label: string; value: number; baseline?: number }[] = [
  { label: "Coherence", value: 0.78, baseline: 0.5 },
  { label: "Symmetry", value: 0.42, baseline: 0.5 },
  { label: "Light Density", value: 0.63, baseline: 0.5 },
  { label: "Pattern Regularity", value: 0.71, baseline: 0.5 },
  { label: "Fractal Dimension", value: 0.58, baseline: 0.5 },
];

const cell: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "0.75rem",
};
const caption: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.7rem",
  letterSpacing: "0.12em",
  textTransform: "uppercase",
  color: "#8A9BA8",
};

export default function ScoreCardPreview() {
  return (
    <main
      style={{
        minHeight: "100dvh",
        background: "#070B1D",
        padding: "3rem",
        display: "flex",
        flexDirection: "column",
        gap: "3rem",
        alignItems: "center",
      }}
    >
      {/* Hero — one large node */}
      <div style={cell}>
        <ScoreCard label="Coherence" value={0.78} baseline={0.5} size="large" />
        <span style={caption}>coherence · large</span>
      </div>

      {/* Variants row — compact, mirrors the spec's variant strip */}
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: "2.5rem",
          alignItems: "flex-start",
          justifyContent: "center",
        }}
      >
        {metrics.map((m) => (
          <div key={m.label} style={cell}>
            <ScoreCard
              label={m.label}
              value={m.value}
              baseline={m.baseline}
              size="compact"
            />
            <span style={caption}>{m.label}</span>
          </div>
        ))}
      </div>

      {/* Mini row — dense readout scale */}
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: "1.75rem",
          alignItems: "flex-start",
          justifyContent: "center",
        }}
      >
        {metrics.map((m) => (
          <div key={m.label} style={cell}>
            <ScoreCard label={m.label} value={m.value} size="mini" />
            <span style={caption}>{m.label} · mini</span>
          </div>
        ))}
      </div>
    </main>
  );
}
