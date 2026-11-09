//! Shared types used across all Noesis engines and services

use chrono::{DateTime, Utc};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
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
}

fn default_current_time() -> DateTime<Utc> {
    Utc::now()
}

/// Output from any consciousness engine
#[derive(Debug, Clone)]
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
}

/// Transport scaffolding shared with the TypeScript bridge. Engine-owned
/// `result.provenance` remains separate from this contract-v1 block.
pub const CONTRACT_PROVENANCE_KEY: &str = "__contract_provenance";

#[derive(Deserialize)]
struct EngineOutputWire {
    engine_id: String,
    result: Value,
    witness_prompt: String,
    consciousness_level: u8,
    metadata: CalculationMetadata,
    #[serde(default)]
    provenance: Option<Value>,
}

impl<'de> Deserialize<'de> for EngineOutput {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut wire = EngineOutputWire::deserialize(deserializer)?;
        if let Some(raw) = wire.provenance {
            let parsed: crate::contract::Provenance =
                serde_json::from_value(raw.clone()).map_err(serde::de::Error::custom)?;
            let result = wire.result.as_object_mut().ok_or_else(|| {
                serde::de::Error::custom("contract provenance requires an object result")
            })?;
            if let Some(existing) = result.get(CONTRACT_PROVENANCE_KEY) {
                let existing: crate::contract::Provenance =
                    serde_json::from_value(existing.clone()).map_err(serde::de::Error::custom)?;
                if existing != parsed {
                    return Err(serde::de::Error::custom(
                        "conflicting contract provenance blocks",
                    ));
                }
            }
            result.insert(CONTRACT_PROVENANCE_KEY.to_string(), raw);
        }
        Ok(Self {
            engine_id: wire.engine_id,
            result: wire.result,
            witness_prompt: wire.witness_prompt,
            consciousness_level: wire.consciousness_level,
            metadata: wire.metadata,
        })
    }
}

impl Serialize for EngineOutput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Keep the public constructor unchanged and reuse the bridge's reserved
        // storage. Only the wire representation lifts it to the envelope.
        let mut result = self.result.clone();
        let provenance = result
            .as_object_mut()
            .and_then(|object| object.remove(CONTRACT_PROVENANCE_KEY));
        if let Some(raw) = &provenance {
            serde_json::from_value::<crate::contract::Provenance>(raw.clone())
                .map_err(serde::ser::Error::custom)?;
        }
        let mut wire =
            serializer.serialize_struct("EngineOutput", 5 + usize::from(provenance.is_some()))?;
        wire.serialize_field("engine_id", &self.engine_id)?;
        wire.serialize_field("result", &result)?;
        wire.serialize_field("witness_prompt", &self.witness_prompt)?;
        wire.serialize_field("consciousness_level", &self.consciousness_level)?;
        wire.serialize_field("metadata", &self.metadata)?;
        if let Some(provenance) = provenance {
            wire.serialize_field("provenance", &provenance)?;
        }
        wire.end()
    }
}

impl EngineOutput {
    /// Server/sidecar contract provenance, if supplied and valid. Old outputs
    /// remain unknown; metadata is never used to invent a replacement claim.
    pub fn contract_provenance(&self) -> Option<crate::contract::Provenance> {
        self.result
            .get(CONTRACT_PROVENANCE_KEY)
            .and_then(|raw| serde_json::from_value(raw.clone()).ok())
    }
}

/// Birth data for chart-based calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
    #[serde(alias = "standard")]
    Standard = 1,
    #[serde(alias = "high")]
    High = 2,
    #[serde(alias = "extreme")]
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
    pub timestamp: DateTime<Utc>,
    /// Engine version (from crate Cargo.toml)
    #[serde(default)]
    pub engine_version: String,
    /// Whether the engine served a degraded/fallback path instead of its
    /// primary computation. Each engine sets this explicitly; the API copies it
    /// into the contract-v1 envelope `provenance` for native engines.
    #[serde(default)]
    pub fallback_used: bool,
    /// External provider that computed the result, when one did (e.g. the
    /// vedic API). `None` for purely local computation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
}

