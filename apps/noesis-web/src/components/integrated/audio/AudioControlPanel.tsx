"use client";

// ─── AudioControlPanel ─────────────────────────────────────────────────
// Bottom-right fixed glass widget:
//   - Mute toggle (speaker / speaker-muted)
//   - Chapter indicator (e.g. "I · STABILIZE")
//   - Volume slider, collapsed by default, expands on hover.
//
// Glass effect: backdrop-filter blur + inset-glow, gold accents.

import { useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import { useAudioState, CHAPTER_LABELS } from "./AudioState";

// Inline SVG icons — keep the bundle small and avoid icon-package adds.
function SpeakerIcon({ muted, size = 16 }: { muted: boolean; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.6}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M3 10v4a1 1 0 0 0 1 1h3l5 4V5L7 9H4a1 1 0 0 0-1 1Z" />
      {muted ? (
        <>
          <line x1="16" y1="9" x2="22" y2="15" />
          <line x1="22" y1="9" x2="16" y2="15" />
        </>
      ) : (
        <>
          <path d="M16 8.5a5 5 0 0 1 0 7" />
          <path d="M19 5.5a9 9 0 0 1 0 13" />
        </>
      )}
    </svg>
  );
}

export function AudioControlPanel() {
  const { chapter, muted, volume, running, toggleMute, setVolume } = useAudioState();
  const [expanded, setExpanded] = useState(false);

  const label = CHAPTER_LABELS[chapter];

  return (
    <div
      onMouseEnter={() => setExpanded(true)}
      onMouseLeave={() => setExpanded(false)}
      onFocus={() => setExpanded(true)}
      onBlur={() => setExpanded(false)}
      style={{
        position: "fixed",
        right: "clamp(0.75rem, 1.4vw, 1.5rem)",
        bottom: "clamp(0.75rem, 1.4vw, 1.5rem)",
        zIndex: 50,
        display: "flex",
        alignItems: "center",
        gap: "0.6rem",
        padding: "0.45rem 0.7rem",
        borderRadius: 999,
        background: "rgba(7, 11, 29, 0.55)",
        backdropFilter: "blur(10px) saturate(140%)",
        WebkitBackdropFilter: "blur(10px) saturate(140%)",
        border: "1px solid var(--line-gold, rgba(197,160,23,0.4))",
        boxShadow:
          "var(--inset-glow, inset 0 1px 0 rgba(255,255,255,0.07)), 0 4px 18px rgba(0,0,0,0.35)",
        color: "var(--c-gold, #C5A017)",
        fontFamily: "var(--font-mono, ui-monospace)",
        fontSize: "0.72rem",
        letterSpacing: "0.18em",
        userSelect: "none",
      }}
    >
      <button
        type="button"
        onClick={toggleMute}
        aria-label={muted ? "Unmute ambient drone" : "Mute ambient drone"}
        aria-pressed={muted}
        title={
          running
            ? muted
              ? "Audio muted — click to unmute"
              : "Audio playing — click to mute"
            : "Click anywhere to start audio · then unmute"
        }
        style={{
          appearance: "none",
          background: muted
            ? "rgba(197, 160, 23, 0.06)"
            : "rgba(197, 160, 23, 0.16)",
          border: "1px solid var(--line-gold, rgba(197,160,23,0.4))",
          width: 30,
          height: 30,
          borderRadius: "50%",
          color: "var(--c-gold, #C5A017)",
          display: "inline-flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: "pointer",
          padding: 0,
          transition: "background 200ms ease, transform 200ms ease",
        }}
      >
        <SpeakerIcon muted={muted} />
      </button>

      <span
        aria-live="polite"
        style={{
          fontWeight: 600,
          color: "var(--c-gold, #C5A017)",
          minWidth: "9ch",
          textAlign: "center",
        }}
      >
        {label}
      </span>

      <AnimatePresence initial={false}>
        {expanded && (
          <motion.div
            key="volume"
            initial={{ width: 0, opacity: 0 }}
            animate={{ width: 80, opacity: 1 }}
            exit={{ width: 0, opacity: 0 }}
            transition={{ duration: 0.28, ease: [0.2, 0.7, 0.2, 1] }}
            style={{ overflow: "hidden", display: "flex", alignItems: "center" }}
          >
            <input
              type="range"
              min={0}
              max={0.3}
              step={0.005}
              value={volume}
              onChange={(e) => setVolume(parseFloat(e.target.value))}
              aria-label="Ambient volume"
              title={`Volume ${(volume * 100).toFixed(0)}%`}
              style={{
                width: 76,
                accentColor: "var(--c-gold, #C5A017)",
                cursor: "pointer",
              }}
            />
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
