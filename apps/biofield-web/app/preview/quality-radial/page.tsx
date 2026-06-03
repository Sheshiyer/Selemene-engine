"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/07-quality-gauge-spec.png.
 */

import { QualityRadial } from "@/components/QualityRadial";
import type { QualityAssessment } from "@selemene/biofield-domain";

const sufficient: QualityAssessment = {
  sharpness: 0.82,
  contrast: 0.74,
  noise_level: 0.18,
  exposure: 0.61,
  sufficient_quality: true,
};

const insufficient: QualityAssessment = {
  sharpness: 0.34,
  contrast: 0.29,
  noise_level: 0.62,
  exposure: 0.4,
  sufficient_quality: false,
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

export default function QualityRadialPreview() {
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
        <QualityRadial quality={sufficient} />
        <span style={label}>sufficient</span>
      </div>
      <div style={cell}>
        <QualityRadial quality={insufficient} />
        <span style={label}>insufficient</span>
      </div>
    </main>
  );
}
