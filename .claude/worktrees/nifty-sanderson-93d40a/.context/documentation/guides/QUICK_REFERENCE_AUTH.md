# Quick Reference: Authentication & Authorization

## For Developers

### Generate Test Credentials
```bash
cargo run --bin generate_test_credentials
```

### Testing Authentication
```bash
# Run the auth test suite
./test_auth.sh
```

## API Authentication Methods

### Method 1: JWT Token
```bash
curl -X POST http://localhost:8080/api/v1/engines/panchanga/calculate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"birth_data": {...}}'
```

### Method 2: API Key
```bash
curl -X POST http://localhost:8080/api/v1/engines/panchanga/calculate \
  -H "Content-Type: application/json" \
  -H "X-API-Key: YOUR_API_KEY" \
  -d '{"birth_data": {...}}'
```

## Consciousness Levels

Engines have minimum consciousness level requirements:
- **Level 0**: Basic engines (Panchanga - available to all)
- **Level 1-5**: Advanced engines (access controlled)

## Error Codes

| Code | HTTP Status | Meaning |
|------|-------------|---------|
| `UNAUTHORIZED` | 401 | No valid authentication provided |
| `PHASE_ACCESS_DENIED` | 403 | User's consciousness level too low |

## Protected Routes

✅ **Require Authentication**: All `/api/v1/*` routes
❌ **No Authentication**: `/health`, `/metrics`, `/api/legacy/*`

## Creating JWT Tokens (in code)

```rust
use noesis_auth::AuthService;

let auth = AuthService::new("your-secret".to_string());
let token = auth.generate_jwt_token(
    "user_id",
    "premium",
    &vec!["basic:access".to_string()],
    0  // consciousness_level
)?;
```

## Creating API Keys (in code)

```rust
use noesis_auth::{ApiKey, AuthService};
use chrono::Utc;

let api_key = ApiKey {
    key: "unique-api-key".to_string(),
    user_id: "user_id".to_string(),
    tier: "premium".to_string(),
    permissions: vec!["basic:access".to_string()],
    created_at: Utc::now(),
    expires_at: None,
    last_used: None,
    rate_limit: 1000,
    consciousness_level: 0,
};

auth.add_api_key(api_key).await?;
```
