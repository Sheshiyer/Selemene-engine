"use client";

// ─── SigilCascade — sigil-bulleted illumination column ─────────────────
// Per design-v2 § 5.7. Replaces non-Native tables. Each entry is a
// motion.div that illuminates 0.22 → 1.0 as it enters viewport-center.
// A 12×12 SVG glyph sits in a 24×24 box as the leading micro-sigil.

import { motion, useInView, useReducedMotion } from "motion/react";
import { useRef } from "react";

export interface SigilCascadeEntry {
  term: string;
  value?: string;
  subEntries?: SigilCascadeEntry[];
}

interface SigilCascadeProps {
  entries: SigilCascadeEntry[];
}

/** Tiny ∴ glyph as an SVG (rendered as three dots). The dots inherit
 *  fill from `currentColor` so per-level coloring works via parent CSS. */
function SigilGlyph({ size = 12 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 12 12"
      aria-hidden="true"
      style={{ display: "block", overflow: "visible" }}
    >
      <circle cx="6" cy="2.5" r="1.1" fill="currentColor" />
      <circle cx="2.5" cy="8.5" r="1.1" fill="currentColor" />
      <circle cx="9.5" cy="8.5" r="1.1" fill="currentColor" />
      {/* Hairline triadic-connection lines */}
      <path
        d="M 6 2.5 L 2.5 8.5 L 9.5 8.5 Z"
        fill="none"
        stroke="currentColor"
        strokeOpacity="0.35"
        strokeWidth="0.4"
      />
    </svg>
  );
}

interface SigilEntryProps {
  entry: SigilCascadeEntry;
  depth: number;
}

function SigilEntry({ entry, depth }: SigilEntryProps) {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { margin: "-30% 0% -30% 0%", once: false });
  const reduced = useReducedMotion();
  const glyphSize = depth === 0 ? 12 : 9;
  return (
    <motion.div
      ref={ref}
      style={{
        position: "relative",
        marginLeft: depth * 32,
        marginBottom: depth === 0 ? "1.05rem" : "0.55rem",
        paddingLeft: 36,
        willChange: "opacity",
      }}
      initial={false}
      animate={reduced ? { opacity: 1 } : { opacity: inView ? 1 : 0.22 }}
      transition={{ duration: 0.55, ease: [0.2, 0.7, 0.2, 1] }}
    >
      <span
        style={{
          position: "absolute",
          left: 0,
          top: depth === 0 ? "0.25rem" : "0.2rem",
          width: 24,
          height: 24,
          display: "inline-flex",
          alignItems: "center",
          justifyContent: "center",
          color: depth === 0 ? "var(--c-gold)" : "var(--c-emerald)",
          opacity: 0.85,
        }}
        aria-hidden="true"
      >
        <SigilGlyph size={glyphSize} />
      </span>
      <div
        style={{
          fontFamily: "var(--font-display)",
          fontWeight: 500,
          fontSize: depth === 0 ? "1.02rem" : "0.92rem",
          color: "var(--c-gold)",
          lineHeight: 1.4,
          letterSpacing: "0.005em",
        }}
      >
        {entry.term}
      </div>
      {entry.value && (
        <div
          style={{
            fontFamily: "var(--font-body)",
            fontWeight: 400,
            fontSize: depth === 0 ? "0.96rem" : "0.88rem",
            color: "var(--c-parchment)",
            opacity: 0.92,
            lineHeight: 1.55,
            marginTop: "0.2rem",
          }}
        >
          {entry.value}
        </div>
      )}
      {entry.subEntries && entry.subEntries.length > 0 && (
        <div style={{ marginTop: "0.6rem" }}>
          {entry.subEntries.map((s, i) => (
            <SigilEntry key={i} entry={s} depth={depth + 1} />
          ))}
        </div>
      )}
    </motion.div>
  );
}

export function SigilCascade({ entries }: SigilCascadeProps) {
  return (
    <div
      className="sigil-cascade"
      role="list"
      style={{
        margin: "clamp(1.5rem, 3vw, 2.5rem) 0",
        padding: "1rem 1.25rem",
        borderLeft: "1px solid rgba(197,160,23,0.18)",
        background:
          "linear-gradient(90deg, rgba(197,160,23,0.025) 0%, rgba(7,11,29,0) 60%)",
      }}
    >
      {entries.map((e, i) => (
        <div role="listitem" key={i}>
          <SigilEntry entry={e} depth={0} />
        </div>
      ))}
    </div>
  );
}
