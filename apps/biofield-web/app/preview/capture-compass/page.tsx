"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/09-capture-compass-spec.png.
 */

import { CaptureCompass } from "@/components/CaptureCompass";

const cell: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "1rem",
};
const label: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.7rem",
  letterSpacing: "0.12em",
  textTransform: "uppercase",
  color: "#8A9BA8",
};

export default function CaptureCompassPreview() {
  return (
    <main
      style={{
        minHeight: "100dvh",
        background: "#070B1D",
        padding: "3rem",
        display: "flex",
        flexWrap: "wrap",
        gap: "3.5rem",
        alignItems: "flex-start",
        justifyContent: "center",
      }}
    >
      <div style={cell}>
        <CaptureCompass state="idle" progress={0} mode="FLOW" />
        <span style={label}>idle · awaiting intent</span>
      </div>

      <div style={cell}>
        <CaptureCompass
          state="uploaded"
          progress={0.6}
          mode="FLOW"
          paused={false}
          onCapture={() => {}}
          onPause={() => {}}
          onResume={() => {}}
        />
        <span style={label}>capturing · progress 0.6</span>
      </div>

      <div style={cell}>
        <CaptureCompass state="persisted" progress={1} mode="COHERENCE" />
        <span style={label}>persisted · emerald pulse</span>
      </div>

      <div style={cell}>
        <CaptureCompass state="rejected" progress={0.45} mode="STRUCTURE" />
        <span style={label}>rejected · terracotta notch</span>
      </div>
    </main>
  );
}