impl Default for CalculationMetadata {
    fn default() -> Self {
        Self {
            calculation_time_ms: 0.0,
            backend: String::new(),
            precision_achieved: String::new(),
            cached: false,
            timestamp: Utc::now(),
            engine_version: String::new(),
            fallback_used: false,
            provider_id: None,
        }
    }
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
    pub timestamp: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// OpenAPI-only engine result schemas
// ---------------------------------------------------------------------------

/// Mirrors `engine_panchanga::PanchangaResult`; the `noesis-api` test
/// `panchanga_result_schema_matches_engine_output` fails if the two drift.
/// Examples are the engine's output for 2000-01-01 12:00 UTC.
#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PanchangaResultSchema {
    /// Tithi index, 0-based (0..30; 0-14 Shukla, 15-29 Krishna).
    #[schema(example = 25)]
    pub tithi_index: u8,
    #[schema(example = "Ekadashi (Krishna)")]
    pub tithi_name: String,
    /// Continuous tithi value (0..30): Moon-Sun elongation / 12°.
    #[schema(example = 25.246)]
    pub tithi_value: f64,
    /// Nakshatra index, 0-based (0..27).
    #[schema(example = 14)]
    pub nakshatra_index: u8,
    #[schema(example = "Swati")]
    pub nakshatra_name: String,
    /// Continuous nakshatra value (0..27): sidereal Moon / 13°20'.
    #[schema(example = 14.96)]
    pub nakshatra_value: f64,
    /// Yoga index, 0-based (0..27).
    #[schema(example = 7)]
    pub yoga_index: u8,
    #[schema(example = "Dhriti")]
    pub yoga_name: String,
    /// Continuous yoga value (0..27).
    #[schema(example = 7.198)]
    pub yoga_value: f64,
    /// Karana name index, 0-based (0..11).
    #[schema(example = 0)]
    pub karana_index: u8,
    #[schema(example = "Bava")]
    pub karana_name: String,
    /// Half-tithi slot (1..=60), not a continuous value.
    #[schema(example = 51.0)]
    pub karana_value: f64,
    /// Weekday, 0 = Sunday.
    #[schema(example = 6)]
    pub vara_index: u8,
    #[schema(example = "Shanivara (Saturday)")]
    pub vara_name: String,
    /// Sidereal (Lahiri) solar longitude in degrees, 0..360.
    #[schema(example = 256.512)]
    pub solar_longitude: f64,
    /// Sidereal (Lahiri) lunar longitude in degrees, 0..360.
    #[schema(example = 199.467)]
    pub lunar_longitude: f64,
    /// Julian Day (UT) the calculation used.
    #[schema(example = 2451545.0)]
    pub julian_day: f64,
    /// Path that produced the solar and lunar longitudes behind tithi,
    /// nakshatra, yoga and karana. Absent only on results serialized before
    /// the field existed.
    #[schema(nullable = true)]
    pub position_source: Option<PanchangaPositionSourceSchema>,
    /// The birth time was missing and local noon was assumed. Input
    /// defaulting, not a calculation fallback.
    #[schema(example = false)]
    pub birth_time_assumed: bool,
    /// The local time assumed when `birth_time_assumed` is true (`"12:00"`);
    /// absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false, example = "12:00")]
    pub assumed_birth_time: Option<String>,
    /// Weekday convention applied by the native engine; absent on old results.
    #[schema(nullable = true)]
    pub vara_basis: Option<PanchangaVaraBasisSchema>,
    /// Dated Lahiri ayanamsa actually subtracted from the tropical positions.
    #[schema(nullable = true, example = 24.2254)]
    pub ayanamsa_degrees: Option<f64>,
}

/// Mirrors `engine_panchanga::PositionSource`. Only `swiss_ephemeris_files`
/// is a non-fallback source.
#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PanchangaPositionSourceSchema {
    SwissEphemerisFiles,
    SwissEphemerisMoshier,
    MeanLongitude,
}

#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PanchangaVaraBasisSchema {
    LocalDate,
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
    #[schema(example = "earth")]
    pub dominant_element: String,
    #[schema(example = 0.72)]
    pub coherence_score: f64,
    #[schema(example = "grounding")]
    pub recommended_practice: String,
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

