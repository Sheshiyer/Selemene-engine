"use client";

/**
 * DyadChamber — the canonical guided-flow chrome for Noesis.
 *
 * Two character portraits flank the form content: Pichet on the left
 * (embodied / structure / "the bone"), Aletheios on the right
 * (witness / flow / "names you into being"). The character owning
 * the current step is lit forward; the other dims back to watching.
 *
 * Why this is canonical:
 *   The brand vision says "sacred geometry as load-bearing architecture,
 *   not decoration." The dyad isn't an illustration — it IS the
 *   navigation chrome. Every guided flow in Noesis (birth-data intake,
 *   biofield session capture, settings save dialogs, onboarding)
 *   should use this pattern instead of generic form headers.
 *
 * Asset contract:
 *   Portraits live at `/depth-reading/characters/{name}-front.png`.
 *   Currently `name` ∈ { "pichet", "aletheios" }.
 *
 * Reusability:
 *   This component is generic over step semantics — it only takes a
 *   `speaker` prop ("pichet" | "aletheios" | "both"). Each consuming
 *   flow defines its own step→speaker mapping.
 *
 * CSS:
 *   Uses the global keyframes `thresholdPulse` and `thresholdRingDrift`
 *   which live in app/globals.css under the DYAD-CHAMBER section.
 */

export type Speaker = "pichet" | "aletheios" | "both";

export const SPEAKER_COLOR: Record<Speaker, string> = {
  pichet: "#C5A017",     // Sacred Gold — structure / activation
  aletheios: "#10B5A7",  // Coherence Emerald — witness / flow
  both: "#F0EDE3",       // Parchment — the joined field
};

export const SPEAKER_LABEL: Record<Speaker, string> = {
  pichet: "PICHET",
  aletheios: "ALETHEIOS",
  both: "ALETHEIOS + PICHET",
};

/**
 * Variants:
 *   - "full":   Full-bleed chamber. Portraits at ~75vh on either edge.
 *               Use on dedicated ritual surfaces (get-reading, auth,
 *               biofield-intake). Mounts position:absolute, fills parent.
 *   - "banner": Compact top-banner. Portraits at ~140px tall in a strip
 *               above the form. Use on content-dense surfaces (engines,
 *               settings, readings detail) where full-bleed would compete.
 */
export type DyadVariant = "full" | "banner";

interface DyadChamberProps {
  /** Which witness owns the current cognitive moment. */
  speaker: Speaker;
  /** True while a submission is in flight — pulses the active witness(es). */
  submitting?: boolean;
  /** Layout variant. Defaults to "full". */
  variant?: DyadVariant;
  /**
   * Optional className for the wrapping stage. Allows consuming flows
   * to control position (absolute vs relative) and z-index without
   * overriding the witness positioning logic.
   */
  className?: string;
}

/**
 * The full dyad chamber: two witnesses + a faint orbital ring sigil.
 * Mounts as `position: absolute` and stretches to fill its parent —
 * the parent should be `position: relative` with intended dimensions.
 */
