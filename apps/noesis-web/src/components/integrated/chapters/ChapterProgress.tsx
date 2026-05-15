"use client";

// ─── ChapterProgress — floating bottom-left progress indicator ─────────
// Per integrated-reading-design-v2.md § 4 (chapter arc) — gives the
// reader an always-visible sense of where they are in the N-chapter
// story.
//
// • Fixed bottom-left, 40px from edges
// • Slim vertical bar with N segments (one per Part)
// • Segments fill in as user scrolls past each Part's #part-N anchor
// • Roman numeral label next to each segment
// • Click a segment → smooth-scroll to that Part
// • Sacred Gold styling with glass effect (backdrop-blur)
// • Hidden on small viewports (<480px) — would compete with content

import { useEffect, useState } from "react";

interface ChapterProgressProps {
  parts: Array<{ partNum: number; romanNumeral: string; title?: string }>;
}

export function ChapterProgress({ parts }: ChapterProgressProps) {
  // 0 = at top of reading, N = past last part. activePart = current Part
  // user is reading (1-indexed; 0 = pre-Parts).
  const [activePart, setActivePart] = useState(0);

  useEffect(() => {
    if (typeof window === "undefined") return;
    if (parts.length === 0) return;

    // Pick the Part whose top has crossed the 40%-from-top line.
    const computeActive = () => {
      const triggerY = window.innerHeight * 0.4;
      let best = 0;
      for (const p of parts) {
        const el = document.getElementById(`part-${p.partNum}`);
        if (!el) continue;
        const rect = el.getBoundingClientRect();
        if (rect.top <= triggerY) {
          best = p.partNum;
        }
      }
      setActivePart(best);
    };

    computeActive();
    let ticking = false;
    const onScroll = () => {
      if (ticking) return;
      ticking = true;
      requestAnimationFrame(() => {
        computeActive();
        ticking = false;
      });
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", computeActive);
    return () => {
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", computeActive);
    };
  }, [parts]);

  if (parts.length === 0) return null;

  const handleJump = (partNum: number) => {
    const el = document.getElementById(`part-${partNum}`);
    if (!el) return;
    el.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  return (
    <nav
      aria-label="Chapter progress"
      style={{
        position: "fixed",
        left: "40px",
        bottom: "40px",
        zIndex: 40,
        display: "flex",
        flexDirection: "column",
        gap: "0.65rem",
        padding: "0.85rem 0.65rem",
        background: "rgba(7, 11, 29, 0.55)",
        backdropFilter: "blur(8px)",
        WebkitBackdropFilter: "blur(8px)",
        border: "1px solid var(--line-mid)",
        borderRadius: "4px",
        boxShadow: "0 4px 24px rgba(0,0,0,0.35)",
      }}
      className="chapter-progress-nav"
    >
      {parts.map((p) => {
        const filled = p.partNum <= activePart;
        const isActive = p.partNum === activePart;
        return (
          <button
            key={p.partNum}
            type="button"
            onClick={() => handleJump(p.partNum)}
            aria-label={`Jump to Part ${p.romanNumeral}${p.title ? ` — ${p.title}` : ""}`}
            aria-current={isActive ? "true" : undefined}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "0.6rem",
              padding: "0.15rem 0.25rem",
              background: "transparent",
              border: "none",
              cursor: "pointer",
              opacity: filled ? 1 : 0.55,
              transition: "opacity 240ms ease",
            }}
          >
            {/* Segment bar */}
            <span
              aria-hidden="true"
              style={{
                width: isActive ? "4px" : "2px",
                height: "22px",
                background: filled ? "var(--c-gold)" : "var(--line-strong)",
                boxShadow: filled ? "var(--glow-gold)" : "none",
                borderRadius: "1px",
                transition: "all 240ms ease",
              }}
            />
            {/* Roman numeral */}
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.72rem",
                letterSpacing: "0.2em",
                color: isActive
                  ? "var(--c-gold)"
                  : filled
                    ? "var(--text-2)"
                    : "var(--muted)",
                minWidth: "1.8rem",
                textAlign: "left",
              }}
            >
              {p.romanNumeral}
            </span>
          </button>
        );
      })}
      <style>{`
        @media (max-width: 480px) {
          .chapter-progress-nav { display: none !important; }
        }
      `}</style>
    </nav>
  );
}
