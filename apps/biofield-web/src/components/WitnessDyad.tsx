"use client";

/**
 * WitnessDyad — Wave 1, built 1:1 to docs/design/biofield-web/03-witness-dyad-spec.png
 *
 * The witness_layer output as a split dyad panel:
 *   LEFT  — ALETHEIOS (Sacred Gold heading)     → perspective paragraph
 *   RIGHT — PICHET     (Coherence Emerald heading) → perspective paragraph
 *   between them — a thin vertical sacred-geometry compass divider (SVG).
 *   BELOW — a centered SYNTHESIS block + an italic WITNESS QUESTION
 *           accented in Witness Violet, then a row of SF-Mono engine chips
 *           and a small LLM-powered badge.
 * A faint Sacred Gold constellation grid sits behind the whole panel.
 *
 * Motion: Anime.js v4 (named `animate`). All guarded by prefers-reduced-motion
 * with cleanup on unmount:
 *   1. Mount — the two columns + synthesis fade/rise in on a gentle stagger.
 *   2. Divider — the compass divider draws itself (strokeDashoffset reveal).
 *
 * States: empty / loading both render an "awaiting capture" placeholder so the
 * panel holds its shape before a reading exists.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";

const GOLD = "#C5A017"; // Sacred Gold — Aletheios
const EMERALD = "#10B5A7"; // Coherence Emerald — Pichet
const VIOLET = "#2D0050"; // Witness Violet — the question
const PARCHMENT = "#F0EDE3"; // body copy
const SILVER = "#8A9BA8"; // muted labels

export interface WitnessDyadProps {
  aletheios?: string;
  pichet?: string;
  synthesis?: string;
  witnessQuestion?: string;
  enginesUsed?: string[];
  llmPowered?: boolean;
  loading?: boolean;
}

/** Deterministic faint constellation (no SSR/client mismatch). */
function useConstellation() {
  return useMemo(() => {
    // Fixed pseudo-random points so the grid is stable across renders.
    const pts: Array<{ x: number; y: number; r: number; o: number }> = [];
    let seed = 1337;
    const rnd = () => {
      seed = (seed * 1664525 + 1013904223) % 4294967296;
      return seed / 4294967296;
    };
    for (let i = 0; i < 46; i += 1) {
      const big = i % 7 === 0;
      pts.push({
        x: rnd() * 100,
        y: rnd() * 100,
        r: big ? 1.4 : 0.7,
        o: big ? 0.34 : 0.16,
      });
    }
    return pts;
  }, []);
}

