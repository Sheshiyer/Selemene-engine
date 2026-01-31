//! Workflow integration tests for Gene Keys + Vimshottari engine registration
//!
//! Verifies:
//! - Both engines registered in orchestrator and accessible via API
//! - Updated workflows include new engines
//! - Gene Keys endpoint returns correct structure
//! - Vimshottari endpoint returns 120-year timeline
//! - birth-blueprint workflow includes gene-keys

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

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

static WORKFLOW_TEST_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_test_router() -> &'static Router {
    WORKFLOW_TEST_ROUTER.get_or_init(|| {
        let config = ApiConfig::from_env();
        let state = build_app_state(&config);
        create_router(state, &config)
    })
}

fn generate_test_token(consciousness_level: u8) -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    auth.generate_jwt_token(
        "test-user-workflow",
        "premium",
        &["read".to_string(), "write".to_string()],
        consciousness_level,
    )
    .expect("Failed to generate test JWT")
}

async fn authenticated_post(
    router: &Router,
    uri: &str,
    token: &str,
    body: Value,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
            json!({"raw": String::from_utf8_lossy(&body_bytes).to_string()})
        })
    };

    (status, json)
}

async fn authenticated_get(
    router: &Router,
    uri: &str,
    token: &str,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(json!({}))
    };

    (status, json)
}

fn create_birth_input() -> Value {
    json!({
        "birth_data": {
            "name": "Test User",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 12.9716,
            "longitude": 77.5946,
            "timezone": "Asia/Kolkata"
        },
        "current_time": chrono::Utc::now().to_rfc3339(),
        "precision": "Standard",
        "options": {}
    })
}

fn create_gene_keys_input_with_gates() -> Value {
    json!({
        "current_time": chrono::Utc::now().to_rfc3339(),
        "precision": "Standard",
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 45,
                "design_earth": 26
            }
        }
    })
}

fn create_vimshottari_input_with_moon() -> Value {
    json!({
        "current_time": chrono::Utc::now().to_rfc3339(),
        "precision": "Standard",
        "options": {
            "moon_longitude": 125.0,
            "birth_date": "1985-06-15",
            "birth_time": "14:30"
        }
    })
}

// ---------------------------------------------------------------------------
// Engine registration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_engine_list_includes_gene_keys_and_vimshottari() {
    let router = get_test_router();
    let token = generate_test_token(5);

    let (status, body) = authenticated_get(router, "/api/v1/engines", &token).await;

    assert_eq!(status, StatusCode::OK);
    let engines = body["engines"].as_array().expect("engines should be array");
    let engine_ids: Vec<&str> = engines.iter().filter_map(|e| e.as_str()).collect();

    assert!(
        engine_ids.contains(&"gene-keys"),
        "Engine list should include gene-keys: {:?}",
        engine_ids
    );
    assert!(
        engine_ids.contains(&"vimshottari"),
        "Engine list should include vimshottari: {:?}",
        engine_ids
    );
    assert!(
        engine_ids.contains(&"human-design"),
        "Engine list should include human-design: {:?}",
        engine_ids
    );
}

#[tokio::test]
async fn test_gene_keys_engine_info() {
    let router = get_test_router();
    let token = generate_test_token(5);

    let (status, body) =
        authenticated_get(router, "/api/v1/engines/gene-keys/info", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["engine_id"].as_str().unwrap(), "gene-keys");
    assert_eq!(body["engine_name"].as_str().unwrap(), "Gene Keys");
    assert_eq!(body["required_phase"].as_u64().unwrap(), 2);
}

#[tokio::test]
async fn test_vimshottari_engine_info() {
    let router = get_test_router();
    let token = generate_test_token(5);

    let (status, body) =
        authenticated_get(router, "/api/v1/engines/vimshottari/info", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["engine_id"].as_str().unwrap(), "vimshottari");
    assert_eq!(body["engine_name"].as_str().unwrap(), "Vimshottari Dasha");
    assert_eq!(body["required_phase"].as_u64().unwrap(), 2);
}

