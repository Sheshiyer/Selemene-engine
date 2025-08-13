use axum::{
    routing::{get, post},
    http::StatusCode,
    response::Json,
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/v1/panchanga", post(calculate_panchanga));

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Selemene Engine - Astronomical Calculation Engine"
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "selemene-engine",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now()
    }))
}

async fn calculate_panchanga(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    // Simple placeholder response
    let response = json!({
        "status": "success",
        "message": "Panchanga calculation completed",
        "data": {
            "date": payload.get("date").unwrap_or(&json!("2025-01-27")),
            "tithi": 15.0,
            "nakshatra": 20.0,
            "yoga": 25.0,
            "karana": 7.0,
            "vara": 1.0,
            "solar_longitude": 120.0,
            "lunar_longitude": 135.0,
            "precision": 2,
            "backend": "placeholder",
            "calculation_time": chrono::Utc::now()
        }
    });

    Ok(Json(response))
}