export function WitnessDyad({
  aletheios,
  pichet,
  synthesis,
  witnessQuestion,
  enginesUsed = [],
  llmPowered = false,
  loading = false,
}: WitnessDyadProps) {
  const rootRef = useRef<HTMLDivElement | null>(null);
  const dividerRef = useRef<SVGPathElement>(null);
  const constellation = useConstellation();

  const hasContent = Boolean(aletheios || pichet || synthesis);
  const isAwaiting = loading || !hasContent;

  // Mount: stagger the columns + synthesis in, and draw the divider.
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce || isAwaiting) return;

    const rises = rootRef.current?.querySelectorAll<HTMLElement>("[data-rise]");
    const anims: Array<{ pause?: () => void }> = [];

    if (rises && rises.length) {
      anims.push(
        animate(rises, {
          opacity: [0, 1],
          translateY: [14, 0],
          duration: 720,
          delay: (_el: Element | undefined, i: number) => 120 + i * 130,
          ease: "out(3)",
        }),
      );
    }

    const path = dividerRef.current;
    if (path) {
      const len = path.getTotalLength();
      path.style.strokeDasharray = `${len}`;
      path.style.strokeDashoffset = `${len}`;
      anims.push(
        animate(path, {
          strokeDashoffset: [len, 0],
          duration: 1100,
          ease: "inOut(3)",
          delay: 160,
        }),
      );
    }

    return () => {
      anims.forEach((a) => a.pause?.());
    };
  }, [isAwaiting]);

  return (
    <section
      ref={rootRef}
      aria-label="Witness Dyad — Aletheios and Pichet perspectives"
      style={{
        position: "relative",
        width: "100%",
        maxWidth: 920,
        margin: "0 auto",
        padding: "clamp(1.6rem, 4vw, 2.8rem)",
        background: "rgba(14, 20, 40, 0.55)",
        border: `1px solid rgba(197, 160, 23, 0.16)`,
        boxShadow:
          "inset 0 1px 0 rgba(240,237,227,0.04), 0 24px 72px rgba(0,0,0,0.5)",
        overflow: "hidden",
        // sharp edges — no rounded SaaS corners
        borderRadius: 0,
        fontFamily: "var(--font-body, system-ui, sans-serif)",
      }}
    >
      {/* faint Sacred Gold constellation grid behind everything */}
      <svg
        aria-hidden
        viewBox="0 0 100 100"
        preserveAspectRatio="none"
        style={{
          position: "absolute",
          inset: 0,
          width: "100%",
          height: "100%",
          pointerEvents: "none",
          opacity: 0.6,
        }}
      >
        {constellation.map((p, i) => (
          <circle key={i} cx={p.x} cy={p.y} r={p.r} fill={GOLD} opacity={p.o} />
        ))}
      </svg>

      {/* header */}
      <header
        style={{
          position: "relative",
          textAlign: "center",
          marginBottom: "clamp(1.4rem, 3vw, 2rem)",
        }}
      >
        <p
          style={{
            margin: 0,
            fontFamily: "var(--font-mono, monospace)",
            fontSize: "0.66rem",
            letterSpacing: "0.22em",
            textTransform: "uppercase",
            color: SILVER,
          }}
        >
          Witness Layer
        </p>
      </header>

      {isAwaiting ? (
        <AwaitingState loading={loading} />
      ) : (
        <>
          {/* split: Aletheios | compass divider | Pichet */}
          <div
            style={{
              position: "relative",
              display: "grid",
              gridTemplateColumns: "1fr auto 1fr",
              gap: "clamp(1.2rem, 3vw, 2.4rem)",
              alignItems: "stretch",
            }}
          >
            <Perspective
              name="ALETHEIOS"
              accent={GOLD}
              text={aletheios}
              align="left"
            />

            <CompassDivider dividerRef={dividerRef} />

            <Perspective
              name="PICHET"
              accent={EMERALD}
              text={pichet}
              align="left"
            />
          </div>

          {/* synthesis + witness question */}
          {(synthesis || witnessQuestion) && (
            <div
              data-rise
              style={{
                position: "relative",
                marginTop: "clamp(1.8rem, 4vw, 2.8rem)",
                paddingTop: "clamp(1.6rem, 3vw, 2.2rem)",
                borderTop: `1px solid rgba(197, 160, 23, 0.18)`,
                textAlign: "center",
              }}
            >
              {synthesis && (
                <>
                  <p
                    style={{
                      margin: 0,
                      fontFamily: "var(--font-display, sans-serif)",
                      fontSize: "0.72rem",
                      letterSpacing: "0.26em",
                      textTransform: "uppercase",
                      color: SILVER,
                      marginBottom: "0.85rem",
                    }}
                  >
                    Synthesis
                  </p>
                  <p
                    style={{
                      margin: "0 auto",
                      maxWidth: 640,
                      fontSize: "clamp(0.95rem, 1.6vw, 1.05rem)",
                      lineHeight: 1.72,
                      color: PARCHMENT,
                    }}
                  >
                    {synthesis}
                  </p>
                </>
              )}

              {witnessQuestion && (
                <p
                  style={{
                    margin: "1.4rem auto 0",
                    maxWidth: 600,
                    fontStyle: "italic",
                    fontSize: "clamp(1rem, 1.8vw, 1.15rem)",
                    lineHeight: 1.6,
                    color: PARCHMENT,
                    paddingTop: "1.2rem",
                    borderTop: `1px solid rgba(45, 0, 80, 0.55)`,
                    textShadow: `0 0 18px rgba(45, 0, 80, 0.9)`,
                  }}
                >
                  <span
                    aria-hidden
                    style={{
                      display: "block",
                      fontFamily: "var(--font-mono, monospace)",
                      fontStyle: "normal",
                      fontSize: "0.6rem",
                      letterSpacing: "0.24em",
                      textTransform: "uppercase",
                      color: VIOLET,
                      filter: "brightness(2.1)",
                      marginBottom: "0.6rem",
                    }}
                  >
                    Witness Question
                  </span>
                  &ldquo;{witnessQuestion}&rdquo;
                </p>
              )}
            </div>
          )}

          {/* engine chips + LLM badge */}
          {(enginesUsed.length > 0 || llmPowered) && (
            <div
              data-rise
              style={{
                position: "relative",
                marginTop: "clamp(1.6rem, 3vw, 2.2rem)",
                display: "flex",
                flexWrap: "wrap",
                alignItems: "center",
                justifyContent: "center",
                gap: "0.5rem",
              }}
            >
              {enginesUsed.map((engine) => (
                <span
                  key={engine}
                  style={{
                    fontFamily: "var(--font-mono, monospace)",
                    fontSize: "0.64rem",
                    letterSpacing: "0.06em",
                    color: SILVER,
                    padding: "0.26rem 0.6rem",
                    border: `1px solid rgba(138, 155, 168, 0.28)`,
                    background: "rgba(138, 155, 168, 0.05)",
                    borderRadius: 0,
                  }}
                >
                  {engine}
                </span>
              ))}

              {llmPowered && (
                <span
                  style={{
                    display: "inline-flex",
                    alignItems: "center",
                    gap: "0.36rem",
                    fontFamily: "var(--font-mono, monospace)",
                    fontSize: "0.62rem",
                    letterSpacing: "0.1em",
                    textTransform: "uppercase",
                    color: GOLD,
                    padding: "0.26rem 0.62rem",
                    border: `1px solid rgba(197, 160, 23, 0.4)`,
                    background: "rgba(197, 160, 23, 0.08)",
                    borderRadius: 0,
                  }}
                >
                  <span
                    aria-hidden
                    style={{
                      width: 5,
                      height: 5,
                      borderRadius: "50%",
                      background: GOLD,
                      boxShadow: `0 0 8px ${GOLD}`,
                    }}
                  />
                  LLM-powered
                </span>
              )}
            </div>
          )}
        </>
      )}
    </section>
  );
}

