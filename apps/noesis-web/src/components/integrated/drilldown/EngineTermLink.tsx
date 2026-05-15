"use client";

// ─── EngineTermLink — inline drill-down trigger for technical terms ─────
// Wraps a single technical term (Mahadasha, Nakshatra, Sade Sati, …) in
// the reading prose. Dotted gold underline, hover tooltip with engine
// name + 1-line definition. Click opens the EngineDrillDown panel.
//
// Per design v2 § 5.10 — taps on bolded engine references slide the
// matching panel in from the right.
//
// This component is used as a React element AND as a hydration target for
// DOM-walked [data-engine-term] spans (see VerseFlow.tsx). Both forms
// reach into DrilldownContext to fire the open() call.

import { useState, useId, type ReactNode } from "react";
import { useDrilldown } from "./DrilldownContext";

interface EngineTermLinkProps {
  term: string;
  engineId: string;
  definition: string;
  children?: ReactNode;
}

const linkStyle: React.CSSProperties = {
  borderBottom: "1px dotted var(--c-gold, #d8b56e)",
  color: "var(--c-gold, #d8b56e)",
  cursor: "help",
  background: "none",
  padding: "0 0.05em",
  minHeight: 24,
  display: "inline-block",
  lineHeight: "inherit",
  fontFamily: "inherit",
  fontSize: "inherit",
  fontWeight: "inherit",
  textDecoration: "none",
  transition: "color 0.15s ease, border-color 0.15s ease",
};

const linkHoverStyle: React.CSSProperties = {
  color: "var(--c-parchment, #f3ead8)",
  borderBottomColor: "var(--c-parchment, #f3ead8)",
};

const tooltipStyle: React.CSSProperties = {
  position: "absolute",
  bottom: "calc(100% + 8px)",
  left: "50%",
  transform: "translateX(-50%)",
  minWidth: 200,
  maxWidth: 280,
  padding: "0.55rem 0.75rem",
  background: "rgba(7,11,29,0.96)",
  border: "1px solid var(--line, rgba(216,181,110,0.35))",
  borderRadius: 6,
  fontSize: "0.78rem",
  lineHeight: 1.45,
  color: "var(--text, rgba(255,255,255,0.92))",
  fontFamily: "var(--font-body)",
  boxShadow: "0 8px 24px rgba(0,0,0,0.45)",
  whiteSpace: "normal",
  zIndex: 60,
  pointerEvents: "none",
  textAlign: "left",
};

const tooltipLabelStyle: React.CSSProperties = {
  fontFamily: "var(--font-mono)",
  fontSize: "0.62rem",
  letterSpacing: "0.18em",
  textTransform: "uppercase",
  color: "var(--c-gold, #d8b56e)",
  marginBottom: "0.25rem",
  display: "block",
};

function engineLabel(id: string): string {
  return id
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

export function EngineTermLink({
  term,
  engineId,
  definition,
  children,
}: EngineTermLinkProps) {
  const { open, engineOutputs } = useDrilldown();
  const [hover, setHover] = useState(false);
  const [focus, setFocus] = useState(false);
  const tooltipId = useId();
  const show = hover || focus;

  return (
    <span style={{ position: "relative", display: "inline-block" }}>
      <button
        type="button"
        style={show ? { ...linkStyle, ...linkHoverStyle } : linkStyle}
        onMouseEnter={() => setHover(true)}
        onMouseLeave={() => setHover(false)}
        onFocus={() => setFocus(true)}
        onBlur={() => setFocus(false)}
        onClick={(e) => {
          e.preventDefault();
          open({ engineId, result: engineOutputs[engineId] });
        }}
        aria-label={`Open ${engineLabel(engineId)} drilldown for ${term}`}
        aria-describedby={tooltipId}
      >
        {children ?? term}
      </button>
      {show && (
        <span role="tooltip" id={tooltipId} style={tooltipStyle}>
          <span style={tooltipLabelStyle}>{engineLabel(engineId)}</span>
          {definition}
        </span>
      )}
    </span>
  );
}
