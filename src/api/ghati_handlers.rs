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
        GhatiTimeRequest, GhatiTimeResponse, GhatiTransitionInfo,
        GhatiBoundariesRequest, GhatiBoundariesResponse, GhatiBoundaryInfo,
        ApiResponse
    },
    time::{
        GhatiCalculatorFactory, GhatiCalculationConfig,
        GhatiCalculationMethod, GhatiPrecision
    },
};

/// Calculate Ghati time for a given UTC timestamp
pub async fn calculate_ghati_time(
    State(_engine): State<Arc<crate::SelemeneEngine>>,
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
        seasonal_adjustment: false, // TODO: Make configurable
    };

    // Create calculator
    let calculator = GhatiCalculatorFactory::create_calculator(config);

    // Use current time if not provided
    let utc_time = request.utc_time.unwrap_or_else(Utc::now);

    // Calculate Ghati time
    let ghati_time = match calculator.calculate_ghati(utc_time, request.location.clone()) {
        Ok(ghati) => ghati,
        Err(e) => {
            let response: ApiResponse<Value> =
                ApiResponse::error(format!("Failed to calculate Ghati time: {}", e));
            return Ok(Json(json!(response)));
        }
    };

    // Get next Ghati transition
    let next_transition = match calculator.get_next_ghati_transition(utc_time, request.location.clone()) {
        Ok(transition) => {
            let time_until = transition.transition_time - utc_time;
            Some(GhatiTransitionInfo {
                from_ghati: transition.from_ghati,
                to_ghati: transition.to_ghati,
                transition_time: transition.transition_time,
                time_until_transition: format_duration(time_until),
            })
        }
        Err(_) => None,
    };

    // Create response
    let response = GhatiTimeResponse {
        ghati: ghati_time.ghati,
        pala: ghati_time.pala,
        vipala: ghati_time.vipala,
        utc_timestamp: ghati_time.utc_timestamp,
        local_time: ghati_time.utc_timestamp, // TODO: Convert to local timezone
        calculation_method: format!("{:?}", ghati_time.calculation_method).to_lowercase(),
        precision: format!("{:?}", ghati_time.precision).to_lowercase(),
        next_ghati_transition: next_transition,
    };

    let api_response = ApiResponse::success(response);
    Ok(Json(json!(api_response)))
}

/// Get Ghati boundaries for a specific date
pub async fn get_ghati_boundaries(
    State(_engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<GhatiBoundariesRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Parse calculation method
    let method = match request.calculation_method.as_deref() {
        Some("fixed") => GhatiCalculationMethod::Fixed,
        Some("hybrid") => GhatiCalculationMethod::Hybrid,
        Some("sunrise_sunset") => GhatiCalculationMethod::SunriseSunset,
        Some("solar_time") => GhatiCalculationMethod::SolarTime,
        _ => GhatiCalculationMethod::Hybrid, // Default to hybrid
    };

    // Create calculator configuration
    let config = GhatiCalculationConfig {
        method,
        precision: GhatiPrecision::Standard, // Boundaries only need Ghati level
        solar_correction: method == GhatiCalculationMethod::Hybrid,
        equation_of_time: method == GhatiCalculationMethod::Hybrid,
        seasonal_adjustment: false,
    };

    // Create calculator
    let calculator = GhatiCalculatorFactory::create_calculator(config);

    // Parse date
    let date = match NaiveDate::parse_from_str(&request.date, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            let response: ApiResponse<Value> =
                ApiResponse::error("Invalid date format. Use YYYY-MM-DD".to_string());
            return Ok(Json(json!(response)));
        }
    };

    // Calculate boundaries
    let boundaries = match calculator.calculate_ghati_boundaries(date, request.location.clone()) {
        Ok(boundaries) => boundaries,
        Err(e) => {
            let response: ApiResponse<Value> =
                ApiResponse::error(format!("Failed to calculate Ghati boundaries: {}", e));
            return Ok(Json(json!(response)));
        }
    };

    // Convert to response format
    let boundary_infos: Vec<GhatiBoundaryInfo> = boundaries
        .into_iter()
        .map(|boundary| {
            let time_since_midnight = boundary.utc_timestamp - date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            GhatiBoundaryInfo {
                ghati_number: boundary.ghati_number,
                utc_timestamp: boundary.utc_timestamp,
                local_time: boundary.local_time, // TODO: Convert to local timezone
                time_since_midnight: format_duration(time_since_midnight),
            }
        })
        .collect();

    // Create response
    let response = GhatiBoundariesResponse {
        date: request.date,
        location: request.location,
        boundaries: boundary_infos,
        calculation_method: format!("{:?}", method).to_lowercase(),
    };

    let api_response = ApiResponse::success(response);
    Ok(Json(json!(api_response)))
}