/** One witness column: name heading in its accent + perspective paragraph. */
function Perspective({
  name,
  accent,
  text,
  align,
}: {
  name: string;
  accent: string;
  text?: string;
  align: "left" | "center";
}) {
  return (
    <div data-rise style={{ minWidth: 0, textAlign: align }}>
      <h3
        style={{
          margin: 0,
          fontFamily: "var(--font-display, sans-serif)",
          fontSize: "clamp(1.15rem, 2.4vw, 1.5rem)",
          fontWeight: 700,
          letterSpacing: "0.14em",
          textTransform: "uppercase",
          color: accent,
          textShadow: `0 0 22px ${accent}55`,
          marginBottom: "0.9rem",
        }}
      >
        {name}
      </h3>
      <p
        style={{
          margin: 0,
          fontSize: "clamp(0.9rem, 1.5vw, 0.98rem)",
          lineHeight: 1.74,
          color: PARCHMENT,
        }}
      >
        {text}
      </p>
    </div>
  );
}

/** Thin vertical sacred-geometry compass divider — draws itself on mount. */
function CompassDivider({
  dividerRef,
}: {
  dividerRef: React.RefObject<SVGPathElement>;
}) {
  // viewBox 24 wide so the compass node has room; the panel stretches its height.
  const W = 24;
  const H = 320;
  const cx = W / 2;
  const cy = H / 2;
  const ring = 9.5;

  // Eight compass spokes radiating from the mid node.
  const spokes = Array.from({ length: 8 }, (_, i) => {
    const a = (Math.PI / 4) * i;
    const inner = i % 2 === 0 ? 0 : ring * 0.45;
    const outer = i % 2 === 0 ? ring * 1.55 : ring * 1.05;
    return {
      x1: cx + Math.cos(a) * inner,
      y1: cy + Math.sin(a) * inner,
      x2: cx + Math.cos(a) * outer,
      y2: cy + Math.sin(a) * outer,
    };
  });

  return (
    <svg
      aria-hidden
      viewBox={`0 0 ${W} ${H}`}
      preserveAspectRatio="none"
      style={{ width: W, height: "100%", overflow: "visible" }}
    >
      {/* the line itself draws in (single path top→node, node→bottom) */}
      <path
        ref={dividerRef}
        d={`M ${cx} 0 L ${cx} ${cy - ring} M ${cx} ${cy + ring} L ${cx} ${H}`}
        stroke={GOLD}
        strokeWidth={0.8}
        fill="none"
        opacity={0.42}
      />

      {/* compass node: outer ring + radiating spokes + emerald core */}
      <circle
        cx={cx}
        cy={cy}
        r={ring}
        fill="none"
        stroke={GOLD}
        strokeWidth={0.8}
        opacity={0.7}
      />
      {spokes.map((s, i) => (
        <line
          key={i}
          x1={s.x1}
          y1={s.y1}
          x2={s.x2}
          y2={s.y2}
          stroke={i % 2 === 0 ? GOLD : SILVER}
          strokeWidth={0.6}
          opacity={i % 2 === 0 ? 0.65 : 0.32}
        />
      ))}
      <circle cx={cx} cy={cy} r={ring + 4} fill="none" stroke={GOLD} strokeWidth={0.4} opacity={0.22} />
      <circle cx={cx} cy={cy} r={1.8} fill={EMERALD} opacity={0.95}>
        <animate
          attributeName="opacity"
          values="0.95;0.5;0.95"
          dur="4.8s"
          repeatCount="indefinite"
        />
      </circle>
    </svg>
  );
}

