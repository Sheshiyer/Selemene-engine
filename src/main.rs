use axum::{
    routing::{get, post},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber;
use tokio::sync::RwLock;
use tokio::signal;

use selemene_engine::{
    SelemeneEngine,
    api::create_api_router,
    cache::CacheManager,
    config::EngineConfig,
    engines::CalculationOrchestrator,
    simple::{BirthData, calculate_panchanga_for_birth},
};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Core engine wiring (orchestrator pattern)
    let config = Arc::new(RwLock::new(EngineConfig::default()));
    let orchestrator = Arc::new(CalculationOrchestrator::new(config.clone()));
    let cache_manager = Arc::new(CacheManager::new(
        // L2 (Redis) is currently disabled in code, so this is a placeholder.
        "redis://127.0.0.1/".to_string(),
        256,
        Duration::from_secs(60 * 60),
        false,
    ));
    let engine = Arc::new(SelemeneEngine::new(orchestrator, cache_manager, config));

    // Build our application with a route
    let app = create_api_router(engine.clone())
        // Keep a simple root + demo endpoints alongside the fuller API router
        .route("/", get(root))
        .route("/api/v1/panchanga", post(calculate_panchanga))
        .route("/test/birth-data", get(test_birth_data));

    // Run it with graceful shutdown
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    
    // Configure server with graceful shutdown
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal());
    
    tracing::info!("Server started. Press Ctrl+C to shut down gracefully.");
    
    if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
    }
    
    tracing::info!("Server shut down gracefully.");
}

/// Listens for SIGTERM and SIGINT signals for graceful shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT (Ctrl+C), starting graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown...");
        },
    }
    
    tracing::info!("Graceful shutdown initiated. Waiting for in-flight requests to complete...");
}

async fn root() -> &'static str {
    "Selemene Engine - Astronomical Calculation Engine"
}

async fn calculate_panchanga(Json(payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
    // Extract birth data from payload
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let date = payload.get("date").and_then(|v| v.as_str()).unwrap_or("1991-08-13");
    let time = payload.get("time").and_then(|v| v.as_str()).unwrap_or("13:31");
    let latitude = payload.get("latitude").and_then(|v| v.as_f64()).unwrap_or(12.9629);
    let longitude = payload.get("longitude").and_then(|v| v.as_f64()).unwrap_or(77.5775);
    let timezone = payload.get("timezone").and_then(|v| v.as_str()).unwrap_or("Asia/Kolkata");

    let birth_data = BirthData {
        name: name.to_string(),
        date: date.to_string(),
        time: time.to_string(),
        latitude,
        longitude,
        timezone: timezone.to_string(),
    };

    let result = calculate_panchanga_for_birth(&birth_data);
    
    let response = json!({
        "status": "success",
        "message": "Panchanga calculation completed",
        "birth_data": {
            "name": birth_data.name,
            "date": birth_data.date,
            "time": birth_data.time,
            "latitude": birth_data.latitude,
            "longitude": birth_data.longitude,
            "timezone": birth_data.timezone
        },
        "panchanga": {
            "tithi": result.tithi,
            "nakshatra": result.nakshatra,
            "yoga": result.yoga,
            "karana": result.karana,
            "vara": result.vara,
            "solar_longitude": result.solar_longitude,
            "lunar_longitude": result.lunar_longitude,
            "julian_day": result.julian_day,
            "calculation_time": result.calculation_time
        }
    });

    Ok(Json(response))
}

async fn test_birth_data() -> Json<Value> {
    // Test with your birth data
    let birth_data = BirthData {
        name: "Cumbipuram Nateshan Sheshanarayan Iyer".to_string(),
        date: "1991-08-13".to_string(),
        time: "13:31".to_string(),
        latitude: 12.9629,
        longitude: 77.5775,
        timezone: "Asia/Kolkata".to_string(),
    };

    let result = calculate_panchanga_for_birth(&birth_data);
    
    let response = json!({
        "status": "success",
        "message": "Test calculation with your birth data",
        "birth_data": {
            "name": birth_data.name,
            "date": birth_data.date,
            "time": birth_data.time,
            "latitude": birth_data.latitude,
            "longitude": birth_data.longitude,
            "timezone": birth_data.timezone
        },
        "panchanga": {
            "tithi": result.tithi,
            "nakshatra": result.nakshatra,
            "yoga": result.yoga,
            "karana": result.karana,
            "vara": result.vara,
            "solar_longitude": result.solar_longitude,
            "lunar_longitude": result.lunar_longitude,
            "julian_day": result.julian_day,
            "calculation_time": result.calculation_time
        },
        "interpretation": {
            "tithi_name": get_tithi_name(result.tithi),
            "nakshatra_name": get_nakshatra_name(result.nakshatra),
            "yoga_name": get_yoga_name(result.yoga),
            "karana_name": get_karana_name(result.karana as i32),
            "vara_name": get_vara_name(result.vara)
        }
    });

    Json(response)
}

fn get_tithi_name(tithi: f64) -> String {
    let tithi_names = [
        "Pratipada", "Dwitiya", "Tritiya", "Chaturthi", "Panchami",
        "Shashthi", "Saptami", "Ashtami", "Navami", "Dashami",
        "Ekadashi", "Dwadashi", "Trayodashi", "Chaturdashi", "Purnima",
        "Pratipada", "Dwitiya", "Tritiya", "Chaturthi", "Panchami",
        "Shashthi", "Saptami", "Ashtami", "Navami", "Dashami",
        "Ekadashi", "Dwadashi", "Trayodashi", "Chaturdashi", "Amavasya"
    ];
    let index = (tithi.floor() as usize) % 30;
    tithi_names[index].to_string()
}

fn get_nakshatra_name(nakshatra: f64) -> String {
    let nakshatra_names = [
        "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira",
        "Ardra", "Punarvasu", "Pushya", "Ashlesha", "Magha",
        "Purva Phalguni", "Uttara Phalguni", "Hasta", "Chitra", "Swati",
        "Vishakha", "Anuradha", "Jyeshtha", "Mula", "Purva Ashadha",
        "Uttara Ashadha", "Shravana", "Dhanishta", "Shatabhisha", "Purva Bhadrapada",
        "Uttara Bhadrapada", "Revati"
    ];
    let index = (nakshatra.floor() as usize) % 27;
    nakshatra_names[index].to_string()
}

fn get_yoga_name(yoga: f64) -> String {
    let yoga_names = [
        "Vishkumbha", "Priti", "Ayushman", "Saubhagya", "Shobhana",
        "Atiganda", "Sukarman", "Dhriti", "Shula", "Ganda",
        "Vriddhi", "Dhruva", "Vyaghata", "Harshana", "Vajra",
        "Siddhi", "Vyatipata", "Variyan", "Parigha", "Shiva",
        "Siddha", "Sadhya", "Shubha", "Shukla", "Brahma",
        "Indra", "Vaidhriti"
    ];
    let index = (yoga.floor() as usize) % 27;
    yoga_names[index].to_string()
}

fn get_karana_name(karana: i32) -> String {
    let karana_names = [
        "Bava", "Balava", "Kaulava", "Taitila", "Garija",
        "Vanija", "Vishti", "Shakuni", "Chatushpada", "Naga"
    ];
    let index = (karana - 1) as usize % 10;
    karana_names[index].to_string()
}

fn get_vara_name(vara: i32) -> String {
    let vara_names = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    let index = vara as usize % 7;
    vara_names[index].to_string()
}
