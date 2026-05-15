"use client";

// ─── EngineDrillDown — slide-in panel for a single engine ───────────────
// Per design v2 § 5.10. When a verse references a specific engine,
// tapping the bolded term opens this panel, which dispatches to the
// matching engine component in apps/noesis-web/src/components/engines/.
//
// Layout:
//   - Backdrop scrim (click-to-close)
//   - 480px wide right-edge panel (100vw on mobile)
//   - Header: engine name in Panchang + close X
//   - Body: <renderEngine(engineId, result)> or empty placeholder
//   - Esc key closes
//
// Animation: motion/react AnimatePresence — slide from right + scrim fade.

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useEffect } from "react";

import Panchanga from "@/components/engines/Panchanga";
import HumanDesign from "@/components/engines/HumanDesign";
import GeneKeys from "@/components/engines/GeneKeys";
import Vimshottari from "@/components/engines/Vimshottari";
import Numerology from "@/components/engines/Numerology";
import Biorhythm from "@/components/engines/Biorhythm";
import Tarot from "@/components/engines/Tarot";
import IChing from "@/components/engines/IChing";
import Transits from "@/components/engines/Transits";
import BiofieldView from "@/components/engines/Biofield";
import VedicClock from "@/components/engines/VedicClock";
import SacredGeometry from "@/components/engines/SacredGeometry";
import SigilForge from "@/components/engines/SigilForge";
import Enneagram from "@/components/engines/Enneagram";
import Nadabrahman from "@/components/engines/Nadabrahman";
import FaceReading from "@/components/engines/FaceReading";
import RaagaView from "@/components/engines/Raaga";
import GenericEngineView from "@/components/engines/GenericEngineView";

// ─── helpers ────────────────────────────────────────────────────────────

function engineLabel(id: string): string {
  return id
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

function renderEngine(id: string, result: Record<string, unknown>) {
  switch (id) {
    case "panchanga":
      return <Panchanga result={result} />;
    case "human-design":
      return <HumanDesign result={result} />;
    case "gene-keys":
      return <GeneKeys result={result} />;
    case "vimshottari":
      return <Vimshottari result={result} />;
    case "numerology":
      return <Numerology result={result} />;
    case "biorhythm":
      return <Biorhythm result={result} />;
    case "vedic-clock":
      return <VedicClock result={result} />;
    case "transits":
      return <Transits result={result} />;
    case "biofield":
      return <BiofieldView result={result} />;
    case "tarot":
      return <Tarot result={result} />;
    case "i-ching":
      return <IChing result={result} />;
    case "sacred-geometry":
      return <SacredGeometry result={result} />;
    case "sigil-forge":
      return <SigilForge result={result} />;
    case "enneagram":
      return <Enneagram result={result} />;
    case "nadabrahman":
      return <Nadabrahman result={result} />;
    case "face-reading":
      return <FaceReading result={result} />;
    case "raaga":
      return <RaagaView result={result} />;
    default:
      return <GenericEngineView result={result} />;
  }
}

// ─── styles ─────────────────────────────────────────────────────────────

const s = {
  scrim: {
    position: "fixed" as const,
    inset: 0,
    background: "rgba(7, 11, 29, 0.62)",
    backdropFilter: "blur(4px)",
    zIndex: 80,
  },
  panel: {
    position: "fixed" as const,
    top: 0,
    right: 0,
    bottom: 0,
    width: "min(480px, 100vw)",
    background:
      "linear-gradient(180deg, rgba(7,11,29,0.98) 0%, rgba(7,11,29,1) 100%)",
    borderLeft: "1px solid var(--line-strong, rgba(255,255,255,0.12))",
    boxShadow:
      "-12px 0 32px rgba(0,0,0,0.55), inset 1px 0 0 rgba(255,255,255,0.04)",
    zIndex: 81,
    display: "flex",
    flexDirection: "column" as const,
    color: "var(--text)",
  },
  header: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: "1rem",
    padding: "1.25rem 1.5rem",
    borderBottom: "1px solid var(--line-faint, rgba(255,255,255,0.08))",
    flexShrink: 0,
  },
  eyebrow: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.65rem",
    letterSpacing: "0.45em",
    textTransform: "uppercase" as const,
    color: "var(--c-gold, #d8b56e)",
    marginBottom: "0.35rem",
  },
  title: {
    fontFamily: "var(--font-display)",
    fontSize: "1.45rem",
    fontWeight: 800,
    color: "var(--c-parchment, #f3ead8)",
    margin: 0,
    letterSpacing: "-0.012em",
  },
  closeBtn: {
    background: "none",
    border: "1px solid var(--line, rgba(255,255,255,0.18))",
    borderRadius: "var(--r-sm, 6px)",
    color: "var(--text-muted, rgba(255,255,255,0.7))",
    cursor: "pointer",
    width: 36,
    height: 36,
    fontSize: "1.1rem",
    lineHeight: 1,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    fontFamily: "var(--font-body)",
    flexShrink: 0,
    transition: "all 0.18s ease",
  },
  body: {
    flex: 1,
    overflowY: "auto" as const,
    padding: "1.25rem 1.5rem 2rem",
  },
  empty: {
    padding: "2rem 0",
    color: "var(--text-dim, rgba(255,255,255,0.55))",
    fontStyle: "italic" as const,
    fontSize: "0.92rem",
    textAlign: "center" as const,
    lineHeight: 1.6,
  },
};

// ─── component ──────────────────────────────────────────────────────────

interface EngineDrillDownProps {
  engineId: string | null;
  result?: Record<string, unknown>;
  onClose: () => void;
}

export function EngineDrillDown({
  engineId,
  result,
  onClose,
}: EngineDrillDownProps) {
  const reduced = useReducedMotion();
  const open = engineId != null;

  // Esc key closes
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  // Lock body scroll while panel is open
  useEffect(() => {
    if (!open) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = prev;
    };
  }, [open]);

  return (
    <AnimatePresence>
      {open && engineId && (
        <>
          <motion.div
            key="drilldown-scrim"
            style={s.scrim}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: reduced ? 0 : 0.22, ease: [0.2, 0.7, 0.2, 1] }}
            onClick={onClose}
            aria-hidden
          />
          <motion.aside
            key="drilldown-panel"
            style={s.panel}
            role="dialog"
            aria-modal="true"
            aria-label={`${engineLabel(engineId)} drilldown`}
            initial={reduced ? { opacity: 0 } : { x: "100%", opacity: 0.9 }}
            animate={reduced ? { opacity: 1 } : { x: 0, opacity: 1 }}
            exit={reduced ? { opacity: 0 } : { x: "100%", opacity: 0.85 }}
            transition={{
              duration: reduced ? 0 : 0.32,
              ease: [0.2, 0.7, 0.2, 1],
            }}
          >
            <header style={s.header}>
              <div>
                <div style={s.eyebrow}>Engine</div>
                <h2 style={s.title}>{engineLabel(engineId)}</h2>
              </div>
              <button
                type="button"
                style={s.closeBtn}
                onClick={onClose}
                aria-label="Close drilldown"
              >
                ✕
              </button>
            </header>
            <div style={s.body}>
              {result ? (
                renderEngine(engineId, result)
              ) : (
                <p style={s.empty}>
                  No engine output recorded for this reading.
                  <br />
                  This view becomes active once the underlying engine has run.
                </p>
              )}
            </div>
          </motion.aside>
        </>
      )}
    </AnimatePresence>
  );
}
