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

      {/* Click-modal — placeholder text-reveal pending GSAP inspiration */}
      {open && (
        <SectionModal
          section={open}
          prose={openProse}
          onClose={() => setOpenSectionId(null)}
        />
      )}
    </div>
  );
}

function ActiveSectionLabel({
  section,
  onOpen,
}: {
  section: SectionData;
  onOpen: () => void;
}) {
  return (
    <div
      style={{
        position: "absolute",
        left: "50%",
        bottom: "clamp(6rem, 12vh, 9rem)",
        transform: "translateX(-50%)",
        maxWidth: "min(36rem, 80vw)",
        textAlign: "center",
        zIndex: 5,
        transition: "opacity 0.4s ease",
      }}
    >
      <div
        style={{
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "clamp(0.7rem, 0.85vw, 0.85rem)",
          letterSpacing: "0.45em",
          color: section.accentColor,
          textTransform: "uppercase",
          marginBottom: "0.85rem",
          opacity: 0.85,
        }}
      >
        {section.numeral} · {section.direction ?? "WITNESS"}
      </div>
      <h2
        style={{
          margin: 0,
          fontFamily: "var(--font-display, 'Panchang', serif)",
          fontWeight: 700,
          fontSize: "clamp(1.75rem, 3vw, 3rem)",
          letterSpacing: "0.01em",
          lineHeight: 1.05,
          color: "var(--c-parchment, #F0EDE3)",
          textShadow: "0 2px 24px rgba(0,0,0,0.55)",
        }}
      >
        {section.title}
      </h2>
      <p
        style={{
          margin: "clamp(0.6rem, 1vw, 1rem) 0 1.4rem",
          fontFamily: "var(--font-display, 'Panchang', serif)",
          fontStyle: "italic",
          fontWeight: 500,
          fontSize: "clamp(0.95rem, 1.05vw, 1.15rem)",
          lineHeight: 1.5,
          color: "rgba(240,237,227,0.78)",
          textShadow: "0 2px 16px rgba(0,0,0,0.5)",
        }}
      >
        {section.summary}
      </p>
      <button
        onClick={onOpen}
        style={{
          background: "transparent",
          border: `1px solid ${section.accentColor}`,
          color: section.accentColor,
          padding: "clamp(0.5rem, 0.8vw, 0.75rem) clamp(1.4rem, 2vw, 2rem)",
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "0.75rem",
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
        Read
      </button>
    </div>
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

/** Modal that opens on plane click. Placeholder for now — full GSAP
 *  text-reveal lands once user provides inspiration. */
function SectionModal({
  section,
  prose,
  onClose,
}: {
  section: SectionData;
  prose: string;
  onClose: () => void;
}) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    document.body.style.overflow = "hidden";
    return () => {
      window.removeEventListener("keydown", handler);
      document.body.style.overflow = "";
    };
  }, [onClose]);

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 100,
        background: "rgba(7,11,29,0.92)",
        backdropFilter: "blur(12px)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "clamp(1rem, 3vw, 3rem)",
      }}
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          maxWidth: "min(56rem, 90vw)",
          maxHeight: "80vh",
          overflowY: "auto",
          background: "var(--c-void, #070B1D)",
          border: `1px solid ${section.accentColor}55`,
          borderRadius: "8px",
          padding: "clamp(1.5rem, 3vw, 3rem)",
          position: "relative",
          boxShadow: `0 0 80px ${section.accentColor}33`,
        }}
      >
        <button
          onClick={onClose}
          aria-label="Close"
          style={{
            position: "absolute",
            top: "1rem",
            right: "1rem",
            background: "transparent",
            border: "none",
            color: "var(--c-parchment, #F0EDE3)",
            fontSize: "1.5rem",
            cursor: "pointer",
            opacity: 0.6,
          }}
        >
          ×
        </button>
        <div
          style={{
            fontFamily: "var(--font-mono, 'SF Mono', monospace)",
            fontSize: "0.75rem",
            letterSpacing: "0.45em",
            color: section.accentColor,
            textTransform: "uppercase",
            marginBottom: "1rem",
          }}
        >
          {section.numeral} · {section.direction ?? "WITNESS"}
        </div>
        <h1
          style={{
            margin: "0 0 1rem",
            fontFamily: "var(--font-display, 'Panchang', serif)",
            fontWeight: 700,
            fontSize: "clamp(1.75rem, 3vw, 2.75rem)",
            color: "var(--c-parchment, #F0EDE3)",
          }}
        >
          {section.title}
        </h1>
        <p
          style={{
            fontStyle: "italic",
            color: "rgba(240,237,227,0.7)",
            marginBottom: "2rem",
          }}
        >
          {section.summary}
        </p>
        {/* TODO: Replace with GSAP text-reveal (3-4 sentence highlight scroll
            per user spec) once inspiration lands. For now: plain prose. */}
        <div
          style={{
            fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
            fontSize: "clamp(1rem, 1.05vw, 1.15rem)",
            lineHeight: 1.7,
            color: "var(--c-parchment, #F0EDE3)",
            whiteSpace: "pre-wrap",
          }}
        >
          {prose || (
            <em style={{ opacity: 0.5 }}>
              [Prose for this section will appear here once the soloLoader is wired.]
            </em>
          )}
        </div>
      </div>
    </div>
  );
}
