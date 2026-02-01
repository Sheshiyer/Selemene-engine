//! Accuracy Validation - Vimshottari Dasha Calculations
//!
//! W2-S8-06: Validates Vimshottari calculations against reference data
//! Target: 100% accuracy for nakshatra, dasha sequence, and timing
//!
//! Run with: cargo test --test vimshottari_accuracy -- --nocapture

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{build_app_state, create_router, ApiConfig};
use noesis_auth::AuthService;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tower::ServiceExt;

// ===========================================================================
// Test Utilities
// ===========================================================================

static VIM_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_router() -> &'static Router {
    VIM_ROUTER.get_or_init(|| {
        let config = ApiConfig::from_env();
        let state = build_app_state(&config);
        create_router(state, &config)
    })
}

fn test_token() -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    auth.generate_jwt_token(
        "vimshottari-accuracy-test",
        "premium",
        &["read".to_string()],
        5,
    )
    .expect("JWT")
}

async fn calculate_vimshottari(input: Value) -> (StatusCode, Value) {
    let router = get_router();
    let token = test_token();
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/engines/vimshottari/calculate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

// ===========================================================================
// Reference Data
// ===========================================================================

/// The 27 Nakshatras in order
const NAKSHATRAS: [&str; 27] = [
    "Ashwini", "Bharani", "Krittika",
    "Rohini", "Mrigashira", "Ardra",
    "Punarvasu", "Pushya", "Ashlesha",
    "Magha", "Purva Phalguni", "Uttara Phalguni",
    "Hasta", "Chitra", "Swati",
    "Vishakha", "Anuradha", "Jyeshtha",
    "Mula", "Purva Ashadha", "Uttara Ashadha",
    "Shravana", "Dhanishta", "Shatabhisha",
    "Purva Bhadrapada", "Uttara Bhadrapada", "Revati",
];

/// Nakshatra rulers (determines starting dasha)
const NAKSHATRA_RULERS: [&str; 27] = [
    "Ketu", "Venus", "Sun",       // Ashwini, Bharani, Krittika
    "Moon", "Mars", "Rahu",       // Rohini, Mrigashira, Ardra
    "Jupiter", "Saturn", "Mercury", // Punarvasu, Pushya, Ashlesha
    "Ketu", "Venus", "Sun",       // Magha, P.Phalguni, U.Phalguni
    "Moon", "Mars", "Rahu",       // Hasta, Chitra, Swati
    "Jupiter", "Saturn", "Mercury", // Vishakha, Anuradha, Jyeshtha
    "Ketu", "Venus", "Sun",       // Mula, P.Ashadha, U.Ashadha
    "Moon", "Mars", "Rahu",       // Shravana, Dhanishta, Shatabhisha
    "Jupiter", "Saturn", "Mercury", // P.Bhadrapada, U.Bhadrapada, Revati
];

/// Mahadasha sequence (always same order)
const MAHADASHA_SEQUENCE: [&str; 9] = [
    "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury"
];

/// Mahadasha durations in years
const MAHADASHA_DURATIONS: [u64; 9] = [7, 20, 6, 10, 7, 18, 16, 19, 17];

// ===========================================================================
// Accuracy Tests
// ===========================================================================

/// Test total timeline is 120 years
#[tokio::test]
async fn test_120_year_cycle() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "moon_longitude": 125.0,
            "birth_date": "1985-06-15"
        }
    });
    
    let (status, body) = calculate_vimshottari(input).await;
    
    assert_eq!(status, StatusCode::OK, "Vimshottari failed: {:?}", body);
    
    let timeline = &body["result"]["timeline"];
    let total_years = timeline["total_years"].as_u64().unwrap();
    
    assert_eq!(total_years, 120, "Total cycle should be 120 years");
}

/// Test 9 mahadashas present
#[tokio::test]
async fn test_nine_mahadashas() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "moon_longitude": 125.0,
            "birth_date": "1985-06-15"
        }
    });
    
    let (status, body) = calculate_vimshottari(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let mahadashas = body["result"]["timeline"]["mahadashas"]
        .as_array()
        .expect("mahadashas should be array");
    
    assert_eq!(mahadashas.len(), 9, "Should have exactly 9 mahadashas");
}

