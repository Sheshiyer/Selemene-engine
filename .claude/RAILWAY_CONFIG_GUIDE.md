# Railway Configuration Guide for Selemene Engine

## Overview
This guide explains the Railway configuration for optimal deployment of the Selemene Engine API.

## Configuration File: `railway.toml`

Railway supports config-as-code through `railway.toml` in your repo root. This allows version-controlled deployment settings.

---

## Key Configuration Areas

### 1. **Build Settings**

```toml
[build]
builder = "NIXPACKS"
dockerfilePath = "Dockerfile.prod"
```

**What it does:**
- Uses Railway's default builder (auto-detects Dockerfile)
- Explicitly points to `Dockerfile.prod` for production builds
- Metal build environment is enabled by default (faster builds)

**When to customize:**
- If you need a custom build command: `buildCommand = "cargo build --release"`
- For non-Docker builds: Remove `dockerfilePath`

---

### 2. **Healthcheck Configuration**

```toml
[deploy]
healthcheckPath = "/health/live"
healthcheckTimeout = 300
```

**What it does:**
- Railway calls `https://your-domain.railway.app/health/live` before marking deployment as successful
- Waits up to 5 minutes (300s) for the app to become healthy
- If healthcheck fails, deployment is marked as failed and old version stays running

**Why 5 minutes?**
- Rust apps take time to start (especially with large binaries)
- Swiss Ephemeris data loading
- Database connection initialization

**Your healthcheck endpoint should return:**
- HTTP 200 status when healthy
- HTTP 503 when not ready

---

### 3. **Restart Policy**

```toml
restartPolicyType = "ON_FAILURE"
restartPolicyMaxRetries = 3
```

**What it does:**
- Automatically restarts container if it crashes (exit code ≠ 0)
- Tries up to 3 times before giving up
- Prevents infinite restart loops

**Options:**
- `ON_FAILURE` - Restart only on crashes (recommended)
- `ALWAYS` - Restart even on clean exits
- `NEVER` - Don't auto-restart

---

### 4. **Watch Paths (Smart Deployments)**

```toml
[deploy.watch]
include = [
  "src/**",
  "crates/**",
  "data/**",
  "Cargo.toml",
  "Cargo.lock",
  "Dockerfile.prod"
]

exclude = [
  "*.md",
  "docs/**",
  "tests/**",
  ".claude/**"
]
```

**What it does:**
- Only triggers deployments when important files change
- Prevents rebuilds when you update README or docs
- Saves build minutes and deployment time

**Example scenarios:**
- ✅ Change `src/main.rs` → Triggers deployment
- ✅ Update `Cargo.toml` → Triggers deployment
- ❌ Edit `README.md` → No deployment
- ❌ Update `.claude/notes.md` → No deployment

---

### 5. **Serverless Mode** (Optional)

```toml
[deploy]
serverless = true
```

**What it does:**
- Scales to zero when no traffic
- Wakes up on first request (cold start ~2-5s)
- Saves costs on low-traffic apps

**When to use:**
- Development/staging environments
- Low-traffic APIs
- Cost optimization

**When NOT to use:**
- Production APIs requiring instant response
- Apps with background jobs
- Services needing persistent connections

---

### 6. **Cron Jobs** (Optional)

```toml
[[deploy.cron]]
schedule = "0 0 * * *"  # Daily at midnight
command = "/app/noesis-server --task cleanup"
```

**What it does:**
- Runs scheduled tasks (like database cleanup)
- Uses standard cron syntax
- Runs in separate container instance

**Use cases:**
- Cache cleanup
- Data prefetching
- Report generation
- Database maintenance

---

### 7. **Pre-deploy Steps** (Optional)

```toml
[deploy]
preDeployCommand = "cargo test --release"
```

**What it does:**
- Runs command before deployment starts
- Deployment fails if command exits with error
- Useful for validation

**Use cases:**
- Run tests before deploying
- Database migrations
- Asset compilation
- Configuration validation

---

## Environment-Specific Configuration

Railway supports multiple environments (production, staging, etc.). You can create environment-specific configs:

```
railway.toml              # Default config
railway.production.toml   # Production overrides
railway.staging.toml      # Staging overrides
```

---

## Dashboard Settings vs Config File

Some settings can be configured in both places:

| Setting | Dashboard | Config File | Recommendation |
|---------|-----------|-------------|----------------|
| Environment Variables | ✅ | ❌ | Use Dashboard (secrets) |
| Build Command | ✅ | ✅ | Use Config File (version control) |
| Start Command | ✅ | ✅ | Use Dockerfile ENTRYPOINT |
| Healthcheck | ✅ | ✅ | Use Config File |
| Watch Paths | ✅ | ✅ | Use Config File |
| Domains | ✅ | ❌ | Use Dashboard |
| Redis/DB Add-ons | ✅ | ❌ | Use Dashboard |

**Priority:** Config file settings override dashboard settings.

---

## Recommended Setup for Selemene Engine

### Current Configuration (Optimal)

✅ **Metal Build Environment** - Faster builds
✅ **Dockerfile-based** - Full control over build process
✅ **Healthcheck enabled** - Ensures app is ready before switching traffic
✅ **Smart watch paths** - Only rebuild when code changes
✅ **Restart on failure** - Auto-recovery from crashes
✅ **5-minute healthcheck timeout** - Accounts for Rust startup time

### Optional Enhancements

**For Production:**
```toml
[deploy]
# Add pre-deploy validation
preDeployCommand = "echo 'Validating deployment...'"

# Configure graceful shutdown
teardownTimeout = 30  # Wait 30s for connections to close
```

**For Staging:**
```toml
[deploy]
serverless = true  # Scale to zero when not in use
```

---

## Monitoring Your Configuration

After pushing `railway.toml`:

1. **Check if Railway detected it:**
   ```bash
   railway logs
   ```
   Look for: "Using railway.toml configuration"

2. **Verify healthcheck:**
   ```bash
   curl https://selemene-engine-production.up.railway.app/health/live
   ```
   Should return: `{"status":"healthy"}`

3. **Test watch paths:**
   - Edit a `.md` file and push → No deployment
   - Edit `src/main.rs` and push → Triggers deployment

---

## Troubleshooting

### Healthcheck Failing
- Increase `healthcheckTimeout` to 600 (10 minutes)
- Check logs: `railway logs`
- Verify endpoint: `curl https://your-app/health/live`

### Builds Taking Too Long
- Enable Metal build environment (should be default)
- Check if cargo-chef is caching properly
- Review Dockerfile layer caching

### Unnecessary Deployments
- Review `[deploy.watch]` paths
- Add more patterns to `exclude`
- Check git history: `git log --oneline`

---

## Next Steps

1. ✅ Push `railway.toml` to your repo
2. ✅ Verify deployment uses the config
3. ✅ Test healthcheck endpoint
4. ✅ Monitor first deployment with new config
5. ⏳ Adjust timeouts if needed

---

## Resources

- [Railway Config Docs](https://docs.railway.app/deploy/config-as-code)
- [Healthcheck Guide](https://docs.railway.app/deploy/healthchecks)
- [Watch Paths](https://docs.railway.app/deploy/deployments#watch-paths)
- [Cron Jobs](https://docs.railway.app/deploy/cron-jobs)
