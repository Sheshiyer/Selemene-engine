# API Scaffolding Quick Reference

This is a quick reference guide for the new API scaffolding features.

## Payload Validation

### Basic Usage

```rust
use noesis_api::validation::{ValidationResult, FieldValidators};

let mut result = ValidationResult::new();

// Validate required field
if let Err(err) = FieldValidators::required(&json_data, "email") {
    result.add_error(err.field, err.message, err.code);
}

// Validate email format
if let Err(err) = FieldValidators::email(&email, "email") {
    result.add_error(err.field, err.message, err.code);
}

// Validate date format (YYYY-MM-DD)
if let Err(err) = FieldValidators::date_format(&date, "date") {
    result.add_error(err.field, err.message, err.code);
}

// Validate latitude (-90 to 90)
if let Err(err) = FieldValidators::latitude(latitude, "latitude") {
    result.add_error(err.field, err.message, err.code);
}

// Validate longitude (-180 to 180)
if let Err(err) = FieldValidators::longitude(longitude, "longitude") {
    result.add_error(err.field, err.message, err.code);
}

if !result.is_valid() {
    // Return validation errors
    return Err(result);
}
```

### Available Validators

| Validator | Description | Example |
|-----------|-------------|---------|
| `required(value, field)` | Field must exist and not be null | `FieldValidators::required(&json, "email")` |
| `string_length(str, field, min, max)` | String length constraints | `FieldValidators::string_length("text", "name", Some(3), Some(50))` |
| `numeric_range(num, field, min, max)` | Numeric value constraints | `FieldValidators::numeric_range(25.0, "age", Some(0.0), Some(120.0))` |
| `email(str, field)` | Email format validation | `FieldValidators::email("test@example.com", "email")` |
| `date_format(str, field)` | ISO 8601 date (YYYY-MM-DD) | `FieldValidators::date_format("2025-01-27", "date")` |
| `latitude(num, field)` | Latitude range (-90 to 90) | `FieldValidators::latitude(19.0760, "latitude")` |
| `longitude(num, field)` | Longitude range (-180 to 180) | `FieldValidators::longitude(72.8777, "longitude")` |

### Middleware

Add middleware to your router for automatic validation:

```rust
use noesis_api::validation::{validate_request_size, validate_content_type};

let app = Router::new()
    .route("/api/v1/calculate", post(handler))
    .layer(axum::middleware::from_fn(validate_request_size))
    .layer(axum::middleware::from_fn(validate_content_type));
```

## Error Enhancement

### Basic Usage

```rust
use noesis_api::error_enhancer::{ErrorEnhancer, ErrorContext};
use axum::http::StatusCode;

// Create context from request
let context = ErrorContext::new()
    .with_method("POST".to_string())
    .with_path("/api/v1/calculate".to_string())
    .with_user_id("user_123".to_string())
    .add_metadata("engine", "panchanga");

// Enhance error
let enhanced = ErrorEnhancer::enhance(
    "validation_error",
    "Invalid date format",
    "INVALID_DATE_FORMAT",
    StatusCode::BAD_REQUEST,
    context,
);

// Return as response
Ok(enhanced.into_response())
```

### Convert from EngineError

```rust
use noesis_api::error_enhancer::{ErrorContext, enhance_engine_error};

let context = ErrorContext::new()
    .with_method(req.method().to_string())
    .with_path(req.uri().path().to_string());

match engine_result {
    Ok(data) => Ok(Json(data)),
    Err(engine_error) => {
        let enhanced = enhance_engine_error(engine_error, context);
        Err(enhanced)
    }
}
```

### Error Types with Automatic Suggestions

| Error Type | HTTP Status | Auto-Generated Suggestions |
|------------|-------------|---------------------------|
| `validation_error` | 400 | Format checking, required fields, docs link |
| `authentication_error` | 401 | Token verification, expiration check, header format |
| `rate_limit_exceeded` | 429 | Wait time, backoff strategy, tier upgrade |
| `calculation_error` | 500 | Parameter validation, range checking, precision tips |
| `engine_not_found` | 404 | Available engines list |
| `workflow_not_found` | 404 | Available workflows list |
| `phase_access_denied` | 403 | Upgrade guidance |
| `internal_error` | 500 | Retry suggestion, support contact |

## Testing

### Test Utilities

```rust
use noesis_api::test_utils::{TestClient, fixtures, assertions};

#[tokio::test]
async fn test_calculation() {
    let client = TestClient::new(create_test_router());
    
    // Use pre-built fixtures
    let response = client
        .post("/api/v1/calculate", fixtures::panchanga_request())
        .await;
    
    // Assert status
    response.assert_success();
    
    // Get JSON response
    let json = response.json_value().await;
    
    // Use assertion helpers
    assertions::assert_json_contains(&json, "data");
    assertions::assert_json_bool(&json, "success", true);
}
```

### Available Fixtures

| Fixture | Description |
|---------|-------------|
| `panchanga_request()` | Valid Panchanga calculation request |
| `invalid_panchanga_request()` | Missing required fields |
| `invalid_date_request()` | Wrong date format |
| `invalid_coordinates_request()` | Out of range coordinates |
| `batch_request()` | Multiple calculations |
| `register_request()` | User registration |
| `login_request()` | User authentication |
| `mock_jwt_token()` | Test JWT token |
| `mock_api_key()` | Test API key |

### Assertion Helpers

| Helper | Description |
|--------|-------------|
| `assert_json_contains(json, key)` | Assert key exists |
| `assert_json_eq(json, key, value)` | Assert value equals |
| `assert_json_str(json, key, str)` | Assert string value |
| `assert_json_num(json, key, num)` | Assert numeric value |
| `assert_json_bool(json, key, bool)` | Assert boolean value |
| `assert_json_array_len(json, key, len)` | Assert array length |
| `assert_error_response(json)` | Assert error format |
| `assert_success_response(json)` | Assert success format |

## API Documentation

### Running Swagger UI

```bash
# Start the server
cargo run --bin noesis-server

# Visit in browser
open http://localhost:8080/api/docs
```

### Generating Documentation

```bash
# Generate all documentation formats
./scripts/generate-api-docs.sh

# Output files:
# - docs/api/generated/openapi.json
# - docs/api/generated/api-docs.html
# - docs/api/generated/API_REFERENCE.md
# - docs/api/generated/API_EXAMPLES.md
```

### Adding to OpenAPI Spec

```rust
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MyRequest {
    #[schema(example = "2025-01-27")]
    pub date: String,
    
    #[schema(example = 19.0760)]
    pub latitude: f64,
}

#[utoipa::path(
    post,
    path = "/api/v1/my-endpoint",
    request_body = MyRequest,
    responses(
        (status = 200, description = "Success", body = MyResponse),
        (status = 400, description = "Validation error", body = EnhancedError),
    ),
    tag = "calculations"
)]
async fn my_handler(Json(req): Json<MyRequest>) -> Result<Json<MyResponse>, ApiError> {
    // Handler implementation
}
```

## Examples

Run the demo to see all features in action:

```bash
cargo run --package noesis-api --example api_scaffolding_demo
```

## Further Reading

- [API Testing Guide](./API_TESTING_GUIDE.md) - Comprehensive testing documentation
- [OpenAPI Specification](https://swagger.io/specification/)
- [Axum Documentation](https://docs.rs/axum/)
- [utoipa Documentation](https://docs.rs/utoipa/)
