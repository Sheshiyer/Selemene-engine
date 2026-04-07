# Runbook: Auth System Failure

Covers the top 5 auth failure modes. Target: diagnose and mitigate in under 15 minutes per scenario.

## Failure Mode 1: JWT Validation Failures

### Symptoms

- All `Authorization: Bearer <token>` requests return `401 UNAUTHORIZED`
- API key requests (`X-API-Key`) still succeed
- Error body contains `"auth_method": "jwt"`
- Logs show `Invalid JWT token: InvalidSignature` or `Invalid JWT token: ExpiredSignature`

### Diagnosis (< 5 min)

```bash
# 1. Check if JWT_SECRET is set and non-empty on Railway
railway variables | grep JWT_SECRET

# 2. Verify the API can generate a token (health proxy)
curl -sS "$NOESIS_URL/health/live"

# 3. Attempt a login to get a fresh token
curl -sS -X POST "$NOESIS_URL/api/v1/auth/login" \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@example.com","password":"TestPass1"}' | jq .token

# 4. Check Railway logs for JWT-specific errors
railway logs | grep -i 'jwt\|token\|auth' | head -20
```

### Root Causes

| Cause | Check | Fix |
|---|---|---|
| `JWT_SECRET` changed or missing | `railway variables \| grep JWT_SECRET` | Restore the previous secret value |
| Token generated with different secret | Compare secrets between environments | Ensure all services share the same `JWT_SECRET` |
| Token expired (24h TTL) | Decode token payload: `echo "<token>" \| cut -d. -f2 \| base64 -d \| jq .exp` | Client must re-login to get a fresh token |
| Clock skew between server and client | Check server time in Railway logs | Railway manages NTP; if client clock is off, token `iat` may be in the future |

### Mitigation

1. **Missing/wrong `JWT_SECRET`**: Set the correct value via `railway variables set JWT_SECRET=<value>`, then redeploy.
2. **Mass token invalidation**: All existing JWTs become invalid when `JWT_SECRET` changes. Communicate to users that they must re-login.
3. **Client fix**: Ensure clients handle 401 by re-authenticating rather than retrying the same expired token.

---

## Failure Mode 2: JWT Secret Rotation

### Context

Noesis uses HS256 (symmetric HMAC) — there is no JWKS endpoint. The signing key is the `JWT_SECRET` environment variable. Rotation invalidates every outstanding JWT.

### When to Rotate

- Secret leaked or suspected compromise
- Periodic rotation policy (quarterly recommended)

### Procedure (< 10 min)

```bash
# 1. Generate a new secret (64+ chars recommended)
NEW_SECRET=$(openssl rand -base64 48)

# 2. Set the new secret on Railway
railway variables set JWT_SECRET="$NEW_SECRET"

# 3. Redeploy to pick up the new secret
railway up --detach

# 4. Verify the API is healthy
sleep 30 && curl -sS "$NOESIS_URL/health/live"

# 5. Test login with the new secret
curl -sS -X POST "$NOESIS_URL/api/v1/auth/login" \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@tryambakam.com","password":"<password>"}' | jq .token
```

### Impact

- All existing JWT tokens become invalid immediately
- API key authentication is unaffected
- Users must re-login to get new tokens
- No data loss — only session disruption

### Rollback

If the new secret causes issues, restore the previous value:

```bash
railway variables set JWT_SECRET="<previous_secret>"
railway up --detach
```

---

## Failure Mode 3: Database (Supabase) Auth Downtime

### Symptoms

- Login (`POST /api/v1/auth/login`) returns `500` or hangs
- Registration returns `500`
- API key validation times out (5-second timeout) or returns `401`
- `GET /health/ready` reports database as degraded
- Logs show `Database error` or `API key validation timed out`

### Diagnosis (< 5 min)

```bash
# 1. Check readiness endpoint for DB status
curl -sS "$NOESIS_URL/health/ready" | jq

# 2. Check Railway logs for DB connection errors
railway logs | grep -i 'database\|postgres\|pool\|connection' | head -20

# 3. Test DB connectivity directly
railway run -- bash -c 'psql "$DATABASE_URL" -c "SELECT 1"'

# 4. Check Supabase status page
# https://status.supabase.com/

# 5. Check connection pool usage
railway run -- bash -c 'psql "$DATABASE_URL" -c "SELECT count(*) FROM pg_stat_activity WHERE datname = current_database()"'
```

### Root Causes

