"use client";

// ─── Constellation Grid — field cartography backdrop ────────────────────
// Fixed-position SVG over the viewport. Hairline Sacred Gold dots in a
// sparse drifting mesh + a few connecting hairlines. Drifts via motion
// (continuous rotate). Hides below 720px viewport via CSS.
//
// Per design MD § 3.8.

import { motion } from "motion/react";
import type { ReactElement } from "react";

const ROWS = 9;
const COLS = 11;

function buildDots(): ReactElement[] {
  const dots: ReactElement[] = [];
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const offsetX = (r % 2 === 0) ? 0 : 50;
      const baseX = (c * 100) + offsetX + 30;
      const baseY = (r * 110) + 40;
      const jx = ((r * 7 + c * 13) % 17) - 8;
      const jy = ((r * 11 + c * 3) % 13) - 6;
      const x = baseX + jx;
      const y = baseY + jy;
      const op = 0.18 + (((r + c) % 3) * 0.06);
      dots.push(
        <circle
          key={`${r}-${c}`}
          cx={x}
          cy={y}
          r={1.1}
          fill="var(--c-gold)"
          opacity={op}
        />,
      );
    }
  }
  return dots;
}

const CONSTELLATION_LINES = [
  "M 130 60 L 380 280 L 520 180 L 720 350",
  "M 80 600 L 280 540 L 410 720 L 660 660 L 880 800",
  "M 220 920 L 470 880 L 690 970",
];

export function ConstellationGrid() {
  return (
    <motion.svg
      aria-hidden="true"
      viewBox="0 0 1100 1000"
      preserveAspectRatio="xMidYMid slice"
      style={{
        position: "fixed",
        inset: 0,
        width: "100vw",
        height: "100vh",
        zIndex: 1,
        pointerEvents: "none",
        opacity: 0.55,
        transformOrigin: "50% 50%",
      }}
      animate={{ rotate: 360 }}
      transition={{ duration: 240, ease: "linear", repeat: Infinity }}
      className="constellation-grid"
    >
      {CONSTELLATION_LINES.map((d, i) => (
        <path
          key={`line-${i}`}
          d={d}
          stroke="var(--c-gold)"
          strokeWidth={0.4}
          fill="none"
          opacity={0.15}
        />
      ))}
      {buildDots()}
    </motion.svg>
  );
}
