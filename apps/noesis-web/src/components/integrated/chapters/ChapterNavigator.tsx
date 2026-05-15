"use client";

// ─── ChapterNavigator — slide-from-right chapter drawer ────────────────
// Per integrated-reading-design-v2.md § 4 — a more verbose companion to
// ChapterProgress. ChapterProgress is the "always-on" minimal indicator;
// ChapterNavigator is the "open-on-demand" full table of contents.
//
// • Hidden by default — a small "≡" handle at the right edge of the
//   viewport opens it
// • Drawer slides in from the right, ~22rem wide
// • Lists all Parts with Roman numeral + title
// • Optional threshold question at the top (Witness-layer hook)
// • Click a chapter to scroll to it; drawer closes after jump
// • Escape key closes; click-outside-the-panel closes
//
// Brand-aligned: Sacred Gold accents, Panchang display, SF Mono labels,
// Void Black surface with backdrop-blur (glass).

import { motion, AnimatePresence } from "motion/react";
import { useEffect, useState } from "react";

interface ChapterNavigatorProps {
  parts: Array<{ partNum: number; romanNumeral: string; title: string }>;
  /** Optional Witness-layer threshold question rendered above the list. */
  thresholdQuestion?: string;
}

export function ChapterNavigator({
  parts,
  thresholdQuestion,
}: ChapterNavigatorProps) {
  const [open, setOpen] = useState(false);

  // Escape closes drawer.
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  const handleJump = (partNum: number) => {
    const el = document.getElementById(`part-${partNum}`);
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "start" });
    }
    setOpen(false);
  };

  if (parts.length === 0) return null;

  return (
    <>
      {/* Right-edge handle — always visible. */}
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        aria-expanded={open}
        aria-controls="chapter-navigator-drawer"
        aria-label={open ? "Close chapter navigator" : "Open chapter navigator"}
        style={{
          position: "fixed",
          right: open ? "calc(22rem + 12px)" : "12px",
          top: "50%",
          transform: "translateY(-50%)",
          zIndex: 50,
          width: "32px",
          height: "64px",
          background: "rgba(7, 11, 29, 0.65)",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
          border: "1px solid var(--line-mid)",
          borderRadius: "4px 0 0 4px",
          color: "var(--c-gold)",
          cursor: "pointer",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontFamily: "var(--font-mono)",
          fontSize: "1.1rem",
          letterSpacing: "0",
          boxShadow: "0 4px 24px rgba(0,0,0,0.35)",
          transition: "right 320ms cubic-bezier(0.2, 0.7, 0.2, 1)",
        }}
        className="chapter-navigator-handle"
      >
        {open ? "×" : "≡"}
      </button>

      <AnimatePresence>
        {open && (
          <>
            {/* Scrim — click to close. */}
            <motion.button
              type="button"
              aria-label="Close chapter navigator"
              onClick={() => setOpen(false)}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.25 }}
              style={{
                position: "fixed",
                inset: 0,
                zIndex: 44,
                background: "rgba(7, 11, 29, 0.45)",
                backdropFilter: "blur(2px)",
                WebkitBackdropFilter: "blur(2px)",
                border: "none",
                cursor: "pointer",
              }}
            />

            {/* Drawer panel. */}
            <motion.aside
              id="chapter-navigator-drawer"
              role="dialog"
              aria-label="Chapter navigator"
              initial={{ x: "100%" }}
              animate={{ x: 0 }}
              exit={{ x: "100%" }}
              transition={{ duration: 0.42, ease: [0.2, 0.7, 0.2, 1] }}
              style={{
                position: "fixed",
                top: 0,
                right: 0,
                bottom: 0,
                width: "22rem",
                maxWidth: "100vw",
                zIndex: 45,
                background: "rgba(14, 20, 40, 0.94)",
                backdropFilter: "blur(12px)",
                WebkitBackdropFilter: "blur(12px)",
                borderLeft: "1px solid var(--line-strong)",
                boxShadow: "-12px 0 48px rgba(0,0,0,0.55)",
                overflowY: "auto",
                padding: "clamp(1.5rem, 3vh, 2.5rem) clamp(1.25rem, 2.4vw, 2rem)",
                display: "flex",
                flexDirection: "column",
                gap: "1.5rem",
              }}
            >
              <header
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.4rem",
                  paddingBottom: "1rem",
                  borderBottom: "1px solid var(--line-mid)",
                }}
              >
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.7rem",
                    letterSpacing: "0.4em",
                    textTransform: "uppercase",
                    color: "var(--c-gold)",
                  }}
                >
                  Chapters
                </div>
                <div
                  style={{
                    fontFamily: "var(--font-display)",
                    fontWeight: 700,
                    fontSize: "1.4rem",
                    color: "var(--c-parchment)",
                    letterSpacing: "-0.01em",
                  }}
                >
                  The Reading
                </div>
              </header>

              {thresholdQuestion ? (
                <section
                  style={{
                    padding: "1rem 1rem",
                    background: "rgba(45, 0, 80, 0.18)",
                    border: "1px solid var(--c-violet)",
                    borderRadius: "3px",
                  }}
                >
                  <div
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.65rem",
                      letterSpacing: "0.4em",
                      textTransform: "uppercase",
                      color: "var(--c-violet)",
                      marginBottom: "0.5rem",
                    }}
                  >
                    Threshold
                  </div>
                  <div
                    style={{
                      fontFamily: "var(--font-display)",
                      fontStyle: "italic",
                      fontWeight: 500,
                      fontSize: "0.98rem",
                      lineHeight: 1.45,
                      color: "var(--text)",
                    }}
                  >
                    {thresholdQuestion}
                  </div>
                </section>
              ) : null}

              <ol
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.4rem",
                }}
              >
                {parts.map((p) => (
                  <li key={p.partNum}>
                    <button
                      type="button"
                      onClick={() => handleJump(p.partNum)}
                      style={{
                        display: "flex",
                        alignItems: "baseline",
                        gap: "0.85rem",
                        width: "100%",
                        padding: "0.75rem 0.5rem",
                        background: "transparent",
                        border: "none",
                        borderTop: "1px solid var(--line-faint)",
                        textAlign: "left",
                        cursor: "pointer",
                        color: "var(--text)",
                        transition: "background 200ms ease",
                      }}
                      onMouseEnter={(e) => {
                        (e.currentTarget as HTMLButtonElement).style.background =
                          "rgba(197, 160, 23, 0.06)";
                      }}
                      onMouseLeave={(e) => {
                        (e.currentTarget as HTMLButtonElement).style.background =
                          "transparent";
                      }}
                    >
                      <span
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontSize: "0.75rem",
                          letterSpacing: "0.25em",
                          color: "var(--c-gold)",
                          minWidth: "2.4rem",
                        }}
                      >
                        {p.romanNumeral}
                      </span>
                      <span
                        style={{
                          fontFamily: "var(--font-display)",
                          fontWeight: 600,
                          fontSize: "1rem",
                          lineHeight: 1.25,
                          letterSpacing: "-0.005em",
                          color: "var(--c-parchment)",
                        }}
                      >
                        {p.title}
                      </span>
                    </button>
                  </li>
                ))}
              </ol>

              <footer
                style={{
                  marginTop: "auto",
                  paddingTop: "1rem",
                  borderTop: "1px solid var(--line-mid)",
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.65rem",
                  letterSpacing: "0.32em",
                  color: "var(--muted)",
                  textTransform: "uppercase",
                }}
              >
                Esc to close
              </footer>
            </motion.aside>
          </>
        )}
      </AnimatePresence>

      <style>{`
        @media (max-width: 480px) {
          .chapter-navigator-handle { right: ${open ? "calc(100vw - 32px - 12px)" : "8px"}; height: 56px; width: 28px; }
        }
      `}</style>
    </>
  );
}
