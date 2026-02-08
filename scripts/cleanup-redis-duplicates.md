# Delete Duplicate Redis Services from Railway

**Problem:** 4 duplicate Redis services exist from failed CLI provisioning attempts:
- Redis-fPqz
- Redis-OiiO
- Redis-c_CR
- Redis-MMs6

**Keep:** Only the main "Redis" service (ID: 37fc197c-19ef-4f5e-a371-6a4abdbee907)

---

## Option 1: Railway Dashboard (Recommended - Easiest)

1. Go to https://railway.app/project/11eedde4-41e6-4f51-b86b-cf77111cf592
2. Click on each duplicate Redis service:
   - Redis-fPqz
   - Redis-OiiO
   - Redis-c_CR
   - Redis-MMs6
3. Click "Settings" tab (⚙️ icon)
4. Scroll down to "Danger Zone"
5. Click "Delete Service"
6. Confirm deletion

**Leave "Redis" (without suffix) - this is the correct one!**

---

## Option 2: Railway API (Advanced)

Railway CLI doesn't have a delete command, but we can use the GraphQL API:

### Get Service IDs:
```bash
railway whoami --json | jq -r '.id' > /tmp/railway-token.txt

# Query services via GraphQL API
curl -X POST https://backboard.railway.app/graphql/v2 \
  -H "Authorization: Bearer $(cat /tmp/railway-token.txt)" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ project(id: \"11eedde4-41e6-4f51-b86b-cf77111cf592\") { services { edges { node { id name } } } } }"
  }' | jq .
```

### Delete Service (repeat for each duplicate):
```bash
SERVICE_ID="<service-id-from-above>"

curl -X POST https://backboard.railway.app/graphql/v2 \
  -H "Authorization: Bearer $(railway whoami --json | jq -r '.id')" \
  -H "Content-Type: application/json" \
  -d "{
    \"query\": \"mutation { serviceDelete(id: \\\"$SERVICE_ID\\\") }\"
  }"
```

---

## Verification

After deletion, verify only one Redis exists:

```bash
railway service Redis
railway variables | grep REDIS_URL
```

Should show:
```
REDIS_URL=${{Redis.REDIS_URL}}
```

---

## Why Duplicates Happened

The `railway add --database redis` command was attempted multiple times due to terminal interaction issues. Each attempt created a new service instead of reusing the existing one.

This is harmless but wasteful ($5/month per Redis instance). Deleting the duplicates saves ~$20/month.
