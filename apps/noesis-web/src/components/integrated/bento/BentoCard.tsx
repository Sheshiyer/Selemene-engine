"use client";

// ─── BentoCard — large rounded dark card matching the Brand Design System ───
// Reference: opensession.substack.com Brand Design System (Figma file
// o9PVqwfQDNOWBTqbjHBs5l). Translated to the Tryambakam Noesis palette
// (Goethe spectrum on Void Black) per apps/noesis-web/DESIGN.md.
//
// Composition:
//   <BentoCard
//     eyebrow="Brand"
//     title="Composite Field"
//     description="The triadic resonance of three souls in one coherent geometry."
//     pill={{ label: "Open", href: "..." }}
//     status="LIVE">
//     {children — feature visual or sub-grid}
//   </BentoCard>
//
// The header section uses Void Black with the dark-card stack. The
// `tone="featured"` prop swaps the featured visual zone to Parchment
// fill (matching the reference's cream-on-black contrast). Tone "void"
// keeps it all dark; tone "auto" uses dark for content cards, parchment
// for spotlight bento.

import { motion, type HTMLMotionProps } from "motion/react";
import { type ReactNode } from "react";

export type BentoTone = "void" | "parchment" | "violet" | "indigo" | "gold" | "emerald";

interface PillProps {
  label: string;
  href?: string;
  onClick?: () => void;
}

interface BentoCardProps {
  /** Small uppercase eyebrow above the title (e.g. "Brand", "Part III") */
  eyebrow?: string;
  /** Massive display title (Panchang 700/800) */
  title?: string;
  /** Body description under the title */
  description?: string;
  /** Optional pill action top-right */
  pill?: PillProps;
  /** Optional status badge (top-right of card if no pill) */
  status?: string;
  /** Tone of the featured/content zone */
  tone?: BentoTone;
  /** Span across the bento grid (1 = standard, 2 = wide, 3 = full row) */
  span?: 1 | 2 | 3;
  /** Vertical row span (1 = standard, 2 = tall) */
  rowSpan?: 1 | 2;
  /** Whether to render the featured visual zone (children go here when present). Default true. */
  hasFeature?: boolean;
  /** Optional className passthrough */
  className?: string;
  /** Child content — when hasFeature is true, this renders inside the Parchment/tinted featured block */
  children?: ReactNode;
  /** ID for anchor links */
  id?: string;
  motionProps?: HTMLMotionProps<"div">;
}

const TONE_BG: Record<BentoTone, string> = {
  void: "linear-gradient(160deg, rgba(14,20,40,0.92) 0%, rgba(7,11,29,0.96) 100%)",
  parchment: "var(--c-parchment, #F0EDE3)",
  violet: "linear-gradient(160deg, rgba(45,0,80,0.55) 0%, rgba(7,11,29,0.92) 100%)",
  indigo: "linear-gradient(160deg, rgba(11,80,251,0.20) 0%, rgba(7,11,29,0.95) 100%)",
  gold: "linear-gradient(160deg, rgba(197,160,23,0.18) 0%, rgba(7,11,29,0.94) 100%)",
  emerald: "linear-gradient(160deg, rgba(16,181,167,0.22) 0%, rgba(7,11,29,0.95) 100%)",
};

const TONE_FG: Record<BentoTone, string> = {
  void: "var(--c-parchment, #F0EDE3)",
  parchment: "var(--c-void, #070B1D)",
  violet: "var(--c-parchment, #F0EDE3)",
  indigo: "var(--c-parchment, #F0EDE3)",
  gold: "var(--c-parchment, #F0EDE3)",
  emerald: "var(--c-parchment, #F0EDE3)",
};

