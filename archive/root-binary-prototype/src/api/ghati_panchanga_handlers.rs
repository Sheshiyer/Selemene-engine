use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use chrono::{Utc, NaiveDate};
use std::sync::Arc;

use crate::{
    models::{
        GhatiTimeRequest, Coordinates, ApiResponse, PanchangaRequest, PrecisionLevel
    },
    time::{
        GhatiPanchangaService,
        PanchangaElement, EnginePanchangaCalculator,
        GhatiCalculationConfig, GhatiCalculationMethod, GhatiPrecision
    },
};

fn map_ghati_precision_to_panchanga_precision(precision: GhatiPrecision) -> PrecisionLevel {
    match precision {
        GhatiPrecision::Standard => PrecisionLevel::Standard,
        GhatiPrecision::High => PrecisionLevel::High,
        GhatiPrecision::Extreme => PrecisionLevel::Extreme,
    }
}

/// Calculate Panchanga for a specific Ghati time
pub async fn calculate_ghati_panchanga(
    State(engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<GhatiTimeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Parse calculation method
    let method = match request.calculation_method.as_deref() {
        Some("fixed") => GhatiCalculationMethod::Fixed,
        Some("hybrid") => GhatiCalculationMethod::Hybrid,
        Some("sunrise_sunset") => GhatiCalculationMethod::SunriseSunset,
        Some("solar_time") => GhatiCalculationMethod::SolarTime,
        _ => GhatiCalculationMethod::Hybrid, // Default to hybrid
    };

    // Parse precision
    let precision = match request.precision.as_deref() {
        Some("standard") => GhatiPrecision::Standard,
        Some("high") => GhatiPrecision::High,
        Some("extreme") => GhatiPrecision::Extreme,
        _ => GhatiPrecision::High, // Default to high precision
    };

    // Create calculator configuration
    let config = GhatiCalculationConfig {
        method,
        precision,
        solar_correction: method == GhatiCalculationMethod::Hybrid,
        equation_of_time: method == GhatiCalculationMethod::Hybrid,
        seasonal_adjustment: false,
    };

    // Create Panchanga calculator (engine-backed)
    let panchanga_calculator = Arc::new(EnginePanchangaCalculator::new(
        engine,
        map_ghati_precision_to_panchanga_precision(precision),
    ));

    // Create Ghati-Panchanga service
    let service = GhatiPanchangaService::new(config, panchanga_calculator);

    // Use current time if not provided
    let utc_time = request.utc_time.unwrap_or_else(Utc::now);

    // Calculate Ghati time
    let ghati_time = match service.calculate_ghati(utc_time, request.location.clone()) {
        Ok(ghati) => ghati,
        Err(e) => {
            let response: ApiResponse<Value> =
                ApiResponse::error(format!("Failed to calculate Ghati time: {}", e));
            return Ok(Json(json!(response)));
        }
    };

    // Calculate Ghati-Panchanga result
    let result = match service.calculate_ghati_panchanga(ghati_time).await {
        Ok(result) => result,
        Err(e) => {
            let response: ApiResponse<Value> =
                ApiResponse::error(format!("Failed to calculate Ghati-Panchanga: {}", e));
            return Ok(Json(json!(response)));
        }
    };

    let api_response = ApiResponse::success(result);
    Ok(Json(json!(api_response)))
}

/// Get current Ghati with Panchanga information
pub async fn get_current_ghati_panchanga(
    State(engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<GhatiTimeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Parse calculation method
    let method = match request.calculation_method.as_deref() {
        Some("fixed") => GhatiCalculationMethod::Fixed,
        Some("hybrid") => GhatiCalculationMethod::Hybrid,
        Some("sunrise_sunset") => GhatiCalculationMethod::SunriseSunset,
        Some("solar_time") => GhatiCalculationMethod::SolarTime,
        _ => GhatiCalculationMethod::Hybrid, // Default to hybrid
    };

    // Parse precision
    let precision = match request.precision.as_deref() {
        Some("standard") => GhatiPrecision::Standard,
        Some("high") => GhatiPrecision::High,
        Some("extreme") => GhatiPrecision::Extreme,
        _ => GhatiPrecision::High, // Default to high precision
    };

    // Create calculator configuration
    let config = GhatiCalculationConfig {
        method,
        precision,
        solar_correction: method == GhatiCalculationMethod::Hybrid,
        equation_of_time: method == GhatiCalculationMethod::Hybrid,
        seasonal_adjustment: false,
    };

    // Create Panchanga calculator (engine-backed)
    let panchanga_calculator = Arc::new(EnginePanchangaCalculator::new(
        engine,
        map_ghati_precision_to_panchanga_precision(precision),
    ));

    // Create Ghati-Panchanga service
    let service = GhatiPanchangaService::new(config, panchanga_calculator);

    // Get current Ghati-Panchanga
    let result = match service.get_current_ghati_panchanga(request.location).await {
        Ok(result) => result,
        Err(e) => {
            let response: ApiResponse<Value> =
                ApiResponse::error(format!("Failed to get current Ghati-Panchanga: {}", e));
            return Ok(Json(json!(response)));
        }
    };

    let api_response = ApiResponse::success(result);
    Ok(Json(json!(api_response)))
}

/// Get daily Ghati-Panchanga for a specific date
pub async fn get_daily_ghati_panchanga(
    State(engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Parse request
    let Some(date_str) = request.get("date").and_then(|v| v.as_str()) else {
        let response: ApiResponse<Value> = ApiResponse::error("Date is required".to_string());
        return Ok(Json(json!(response)));
    };

    let location = match request.get("location") {
        Some(loc) => {
            let lat = loc.get("latitude").and_then(|v| v.as_f64()).unwrap_or(12.9629);
            let lon = loc.get("longitude").and_then(|v| v.as_f64()).unwrap_or(77.5775);
            let alt = loc.get("altitude").and_then(|v| v.as_f64());
            Coordinates {
                latitude: lat,
                longitude: lon,
                altitude: alt,
            }
        }
        None => {
            let response: ApiResponse<Value> = ApiResponse::error("Location is required".to_string());
            return Ok(Json(json!(response)));
        }
    };

    let method = match request.get("calculation_method").and_then(|v| v.as_str()) {
        Some("fixed") => GhatiCalculationMethod::Fixed,
        Some("hybrid") => GhatiCalculationMethod::Hybrid,
        Some("sunrise_sunset") => GhatiCalculationMethod::SunriseSunset,
        Some("solar_time") => GhatiCalculationMethod::SolarTime,
        _ => GhatiCalculationMethod::Hybrid, // Default to hybrid
    };

    // Parse date
    let date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            let response: ApiResponse<Value> =
                ApiResponse::error("Invalid date format. Use YYYY-MM-DD".to_string());
            return Ok(Json(json!(response)));
        }
    };

    // Create calculator configuration
    let config = GhatiCalculationConfig {
        method,
        precision: GhatiPrecision::Standard, // Daily view only needs Ghati level
        solar_correction: method == GhatiCalculationMethod::Hybrid,
        equation_of_time: method == GhatiCalculationMethod::Hybrid,
        seasonal_adjustment: false,
    };

    // Create Panchanga calculator (engine-backed)
    let panchanga_calculator = Arc::new(EnginePanchangaCalculator::new(
        engine,
        PrecisionLevel::High,
    ));

    // Create Ghati-Panchanga service
    let service = GhatiPanchangaService::new(config, panchanga_calculator);

    // Get daily Ghati-Panchanga
    let results = match service.calculate_daily_ghati_panchanga(date, location).await {
        Ok(results) => results,
        Err(e) => {
            let response: ApiResponse<Value> =
                ApiResponse::error(format!("Failed to calculate daily Ghati-Panchanga: {}", e));
            return Ok(Json(json!(response)));
        }
    };

    let api_response = ApiResponse::success(results);
    Ok(Json(json!(api_response)))
}

/// Find next Panchanga change within Ghati boundaries
pub async fn find_next_panchanga_change(
    State(engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let location = match request.get("location") {
        Some(loc) => {
            let lat = loc.get("latitude").and_then(|v| v.as_f64()).unwrap_or(12.9629);
            let lon = loc.get("longitude").and_then(|v| v.as_f64()).unwrap_or(77.5775);
            let alt = loc.get("altitude").and_then(|v| v.as_f64());
            Coordinates {
                latitude: lat,
                longitude: lon,
                altitude: alt,
            }
        }
        None => {
            let response: ApiResponse<Value> = ApiResponse::error("Location is required".to_string());
            return Ok(Json(json!(response)));
        }
    };

    let max_ghatis = request.get("max_ghatis")
        .and_then(|v| v.as_u64())
        .unwrap_or(60) as u8;

    let method = match request.get("calculation_method").and_then(|v| v.as_str()) {
        Some("fixed") => GhatiCalculationMethod::Fixed,
        Some("hybrid") => GhatiCalculationMethod::Hybrid,
        Some("sunrise_sunset") => GhatiCalculationMethod::SunriseSunset,
        Some("solar_time") => GhatiCalculationMethod::SolarTime,
        _ => GhatiCalculationMethod::Hybrid, // Default to hybrid
    };

    // Create calculator configuration
    let config = GhatiCalculationConfig {
        method,
        precision: GhatiPrecision::High,
        solar_correction: method == GhatiCalculationMethod::Hybrid,
        equation_of_time: method == GhatiCalculationMethod::Hybrid,
        seasonal_adjustment: false,
    };

    // Create Panchanga calculator (engine-backed)
    let panchanga_calculator = Arc::new(EnginePanchangaCalculator::new(
        engine,
        PrecisionLevel::High,
    ));

    // Create Ghati-Panchanga service
    let service = GhatiPanchangaService::new(config, panchanga_calculator);

    // Find next Panchanga change
    let change = match service.find_next_panchanga_change(location, max_ghatis).await {
        Ok(change) => change,
        Err(e) => {
            let response: ApiResponse<Value> =
                ApiResponse::error(format!("Failed to find next Panchanga change: {}", e));
            return Ok(Json(json!(response)));
        }
    };

    let api_response = ApiResponse::success(change);
    Ok(Json(json!(api_response)))
}

/// Get Ghati timing for specific Panchanga element changes
pub async fn get_ghati_timing_for_panchanga_changes(
    State(engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    let date_str = request
        .get("date")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let location = match request.get("location") {
        Some(loc) => {
            let lat = loc.get("latitude").and_then(|v| v.as_f64()).unwrap_or(12.9629);
            let lon = loc.get("longitude").and_then(|v| v.as_f64()).unwrap_or(77.5775);
            let alt = loc.get("altitude").and_then(|v| v.as_f64());
            Coordinates {
                latitude: lat,
                longitude: lon,
                altitude: alt,
            }
        }
        None => {
            let response: ApiResponse<Value> = ApiResponse::error("Location is required".to_string());
            return Ok(Json(json!(response)));
        }
    };

    let element = match request
        .get("element")
        .and_then(|v| v.as_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("tithi") => PanchangaElement::Tithi,
        Some("nakshatra") => PanchangaElement::Nakshatra,
        Some("yoga") => PanchangaElement::Yoga,
        Some("karana") => PanchangaElement::Karana,
        Some("vara") => PanchangaElement::Vara,
        _ => {
            let response: ApiResponse<Value> = ApiResponse::error(
                "Element is required (tithi, nakshatra, yoga, karana, vara)".to_string(),
            );
            return Ok(Json(json!(response)));
        }
    };

    let method = match request.get("calculation_method").and_then(|v| v.as_str()) {
        Some("fixed") => GhatiCalculationMethod::Fixed,
        Some("hybrid") => GhatiCalculationMethod::Hybrid,
        Some("sunrise_sunset") => GhatiCalculationMethod::SunriseSunset,
        Some("solar_time") => GhatiCalculationMethod::SolarTime,
        _ => GhatiCalculationMethod::Hybrid, // Default to hybrid
    };

    // Parse date (defaults to today UTC)
    let date = if let Some(date_str) = date_str {
        match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => {
                let response: ApiResponse<Value> =
                    ApiResponse::error("Invalid date format. Use YYYY-MM-DD".to_string());
                return Ok(Json(json!(response)));
            }
        }
    } else {
        Utc::now().date_naive()
    };

    // Create calculator configuration
    let config = GhatiCalculationConfig {
        method,
        precision: GhatiPrecision::High,
        solar_correction: method == GhatiCalculationMethod::Hybrid,
        equation_of_time: method == GhatiCalculationMethod::Hybrid,
        seasonal_adjustment: false,
    };

    // Create Panchanga calculator (engine-backed)
    let panchanga_calculator = Arc::new(EnginePanchangaCalculator::new(
        engine,
        PrecisionLevel::High,
    ));

    // Create Ghati-Panchanga service
    let service = GhatiPanchangaService::new(config, panchanga_calculator);

    // Get Ghati timing for Panchanga changes
    let changes = match service.get_ghati_timing_for_panchanga_changes(date, location, element).await {
        Ok(changes) => changes,
        Err(e) => {
            let response: ApiResponse<Value> = ApiResponse::error(
                format!("Failed to get Ghati timing for Panchanga changes: {}", e),
            );
            return Ok(Json(json!(response)));
        }
    };

    let api_response = ApiResponse::success(changes);
    Ok(Json(json!(api_response)))
}

/// Calculate Panchanga with Ghati precision
pub async fn calculate_panchanga_with_ghati_precision(
    State(engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // Parse Panchanga request
    let panchanga_request = match serde_json::from_value::<PanchangaRequest>(request.clone()) {
        Ok(req) => req,
        Err(_) => {
            let response: ApiResponse<Value> =
                ApiResponse::error("Invalid Panchanga request format".to_string());
            return Ok(Json(json!(response)));
        }
    };

    let precision = match request.get("ghati_precision").and_then(|v| v.as_str()) {
        Some("standard") => GhatiPrecision::Standard,
        Some("high") => GhatiPrecision::High,
        Some("extreme") => GhatiPrecision::Extreme,
        _ => GhatiPrecision::High, // Default to high precision
    };

    let method = match request.get("calculation_method").and_then(|v| v.as_str()) {
        Some("fixed") => GhatiCalculationMethod::Fixed,
        Some("hybrid") => GhatiCalculationMethod::Hybrid,
        Some("sunrise_sunset") => GhatiCalculationMethod::SunriseSunset,
        Some("solar_time") => GhatiCalculationMethod::SolarTime,
        _ => GhatiCalculationMethod::Hybrid, // Default to hybrid
    };

    // Create calculator configuration
    let config = GhatiCalculationConfig {
        method,
        precision,
        solar_correction: method == GhatiCalculationMethod::Hybrid,
        equation_of_time: method == GhatiCalculationMethod::Hybrid,
        seasonal_adjustment: false,
    };

    // Create Panchanga calculator (engine-backed)
    let panchanga_calculator = Arc::new(EnginePanchangaCalculator::new(
        engine,
        map_ghati_precision_to_panchanga_precision(precision),
    ));

    // Create Ghati-Panchanga service
    let service = GhatiPanchangaService::new(config, panchanga_calculator);

    // Calculate Panchanga with Ghati precision
    let result = match service.calculate_panchanga_with_ghati_precision(panchanga_request, precision).await {
        Ok(result) => result,
        Err(e) => {
            let response: ApiResponse<Value> = ApiResponse::error(
                format!("Failed to calculate Panchanga with Ghati precision: {}", e),
            );
            return Ok(Json(json!(response)));
        }
    };

    let api_response = ApiResponse::success(result);
    Ok(Json(json!(api_response)))
}

/// Get available Panchanga elements
pub async fn get_panchanga_elements(
    State(_engine): State<Arc<crate::SelemeneEngine>>,
) -> Result<Json<Value>, StatusCode> {
    let elements = json!({
        "tithi": {
            "name": "Tithi",
            "description": "Lunar day (1-30)",
            "unit": "days",
            "precision": "high"
        },
        "nakshatra": {
            "name": "Nakshatra",
            "description": "Lunar mansion (1-27)",
            "unit": "degrees",
            "precision": "high"
        },
        "yoga": {
            "name": "Yoga",
            "description": "Auspicious combination (1-27)",
            "unit": "degrees",
            "precision": "medium"
        },
        "karana": {
            "name": "Karana",
            "description": "Half Tithi (1-11)",
            "unit": "half_days",
            "precision": "high"
        },
        "vara": {
            "name": "Vara",
            "description": "Weekday (1-7)",
            "unit": "days",
            "precision": "low"
        }
    });

    let response = ApiResponse::success(elements);
    Ok(Json(json!(response)))
}
