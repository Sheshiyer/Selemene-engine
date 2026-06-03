"use client";

/**
 * ConsciousnessSpectrum — Wave 1, built 1:1 to
 * docs/design/biofield-web/08-consciousness-spectrum-spec.png
 *
 * A horizontal arc spectrum of FIVE consciousness states, each rendered as a
 * distinct sacred-geometry glyph along a rising-then-falling curve (node 03
 * Flow is the apex). Order follows Goethe's Zur Farbenlehre / Kha-Ba-La:
 *
 *   01 Void Black   #070B1D  La        Source      — nested squares (inertia)
 *   02 Witness      #2D0050  Kha       Observer     — vesica piscis (two circles)
 *   03 Flow Indigo  #0B50FB  Kha→Ba    Flow         — nested rotated diamonds
 *   04 Coherence    #10B5A7  Ba↔La     Coherence    — Metatron hexagram lattice
 *   05 Sacred Gold  #C5A017  Ba        Activation   — merkaba star tetrahedron
 *
 * The CURRENT level glows bioluminescent; the others stay dim outlines.
 * Behind sits a hairline Sacred Gold constellation grid. An SF-Mono readout
 * shows the level + Kha-Ba-La mapping in Parchment.
 *
 * Motion: Anime.js v4 (named `animate`), mirroring CosmogramRing. Three
 * behaviours, all guarded by prefers-reduced-motion + cleaned up on unmount:
 *   1. Mount  — the spectrum connector path draws itself (strokeDashoffset).
 *   2. Pulse  — the active node's glow halo breathes gently (loop).
 *   3. Change — when `level` changes, the new active glow tweens up while the
 *               old one settles back to dim.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";

const VOID = "#070B1D";
const VIOLET = "#2D0050";
const INDIGO = "#0B50FB";
const EMERALD = "#10B5A7";
const GOLD = "#C5A017";
const PARCHMENT = "#F0EDE3";
const SILVER = "#8A9BA8";

type GlyphKind = "squares" | "vesica" | "diamond" | "metatron" | "merkaba";

interface SpectrumState {
  index: number; // 0..4
  ordinal: string; // "01".."05"
  glyph: GlyphKind;
  color: string; // bioluminescent color when active
  name: string; // SOURCE / OBSERVER / ...
  swatch: string; // human-readable color name
  khaBaLa: string; // mapping line
}

// Order is canonical: Void -> Witness Violet -> Flow Indigo -> Coherence -> Gold
const STATES: SpectrumState[] = [
  {
    index: 0,
    ordinal: "01",
    glyph: "squares",
    color: SILVER, // Void is near-black; render its outline in silver so it reads
    name: "SOURCE",
    swatch: "VOID BLACK",
    khaBaLa: "La / Source",
  },
  {
    index: 1,
    ordinal: "02",
    glyph: "vesica",
    color: "#7A4FB5", // a luminous lift of Witness Violet (2D0050 is too dark to glow)
    name: "OBSERVER",
    swatch: "WITNESS VIOLET",
    khaBaLa: "Kha / Observer",
  },
  {
    index: 2,
    ordinal: "03",
    glyph: "diamond",
    color: INDIGO,
    name: "FLOW",
    swatch: "FLOW INDIGO",
    khaBaLa: "Kha -> Ba / Flow",
  },
  {
    index: 3,
    ordinal: "04",
    glyph: "metatron",
    color: EMERALD,
    name: "COHERENCE",
    swatch: "COHERENCE EMERALD",
    khaBaLa: "Ba <-> La / Coherence",
  },
  {
    index: 4,
    ordinal: "05",
    glyph: "merkaba",
    color: GOLD,
    name: "ACTIVATION",
    swatch: "SACRED GOLD",
    khaBaLa: "Ba / Activation",
  },
];

const VB_W = 760;
const VB_H = 280;
const NODE_R = 26; // glyph radius
const ARC_TOP = 96; // y of the apex (node 03)
const ARC_BOTTOM = 150; // y of the outer nodes

/** Node centers along a shallow parabola, apex at the middle node. */
function nodeCenters(): Array<{ x: number; y: number }> {
  const left = 90;
  const right = VB_W - 90;
  const span = right - left;
  return STATES.map((s) => {
    const t = s.index / (STATES.length - 1); // 0..1
    const x = left + t * span;
    // parabola: 0 at ends, 1 at center
    const lift = 1 - Math.pow((t - 0.5) * 2, 2);
    const y = ARC_BOTTOM - lift * (ARC_BOTTOM - ARC_TOP);
    return { x, y };
  });
}

