# Identify Which Redis Instance is Active

**Problem:** You have 4 Redis services with suffixes, but no clear "Redis" service:
- Redis-MMs6
- Redis-fPqz
- Redis-OiiO
- Redis-c_CR

**Goal:** Find which one is actually connected to Selemene-engine

---

## Method 1: Check Service ID (Fastest)

From Agent Track C output, the active Redis has:
- Service ID: `37fc197c-19ef-4f5e-a371-6a4abdbee907`
- Internal URL: `redis.railway.internal:6379`

**Steps:**

1. Go to Railway dashboard: https://railway.app/project/11eedde4-41e6-4f51-b86b-cf77111cf592

2. Click on **each** Redis service (Redis-MMs6, Redis-fPqz, Redis-OiiO, Redis-c_CR)

3. Look for the **Service ID** in the service details (usually in Settings or Overview)

4. **Match the ID:** The service with ID `37fc197c-19ef-4f5e-a371-6a4abdbee907` is the active one

5. **Keep that one**, delete the other 3

---

## Method 2: Check Environment Variables (More Reliable)

1. Go to Railway dashboard → **Selemene-engine** service

2. Click **Variables** tab

3. Look for `REDIS_URL` variable

4. Check if it's:
   - **Reference variable:** `${{Redis-XXX.REDIS_URL}}` (where XXX is one of the suffixes)
   - **Direct value:** `redis://default:...@redis.railway.internal:6379`

5. If it's a reference variable, the `Redis-XXX` part tells you which service to keep

6. If it's a direct value, proceed to Method 3

---

## Method 3: Check Redis Service Variables (Definitive)

1. Click on **each** Redis service

2. Go to **Variables** tab (or **Connect** tab)

3. Look for the **REDIS_URL** value that each Redis service exposes

4. One of them will match the pattern shown in Selemene-engine's REDIS_URL:
   ```
   redis://default:pQXKMKOpiBfHtSLFBgPNFMgKSKeKlcjv@redis.railway.internal:6379
   ```

5. The Redis service with the matching password (`pQXKMKOpiBfHtSLFBgPNFMgKSKeKlcjv`) is the active one

6. **Keep that one**, delete the other 3

---

## Method 4: Process of Elimination (If all else fails)

1. Note down all 4 Redis service names

2. Delete ONE Redis service (start with the one created most recently)

3. Check if Selemene-engine still works:
   ```bash
   ./scripts/railway-health-check.sh
   ```

4. If health check passes (redis=ok), continue deleting the next one

5. If health check fails (redis=down), **restore** the deleted service immediately:
   - Railway keeps deleted services for 7 days
   - Go to Project Settings → Deleted Services → Restore

6. Repeat until only 1 Redis remains

**⚠️ WARNING:** This method is risky. Only use if Methods 1-3 don't work.

---

## Recommended Approach

**Use Method 2** (Check Environment Variables):
1. Fastest and safest
2. Shows exactly which Redis service is linked
3. No guessing required

After identifying the correct Redis:
- Note its name (e.g., "Redis-MMs6")
- Delete the other 3 via Railway dashboard
- Verify health: `./scripts/railway-health-check.sh`

---

## Cost Savings

Each Redis instance costs **~$5/month** on Railway.

Deleting 3 unused Redis services = **~$15/month saved** ✅
