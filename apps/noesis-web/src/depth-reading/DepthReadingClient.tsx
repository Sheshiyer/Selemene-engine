"use client";

// ─── DepthReadingClient — React mount for the DepthScene ──────────────
// Mounts the vanilla-Three.js DepthScene into a canvas, tracks the
// "active plane" as the reader scrolls, and renders DOM overlays:
//   • Per-section label (Roman numeral + title + 1-2 line summary)
//     positioned over the active plane
//   • Click any plane → triggers onSectionOpen which the page renders
//     as a modal (placeholder for now; awaiting GSAP text-reveal spec)
//   • Floating scroll-cue (chevron) when at top
//   • Bottom progress dots showing position in the 15-section arc

import { useEffect, useRef, useState } from "react";
import { DepthScene } from "./DepthScene";
import { ProseReader } from "./ProseReader";
import type { SectionData } from "./data/sections";

interface DepthReadingClientProps {
  sections: SectionData[];
  /** Pre-loaded prose per section id, indexed for the modal */
  proseBySection: Record<string, string>;
}

export function DepthReadingClient({
  sections,
  proseBySection,
}: DepthReadingClientProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sceneRef = useRef<DepthScene | null>(null);
  const [activeSectionId, setActiveSectionId] = useState<string>(sections[0]?.id ?? "");
  const [openSectionId, setOpenSectionId] = useState<string | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const scene = new DepthScene({
      canvas,
      sections,
      onActivePlaneChange: (id) => setActiveSectionId(id),
      onPlaneClick: (id) => setOpenSectionId(id),
    });
    sceneRef.current = scene;
    scene.start();
    return () => {
      scene.dispose();
      sceneRef.current = null;
    };
  }, [sections]);

  const active = sections.find((s) => s.id === activeSectionId) ?? sections[0];
  const open = openSectionId ? sections.find((s) => s.id === openSectionId) : null;
  const openProse = open ? proseBySection[open.id] ?? "" : "";

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "var(--c-void, #070B1D)",
        color: "var(--c-parchment, #F0EDE3)",
        overflow: "hidden",
      }}
    >
      <canvas
        ref={canvasRef}
        style={{
          position: "absolute",
          inset: 0,
          width: "100%",
          height: "100%",
          touchAction: "none",
          display: "block",
        }}
      />

      {/* Active section label — title + summary float over the active plane */}
      <ActiveSectionLabel section={active} onOpen={() => setOpenSectionId(active.id)} />

      {/* Top mono band */}
      <header
        style={{
          position: "absolute",
          top: "clamp(1rem, 2.5vh, 2rem)",
          left: "50%",
          transform: "translateX(-50%)",
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "clamp(0.625rem, 0.85vw, 0.8rem)",
          letterSpacing: "0.4em",
          color: "var(--c-gold, #C5A017)",
          opacity: 0.78,
          textTransform: "uppercase",
          zIndex: 10,
          pointerEvents: "none",
        }}
      >
        TRYAMBAKAM · NOESIS · INTEGRATED READING
      </header>

      {/* Progress dots — 15 small dots; active glows gold */}
      <ProgressDots
        sections={sections}
        activeId={active.id}
        onJump={(id) => setOpenSectionId(id)}
      />

      {/* Bottom-right: scroll cue + section count */}
      <div
        style={{
          position: "absolute",
          bottom: "clamp(1rem, 2vh, 1.75rem)",
          right: "clamp(1rem, 2vw, 2rem)",
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "0.7rem",
          letterSpacing: "0.3em",
          color: "var(--c-gold, #C5A017)",
          opacity: 0.7,
          zIndex: 10,
          pointerEvents: "none",
        }}
      >
        ↓ SCROLL · {sections.findIndex((s) => s.id === active.id) + 1} / {sections.length}
      </div>

      {/* Click-modal — codrops-driven text reveal:
            • Headline: 3D char-stagger reveal on mount (OnScrollTextHighlight effect-1)
            • Body: 3-4 sentence focus zone with blur-to-sharp scrub
              (ScrollBlurTypography effect-2 inverted as IntersectionObserver) */}
      {open && (
        <ProseReader
          section={open}
          prose={openProse}
          onClose={() => setOpenSectionId(null)}
        />
      )}
    </div>
  );
}