/** Smooth connector path through the node centers (Catmull-Rom -> cubic). */
function connectorPath(pts: Array<{ x: number; y: number }>): string {
  if (pts.length < 2) return "";
  let d = `M ${pts[0].x.toFixed(2)} ${pts[0].y.toFixed(2)}`;
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[i - 1] ?? pts[i];
    const p1 = pts[i];
    const p2 = pts[i + 1];
    const p3 = pts[i + 2] ?? p2;
    const c1x = p1.x + (p2.x - p0.x) / 6;
    const c1y = p1.y + (p2.y - p0.y) / 6;
    const c2x = p2.x - (p3.x - p1.x) / 6;
    const c2y = p2.y - (p3.y - p1.y) / 6;
    d += ` C ${c1x.toFixed(2)} ${c1y.toFixed(2)} ${c2x.toFixed(2)} ${c2y.toFixed(2)} ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`;
  }
  return d;
}

/** A deterministic scatter of background "stars" for the constellation grid. */
function constellation(): Array<{ x: number; y: number; r: number }> {
  // simple LCG so the layout is stable across renders (no hydration drift)
  let seed = 0x9e3779b1;
  const rnd = () => {
    seed = (seed * 1664525 + 1013904223) >>> 0;
    return seed / 0xffffffff;
  };
  return Array.from({ length: 54 }, () => ({
    x: rnd() * VB_W,
    y: rnd() * VB_H,
    r: rnd() > 0.82 ? 1.2 : 0.6,
  }));
}

/** SVG children for a single sacred-geometry glyph, drawn centered at (0,0). */
function Glyph({ kind, r, stroke }: { kind: GlyphKind; r: number; stroke: string }) {
  const sw = 1.4;
  const common = {
    fill: "none",
    stroke,
    strokeWidth: sw,
    strokeLinejoin: "round" as const,
    strokeLinecap: "round" as const,
    vectorEffect: "non-scaling-stroke" as const,
  };

  switch (kind) {
    case "squares": {
      // nested squares — earth / inertia / source
      const o = r * 0.82;
      const inner = r * 0.42;
      return (
        <g>
          <rect x={-o} y={-o} width={o * 2} height={o * 2} {...common} />
          <rect x={-inner} y={-inner} width={inner * 2} height={inner * 2} {...common} opacity={0.85} />
        </g>
      );
    }
    case "vesica": {
      // vesica piscis — two overlapping circles (the observer's gaze)
      const cr = r * 0.62;
      const dx = cr * 0.62;
      return (
        <g>
          <circle cx={-dx} cy={0} r={cr} {...common} />
          <circle cx={dx} cy={0} r={cr} {...common} />
        </g>
      );
    }
    case "diamond": {
      // nested rotated squares + cross — the threshold into flow
      const o = r * 0.9;
      const inner = r * 0.46;
      return (
        <g>
          <path d={`M 0 ${-o} L ${o} 0 L 0 ${o} L ${-o} 0 Z`} {...common} />
          <path d={`M 0 ${-inner} L ${inner} 0 L 0 ${inner} L ${-inner} 0 Z`} {...common} opacity={0.85} />
          <line x1={0} y1={-o} x2={0} y2={o} {...common} opacity={0.4} />
          <line x1={-o} y1={0} x2={o} y2={0} {...common} opacity={0.4} />
        </g>
      );
    }
    case "metatron": {
      // hexagon + inscribed star lattice — Metatron's cube (coherence)
      const verts = Array.from({ length: 6 }, (_, i) => {
        const a = (Math.PI / 3) * i - Math.PI / 2;
        return { x: Math.cos(a) * r, y: Math.sin(a) * r };
      });
      const hex = verts.map((v, i) => `${i === 0 ? "M" : "L"} ${v.x.toFixed(2)} ${v.y.toFixed(2)}`).join(" ") + " Z";
      // connect every vertex to every other -> the lattice
      const lattice: string[] = [];
      for (let i = 0; i < verts.length; i++) {
        for (let j = i + 1; j < verts.length; j++) {
          lattice.push(`M ${verts[i].x.toFixed(2)} ${verts[i].y.toFixed(2)} L ${verts[j].x.toFixed(2)} ${verts[j].y.toFixed(2)}`);
        }
      }
      return (
        <g>
          <path d={hex} {...common} />
          <path d={lattice.join(" ")} {...common} strokeWidth={sw * 0.75} opacity={0.55} />
          <circle cx={0} cy={0} r={r * 0.16} {...common} opacity={0.9} />
        </g>
      );
    }
    case "merkaba": {
      // two interlocked triangles — star tetrahedron / activation
      const up = `M 0 ${-r} L ${r * 0.87} ${r * 0.5} L ${-r * 0.87} ${r * 0.5} Z`;
      const down = `M 0 ${r} L ${r * 0.87} ${-r * 0.5} L ${-r * 0.87} ${-r * 0.5} Z`;
      return (
        <g>
          <path d={up} {...common} />
          <path d={down} {...common} />
        </g>
      );
    }
  }
}

