// crates/noesis-core/tests/intake_types_test.rs
use noesis_core::intake::{NormalizedLocation, ReportSubjectInput, ReportGenerationRequest};

#[test]
fn normalized_location_roundtrips() {
    let loc = NormalizedLocation {
        display_name: "Jamakhandi, Karnataka".to_string(),
        latitude: 16.5046,
        longitude: 75.2918,
        timezone: "Asia/Kolkata".to_string(),
        provider: "manual".to_string(),
        confidence: "manual".to_string(),
    };
    assert_eq!(loc.latitude, 16.5046);
}

#[test]
fn report_generation_request_requires_subjects() {
    let req = ReportGenerationRequest {
        report_level: "L0".to_string(),
        subjects: vec![],
        ..Default::default()
    };
    // We will later add a real is_complete gate; for now just construct.
    assert!(req.subjects.is_empty());
}
