"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/10-result-mandala-spec.png.
 */

import { ResultMandala } from "@/components/ResultMandala";
import type { BiofieldMetrics } from "@/lib/selemene/biofield-domain";

// Current reading — strong, mostly above baseline.
const reading: BiofieldMetrics = {
  light_quanta_density: 0.58,
  normalized_area: 0.55,
  average_intensity: 0.78,
  inner_noise: 0.28, // -> coherence 0.72
  energy_analysis: { low: 0.3, medium: 0.45, high: 0.25, total: 1 },
  entropy_form_coefficient: 0.64,
  fractal_dimension: 1.7,
  correlation_dimension: 1.4,
  body_symmetry: 0.81,
  contour_complexity: 0.5,
  pattern_regularity: 0.88,
};

// Baseline — earlier session, generally lower so deltas read as improvement,
// but a couple of metrics regressed (luminosity, entropy) to show gold deltas.
const baseline: BiofieldMetrics = {
  light_quanta_density: 0.7, // declined -> gold
  normalized_area: 0.5,
  average_intensity: 0.6, // improved -> emerald
  inner_noise: 0.42, // coherence improved -> emerald
  energy_analysis: { low: 0.34, medium: 0.4, high: 0.26, total: 1 },
  entropy_form_coefficient: 0.72, // declined -> gold
  fractal_dimension: 1.6,
  correlation_dimension: 1.3,
  body_symmetry: 0.66, // improved -> emerald
  contour_complexity: 0.45,
  pattern_regularity: 0.74, // improved -> emerald
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
const legend: React.CSSProperties = {
  display: "flex",
  gap: "1.5rem",
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.65rem",
  letterSpacing: "0.1em",
  color: "#8A9BA8",
};
const swatch = (c: string): React.CSSProperties => ({
  display: "inline-block",
  width: 10,
  height: 10,
  background: c,
  marginRight: 6,
  verticalAlign: "middle",
});

export default function ResultMandalaPreview() {
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
        <ResultMandala
          metrics={reading}
          baseline={baseline}
          analysisVersion="ANL-V3.2.0-2305Z9"
          accepted={true}
        />
        <span style={label}>accepted · with baseline deltas</span>
        <div style={legend}>
          <span>
            <span style={swatch("#10B5A7")} />
            improvement
          </span>
          <span>
            <span style={swatch("#C5A017")} />
            decline
          </span>
        </div>
      </div>

      <div style={cell}>
        <ResultMandala
          metrics={baseline}
          baseline={null}
          analysisVersion="ANL-V3.2.0-2305Z9"
          accepted={false}
        />
        <span style={label}>rejected · no baseline</span>
      </div>
    </main>
  );
}
