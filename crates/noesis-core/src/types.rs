//! Shared types used across all Noesis engines and services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Input to any consciousness engine calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct EngineInput {
    /// Birth data (required for birth-chart-based engines)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub birth_data: Option<BirthData>,
    /// Current timestamp for time-based calculations
    #[serde(default = "default_current_time")]
    pub current_time: DateTime<Utc>,
    /// Geographic location
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub location: Option<Coordinates>,
    /// Calculation precision level
    #[serde(default)]
    pub precision: Precision,
    /// Engine-specific options
    #[serde(default)]
    pub options: HashMap<String, Value>,

    // --- P1 W1 media contract extensions (T-002) ---
    /// Image input (b64 or ref) for face-reading, sigil, biofield-capture
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub image_data: Option<MediaRef>,
    /// Video reference (future capture)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub video_ref: Option<String>,
    /// Audio reference for raaga or other
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub audio_ref: Option<String>,
    /// Explicit consent for media ops (required for camera/image/gen)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub consent: Option<Consent>,
    /// Quality requirements / gate
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub quality: Option<QualitySpec>,
}

fn default_current_time() -> DateTime<Utc> {
    Utc::now()
}

/// Output from any consciousness engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct EngineOutput {
    /// Which engine produced this output
    pub engine_id: String,
    /// Engine-specific result data (each engine defines its own schema)
    #[cfg_attr(feature = "openapi", schema(value_type = EngineResultData))]
    pub result: Value,
    /// Self-inquiry question generated from the calculation
    pub witness_prompt: String,
    /// User's current consciousness development level (0-5)
    pub consciousness_level: u8,
    /// Calculation metadata (timing, backend, precision)
    pub metadata: CalculationMetadata,

    // --- P1 W1 media contract extensions (T-002) ---
    /// Generated image (sigil etc) with b64 or url + metadata
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub generated_image: Option<GeneratedImage>,
    /// Generated / clip audio (raaga strudel + optional server clip)
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub generated_audio: Option<GeneratedAudio>,
}

/// Birth data for chart-based calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct BirthData {
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub name: Option<String>,
    /// Date in YYYY-MM-DD format
    #[cfg_attr(feature = "openapi", schema(example = "1990-01-01"))]
    pub date: String,
    /// Time in HH:MM format
    #[cfg_attr(feature = "openapi", schema(example = "14:30", nullable = true))]
    pub time: Option<String>,
    /// Latitude in decimal degrees
    #[cfg_attr(feature = "openapi", schema(example = 12.9716))]
    pub latitude: f64,
    /// Longitude in decimal degrees
    #[cfg_attr(feature = "openapi", schema(example = 77.5946))]
    pub longitude: f64,
    /// IANA timezone identifier
    #[cfg_attr(feature = "openapi", schema(example = "Asia/Kolkata"))]
    pub timezone: String,
}

