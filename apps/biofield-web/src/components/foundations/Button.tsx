"use client";

/**
 * SigilButton — Wave 1, built 1:1 to docs/design/biofield-web/11-foundations-spec.png
 * (section 1 · BUTTONS).
 *
 * Sharp-edged sigil buttons — NO rounded corners, NO pills. Three text variants
 * plus an icon variant:
 *   - primary   — Sacred Gold wireframe that fills with the Ba Arc gradient
 *                 (emerald -> gold) on hover/active.
 *   - secondary — Muted Silver hairline outline.
 *   - ghost     — Parchment text only, no frame.
 *   - icon      — square compass-glyph button (set `icon`).
 * States: default / hover / active / disabled.
 *
 * Motion: Anime.js v4 (named `animate`). On hover a gentle gold glow-blooms
 * from within (box-shadow inset->outset). Guarded by prefers-reduced-motion
 * and cleaned up on unmount / pointer-leave. Mirrors CosmogramRing.tsx.
 */

import { useEffect, useRef, useState } from "react";
import { animate } from "animejs";

const GOLD = "#C5A017";
const EMERALD = "#10B5A7";
const SILVER = "#8A9BA8";
const PARCHMENT = "#F0EDE3";

type Variant = "primary" | "secondary" | "ghost";

export interface SigilButtonProps {
  variant?: Variant;
  disabled?: boolean;
  onClick?: () => void;
  children: React.ReactNode;
  /** When true, render the compass-glyph icon variant (children become aria-label). */
  icon?: boolean;
  className?: string;
}

/** Reduced-motion check, identical guard to CosmogramRing. */
function prefersReducedMotion(): boolean {
  return (
    typeof window !== "undefined" &&
    !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches
  );
}

/** Sharp compass rose glyph used by the icon variant. */
function CompassGlyph({ color }: { color: string }) {
  return (
    <svg width={18} height={18} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <circle cx={12} cy={12} r={9} stroke={color} strokeWidth={1.2} opacity={0.55} />
      {/* four cardinal ticks */}
      <path d="M12 3 V7 M12 17 V21 M3 12 H7 M17 12 H21" stroke={color} strokeWidth={1.2} />
      {/* needle — N gold, S hollow */}
      <path d="M12 5 L14.4 12 L12 11 L9.6 12 Z" fill={color} />
      <path d="M12 19 L9.6 12 L12 13 L14.4 12 Z" fill={color} opacity={0.35} />
    </svg>
  );
}

export function SigilButton({
  variant = "primary",
  disabled = false,
  onClick,
  children,
  icon = false,
  className,
}: SigilButtonProps) {
  const ref = useRef<HTMLButtonElement | null>(null);
  const [hovered, setHovered] = useState(false);

  // Hover glow-from-within. primary + icon glow gold; secondary glows silver;
  // ghost stays quiet (text only). Animated on enter, eased back on leave.
  useEffect(() => {
    const el = ref.current;
    if (!el || disabled) return;
    if (prefersReducedMotion()) return;

    const glowColor = variant === "secondary" ? SILVER : GOLD;
    const rgb =
      variant === "secondary"
        ? "138, 155, 168" // silver
        : "197, 160, 23"; // gold

    if (variant === "ghost" && !icon) return; // ghost text has no frame to glow

    const lit = `inset 0 0 12px rgba(${rgb}, 0.35), 0 0 18px rgba(${rgb}, 0.45)`;
    const dark = `inset 0 0 0 rgba(${rgb}, 0), 0 0 0 rgba(${rgb}, 0)`;

    const anim = animate(el, {
      boxShadow: hovered ? [dark, lit] : [lit, dark],
      duration: hovered ? 420 : 300,
      ease: "outQuad",
    });

    return () => {
      anim.pause();
    };
  }, [hovered, variant, disabled, icon]);

  // ── Static (non-animated) styling per variant/state ──
  const isPrimary = variant === "primary";
  const isSecondary = variant === "secondary";
  const isGhost = variant === "ghost";

  const fg = isPrimary
    ? hovered && !disabled
      ? "#070B1D" // dark text reads on the gold fill
      : GOLD
    : isSecondary
      ? SILVER
      : PARCHMENT;

  const base: React.CSSProperties = {
    position: "relative",
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: "0.5rem",
    padding: icon ? "0.6rem" : "0.62rem 1.25rem",
    aspectRatio: icon ? "1 / 1" : undefined,
    borderRadius: 0, // sharp edges — never rounded
    fontFamily: "var(--font-mono, monospace)",
    fontSize: "0.74rem",
    fontWeight: 600,
    letterSpacing: "0.16em",
    textTransform: "uppercase",
    color: fg,
    background:
      isPrimary && hovered && !disabled
        ? "var(--grad-ba, linear-gradient(90deg, #10B5A7 0%, #C5A017 100%))"
        : "transparent",
    border: isGhost
      ? "1px solid transparent"
      : isPrimary
        ? `1px solid ${GOLD}`
        : `1px solid ${SILVER}`,
    cursor: disabled ? "not-allowed" : "pointer",
    opacity: disabled ? 0.32 : 1,
    transition:
      "color 0.18s ease, background 0.22s ease, border-color 0.18s ease, transform 0.08s ease",
    outline: "none",
    transformOrigin: "center",
    WebkitTapHighlightColor: "transparent",
  };

  // Secondary/ghost hover: brighten the line/text (cheap, non-anime fallback path).
  if (!disabled && hovered) {
    if (isSecondary) base.borderColor = PARCHMENT;
    if (isSecondary) base.color = PARCHMENT;
    if (isGhost) base.color = GOLD;
  }

  return (
    <button
      ref={ref}
      type="button"
      disabled={disabled}
      onClick={disabled ? undefined : onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onFocus={() => setHovered(true)}
      onBlur={() => setHovered(false)}
      onMouseDown={(e) => {
        // active state: subtle inward press
        if (!disabled) e.currentTarget.style.transform = "scale(0.97)";
      }}
      onMouseUp={(e) => {
        e.currentTarget.style.transform = "scale(1)";
      }}
      aria-label={icon && typeof children === "string" ? children : undefined}
      className={className}
      style={base}
    >
      {icon ? (
        <CompassGlyph color={fg} />
      ) : (
        children
      )}
    </button>
  );
}

export default SigilButton;