/**
 * Codrops Label.js pattern adapted to our reading.
 *
 * One fixed full-viewport overlay that flanks the 3D canvas:
 *   LEFT column  (vertically centered, far-left, ~8vw inset)
 *     - Index (01, 02, … padded)
 *     - Big WORD / title (Panchang display — the visual focal point)
 *     - Section summary (one italic line)
 *     - Color chip (the section's accent)
 *     - "Read" button to open the prose modal
 *   RIGHT column (vertically centered, far-right, ~7vw inset)
 *     - Specs dl — { label: value } rows in monospace
 *
 * Mobile (<53em): both columns collapse to a 2-column bottom row.
 *
 * The 260ms cross-fade as the active plane changes — content swap is
 * masked by a brief opacity dip on the whole overlay.
 */
function ActiveSectionLabel({
  section,
  onOpen,
}: {
  section: SectionData;
  onOpen: () => void;
}) {
  const indexNumber = section.kind === "part"
    ? String(parseInt(section.numeral.replace(/[^IVXLCDM]/gi, "") || "1", 10)).padStart(2, "0")
    : section.numeral;
  return (
    <section
      className="depth-label-overlay"
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 5,
        pointerEvents: "none",
        color: "var(--c-parchment, #F0EDE3)",
        transition: "opacity 260ms ease",
        fontFamily: "var(--font-mono, 'SF Mono', monospace)",
        textTransform: "uppercase",
        letterSpacing: "0.08em",
        lineHeight: 1.2,
      }}
    >
      {/* ─── LEFT column — index + WORD + summary + chip + button ─── */}
      <div
        className="depth-label-overlay__left"
        style={{
          position: "absolute",
          left: "clamp(2.5rem, 8vw, 12rem)",
          top: "50%",
          transform: "translateY(-50%)",
          display: "grid",
          gap: "clamp(0.5rem, 1vw, 0.85rem)",
          maxWidth: "min(38rem, 42vw)",
          pointerEvents: "auto",
        }}
      >
        {/* Index — 01 / 02 / etc. */}
        <p
          style={{
            margin: 0,
            fontSize: "9px",
            opacity: 0.85,
          }}
        >
          {indexNumber} · {section.direction ?? "WITNESS"}
        </p>

        {/* The WORD — visual focal point of the whole interface */}
        <h1
          style={{
            margin: 0,
            fontFamily: "var(--font-display, 'Panchang', serif)",
            fontVariationSettings: "'wght' 720",
            fontSize: "clamp(2em, 5vw, 4em)",
            letterSpacing: "-0.01em",
            lineHeight: 0.95,
            textTransform: "none",
            color: "var(--c-parchment, #F0EDE3)",
            textShadow: "0 2px 32px rgba(0,0,0,0.65)",
          }}
        >
          {section.title}
        </h1>

        {/* Summary — one italic line under the word */}
        <p
          style={{
            margin: "0.25rem 0 0",
            maxWidth: "32ch",
            fontFamily: "var(--font-display, 'Panchang', serif)",
            fontStyle: "italic",
            fontVariationSettings: "'wght' 500",
            fontSize: "clamp(0.95em, 1.1vw, 1.1em)",
            lineHeight: 1.5,
            color: "rgba(240,237,227,0.78)",
            textShadow: "0 2px 14px rgba(0,0,0,0.55)",
            textTransform: "none",
            letterSpacing: 0,
          }}
        >
          {section.summary}
        </p>

        {/* Chip + Read button — paired in a tight row */}
        <div
          style={{
            marginTop: "clamp(0.4rem, 0.8vw, 0.75rem)",
            display: "flex",
            alignItems: "center",
            gap: "0.85rem",
          }}
        >
          <span
            aria-hidden="true"
            style={{
              width: "18px",
              height: "18px",
              borderRadius: "50%",
              display: "inline-block",
              boxShadow: "0 0 0 1px rgba(255,255,255,0.18)",
              background: section.accentColor,
            }}
          />
          <button
            onClick={onOpen}
            style={{
              background: "transparent",
              border: `1px solid ${section.accentColor}`,
              color: section.accentColor,
              padding: "clamp(0.45rem, 0.7vw, 0.65rem) clamp(1.2rem, 1.8vw, 1.6rem)",
              fontFamily: "var(--font-mono, 'SF Mono', monospace)",
              fontSize: "0.72rem",
              letterSpacing: "0.32em",
              textTransform: "uppercase",
              cursor: "pointer",
              borderRadius: "999px",
              transition: "all 0.2s ease",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = section.accentColor;
              e.currentTarget.style.color = "var(--c-void, #070B1D)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "transparent";
              e.currentTarget.style.color = section.accentColor;
            }}
          >
            Read full text
          </button>
        </div>
      </div>

      {/* ─── RIGHT column — specs dl (label : value rows) ─── */}
      {section.specs && section.specs.length > 0 && (
        <article
          className="depth-label-overlay__right"
          style={{
            position: "absolute",
            right: "clamp(2.5rem, 7vw, 10rem)",
            top: "50%",
            transform: "translateY(-50%)",
            width: "min(28vw, 320px)",
            opacity: 1,
            pointerEvents: "none",
          }}
        >
          <dl
            style={{
              margin: 0,
              display: "grid",
              gap: "0.5rem",
            }}
          >
            {section.specs.map((spec, i) => (
              <div
                key={`${spec.label}-${i}`}
                style={{
                  display: "grid",
                  gridTemplateColumns: "5.5rem 1fr",
                  alignItems: "baseline",
                  gap: "0.85rem",
                  paddingBottom: "0.5rem",
                  borderBottom: i < section.specs!.length - 1
                    ? "1px solid rgba(240,237,227,0.08)"
                    : "none",
                }}
              >
                <dt
                  style={{
                    margin: 0,
                    fontSize: "9px",
                    opacity: 0.6,
                  }}
                >
                  {spec.label}
                </dt>
                <dd
                  style={{
                    margin: 0,
                    fontSize: "clamp(9.5px, 0.75vw, 11.5px)",
                    color: section.accentColor,
                  }}
                >
                  {spec.value}
                </dd>
              </div>
            ))}
          </dl>
        </article>
      )}
    </section>
  );
}

function ProgressDots({
  sections,
  activeId,
  onJump,
}: {
  sections: SectionData[];
  activeId: string;
  onJump: (id: string) => void;
}) {
  return (
    <div
      style={{
        position: "absolute",
        left: "clamp(1rem, 2vw, 2rem)",
        top: "50%",
        transform: "translateY(-50%)",
        display: "flex",
        flexDirection: "column",
        gap: "0.55rem",
        zIndex: 10,
      }}
    >
      {sections.map((s) => {
        const isActive = s.id === activeId;
        return (
          <button
            key={s.id}
            onClick={() => onJump(s.id)}
            aria-label={`Jump to ${s.title}`}
            title={`${s.numeral} · ${s.title}`}
            style={{
              width: isActive ? "0.65rem" : "0.42rem",
              height: isActive ? "0.65rem" : "0.42rem",
              background: isActive
                ? s.accentColor
                : "rgba(240,237,227,0.25)",
              border: "none",
              padding: 0,
              borderRadius: "50%",
              cursor: "pointer",
              boxShadow: isActive
                ? `0 0 12px ${s.accentColor}aa`
                : "none",
              transition: "all 0.35s ease",
            }}
          />
        );
      })}
    </div>
  );
}

