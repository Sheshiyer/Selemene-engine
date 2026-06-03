"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/06-fractal-chaos-spec.png.
 */

import { FractalSignature } from "@/components/FractalSignature";

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

export default function FractalSignaturePreview() {
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
        <FractalSignature
          fractalDimension={1.42}
          correlationDimension={1.18}
          entropy={0.67}
        />
        <span style={label}>complex · hero</span>
      </div>
    </main>
  );
}
