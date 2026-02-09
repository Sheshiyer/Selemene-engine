//! Enhanced API documentation with examples and schemas
//!
//! Provides comprehensive OpenAPI documentation with:
//! - Request/response examples
//! - Schema definitions
//! - Error response examples
//! - Authentication examples

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Example request for Panchanga calculation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PanchangaCalculationRequest {
    /// Date in YYYY-MM-DD format
    #[schema(example = "2025-01-27")]
    pub date: String,
    
    /// Latitude in decimal degrees (-90 to 90)
    #[schema(example = 19.0760)]
    pub latitude: f64,
    
    /// Longitude in decimal degrees (-180 to 180)
    #[schema(example = 72.8777)]
    pub longitude: f64,
    
    /// Timezone identifier (IANA timezone database)
    #[schema(example = "Asia/Kolkata")]
    pub timezone: Option<String>,
    
    /// Calculation precision level
    #[schema(example = "high")]
    pub precision: Option<String>,
    
    /// Include detailed calculation information
    #[schema(example = true)]
    pub include_details: Option<bool>,
}

/// Example request for batch calculations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchCalculationRequest {
    /// List of calculation requests
    pub requests: Vec<PanchangaCalculationRequest>,
    
    /// Process requests in parallel
    #[schema(example = true)]
    pub parallel: Option<bool>,
    
    /// Maximum concurrent calculations
    #[schema(example = 10)]
    pub max_concurrent: Option<usize>,
}

/// Example response for Panchanga calculation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PanchangaCalculationResponse {
    /// Calculation success status
    pub success: bool,
    
    /// Calculation result data
    pub data: Option<PanchangaData>,
    
    /// Error information if calculation failed
    pub error: Option<ErrorDetails>,
    
    /// Calculation metadata
    pub metadata: CalculationMetadata,
}

/// Panchanga calculation result data
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PanchangaData {
    /// Tithi (lunar day)
    #[schema(example = "Shukla Panchami")]
    pub tithi: String,
    
    /// Nakshatra (lunar mansion)
    #[schema(example = "Rohini")]
    pub nakshatra: String,
    
    /// Yoga
    #[schema(example = "Siddha")]
    pub yoga: String,
    
    /// Karana (half lunar day)
    #[schema(example = "Bava")]
    pub karana: String,
    
    /// Vara (weekday)
    #[schema(example = "Monday")]
    pub vara: String,
    
    /// Sunrise time in ISO 8601 format
    #[schema(example = "2025-01-27T06:45:23+05:30")]
    pub sunrise: String,
    
    /// Sunset time in ISO 8601 format
    #[schema(example = "2025-01-27T18:12:45+05:30")]
    pub sunset: String,
}

/// Calculation metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CalculationMetadata {
    /// Calculation duration in milliseconds
    #[schema(example = 42)]
    pub duration_ms: u64,
    
    /// Backend engine used (native_solar, swiss_ephemeris, etc.)
    #[schema(example = "native_solar")]
    pub backend: String,
    
    /// Whether result was served from cache
    #[schema(example = false)]
    pub cached: bool,
    
    /// Calculation timestamp
    #[schema(example = "2025-01-27T10:30:45Z")]
    pub timestamp: String,
}

/// Error details in response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetails {
    /// Error type
    #[schema(example = "validation_error")]
    pub error: String,
    
    /// Error message
    #[schema(example = "Invalid date format")]
    pub message: String,
    
    /// Error code
    #[schema(example = "INVALID_DATE_FORMAT")]
    pub code: String,
}

/// Authentication request (login)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// User email address
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// User password
    #[schema(example = "secure_password_123")]
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    /// JWT access token
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    
    /// Token type
    #[schema(example = "Bearer")]
    pub token_type: String,
    
    /// Token expiration in seconds
    #[schema(example = 3600)]
    pub expires_in: u64,
    
    /// User information
    pub user: UserInfo,
}

/// User information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    /// User ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    
    /// User email
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// User phase level (0-12)
    #[schema(example = 3)]
    pub phase: u8,
}

/// Rate limit information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RateLimitInfo {
    /// Requests remaining in current window
    #[schema(example = 95)]
    pub remaining: u32,
    
    /// Total requests allowed per window
    #[schema(example = 100)]
    pub limit: u32,
    
    /// Window reset time (Unix timestamp)
    #[schema(example = 1706342400)]
    pub reset: i64,
    
    /// Window duration in seconds
    #[schema(example = 3600)]
    pub window_seconds: u64,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthCheckResponse {
    /// Service status
    #[schema(example = "healthy")]
    pub status: String,
    
    /// Service version
    #[schema(example = "0.1.0")]
    pub version: String,
    
    /// Uptime in seconds
    #[schema(example = 86400)]
    pub uptime_seconds: u64,
    
    /// Number of engines loaded
    #[schema(example = 8)]
    pub engines_loaded: usize,
    
    /// Number of workflows loaded
    #[schema(example = 5)]
    pub workflows_loaded: usize,
}

/// Common error responses for documentation
pub mod error_examples {
    use super::ErrorDetails;
    
    pub fn bad_request() -> ErrorDetails {
        ErrorDetails {
            error: "validation_error".to_string(),
            message: "Invalid request format or parameters".to_string(),
            code: "VALIDATION_FAILED".to_string(),
        }
    }
    
    pub fn unauthorized() -> ErrorDetails {
        ErrorDetails {
            error: "authentication_error".to_string(),
            message: "Authentication required or token invalid".to_string(),
            code: "AUTH_FAILED".to_string(),
        }
    }
    
    pub fn forbidden() -> ErrorDetails {
        ErrorDetails {
            error: "phase_access_denied".to_string(),
            message: "Insufficient phase level to access this resource".to_string(),
            code: "PHASE_ACCESS_DENIED".to_string(),
        }
    }
    
    pub fn not_found() -> ErrorDetails {
        ErrorDetails {
            error: "not_found".to_string(),
            message: "Requested resource not found".to_string(),
            code: "RESOURCE_NOT_FOUND".to_string(),
        }
    }
    
    pub fn rate_limit_exceeded() -> ErrorDetails {
        ErrorDetails {
            error: "rate_limit_exceeded".to_string(),
            message: "Rate limit exceeded. Please try again later.".to_string(),
            code: "RATE_LIMIT_EXCEEDED".to_string(),
        }
    }
    
    pub fn internal_server_error() -> ErrorDetails {
        ErrorDetails {
            error: "internal_error".to_string(),
            message: "Internal server error occurred".to_string(),
            code: "INTERNAL_ERROR".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_schemas() {
        let request = PanchangaCalculationRequest {
            date: "2025-01-27".to_string(),
            latitude: 19.0760,
            longitude: 72.8777,
            timezone: Some("Asia/Kolkata".to_string()),
            precision: Some("high".to_string()),
            include_details: Some(true),
        };
        
        assert_eq!(request.date, "2025-01-27");
        assert_eq!(request.latitude, 19.0760);
    }
    
    #[test]
    fn test_error_examples() {
        let error = error_examples::bad_request();
        assert_eq!(error.error, "validation_error");
        
        let error = error_examples::unauthorized();
        assert_eq!(error.code, "AUTH_FAILED");
        
        let error = error_examples::rate_limit_exceeded();
        assert!(error.message.contains("Rate limit"));
    }
}
