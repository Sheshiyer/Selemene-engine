# Railway + Cloudflare Setup Checklist
**Generated:** 2026-02-08
**Status:** Ready for execution
**Purpose:** Unblock Railway deployment before Wave 2

---

## 🚨 Railway Build Failure - Root Cause Analysis

Your Railway build is likely failing due to ONE of these reasons:

### 1. Missing Environment Variables (Most Likely)
Railway needs certain env vars **at build time** or **at startup**. Without them, the app crashes.

**Quick check:** Look at Railway build logs for errors like:
- `DATABASE_URL not found` → sqlx trying to connect at startup
- `JWT_SECRET not configured` → ApiConfig validation failing
- `missing field ALLOWED_ORIGINS` → environment parsing error

### 2. Dockerfile Path Issue
Railway might not be detecting `Dockerfile.prod`. Check:
- Railway dashboard → Settings → Builder → Dockerfile path = `Dockerfile.prod`
- Or verify `railway.toml` is present (we created it in Wave 1)

### 3. Health Check Timeout
If the app starts but Railway marks it as "unhealthy":
- Health check path: `/health` (configured in railway.toml)
- Timeout: 5 seconds
- The app must respond to `GET /health` within 5s

**Next step:** Check Railway build logs. If you see environment-related errors, proceed with the env vars below.

---

## ✅ Required Railway Environment Variables

Copy these into Railway Dashboard → Variables tab. Replace `[PLACEHOLDER]` values.

### Core Application Settings

```bash
# Runtime environment
RUST_ENV=production

# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info
LOG_FORMAT=json
```

### Authentication & Security

```bash
# JWT signing key (GENERATED SECURELY - use this value)
JWT_SECRET=yM9HmenTks5FCXMsgXcUrjkhRLSu2CnGCU6egQcODcSdgSa87p7K4nToBg2XULKd

# JWT expiry in seconds
JWT_EXPIRY=3600
```

### Database (Supabase)

```bash
# Supabase Postgres connection string
# FORMAT: postgresql://postgres.PROJECT_REF:[PASSWORD]@aws-0-[REGION].pooler.supabase.com:6543/postgres?sslmode=require
#
# HOW TO GET THIS:
# 1. Go to Supabase Dashboard → Project Settings → Database
# 2. Copy "Connection Pooling" → "Connection string" (NOT "Direct connection")
# 3. Use Pooler (port 6543) for better connection management
# 4. Replace [YOUR-PASSWORD] with your Supabase database password
#
DATABASE_URL=postgresql://postgres.PROJECT_REF:[YOUR-PASSWORD]@aws-0-us-west-1.pooler.supabase.com:6543/postgres?sslmode=require
```

**⚠️ CRITICAL:** If you have Supabase MCP connected, you can get the exact connection string by running:
```bash
# Check if MCP provides connection string
echo $SUPABASE_DB_URL
# Or use Supabase CLI
supabase projects list
supabase db show --project-ref [PROJECT_REF]
```

### CORS Origins

```bash
# Allowed CORS origins (comma-separated)
# Include Railway preview URLs AND production domain
ALLOWED_ORIGINS=https://tryambakam.space,https://*.railway.app
```

### FreeAstrologyAPI Integration

```bash
# Your FreeAstrologyAPI.com API key
# Get from: https://freeastrologyapi.com/dashboard
FREE_ASTROLOGY_API_KEY=[YOUR_API_KEY_HERE]

# API configuration
FREE_ASTROLOGY_API_BASE_URL=https://json.freeastrologyapi.com
FREE_ASTROLOGY_API_TIMEOUT=30
FREE_ASTROLOGY_API_RETRY_COUNT=3

# Rate limiting (free tier: 50/day, 1/sec)
FREE_ASTROLOGY_RATE_LIMIT_PER_DAY=50
FREE_ASTROLOGY_RATE_LIMIT_PER_SECOND=1
FREE_ASTROLOGY_RATE_LIMIT_BUFFER=5

# Caching strategy
FREE_ASTROLOGY_CACHE_BIRTH_TTL=0
FREE_ASTROLOGY_CACHE_DAILY_TTL=86400
FREE_ASTROLOGY_CACHE_TRANSIT_TTL=3600
FREE_ASTROLOGY_PREFETCH_DAYS=7

# Engine provider
VEDIC_ENGINE_PROVIDER=api
VEDIC_ENGINE_FALLBACK_ENABLED=true
VEDIC_ENGINE_FALLBACK_ON_RATE_LIMIT=true
```

