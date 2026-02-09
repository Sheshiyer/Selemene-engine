//! Example demonstrating API documentation, payload validation, and error enhancement
//!
//! This example shows how to use the new scaffolding for API development:
//! - Payload validation with detailed error messages
//! - Error enhancement with contextual suggestions
//! - OpenAPI documentation integration
//!
//! Run with: cargo run --example api_scaffolding_demo

use noesis_api::{
    validation::{ValidationResult, FieldValidators},
    error_enhancer::{ErrorEnhancer, ErrorContext},
};
use axum::http::StatusCode;
use serde_json::json;

fn main() {
    println!("\n=== API Scaffolding Demo ===\n");

    // Demo 1: Payload Validation
    demo_payload_validation();

    // Demo 2: Error Enhancement
    demo_error_enhancement();

    // Demo 3: Field Validators
    demo_field_validators();

    println!("\n=== Demo Complete ===\n");
}

fn demo_payload_validation() {
    println!("--- Payload Validation Demo ---\n");

    // Example 1: Valid request
    let valid_request = json!({
        "date": "2025-01-27",
        "latitude": 19.0760,
        "longitude": 72.8777,
        "timezone": "Asia/Kolkata"
    });

    println!("Valid request:");
    println!("{}", serde_json::to_string_pretty(&valid_request).unwrap());
    println!("✓ Validation would pass\n");

    // Example 2: Invalid request with validation errors
    let invalid_request = json!({
        "date": "27/01/2025",  // Wrong format
        "latitude": 100.0,     // Out of range
        "longitude": 200.0     // Out of range
    });

    println!("Invalid request:");
    println!("{}", serde_json::to_string_pretty(&invalid_request).unwrap());

    let mut result = ValidationResult::new();

    // Validate date
    if let Some(date) = invalid_request.get("date").and_then(|v| v.as_str()) {
        if let Err(err) = FieldValidators::date_format(date, "date") {
            result.add_error(err.field, err.message, err.code);
        }
    }

    // Validate latitude
    if let Some(lat) = invalid_request.get("latitude").and_then(|v| v.as_f64()) {
        if let Err(err) = FieldValidators::latitude(lat, "latitude") {
            result.add_error(err.field, err.message, err.code);
        }
    }

    // Validate longitude
    if let Some(lon) = invalid_request.get("longitude").and_then(|v| v.as_f64()) {
        if let Err(err) = FieldValidators::longitude(lon, "longitude") {
            result.add_error(err.field, err.message, err.code);
        }
    }

    println!("\nValidation errors:");
    for error in &result.errors {
        println!("  - Field '{}': {} (code: {})", error.field, error.message, error.code);
    }
    println!();
}

fn demo_error_enhancement() {
    println!("--- Error Enhancement Demo ---\n");

    // Example 1: Validation error with enhancement
    let context = ErrorContext::new()
        .with_method("POST".to_string())
        .with_path("/api/v1/engines/panchanga/calculate".to_string())
        .with_user_id("user_123".to_string())
        .add_metadata("engine", "panchanga")
        .add_metadata("precision", "high");

    let enhanced = ErrorEnhancer::enhance(
        "validation_error",
        "Invalid date format. Expected YYYY-MM-DD format.",
        "INVALID_DATE_FORMAT",
        StatusCode::BAD_REQUEST,
        context,
    );

    println!("Enhanced validation error:");
    println!("{}", serde_json::to_string_pretty(&enhanced).unwrap());
    println!();

    // Example 2: Rate limit error
    let context = ErrorContext::new()
        .with_method("POST".to_string())
        .with_path("/api/v1/engines/panchanga/calculate".to_string());

    let rate_limit_error = ErrorEnhancer::enhance(
        "rate_limit_exceeded",
        "Rate limit of 100 requests per hour exceeded",
        "RATE_LIMIT_EXCEEDED",
        StatusCode::TOO_MANY_REQUESTS,
        context,
    );

    println!("Enhanced rate limit error:");
    println!("{}", serde_json::to_string_pretty(&rate_limit_error).unwrap());
    println!();

    // Example 3: Authentication error
    let context = ErrorContext::new()
        .with_path("/api/v1/engines/panchanga/calculate".to_string());

    let auth_error = ErrorEnhancer::enhance(
        "authentication_error",
        "Invalid or expired JWT token",
        "AUTH_FAILED",
        StatusCode::UNAUTHORIZED,
        context,
    );

    println!("Enhanced authentication error:");
    println!("Error: {}", auth_error.error);
    println!("Message: {}", auth_error.message);
    println!("Code: {}", auth_error.code);
    println!("Status: {}", auth_error.status);
    println!("Suggestions:");
    for suggestion in &auth_error.suggestions {
        println!("  - {}", suggestion);
    }
    println!();
}

