//! Tests for payload validation functionality

#[cfg(test)]
mod validation_tests {
    use noesis_api::validation::{
        ValidationResult, FieldValidators, SizeValidator, ContentTypeValidator,
    };
    use serde_json::json;

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult::new();
        assert!(result.is_valid());
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validation_result_with_errors() {
        let mut result = ValidationResult::new();
        result.add_error("email", "Invalid email format", "INVALID_EMAIL");
        result.add_error("password", "Password too short", "MIN_LENGTH");

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2);
        assert_eq!(result.errors[0].field, "email");
        assert_eq!(result.errors[0].code, "INVALID_EMAIL");
        assert_eq!(result.errors[1].field, "password");
    }

    #[test]
    fn test_string_length_validation() {
        // Valid length
        assert!(FieldValidators::string_length("hello", "test", Some(3), Some(10)).is_ok());

        // Too short
        assert!(FieldValidators::string_length("hi", "test", Some(3), None).is_err());

        // Too long
        assert!(FieldValidators::string_length("hello world!", "test", None, Some(10)).is_err());

        // Within range
        assert!(FieldValidators::string_length("test", "test", Some(1), Some(10)).is_ok());
    }

    #[test]
    fn test_numeric_range_validation() {
        // Valid range
        assert!(FieldValidators::numeric_range(5.0, "test", Some(0.0), Some(10.0)).is_ok());

        // Below minimum
        assert!(FieldValidators::numeric_range(-1.0, "test", Some(0.0), None).is_err());

        // Above maximum
        assert!(FieldValidators::numeric_range(11.0, "test", None, Some(10.0)).is_err());

        // Exactly at boundaries
        assert!(FieldValidators::numeric_range(0.0, "test", Some(0.0), Some(10.0)).is_ok());
        assert!(FieldValidators::numeric_range(10.0, "test", Some(0.0), Some(10.0)).is_ok());
    }

    #[test]
    fn test_email_validation() {
        // Valid emails
        assert!(FieldValidators::email("test@example.com", "email").is_ok());
        assert!(FieldValidators::email("user.name+tag@example.co.uk", "email").is_ok());

        // Invalid emails
        assert!(FieldValidators::email("invalid", "email").is_err());
        assert!(FieldValidators::email("@example.com", "email").is_err());
        assert!(FieldValidators::email("test@", "email").is_err());
        assert!(FieldValidators::email("test", "email").is_err());
    }

    #[test]
    fn test_date_format_validation() {
        // Valid dates
        assert!(FieldValidators::date_format("2025-01-27", "date").is_ok());
        assert!(FieldValidators::date_format("2024-12-31", "date").is_ok());

        // Invalid formats
        assert!(FieldValidators::date_format("27/01/2025", "date").is_err());
        assert!(FieldValidators::date_format("2025/01/27", "date").is_err());
        assert!(FieldValidators::date_format("invalid", "date").is_err());
        assert!(FieldValidators::date_format("2025-13-01", "date").is_err()); // Invalid month
    }

    #[test]
    fn test_latitude_validation() {
        // Valid latitudes
        assert!(FieldValidators::latitude(0.0, "lat").is_ok());
        assert!(FieldValidators::latitude(45.5, "lat").is_ok());
        assert!(FieldValidators::latitude(-45.5, "lat").is_ok());
        assert!(FieldValidators::latitude(90.0, "lat").is_ok());
        assert!(FieldValidators::latitude(-90.0, "lat").is_ok());

        // Invalid latitudes
        assert!(FieldValidators::latitude(90.1, "lat").is_err());
        assert!(FieldValidators::latitude(-90.1, "lat").is_err());
        assert!(FieldValidators::latitude(100.0, "lat").is_err());
        assert!(FieldValidators::latitude(-100.0, "lat").is_err());
    }

    #[test]
    fn test_longitude_validation() {
        // Valid longitudes
        assert!(FieldValidators::longitude(0.0, "lon").is_ok());
        assert!(FieldValidators::longitude(120.5, "lon").is_ok());
        assert!(FieldValidators::longitude(-120.5, "lon").is_ok());
        assert!(FieldValidators::longitude(180.0, "lon").is_ok());
        assert!(FieldValidators::longitude(-180.0, "lon").is_ok());

        // Invalid longitudes
        assert!(FieldValidators::longitude(180.1, "lon").is_err());
        assert!(FieldValidators::longitude(-180.1, "lon").is_err());
        assert!(FieldValidators::longitude(200.0, "lon").is_err());
        assert!(FieldValidators::longitude(-200.0, "lon").is_err());
    }

    #[test]
    fn test_size_validator() {
        let validator = SizeValidator::new(1000);

        // Valid sizes
        assert!(validator.validate_size(0).is_ok());
        assert!(validator.validate_size(500).is_ok());
        assert!(validator.validate_size(1000).is_ok());

        // Invalid sizes
        assert!(validator.validate_size(1001).is_err());
        assert!(validator.validate_size(2000).is_err());
    }

    #[test]
    fn test_size_validator_error_message() {
        let validator = SizeValidator::new(1000);
        let result = validator.validate_size(2000);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, "REQUEST_TOO_LARGE");
        assert!(error.message.contains("1000"));
    }

    #[test]
    fn test_content_type_validator_json() {
        let validator = ContentTypeValidator::json();

        // Valid JSON content types
        assert!(validator.validate(Some("application/json")).is_ok());
        assert!(validator.validate(Some("application/json; charset=utf-8")).is_ok());
        assert!(validator.validate(Some("APPLICATION/JSON")).is_ok()); // Case insensitive

        // Invalid content types
        assert!(validator.validate(Some("text/plain")).is_err());
        assert!(validator.validate(Some("text/html")).is_err());
        assert!(validator.validate(Some("application/xml")).is_err());
        assert!(validator.validate(None).is_err());
    }

    #[test]
    fn test_content_type_validator_custom() {
        let validator = ContentTypeValidator::new(vec![
            "application/json".to_string(),
            "application/xml".to_string(),
        ]);

        // Both should be valid
        assert!(validator.validate(Some("application/json")).is_ok());
        assert!(validator.validate(Some("application/xml")).is_ok());

        // Others should be invalid
        assert!(validator.validate(Some("text/plain")).is_err());
    }

    #[test]
    fn test_required_field_validation() {
        let json = json!({
            "field1": "value",
            "field2": null,
        });

        // Present field should pass
        assert!(FieldValidators::required(&json, "field1").is_ok());

        // Null field should fail
        assert!(FieldValidators::required(&json, "field2").is_err());

        // Missing field should fail
        assert!(FieldValidators::required(&json, "field3").is_err());
    }

    #[test]
    fn test_validation_error_details() {
        let result = FieldValidators::string_length("hi", "username", Some(5), None);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.field, "username");
        assert_eq!(error.code, "MIN_LENGTH");
        assert!(error.message.contains("5"));
    }

    #[test]
    fn test_multiple_validations() {
        let mut result = ValidationResult::new();

        // Validate multiple fields
        let json = json!({
            "email": "invalid",
            "age": 150,
            "username": "ab",
        });

        if let Some(email) = json.get("email").and_then(|v| v.as_str()) {
            if let Err(err) = FieldValidators::email(email, "email") {
                result.add_error(err.field, err.message, err.code);
            }
        }

        if let Some(age) = json.get("age").and_then(|v| v.as_f64()) {
            if let Err(err) = FieldValidators::numeric_range(age, "age", Some(0.0), Some(120.0)) {
                result.add_error(err.field, err.message, err.code);
            }
        }

        if let Some(username) = json.get("username").and_then(|v| v.as_str()) {
            if let Err(err) = FieldValidators::string_length(username, "username", Some(3), Some(20)) {
                result.add_error(err.field, err.message, err.code);
            }
        }

        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 3);
    }
}
