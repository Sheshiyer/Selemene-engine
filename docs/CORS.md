# CORS Configuration

## Overview

The Noesis API implements production-ready CORS (Cross-Origin Resource Sharing) middleware to control which frontend applications can access the API from different origins.

## Configuration

### Environment Variable

**`ALLOWED_ORIGINS`** - Comma-separated list of allowed origin URLs.

```bash
# Development (default if not set)
ALLOWED_ORIGINS="http://localhost:3000,http://localhost:5173"

# Production
ALLOWED_ORIGINS="https://app.example.com,https://dashboard.example.com"

# Mixed environments
ALLOWED_ORIGINS="https://app.example.com,http://localhost:3000"
```

### CORS Policy Settings

The API applies the following CORS policy:

| Setting | Value | Description |
|---------|-------|-------------|
| **Allowed Methods** | `GET`, `POST`, `OPTIONS` | HTTP methods permitted for cross-origin requests |
| **Allowed Headers** | `Content-Type`, `Authorization`, `X-API-Key` | Headers that can be sent in cross-origin requests |
| **Allow Credentials** | `true` | Allows cookies and authentication headers in cross-origin requests |
| **Max Age** | `3600` seconds (1 hour) | Duration browsers cache preflight response |

## Implementation

CORS is configured in `crates/noesis-api/src/lib.rs` via the `create_cors_layer()` function:

```rust
fn create_cors_layer() -> CorsLayer {
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string());

    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                trimmed.parse().ok()
            }
        })
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-api-key"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}
```

## How CORS Works

### Simple Requests

For simple requests (GET with standard headers), the browser:
1. Sends request with `Origin` header
2. Server responds with `Access-Control-Allow-Origin` header
3. Browser allows response if origin matches

Example response headers:
```
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Credentials: true
```

### Preflight Requests

For complex requests (POST with custom headers, or requests with credentials), the browser sends a **preflight OPTIONS request** first:

**Preflight Request:**
```http
OPTIONS /api/v1/engines/panchanga/calculate HTTP/1.1
Origin: https://app.example.com
Access-Control-Request-Method: POST
Access-Control-Request-Headers: authorization, content-type
```

**Preflight Response:**
```http
HTTP/1.1 200 OK
Access-Control-Allow-Origin: https://app.example.com
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: content-type, authorization, x-api-key
Access-Control-Allow-Credentials: true
Access-Control-Max-Age: 3600
```

If preflight succeeds, the browser sends the actual request.

## Testing CORS

### Local Development

1. **Start the API server:**
   ```bash
   cargo run --bin noesis-server
   ```

2. **Test from browser console** (open DevTools on any page):
   ```javascript
   fetch('http://localhost:8080/health', {
     method: 'GET',
     headers: { 'Content-Type': 'application/json' }
   })
   .then(r => r.json())
   .then(console.log)
   .catch(console.error);
   ```

3. **Test with curl** (preflight request):
   ```bash
   curl -X OPTIONS http://localhost:8080/api/v1/status \
     -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -H "Access-Control-Request-Headers: content-type" \
     -i
   ```

   Expected response should include:
   ```
   access-control-allow-origin: http://localhost:3000
   access-control-allow-methods: GET, POST, OPTIONS
   access-control-allow-headers: content-type, authorization, x-api-key
   access-control-allow-credentials: true
   access-control-max-age: 3600
   ```

4. **Test actual request:**
   ```bash
   curl http://localhost:8080/api/v1/status \
     -H "Origin: http://localhost:3000" \
     -H "Content-Type: application/json" \
     -i
   ```

### Production Testing

1. **Set production origins:**
   ```bash
   export ALLOWED_ORIGINS="https://app.example.com,https://dashboard.example.com"
   cargo run --bin noesis-server --release
   ```

2. **Test from production frontend:**
   - Deploy frontend to `https://app.example.com`
   - Make API call from frontend code
   - Verify in browser DevTools Network tab that CORS headers are present

3. **Test unauthorized origin** (should fail):
   ```bash
   curl -X OPTIONS http://localhost:8080/api/v1/status \
     -H "Origin: https://unauthorized-site.com" \
     -H "Access-Control-Request-Method: GET" \
     -i
   ```
   
   Response should NOT include `access-control-allow-origin` header for unauthorized origin.

## Troubleshooting

### "CORS policy: No 'Access-Control-Allow-Origin' header"

