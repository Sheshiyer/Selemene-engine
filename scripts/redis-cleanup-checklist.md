# Redis Cleanup Checklist

**Active Redis Service ID:** `37fc197c-19ef-4f5e-a371-6a4abdbee907`

---

## Step-by-Step Cleanup

### 1. Identify Active Redis (2 minutes)

Go to Railway dashboard: https://railway.app/project/11eedde4-41e6-4f51-b86b-cf77111cf592

Check each Redis service for the matching ID:

- [ ] **Redis-MMs6**
  - Click service → Settings tab
  - Check "Service ID"
  - Does it match `37fc197c-19ef-4f5e-a371-6a4abdbee907`?
  - If YES: ✅ **KEEP THIS ONE**
  - If NO: ❌ Mark for deletion

- [ ] **Redis-fPqz**
  - Click service → Settings tab
  - Check "Service ID"
  - Does it match `37fc197c-19ef-4f5e-a371-6a4abdbee907`?
  - If YES: ✅ **KEEP THIS ONE**
  - If NO: ❌ Mark for deletion

- [ ] **Redis-OiiO**
  - Click service → Settings tab
  - Check "Service ID"
  - Does it match `37fc197c-19ef-4f5e-a371-6a4abdbee907`?
  - If YES: ✅ **KEEP THIS ONE**
  - If NO: ❌ Mark for deletion

- [ ] **Redis-c_CR**
  - Click service → Settings tab
  - Check "Service ID"
  - Does it match `37fc197c-19ef-4f5e-a371-6a4abdbee907`?
  - If YES: ✅ **KEEP THIS ONE**
  - If NO: ❌ Mark for deletion

---

### 2. Delete Unused Redis (3 minutes)

For each Redis service marked ❌ for deletion:

1. Click on the Redis service
2. Go to **Settings** tab
3. Scroll down to **Danger Zone**
4. Click **"Delete Redis-XXX"**
5. Confirm deletion
6. Repeat for all 3 unused services

**Result:** Only 1 Redis service remains (the one with ID `37fc197c...`)

---

### 3. Verify Connection (30 seconds)

After deletion, verify Selemene-engine still connects:

```bash
./scripts/railway-health-check.sh
```

**Expected output:**
```json
{
  "status": "healthy",
  "redis": "ok",
  "orchestrator": "ready"
}
```

If you see `redis=ok`, you're done! ✅

If you see `redis=down`, you may have deleted the wrong one. Railway keeps deleted services for 7 days - you can restore it immediately.

---

## Cost Savings

| Before | After | Savings |
|--------|-------|---------|
| 4 Redis instances | 1 Redis instance | ~$15/month |
| ~$20/month | ~$5/month | 75% reduction |

---

## Troubleshooting

**If health check fails after deletion:**

1. Go to Railway dashboard → Project Settings → Deleted Services
2. Find the recently deleted Redis service
3. Click "Restore"
4. Wait 30 seconds for it to reconnect
5. Re-run health check

**If you're unsure which service to delete:**

Don't delete any! Instead:
1. Take a screenshot of each Redis service's Settings page (showing Service ID)
2. Share with me
3. I'll confirm which one to keep

---

## Next Steps After Cleanup

Once Redis cleanup is complete:

- [ ] Run health check to verify deployment
- [ ] Check Railway billing to confirm cost reduction
- [ ] Proceed with Phase 2: Sentry, Posthog, Admin endpoints
- [ ] Document the active Redis service name for future reference
