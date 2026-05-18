// ─── DecisionPlate — vertical action-moment stack for blockquotes ──────
// When the prose has a `> ...` block (typically an action prompt or
// pivotal moment), render as a centered vertical stack:
//
//   ╭───────────────╮
//   │   ◉ COHERENCE │
//   │     OPTIMAL  │   ← gold pulse circle
//   ╰───────────────╯
//
//   ┌────────────────────┐
//   │  ⌬  the line       │   ← sacred-gold CTA pill (first line)
//   └────────────────────┘
//
//   the rest of the lines as supporting verse
//
//   "trailing quote ―"
//
// Inspired by decisionscreen.png from brand reference.

import { renderInline } from "../parseBlocks";

interface DecisionPlateProps {
  lines: string[];
  accentColor: string;
}

function withAlpha(hex: string, alpha: number): string {
  const a = Math.round(Math.max(0, Math.min(1, alpha)) * 255)
    .toString(16)
    .padStart(2, "0");
  return `${hex}${a}`;
}

export function DecisionPlate({ lines, accentColor }: DecisionPlateProps) {
  if (lines.length === 0) return null;
  const [head, ...rest] = lines.filter((l) => l.trim());

  return (
    <aside
      style={{
        margin: "clamp(3rem, 6vh, 5rem) auto",
        maxWidth: "min(520px, 90%)",
        padding: "clamp(1.5rem, 3vh, 2.25rem) clamp(1.5rem, 3vw, 2.5rem)",
        background: `linear-gradient(160deg, ${withAlpha(accentColor, 0.10)} 0%, ${withAlpha("#070B1D", 0.6)} 100%)`,
        border: `1px solid ${withAlpha(accentColor, 0.30)}`,
        borderRadius: "20px",
        textAlign: "center",
        position: "relative",
        boxShadow: `0 30px 80px -40px ${withAlpha(accentColor, 0.5)}, inset 0 1px 0 ${withAlpha("#F0EDE3", 0.1)}`,
      }}
      aria-label="Decision plate"
    >
      {/* Mandala pulse — small gold dot inside concentric rings */}
      <div
        aria-hidden="true"
        style={{
          width: "44px",
          height: "44px",
          margin: "0 auto clamp(1rem, 2vh, 1.5rem)",
          borderRadius: "50%",
          background: `radial-gradient(circle, ${accentColor} 0%, ${withAlpha(accentColor, 0.5)} 30%, transparent 70%)`,
          animation: "depth-coherence 3.6s ease-in-out infinite",
        }}
      />

      {/* CTA pill — the headline line */}
      <div
        style={{
          display: "inline-block",
          padding: "0.6rem 1.4rem",
          fontFamily: "var(--font-display, 'Panchang', serif)",
          fontVariationSettings: "'wght' 640",
          fontSize: "clamp(1rem, 1.4vw, 1.25rem)",
          letterSpacing: "-0.005em",
          lineHeight: 1.25,
          color: "var(--c-void, #070B1D)",
          background: `linear-gradient(135deg, ${accentColor} 0%, ${withAlpha(accentColor, 0.85)} 100%)`,
          borderRadius: "999px",
          boxShadow: `0 10px 28px -12px ${withAlpha(accentColor, 0.65)}`,
          marginBottom: rest.length > 0 ? "clamp(1.25rem, 2.5vh, 1.75rem)" : 0,
          maxWidth: "100%",
        }}
        dangerouslySetInnerHTML={{ __html: renderInline(head ?? "") }}
      />

      {/* Supporting lines */}
      {rest.length > 0 && (
        <div
          style={{
            display: "grid",
            gap: "0.6rem",
            fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
            fontSize: "clamp(0.9rem, 1.05vw, 1.05rem)",
            lineHeight: 1.55,
            color: "rgba(240, 237, 227, 0.88)",
            maxWidth: "44ch",
            margin: "0 auto",
          }}
        >
          {rest.map((line, i) => (
            <p
              key={i}
              style={{ margin: 0 }}
              dangerouslySetInnerHTML={{ __html: renderInline(line) }}
            />
          ))}
        </div>
      )}

      <style>{`
        @keyframes depth-coherence {
          0%, 100% { transform: scale(0.92); opacity: 0.78; }
          50%      { transform: scale(1.08); opacity: 1.0;  }
        }
      `}</style>
    </aside>
  );
}
