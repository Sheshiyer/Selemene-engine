"use client";

/**
 * CaptureCompass — Wave 1, built 1:1 to docs/design/biofield-web/09-capture-compass-spec.png
 *
 * A circular capture control. Anatomy (top → bottom, matching the spec):
 *   1. ANALYSIS MODE selector — a four-point compass (curved-side diamond),
 *      one point active (Flow Indigo blade). Labels at the compass points.
 *   2. CENTRAL CAPTURE NODE — a sacred-geometry mandala (12 radial spokes,
 *      hexagram with vertex dots, inner hexagon) ringed by:
 *        · the PROGRESS RING — a Ba-Arc gauge that fills emerald → gold with
 *          `progress`, starting at top and sweeping clockwise.
 *        · the BREATH CADENCE ring — an emerald ring that expands/contracts on
 *          the 4:7:8 inhale/hold/exhale cadence.
 *      A bioluminescent emerald core breathes at the centre.
 *   3. PAUSE / RESUME — two minimal compass glyphs flanking the node (NOT pill
 *      buttons): a ringed four-petal compass with crosshair ticks, holding the
 *      pause bars (‖) or play triangle (▸). Rendered as real <button>s with
 *      aria-labels for keyboard access, styled as glyphs.
 *   4. CAPTURE STATE STEPPER — a horizontal hairline path:
 *        requested → uploaded → analyzed → persisted
 *      with rejected / reprocessed branches. The active node glows Coherence
 *      Emerald; rejected is a Terracotta notch.
 *
 * Motion: Anime.js v4 (named `animate`). Behaviours, all guarded by
 * prefers-reduced-motion + cleaned up on unmount / dep change:
 *   1. Breath  — the core + breath ring pulse on the 4:7:8 ratio.
 *   2. Progress — the Ba-Arc fill tweens to `progress`.
 *   3. Success — `state === "persisted"` fires an emerald core pulse.
 *   4. Rejected — `state === "rejected"` flashes the Terracotta notch.
 */

import { useEffect, useMemo, useRef } from "react";
import { animate } from "animejs";
import {
  BIOFIELD_CAPTURE_STATES,
  type BiofieldCaptureState,
} from "@/lib/selemene/biofield-domain";

const GOLD = "#C5A017";
const EMERALD = "#10B5A7";
const INDIGO = "#0B50FB";
const VIOLET = "#2D0050";
const PARCHMENT = "#F0EDE3";
const SILVER = "#8A9BA8";
const TERRACOTTA = "#C65D3B"; // reject-only

const clamp01 = (v: number) => Math.max(0, Math.min(1, v));
const rad = (deg: number) => ((deg - 90) * Math.PI) / 180; // 0deg = top