export function DyadChamber({
  speaker,
  submitting = false,
  variant = "full",
  className,
}: DyadChamberProps) {
  const pichetActive = speaker === "pichet" || speaker === "both";
  const aletheiosActive = speaker === "aletheios" || speaker === "both";

  /* Banner variant: an in-flow strip rather than absolute overlay.
     Portraits are smaller, sit on a single horizontal row with the
     orbital ring small in the middle. Consuming page lays out
     normally below the banner. */
  if (variant === "banner") {
    return (
      <div
        aria-hidden="true"
        className={className}
        style={{
          position: "relative",
          width: "100%",
          height: 200,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          pointerEvents: "none",
          overflow: "hidden",
        }}
      >
        <svg
          viewBox="0 0 200 200"
          style={{
            position: "absolute",
            width: 180,
            height: 180,
            opacity: 0.28,
            pointerEvents: "none",
          }}
        >
          <g
            style={{
              animation: "thresholdRingDrift 120s linear infinite",
              transformOrigin: "100px 100px",
            }}
          >
            <circle cx="100" cy="100" r="94" fill="none" stroke="#C5A017" strokeOpacity="0.4" strokeWidth="0.6" />
            <circle cx="100" cy="100" r="78" fill="none" stroke="#10B5A7" strokeOpacity="0.4" strokeWidth="0.6" />
          </g>
        </svg>

        <BannerWitness
          side="left"
          name="pichet"
          active={pichetActive}
          submitting={submitting && (speaker === "pichet" || speaker === "both")}
        />
        <BannerWitness
          side="right"
          name="aletheios"
          active={aletheiosActive}
          submitting={submitting && (speaker === "aletheios" || speaker === "both")}
        />
      </div>
    );
  }

  /* Full variant — original chamber, absolute fill */
  return (
    <div
      aria-hidden="true"
      className={className}
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        pointerEvents: "none",
        overflow: "hidden",
        zIndex: 0,
      }}
    >
      {/* Faint orbital sigil between the two witnesses — a thin gold
          and emerald double ring slowly rotating. Atmospheric, not
          decorative: it grounds the dyad as ONE field, not two
          separate portraits. */}
      <svg
        viewBox="0 0 200 200"
        style={{
          position: "absolute",
          width: "clamp(360px, 50vmin, 720px)",
          height: "clamp(360px, 50vmin, 720px)",
          opacity: 0.25,
          pointerEvents: "none",
        }}
      >
        <g
          style={{
            animation: "thresholdRingDrift 120s linear infinite",
            transformOrigin: "100px 100px",
          }}
        >
          <circle cx="100" cy="100" r="94" fill="none" stroke="#C5A017" strokeOpacity="0.4" strokeWidth="0.4" />
          <circle cx="100" cy="100" r="78" fill="none" stroke="#10B5A7" strokeOpacity="0.4" strokeWidth="0.4" />
        </g>
      </svg>

      <WitnessFigure
        side="left"
        name="pichet"
        active={pichetActive}
        submitting={submitting && (speaker === "pichet" || speaker === "both")}
      />
      <WitnessFigure
        side="right"
        name="aletheios"
        active={aletheiosActive}
        submitting={submitting && (speaker === "aletheios" || speaker === "both")}
      />
    </div>
  );
}

/* ── WitnessFigure ─────────────────────────────────────
 *
 * A single character portrait. Active state: full opacity + small
 * forward step + soft drop-shadow glow tinted to the character's
 * Goethe-palette color. Inactive: dim (~35%) + saturate-down +
 * subtle backward step. Submitting: gentle scale pulse on the
 * active witness(es).
 */

interface WitnessFigureProps {
  side: "left" | "right";
  name: "pichet" | "aletheios";
  active: boolean;
  submitting: boolean;
}

/* ── BannerWitness ────────────────────────────────────
 *
 * Compact portrait used by the "banner" variant. Same active/dim
 * semantics as WitnessFigure but sized at 160px tall, placed inline
 * with horizontal padding rather than positioned to the viewport edge.
 */

function BannerWitness({ side, name, active, submitting }: WitnessFigureProps) {
  const opacity = active ? 1 : 0.4;
  const translateX = active
    ? side === "left" ? "4px" : "-4px"
    : side === "left" ? "-6px" : "6px";
  const glowColor = name === "pichet" ? "rgba(197,160,23,0.45)" : "rgba(16,181,167,0.45)";
  const filter = active
    ? `drop-shadow(0 0 14px ${glowColor})`
    : "saturate(0.45) brightness(0.7)";

  return (
    <div
      style={{
        position: "relative",
        height: 160,
        width: 120,
        margin: side === "left" ? "0 7rem 0 0" : "0 0 0 7rem",
        backgroundImage: `url(/depth-reading/characters/${name}-front.png)`,
        backgroundSize: "contain",
        backgroundRepeat: "no-repeat",
        backgroundPosition: side === "left" ? "left center" : "right center",
        transform: `translateX(${translateX})`,
        opacity,
        filter,
        transition:
          "opacity 500ms cubic-bezier(0.2,0.7,0.2,1), transform 500ms cubic-bezier(0.2,0.7,0.2,1), filter 500ms cubic-bezier(0.2,0.7,0.2,1)",
        pointerEvents: "none",
        animation: submitting ? "thresholdPulse 1.2s ease-in-out infinite" : undefined,
      }}
    />
  );
}

