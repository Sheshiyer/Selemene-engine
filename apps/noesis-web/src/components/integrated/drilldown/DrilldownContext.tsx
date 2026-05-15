"use client";

// ─── DrilldownContext — share drill-down opener across the article tree ──
// Deep components (VerseFlow → EngineTermLink) need to open the EngineDrillDown
// panel that lives at the IntegratedReadingView root. Rather than thread
// callbacks through every intermediate, we provide a React context.
//
// Per design v2 § 5.10 — a single slide-in panel above the article z-index.

import { createContext, useContext } from "react";

export interface DrilldownTarget {
  engineId: string;
  /** Optional pre-fetched engine result. When absent, the panel shows the
   *  empty / "no data yet" placeholder for that engine. */
  result?: Record<string, unknown>;
}

export interface DrilldownContextValue {
  /** Open the drill-down panel for a given engine. */
  open: (target: DrilldownTarget) => void;
  /** Lookup table from engineId → existing engine result (for term-link clicks
   *  that don't carry their own payload). May be empty when no engine outputs
   *  are present in the loaded reading. */
  engineOutputs: Record<string, Record<string, unknown> | undefined>;
}

const noopContext: DrilldownContextValue = {
  open: () => {},
  engineOutputs: {},
};

export const DrilldownContext = createContext<DrilldownContextValue>(noopContext);

export function useDrilldown(): DrilldownContextValue {
  return useContext(DrilldownContext);
}