**Cause:** Origin not in allowlist or `ALLOWED_ORIGINS` misconfigured.

**Solution:**
1. Check current `ALLOWED_ORIGINS` env var:
   ```bash
   echo $ALLOWED_ORIGINS
   ```
2. Add your frontend origin to the list:
   ```bash
   export ALLOWED_ORIGINS="http://localhost:3000,https://your-frontend.com"
   ```
3. Restart the API server

### "CORS policy: Request header field X is not allowed"

**Cause:** Custom header not in allowed headers list.

**Solution:**
- If you need additional headers beyond `Content-Type`, `Authorization`, `X-API-Key`, modify `create_cors_layer()` in `lib.rs`:
  ```rust
  .allow_headers([
      axum::http::header::CONTENT_TYPE,
      axum::http::header::AUTHORIZATION,
      axum::http::HeaderName::from_static("x-api-key"),
      axum::http::HeaderName::from_static("x-custom-header"), // Add here
  ])
  ```

### "CORS policy: Credentials flag is 'true', but 'Access-Control-Allow-Credentials' header is ''"

**Cause:** Trying to send credentials to a wildcard origin.

**Solution:**
- CORS spec requires specific origins when `credentials: true`
- Ensure `ALLOWED_ORIGINS` contains explicit origin URLs, not wildcards
- Our implementation already enforces this correctly

### OPTIONS requests return 404

**Cause:** Router doesn't handle OPTIONS method for the route.

**Solution:**
- Axum's `CorsLayer` automatically handles OPTIONS requests
- Ensure CORS layer is applied BEFORE `.with_state()` in router chain (it is in our implementation)

## Security Considerations

### Why Not Use `permissive()`?

The old `CorsLayer::permissive()` configuration allowed **any origin** to access the API:
```rust
// ❌ OLD - Insecure for production
.layer(CorsLayer::permissive())
```

This is dangerous because:
- Any website can call your API from the browser
- Attackers can steal user data via malicious sites
- No control over which frontends can access the API

### Production Best Practices

1. **Explicit Origins Only**
   - Always specify exact origin URLs in `ALLOWED_ORIGINS`
   - Never use wildcards (`*`) in production with credentials

2. **HTTPS in Production**
   - Use `https://` origins for production environments
   - Mixed content (HTTPS frontend → HTTP API) will be blocked by browsers

3. **Minimal Headers**
   - Only allow headers actually used by your frontend
   - Current list (`Content-Type`, `Authorization`, `X-API-Key`) covers common auth patterns

4. **Separate Dev/Prod Configs**
   ```bash
   # .env.development
   ALLOWED_ORIGINS="http://localhost:3000,http://localhost:5173"
   
   # .env.production
   ALLOWED_ORIGINS="https://app.example.com"
   ```

5. **Monitor CORS Errors**
   - Browser console shows CORS errors clearly
   - API logs can track preflight OPTIONS requests
   - Metrics can alert on unusual CORS patterns

## Advanced Configuration

### Multiple Environments

Use different `.env` files:

```bash
# Start with dev config
cp .env.development .env
cargo run

# Start with prod config
cp .env.production .env
cargo run --release
```

### Dynamic Origin Validation

For advanced use cases (e.g., subdomain wildcards, tenant-based origins), modify `create_cors_layer()`:

```rust
fn create_cors_layer() -> CorsLayer {
    use axum::http::header::HeaderValue;
    use tower_http::cors::Any;
    
    // Custom origin validation logic
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    // Parse and validate origins
    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    
    CorsLayer::new()
        .allow_origin(origins)
        // ... rest of config
}
```

### Per-Route CORS

To apply different CORS rules to specific routes:

```rust
// In create_router()
let public_routes = Router::new()
    .route("/health", get(health_handler))
    .layer(CorsLayer::permissive()); // More open for public endpoints

let api_routes = Router::new()
    .route("/engines/{id}/calculate", post(calculate_handler))
    .layer(create_cors_layer()); // Restricted for API endpoints

Router::new()
    .merge(public_routes)
    .merge(api_routes)
    .with_state(state)
```

## References

- [MDN CORS Documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
- [tower-http CORS Layer](https://docs.rs/tower-http/latest/tower_http/cors/index.html)
- [CORS Specification](https://fetch.spec.whatwg.org/#http-cors-protocol)
