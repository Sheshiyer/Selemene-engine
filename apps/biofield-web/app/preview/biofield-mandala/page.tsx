"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/05-metrics-panel-spec.png.
 */

import { BiofieldMandala } from "@/components/BiofieldMandala";
import type { BiofieldMetrics } from "@selemene/biofield-domain";

// Realistic capture — values land mid-to-high, mirroring the spec sample sheet
// (LQD 0.73 · NORM 0.68 · INTENSITY 0.81 · NOISE 0.22 · SYMMETRY 0.76 · …).
const populated: BiofieldMetrics = {
  light_quanta_density: 730, // /1000 -> 0.73
  normalized_area: 0.68,
  average_intensity: 206, // /255 -> ~0.81
  inner_noise: 0.22,
  energy_analysis: { low: 0.31, medium: 0.44, high: 0.25, total: 1 },
  entropy_form_coefficient: 0.47,
  fractal_dimension: 1.62, // -1 -> 0.62
  correlation_dimension: 0.54,
  body_symmetry: 0.76,
  contour_complexity: 1.18, // /2 -> 0.59
  pattern_regularity: 0.71,
};

// Partial / attuning capture — lower coherence across the board.
const partial: BiofieldMetrics = {
  light_quanta_density: 340,
  normalized_area: 0.33,
  average_intensity: 92,
  inner_noise: 0.58,
  energy_analysis: { low: 0.6, medium: 0.28, high: 0.12, total: 1 },
  entropy_form_coefficient: 0.71,
  fractal_dimension: 1.28,
  correlation_dimension: 0.31,
  body_symmetry: 0.41,
  contour_complexity: 0.72,
  pattern_regularity: 0.27,
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

export default function BiofieldMandalaPreview() {
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
        <BiofieldMandala metrics={populated} size={420} />
        <span style={label}>populated · coherent</span>
      </div>
      <div style={cell}>
        <BiofieldMandala metrics={partial} size={420} />
        <span style={label}>partial · attuning</span>
      </div>
      <div style={cell}>
        <BiofieldMandala metrics={populated} size={260} />
        <span style={label}>compact</span>
      </div>
    </main>
  );
}
