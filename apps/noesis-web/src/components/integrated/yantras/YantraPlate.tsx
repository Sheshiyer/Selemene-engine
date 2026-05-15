"use client";

// ─── YantraPlate — registry/dispatcher for 4 mandala variants ────────────
// Per integrated-reading-design-v2.md § 5.4.
//
// kind:
//   triad-mandala  — Pass α / opening    (TriadMandala)
//   vesica-trio    — Pass β / resonance  (VesicaTrio)
//   dasha-spiral   — Pass γ / phase-lock (DashaSpiral)
//   compass-trine  — Pass δ / anti-dep   (CompassTrine)
//
// Variants gracefully ignore irrelevant fields in `data`.

import { TriadMandala } from "./TriadMandala";
import { VesicaTrio } from "./VesicaTrio";
import { DashaSpiral } from "./DashaSpiral";
import { CompassTrine } from "./CompassTrine";

export type YantraKind =
  | "triad-mandala"
  | "vesica-trio"
  | "dasha-spiral"
  | "compass-trine";

export interface YantraData {
  subjects?: string[];
  dashaPeriods?: Array<{
    lord: string;
    start_iso: string;
    end_iso: string;
    current?: boolean;
  }>;
  cardinals?: {
    stabilize?: string;
    heal?: string;
    create?: string;
    mutate?: string;
  };
  /** Direct injection of the witness-agents topology SVG (for triad). */
  topologySvg?: string;
}

interface YantraPlateProps {
  kind: YantraKind;
  data?: YantraData;
  /** Convenience pass-through for triad-mandala. */
  topologySvg?: string;
}

export function YantraPlate({ kind, data, topologySvg }: YantraPlateProps) {
  const d = data ?? {};
  switch (kind) {
    case "triad-mandala":
      return (
        <TriadMandala
          topologySvg={topologySvg ?? d.topologySvg}
          subjects={d.subjects}
        />
      );
    case "vesica-trio":
      return <VesicaTrio subjects={d.subjects} />;
    case "dasha-spiral":
      return <DashaSpiral dashaPeriods={d.dashaPeriods} />;
    case "compass-trine":
      return <CompassTrine cardinals={d.cardinals} />;
  }
}
