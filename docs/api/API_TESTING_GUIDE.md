# API Documentation and Testing Guide

This directory contains comprehensive API documentation, testing utilities, and validation tools for the Noesis API.

## Overview

The Noesis API provides endpoints for astrological calculations, numerology, biorhythms, and multi-engine workflows. All features are enhanced with:

- **Comprehensive payload validation** - Field-level validation with detailed error messages
- **Error enhancement** - Contextual error information with actionable suggestions
- **OpenAPI documentation** - Interactive Swagger UI and machine-readable specs
- **Extensive test coverage** - Unit, integration, and validation tests

## Documentation

### Interactive Documentation

Start the server and visit:
- **Swagger UI**: http://localhost:8080/api/docs
- **OpenAPI JSON**: http://localhost:8080/api/openapi.json

### Generated Documentation

Generate comprehensive documentation:

```bash
./scripts/generate-api-docs.sh
```

This creates:
- `docs/api/generated/openapi.json` - OpenAPI 3.0 specification
- `docs/api/generated/api-docs.html` - Standalone HTML documentation
- `docs/api/generated/API_REFERENCE.md` - Markdown reference
- `docs/api/generated/API_EXAMPLES.md` - Usage examples

## Payload Validation

### Built-in Validators

The API includes comprehensive validation for:

- **Required fields** - Ensures all mandatory fields are present
- **String length** - Min/max character limits
- **Numeric ranges** - Min/max value constraints
- **Email format** - RFC-compliant email validation
- **Date format** - ISO 8601 date validation (YYYY-MM-DD)
- **Coordinates** - Latitude (-90 to 90) and longitude (-180 to 180)
- **Content-Type** - Validates JSON content type
- **Request size** - Maximum 10MB payload limit

### Usage Example

```rust
use noesis_api::validation::{ValidationResult, FieldValidators};

let mut result = ValidationResult::new();

// Validate latitude
if let Err(err) = FieldValidators::latitude(latitude, "latitude") {
    result.add_error(err.field, err.message, err.code);
}

// Validate date format
if let Err(err) = FieldValidators::date_format(&date, "date") {
    result.add_error(err.field, err.message, err.code);
}

if !result.is_valid() {
    // Handle validation errors
    return Err(result);
}
```

### Validation Middleware

The API includes middleware for automatic validation:

- `validate_request_size` - Checks Content-Length header
- `validate_content_type` - Ensures proper Content-Type
- Field-level validators in handlers

## Error Enhancement

### Enhanced Error Responses

All errors include:

```json
{
  "error": "validation_error",
  "message": "Invalid date format",
  "code": "INVALID_DATE_FORMAT",
  "status": 400,
  "context": {
    "method": "POST",
    "path": "/api/v1/engines/panchanga/calculate",
    "user_id": "user_123",
    "metadata": {}
  },
  "timestamp": "2025-01-27T10:30:45Z",
  "request_id": null,
  "suggestions": [
    "Check the request payload format and field types",
    "Ensure all required fields are present",
    "Review the API documentation for endpoint: /api/v1/engines/panchanga/calculate"
  ]
}
```

### Error Types and Suggestions

The error enhancer provides contextual suggestions for:

- **validation_error** - Format and field guidance
- **authentication_error** - Token and API key help
- **rate_limit_exceeded** - Backoff strategies
- **calculation_error** - Parameter and precision tips
- **engine_not_found** - Available engines list
- **workflow_not_found** - Available workflows list
- **phase_access_denied** - Upgrade guidance
- **internal_error** - Retry and support contact

### Usage Example

```rust
use noesis_api::error_enhancer::{ErrorEnhancer, ErrorContext, enhance_engine_error};
use axum::http::StatusCode;

let context = ErrorContext::new()
    .with_method("POST".to_string())
    .with_path("/api/v1/calculate".to_string())
    .with_user_id("user_123".to_string());

let enhanced = ErrorEnhancer::enhance(
    "validation_error",
    "Invalid date format",
    "INVALID_DATE_FORMAT",
    StatusCode::BAD_REQUEST,
    context,
);

// Or convert from EngineError
let enhanced = enhance_engine_error(engine_error, context);
```

## Testing

### Test Structure

```
tests/integration/
├── api_validation_tests.rs      # API endpoint validation tests
├── error_enhancement_tests.rs   # Error enhancer tests
└── payload_validation_tests.rs  # Payload validator tests
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test api_validation_tests
cargo test --test error_enhancement_tests
cargo test --test payload_validation_tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_validation_missing_fields
```

### Test Utilities

The `test_utils` module provides:

- **TestClient** - Mock HTTP client for testing
- **TestResponse** - Response wrapper with assertions
- **fixtures** - Pre-built test data
- **assertions** - JSON assertion helpers

#### Example Usage

```rust
use noesis_api::test_utils::{TestClient, fixtures, assertions};

#[tokio::test]
async fn test_panchanga_calculation() {
    let client = TestClient::new(create_router());
    
    let response = client
        .post("/api/v1/engines/panchanga/calculate", fixtures::panchanga_request())
        .await;
    
    response.assert_success();
    
    let json = response.json_value().await;
    assertions::assert_json_contains(&json, "data");
    assertions::assert_json_bool(&json, "success", true);
}
```

### Test Fixtures

Pre-built test data includes:

- `panchanga_request()` - Valid calculation request
- `invalid_panchanga_request()` - Missing fields
- `invalid_date_request()` - Wrong date format
- `invalid_coordinates_request()` - Out of range coordinates
- `batch_request()` - Multiple calculations
- `register_request()` - User registration
- `login_request()` - User authentication
- `mock_jwt_token()` - Test JWT
- `mock_api_key()` - Test API key

## Modules

### `validation`

Comprehensive payload validation with:
- `ValidationResult` - Validation result container
- `ValidationError` - Error details
- `PayloadValidator` - Custom validator trait
- `SizeValidator` - Request size validation
- `ContentTypeValidator` - Content-Type validation
- `FieldValidators` - Field-level validators

### `error_enhancer`

Error enhancement and enrichment:
- `EnhancedError` - Rich error response
- `ErrorContext` - Request context
- `ErrorEnhancer` - Enhancement logic
- `enhance_engine_error()` - EngineError converter
- Error enhancement middleware

### `api_docs`

OpenAPI documentation schemas:
- Request/response examples
- Schema definitions
- Error response examples
- Authentication examples
- Common error responses

### `test_utils`

Testing utilities:
- `TestClient` - HTTP test client
- `TestResponse` - Response assertions
- `fixtures` - Test data
- `assertions` - JSON assertions

## API Endpoints

### Health & Status

- `GET /health` - Health check
- `GET /health/ready` - Readiness probe
- `GET /metrics` - Prometheus metrics

### Documentation

- `GET /api/docs` - Swagger UI
- `GET /api/openapi.json` - OpenAPI specification

### Authentication

- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/forgot-password` - Password reset request
- `POST /api/v1/auth/reset-password` - Reset password

### User Management

- `GET /api/v1/users/me` - Get current user
- `PATCH /api/v1/users/me` - Update current user

### Engine Calculations

- `GET /api/v1/engines` - List available engines
- `GET /api/v1/engines/:id/info` - Engine information
- `POST /api/v1/engines/:id/calculate` - Perform calculation
- `POST /api/v1/engines/:id/validate` - Validate input

### Workflows

- `GET /api/v1/workflows` - List available workflows
- `GET /api/v1/workflows/:id/info` - Workflow information
- `POST /api/v1/workflows/:id/execute` - Execute workflow

## Authentication

### Bearer Token (JWT)

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
     http://localhost:8080/api/v1/engines/panchanga/calculate
```

### API Key

```bash
curl -H "X-API-Key: YOUR_API_KEY" \
     http://localhost:8080/api/v1/engines/panchanga/calculate
```

## Rate Limiting

Default limits:
- 100 requests per hour per user
- 10 requests per second per IP

Headers included in responses:
- `X-RateLimit-Limit` - Total allowed
- `X-RateLimit-Remaining` - Remaining requests
- `X-RateLimit-Reset` - Reset timestamp

## Best Practices

### Request Validation

1. Always validate input before processing
2. Return detailed validation errors
3. Use appropriate HTTP status codes
4. Include suggestions in errors

### Error Handling

1. Catch and enhance all errors
2. Include request context
3. Provide actionable suggestions
4. Log errors for monitoring

### Testing

1. Write tests for all endpoints
2. Test validation rules
3. Test error cases
4. Use test fixtures for consistency

### Documentation

1. Keep OpenAPI spec up to date
2. Include request/response examples
3. Document error responses
4. Provide code examples

## Contributing

When adding new endpoints:

1. Add to OpenAPI documentation (`#[utoipa::path]`)
2. Implement request validation
3. Add error enhancement
4. Write integration tests
5. Update API examples
6. Regenerate documentation

## Resources

- [OpenAPI Specification](https://swagger.io/specification/)
- [Axum Documentation](https://docs.rs/axum/)
- [utoipa Documentation](https://docs.rs/utoipa/)
- [Testing Best Practices](https://rust-lang.github.io/api-guidelines/documentation.html)
