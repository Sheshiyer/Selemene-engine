/**
 * FROZEN media contract types for the 4 focus engines (biofield, face-reading, raaga, sigil-forge).
 *
 * Mirrors the P1 W1 frozen contracts 1:1:
 * - .worktrees/T-002-copilot/docs/plans/engine-integration/P1W1-CONTRACTS-FROZEN.md
 * - crates/noesis-core/src/types.rs:437+ (MediaRef, Consent, QualitySpec, GeneratedImage/GeneratedAudio)
 * - ts-engines/src/types/engine.ts:30-71 (image_data, audio_ref, consent, quality, generated_*)
 * - scripts/ext-contract-harness.ts:13-29 (payload shapes exercised against live sidecars)
 * - sankalpa/src/renderer/data/engine-media-contracts.ts:55-85 (Backend* mirror)
 *
 * Cites (mandatory): p1-w1-worker-bootstrap-packet.md, resources-and-assets.md,
 * gaps-and-improvements.md, goal-understanding.md, EXECUTION-STATUS.md, P1W2-HANDOFF.md,
 * detailed-task-list.md (Phase 4 T-080..T-094 sdk).
 *
 * Invariants (goal-understanding.md): two-prong architecture, local-first + explicit consent
 * before any network escalation, no phantom vector_path on sigil output.
 *
 * Tags: phase:integration-p1 wave:integration-w2 area:engine-integration
 */

/** Phase of consciousness required to access an engine */
export type ConsciousnessPhase = 0 | 1 | 2 | 3 | 4 | 5

/** Consent for media (FROZEN). `token` is the consent_token proof. */
export interface Consent {
  granted: boolean
  scopes: string[]
  timestamp: string
  token?: string
}

/** Media reference (FROZEN MediaRef). b64 for inline payloads, reference for upload ids / file refs. */
export interface MediaRef {
  b64?: string
  reference?: string
  mime_type?: string
  /** Optional multipart filename used by capture-upload endpoints. */
  file_name?: string
  consent?: Consent
}

/** Quality spec (FROZEN QualitySpec). */
export interface QualitySpec {
  sufficient?: boolean
  min_coherence?: number
  scores?: Record<string, number>
}

/** Witness prompt — non-prescriptive inquiry for self-reflection. */
export interface WitnessPrompt {
  prompt: string
  context?: string
  themes?: string[]
}

/** Generated image output (FROZEN generated_image). No vector_path (fixed per FROZEN). */
export interface GeneratedImageRef {
  b64_json?: string
  url?: string
  metadata?: {
    model?: string
    prompt?: string
    provider?: string
    style?: string
  }
}

/** Generated audio output (FROZEN generated_audio; raaga T-005/T-031). clip_url null until server clip gen. */
export interface GeneratedAudioRef {
  clip_url?: string | null
  strudel_ratios?: number[]
  root_hz?: number
  metadata?: {
    engine?: string
    melakarta?: number
    timbre?: string
    dosha?: string
    prahar?: number
  }
}

/**
 * Engine input with FROZEN media extensions at top level.
 * Matches ts-engines/src/types/engine.ts EngineInput (media block lines 30-37).
 */
export interface EngineInput {
  consciousness_level: ConsciousnessPhase
  parameters: Record<string, unknown>
  seed?: number
  question?: string
  /** FROZEN: inline/uploaded image for biofield-capture + face-reading (T-004). */
  image_data?: MediaRef
  /** FROZEN: audio reference for raaga (T-005). */
  audio_ref?: MediaRef
  /** FROZEN: explicit consent; required whenever media is sent or generative output is requested. */
  consent?: Consent
  quality?: QualitySpec
}

/** Engine output with FROZEN generated_* at top level. */
export interface EngineOutput<TResult = Record<string, unknown>> {
  engine_id: string
  result: TResult
  witness_prompts: WitnessPrompt[]
  calculated_at: string
  processing_time_ms: number
  /** FROZEN: sigil-forge image (provider T-003/T-060/T-061). */
  generated_image?: GeneratedImageRef
  /** FROZEN: raaga audio (strudel_ratios + optional clip_url). */
  generated_audio?: GeneratedAudioRef
}

/** Error response shape returned by ts-engines server + noesis-api. */
export interface ErrorResponse {
  error: string
  error_code: string
  details?: Record<string, unknown>
}

/** ts-engines /health response. */
export interface HealthResponse {
  status: 'healthy' | 'degraded' | 'unhealthy'
  engines: string[]
  uptime_ms: number
  version: string
}

/** python biofield sidecar /health response (python-services/shared/models.py HealthResponse). */
export interface PythonSidecarHealthResponse {
  status: string
  service: string
  version: string
  opencv_available?: boolean
  numpy_available?: boolean
}

// ---------------------------------------------------------------------------
// Per-engine payloads
// ---------------------------------------------------------------------------

/** Biofield 11-metric spatial metrics (python-services/shared/models.py SpatialMetrics; FROZEN T-004). */
export interface BiofieldMetrics {
  light_quanta_density: number
  normalized_area: number
  average_intensity: number
  inner_noise: number
  energy_analysis: { low: number; medium: number; high: number; total: number }
  entropy_form_coefficient: number
  fractal_dimension: number
  correlation_dimension: number
  body_symmetry: number
  contour_complexity: number
  pattern_regularity: number
}

/** Biofield capture quality (python-services/shared/models.py QualityAssessment). */
export interface BiofieldQualityAssessment {
  sharpness: number
  contrast: number
  noise_level: number
  exposure: number
  sufficient_quality: boolean
}