/** Empty / loading placeholder — "awaiting capture". */
function AwaitingState({ loading }: { loading: boolean }) {
  return (
    <div
      style={{
        position: "relative",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "1.1rem",
        padding: "clamp(2.4rem, 6vw, 4rem) 1rem",
        textAlign: "center",
      }}
    >
      {/* small breathing compass to echo the divider node */}
      <svg
        aria-hidden
        viewBox="0 0 60 60"
        width={60}
        height={60}
        style={{ opacity: 0.7 }}
      >
        <circle cx={30} cy={30} r={20} fill="none" stroke={GOLD} strokeWidth={0.8} opacity={0.4} />
        <circle cx={30} cy={30} r={13} fill="none" stroke={GOLD} strokeWidth={0.6} opacity={0.25} />
        {Array.from({ length: 8 }, (_, i) => {
          const a = (Math.PI / 4) * i;
          return (
            <line
              key={i}
              x1={30 + Math.cos(a) * 5}
              y1={30 + Math.sin(a) * 5}
              x2={30 + Math.cos(a) * 24}
              y2={30 + Math.sin(a) * 24}
              stroke={i % 2 === 0 ? GOLD : SILVER}
              strokeWidth={0.5}
              opacity={i % 2 === 0 ? 0.5 : 0.25}
            />
          );
        })}
        <circle cx={30} cy={30} r={3} fill={EMERALD} opacity={0.9}>
          <animate
            attributeName="r"
            values="3;4.5;3"
            dur="4.8s"
            repeatCount="indefinite"
          />
          <animate
            attributeName="opacity"
            values="0.9;0.45;0.9"
            dur="4.8s"
            repeatCount="indefinite"
          />
        </circle>
      </svg>

      <p
        style={{
          margin: 0,
          fontFamily: "var(--font-mono, monospace)",
          fontSize: "0.72rem",
          letterSpacing: "0.2em",
          textTransform: "uppercase",
          color: SILVER,
        }}
      >
        {loading ? "Witnessing the field…" : "Awaiting capture"}
      </p>
      <p
        style={{
          margin: 0,
          maxWidth: 360,
          fontSize: "0.86rem",
          lineHeight: 1.7,
          color: "rgba(240,237,227,0.5)",
        }}
      >
        {loading
          ? "Aletheios and Pichet are reading the biofield."
          : "Aletheios and Pichet will speak once a capture is taken."}
      </p>
    </div>
  );
}

export default WitnessDyad;
