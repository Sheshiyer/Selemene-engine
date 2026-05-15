"use client";

// ─── HexagonTrio — Native × dimension comparison as triadic yantra ─────
// Per design-v2 § 5.6. Replaces ALL Native-comparison tables in the
// reading. Hexagonal cells sit at the vertices of an inverted triangle
// (3-subject triad), a horizontal dyad (2), or a pentagon (5 — family-
// penta mode). Faint Sacred Gold mutual-aspect lines connect every pair
// of hexes so the eye reads the table as a *figure* rather than a grid.

import { motion, useInView, useReducedMotion } from "motion/react";
import { useId, useRef } from "react";

interface HexagonTrioProps {
  subjects: string[];
  columns: string[];
  rows: string[][];
}

// Cell positions as (cx%, cy%) within the bounding container, indexed by
// total subject count. Empty arrays for unsupported counts are caught and
// fall back to a horizontal row.
const POSITIONS: Record<number, Array<[number, number]>> = {
  2: [
    [28, 50],
    [72, 50],
  ],
  3: [
    [50, 18],
    [22, 70],
    [78, 70],
  ],
  4: [
    [28, 22],
    [72, 22],
    [22, 72],
    [78, 72],
  ],
  5: [
    [50, 14],
    [16, 40],
    [84, 40],
    [30, 82],
    [70, 82],
  ],
};

/** Hexagon SVG path centred at the origin with flat-top orientation and
 *  the given "radius" (vertex-distance from centre). Drawn in user units;
 *  the parent SVG decides actual scale. */
function hexPath(r: number): string {
  const pts: Array<[number, number]> = [];
  for (let i = 0; i < 6; i++) {
    const a = (Math.PI / 3) * i + Math.PI / 6; // pointy-top
    pts.push([Math.cos(a) * r, Math.sin(a) * r]);
  }
  return `M ${pts.map(([x, y]) => `${x.toFixed(2)} ${y.toFixed(2)}`).join(" L ")} Z`;
}

interface HexCellProps {
  subject: string;
  columns: string[];
  values: string[];
  index: number;
  total: number;
  onHover: (i: number | null) => void;
  hovered: number | null;
}

function HexCell({ subject, columns, values, index, total, onHover, hovered }: HexCellProps) {
  const ref = useRef<HTMLDivElement>(null);
  const inView = useInView(ref, { margin: "-40% 0% -40% 0%", once: false });
  const focused = inView || hovered === index;
  const reduced = useReducedMotion();
  const gradId = useId();
  return (
    <motion.div
      ref={ref}
      data-hex-index={index}
      style={{
        position: "relative",
        width: 220,
        height: 250,
        margin: "auto",
        cursor: "default",
      }}
      onMouseEnter={() => onHover(index)}
      onMouseLeave={() => onHover(null)}
      onFocus={() => onHover(index)}
      onBlur={() => onHover(null)}
      animate={reduced ? undefined : { scale: focused ? 1.03 : 1 }}
      transition={{ duration: 0.45, ease: [0.2, 0.7, 0.2, 1] }}
      tabIndex={0}
      aria-label={`${subject} — ${total} of ${total} subjects in triad`}
    >
      <svg
        viewBox="-110 -125 220 250"
        width="220"
        height="250"
        style={{ position: "absolute", inset: 0, overflow: "visible" }}
        aria-hidden="true"
      >
        <defs>
          <radialGradient id={`hex-fill-${gradId}`} cx="50%" cy="42%" r="60%">
            <stop offset="0%" stopColor="rgba(197,160,23,0.10)" />
            <stop offset="70%" stopColor="rgba(11,80,251,0.04)" />
            <stop offset="100%" stopColor="rgba(7,11,29,0)" />
          </radialGradient>
        </defs>
        <path
          d={hexPath(110)}
          fill={`url(#hex-fill-${gradId})`}
          stroke="var(--c-gold)"
          strokeWidth={focused ? 2 : 1}
          strokeOpacity={focused ? 0.95 : 0.55}
          style={{ transition: "stroke-width 0.4s ease, stroke-opacity 0.4s ease" }}
        />
      </svg>
      <div
        style={{
          position: "absolute",
          inset: 0,
          padding: "1.4rem 1.4rem 1.2rem",
          display: "flex",
          flexDirection: "column",
          alignItems: "stretch",
          justifyContent: "flex-start",
          textAlign: "center" as const,
          pointerEvents: "none",
        }}
      >
        <div
          style={{
            fontFamily: "var(--font-display)",
            fontWeight: 600,
            fontSize: "0.78rem",
            letterSpacing: "0.22em",
            textTransform: "uppercase",
            color: "var(--c-gold)",
            marginBottom: "0.7rem",
            lineHeight: 1.15,
          }}
        >
          {subject}
        </div>
        <dl
          style={{
            margin: 0,
            display: "flex",
            flexDirection: "column",
            gap: "0.45rem",
            fontFamily: "var(--font-mono)",
            fontSize: "0.62rem",
            letterSpacing: "0.06em",
            color: "var(--c-parchment)",
            lineHeight: 1.32,
            overflow: "hidden",
          }}
        >
          {columns.map((c, i) => (
            <div key={i} style={{ display: "flex", flexDirection: "column", gap: "1px" }}>
              <dt
                style={{
                  textTransform: "uppercase",
                  fontSize: "0.55rem",
                  letterSpacing: "0.16em",
                  color: "rgba(240,237,227,0.55)",
                }}
              >
                {c}
              </dt>
              <dd style={{ margin: 0, color: "var(--c-parchment)", fontWeight: 500 }}>
                {values[i] ?? "—"}
              </dd>
            </div>
          ))}
        </dl>
      </div>
    </motion.div>
  );
}