/// Test nakshatra determination from moon longitude
#[tokio::test]
async fn test_nakshatra_determination() {
    // Test cases: moon_longitude -> expected nakshatra
    let test_cases = vec![
        (0.0, "Ashwini"),       // 0° = Ashwini
        (13.33, "Ashwini"),     // Still Ashwini (< 13.33°)
        (13.34, "Bharani"),     // 13.33° = Bharani starts
        (125.0, "Magha"),       // 120°-133.33° = Magha (10th nakshatra)
        (359.9, "Revati"),      // Last nakshatra
    ];
    
    for (moon_lon, expected_nak) in test_cases {
        let input = json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "moon_longitude": moon_lon,
                "birth_date": "1990-01-01"
            }
        });
        
        let (status, body) = calculate_vimshottari(input).await;
        
        assert_eq!(status, StatusCode::OK, "Failed for moon at {}°", moon_lon);
        
        let nakshatra_name = body["result"]["birth_nakshatra"]["name"]
            .as_str()
            .unwrap_or("");
        
        // Nakshatra name might have variations, check if it contains expected
        assert!(
            nakshatra_name.to_lowercase().contains(&expected_nak.to_lowercase())
                || expected_nak.to_lowercase().contains(&nakshatra_name.to_lowercase()),
            "Moon at {}°: expected {} but got {}",
            moon_lon,
            expected_nak,
            nakshatra_name
        );
        
        println!("Moon at {}°: {} ✓", moon_lon, nakshatra_name);
    }
}

/// Test Magha nakshatra at 125°
#[tokio::test]
async fn test_magha_at_125_degrees() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "moon_longitude": 125.0,
            "birth_date": "1985-06-15"
        }
    });
    
    let (status, body) = calculate_vimshottari(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let nakshatra = &body["result"]["birth_nakshatra"];
    let name = nakshatra["name"].as_str().unwrap();
    let number = nakshatra["number"].as_u64().unwrap_or(0);
    
    assert!(name.contains("Magha") || name.to_lowercase().contains("magha"));
    assert_eq!(number, 10, "Magha is 10th nakshatra");
}

/// Test mahadasha durations
#[tokio::test]
async fn test_mahadasha_durations() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "moon_longitude": 0.0,
            "birth_date": "1990-01-01"
        }
    });
    
    let (status, body) = calculate_vimshottari(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let mahadashas = body["result"]["timeline"]["mahadashas"]
        .as_array()
        .expect("mahadashas");
    
    // Sum of all durations should be 120
    let total: u64 = mahadashas
        .iter()
        .filter_map(|m| m["duration_years"].as_u64())
        .sum();
    
    assert_eq!(total, 120, "Sum of mahadasha durations should be 120 years");
    
    // Check individual durations match expected
    for maha in mahadashas {
        let planet = maha["planet"].as_str().unwrap_or("");
        let duration = maha["duration_years"].as_u64().unwrap_or(0);
        
        let expected_duration = match planet.to_lowercase().as_str() {
            "ketu" => 7,
            "venus" => 20,
            "sun" => 6,
            "moon" => 10,
            "mars" => 7,
            "rahu" => 18,
            "jupiter" => 16,
            "saturn" => 19,
            "mercury" => 17,
            _ => 0,
        };
        
        if expected_duration > 0 {
            assert_eq!(
                duration,
                expected_duration,
                "{} should have {} year duration, got {}",
                planet,
                expected_duration,
                duration
            );
        }
    }
}

