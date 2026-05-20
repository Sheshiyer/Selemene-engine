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
 * Witness poses. The portrait PNGs come in three usable angles per
 * character; the consuming flow chooses which to render per step so
 * the dyad feels alive across the ritual instead of static.
 *
 * Asset availability:
 *   - Pichet:    front, left, right   (3 native poses)
 *   - Aletheios: front, right         (2 native poses; left is not
 *                                      shipped — the *-left.png slot
 *                                      was a different-project leak)
 *
 * Callers that ask for an unavailable pose silently fall back to
 * "front" so the page never 404s on a portrait.
 */
export type WitnessPose = "front" | "left" | "right";

const AVAILABLE_POSES: Record<"pichet" | "aletheios", WitnessPose[]> = {
  pichet: ["front", "left", "right"],
  aletheios: ["front", "right"],
};

function resolvePose(name: "pichet" | "aletheios", pose: WitnessPose | undefined): WitnessPose {
  if (!pose) return "front";
  return AVAILABLE_POSES[name].includes(pose) ? pose : "front";
}

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
   * Pose override per witness. Consuming flow can rotate poses
   * across steps so the dyad feels physically alive instead of
   * static. Defaults to "front" if omitted or unavailable.
   */
  pichetPose?: WitnessPose;
  aletheiosPose?: WitnessPose;
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
  pichetPose,
  aletheiosPose,
  className,
}: DyadChamberProps) {
  const pichetActive = speaker === "pichet" || speaker === "both";
  const aletheiosActive = speaker === "aletheios" || speaker === "both";
  const resolvedPichetPose = resolvePose("pichet", pichetPose);
  const resolvedAletheiosPose = resolvePose("aletheios", aletheiosPose);

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
        pose={resolvedPichetPose}
        active={pichetActive}
        submitting={submitting && (speaker === "pichet" || speaker === "both")}
      />
      <WitnessFigure
        side="right"
        name="aletheios"
        pose={resolvedAletheiosPose}
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
  /** Pose to render. Optional; the dyad chamber resolves it before calling. */
  pose?: WitnessPose;
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

export function WitnessFigure({ side, name, active, submitting, pose }: WitnessFigureProps) {
  /* Pose is resolved by DyadChamber (falls back to "front" if a caller
     passes an unavailable angle). Pichet has front/left/right native
     poses; Aletheios has front/right. Rotating poses across steps gives
     the dyad visible motion across the ritual. */
  const resolvedPose = resolvePose(name, pose);
  const imageUrl = `/depth-reading/characters/${name}-${resolvedPose}.png`;

  const opacity = active ? 1 : 0.55;

  /* Active witness leans into the room; inactive recedes toward the edge.
     The translateX stays subtle so the pose-swap (front ↔ right) carries
     most of the "this witness just shifted attention" signal. */
  const translateX = active
    ? side === "left" ? "10px" : "-10px"
    : side === "left" ? "-18px" : "18px";

  /* Very subtle faux-3D rotation. The PNG itself stays flat so anything
     stronger reads as "tilted trading card." Keep this low. */
  const rotateY = active
    ? side === "left" ? "1.5deg" : "-1.5deg"
    : side === "left" ? "-4deg" : "4deg";

  /* The portraits ship with real alpha channels now (chroma-keyed
     against the noesis void color in tools/alpha_witnesses.py), so
     we don't need mix-blend-mode tricks — drop-shadow would just
     pile on. The character silhouette IS the figure. Apply only
     subtle saturation/brightness for the active/inactive state. */
  const filter = active
    ? "saturate(1.15) brightness(1.06) contrast(1.02)"
    : "saturate(0.35) brightness(0.55) blur(0.5px)";

  return (
    <div
      style={{
        position: "absolute",
        top: "50%",
        [side]: "clamp(0px, 2vw, 3rem)",
        /* ~80vh per user request — fills the chamber. With
           mix-blend-mode: screen the black PNG bg disappears so
           "bigger" no longer means "more rectangle in the user's
           face," only "more character." */
        height: "clamp(58vh, 80vh, 92vh)",
        width: "auto",
        aspectRatio: "1 / 1.4",
        backgroundImage: `url(${imageUrl})`,
        backgroundSize: "contain",
        backgroundRepeat: "no-repeat",
        backgroundPosition: side === "left" ? "left center" : "right center",
        opacity,
        filter,
        /* No mix-blend-mode — the PNGs now have real alpha channels
           (chroma-keyed in tools/alpha_witnesses.py) so character art
           composites cleanly against the new fluid backdrop. */
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
