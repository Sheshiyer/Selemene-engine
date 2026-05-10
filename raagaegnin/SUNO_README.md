# Suno Integration — Setup Guide

This is the "your turn" checklist after the code-side Phase 1 lands. The TypeScript modules, scripts, SQL migration, and runbooks are all committed. To run the smoke test (S-015) you need to provision the credentials below.

> **Plan reference:** [`SUNO_INTEGRATION_PLAN.md`](./SUNO_INTEGRATION_PLAN.md) — the full 52-task swarm-architect plan.
> **Verifier:** `node apps/noesis-web/src/lib/raaga/suno/verify-suno.mjs` — confirms all 288 prompts (72 ragas × 4 styles) generate cleanly and ≤ 300 chars.

---

## Phase 1 status: **code-complete · awaiting credentials**

| Task | What's done (this branch) | What you need to do |
|---|---|---|
| **S-001** Pin gcui-art/suno-api version | Deploy template documented | Fork the repo + pin a tag |
| **S-002** Deploy bridge to Vercel | Deploy commands in `infra/suno-bridge/README.md` | Run `vercel --prod` |
| **S-003** Custom domain | Instructions documented | Add CNAME + run `vercel domains add` |
| **S-004** Capture Suno cookie | Extraction steps in runbook | Open suno.com → DevTools → Network → copy `Cookie:` header → paste into Vercel env as `SUNO_COOKIE` |
| **S-005** Cookie-refresh runbook | `infra/suno-bridge/SUNO_AUTH_RUNBOOK.md` | Read, schedule rotation reminder |
| **S-006** Sentry alarm on 401 | — | Wire to your existing Sentry/log infra |
| **S-007** Provision R2 bucket | — | Cloudflare dashboard → R2 → create `selemene-raga-clips` bucket |
| **S-008** R2 lifecycle policy | — | No expiry, public-via-CDN |
| **S-009** R2 credentials env | — | Add to Vercel/Railway: `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`, `R2_PUBLIC_BASE_URL` |
| **S-010** Migration `028_raga_clips.sql` | ✓ committed | Run `sqlx migrate run` (or your migration tool) |
| **S-011** Indices + trigger for `approved_at` | ✓ included in migration | (above) |
| **S-012** Rust `RagaClip` model in `noesis-core` | — *deferred to Phase 3 API task* | (Phase 3 task S-035) |
| **S-013** Prompt template `buildSunoPrompt` | ✓ `apps/noesis-web/src/lib/raaga/suno/prompt.ts` | — |
| **S-014** Hand-review canonical prompts | ✓ verifier prints 5 canonical samples | Run verifier; review console output |
| **S-015** Smoke gen script | ✓ `apps/noesis-web/scripts/suno-smoke.ts` | After steps S-002, S-004, S-007, S-009: `pnpm tsx apps/noesis-web/scripts/suno-smoke.ts 15` |
| **S-016** Verify CDN URL plays | — | Open the CDN URL printed by smoke script in a browser |
| **S-017** This README | ✓ you are here | — |
| **S-018** Phase-1 sign-off | — | After S-015 + S-016 succeed |

## Quickstart (after creds are set up)

```bash
# 1. Verify the prompt template + migration are ready
cd apps/noesis-web
node src/lib/raaga/suno/verify-suno.mjs
# → expect "✅ Suno P1 contracts VERIFIED"

# 2. Apply the DB migration to Selemene's Postgres
cd ../..
sqlx migrate run --source ./migrations
# → migration 028_raga_clips applied

# 3. Set up the bridge + R2 envs (see infra/suno-bridge/README.md)
# Once ready, copy the env keys into apps/noesis-web/.env.local

# 4. Smoke test ONE raga
cd apps/noesis-web
pnpm install -D tsx @aws-sdk/client-s3   # one-time
pnpm tsx scripts/suno-smoke.ts 15
# → ~60 seconds, prints CDN URL on success

# 5. If smoke succeeds, kick off bulk gen for ambient style on all 72 ragas
pnpm tsx scripts/suno-bulk-gen.ts ambient
# → ~5-10 minutes per batch of 5; resumable via .suno-checkpoint.json
# → uses ~144 credits (≈6% of monthly Pro budget)
```

## Cost reminder

- Suno Pro: $10/mo · 2,500 credits · ~250 generations/month
- Each generation = 10 credits, returns 2 song variants
- 72 ragas × 1 take = **144 credits = 6% of monthly quota** (well within budget)
- Phase 4 (3 more styles) = 216 more = spread over months
- R2: ~$5/mo for ~250 MB storage + free egress under 10TB
- **Steady state: ~$15/mo total**

## What this gets you

After the smoke test succeeds and bulk gen completes:
- All 72 melakartas have one approved Suno-rendered ambient clip in R2
- Each clip is 30-60s of just-instrumental Indian classical raga audio
- Served via Cloudflare CDN with 1y immutable cache
- Available at `https://clips.tryambakam.space/clips/ambient/15-{songId}.mp3` (etc.)
- Phase 3 wires this into the Nadabrahman UI as the primary playback path, with V2.5 Strudel as fallback

## When something breaks

| Problem | Where to look |
|---|---|
| Cookie expired (401 from bridge) | [`infra/suno-bridge/SUNO_AUTH_RUNBOOK.md`](../infra/suno-bridge/SUNO_AUTH_RUNBOOK.md) |
| R2 upload fails | Check `R2_ACCOUNT_ID` matches your Cloudflare account; test with `aws s3 cp` |
| Bulk gen stops mid-run | Just re-run — `.suno-checkpoint.json` resumes |
| Generation quality poor | Edit prompt template in `apps/noesis-web/src/lib/raaga/suno/prompt.ts`; re-run for affected ragas only via `pnpm tsx scripts/suno-bulk-gen.ts ambient 15 15` |
| Rolling back the whole feature | Set `NEXT_PUBLIC_SUNO_DISABLED=true` in Vercel/Railway env; Nadabrahman falls back to V2.5 Strudel |

---

🎵 *"The synthesizer dreams the raga. The library remembers it. The API delivers it."*