const clampLevel = (level: number): number => {
  // Accept a 0..4 index directly, OR map a 0..5 consciousness_level onto 0..4.
  // Heuristic: values > 4 are treated as the 0..5 scale (5 -> index 4).
  const v = level > 4 ? (level / 5) * 4 : level;
  return Math.max(0, Math.min(STATES.length - 1, Math.round(v)));
};

export interface ConsciousnessSpectrumProps {
  /** 0..4 index, or a 0..5 consciousness_level which is mapped onto 0..4. */
  level: number;
}

export function ConsciousnessSpectrum({ level }: ConsciousnessSpectrumProps) {
  const active = clampLevel(level);
  const activeState = STATES[active];

  const rootRef = useRef<SVGSVGElement | null>(null);
  const pathRef = useRef<SVGPathElement | null>(null);
  const haloRefs = useRef<Array<SVGCircleElement | null>>([]);

  const centers = useMemo(nodeCenters, []);
  const path = useMemo(() => connectorPath(centers), [centers]);
  const stars = useMemo(constellation, []);

  const prefersReduce = () =>
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

  // Mount: draw the connector spectrum path in.
  useEffect(() => {
    if (prefersReduce()) return;
    const p = pathRef.current;
    if (!p) return;
    const len = p.getTotalLength();
    p.style.strokeDasharray = `${len}`;
    p.style.strokeDashoffset = `${len}`;
    const anim = animate(p, {
      strokeDashoffset: [len, 0],
      duration: 1600,
      ease: "outQuart",
      delay: 120,
    });
    return () => {
      anim.pause();
    };
  }, []);

  // Active node: pulse the glow halo; settle the inactive ones. Re-runs on level.
  useEffect(() => {
    const reduce = prefersReduce();
    const anims: Array<{ pause: () => void }> = [];

    haloRefs.current.forEach((halo, i) => {
      if (!halo) return;
      if (i === active) {
        if (reduce) {
          halo.style.opacity = "0.55";
          halo.style.transform = "scale(1)";
          return;
        }
        // tween into a visible glow, then breathe
        anims.push(
          animate(halo, {
            opacity: [{ to: 0.6, duration: 600, ease: "outQuad" }],
          }),
        );
        anims.push(
          animate(halo, {
            scale: [
              { to: 1.18, duration: 1900, ease: "inOutSine" },
              { to: 1.0, duration: 1900, ease: "inOutSine" },
            ],
            opacity: [
              { to: 0.6, duration: 1900, ease: "inOutSine" },
              { to: 0.32, duration: 1900, ease: "inOutSine" },
            ],
            loop: true,
            delay: 250,
          }),
        );
      } else if (reduce) {
        halo.style.opacity = "0";
      } else {
        anims.push(
          animate(halo, {
            opacity: [{ to: 0, duration: 500, ease: "outQuad" }],
            scale: [{ to: 1, duration: 500 }],
          }),
        );
      }
    });

    return () => anims.forEach((a) => a.pause());
  }, [active]);

  return (
    <figure
      style={{
        margin: 0,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "0.9rem",
        width: "100%",
        maxWidth: 760,
      }}
    >
      <svg
        ref={rootRef}
        viewBox={`0 0 ${VB_W} ${VB_H}`}
        width="100%"
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label={`Consciousness spectrum, current level ${activeState.ordinal} ${activeState.name}`}
      >
        <defs>
          {STATES.map((s) => (
            <radialGradient key={s.ordinal} id={`cs-halo-${s.ordinal}`} cx="50%" cy="50%" r="50%">
              <stop offset="0%" stopColor={s.color} stopOpacity="0.85" />
              <stop offset="55%" stopColor={s.color} stopOpacity="0.22" />
              <stop offset="100%" stopColor={s.color} stopOpacity="0" />
            </radialGradient>
          ))}
          <filter id="cs-glow" x="-80%" y="-80%" width="260%" height="260%">
            <feGaussianBlur stdDeviation="2.4" result="b" />
            <feMerge>
              <feMergeNode in="b" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* hairline Sacred Gold constellation grid */}
        <g opacity={0.5}>
          {stars.map((p, i) => (
            <circle key={i} cx={p.x} cy={p.y} r={p.r} fill={GOLD} opacity={p.r > 1 ? 0.32 : 0.14} />
          ))}
          {/* faint connecting filaments between a few stars */}
          {stars.slice(0, 10).map((p, i) => {
            const q = stars[(i * 7 + 3) % stars.length];
            return (
              <line
                key={`f-${i}`}
                x1={p.x}
                y1={p.y}
                x2={q.x}
                y2={q.y}
                stroke={GOLD}
                strokeWidth={0.4}
                opacity={0.06}
              />
            );
          })}
        </g>

        {/* spectrum connector path (draws itself on mount) */}
        <path
          ref={pathRef}
          d={path}
          fill="none"
          stroke={SILVER}
          strokeWidth={1.2}
          strokeLinecap="round"
          opacity={0.4}
        />

        {/* nodes */}
        {STATES.map((s, i) => {
          const c = centers[i];
          const isActive = i === active;
          return (
            <g key={s.ordinal} transform={`translate(${c.x} ${c.y})`}>
              {/* bioluminescent glow halo (animated opacity/scale) */}
              <circle
                ref={(el) => {
                  haloRefs.current[i] = el;
                }}
                cx={0}
                cy={0}
                r={NODE_R * 1.7}
                fill={`url(#cs-halo-${s.ordinal})`}
                opacity={0}
                style={{ transformOrigin: "center", transformBox: "fill-box" }}
              />
              {/* the glyph — active glows bright + filtered, others dim */}
              <g
                opacity={isActive ? 1 : 0.34}
                filter={isActive ? "url(#cs-glow)" : undefined}
              >
                <Glyph kind={s.glyph} r={NODE_R} stroke={isActive ? s.color : SILVER} />
              </g>
              {/* ordinal under each node */}
              <text
                x={0}
                y={NODE_R + 26}
                textAnchor="middle"
                fontFamily="var(--font-mono, monospace)"
                fontSize={12}
                letterSpacing="1.5"
                fill={isActive ? s.color : SILVER}
                opacity={isActive ? 0.95 : 0.55}
              >
                {s.ordinal}
              </text>
              {/* name under the ordinal */}
              <text
                x={0}
                y={NODE_R + 44}
                textAnchor="middle"
                fontFamily="var(--font-mono, monospace)"
                fontSize={9}
                letterSpacing="1.5"
                fill={PARCHMENT}
                opacity={isActive ? 0.85 : 0.4}
              >
                {s.name}
              </text>
            </g>
          );
        })}
      </svg>

      {/* SF-Mono level readout + Kha-Ba-La mapping */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "0.3rem",
          fontFamily: "var(--font-mono, monospace)",
          textAlign: "center",
        }}
      >
        <div
          style={{
            fontSize: "0.95rem",
            letterSpacing: "0.18em",
            color: PARCHMENT,
          }}
        >
          <span style={{ color: activeState.color }}>LEVEL {activeState.ordinal}</span>
          <span style={{ opacity: 0.5 }}> · </span>
          <span>{activeState.name}</span>
        </div>
        <div
          style={{
            fontSize: "0.72rem",
            letterSpacing: "0.14em",
            color: SILVER,
            opacity: 0.8,
          }}
        >
          KHA-BA-LA · {activeState.khaBaLa.toUpperCase()}
        </div>
        <div
          style={{
            fontSize: "0.64rem",
            letterSpacing: "0.12em",
            color: SILVER,
            opacity: 0.5,
          }}
        >
          {activeState.swatch}
        </div>
      </div>
    </figure>
  );
}

export default ConsciousnessSpectrum;