fn demo_field_validators() {
    println!("--- Field Validators Demo ---\n");

    // Example 1: String length validation
    println!("String length validation:");
    match FieldValidators::string_length("hi", "username", Some(3), Some(20)) {
        Ok(_) => println!("  ✓ Valid"),
        Err(err) => println!("  ✗ {}", err.message),
    }
    match FieldValidators::string_length("john_doe", "username", Some(3), Some(20)) {
        Ok(_) => println!("  ✓ Valid"),
        Err(err) => println!("  ✗ {}", err.message),
    }
    println!();

    // Example 2: Email validation
    println!("Email validation:");
    match FieldValidators::email("invalid", "email") {
        Ok(_) => println!("  ✓ Valid"),
        Err(err) => println!("  ✗ {}", err.message),
    }
    match FieldValidators::email("user@example.com", "email") {
        Ok(_) => println!("  ✓ Valid"),
        Err(err) => println!("  ✗ {}", err.message),
    }
    println!();

    // Example 3: Date format validation
    println!("Date format validation:");
    match FieldValidators::date_format("27/01/2025", "date") {
        Ok(_) => println!("  ✓ Valid"),
        Err(err) => println!("  ✗ {}", err.message),
    }
    match FieldValidators::date_format("2025-01-27", "date") {
        Ok(_) => println!("  ✓ Valid"),
        Err(err) => println!("  ✗ {}", err.message),
    }
    println!();

    // Example 4: Coordinate validation
    println!("Coordinate validation:");
    match FieldValidators::latitude(100.0, "latitude") {
        Ok(_) => println!("  ✓ Latitude 100.0 is valid"),
        Err(err) => println!("  ✗ Latitude 100.0: {}", err.message),
    }
    match FieldValidators::latitude(19.0760, "latitude") {
        Ok(_) => println!("  ✓ Latitude 19.0760 is valid"),
        Err(err) => println!("  ✗ Latitude 19.0760: {}", err.message),
    }
    match FieldValidators::longitude(200.0, "longitude") {
        Ok(_) => println!("  ✓ Longitude 200.0 is valid"),
        Err(err) => println!("  ✗ Longitude 200.0: {}", err.message),
    }
    match FieldValidators::longitude(72.8777, "longitude") {
        Ok(_) => println!("  ✓ Longitude 72.8777 is valid"),
        Err(err) => println!("  ✗ Longitude 72.8777: {}", err.message),
    }
    println!();

    // Example 5: Numeric range validation
    println!("Numeric range validation (age 0-120):");
    match FieldValidators::numeric_range(-5.0, "age", Some(0.0), Some(120.0)) {
        Ok(_) => println!("  ✓ Age -5 is valid"),
        Err(err) => println!("  ✗ Age -5: {}", err.message),
    }
    match FieldValidators::numeric_range(25.0, "age", Some(0.0), Some(120.0)) {
        Ok(_) => println!("  ✓ Age 25 is valid"),
        Err(err) => println!("  ✗ Age 25: {}", err.message),
    }
    match FieldValidators::numeric_range(150.0, "age", Some(0.0), Some(120.0)) {
        Ok(_) => println!("  ✓ Age 150 is valid"),
        Err(err) => println!("  ✗ Age 150: {}", err.message),
    }
    println!();
}
