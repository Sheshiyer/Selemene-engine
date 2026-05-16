// V2 barrel — frozen contracts from Phase 1 of V2_AUDIO_RICHNESS_PLAN.md.
// Phase 2 swarms import from here; v1 callsites do not need to.

export type { Gamaka, GamakaKind } from './gamakas/types';
export { GAMAKA_DEFAULTS, assertNeverGamaka } from './gamakas/types';
export type { GamakaAnnotation, ApplyGamaka, ApplyGamakaInput, ApplyGamakaOutput } from './gamakas/apply';
export { applyGamaka } from './gamakas/apply';
export { deriveGamakas } from './gamakas/derive';

export type { Tala, TalaName } from './talas/types';
export { TALAS, ALL_TALAS, DEFAULT_TALA } from './talas/data';
export { talaGainPattern, talaEuclidExpr, defaultCpsForTala } from './talas/strudel';

export type { Breath, BreathName } from './breaths/data';
export { BREATHS, ALL_BREATHS, breathForChakra } from './breaths/data';
export { breathToArticulation, articulationStrudelSuffix } from './breaths/strudel';
export type { BreathArticulation } from './breaths/strudel';

export { isV2Enabled, v2Enabled, setV2EnabledOverride } from './feature-flag';

// P2 implementations
export { renderGamaka, renderKampita, renderAndolana, renderKurula, renderNokku, renderSphurita, renderPlain } from './gamakas/render';
export { TIMBRES, timbreStrudelSuffix, type Timbre, type TimbreProfile } from './samples/timbres';
export { loadRaagaSamples, FALLBACK_CDNS, activeManifestUrl, resetSamplesForTests } from './samples/loader';
export { emitTala } from './talas/emitter';
export type { TalaEmitterOptions, TalaEmitterResult } from './talas/emitter';
export { compose } from './compose';
export type { ComposeInput, ComposeOutput } from './compose';
export { buildTanpuraCode } from './tanpura';
export type { TanpuraOptions, TanpuraResult } from './tanpura';
export { encodeWav, wavBlobUrl } from './render/wav-encoder';
export type { WavEncodeInput } from './render/wav-encoder';
export { renderToWavBlobUrl } from './render/offline';

// Presets
export { MAYAMALAVAGAULA_GAMAKAS } from './presets/mayamalavagaula';

// V2.5 — synthesizers, paltas (arpeggios), alaap
export { TIMBRES as TIMBRES_FULL, SYNTH_TIMBRES } from './samples/timbres';
export { PALTAS, ALL_PALTAS } from './arpeggios/patterns';
export type { Palta, PaltaName } from './arpeggios/types';
export { renderAlaap } from './alaap/render';
export { DEFAULT_PHASE_CONFIGS } from './alaap/types';
export type { AlaapPhase, AlaapConfig, AlaapPhaseConfig } from './alaap/types';