export function HexagonTrio({ subjects, columns, rows }: HexagonTrioProps) {
  const total = subjects.length;
  const positions = POSITIONS[total] ?? subjects.map((_, i) => {
    const x = ((i + 0.5) / total) * 100;
    return [x, 50] as [number, number];
  });
  const hoverRef = useRef<number | null>(null);
  const reduced = useReducedMotion();

  // Two-state hover proxy for re-rendering connection-line opacity
  const [, force] = useStateLikeRef();
  const setHover = (i: number | null) => {
    hoverRef.current = i;
    force();
  };

  // Mobile fallback: stack hexes vertically when viewport narrow. We use a
  // CSS media query inside a `style` tag so SSR matches client.
  const id = useId();

  return (
    <div
      className={`hex-trio hex-trio-${total}`}
      role="group"
      aria-label="Native comparison — three hexagonal cells"
      style={{
        position: "relative",
        width: "100%",
        maxWidth: 760,
        margin: "clamp(2rem, 4vw, 3.5rem) auto",
        aspectRatio: total === 2 ? "16/7" : total === 5 ? "5/4" : "5/4",
      }}
    >
      <style>{`
        @media (max-width: 720px) {
          .hex-trio.hex-trio-${id.replace(/[^a-z0-9]/gi, "")} { aspect-ratio: auto; }
        }
      `}</style>
      {/* Mutual-aspect connection lines — drawn behind the hexes */}
      <svg
        viewBox="0 0 100 100"
        preserveAspectRatio="none"
        width="100%"
        height="100%"
        style={{ position: "absolute", inset: 0, pointerEvents: "none" }}
        aria-hidden="true"
      >
        {positions.map((a, i) =>
          positions.slice(i + 1).map((b, j) => {
            const targetIdx = i + 1 + j;
            const isHot = hoverRef.current === i || hoverRef.current === targetIdx;
            return (
              <line
                key={`${i}-${targetIdx}`}
                x1={a[0]}
                y1={a[1]}
                x2={b[0]}
                y2={b[1]}
                stroke="var(--c-gold)"
                strokeWidth={isHot ? 0.35 : 0.18}
                strokeOpacity={isHot ? 0.7 : 0.3}
                style={{
                  transition: "stroke-opacity 0.35s ease, stroke-width 0.35s ease",
                  animation: isHot && !reduced ? "hex-pulse 1.8s ease-in-out infinite" : undefined,
                }}
              />
            );
          })
        )}
      </svg>
      <style>{`
        @keyframes hex-pulse {
          0%, 100% { stroke-opacity: 0.45; }
          50%      { stroke-opacity: 0.85; }
        }
        @media (max-width: 720px) {
          .hex-trio .hex-positioned {
            position: static !important;
            transform: none !important;
            margin: 0.5rem auto !important;
          }
          .hex-trio { aspect-ratio: auto !important; }
        }
      `}</style>
      {subjects.map((s, i) => {
        const [cx, cy] = positions[i] ?? [50, 50];
        return (
          <div
            key={i}
            className="hex-positioned"
            style={{
              position: "absolute",
              left: `${cx}%`,
              top: `${cy}%`,
              transform: "translate(-50%, -50%)",
            }}
          >
            <HexCell
              subject={s}
              columns={columns}
              values={rows[i] ?? []}
              index={i}
              total={total}
              onHover={setHover}
              hovered={hoverRef.current}
            />
          </div>
        );
      })}
    </div>
  );
}

// ─── tiny utility: a useState-shaped ref for forcing rerenders ─────────
// We avoid useState's full machinery for the hover proxy because we only
// need a re-render kick, not actual state storage (the source of truth is
// the ref). This keeps the hover hand-off allocation-free.
import { useReducer } from "react";
function useStateLikeRef(): [number, () => void] {
  const [v, kick] = useReducer((n: number) => n + 1, 0);
  return [v, kick];
}