### Data Paths (Railway auto-mounts these from Docker)

```bash
SWISS_EPHEMERIS_PATH=/app/data/ephemeris
DATA_PATH=/app/data
WISDOM_DOCS_PATH=/app/data/wisdom-docs
```

### Rate Limiting

```bash
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=60
```

### Feature Flags

```bash
ENABLE_METRICS=true
ENABLE_WITNESS=true
```

---

---

## ✅ Railway Configuration (railway.toml)

**Status:** ✅ Created and committed

**Configuration file:** `railway.toml` (in repo root)
**Documentation:** `.claude/RAILWAY_CONFIG_GUIDE.md`

**Key settings configured:**
- ✅ **Healthcheck**: `/health/live` endpoint with 5-minute timeout
- ✅ **Restart policy**: Auto-restart on failure (max 3 retries)
- ✅ **Smart deployments**: Only rebuild when code/data changes
  - Triggers: `src/`, `crates/`, `data/`, `Cargo.*`, `Dockerfile.prod`
  - Ignores: `*.md`, `docs/`, `tests/`, `.claude/`, `.context/`
- ✅ **Builder**: Dockerfile.prod with Metal build environment

**Benefits:**
- Prevents unnecessary rebuilds when updating documentation
- Ensures app is healthy before switching traffic
- Auto-recovers from crashes
- Version-controlled deployment settings

**To verify it's working:**
```bash
# After deployment, check logs for:
railway logs | grep "railway.toml"
# Should see: "Using railway.toml configuration"
```

---

## 🔧 Railway Redis Add-on (Provision Now)

Redis is required for:
- L2 cache (ephemeris data, API responses)
- Rate limiting counters (Wave 2 - deferred but provision now)
- Session storage (future)

**How to provision:**

1. Railway Dashboard → Your Project → "+ New"
2. Select "Database" → "Redis"
3. Railway will auto-inject `REDIS_URL` environment variable
4. No manual configuration needed

**After provisioning, verify:**
- `REDIS_URL` appears in Variables tab
- Format: `redis://[host]:[port]` or `rediss://[host]:[port]` (SSL)

**Additional Redis config (if needed):**
```bash
REDIS_POOL_SIZE=10
CACHE_L1_SIZE=268435456  # 256MB
CACHE_L1_TTL=3600        # 1 hour
CACHE_L2_TTL=86400       # 24 hours
```

---

## 🌐 Cloudflare DNS + Cache Rules Setup

### Option 1: Using Wrangler CLI (Recommended)

If you're logged into Wrangler:

```bash
# Login to Cloudflare (if not already)
wrangler login

# Add CNAME record pointing to Railway
wrangler dns add tryambakam.space --type CNAME --name @ --content [RAILWAY_URL] --proxied true

# Example Railway URL format: selemene-engine-production.up.railway.app
```

### Option 2: Cloudflare Dashboard (Manual)

1. **Add CNAME Record:**
   - Go to: Cloudflare Dashboard → tryambakam.space → DNS → Records
   - Click "Add record"
   - Type: `CNAME`
   - Name: `@` (or `api` if you want `api.tryambakam.space`)
   - Target: `[YOUR-RAILWAY-URL].railway.app` (get from Railway dashboard)
   - Proxy status: **Proxied** (orange cloud) ✅
   - TTL: Auto
   - Click "Save"

2. **Configure SSL/TLS:**
   - Go to: SSL/TLS → Overview
   - Set to: **Full (strict)** (Railway has valid SSL cert)

3. **Cache Rules:**
   - Go to: Rules → Page Rules → "+ Create Page Rule"

   **Rule 1: Health Check Caching**
   ```
   URL: tryambakam.space/health*
   Settings:
     - Cache Level: Cache Everything
     - Edge Cache TTL: 30 seconds
   ```

   **Rule 2: API Docs Caching**
   ```
   URL: tryambakam.space/api/docs*
   Settings:
     - Cache Level: Cache Everything
     - Edge Cache TTL: 1 hour
   ```

   **Rule 3: API Bypass Cache**
   ```
   URL: tryambakam.space/api/v1/*
   Settings:
     - Cache Level: Bypass
   ```

### Option 3: Using Cloudflare API (Automated)

If you have a Cloudflare API token:

