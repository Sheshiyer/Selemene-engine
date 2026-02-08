# Redis Fresh Setup - Clean Slate Approach

**Problem:** Can't identify which of the 4 Redis instances is active
**Solution:** Create a new Redis with clear naming, link it properly, delete old ones

---

## Step 1: Create New Redis (2 minutes)

### Option A: Railway Dashboard (Recommended)

1. Go to https://railway.app/project/11eedde4-41e6-4f51-b86b-cf77111cf592

2. Click **"+ New"** button (top right)

3. Select **"Database"** → **"Add Redis"**

4. **IMPORTANT:** When prompted for name, enter: **`Redis-Production`**
   - This name must be exact for the reference variable to work
   - Don't let Railway auto-generate a random suffix

5. Wait for provisioning (30-60 seconds)

6. You should see a new service called **"Redis-Production"** in your project

### Option B: Railway CLI

```bash
# This may auto-generate a random name - dashboard is more reliable
railway add redis
```

---

## Step 2: Update REDIS_URL Variable (1 minute)

1. In Railway dashboard, click on **"Selemene-engine"** service

2. Go to **"Variables"** tab

3. Find the **`REDIS_URL`** variable

4. Click to edit it

5. Change the value from:
   ```
   ${{37fc197c-19ef-4f5e-a371-6a4abdbee907.REDIS_URL}}
   ```

6. Change to:
   ```
   ${{Redis-Production.REDIS_URL}}
   ```

7. Click **"Save"** or **"Update"**

8. Railway will automatically trigger a redeployment

---

## Step 3: Wait for Deployment (2-3 minutes)

Railway will:
1. Rebuild the application (if needed)
2. Restart with new REDIS_URL pointing to Redis-Production
3. Run health checks

**Watch the deployment logs:**
- Railway dashboard → Selemene-engine → Deployments tab
- Wait for status to show "Active" or "Healthy"

---

## Step 4: Verify New Redis Works (30 seconds)

Run the health check script:

```bash
./scripts/railway-health-check.sh
```

**Expected output (success):**
```json
{
  "status": "healthy",
  "uptime": "120s",
  "engines_loaded": 9,
  "workflows_loaded": 6,
  "redis": "ok",
  "orchestrator": "ready"
}
```

**If you see `redis=ok`** → Success! Proceed to Step 5

**If you see `redis=down` or 502 error** → STOP! Do not delete old Redis yet

### Troubleshooting if New Redis Fails:

1. Check REDIS_URL variable spelling: must be exactly `Redis-Production`
2. Verify the new Redis service is running (green status in dashboard)
3. Check deployment logs for connection errors
4. If still broken, rollback:
   - Change REDIS_URL back to old value: `${{37fc197c-19ef-4f5e-a371-6a4abdbee907.REDIS_URL}}`
   - Wait for redeployment
   - Verify old Redis still works

---

## Step 5: Delete Old Redis Instances (3 minutes)

**ONLY DO THIS AFTER STEP 4 SUCCEEDS!**

For each old Redis service (MMs6, fPqz, OiiO, c_CR):

1. Click on the Redis service in Railway dashboard
2. Go to **"Settings"** tab
3. Scroll to **"Danger Zone"** at bottom
4. Click **"Delete Redis-XXX"** button
5. Confirm deletion
6. Repeat for all 4 old instances

**Result:**
- ✅ Only 1 Redis instance remains: **Redis-Production**
- ✅ Cost reduced from ~$20/month to ~$5/month
- ✅ Clear, unambiguous naming

---

## Step 6: Final Verification (30 seconds)

After deleting all old Redis instances:

```bash
./scripts/railway-health-check.sh
```

Should still show `redis=ok`

Check Railway variables:
```bash
railway variables | grep REDIS
```

Should show:
```
REDIS_URL=${{Redis-Production.REDIS_URL}}
```

---

## Why This Approach Works

**Old Problem:**
- Multiple Redis with random names (MMs6, fPqz, OiiO, c_CR)
- REDIS_URL referenced a UUID that's not visible in UI
- No way to identify which one is active

**New Solution:**
- Single Redis with explicit name: "Redis-Production"
- REDIS_URL uses clear reference: `${{Redis-Production.REDIS_URL}}`
- Easy to identify and manage

---

## Safety Notes

✅ **Zero downtime:** New Redis is created BEFORE old ones are deleted

✅ **Rollback possible:** If new Redis fails, old ones are still available

✅ **Railway backup:** Deleted services are kept for 7 days, can be restored

⚠️ **Cache data:** New Redis starts empty - any cached data from old Redis will be lost. This is fine for caching (will rebuild), but if you were using Redis for persistent data, you'd need to migrate it first.

---

## Cost Impact

| Item | Before | After | Savings |
|------|--------|-------|---------|
| Redis instances | 4 | 1 | 3 instances |
| Monthly cost | ~$20 | ~$5 | ~$15/month |
| Annual savings | - | - | **~$180/year** |

---

## Next Steps After Cleanup

- [ ] Document Redis-Production configuration
- [ ] Update deployment runbook
- [ ] Proceed with Phase 2: Sentry, Posthog, Admin endpoints
- [ ] Set up monitoring alerts for Redis connection health
