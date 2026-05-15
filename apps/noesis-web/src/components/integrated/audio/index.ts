// Wave 5 — Audio, atmosphere, cursor proximity.
//
// Per integrated-reading-design-v2.md § 5.11.
//
// Usage:
//   <AudioStateProvider>
//     <AmbientAudio nakshatra="rohini" />
//     <CoherenceBreath />
//     <AudioControlPanel />
//     {/* page content */}
//   </AudioStateProvider>

export { AmbientAudio } from "./AmbientAudio";
export { CoherenceBreath } from "./CoherenceBreath";
export { AudioControlPanel } from "./AudioControlPanel";
export { CursorProximityScene } from "./CursorProximityScene";
export {
  AudioStateProvider,
  useAudioState,
  CHAPTER_LABELS,
  type ChapterIndex,
} from "./AudioState";
