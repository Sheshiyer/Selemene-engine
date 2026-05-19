"use client";

/**
 * /preview/engine-grid — dev preview route for the hex honeycomb.
 *
 * Mounts EngineGrid with the canonical 17 engines so we can verify
 * the visual against the T1B-03 mockup without needing an authenticated
 * workflow run. Not linked from anywhere in production navigation.
 */

import { useState } from "react";
import NavBar from "@/components/NavBar";
import EngineGrid, { type EngineGridItem } from "@/components/EngineGrid";

const SEVENTEEN: EngineGridItem[] = [
  // row 1 (3 cells, offset)
  { id: "panchanga",       label: "Panchanga",       sigil: "☽", direction: "stabilize" },
  { id: "human-design",    label: "Human Design",    sigil: "⬡", direction: "mutate" },
  { id: "gene-keys",       label: "Gene Keys",       sigil: "✦", direction: "mutate" },
  // row 2 (4 cells)
  { id: "vimshottari",     label: "Vimshottari",     sigil: "◌", direction: "stabilize" },
  { id: "numerology",      label: "Numerology",      sigil: "9", direction: "mutate" },
  { id: "biorhythm",       label: "Biorhythm",       sigil: "∞", direction: "heal" },
  { id: "vedic-clock",     label: "Vedic Clock",     sigil: "◷", direction: "heal" },
  // row 3 (3 cells, offset) — index 8 is the WITNESS hub center
  { id: "transits",        label: "Transits",        sigil: "☍", direction: "stabilize" },
  { id: "biofield",        label: "Biofield",        sigil: "◎", direction: "heal" },
  { id: "tarot",           label: "Tarot",           sigil: "▯", direction: "mutate" },
  // row 4 (4 cells)
  { id: "i-ching",         label: "I-Ching",         sigil: "☷", direction: "mutate" },
  { id: "sacred-geometry", label: "Sacred Geometry", sigil: "✺", direction: "create" },
  { id: "sigil-forge",     label: "Sigil Forge",     sigil: "⌁", direction: "create" },
  { id: "enneagram",       label: "Enneagram",       sigil: "✶", direction: "mutate" },
  // row 5 (3 cells, offset)
  { id: "nadabrahman",     label: "Nadabrahman",     sigil: "ॐ", direction: "create" },
  { id: "face-reading",    label: "Face Reading",    sigil: "◉", direction: "mutate" },
  { id: "raaga",           label: "Raaga",           sigil: "♪", direction: "create" },
];

// Pretend a few engines have already returned data
const AVAILABLE = new Set<string>([
  "panchanga",
  "vimshottari",
  "biofield",
  "human-design",
  "i-ching",
  "sigil-forge",
  "tarot",
]);

export default function EngineGridPreview() {
  const [active, setActive] = useState<string>("biofield");

  return (
    <div style={{ minHeight: "100vh", background: "var(--bg)" }}>
      <NavBar />
      <main
        style={{
          maxWidth: 980,
          margin: "0 auto",
          padding: "2rem 1.5rem",
          display: "flex",
          flexDirection: "column",
          gap: "1.5rem",
        }}
      >
        <header style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "0.7rem",
              letterSpacing: "0.12em",
              color: "var(--muted)",
              textTransform: "uppercase",
            }}
          >
            DEV PREVIEW · NOT FOR PRODUCTION
          </span>
          <h1
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "1.6rem",
              color: "var(--signal)",
              letterSpacing: "0.06em",
              margin: 0,
            }}
          >
            Engine Honeycomb
          </h1>
          <p
            style={{
              color: "var(--text-2)",
              fontSize: "0.9rem",
              lineHeight: 1.6,
              maxWidth: 640,
              margin: 0,
            }}
          >
            17 engines in a staggered hex grid. Center cell is the WITNESS
            hub. Cells with returned data show a coherence-emerald pulse.
            Click a cell to set the active engine.
          </p>
        </header>

        <EngineGrid
          engines={SEVENTEEN}
          activeEngineId={active}
          availableEngineIds={AVAILABLE}
          onSelect={setActive}
        />

        <footer
          style={{
            marginTop: "1rem",
            padding: "0.875rem 1rem",
            border: "1px solid var(--line-mid)",
            borderRadius: "var(--r-sm)",
            background: "rgba(11,80,251,0.04)",
            fontFamily: "var(--font-mono)",
            fontSize: "0.78rem",
            color: "var(--text-2)",
            display: "flex",
            gap: "1.25rem",
            flexWrap: "wrap",
          }}
        >
          <span>
            <strong style={{ color: "var(--signal)" }}>Active:</strong> {active}
          </span>
          <span>
            <strong style={{ color: "var(--c-emerald)" }}>Data ready:</strong>{" "}
            {Array.from(AVAILABLE).join(", ")}
          </span>
        </footer>
      </main>
    </div>
  );
}
