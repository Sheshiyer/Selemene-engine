// ─── 3D yantra barrel — per W10 brief ─────────────────────────────────
// Each 3D variant has the same prop shape as its W2 2D counterpart.
// This barrel re-exports them under the canonical names so the parent
// reading view can swap with a single import-source change:
//
//   import {
//     WitnessPulse,
//     YantraPlate,
//     DashaWaveform,
//   } from "@/components/integrated/3d";
//
// versus the W2 imports from "@/components/integrated/yantras".

export { WitnessPulse3D, WitnessPulse3D as WitnessPulse } from "./WitnessPulse3D";
export { YantraPlate3D, YantraPlate3D as YantraPlate } from "./YantraPlate3D";
export { DashaRibbon3D, DashaRibbon3D as DashaWaveform } from "./DashaRibbon3D";

// Re-export the underlying scene modules in case a caller wants to
// compose them into their own Canvas with bespoke lighting.
export { TriadMandala3DScene } from "./TriadMandala3D";
export { VesicaTrio3DScene } from "./VesicaTrio3D";
export { DashaSpiral3DScene } from "./DashaSpiral3D";
export { CompassTrine3DScene } from "./CompassTrine3D";
