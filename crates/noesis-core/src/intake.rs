use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NormalizedLocation {
    pub display_name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    /// "manual" | "nominatim" | "google-places" | "mapbox" | "geonames"
    pub provider: String,
    /// "exact" | "selected" | "ambiguous" | "manual"
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReportSubjectInput {
    pub role: String, // "primary" | "partner" | ...
    pub name: Option<String>,
    pub gender: Option<String>,
    pub sex_for_external_chart_source: Option<String>,
    pub birth_date: String,
    pub birth_time: Option<String>,
    pub birth_time_confidence: Option<String>,
    pub birth_location_query: Option<String>,
    pub normalized_location: Option<NormalizedLocation>,
    pub relationship_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RelationshipContext {
    pub r#type: Option<String>,
    pub mapping_goal: Option<String>,
    pub sensitivity_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReportGenerationRequest {
    pub report_level: String, // "L0".."L5"
    pub report_mode: Option<String>,
    pub subjects: Vec<ReportSubjectInput>,
    pub relationship_context: Option<RelationshipContext>,
    pub output: Option<serde_json::Value>,
    /// Optional language code for prompt/orchestrator selection (e.g. "hi", "en").
    /// Top-level on request (orchestrator concern, not relationship_context).
    pub language: Option<String>,
}