/// Get current Ghati time
pub async fn get_current_ghati_time(
    State(engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<GhatiTimeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Create request with current time
    let current_request = GhatiTimeRequest {
        utc_time: Some(Utc::now()),
        location: request.location,
        calculation_method: request.calculation_method,
        precision: request.precision,
    };

    // Use the existing calculate_ghati_time handler
    calculate_ghati_time(State(engine), Json(current_request)).await
}

/// Convert UTC time to Ghati time
pub async fn utc_to_ghati(
    State(engine): State<Arc<crate::SelemeneEngine>>,
    Json(request): Json<GhatiTimeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Ensure UTC time is provided
    if request.utc_time.is_none() {
        let response: ApiResponse<Value> =
            ApiResponse::error("UTC time is required for conversion".to_string());
        return Ok(Json(json!(response)));
    }

    // Use the existing calculate_ghati_time handler
    calculate_ghati_time(State(engine), Json(request)).await
}

/// Convert Ghati time to UTC time
pub async fn ghati_to_utc(
    State(_engine): State<Arc<crate::SelemeneEngine>>,
    Json(_request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement Ghati to UTC conversion
    let response: ApiResponse<Value> =
        ApiResponse::error("Ghati to UTC conversion not yet implemented".to_string());
    Ok(Json(json!(response)))
}

/// Get Ghati time information for a date range
pub async fn get_ghati_time_range(
    State(_engine): State<Arc<crate::SelemeneEngine>>,
    Json(_request): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement Ghati time range calculation
    let response: ApiResponse<Value> =
        ApiResponse::error("Ghati time range calculation not yet implemented".to_string());
    Ok(Json(json!(response)))
}

/// Get Ghati calculation methods and their descriptions
pub async fn get_ghati_methods(
    State(_engine): State<Arc<crate::SelemeneEngine>>,
) -> Result<Json<Value>, StatusCode> {
    let methods = json!({
        "fixed": {
            "name": "Fixed Interval",
            "description": "Fixed 24-minute intervals from midnight UTC",
            "accuracy": "Low",
            "complexity": "Low",
            "use_case": "Simple applications, educational purposes"
        },
        "hybrid": {
            "name": "Hybrid System",
            "description": "Fixed intervals with solar time corrections",
            "accuracy": "High",
            "complexity": "Medium",
            "use_case": "Production applications, modern Vedic software"
        },
        "sunrise_sunset": {
            "name": "Sunrise to Sunset",
            "description": "Divide daylight hours into 60 equal parts",
            "accuracy": "High",
            "complexity": "High",
            "use_case": "Traditional Vedic applications, astrological calculations"
        },
        "solar_time": {
            "name": "Solar Time",
            "description": "Based on local solar time and longitude",
            "accuracy": "Very High",
            "complexity": "High",
            "use_case": "Scientific applications, high-precision calculations"
        }
    });

    let response = ApiResponse::success(methods);
    Ok(Json(json!(response)))
}

/// Helper function to format duration as human-readable string
fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        let duration = chrono::Duration::seconds(3661); // 1 hour, 1 minute, 1 second
        assert_eq!(format_duration(duration), "1h 1m 1s");

        let duration = chrono::Duration::seconds(61); // 1 minute, 1 second
        assert_eq!(format_duration(duration), "1m 1s");

        let duration = chrono::Duration::seconds(30); // 30 seconds
        assert_eq!(format_duration(duration), "30s");
    }
}
