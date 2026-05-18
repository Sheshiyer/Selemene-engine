// ─── WitnessPulse — breathing ring opener for major sections (h2) ──────
// Renders the heading as a witness pulse: concentric rings on the left,
// cardinal-direction eyebrow on the right, the heading text below.
// Matches the breathnav-screen.png pattern from the brand reference.

import { ConcentricRings } from "../sigils";

interface WitnessPulseProps {
  text: string;
  /** Cardinal direction shown above the heading (STABILIZE/HEAL/CREATE/MUTATE). */
  cardinal?: string;
  accentColor: string;
  /** Level — h2 gets the full pulse; h3/h4 get a smaller variant. */
  level?: 2 | 3 | 4;
}

export function WitnessPulse({
  text,
  cardinal,
  accentColor,
  level = 2,
}: WitnessPulseProps) {
  const isMajor = level === 2;
  return (
    <header
      style={{
        position: "relative",
        margin: isMajor
          ? "clamp(3rem, 6vh, 5rem) 0 clamp(1.5rem, 3vh, 2.5rem)"
          : "clamp(2rem, 4vh, 3rem) 0 clamp(1rem, 2vh, 1.75rem)",
        display: "grid",
        gridTemplateColumns: "auto 1fr",
        alignItems: "center",
        gap: "clamp(1rem, 2vw, 1.75rem)",
      }}
    >
      {/* Concentric ring ornament */}
      <div
        aria-hidden="true"
        style={{
          width: isMajor ? "clamp(56px, 7vw, 88px)" : "clamp(40px, 5vw, 56px)",
          height: isMajor ? "clamp(56px, 7vw, 88px)" : "clamp(40px, 5vw, 56px)",
          opacity: 0.92,
          flexShrink: 0,
          filter: `drop-shadow(0 0 14px ${accentColor}55)`,
          animation: "depth-pulse 4.2s ease-in-out infinite",
        }}
      >
        <ConcentricRings
          size={isMajor ? 88 : 56}
          color={accentColor}
          innerColor="#F0EDE3"
        />
      </div>

      <div style={{ display: "grid", gap: "0.35rem", minWidth: 0 }}>
        {cardinal && (
          <div
            style={{
              fontFamily: "var(--font-mono, 'SF Mono', monospace)",
              fontSize: "clamp(0.6rem, 0.72vw, 0.72rem)",
              letterSpacing: "0.5em",
              textTransform: "uppercase",
              color: accentColor,
              opacity: 0.82,
            }}
          >
            {cardinal}
          </div>
        )}
        <h2
          style={{
            margin: 0,
            fontFamily: "var(--font-display, 'Panchang', serif)",
            fontVariationSettings: `'wght' ${isMajor ? 700 : 620}`,
            fontSize: isMajor
              ? "clamp(1.5em, 2.8vw, 2.4em)"
              : "clamp(1.2em, 2vw, 1.7em)",
            lineHeight: 1.05,
            letterSpacing: "-0.01em",
            color: "var(--c-parchment, #F0EDE3)",
          }}
        >
          {text}
        </h2>
      </div>

      <style>{`
        @keyframes depth-pulse {
          0%, 100% { transform: scale(0.96); opacity: 0.85; }
          50%      { transform: scale(1.02); opacity: 1.0; }
        }
      `}</style>
    </header>
  );
}