export function BentoCard({
  eyebrow,
  title,
  description,
  pill,
  status,
  tone = "void",
  span = 1,
  rowSpan = 1,
  hasFeature = true,
  className,
  children,
  id,
  motionProps,
}: BentoCardProps) {
  const isParchment = tone === "parchment";

  return (
    <motion.section
      id={id}
      className={`bento-card ${className ?? ""}`}
      initial={{ opacity: 0, y: 24 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: false, margin: "0% 0% -10% 0%" }}
      transition={{ duration: 0.7, ease: [0.16, 1, 0.3, 1] }}
      {...motionProps}
      style={{
        position: "relative",
        gridColumn: `span ${span}`,
        gridRow: `span ${rowSpan}`,
        background:
          "linear-gradient(160deg, rgba(14,20,40,0.92) 0%, rgba(7,11,29,0.96) 100%)",
        borderRadius: "clamp(1rem, 1.4vw, 1.75rem)",
        border: "1px solid rgba(197,160,23,0.10)",
        boxShadow:
          "0 1px 0 rgba(255,255,255,0.04) inset, 0 24px 64px rgba(0,0,0,0.40)",
        padding: "clamp(1.25rem, 2vw, 2rem) clamp(1.25rem, 2vw, 2rem) 0",
        display: "flex",
        flexDirection: "column",
        gap: "clamp(0.75rem, 1.2vw, 1.2rem)",
        overflow: "hidden",
        color: "var(--c-parchment, #F0EDE3)",
        ...motionProps?.style,
      }}
    >
      {/* Header zone — eyebrow + title + description + pill/status */}
      {(eyebrow || title || description || pill || status) && (
        <header
          style={{
            display: "grid",
            gridTemplateColumns: "1fr auto",
            gap: "clamp(1rem, 2vw, 1.6rem)",
            alignItems: "start",
            paddingBottom: "clamp(0.5rem, 1vw, 1rem)",
          }}
        >
          <div style={{ display: "flex", flexDirection: "column", gap: "clamp(0.5rem, 1vw, 1rem)" }}>
            {eyebrow && (
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "clamp(0.7rem, 0.65rem + 0.12vw, 0.82rem)",
                  letterSpacing: "0.18em",
                  textTransform: "uppercase",
                  color: "var(--c-gold, #C5A017)",
                  fontWeight: 600,
                }}
              >
                {eyebrow}
              </div>
            )}
            {title && (
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontWeight: 800,
                  fontSize: "clamp(1.75rem, 1.4rem + 1.8vw, 3.8rem)",
                  lineHeight: 1.0,
                  letterSpacing: "-0.035em",
                  margin: 0,
                  color: "var(--c-parchment, #F0EDE3)",
                }}
              >
                {title}
              </h2>
            )}
            {description && (
              <p
                style={{
                  fontFamily: "var(--font-body)",
                  fontSize: "clamp(0.9rem, 0.82rem + 0.2vw, 1.05rem)",
                  lineHeight: 1.5,
                  color: "var(--muted, rgba(240,237,227,0.65))",
                  maxWidth: "64ch",
                  margin: 0,
                }}
              >
                {description}
              </p>
            )}
          </div>
          {pill ? (
            <BentoPill {...pill} />
          ) : status ? (
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.7rem",
                letterSpacing: "0.2em",
                textTransform: "uppercase",
                padding: "0.4rem 0.9rem",
                borderRadius: "999px",
                background: "rgba(16,181,167,0.14)",
                border: "1px solid rgba(16,181,167,0.32)",
                color: "var(--c-emerald, #10B5A7)",
                whiteSpace: "nowrap",
                alignSelf: "start",
              }}
            >
              {status}
            </span>
          ) : null}
        </header>
      )}

      {/* Featured visual zone — sits at the bottom of the card, optionally parchment-filled */}
      {hasFeature && children && (
        <div
          style={{
            position: "relative",
            background: TONE_BG[tone],
            color: TONE_FG[tone],
            borderRadius: "clamp(0.75rem, 1vw, 1.25rem)",
            marginTop: "auto",
            marginLeft: "calc(-1 * clamp(0.5rem, 0.6vw, 0.6rem))",
            marginRight: "calc(-1 * clamp(0.5rem, 0.6vw, 0.6rem))",
            marginBottom: "calc(-1 * clamp(0.5rem, 0.6vw, 0.6rem))",
            padding: "clamp(1.25rem, 2vw, 2rem)",
            border: isParchment ? "none" : "1px solid rgba(255,255,255,0.04)",
            minHeight: "clamp(8rem, 18vw, 14rem)",
            display: "flex",
            flexDirection: "column",
            justifyContent: "center",
            overflow: "hidden",
          }}
        >
          {children}
        </div>
      )}

      {/* When no feature zone, render children directly in the card body */}
      {!hasFeature && children && (
        <div style={{ marginTop: "auto" }}>{children}</div>
      )}
    </motion.section>
  );
}

// ─── BentoPill — rounded link action ──────────────────────────────────────
export function BentoPill({ label, href, onClick }: PillProps) {
  const Inner = (
    <>
      <span>{label}</span>
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
      >
        <path d="M7 17L17 7" />
        <path d="M7 7h10v10" />
      </svg>
    </>
  );
  const sharedStyle: React.CSSProperties = {
    display: "inline-flex",
    alignItems: "center",
    gap: "0.45rem",
    padding: "0.55rem 1rem",
    borderRadius: "999px",
    background: "rgba(240,237,227,0.06)",
    border: "1px solid rgba(240,237,227,0.18)",
    color: "var(--c-parchment, #F0EDE3)",
    fontFamily: "var(--font-mono)",
    fontSize: "0.78rem",
    letterSpacing: "0.05em",
    textTransform: "uppercase" as const,
    textDecoration: "none",
    cursor: "pointer",
    transition: "background 200ms ease, transform 200ms ease, border-color 200ms ease",
    whiteSpace: "nowrap",
  };
  if (href) {
    return (
      <a
        href={href}
        style={sharedStyle}
        onMouseEnter={(e) => {
          e.currentTarget.style.background = "rgba(197,160,23,0.18)";
          e.currentTarget.style.borderColor = "rgba(197,160,23,0.55)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.background = "rgba(240,237,227,0.06)";
          e.currentTarget.style.borderColor = "rgba(240,237,227,0.18)";
        }}
      >
        {Inner}
      </a>
    );
  }
  return (
    <button type="button" onClick={onClick} style={{ ...sharedStyle, font: "inherit", fontFamily: sharedStyle.fontFamily }}>
      {Inner}
    </button>
  );
}