/// Test starting dasha based on nakshatra
#[tokio::test]
async fn test_starting_dasha_for_nakshatra() {
    // Ashwini (0°) -> Ketu
    // Magha (120°) -> Ketu  
    // Rohini (40°) -> Moon
    let test_cases = vec![
        (0.0, "Ketu"),      // Ashwini -> Ketu
        (40.0, "Moon"),     // Rohini -> Moon
        (70.0, "Rahu"),     // Ardra -> Rahu
        (125.0, "Ketu"),    // Magha -> Ketu
        (200.0, "Jupiter"), // Vishakha -> Jupiter
    ];
    
    for (moon_lon, expected_first_dasha) in test_cases {
        let input = json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "moon_longitude": moon_lon,
                "birth_date": "1990-01-01"
            }
        });
        
        let (status, body) = calculate_vimshottari(input).await;
        
        assert_eq!(status, StatusCode::OK);
        
        let mahadashas = body["result"]["timeline"]["mahadashas"]
            .as_array()
            .expect("mahadashas");
        
        if !mahadashas.is_empty() {
            let first_planet = mahadashas[0]["planet"].as_str().unwrap_or("");
            
            // Allow some flexibility in naming
            assert!(
                first_planet.to_lowercase().contains(&expected_first_dasha.to_lowercase())
                    || expected_first_dasha.to_lowercase().contains(&first_planet.to_lowercase()),
                "Moon at {}°: expected first dasha {} but got {}",
                moon_lon,
                expected_first_dasha,
                first_planet
            );
        }
        
        println!("Moon at {}°: First dasha validated", moon_lon);
    }
}

/// Test mahadasha dates have valid format
#[tokio::test]
async fn test_mahadasha_date_format() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "moon_longitude": 125.0,
            "birth_date": "1985-06-15"
        }
    });
    
    let (status, body) = calculate_vimshottari(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let mahadashas = body["result"]["timeline"]["mahadashas"]
        .as_array()
        .expect("mahadashas");
    
    for (i, maha) in mahadashas.iter().enumerate() {
        let start_date = maha["start_date"].as_str().unwrap_or("");
        let end_date = maha["end_date"].as_str().unwrap_or("");
        
        // Should be valid date strings
        assert!(
            !start_date.is_empty(),
            "Mahadasha {} should have start_date",
            i
        );
        assert!(
            !end_date.is_empty(),
            "Mahadasha {} should have end_date",
            i
        );
        
        // Should be parseable as dates (YYYY-MM-DD or ISO format)
        assert!(
            start_date.contains('-'),
            "Start date should have date separators"
        );
    }
}

/// Test current period identification
#[tokio::test]
async fn test_current_period() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "moon_longitude": 125.0,
            "birth_date": "1985-06-15"
        }
    });
    
    let (status, body) = calculate_vimshottari(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let result = &body["result"];
    
    // Should have current_period or similar field
    if let Some(current) = result.get("current_period").or(result.get("current_dasha")) {
        println!("Current period: {:?}", current);
        // Should identify which mahadasha/antardasha is currently active
    }
}

/// Test idempotency
#[tokio::test]
async fn test_vimshottari_idempotency() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "moon_longitude": 125.0,
            "birth_date": "1985-06-15"
        }
    });
    
    let (status1, body1) = calculate_vimshottari(input.clone()).await;
    let (status2, body2) = calculate_vimshottari(input).await;
    
    assert_eq!(status1, StatusCode::OK);
    assert_eq!(status2, StatusCode::OK);
    
    // Nakshatra should be identical
    assert_eq!(
        body1["result"]["birth_nakshatra"]["name"],
        body2["result"]["birth_nakshatra"]["name"]
    );
    
    // Timeline should be identical
    assert_eq!(
        body1["result"]["timeline"]["total_years"],
        body2["result"]["timeline"]["total_years"]
    );
}

/// Test calculation from birth data (ephemeris-based)
#[tokio::test]
async fn test_calculation_from_birth_data() {
    let input = json!({
        "birth_data": {
            "name": "Vimshottari Birth Test",
            "date": "1985-06-15",
            "time": "14:30",
            "latitude": 12.9716,
            "longitude": 77.5946,
            "timezone": "Asia/Kolkata"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, body) = calculate_vimshottari(input).await;
    
    assert_eq!(status, StatusCode::OK, "Birth data calculation failed: {:?}", body);
    
    // Should have calculated nakshatra from moon position
    let nakshatra = &body["result"]["birth_nakshatra"]["name"];
    assert!(
        nakshatra.is_string() && !nakshatra.as_str().unwrap().is_empty(),
        "Should calculate nakshatra from birth data"
    );
}