impl BirthData {
    /// Validate birth data for correctness
    pub fn validate(&self) -> Result<(), String> {
        // Validate Latitude (-90 to 90)
        if !(self.latitude >= -90.0 && self.latitude <= 90.0) {
            return Err(format!(
                "Invalid latitude: {}. Must be between -90 and 90.",
                self.latitude
            ));
        }

        // Validate Longitude (-180 to 180)
        if !(self.longitude >= -180.0 && self.longitude <= 180.0) {
            return Err(format!(
                "Invalid longitude: {}. Must be between -180 and 180.",
                self.longitude
            ));
        }

        // Validate Date (basic format check YYYY-MM-DD)
        if self.date.len() != 10
            || self.date.chars().nth(4) != Some('-')
            || self.date.chars().nth(7) != Some('-')
        {
            return Err("Invalid date format. Expected YYYY-MM-DD.".to_string());
        }

        // Check realistic year (1000 - 3000)
        if let Ok(year) = self.date[0..4].parse::<i32>() {
            if !(1000..=3000).contains(&year) {
                return Err(format!("Year {} out of supported range (1000-3000)", year));
            }
        } else {
            return Err("Invalid year format".to_string());
        }

        // Validate Timezone (basic check)
        if self.timezone.trim().is_empty() {
            return Err("Timezone is required".to_string());
        }

        Ok(())
    }
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Coordinates {
    /// Latitude in decimal degrees
    pub latitude: f64,
    /// Longitude in decimal degrees
    pub longitude: f64,
    /// Altitude in meters above sea level
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub altitude: Option<f64>,
}

/// Calculation precision levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(Default)]
pub enum Precision {
    #[default]
    Standard = 1,
    High = 2,
    Extreme = 3,
}

/// Metadata about how a calculation was performed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CalculationMetadata {
    /// Time taken for the calculation in milliseconds
    pub calculation_time_ms: f64,
    /// Backend used for calculation (e.g., "native", "swiss_ephemeris")
    pub backend: String,
    /// Precision level achieved
    pub precision_achieved: String,
    /// Whether the result was retrieved from cache
    pub cached: bool,
    /// Timestamp of calculation
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub timestamp: DateTime<Utc>,
    /// Engine version (from crate Cargo.toml)
    #[serde(default)]
    pub engine_version: String,
}

/// Multi-engine workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub engine_ids: Vec<String>,
}

/// Result from executing a multi-engine workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub engine_outputs: HashMap<String, EngineOutput>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub synthesis: Option<Value>,
    pub total_time_ms: f64,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub timestamp: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// OpenAPI-only engine result schemas
