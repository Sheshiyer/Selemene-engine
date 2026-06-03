"use client";

/**
 * Wave-1 foundations preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/11-foundations-spec.png.
 */

import { SigilButton } from "@/components/foundations/Button";
import { HrvGraph } from "@/components/foundations/HrvGraph";
import { SessionStrip } from "@/components/foundations/SessionStrip";

const sectionTitle: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.72rem",
  letterSpacing: "0.22em",
  textTransform: "uppercase",
  color: "#C5A017",
  opacity: 0.8,
  marginBottom: "1.25rem",
};

const rowLabel: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.6rem",
  letterSpacing: "0.16em",
  textTransform: "uppercase",
  color: "#8A9BA8",
  width: 96,
  flexShrink: 0,
};

const row: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: "1rem",
  flexWrap: "wrap",
  marginBottom: "1rem",
};

const block: React.CSSProperties = {
  width: "100%",
  maxWidth: 960,
};

export default function FoundationsPreview() {
  return (
    <main
      style={{
        minHeight: "100dvh",
        background: "#070B1D",
        padding: "3rem clamp(1.5rem, 4vw, 3rem)",
        display: "flex",
        flexDirection: "column",
        gap: "3.5rem",
        alignItems: "center",
      }}
    >
      {/* ── 1 · BUTTONS ── */}
      <section style={block}>
        <h2 style={sectionTitle}>1 · Buttons</h2>

        <div style={row}>
          <span style={rowLabel}>Primary</span>
          <SigilButton variant="primary">Enter Field</SigilButton>
          <SigilButton variant="primary" disabled>
            Disabled
          </SigilButton>
        </div>

        <div style={row}>
          <span style={rowLabel}>Secondary</span>
          <SigilButton variant="secondary">View History</SigilButton>
          <SigilButton variant="secondary" disabled>
            Disabled
          </SigilButton>
        </div>

        <div style={row}>
          <span style={rowLabel}>Ghost</span>
          <SigilButton variant="ghost">Learn More</SigilButton>
          <SigilButton variant="ghost" disabled>
            Disabled
          </SigilButton>
        </div>

        <div style={row}>
          <span style={rowLabel}>Icon</span>
          <SigilButton variant="primary" icon>
            Compass primary
          </SigilButton>
          <SigilButton variant="secondary" icon>
            Compass secondary
          </SigilButton>
          <SigilButton variant="primary" icon disabled>
            Compass disabled
          </SigilButton>
        </div>
      </section>

      {/* ── 2 · GRAPH ── */}
      <section style={block}>
        <h2 style={sectionTitle}>2 · Graph</h2>
        <HrvGraph />
      </section>

      {/* ── 3 · SESSION STRIP ── */}
      <section style={block}>
        <h2 style={sectionTitle}>3 · Session Strip</h2>
        <div style={{ display: "flex", flexDirection: "column", gap: "1.5rem" }}>
          <SessionStrip
            email="seeker@noesis.field"
            status="active"
            tier="Practitioner"
            onStart={() => {}}
            onEnd={() => {}}
          />
          <SessionStrip status="none" onStart={() => {}} onEnd={() => {}} />
        </div>
      </section>
    </main>
  );
}
