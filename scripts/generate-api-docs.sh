#!/bin/bash
# API Documentation Generation Script
#
# Generates comprehensive API documentation including:
# - OpenAPI/Swagger JSON specification
# - HTML documentation
# - Markdown API reference

set -e

echo "=========================================="
echo "API Documentation Generation"
echo "=========================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DOCS_DIR="docs/api"
OUTPUT_DIR="$DOCS_DIR/generated"
OPENAPI_FILE="$OUTPUT_DIR/openapi.json"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}Step 1: Building project...${NC}"
cargo build --release --bin noesis-server

echo -e "${BLUE}Step 2: Generating OpenAPI specification...${NC}"
# Start the server temporarily to extract OpenAPI spec
cargo run --release --bin noesis-server &
SERVER_PID=$!

# Wait for server to start
sleep 5

# Download OpenAPI spec
curl -s http://localhost:8080/api/openapi.json > "$OPENAPI_FILE" || {
    echo "Failed to download OpenAPI spec"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
}

# Stop the server
kill $SERVER_PID 2>/dev/null || true

echo -e "${GREEN}✓ OpenAPI spec saved to: $OPENAPI_FILE${NC}"

echo -e "${BLUE}Step 3: Validating OpenAPI specification...${NC}"
# Check if openapi-generator is available
if command -v openapi-generator &> /dev/null; then
    openapi-generator validate -i "$OPENAPI_FILE"
    echo -e "${GREEN}✓ OpenAPI spec is valid${NC}"
else
    echo "Warning: openapi-generator not found, skipping validation"
    echo "Install with: npm install -g @openapitools/openapi-generator-cli"
fi

echo -e "${BLUE}Step 4: Generating HTML documentation...${NC}"
# Generate Redoc HTML documentation
if command -v redoc-cli &> /dev/null; then
    redoc-cli bundle "$OPENAPI_FILE" -o "$OUTPUT_DIR/api-docs.html"
    echo -e "${GREEN}✓ HTML docs generated: $OUTPUT_DIR/api-docs.html${NC}"
else
    echo "Warning: redoc-cli not found, skipping HTML generation"
    echo "Install with: npm install -g redoc-cli"
fi

echo -e "${BLUE}Step 5: Generating Markdown documentation...${NC}"
# Generate Markdown from OpenAPI spec
if command -v widdershins &> /dev/null; then
    widdershins "$OPENAPI_FILE" -o "$OUTPUT_DIR/API_REFERENCE.md"
    echo -e "${GREEN}✓ Markdown docs generated: $OUTPUT_DIR/API_REFERENCE.md${NC}"
else
    echo "Warning: widdershins not found, skipping Markdown generation"
    echo "Install with: npm install -g widdershins"
fi

echo -e "${BLUE}Step 6: Creating API examples...${NC}"
cat > "$OUTPUT_DIR/API_EXAMPLES.md" << 'EOF'
# API Examples

## Authentication

### Register a new user

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "secure_password_123",
    "full_name": "John Doe"
  }'
```

### Login

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "secure_password_123"
  }'
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "phase": 3
  }
}
```

## Panchanga Calculation

### Calculate Panchanga

```bash
curl -X POST http://localhost:8080/api/v1/engines/panchanga/calculate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "date": "2025-01-27",
    "latitude": 19.0760,
    "longitude": 72.8777,
    "timezone": "Asia/Kolkata",
    "precision": "high",
    "include_details": true
  }'
```

Response:
```json
{
  "success": true,
  "data": {
    "tithi": "Shukla Panchami",
    "nakshatra": "Rohini",
    "yoga": "Siddha",
    "karana": "Bava",
    "vara": "Monday",
    "sunrise": "2025-01-27T06:45:23+05:30",
    "sunset": "2025-01-27T18:12:45+05:30"
  },
  "metadata": {
    "duration_ms": 42,
    "backend": "native_solar",
    "cached": false,
    "timestamp": "2025-01-27T10:30:45Z"
  }
}
```

## Health Check

```bash
curl http://localhost:8080/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "engines_loaded": 8,
  "workflows_loaded": 5
}
```

## Error Handling

### Validation Error

```bash
curl -X POST http://localhost:8080/api/v1/engines/panchanga/calculate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "date": "invalid",
    "latitude": 100
  }'
```

Response (400 Bad Request):
```json
{
  "error": "validation_error",
  "message": "Invalid date format",
  "code": "INVALID_DATE_FORMAT",
  "status": 400,
  "context": {
    "method": "POST",
    "path": "/api/v1/engines/panchanga/calculate",
    "metadata": {}
  },
  "timestamp": "2025-01-27T10:30:45Z",
  "suggestions": [
    "Check the request payload format and field types",
    "Ensure all required fields are present",
    "Review the API documentation for endpoint: /api/v1/engines/panchanga/calculate"
  ]
}
```

### Rate Limit Error

Response (429 Too Many Requests):
```json
{
  "error": "rate_limit_exceeded",
  "message": "Rate limit exceeded. Please try again later.",
  "code": "RATE_LIMIT_EXCEEDED",
  "status": 429,
  "context": {
    "method": "POST",
    "path": "/api/v1/engines/panchanga/calculate"
  },
  "timestamp": "2025-01-27T10:30:45Z",
  "suggestions": [
    "Wait before making additional requests",
    "Consider implementing exponential backoff",
    "Review your rate limit tier and upgrade if needed"
  ]
}
```
EOF

echo -e "${GREEN}✓ API examples created: $OUTPUT_DIR/API_EXAMPLES.md${NC}"

echo ""
echo "=========================================="
echo -e "${GREEN}Documentation generation complete!${NC}"
echo "=========================================="
echo ""
echo "Generated files:"
echo "  - OpenAPI spec: $OPENAPI_FILE"
echo "  - HTML docs: $OUTPUT_DIR/api-docs.html (if redoc-cli available)"
echo "  - Markdown: $OUTPUT_DIR/API_REFERENCE.md (if widdershins available)"
echo "  - Examples: $OUTPUT_DIR/API_EXAMPLES.md"
echo ""
echo "To view the Swagger UI, start the server and visit:"
echo "  http://localhost:8080/api/docs"
echo ""