/** python /analyze response (BiofieldCVResponse; contract biofield-cv/v1). */
export interface BiofieldAnalyzeResponse {
  contract_version: string
  analysis_version: string
  metrics: BiofieldMetrics
  quality_assessment: BiofieldQualityAssessment
  algorithms_run: string[]
  processing_time_ms: number
}

/** Face-reading result (crates/engine-face-reading FaceAnalysis; FROZEN T-004/T-027; most-stubbed engine per gaps). */
export interface FaceReadingResult {
  constitutional_type?: string
  confidence?: number
  key_observation?: string
  zones?: Array<{
    name: string
    score: number
    observation: string
    elemental?: string
  }>
  elemental_balance?: Record<string, number>
  backend?: string
  is_mock_data?: boolean
  [key: string]: unknown
}

/** Raaga result (ts-engines/src/engines/raaga/engine.ts:186-208). */
export interface RaagaResult {
  melakarta: { num: number; name: string; chakra?: string; ma_type?: string }
  swaras: Array<{ swara?: string; ratio?: number; [key: string]: unknown }>
  strudel_ratios: number[]
  root_hz: number
  arohana_indices: number[]
  avarohana_indices: number[]
  prahar: { num: number; label: string; is_recommended_time: boolean }
  dosha_affinities?: Record<string, boolean>
  alternate_ragas?: Array<{ num: number; name: string; chakra?: string; ma_type?: string }>
  total_melakartas: number
  [key: string]: unknown
}

/** Sigil-forge result (ts-engines/src/engines/sigil-forge/engine.ts:279+). No vector_path. */
export interface SigilForgeResult {
  intention: string
  method: {
    id: string
    name: string
    description: string
    steps: string[]
  }
  processing: {
    type: string
    original: string
    remaining_letters: string
    letter_count: number
  } | null
  charging_suggestions?: Array<{ name: string; description: string }>
  guidance?: Record<string, unknown>
  image_prompt?: string
  provider?: string
  [key: string]: unknown
}

// ---------------------------------------------------------------------------
// Per-engine method inputs
// ---------------------------------------------------------------------------

/** Base for all engine calculate calls through the SDK. */
export interface EngineCallBase {
  consciousness_level?: ConsciousnessPhase
  parameters?: Record<string, unknown>
  seed?: number
  question?: string
  quality?: QualitySpec
}

/** biofield birth/engine calculate (Rust engine via noesis-api when P4 lands, or ts proxy). */
export interface BiofieldCalculateInput extends EngineCallBase {
  consent?: Consent
  image_data?: MediaRef
}

/** biofield capture analyze (python sidecar /analyze, multipart). Consent scope biofield-capture required. */
export interface BiofieldAnalyzeInput {
  /** Inline FROZEN media ref with b64 image payload. */
  image_data: MediaRef
  /** Explicit consent (or set image_data.consent). Scope biofield-capture required. */
  consent?: Consent
  /** Subset of the 11 algorithms to run; default = all (python sidecar decides). */
  algorithms?: string[]
  options?: Record<string, unknown>
  capture_metadata?: Record<string, unknown>
}

/** Request body for `POST /api/v1/biofield/sessions`. */
export interface BiofieldCreateSessionInput {
  client_device_id?: string
  viewer_version?: string
  context?: Record<string, unknown>
}

/** Typed session resource returned by the noesis-api biofield lifecycle routes. */
export interface BiofieldSession {
  id: string
  status: string
  started_at: string
  closed_at: string | null
  client_device_id: string | null
  viewer_version: string | null
}

/** Inline image accepted by the authenticated noesis-api capture route. */
export interface BiofieldCaptureImage extends MediaRef {
  b64: string
  file_name?: string
}

/**
 * Capture upload for `POST /api/v1/biofield/sessions/{session_id}/captures`.
 * The SDK serializes `image_data` as the multipart field named exactly `image`.
 */
export interface BiofieldCreateCaptureInput {
  image_data: BiofieldCaptureImage
  /** Explicit consent (or set `image_data.consent`). Scope biofield-capture is required. */
  consent?: Consent
  algorithms?: string[]
  options?: Record<string, unknown>
  capture_metadata?: Record<string, unknown>
}

export interface BiofieldCaptureArtifact {
  id: string | null
  kind: string
  mime_type: string
  storage_path: string | null
  byte_size: number | null
}

/** Typed server response after analysis and persistence of a session capture. */
export interface BiofieldCapture {
  reading_id: string
  session_id: string
  analysis_version: string
  metrics: BiofieldMetrics
  quality_assessment: BiofieldQualityAssessment
  artifacts: BiofieldCaptureArtifact[]
}

/** face-reading calculate. Consent scope face-image required when image_data present. */
export interface FaceReadingCalculateInput extends EngineCallBase {
  image_data?: MediaRef
  consent?: Consent
}

/** raaga calculate. Consent scope raaga-audio required for audio_ref or parameters.request_clip. */
export interface RaagaCalculateInput extends EngineCallBase {
  parameters?: Record<string, unknown> & {
    melakarta?: number | string
    dosha?: string
    root_hz?: number
  }
  audio_ref?: MediaRef
  consent?: Consent
}

/** sigil-forge calculate. Consent scope sigil-gen required when generate_image is requested. */
export interface SigilForgeCalculateInput extends EngineCallBase {
  parameters?: Record<string, unknown> & {
    intention?: string
    method?: string
    generate_image?: boolean
    provider?: string
    style?: string
  }
  consent?: Consent
}
