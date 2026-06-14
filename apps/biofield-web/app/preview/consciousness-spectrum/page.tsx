"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/08-consciousness-spectrum-spec.png.
 */

import { ConsciousnessSpectrum } from "@/components/ConsciousnessSpectrum";

const cell: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "1rem",
  width: "100%",
  maxWidth: 760,
};
const label: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.7rem",
  letterSpacing: "0.12em",
  textTransform: "uppercase",
  color: "#8A9BA8",
};

export default function ConsciousnessSpectrumPreview() {
  return (
    <main
      style={{
        minHeight: "100dvh",
        background: "#070B1D",
        padding: "3rem",
        display: "flex",
        flexDirection: "column",
        gap: "4rem",
        alignItems: "center",
      }}
    >
      <div style={cell}>
        <ConsciousnessSpectrum level={2} />
        <span style={label}>level 2 · flow</span>
      </div>
      <div style={cell}>
        <ConsciousnessSpectrum level={4} />
        <span style={label}>level 4 · activation</span>
      </div>
    </main>
  );
}