// ---------------------------------------------------------------------------

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PanchangaResultSchema {
    #[schema(example = "Sunday")]
    pub vara: String,
    #[schema(example = "Shukla")]
    pub paksha: String,
    #[schema(example = "Punarvasu")]
    pub nakshatra: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NumerologyResultSchema {
    #[schema(example = 7)]
    pub life_path_number: i32,
    #[schema(example = 3)]
    pub expression_number: i32,
    #[schema(example = "Reflective and analytical")]
    pub core_theme: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BiorhythmResultSchema {
    #[schema(example = 82.4)]
    pub physical: f64,
    #[schema(example = 41.7)]
    pub emotional: f64,
    #[schema(example = -15.2)]
    pub intellectual: f64,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HumanDesignResultSchema {
    #[schema(example = "Manifesting Generator")]
    pub type_name: String,
    #[schema(example = "Sacral")]
    pub authority: String,
    #[schema(example = "Respond then inform")]
    pub strategy: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeneKeysResultSchema {
    #[schema(example = "Gift of Patience")]
    pub activation_sequence: String,
    #[schema(example = "Venus Sequence")]
    pub venus_sequence: String,
    #[schema(example = "Pearl Sequence")]
    pub pearl_sequence: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VimshottariResultSchema {
    #[schema(example = "Jupiter")]
    pub current_mahadasha: String,
    #[schema(example = "Saturn")]
    pub current_antardasha: String,
    #[schema(example = 1460)]
    pub days_remaining: i64,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BiofieldResultSchema {
    // NOTE (T-002 fix): stub was inaccurate for both paths.
    // Birth biofield (engine-biofield): nested "metrics" + chakra_readings (see engine-biofield/models.rs)
    // Capture (biofield-capture): 11 flat metrics + quality from Sankalpa biofieldDomain.ts + python sidecar.
    // Use result as Value; this schema is illustrative only. Do not generate clients from it.
    #[schema(example = "earth")]
    pub dominant_element: Option<String>,
    #[schema(example = 0.72)]
    pub coherence_score: f64,
    #[schema(example = "grounding")]
    pub recommended_practice: Option<String>,
    // For capture path example
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub metrics: Option<serde_json::Value>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub quality_assessment: Option<serde_json::Value>,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VedicClockResultSchema {
    #[schema(example = "Brahma Muhurta")]
    pub current_segment: String,
    #[schema(example = "Sattva")]
    pub guna_bias: String,
    #[schema(example = "06:12")]
    pub next_transition_local: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FaceReadingResultSchema {
    #[schema(example = "vata-pitta")]
    pub constitutional_type: String,
    #[schema(example = 0.81)]
    pub confidence: f64,
    #[schema(example = "soft jawline with high brow curvature")]
    pub key_observation: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NadabrahmanResultSchema {
    #[schema(example = 432.0)]
    pub root_frequency_hz: f64,
    #[schema(example = "Bhairavi")]
    pub suggested_raga: String,
    #[schema(example = "AUM")]
    pub mantra_seed: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TransitsResultSchema {
    #[schema(example = "Saturn return window")]
    pub dominant_transit: String,
    #[schema(example = "High")]
    pub intensity: String,
    #[schema(example = "2026-04-12")]
    pub next_peak_date: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnneagramResultSchema {
    #[schema(example = 4)]
    pub core_type: i32,
    #[schema(example = "4w5")]
    pub wing: String,
    #[schema(example = "Individualist")]
    pub archetype: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TarotResultSchema {
    #[schema(example = "three-card")]
    pub spread_type: String,
    #[schema(example = "The Star")]
    pub focal_card: String,
    #[schema(example = "Renewed trust in unfolding")]
    pub synthesis: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IChingResultSchema {
    #[schema(example = 24)]
    pub hexagram: i32,
    #[schema(example = "Return")]
    pub hexagram_name: String,
    #[schema(example = "Return to the center before moving outward")]
    pub guidance: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SacredGeometryResultSchema {
    #[schema(example = "flower-of-life")]
    pub pattern: String,
    #[schema(example = 144)]
    pub point_count: i32,
    #[schema(example = "Harmonic expansion")]
    pub symbolic_theme: String,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigilForgeResultSchema {
    // NOTE (T-002 fix): removed phantom `vector_path` / sigil_id (see gaps: no code ever emits vector/SVG path).
    // Real runtime (ts-engines/sigil-forge/engine.ts): intention + method obj + optional generated_image {b64_json}
    // + processing, charging_suggestions. svg_preview is {status} object, not path.
    #[schema(example = "I attract peace and clarity")]
    pub intention: String,
    #[schema(example = "word-elimination")]
    pub method: String,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub generated_image: Option<serde_json::Value>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub processing: Option<serde_json::Value>,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum EngineResultData {
    Panchanga(PanchangaResultSchema),
    Numerology(NumerologyResultSchema),
    Biorhythm(BiorhythmResultSchema),
    HumanDesign(HumanDesignResultSchema),
    GeneKeys(GeneKeysResultSchema),
    Vimshottari(VimshottariResultSchema),
    Biofield(BiofieldResultSchema),
    VedicClock(VedicClockResultSchema),
    FaceReading(FaceReadingResultSchema),
    Nadabrahman(NadabrahmanResultSchema),
    Transits(TransitsResultSchema),
    Enneagram(EnneagramResultSchema),
    Tarot(TarotResultSchema),
    IChing(IChingResultSchema),
    SacredGeometry(SacredGeometryResultSchema),
    SigilForge(SigilForgeResultSchema),
}

// ---------------------------------------------------------------------------
// Media & Consent extensions for P1 contract freeze (T-002)
// image_data supports b64 (small) | reference (url/ref for large/video/audio)
// generated_* for outputs from sigil, raaga clip etc.
// Fixes: biofield stub was inaccurate (nested vs flat); sigil had phantom vector_path
// ---------------------------------------------------------------------------

/// Consent record for any media capture or generation (local-first + opt-in)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Consent {
    pub granted: bool,
    /// e.g. ["biofield-capture", "face-image", "sigil-gen"]
    pub scopes: Vec<String>,
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub timestamp: DateTime<Utc>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub token: Option<String>,
}

/// Quality gate for capture (from Sankalpa PIP + sidecar)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct QualitySpec {
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub sufficient: Option<bool>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub min_coherence: Option<f64>,
    /// engine-specific scores e.g. sharpness, contrast
    #[serde(default)]
    pub scores: HashMap<String, f64>,
}

/// Unified media reference for input (image_data, audio, video)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct MediaRef {
    /// Base64 data (for small images, <~1-2MB recommended to keep payloads sane)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub b64: Option<String>,
    /// External ref (URL, R2 key, upload_id, file path token)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub reference: Option<String>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub mime_type: Option<String>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub consent: Option<Consent>,
}

/// Generated image output (sigil, future yantra etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GeneratedImage {
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub b64_json: Option<String>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub url: Option<String>,
    pub metadata: GeneratedImageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GeneratedImageMetadata {
    pub model: String,
    pub prompt: String,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub style: Option<String>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub provider: Option<String>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub seed: Option<u64>,
}

/// Generated audio output (raaga clips, future)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GeneratedAudio {
    /// Optional server-side clip URL (once generated/exported)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub clip_url: Option<String>,
    /// Strudel-compatible just-intonation ratios (for raaga)
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub strudel_ratios: Option<Vec<f64>>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub root_hz: Option<f64>,
    pub metadata: GeneratedAudioMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct GeneratedAudioMetadata {
    pub engine: String,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub melakarta: Option<u32>,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub timbre: Option<String>,
}

// T-004: Biofield-capture + face image lifecycle contract (shared with sankalpa biofieldDomain.ts)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum CaptureState {
    Requested,
    Uploaded,
    Analyzed,
    Persisted,
    Rejected,
    Reprocessed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum SessionStatus {
    Active,
    Closed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CaptureLifecycle {
    pub state: CaptureState,
    pub session_id: String,
    pub reading_id: Option<String>,
    pub consent: Consent,
    pub quality: QualitySpec,
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub artifacts: Option<Vec<String>>, // refs
}

// ---------------------------------------------------------------------------
// Contract examples (T-002) - validate for 4 engines. Roundtrip + schema ok.
// These are source of truth for P1; update engine docs if drift.
// ---------------------------------------------------------------------------

/*
Example EngineInput for biofield-capture (matches sankalpa biofieldDomain + consent gate):
{
  "current_time": "2026-07-17T12:00:00Z",
  "options": { "consciousness_level": 2 },
  "image_data": {
    "b64": "iVBORw0KGgoAAAANSUhEUgAA...",  // or reference
    "mime_type": "image/png",
    "consent": { "granted": true, "scopes": ["biofield-capture"], "timestamp": "2026-..." }
  },
  "consent": { "granted": true, "scopes": ["biofield-capture"], "timestamp": "..." },
  "quality": { "sufficient": true, "scores": { "sharpness": 0.82 } }
}

Example EngineOutput for biofield-capture (11 metrics + quality; flat here, nested in birth path):
{ "engine_id": "biofield-capture", "result": { "metrics": { "fractal_dimension": 1.52, ... "body_symmetry": 0.74 }, "quality_assessment": {..} }, ... "generated_image": null }

Example for sigil-forge (no vector_path):
Input: { "options": { "intention": "I attract peace", "generate_image": true, "image_style": "runic" }, "consent": {...} }
Output: { "engine_id": "sigil-forge", "result": { "intention": "...", "method": { "id": "word-elimination", ... }, "generated_image": { "b64_json": "...", "metadata": { "model": "flux...", "prompt": "..." } } }, "generated_image": { "b64_json": "...", "metadata": {...} } }

Example for raaga (T-005 extension ready):
Output result has "strudel_ratios": [1.0, 1.066..., ...], and top-level "generated_audio": { "strudel_ratios": [...], "clip_url": null, "metadata": { "melakarta": 15 } }

Example for face-reading (image_data input):
Input adds "image_data": { "b64": "...", "consent": ... }
Output: { "result": { "analysis": { "constitution": { "primary_dosha": "vata", ... }, "elemental_balance": {...} } } }

Schema validation: cargo test (future) + utoipa openapi gen will cover. Existing form engines unaffected (all new fields default/Option).
*/