```bash
# Set your API token
export CLOUDFLARE_API_TOKEN="[YOUR_TOKEN]"
export ZONE_ID="[YOUR_ZONE_ID]"

# Add DNS record
curl -X POST "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records" \
  -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
  -H "Content-Type: application/json" \
  --data '{
    "type": "CNAME",
    "name": "@",
    "content": "[RAILWAY_URL].railway.app",
    "proxied": true,
    "ttl": 1
  }'
```

**Get your Zone ID:**
```bash
curl -X GET "https://api.cloudflare.com/client/v4/zones?name=tryambakam.space" \
  -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN"
```

---

## 🚀 Deployment Verification Checklist

After setting all env vars and provisioning Redis:

1. **Trigger Railway Deployment:**
   - Railway auto-deploys on git push to main
   - Or: Railway Dashboard → Deployments → "Redeploy"

2. **Watch Build Logs:**
   - Look for: "Compiling noesis-api"
   - Build should complete in 20-25 minutes (first time)
   - Subsequent builds: 2-5 minutes (cargo-chef caching)

3. **Check Health Endpoints:**
   ```bash
   # Get your Railway URL from dashboard, then:
   curl https://[YOUR-RAILWAY-URL].railway.app/health
   # Expected: {"status":"healthy","engines_loaded":9,"workflows_loaded":6}

   curl https://[YOUR-RAILWAY-URL].railway.app/health/ready
   # Expected: {"redis":"ok","orchestrator":"ready"}
   ```

4. **Verify Domain Resolution:**
   ```bash
   curl https://tryambakam.space/health
   # Should return same as Railway URL (once DNS propagates)
   ```

5. **Check Railway Logs:**
   - Railway Dashboard → Logs
   - Look for: `Starting noesis-api on 0.0.0.0:8080`
   - No errors about missing env vars

---

## 🔍 Common Issues & Solutions

### Issue: "DATABASE_URL not found"
**Solution:** Verify DATABASE_URL is set in Railway Variables tab. Check for typos in connection string format.

### Issue: "Failed to connect to database"
**Solution:**
- Use Supabase **Pooler** connection string (port 6543), not direct connection (port 5432)
- Verify password is correct
- Check `?sslmode=require` is appended

### Issue: "Redis connection failed"
**Solution:**
- Verify Redis add-on is provisioned
- Check `REDIS_URL` environment variable exists
- Railway sometimes takes 30-60s to inject REDIS_URL after provisioning

### Issue: "Health check timeout"
**Solution:**
- Verify `/health` endpoint exists (it does, from codebase)
- Check if app is crashing on startup (look at logs)
- Increase `healthcheckTimeout` in railway.toml to 10 seconds temporarily

### Issue: Build succeeds but app crashes
**Solution:** Check Railway logs for startup errors. Common causes:
- Missing required env var
- Database connection failure
- Port mismatch (ensure PORT=8080 or Railway sets $PORT)

---

## 📋 Quick Copy-Paste Checklist

- [ ] Copy all env vars to Railway Dashboard → Variables
- [ ] Replace `[PLACEHOLDER]` values with real credentials
- [ ] Provision Railway Redis add-on
- [ ] Verify `REDIS_URL` auto-injected
- [ ] Add Cloudflare CNAME record pointing to Railway URL
- [ ] Set Cloudflare SSL to "Full (strict)"
- [ ] Configure Cloudflare cache rules (3 rules)
- [ ] Trigger Railway deployment (git push or manual redeploy)
- [ ] Watch build logs for success
- [ ] Test `/health` and `/health/ready` endpoints
- [ ] Verify domain resolution after DNS propagation (can take 5-30 min)

---

## 🎯 What's Missing from User

Please provide these values so I can help you configure:

1. **Supabase DATABASE_URL:**
   - Format: `postgresql://postgres.PROJECT_REF:[PASSWORD]@aws-0-[REGION].pooler.supabase.com:6543/postgres?sslmode=require`
   - Get from: Supabase Dashboard → Settings → Database → "Connection Pooling"

2. **FreeAstrologyAPI Key:**
   - Get from: https://freeastrologyapi.com/dashboard
   - Free tier is fine (50 requests/day)

3. **Railway URL (after first deploy):**
   - Format: `[PROJECT-NAME]-production.up.railway.app`
   - Needed for Cloudflare CNAME target

---

**Next Step:** Copy the env vars above into Railway, provision Redis, then let me know if the build succeeds or what error you see!