/// Sigil Forge — the selected creation method (what the TS producer emits).
#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigilForgeMethodSchema {
    #[schema(example = "word-elimination")]
    pub id: String,
    #[schema(example = "Word Elimination Method")]
    pub name: String,
    #[schema(
        example = "Write the intention, strike repeated letters, fuse the remainder into one glyph."
    )]
    pub description: String,
    #[schema(example = json!(["Write your intention as a present-tense statement", "Remove repeated letters"]))]
    pub steps: Vec<String>,
}

/// Sigil Forge — server-side verdict on an optional caller-supplied SVG template.
/// This is a status object, never the SVG markup itself.
#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigilForgeSvgPreviewSchema {
    /// `absent` (no template supplied), `accepted`, or `rejected`.
    #[schema(example = "absent")]
    pub status: String,
    #[schema(nullable = true, example = "Template must be a complete SVG document.")]
    pub reason: Option<String>,
}

/// Sigil Forge — provider metadata for an image attempt.
#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigilForgeImageMetadataSchema {
    /// Provider that handled the attempt: `nvidia`, `nano-banana`, `kimi`, or `mock` (tests only).
    #[schema(example = "nvidia")]
    pub provider: String,
    #[schema(nullable = true, example = "black-forest-labs/flux.1-dev")]
    pub model: Option<String>,
    /// Provider completion status (`SUCCESS`, `PROVIDER_UNAVAILABLE`, `MOCK_NO_TOKEN`, `ERROR`, ...).
    #[schema(nullable = true, example = "SUCCESS")]
    pub finish_reason: Option<String>,
    /// Set when `parameters.provider` asked for a different provider than the one used.
    #[schema(nullable = true, example = "kimi")]
    pub provider_requested: Option<String>,
    /// Why the requested provider was not used.
    #[schema(
        nullable = true,
        example = "Requested provider 'kimi' is not configured; using nvidia."
    )]
    pub provider_note: Option<String>,
}

/// Sigil Forge — the image attempt. Present only when `generate_image` or
/// `edit_image_b64` was requested. A real image carries `b64_json`/`url`;
/// a failed, unconfigured, or mock attempt carries `error` instead and the
/// envelope-level `provenance.fallback_used` is `true`.
#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigilForgeGeneratedImageSchema {
    #[schema(nullable = true, example = "iVBORw0KGgo...")]
    pub b64_json: Option<String>,
    #[schema(nullable = true, example = "https://example.invalid/sigil.png")]
    pub url: Option<String>,
    #[schema(nullable = true, example = "Minimalist chaos-magick sigil ...")]
    pub prompt_used: Option<String>,
    #[schema(nullable = true, example = "chaos")]
    pub style: Option<String>,
    #[schema(nullable = true, example = "black-forest-labs/flux.1-dev")]
    pub model: Option<String>,
    #[schema(
        nullable = true,
        example = "nvidia not configured. Set provider credentials for image generation."
    )]
    pub error: Option<String>,
    pub metadata: SigilForgeImageMetadataSchema,
}

/// Sigil Forge result as actually produced by the TypeScript engine
/// (`ts-engines/src/engines/sigil-forge/engine.ts`). There is no `sigil_id`
/// and no `vector_path`: the engine guides the practitioner through a
/// method and optionally attaches an AI-generated raster image.
#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigilForgeResultSchema {
    #[schema(example = "I attract peace and clarity")]
    pub intention: String,
    pub method: SigilForgeMethodSchema,
    pub svg_preview: SigilForgeSvgPreviewSchema,
    #[schema(nullable = true)]
    pub generated_image: Option<SigilForgeGeneratedImageSchema>,
    /// Contract provenance as emitted by the engine (also lifted to the
    /// envelope's top-level `provenance` by the API).
    #[schema(nullable = true, inline)]
    pub provenance: Option<crate::contract::Provenance>,
}

/// Financial Biosensor — a composed decision-reflection surface.
///
/// The index is a declared house model, not a forecast. `formula_version`
/// identifies the constants that produced it, so two numbers from two builds
/// are never silently compared.
#[cfg(feature = "openapi")]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FinancialBiosensorResultSchema {
    #[schema(example = 0.612)]
    pub daily_decision_index: f64,
    #[schema(example = "partial")]
    pub convergence: String,
    #[schema(example = "financial-biosensor/composite@1")]
    pub formula_version: String,
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
    FinancialBiosensor(FinancialBiosensorResultSchema),
}