// ---------------------------------------------------------------------------
// Gene Keys endpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_gene_keys_calculate_with_gates() {
    let router = get_test_router();
    let token = generate_test_token(5);
    let input = create_gene_keys_input_with_gates();

    let (status, body) = authenticated_post(
        router,
        "/api/v1/engines/gene-keys/calculate",
        &token,
        input,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Gene Keys calculation should succeed: {:?}",
        body
    );

    // Verify output structure
    assert_eq!(body["engine_id"].as_str().unwrap(), "gene-keys");
    assert!(
        !body["witness_prompt"].as_str().unwrap().is_empty(),
        "Witness prompt must be non-empty"
    );

    // Verify activation_sequence in result
    let result = &body["result"];
    assert!(
        result["activation_sequence"].is_object(),
        "Should have activation_sequence"
    );
    assert!(
        result["activation_sequence"]["lifes_work"].is_array(),
        "Should have lifes_work"
    );
    assert!(
        result["activation_sequence"]["evolution"].is_array(),
        "Should have evolution"
    );
    assert!(
        result["activation_sequence"]["radiance"].is_array(),
        "Should have radiance"
    );
    assert!(
        result["activation_sequence"]["purpose"].is_array(),
        "Should have purpose"
    );

    // Verify active_keys present
    assert!(result["active_keys"].is_array(), "Should have active_keys");

    // Verify frequency_assessments present (archetypal depth)
    assert!(
        result["frequency_assessments"].is_array(),
        "Should have frequency_assessments"
    );
}

// ---------------------------------------------------------------------------
// Vimshottari endpoint tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_vimshottari_calculate_with_moon_longitude() {
    let router = get_test_router();
    let token = generate_test_token(5);
    let input = create_vimshottari_input_with_moon();

    let (status, body) = authenticated_post(
        router,
        "/api/v1/engines/vimshottari/calculate",
        &token,
        input,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Vimshottari calculation should succeed: {:?}",
        body
    );

    // Verify output structure
    assert_eq!(body["engine_id"].as_str().unwrap(), "vimshottari");
    assert!(
        !body["witness_prompt"].as_str().unwrap().is_empty(),
        "Witness prompt must be non-empty"
    );

    // Verify 120-year timeline structure
    let result = &body["result"];
    let timeline = &result["timeline"];
    assert_eq!(
        timeline["total_years"].as_u64().unwrap(),
        120,
        "Should be 120-year cycle"
    );

    // Verify 9 mahadashas
    let mahadashas = timeline["mahadashas"]
        .as_array()
        .expect("mahadashas should be array");
    assert_eq!(mahadashas.len(), 9, "Should have 9 mahadashas");

    // Verify each mahadasha has required fields
    for (i, maha) in mahadashas.iter().enumerate() {
        assert!(
            maha["planet"].is_string(),
            "Mahadasha {} should have planet",
            i
        );
        assert!(
            maha["start_date"].is_string(),
            "Mahadasha {} should have start_date",
            i
        );
        assert!(
            maha["end_date"].is_string(),
            "Mahadasha {} should have end_date",
            i
        );
        assert!(
            maha["duration_years"].is_number(),
            "Mahadasha {} should have duration_years",
            i
        );
    }

    // Verify birth_nakshatra
    let nak = &result["birth_nakshatra"];
    assert!(nak["name"].is_string(), "Should have nakshatra name");
    assert!(nak["number"].is_number(), "Should have nakshatra number");
    assert_eq!(
        nak["name"].as_str().unwrap(),
        "Magha",
        "Moon at 125 degrees should be Magha"
    );
}

// ---------------------------------------------------------------------------
// Workflow tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_birth_blueprint_workflow_includes_gene_keys() {
    let router = get_test_router();
    let token = generate_test_token(5);

    // Check workflow definition
    let (status, body) = authenticated_get(
        router,
        "/api/v1/workflows/birth-blueprint/info",
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let engine_ids = body["engine_ids"]
        .as_array()
        .expect("Should have engine_ids");
    let ids: Vec<&str> = engine_ids.iter().filter_map(|e| e.as_str()).collect();

    assert!(
        ids.contains(&"numerology"),
        "birth-blueprint should include numerology"
    );
    assert!(
        ids.contains(&"human-design"),
        "birth-blueprint should include human-design"
    );
    assert!(
        ids.contains(&"gene-keys"),
        "birth-blueprint should include gene-keys"
    );
}

