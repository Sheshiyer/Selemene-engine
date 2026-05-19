"use client";

/**
 * StepIndicator — progress meter for DyadChamber flows.
 *
 * Each step is represented as a small sigil token whose shape encodes
 * the WITNESS that owns the step:
 *   - vesica (two interlocking circles)   → Aletheios (flow / witness)
 *   - hex (single point-top hexagon)      → Pichet (structure / bone)
 *   - trinity (three dots in a triangle)  → Both (joined field)
 *
 * Token color is the speaker's Goethe-palette accent. Active token
 * scales up + glows; past tokens stay visible at reduced opacity;
 * future tokens are barely-there.
 *
 * Decoupled from any specific flow's Step type — pass the steps array
 * and let the consuming page define the semantics.
 */

import { SPEAKER_COLOR, SPEAKER_LABEL, type Speaker } from "./DyadChamber";

export type SigilSymbol = "vesica" | "hex" | "trinity";

/** One step descriptor in a DyadChamber flow. */
export interface DyadStep {
  speaker: Speaker;
  symbol: SigilSymbol;
}

interface StepIndicatorProps {
  /** Ordered list of steps in this flow. */
  steps: ReadonlyArray<DyadStep>;
  /** Current step index (zero-based). */
  currentIndex: number;
  /** Called when the user clicks a token to jump back/forward. */
  onJump?: (index: number) => void;
  /** Optional aria-label for the progress container. */
  ariaLabel?: string;
}

export function StepIndicator({
  steps,
  currentIndex,
  onJump,
  ariaLabel = "Flow progress",
}: StepIndicatorProps) {
  return (
    <div
      style={{
        display: "flex",
        gap: "1.25rem",
        alignItems: "center",
        justifyContent: "center",
      }}
      role="progressbar"
      aria-valuemin={1}
      aria-valuemax={steps.length}
      aria-valuenow={currentIndex + 1}
      aria-label={ariaLabel}
    >
      {steps.map((step, i) => {
        const isActive = i === currentIndex;
        const isPast = i < currentIndex;
        const color = SPEAKER_COLOR[step.speaker];
        const opacity = isActive ? 1 : isPast ? 0.55 : 0.18;

        return (
          <button
            key={i}
            type="button"
            onClick={() => onJump?.(i)}
            aria-label={`Step ${i + 1} · ${SPEAKER_LABEL[step.speaker]}`}
            title={SPEAKER_LABEL[step.speaker]}
            disabled={!onJump}
            style={{
              border: "none",
              background: "transparent",
              padding: 4,
              cursor: onJump ? "pointer" : "default",
              transform: isActive ? "scale(1.25)" : "scale(1)",
              filter: isActive ? `drop-shadow(0 0 8px ${color})` : undefined,
              transition: "transform 200ms ease, filter 200ms ease",
            }}
          >
            <SigilToken symbol={step.symbol} color={color} opacity={opacity} />
          </button>
        );
      })}
    </div>
  );
}

/* ── SigilToken — primitive SVG shape used by StepIndicator ── */

interface SigilTokenProps {
  symbol: SigilSymbol;
  color: string;
  opacity: number;
}

export function SigilToken({ symbol, color, opacity }: SigilTokenProps) {
  if (symbol === "hex") {
    return (
      <svg width="14" height="16" viewBox="0 0 14 16" aria-hidden="true">
        <polygon
          points="7,1 13,4.5 13,11.5 7,15 1,11.5 1,4.5"
          fill="none"
          stroke={color}
          strokeWidth="1.2"
          opacity={opacity}
        />
      </svg>
    );
  }
  if (symbol === "vesica") {
    return (
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
        <circle cx="5.5" cy="8" r="4.2" fill="none" stroke={color} strokeWidth="1.0" opacity={opacity} />
        <circle cx="10.5" cy="8" r="4.2" fill="none" stroke={color} strokeWidth="1.0" opacity={opacity} />
      </svg>
    );
  }
  // trinity-dot — three small filled dots in a triangle
  return (
    <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
      <circle cx="7"  cy="3"  r="1.4" fill={color} opacity={opacity} />
      <circle cx="3"  cy="10" r="1.4" fill={color} opacity={opacity} />
      <circle cx="11" cy="10" r="1.4" fill={color} opacity={opacity} />
    </svg>
  );
}
