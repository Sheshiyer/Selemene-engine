// ─── sigils — shared micro-sigil SVG primitives ────────────────────────
// Three archetypal sacred-geometry forms used as row markers, frame
// outlines, and decorative anchors throughout the prose carriers.
// Cycled by row index so adjacent rows visually distinguish.
//
// All sigils are 24×24 unit space, scaled by the caller via the `size`
// prop. Strokes are tuned for legibility at 12px–48px.

import type { CSSProperties } from "react";

interface SigilProps {
  /** Size in px. */
  size?: number;
  /** Primary stroke / fill color. */
  color?: string;
  /** Optional inner accent (defaults to color at half opacity). */
  accent?: string;
  /** Extra style overrides (e.g. transform). */
  style?: CSSProperties;
  /** Stroke width override; defaults to 1.25 (scaled with size). */
  strokeWidth?: number;
}

const VIEWBOX = "0 0 24 24";

/** Triad — three small circles arranged in an equilateral triangle. */
export function SigilTriad({
  size = 24,
  color = "#C5A017",
  accent,
  style,
  strokeWidth = 1.25,
}: SigilProps) {
  const acc = accent ?? color;
  return (
    <svg width={size} height={size} viewBox={VIEWBOX} style={style} aria-hidden="true">
      <circle cx="12" cy="6.5" r="2.6" fill="none" stroke={color} strokeWidth={strokeWidth} />
      <circle cx="6"  cy="16"  r="2.6" fill="none" stroke={color} strokeWidth={strokeWidth} />
      <circle cx="18" cy="16"  r="2.6" fill="none" stroke={color} strokeWidth={strokeWidth} />
      <circle cx="12" cy="12.5" r="0.9" fill={acc} opacity="0.85" />
    </svg>
  );
}

/** Vesica — two overlapping circles forming a vesica piscis. */
export function SigilVesica({
  size = 24,
  color = "#C5A017",
  accent,
  style,
  strokeWidth = 1.25,
}: SigilProps) {
  const acc = accent ?? color;
  return (
    <svg width={size} height={size} viewBox={VIEWBOX} style={style} aria-hidden="true">
      <circle cx="9"  cy="12" r="5.5" fill="none" stroke={color} strokeWidth={strokeWidth} />
      <circle cx="15" cy="12" r="5.5" fill="none" stroke={color} strokeWidth={strokeWidth} />
      <line x1="12" y1="6.5" x2="12" y2="17.5" stroke={acc} strokeWidth={strokeWidth * 0.6} opacity="0.55" />
    </svg>
  );
}

/** Hex — pointy-top hexagon with center dot. */
export function SigilHex({
  size = 24,
  color = "#C5A017",
  accent,
  style,
  strokeWidth = 1.25,
}: SigilProps) {
  const acc = accent ?? color;
  return (
    <svg width={size} height={size} viewBox={VIEWBOX} style={style} aria-hidden="true">
      <polygon
        points="12,3 20.5,8 20.5,16 12,21 3.5,16 3.5,8"
        fill="none"
        stroke={color}
        strokeWidth={strokeWidth}
      />
      <circle cx="12" cy="12" r="1.2" fill={acc} opacity="0.85" />
    </svg>
  );
}

/** Compass — four cardinal ticks around a center crosshair. */
export function SigilCompass({
  size = 24,
  color = "#C5A017",
  accent,
  style,
  strokeWidth = 1.25,
}: SigilProps) {
  const acc = accent ?? color;
  return (
    <svg width={size} height={size} viewBox={VIEWBOX} style={style} aria-hidden="true">
      <circle cx="12" cy="12" r="8.5" fill="none" stroke={color} strokeWidth={strokeWidth * 0.7} opacity="0.55" />
      <line x1="12" y1="2"  x2="12" y2="5.5" stroke={color} strokeWidth={strokeWidth} />
      <line x1="12" y1="18.5" x2="12" y2="22" stroke={color} strokeWidth={strokeWidth} />
      <line x1="2"  y1="12" x2="5.5" y2="12" stroke={color} strokeWidth={strokeWidth} />
      <line x1="18.5" y1="12" x2="22" y2="12" stroke={color} strokeWidth={strokeWidth} />
      <circle cx="12" cy="12" r="1.4" fill={acc} />
    </svg>
  );
}

/** Pick a sigil from the rotation by row index. */
export function MicroSigil({
  index,
  size = 24,
  color = "#C5A017",
  accent,
  style,
}: SigilProps & { index: number }) {
  const variant = index % 4;
  const props = { size, color, accent, style };
  if (variant === 0) return <SigilTriad {...props} />;
  if (variant === 1) return <SigilVesica {...props} />;
  if (variant === 2) return <SigilHex {...props} />;
  return <SigilCompass {...props} />;
}

// ─── Frame primitives — large geometric containers ──────────────────────

/** Hexagonal SVG frame, drawn at any size, with the content rendered as
 *  HTML positioned over the frame via foreignObject would be ideal but
 *  doesn't always honor styles. Instead the caller positions an HTML
 *  div ABOVE the SVG, and uses the same dimensions. We expose just the
 *  outline. */
export function HexFrame({
  size = 200,
  color = "#C5A017",
  fill,
  strokeWidth = 1.25,
  style,
}: SigilProps & { fill?: string }) {
  // Pointy-top hexagon proportions: width = √3 × radius, height = 2 × radius
  // We use 100×116 coordinate space so the geometry is precise.
  return (
    <svg
      width={size}
      height={size * (116 / 100)}
      viewBox="0 0 100 116"
      style={style}
      aria-hidden="true"
    >
      <polygon
        points="50,3 96,28 96,88 50,113 4,88 4,28"
        fill={fill ?? "none"}
        stroke={color}
        strokeWidth={strokeWidth}
        strokeLinejoin="round"
      />
    </svg>
  );
}

/** Concentric ring set — used by WitnessPulse and as a decorative
 *  background under headings. Renders 4 rings with decreasing opacity. */
export function ConcentricRings({
  size = 180,
  color = "#C5A017",
  innerColor,
  style,
  strokeWidth = 0.8,
}: SigilProps & { innerColor?: string }) {
  const inner = innerColor ?? color;
  return (
    <svg width={size} height={size} viewBox="0 0 100 100" style={style} aria-hidden="true">
      <circle cx="50" cy="50" r="46" fill="none" stroke={color} strokeWidth={strokeWidth} opacity="0.22" />
      <circle cx="50" cy="50" r="36" fill="none" stroke={color} strokeWidth={strokeWidth} opacity="0.42" />
      <circle cx="50" cy="50" r="26" fill="none" stroke={color} strokeWidth={strokeWidth} opacity="0.62" />
      <circle cx="50" cy="50" r="16" fill="none" stroke={color} strokeWidth={strokeWidth} opacity="0.85" />
      <circle cx="50" cy="50" r="4"  fill={inner} opacity="0.92" />
    </svg>
  );
}