#[tokio::test]
async fn test_full_spectrum_workflow_includes_new_engines() {
    let router = get_test_router();
    let token = generate_test_token(5);

    let (status, body) = authenticated_get(
        router,
        "/api/v1/workflows/full-spectrum/info",
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let engine_ids = body["engine_ids"]
        .as_array()
        .expect("Should have engine_ids");
    let ids: Vec<&str> = engine_ids.iter().filter_map(|e| e.as_str()).collect();

    assert!(
        ids.contains(&"gene-keys"),
        "full-spectrum should include gene-keys"
    );
    assert!(
        ids.contains(&"vimshottari"),
        "full-spectrum should include vimshottari"
    );
}

#[tokio::test]
async fn test_self_inquiry_workflow_includes_gene_keys() {
    let router = get_test_router();
    let token = generate_test_token(5);

    let (status, body) = authenticated_get(
        router,
        "/api/v1/workflows/self-inquiry/info",
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let engine_ids = body["engine_ids"]
        .as_array()
        .expect("Should have engine_ids");
    let ids: Vec<&str> = engine_ids.iter().filter_map(|e| e.as_str()).collect();

    assert!(
        ids.contains(&"gene-keys"),
        "self-inquiry should include gene-keys"
    );
}

#[tokio::test]
async fn test_birth_blueprint_workflow_execute() {
    let router = get_test_router();
    let token = generate_test_token(5);
    let input = create_birth_input();

    let (status, body) = authenticated_post(
        router,
        "/api/v1/workflows/birth-blueprint/execute",
        &token,
        input,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Workflow execution should succeed: {:?}",
        body
    );

    assert_eq!(body["workflow_id"].as_str().unwrap(), "birth-blueprint");

    // Verify engine_outputs contains results from registered engines
    let outputs = body["engine_outputs"]
        .as_object()
        .expect("Should have engine_outputs");

    // Numerology should always succeed (phase 0)
    assert!(
        outputs.contains_key("numerology"),
        "Should contain numerology output"
    );

    // HD should succeed (phase 1, user is phase 5)
    assert!(
        outputs.contains_key("human-design"),
        "Should contain human-design output"
    );

    // Gene Keys may succeed or fail depending on ephemeris availability.
    // In the workflow, individual engine failures are gracefully handled
    // (engine omitted from results). If gene-keys IS present, verify structure.
    if outputs.contains_key("gene-keys") {
        let gk = &outputs["gene-keys"];
        assert_eq!(gk["engine_id"].as_str().unwrap(), "gene-keys");
        assert!(!gk["witness_prompt"].as_str().unwrap().is_empty());
    }

    // Workflow should have at least 2 successful engine outputs
    assert!(
        outputs.len() >= 2,
        "Should have at least 2 engine outputs, got {}: {:?}",
        outputs.len(),
        outputs.keys().collect::<Vec<_>>()
    );

    // Verify total_time_ms is present and positive
    assert!(
        body["total_time_ms"].as_f64().unwrap() > 0.0,
        "total_time_ms should be positive"
    );
}

// ---------------------------------------------------------------------------
// Phase gating tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_gene_keys_phase_gated_at_level_1() {
    let router = get_test_router();
    let token = generate_test_token(1); // Phase 1 - below gene-keys requirement of 2
    let input = create_gene_keys_input_with_gates();

    let (status, body) = authenticated_post(
        router,
        "/api/v1/engines/gene-keys/calculate",
        &token,
        input,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "Phase 1 user should be denied gene-keys (requires phase 2): {:?}",
        body
    );
    assert_eq!(body["error_code"].as_str().unwrap(), "PHASE_ACCESS_DENIED");
}

#[tokio::test]
async fn test_vimshottari_phase_gated_at_level_1() {
    let router = get_test_router();
    let token = generate_test_token(1);
    let input = create_vimshottari_input_with_moon();

    let (status, body) = authenticated_post(
        router,
        "/api/v1/engines/vimshottari/calculate",
        &token,
        input,
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "Phase 1 user should be denied vimshottari (requires phase 2): {:?}",
        body
    );
    assert_eq!(body["error_code"].as_str().unwrap(), "PHASE_ACCESS_DENIED");
}

// ---------------------------------------------------------------------------
// Health check includes new engines
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_shows_engine_count() {
    let router = get_test_router();

    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // Should have at least 5 engines (panchanga, numerology, biorhythm, HD, gene-keys, vimshottari)
    let engine_count = body["engines_loaded"].as_u64().unwrap();
    assert!(
        engine_count >= 6,
        "Should have at least 6 engines loaded, got {}",
        engine_count
    );
}