export function WitnessFigure({ side, name, active, submitting }: WitnessFigureProps) {
  /* Use the canonical front-facing portraits. The side-direction PNGs
     (*-left, *-right, *-back) are inconsistent — some are profile views
     of the same character, some are entirely different art assets.
     The *-front.png views are the canonical Pichet/Aletheios renders
     and both are already naturally posed gesturing inward, so two
     front views read as a dyad facing each other. */
  const imageUrl = `/depth-reading/characters/${name}-front.png`;

  const opacity = active ? 1 : 0.42;

  /* Active witness leans into the room; inactive recedes toward the edge.
     Combined with the rotateY below this reads as physical orientation,
     not a CSS scale-up. */
  const translateX = active
    ? side === "left" ? "12px" : "-12px"
    : side === "left" ? "-22px" : "22px";

  /* Very subtle faux-3D rotation. The portraits already include their
     own body angle in the art, so the CSS rotateY is just a tiny
     reinforcement (~1.5° toward center when active, slightly away
     when inactive). Anything stronger reads as "tilted trading card"
     because the PNG itself stays flat. */
  const rotateY = active
    ? side === "left" ? "1.5deg" : "-1.5deg"
    : side === "left" ? "-4deg" : "4deg";

  /* Goethe-tinted key light cast inward + ambient drop for ground feel.
     Pichet (left) casts gold to the right; Aletheios (right) casts
     emerald to the left. Both characters also drop a soft ambient
     shadow below them so they sit in space, not float. */
  const keyLight =
    name === "pichet"
      ? "drop-shadow(8px 4px 28px rgba(197,160,23,0.45))"
      : "drop-shadow(-8px 4px 28px rgba(16,181,167,0.45))";
  const ambient = "drop-shadow(0 26px 36px rgba(0,0,0,0.55))";

  const filter = active
    ? `${keyLight} ${ambient}`
    : `saturate(0.35) brightness(0.55) ${ambient}`;

  return (
    <div
      style={{
        position: "absolute",
        top: "50%",
        [side]: "clamp(0px, 4vw, 6rem)",
        /* Sized so two witnesses + center column fit cleanly at 1920px:
           each ≈ 420px wide leaves ~1080px clear horizontal channel
           for the form (vs. 770px at the previous 62vh height). */
        height: "clamp(42vh, 55vh, 70vh)",
        width: "auto",
        aspectRatio: "1 / 1.4",
        backgroundImage: `url(${imageUrl})`,
        backgroundSize: "contain",
        backgroundRepeat: "no-repeat",
        backgroundPosition: side === "left" ? "left center" : "right center",
        opacity,
        filter,
        /* perspective() turns rotateY into foreshortening — characters
           feel like they're standing in a room rather than pasted
           against the screen. transformOrigin anchored to the edge so
           the rotation hinges on their outside foot. */
        transform: `translateY(-50%) translateX(${translateX}) perspective(1400px) rotateY(${rotateY})`,
        transformOrigin: side === "left" ? "left center" : "right center",
        transition:
          "opacity 700ms cubic-bezier(0.2,0.7,0.2,1), transform 900ms cubic-bezier(0.2,0.7,0.2,1), filter 700ms cubic-bezier(0.2,0.7,0.2,1)",
        pointerEvents: "none",
        animation: submitting ? "thresholdPulse 1.2s ease-in-out infinite" : undefined,
      }}
    />
  );
}