function polar(cx: number, cy: number, r: number, deg: number) {
  const a = rad(deg);
  return { x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
}

/** Arc path from startDeg sweeping `spanDeg` clockwise at radius r. */
function arc(cx: number, cy: number, r: number, startDeg: number, spanDeg: number) {
  const s = polar(cx, cy, r, startDeg);
  const e = polar(cx, cy, r, startDeg + spanDeg);
  const large = spanDeg > 180 ? 1 : 0;
  return `M ${s.x.toFixed(2)} ${s.y.toFixed(2)} A ${r} ${r} 0 ${large} 1 ${e.x.toFixed(2)} ${e.y.toFixed(2)}`;
}

/** Closed regular polygon path (n points) at radius r, first point at startDeg. */
function polygon(cx: number, cy: number, r: number, n: number, startDeg: number) {
  const pts = Array.from({ length: n }, (_, i) => polar(cx, cy, r, startDeg + (360 / n) * i));
  return (
    pts.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x.toFixed(2)} ${p.y.toFixed(2)}`).join(" ") + " Z"
  );
}

/** Four-point "compass star" — a diamond whose sides bow inward via quadratic
 *  curves through control points pulled toward centre. Used by the mode
 *  selector and the pause/resume glyphs. */
function compassStar(cx: number, cy: number, r: number, concavity = 0.45) {
  const tips = [0, 90, 180, 270].map((d) => polar(cx, cy, r, d));
  const ctrlR = r * concavity;
  const ctrls = [45, 135, 225, 315].map((d) => polar(cx, cy, ctrlR, d));
  return (
    `M ${tips[0].x.toFixed(2)} ${tips[0].y.toFixed(2)} ` +
    `Q ${ctrls[0].x.toFixed(2)} ${ctrls[0].y.toFixed(2)} ${tips[1].x.toFixed(2)} ${tips[1].y.toFixed(2)} ` +
    `Q ${ctrls[1].x.toFixed(2)} ${ctrls[1].y.toFixed(2)} ${tips[2].x.toFixed(2)} ${tips[2].y.toFixed(2)} ` +
    `Q ${ctrls[2].x.toFixed(2)} ${ctrls[2].y.toFixed(2)} ${tips[3].x.toFixed(2)} ${tips[3].y.toFixed(2)} ` +
    `Q ${ctrls[3].x.toFixed(2)} ${ctrls[3].y.toFixed(2)} ${tips[0].x.toFixed(2)} ${tips[0].y.toFixed(2)} Z`
  );
}

export type CaptureCompassState =
  | "idle"
  | "requested"
  | "uploaded"
  | "analyzed"
  | "persisted"
  | "rejected";

export interface CaptureCompassProps {
  state: CaptureCompassState;
  /** Progress ring fill, 0..1. */
  progress?: number;
  /** Active analysis mode label (matches one of the compass points). */
  mode?: string;
  onCapture?: () => void;
  onPause?: () => void;
  onResume?: () => void;
  paused?: boolean;
}

// Analysis-mode compass points — top / right / bottom / left.
const MODE_POINTS = [
  { label: "STRUCTURE", deg: 0 },
  { label: "FLOW", deg: 90 },
  { label: "SYMMETRY", deg: 180 },
  { label: "COHERENCE", deg: 270 },
] as const;

// Stepper main path (rejected + reprocessed are branches, handled separately).
const STEPPER_MAIN: BiofieldCaptureState[] = ["requested", "uploaded", "analyzed", "persisted"];

const STEP_ORDER: Record<string, number> = {
  requested: 0,
  uploaded: 1,
  analyzed: 2,
  persisted: 3,
};

export function CaptureCompass({
  state,
  progress = 0,
  mode = "FLOW",
  onCapture,
  onPause,
  onResume,
  paused = false,
}: CaptureCompassProps) {
  const VB = 360;
  const C = VB / 2;
  const NODE_R = 96; // capture node radius (outer guide ring)
  const PROGRESS_R = 84; // Ba-Arc progress ring radius
  const BREATH_R = 58; // breath cadence ring radius
  const GUIDES = [96, 72]; // faint gold guide rings

  const capturing = state === "requested" || state === "uploaded" || state === "analyzed";
  const isPersisted = state === "persisted";
  const isRejected = state === "rejected";

  const coreColor = isRejected ? TERRACOTTA : isPersisted ? EMERALD : capturing ? EMERALD : SILVER;
  const stateWord = isRejected
    ? "REJECTED"
    : isPersisted
      ? "PERSISTED"
      : capturing
        ? "CAPTURING"
        : "IDLE";

  const svgRef = useRef<SVGSVGElement | null>(null);
  const coreRef = useRef<SVGCircleElement | null>(null);
  const breathRef = useRef<SVGCircleElement | null>(null);
  const progressRef = useRef<SVGPathElement | null>(null);
  const notchRef = useRef<SVGCircleElement | null>(null);

  // Static geometry (independent of live values).
  const guideDefs = useMemo(
    () => GUIDES.map((r) => ({ r, d: arc(C, C, r, 0, 359.9) })),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  const spokes = useMemo(
    () =>
      Array.from({ length: 12 }, (_, i) => {
        const inner = polar(C, C, 30, (360 / 12) * i);
        const outer = polar(C, C, PROGRESS_R - 6, (360 / 12) * i);
        return { inner, outer };
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  const hexA = useMemo(() => polygon(C, C, 44, 3, 0), []); // upward triangle
  const hexB = useMemo(() => polygon(C, C, 44, 3, 180), []); // downward triangle
  const hexInner = useMemo(() => polygon(C, C, 30, 6, 0), []); // inner hexagon
  const hexDots = useMemo(
    () => [0, 60, 120, 180, 240, 300].map((d) => polar(C, C, 44, d)),
    [],
  );

  const progressTrack = useMemo(() => arc(C, C, PROGRESS_R, 0, 359.9), []);

  // Mount: breath cadence on the core + breath ring (4:7:8). Runs once.
  useEffect(() => {
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;

    const targets = [coreRef.current, breathRef.current].filter(
      (el): el is SVGCircleElement => el !== null,
    );
    if (targets.length === 0) return;

    // 4:7:8 — inhale 4s (expand), hold 7s, exhale 8s (contract).
    const breath = animate(targets, {
      scale: [
        { to: 1.14, duration: 4000, ease: "inOutSine" },
        { to: 1.14, duration: 7000 },
        { to: 1.0, duration: 8000, ease: "inOutSine" },
      ],
      opacity: [
        { to: 1, duration: 4000 },
        { to: 1, duration: 7000 },
        { to: 0.6, duration: 8000 },
      ],
      loop: true,
    });

    return () => {
      breath.pause();
    };
  }, []);

  // Progress: fill the Ba-Arc to `progress` (clamped). Re-runs on change.
  useEffect(() => {
    const path = progressRef.current;
    if (!path) return;
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

    const len = path.getTotalLength();
    const value = clamp01(progress);
    const hidden = len * (1 - value);
    path.style.strokeDasharray = `${len}`;

    if (reduce) {
      path.style.strokeDashoffset = `${hidden}`;
      return;
    }
    const anim = animate(path, {
      strokeDashoffset: [len, hidden],
      duration: 1100,
      ease: "outExpo",
    });
    return () => {
      anim.pause();
    };
  }, [progress]);

  // Success: emerald core pulse when persisted.
  useEffect(() => {
    if (!isPersisted) return;
    const core = coreRef.current;
    if (!core) return;
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;

    const pulse = animate(core, {
      scale: [{ to: 1.5, duration: 360, ease: "outQuad" }, { to: 1.0, duration: 620, ease: "outElastic(1, 0.6)" }],
      opacity: [{ to: 1, duration: 360 }, { to: 0.85, duration: 620 }],
    });
    return () => {
      pulse.pause();
    };
  }, [isPersisted]);

  // Rejected: Terracotta notch flash.
  useEffect(() => {
    if (!isRejected) return;
    const notch = notchRef.current;
    if (!notch) return;
    const reduce =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;

    const flash = animate(notch, {
      opacity: [{ to: 1, duration: 160 }, { to: 0.4, duration: 540 }],
      scale: [{ to: 1.25, duration: 160, ease: "outQuad" }, { to: 1.0, duration: 540 }],
      loop: 3,
    });
    return () => {
      flash.pause();
    };
  }, [isRejected]);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "1.6rem",
        width: "100%",
        maxWidth: 520,
      }}
    >
      {/* ── ANALYSIS MODE selector (four-point compass) ── */}
      <ModeSelector mode={mode} />

      {/* ── Capture node + flanking pause/resume glyphs ── */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: "0.5rem" }}>
        <GlyphButton
          kind="pause"
          label="Pause capture"
          onClick={onPause}
          dimmed={paused}
        />

        <svg
          ref={svgRef}
          viewBox={`0 0 ${VB} ${VB}`}
          width={300}
          height={300}
          preserveAspectRatio="xMidYMid meet"
          role="img"
          aria-label={`Capture compass, state ${stateWord.toLowerCase()}`}
        >
          <defs>
            <linearGradient id="cc-ba" x1="0%" y1="100%" x2="100%" y2="0%">
              <stop offset="0%" stopColor={EMERALD} />
              <stop offset="100%" stopColor={GOLD} />
            </linearGradient>
            <radialGradient id="cc-core" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stopColor={coreColor} stopOpacity="0.95" />
              <stop offset="55%" stopColor={coreColor} stopOpacity="0.22" />
              <stop offset="100%" stopColor={coreColor} stopOpacity="0" />
            </radialGradient>
            <filter id="cc-glow" x="-80%" y="-80%" width="260%" height="260%">
              <feGaussianBlur stdDeviation="3.2" result="b" />
              <feMerge>
                <feMergeNode in="b" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          {/* faint gold guide rings */}
          {guideDefs.map((g, i) => (
            <path
              key={i}
              d={g.d}
              fill="none"
              stroke={GOLD}
              strokeWidth={0.6}
              opacity={0.2 - i * 0.07}
            />
          ))}

          {/* radial spokes */}
          <g opacity={0.18}>
            {spokes.map((s, i) => (
              <line
                key={i}
                x1={s.inner.x}
                y1={s.inner.y}
                x2={s.outer.x}
                y2={s.outer.y}
                stroke={GOLD}
                strokeWidth={0.5}
              />
            ))}
          </g>

          {/* sacred-geometry hexagram + inner hexagon */}
          <g opacity={isRejected ? 0.22 : 0.4}>
            <path d={hexA} fill="none" stroke={GOLD} strokeWidth={0.9} />
            <path d={hexB} fill="none" stroke={GOLD} strokeWidth={0.9} />
            <path d={hexInner} fill="none" stroke={GOLD} strokeWidth={0.7} opacity={0.7} />
            {hexDots.map((p, i) => (
              <circle key={i} cx={p.x} cy={p.y} r={1.6} fill={GOLD} opacity={0.8} />
            ))}
          </g>

          {/* progress ring — track + Ba-Arc fill */}
          <path
            d={progressTrack}
            fill="none"
            stroke={SILVER}
            strokeWidth={2}
            opacity={0.12}
            strokeLinecap="round"
          />
          <path
            ref={progressRef}
            d={progressTrack}
            fill="none"
            stroke="url(#cc-ba)"
            strokeWidth={3.5}
            strokeLinecap="round"
            opacity={isRejected ? 0.25 : 0.95}
          />

          {/* breath cadence ring (emerald, 4:7:8) */}
          <circle
            ref={breathRef}
            cx={C}
            cy={C}
            r={BREATH_R}
            fill="none"
            stroke={EMERALD}
            strokeWidth={1}
            strokeDasharray="2 6"
            opacity={0.55}
            style={{ transformOrigin: `${C}px ${C}px` }}
          />

          {/* bioluminescent core */}
          <circle cx={C} cy={C} r={34} fill="url(#cc-core)" />

          {/* central capture node — clickable */}
          <g
            role="button"
            tabIndex={onCapture ? 0 : -1}
            aria-label="Begin capture"
            onClick={onCapture}
            onKeyDown={(e) => {
              if (onCapture && (e.key === "Enter" || e.key === " ")) {
                e.preventDefault();
                onCapture();
              }
            }}
            style={{ cursor: onCapture ? "pointer" : "default", outline: "none" }}
          >
            <circle
              ref={coreRef}
              cx={C}
              cy={C}
              r={7}
              fill={coreColor}
              filter="url(#cc-glow)"
              style={{ transformOrigin: `${C}px ${C}px` }}
            />
            {/* rejected notch — a Terracotta mark on the progress ring */}
            <circle
              ref={notchRef}
              cx={polar(C, C, PROGRESS_R, 38).x}
              cy={polar(C, C, PROGRESS_R, 38).y}
              r={4}
              fill={TERRACOTTA}
              opacity={isRejected ? 0.9 : 0}
              filter="url(#cc-glow)"
              style={{ transformOrigin: `${polar(C, C, PROGRESS_R, 38).x}px ${polar(C, C, PROGRESS_R, 38).y}px` }}
            />
          </g>

          {/* 4:7:8 breath label */}
          <text
            x={C}
            y={C + NODE_R - 4}
            textAnchor="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={11}
            letterSpacing="3"
            fill={EMERALD}
            opacity={0.8}
          >
            4:7:8
          </text>
          <text
            x={C}
            y={C + NODE_R + 9}
            textAnchor="middle"
            fontFamily="var(--font-mono, monospace)"
            fontSize={7}
            letterSpacing="3"
            fill={SILVER}
            opacity={0.6}
          >
            BREATH
          </text>
        </svg>

        <GlyphButton
          kind="resume"
          label="Resume capture"
          onClick={onResume}
          dimmed={!paused}
        />
      </div>

      {/* ── CAPTURE STATE STEPPER ── */}
      <StateStepper state={state} />
    </div>
  );
}

/* ──────────────────────────────────────────────────────────────────────────
   ANALYSIS MODE selector — four-point compass, one point active (Indigo blade)
   ────────────────────────────────────────────────────────────────────────── */
function ModeSelector({ mode }: { mode: string }) {
  const VB = 200;
  const C = VB / 2;
  const R = 56;
  const active = mode.trim().toUpperCase();
  const star = useMemo(() => compassStar(C, R, R, 0.5), [C, R]);

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "0.4rem" }}>
      <span
        style={{
          fontFamily: "var(--font-mono, monospace)",
          fontSize: "0.62rem",
          letterSpacing: "0.28em",
          textTransform: "uppercase",
          color: SILVER,
          opacity: 0.65,
        }}
      >
        Analysis Mode
      </span>
      <svg viewBox={`0 0 ${VB} ${VB}`} width={188} height={188} role="img" aria-label={`Analysis mode ${active}`}>
        <defs>
          <filter id="cc-mode-glow" x="-60%" y="-60%" width="220%" height="220%">
            <feGaussianBlur stdDeviation="3.5" result="b" />
            <feMerge>
              <feMergeNode in="b" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* curved-side diamond */}
        <path d={star} fill="none" stroke={GOLD} strokeWidth={1} opacity={0.7} />

        {/* axes to each tip */}
        {MODE_POINTS.map((p) => {
          const tip = polar(C, C, R, p.deg);
          const isActive = p.label === active;
          return (
            <g key={p.label}>
              <line
                x1={C}
                y1={C}
                x2={tip.x}
                y2={tip.y}
                stroke={isActive ? INDIGO : GOLD}
                strokeWidth={isActive ? 2.5 : 0.8}
                opacity={isActive ? 0.95 : 0.55}
                filter={isActive ? "url(#cc-mode-glow)" : undefined}
              />
              {isActive && (
                <polygon
                  points={`${C},${C} ${polar(C, C, R, p.deg - 7).x},${polar(C, C, R, p.deg - 7).y} ${tip.x},${tip.y} ${polar(C, C, R, p.deg + 7).x},${polar(C, C, R, p.deg + 7).y}`}
                  fill={INDIGO}
                  opacity={0.85}
                  filter="url(#cc-mode-glow)"
                />
              )}
              {/* point label */}
              <text
                x={polar(C, C, R + 18, p.deg).x}
                y={polar(C, C, R + 18, p.deg).y}
                textAnchor="middle"
                dominantBaseline="middle"
                fontFamily="var(--font-mono, monospace)"
                fontSize={8}
                letterSpacing="1.5"
                fill={isActive ? PARCHMENT : SILVER}
                opacity={isActive ? 0.95 : 0.5}
              >
                {p.label}
              </text>
            </g>
          );
        })}

        {/* centre node */}
        <circle cx={C} cy={C} r={7} fill="none" stroke={GOLD} strokeWidth={1} opacity={0.7} />
        <circle cx={C} cy={C} r={2} fill={GOLD} />
      </svg>
    </div>
  );
}

/* ──────────────────────────────────────────────────────────────────────────
   PAUSE / RESUME compass glyph buttons (real <button>s, styled as glyphs)
   ────────────────────────────────────────────────────────────────────────── */
function GlyphButton({
  kind,
  label,
  onClick,
  dimmed,
}: {
  kind: "pause" | "resume";
  label: string;
  onClick?: () => void;
  dimmed?: boolean;
}) {
  const VB = 120;
  const C = VB / 2;
  const R = 38;
  const star = useMemo(() => compassStar(C, R, R, 0.42), [C, R]);
  const ticks = [0, 90, 180, 270];

  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      style={{
        background: "transparent",
        border: "none",
        padding: 0,
        cursor: onClick ? "pointer" : "default",
        lineHeight: 0,
        opacity: dimmed ? 0.42 : 1,
        transition: "opacity 0.2s ease, transform 0.15s ease",
      }}
    >
      <svg viewBox={`0 0 ${VB} ${VB}`} width={64} height={64} aria-hidden="true">
        {/* outer ring */}
        <circle cx={C} cy={C} r={R} fill="none" stroke={GOLD} strokeWidth={1} opacity={0.6} />
        {/* curved-side compass star */}
        <path d={star} fill="none" stroke={GOLD} strokeWidth={1} opacity={0.75} />
        {/* crosshair ticks extending past the ring */}
        {ticks.map((d) => {
          const a = polar(C, C, R + 2, d);
          const b = polar(C, C, R + 12, d);
          return <line key={d} x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke={GOLD} strokeWidth={0.8} opacity={0.55} />;
        })}
        {/* small diamond on the horizontal axis */}
        <path
          d={polygon(C + R + 18, C, 3.2, 4, 0)}
          fill="none"
          stroke={GOLD}
          strokeWidth={0.8}
          opacity={0.5}
        />

        {/* glyph */}
        {kind === "pause" ? (
          <g fill={GOLD}>
            <rect x={C - 7} y={C - 9} width={4.5} height={18} rx={1} />
            <rect x={C + 2.5} y={C - 9} width={4.5} height={18} rx={1} />
          </g>
        ) : (
          <polygon points={`${C - 6},${C - 9} ${C - 6},${C + 9} ${C + 9},${C}`} fill={GOLD} />
        )}
      </svg>
    </button>
  );
}

/* ──────────────────────────────────────────────────────────────────────────
   CAPTURE STATE STEPPER — hairline path with node glyphs + branches
   ────────────────────────────────────────────────────────────────────────── */
function StateStepper({ state }: { state: CaptureCompassState }) {
  const activeIdx = state in STEP_ORDER ? STEP_ORDER[state] : -1;
  const isRejected = state === "rejected";

  // Geometry: 4 main nodes evenly spaced; rejected + reprocessed branch right.
  const VB_W = 760;
  const VB_H = 150;
  const mainY = 78;
  const nodeR = 16;
  const startX = 60;
  const gap = 150;
  const mainX = STEPPER_MAIN.map((_, i) => startX + i * gap);
  const rejectX = mainX[3] + gap; // 4th node + one gap
  const reprocX = rejectX + 120;
  const reprocY = 36;

  return (
    <svg
      viewBox={`0 0 ${VB_W} ${VB_H}`}
      width="100%"
      style={{ maxWidth: 720 }}
      role="img"
      aria-label={`Capture state stepper, current ${state}`}
    >
      <defs>
        <filter id="cc-step-glow" x="-120%" y="-120%" width="340%" height="340%">
          <feGaussianBlur stdDeviation="3" result="b" />
          <feMerge>
            <feMergeNode in="b" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      {/* main connectors + midpoint diamonds */}
      {mainX.slice(0, -1).map((x, i) => {
        const x2 = mainX[i + 1];
        const reached = activeIdx > i;
        const mid = (x + x2) / 2;
        return (
          <g key={`conn-${i}`}>
            <line
              x1={x + nodeR}
              y1={mainY}
              x2={x2 - nodeR}
              y2={mainY}
              stroke={reached ? GOLD : SILVER}
              strokeWidth={1}
              opacity={reached ? 0.6 : 0.22}
            />
            <path
              d={polygon(mid, mainY, 3.4, 4, 0)}
              fill="none"
              stroke={reached ? GOLD : SILVER}
              strokeWidth={0.8}
              opacity={reached ? 0.6 : 0.3}
            />
          </g>
        );
      })}

      {/* rejected branch — dashed terracotta from persisted node */}
      <line
        x1={mainX[3] + nodeR}
        y1={mainY}
        x2={rejectX - nodeR}
        y2={mainY}
        stroke={TERRACOTTA}
        strokeWidth={1}
        strokeDasharray="4 4"
        opacity={isRejected ? 0.8 : 0.3}
      />

      {/* reprocessed branch — dashed gold up-right from rejected node */}
      <line
        x1={rejectX + nodeR * 0.7}
        y1={mainY - nodeR * 0.7}
        x2={reprocX - nodeR * 0.7}
        y2={reprocY + nodeR * 0.7}
        stroke={GOLD}
        strokeWidth={1}
        strokeDasharray="4 4"
        opacity={0.45}
      />

      {/* main nodes */}
      {STEPPER_MAIN.map((s, i) => (
        <StepNode
          key={s}
          x={mainX[i]}
          y={mainY}
          r={nodeR}
          step={s}
          active={activeIdx === i}
          reached={activeIdx >= i}
        />
      ))}

      {/* rejected node */}
      <StepNode x={rejectX} y={mainY} r={nodeR} step="rejected" active={isRejected} reached={isRejected} />

      {/* reprocessed node */}
      <StepNode x={reprocX} y={reprocY} r={nodeR} step="reprocessed" active={false} reached={false} />
    </svg>
  );
}

const STEP_LABEL: Record<BiofieldCaptureState, string> = {
  requested: "REQUESTED",
  uploaded: "UPLOADED",
  analyzed: "ANALYZED",
  persisted: "PERSISTED",
  rejected: "REJECTED",
  reprocessed: "REPROCESSED",
};

function StepNode({
  x,
  y,
  r,
  step,
  active,
  reached,
}: {
  x: number;
  y: number;
  r: number;
  step: BiofieldCaptureState;
  active: boolean;
  reached: boolean;
}) {
  const isReject = step === "rejected";
  const isReproc = step === "reprocessed";
  const stroke = isReject ? TERRACOTTA : isReproc ? GOLD : reached ? GOLD : SILVER;
  const labelColor = active && step === "requested"
    ? EMERALD
    : isReject
      ? TERRACOTTA
      : isReproc
        ? GOLD
        : reached
          ? PARCHMENT
          : SILVER;

  return (
    <g>
      {/* active emerald halo (requested = bioluminescent solid core) */}
      {active && step === "requested" ? (
        <>
          <circle cx={x} cy={y} r={r + 3} fill={EMERALD} opacity={0.16} filter="url(#cc-step-glow)" />
          <circle cx={x} cy={y} r={r - 3} fill={EMERALD} opacity={0.9} filter="url(#cc-step-glow)" />
        </>
      ) : (
        <circle
          cx={x}
          cy={y}
          r={r}
          fill="none"
          stroke={stroke}
          strokeWidth={1.2}
          opacity={active ? 0.95 : reached ? 0.7 : 0.4}
          filter={active ? "url(#cc-step-glow)" : undefined}
        />
      )}

      {/* glyph per state */}
      <g
        stroke={stroke}
        strokeWidth={1.2}
        fill="none"
        opacity={active ? 0.95 : reached ? 0.7 : 0.4}
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        {step === "uploaded" && (
          <>
            <line x1={x} y1={y + 5} x2={x} y2={y - 6} />
            <path d={`M ${x - 4} ${y - 2} L ${x} ${y - 6} L ${x + 4} ${y - 2}`} />
          </>
        )}
        {step === "analyzed" && (
          <>
            <path d={polygon(x, y, 7, 3, 0)} />
            <path d={polygon(x, y, 7, 3, 180)} />
          </>
        )}
        {step === "persisted" && (
          <>
            <path d={polygon(x, y, 7, 4, 0)} />
            <path d={polygon(x, y, 3.4, 4, 0)} />
          </>
        )}
        {isReject && (
          <>
            <line x1={x - 4.5} y1={y - 4.5} x2={x + 4.5} y2={y + 4.5} />
            <line x1={x + 4.5} y1={y - 4.5} x2={x - 4.5} y2={y + 4.5} />
          </>
        )}
        {isReproc && (
          <>
            <path d={arc(x, y, 6, 30, 230)} />
            <path d={`M ${polar(x, y, 6, 260).x} ${polar(x, y, 6, 260).y} l 3 -3 m -3 3 l 3 3`} />
          </>
        )}
      </g>

      {/* crosshair ticks for reject/reproc (per spec) */}
      {(isReject || isReproc) &&
        [0, 90, 180, 270].map((d) => {
          const a = polar(x, y, r + 2, d);
          const b = polar(x, y, r + 8, d);
          return <line key={d} x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke={stroke} strokeWidth={0.8} opacity={0.5} />;
        })}

      {/* label */}
      <text
        x={x}
        y={isReproc ? y + r + 14 : y + r + 16}
        textAnchor="middle"
        fontFamily="var(--font-mono, monospace)"
        fontSize={8.5}
        letterSpacing="1.2"
        fill={labelColor}
        opacity={active || reached || isReject || isReproc ? 0.9 : 0.5}
      >
        {STEP_LABEL[step]}
      </text>
    </g>
  );
}

// Re-export the domain state list for consumers that want to enumerate steps.
export { BIOFIELD_CAPTURE_STATES };

export default CaptureCompass;
