"use client";

/**
 * SessionStrip — Wave 1, built 1:1 to docs/design/biofield-web/11-foundations-spec.png
 * (section 3 · SESSION STRIP).
 *
 * A single horizontal hairline strip — NO boxy card. Cells separated by faint
 * vertical rules show: Account (email), Session status as a small
 * sacred-geometry status glyph, Tier, and Start / End as compass-glyph
 * <button>s (keyboard-accessible, aria-labelled).
 *
 * Status glyphs (SVG, matches BIOFIELD_SESSION_STATUSES):
 *   active    — emerald sealed cross (live)
 *   closed    — silver closed ring
 *   abandoned — terracotta open triangle
 *   none      — hollow silver dot (no session)
 */

import type { BiofieldSessionStatus } from "@selemene/biofield-domain";

const GOLD = "#C5A017";
const EMERALD = "#10B5A7";
const SILVER = "#8A9BA8";
const PARCHMENT = "#F0EDE3";
const TERRACOTTA = "#C65D3B";
const INDIGO = "#0B50FB";

type StripStatus = BiofieldSessionStatus | "none";

export interface SessionStripProps {
  email?: string;
  status?: StripStatus;
  tier?: string;
  onStart?: () => void;
  onEnd?: () => void;
}

const STATUS_META: Record<StripStatus, { color: string; label: string }> = {
  active: { color: EMERALD, label: "Active" },
  closed: { color: SILVER, label: "Closed" },
  abandoned: { color: TERRACOTTA, label: "Abandoned" },
  none: { color: SILVER, label: "No session" },
};

/** Sacred-geometry status glyph per session state. */
function StatusGlyph({ status }: { status: StripStatus }) {
  const { color } = STATUS_META[status];
  return (
    <svg width={20} height={20} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <circle cx={12} cy={12} r={9} stroke={color} strokeWidth={1} opacity={0.35} />
      {status === "active" && (
        <>
          {/* sealed cross — live, radiant */}
          <path d="M12 6 V18 M6 12 H18" stroke={color} strokeWidth={1.6} strokeLinecap="round" />
          <circle cx={12} cy={12} r={3} fill={color} />
        </>
      )}
      {status === "closed" && (
        // closed ring — sealed circle
        <circle cx={12} cy={12} r={5} stroke={color} strokeWidth={1.6} />
      )}
      {status === "abandoned" && (
        // open triangle — broken / left
        <path d="M12 7 L17 16 H7 Z" stroke={color} strokeWidth={1.5} strokeLinejoin="round" />
      )}
      {status === "none" && <circle cx={12} cy={12} r={2.2} fill={color} opacity={0.5} />}
    </svg>
  );
}

/** Compass-glyph button used for Start / End. */
function CompassButton({
  label,
  color,
  onClick,
  disabled,
}: {
  label: string;
  color: string;
  onClick?: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={disabled ? undefined : onClick}
      disabled={disabled}
      aria-label={label}
      title={label}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "0.5rem",
        padding: "0.4rem 0.7rem",
        borderRadius: 0,
        background: "transparent",
        border: `1px solid ${disabled ? SILVER : color}`,
        color: disabled ? SILVER : color,
        opacity: disabled ? 0.35 : 1,
        cursor: disabled ? "not-allowed" : "pointer",
        fontFamily: "var(--font-mono, monospace)",
        fontSize: "0.7rem",
        fontWeight: 600,
        letterSpacing: "0.16em",
        textTransform: "uppercase",
        transition: "color 0.15s ease, border-color 0.15s ease, background 0.15s ease",
      }}
      onMouseEnter={(e) => {
        if (disabled) return;
        e.currentTarget.style.background = "rgba(255,255,255,0.04)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = "transparent";
      }}
    >
      <svg width={14} height={14} viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle cx={12} cy={12} r={9} stroke="currentColor" strokeWidth={1.2} opacity={0.6} />
        <path d="M12 3 V6 M12 18 V21 M3 12 H6 M18 12 H21" stroke="currentColor" strokeWidth={1.2} />
        <path d="M12 6 L14 12 L12 11 L10 12 Z" fill="currentColor" />
      </svg>
      {label}
    </button>
  );
}

const cellLabel: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.62rem",
  letterSpacing: "0.16em",
  textTransform: "uppercase",
  color: SILVER,
  opacity: 0.7,
  display: "block",
  marginBottom: "0.2rem",
};

const cellValue: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.82rem",
  color: PARCHMENT,
  letterSpacing: "0.02em",
};

export function SessionStrip({
  email,
  status = "none",
  tier,
  onStart,
  onEnd,
}: SessionStripProps) {
  const meta = STATUS_META[status];
  const sessionLive = status === "active";

  const divider: React.CSSProperties = {
    width: 1,
    alignSelf: "stretch",
    background: "rgba(138,155,168,0.18)",
  };

  return (
    <div
      role="group"
      aria-label="Session strip"
      style={{
        display: "flex",
        alignItems: "center",
        gap: "1.25rem",
        padding: "0.85rem 0",
        // hairline top & bottom only — no card, no rounded box
        borderTop: "1px solid rgba(138,155,168,0.18)",
        borderBottom: "1px solid rgba(138,155,168,0.18)",
        width: "100%",
        flexWrap: "wrap",
      }}
    >
      {/* Account */}
      <div style={{ minWidth: 180 }}>
        <span style={cellLabel}>Account</span>
        <span style={cellValue}>{email ?? "—"}</span>
      </div>

      <div style={divider} aria-hidden="true" />

      {/* Status — sacred-geometry glyph + word */}
      <div style={{ display: "flex", alignItems: "center", gap: "0.55rem" }}>
        <StatusGlyph status={status} />
        <div>
          <span style={cellLabel}>Session</span>
          <span style={{ ...cellValue, color: meta.color }}>{meta.label}</span>
        </div>
      </div>

      <div style={divider} aria-hidden="true" />

      {/* Tier */}
      <div style={{ minWidth: 90 }}>
        <span style={cellLabel}>Tier</span>
        <span style={{ ...cellValue, color: GOLD }}>{tier ?? "—"}</span>
      </div>

      {/* Actions — pushed to the right */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "0.6rem",
          marginLeft: "auto",
        }}
      >
        <CompassButton
          label="Start session"
          color={INDIGO}
          onClick={onStart}
          disabled={sessionLive}
        />
        <CompassButton
          label="End session"
          color={GOLD}
          onClick={onEnd}
          disabled={!sessionLive}
        />
      </div>
    </div>
  );
}

export default SessionStrip;