| Cause | Check | Fix |
|---|---|---|
| Supabase outage | [status.supabase.com](https://status.supabase.com/) | Wait for resolution; auth degrades but existing JWTs still work |
| `DATABASE_URL` missing/wrong | `railway variables \| grep DATABASE_URL` | Restore correct connection string |
| Connection pool exhaustion | `pg_stat_activity` count | See [incident-db-pool-exhaustion.md](incident-db-pool-exhaustion.md) |
| Network partition (Railway ↔ Supabase) | Railway logs show timeouts | Redeploy; if persistent, contact Railway support |

### Mitigation

1. **Existing JWTs still work**: JWT validation is stateless (HS256 with local secret). Only new logins, registrations, and API key validations require DB.
2. **API key timeout**: The 5-second timeout in `validate_from_postgres` prevents hung requests from cascading. Users get `401` with "API key validation timed out".
3. **Graceful degradation**: If DB is down, users with valid JWTs can still use all engine/workflow endpoints. Only auth-mutation endpoints (login, register, password reset) fail.

---

## Failure Mode 4: Token Expiry Cascade

### Symptoms

- Spike of `401` errors across multiple users simultaneously
- All occur approximately 24 hours after a mass-login event (e.g., after a deploy or JWT_SECRET rotation)
- API key users are unaffected

### Diagnosis (< 3 min)

```bash
# 1. Check error rate spike timing
railway logs | grep '401' | tail -30

# 2. Decode a failing token to check expiry
# (user provides the token, or extract from logs)
echo "<token>" | cut -d. -f2 | base64 -d 2>/dev/null | jq '{exp: .exp, iat: .iat, exp_human: (.exp | todate), iat_human: (.iat | todate)}'
```

### Root Causes

| Cause | Evidence | Fix |
|---|---|---|
| Normal 24h expiry | `exp` timestamp is in the past | Clients must re-authenticate |
| Post-rotation cascade | All tokens issued around the same time | Expected behavior after secret rotation |
| Client not handling token refresh | Same expired token retried repeatedly | Client-side fix: catch 401, call `/auth/login` |

### Mitigation

1. **No server-side action needed** if this is normal expiry. The 24-hour TTL is set in `AuthService::generate_jwt_token()`.
2. **Client guidance**: Clients should:
   - Store token expiry time (`exp` claim)
   - Proactively re-authenticate before expiry
   - On 401, re-login rather than retry with the same token
3. **If TTL needs adjustment**: Change `Duration::hours(24)` in `crates/noesis-auth/src/lib.rs` → `generate_jwt_token()`. Redeploy.

---

## Failure Mode 5: API Key Hash Mismatch

### Symptoms

- Specific API key returns `401` with `"auth_method": "api_key"`
- Other API keys work fine
- User confirms the key was previously working
- Logs show `API key not found or expired`

### Diagnosis (< 5 min)

```bash
# 1. Hash the user's key and check the DB
railway run -- bash -c 'psql "$DATABASE_URL" -c "
  SELECT id, user_id, tier, is_active, expires_at, last_used
  FROM api_keys
  WHERE key_hash = encode(sha256('"'"'<raw_api_key>'"'"'::bytea), '"'"'hex'"'"')
"'

# 2. If no row found, the key was never inserted or was deleted
# Check if user has ANY keys:
railway run -- bash -c 'psql "$DATABASE_URL" -c "
  SELECT id, tier, is_active, expires_at, created_at
  FROM api_keys
  WHERE user_id = '"'"'<user_uuid>'"'"'
  ORDER BY created_at DESC
"'

# 3. Check if the key was deactivated (rotated)
railway run -- bash -c 'psql "$DATABASE_URL" -c "
  SELECT id, is_active, created_at
  FROM api_keys
  WHERE user_id = '"'"'<user_uuid>'"'"' AND is_active = false
  ORDER BY created_at DESC
"'
```

### Root Causes

| Cause | Evidence | Fix |
|---|---|---|
| Key was rotated | Old key has `is_active = false`, new key exists | User must use the new key |
| Key expired | `expires_at` is in the past | Issue a new key or extend expiry |
| Key deactivated by admin | `is_active = false`, no replacement | Admin must re-activate or issue new key |
| Key never persisted (in-memory only) | No row in `api_keys` table | Re-create via seed script or admin endpoint |
| Hash algorithm mismatch | Key present but hash doesn't match | Verify SHA-256 hex encoding matches `sha256_hex()` |

### Mitigation

1. **Rotate to a new key** using the rotation script:
   ```bash
   DATABASE_URL="$DATABASE_URL" cargo run --package noesis-auth --features postgres --example rotate_api_key -- <OLD_KEY>
   ```
2. **Re-activate a deactivated key** (if rotation was accidental):
   ```bash
   railway run -- bash -c 'psql "$DATABASE_URL" -c "UPDATE api_keys SET is_active = true WHERE key_hash = encode(sha256('"'"'<raw_key>'"'"'::bytea), '"'"'hex'"'"')"'
   ```
3. **Seed new keys** if the table is empty:
   ```bash
   DATABASE_URL="$DATABASE_URL" cargo run --package noesis-auth --features postgres --example seed_api_keys
   ```

---

## Failure Mode 6: Account Lockout

### Symptoms

- User receives: `"Account temporarily locked. Try again in N seconds"`
- Login returns `401` even with correct credentials
- `locked_until` column has a future timestamp

### Diagnosis (< 3 min)

```bash
# Check user lockout status
railway run -- bash -c 'psql "$DATABASE_URL" -c "
  SELECT id, email, failed_login_attempts, locked_until, last_login_at
  FROM users
  WHERE email = '"'"'<user_email>'"'"'
"'
```

### Mitigation

```bash
# Unlock the account immediately
railway run -- bash -c 'psql "$DATABASE_URL" -c "
  UPDATE users SET failed_login_attempts = 0, locked_until = NULL
  WHERE email = '"'"'<user_email>'"'"'
"'
```

Lockout triggers after repeated failed login attempts. The lock duration is 15 minutes (set in `crates/noesis-api/src/handlers/auth.rs`).

---

## Failure Mode 7: Discord OAuth Failures

### Symptoms

- `GET /api/v1/auth/discord/authorize` returns empty or error URL
- `POST /api/v1/auth/discord/callback` returns `500`
- Users report "Discord login doesn't work"

### Diagnosis (< 5 min)

```bash
# 1. Verify Discord env vars are set
railway variables | grep -i discord

# 2. Check the authorize endpoint returns a valid URL
curl -sS "$NOESIS_URL/api/v1/auth/discord/authorize" | jq

# 3. Check Railway logs for Discord-specific errors
railway logs | grep -i 'discord\|oauth' | head -20
```

### Root Causes

| Cause | Check | Fix |
|---|---|---|
| `DISCORD_CLIENT_ID` missing | `railway variables` | Set the correct value from Discord Developer Portal |
| `DISCORD_CLIENT_SECRET` missing/wrong | `railway variables` | Reset secret in Discord Developer Portal, update Railway |
| `DISCORD_REDIRECT_URI` mismatch | Compare Railway var vs Discord app settings | Must match exactly (including protocol and path) |
| Discord API outage | [discordstatus.com](https://discordstatus.com/) | Wait for resolution |
| Redirect URI not added to Discord app | Discord Developer Portal → OAuth2 → Redirects | Add the production callback URL |

### Mitigation

1. Verify all three env vars are set: `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET`, `DISCORD_REDIRECT_URI`
2. Ensure the redirect URI in Railway matches exactly what's configured in the Discord Developer Portal OAuth2 settings
3. Test the full flow: authorize URL → Discord consent → callback with code

---

## Quick Reference

| Failure Mode | Auth Method Affected | Other Methods OK? | Server Action Needed? |
|---|---|---|---|
| JWT validation | JWT Bearer | API keys work | Only if `JWT_SECRET` changed |
| JWT secret rotation | JWT Bearer | API keys work | Planned — redeploy with new secret |
| DB downtime | Login, Register, API keys | Existing JWTs work | Wait for DB recovery |
| Token expiry cascade | JWT Bearer | API keys work | No — client must re-login |
| API key hash mismatch | API Key | JWT works | Rotate or re-issue key |
| Account lockout | Login | API keys, existing JWTs work | Unlock via DB |
| Discord OAuth | Discord login | Email login, API keys work | Check env vars |

## Architecture Reference

- **JWT**: HS256, 24-hour expiry, signed with `JWT_SECRET` env var
- **API Keys**: SHA-256 hashed, stored in PostgreSQL `api_keys` table, 5-second validation timeout
- **Passwords**: Argon2id hashed
- **Account Lockout**: 15-minute lockout after repeated failed attempts
- **Auth Code**: `crates/noesis-auth/src/lib.rs` (JWT + API key validation), `crates/noesis-api/src/handlers/auth.rs` (login, register, OAuth), `crates/noesis-api/src/middleware.rs` (auth middleware)
- **Key Rotation Script**: `cargo run --package noesis-auth --features postgres --example rotate_api_key -- <OLD_KEY>`
- **Key Seeding Script**: `cargo run --package noesis-auth --features postgres --example seed_api_keys`
