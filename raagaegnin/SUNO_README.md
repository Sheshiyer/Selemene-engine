# Suno Integration — Setup Guide

This is the "your turn" checklist after the code-side Phase 1 lands. The TypeScript modules, scripts, SQL migration, and runbooks are all committed. Below: what's already provisioned for you (R2 bucket + CDN URL) vs what still needs your hands.

> **Plan reference:** [`SUNO_INTEGRATION_PLAN.md`](./SUNO_INTEGRATION_PLAN.md) — full 52-task swarm plan.
> **Verifier:** `node apps/noesis-web/src/lib/raaga/suno/verify-suno.mjs` → must show "✅ 14/14 VERIFIED"

---

## ✅ Already provisioned (Claude did via wrangler)

| Resource | Value |
|---|---|
| **Cloudflare account ID** | `9d9d23b27f32e70ae3afb6a1aa2c0f10` |
| **R2 bucket** | `selemene-raga-clips` (created, public access enabled) |
| **R2 public CDN URL** | `https://pub-1f3a1b9dd04b4178b521c06332f81a37.r2.dev` |
| **End-to-end upload tested** | wrangler put → CDN HTTP 200 ✓ |

**No R2 API token needed for dev.** The smoke + bulk-gen scripts use `wrangler r2 object put` directly via your existing `wrangler` OAuth (they shell out via `r2-wrangler.ts`). When you eventually deploy serverless code that uploads (Phase 3), you'll mint S3-compatible keys then. For now: skip the dashboard step.

## 🔑 The ONE step that still needs your hands

### Step 1 — Capture your Suno cookie

This is the screenshot you sent. Re-do it now:
1. Open https://suno.com/create in a browser **logged in to your Pro account**
2. DevTools (⌘⌥I) → **Network** tab
3. Click any button or trigger any request
4. Click the request → **Headers** → **Request Headers**
5. Find `cookie:` and **copy the full value** (long string starting with `__client=...`)

**DO NOT paste the cookie into chat.** Keep it in your clipboard or a password manager.

### Step 3 — Deploy the Suno bridge to Vercel

```bash
cd Selemene-engine/infra/suno-bridge
cp .env.template .env
# Open .env in your editor; paste your cookie value into SUNO_COOKIE=
# Save. Then:
bash deploy.sh
```

The script:
1. Clones `gcui-art/suno-api` at pinned tag `v1.0.0` (audit it first if you want — `cat suno-api-build/pages/api/custom_generate.ts`)
2. Deploys to Vercel as `selemene-suno-bridge`
3. Sets `SUNO_COOKIE` env var (read from your local `.env`, never echoed)
4. Redeploys + curls `/api/get_limit` to verify

When done, the script prints the bridge URL. Save it as `SUNO_BRIDGE_URL`.

### Step 4 — Wire env into noesis-web

Append to `apps/noesis-web/.env.local`:

```bash
# Suno bridge (from step 3 — replace with your actual URL)
SUNO_BRIDGE_URL=https://selemene-suno-bridge.vercel.app

# That's all the env you need for dev — R2 uploads happen via wrangler OAuth.
# (Defaults for R2_ACCOUNT_ID, R2_BUCKET, R2_PUBLIC_BASE_URL are already
#  hardcoded in apps/noesis-web/src/lib/raaga/suno/r2.ts — override only if needed.)
```

### Step 5 — Apply the DB migration

Pick the right command for your migration tool:

```bash
# If you use sqlx (most likely — Selemene's pattern):
sqlx migrate run --source ./migrations --database-url "$DATABASE_URL"

# If you use a different runner, the file is at:
# migrations/028_raga_clips.sql
```

### Step 6 — Smoke test (uses ~10 of your 2,500 monthly Suno credits)

```bash
cd Selemene-engine/apps/noesis-web
pnpm install -D tsx                # only tsx needed; wrangler is global
pnpm tsx scripts/suno-smoke.ts 15
```

Expected output (~60-90 seconds):
```
[smoke] Smoke test: melakarta=15 style=ambient duration=45s
[smoke] Quota before: 2500 credits remaining
[smoke] Submitting generation…
[smoke] Submitted: 2 variants returned. IDs: abc..., def...
[smoke] Polling song abc... for completion…
[smoke] Song ready: status=streaming, duration=45s, audio_url=...
[smoke] Downloading MP3…
[smoke] Downloaded 720.3 KiB
[smoke] Uploading to R2…
[smoke] Uploaded → https://pub-1f3a1b9dd04b4178b521c06332f81a37.r2.dev/clips/ambient/15-abc...mp3
[smoke] Quota after: 2490 credits remaining (used 10)
[smoke] ✅ Smoke test complete. Open the CDN URL above in a browser to listen.
```

Open the CDN URL → hear Mayamalavagaula. If audible, sign off Phase 1 and proceed to **Phase 2 bulk gen**:

```bash
pnpm tsx scripts/suno-bulk-gen.ts ambient
# 5-min batches; resumable; ~144 credits total for all 72 ragas
```

---

## When something breaks

| Problem | Where to look |
|---|---|
| Cookie expired (401 from bridge) | [`infra/suno-bridge/SUNO_AUTH_RUNBOOK.md`](../infra/suno-bridge/SUNO_AUTH_RUNBOOK.md) |
| R2 upload fails | Verify `R2_ACCOUNT_ID` matches Cloudflare account; test with `aws s3 cp test.mp3 s3://selemene-raga-clips/test/test.mp3 --endpoint-url=https://9d9d23b27f32e70ae3afb6a1aa2c0f10.r2.cloudflarestorage.com` |
| Bulk gen stops mid-run | Re-run — `.suno-checkpoint.json` resumes |
| Generation quality poor | Edit prompt template in `apps/noesis-web/src/lib/raaga/suno/prompt.ts`; re-run for affected ragas only via `pnpm tsx scripts/suno-bulk-gen.ts ambient 15 15` |
| Roll back the whole feature | Set `NEXT_PUBLIC_SUNO_DISABLED=true` in Vercel/Railway env; Nadabrahman falls back to V2.5 Strudel |

---

## Cost reminder

- Suno Pro: $10/mo · 2,500 credits/mo · ~250 generations/mo
- Phase 2 first run: 144 credits = **6%** of monthly quota
- R2: ~$0/mo until you exceed 10GB egress (highly unlikely)
- **Steady state: Suno Pro $10/mo + R2 ~$0/mo = $10/mo total**

---

🎵 *"The synthesizer dreams the raga. The library remembers it. The API delivers it."*
