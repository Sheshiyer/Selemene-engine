# Suno Bridge — gcui-art/suno-api deployment

This folder documents how Selemene deploys the [gcui-art/suno-api](https://github.com/gcui-art/suno-api) wrapper as `suno-bridge.tryambakam.space`. The bridge is the **only** point of contact between Selemene and Suno's unofficial API surface — keeping the cookie + retry + auth logic in one place.

## Architecture

```
                                    ┌──────────────────────┐
  Selemene backend / Sankalpa       │  suno-bridge         │
  (consumer TBD)           ───►     │  (Vercel)            │
                                    │                      │
                                    │  - /api/get_limit    │
                                    │  - /api/custom_gen.. │   ┌─────────┐
                                    │  - /api/get          │──►│ suno.com│
                                    │                      │   └─────────┘
                                    │  env: SUNO_COOKIE    │
                                    └──────────────────────┘
```

## One-time deploy (S-001 → S-003)

1. **Fork or clone** `gcui-art/suno-api` to your own repo (recommended: pin to a known-good tag).

   ```bash
   gh repo fork gcui-art/suno-api --clone --remote
   cd suno-api
   git checkout v1.0.0  # or latest stable tag
   ```

2. **Deploy to Vercel:**

   ```bash
   vercel --prod
   ```

   Choose project name like `selemene-suno-bridge`. Vercel will assign you a `*.vercel.app` URL initially.

3. **Add the Suno cookie** as an env var in Vercel:

   - **Get your cookie:** open https://suno.com/create in your browser → DevTools → Network tab → make any request → Headers → Request Headers → copy the entire `Cookie:` value (string starting with `__cf_bm=`, etc.)
   - In Vercel project settings → Environment Variables → add:
     - **Name:** `SUNO_COOKIE`
     - **Value:** (paste the entire cookie string)
     - **Environment:** Production
   - Trigger a redeploy: `vercel --prod` again, or click "Redeploy" in Vercel UI

4. **Custom domain (optional but recommended):**

   ```bash
   vercel domains add suno-bridge.tryambakam.space selemene-suno-bridge
   ```

   Add the CNAME record per Vercel's DNS instructions.

5. **Verify:**

   ```bash
   curl https://suno-bridge.tryambakam.space/api/get_limit
   ```

   Expected response:
   ```json
   {
     "credits_left": 2500,
     "period": "monthly",
     "monthly_limit": 2500,
     "monthly_usage": 0
   }
   ```

   If `credits_left < 2400`, your cookie may be wrong / for a different account.

## Pinning the upstream version (S-001)

The wrapper is community-maintained — upstream may break Suno auth without notice. Always pin:

1. Fork `gcui-art/suno-api` to your org.
2. Use a fixed tag in production: `git checkout v1.0.0` before `vercel --prod`.
3. Subscribe to upstream release notifications.
4. When upgrading, smoke-test on a preview deployment before promoting to prod.

## Wiring a client to use the bridge

> **Status:** `apps/noesis-web` has been retired. The Suno bridge remains a backend-only service until the Sankalpa desktop integration is ready.

Set `SUNO_BRIDGE_URL` in your backend environment or in `sankalpa/.env.local`:

```
SUNO_BRIDGE_URL=https://suno-bridge.tryambakam.space
```

The former Suno client was at `apps/noesis-web/src/lib/raaga/suno/client.ts` (deleted).

## Testing the full chain

After all env vars are set, test from a backend or scripts directory that still has access to the Suno client code:

```bash
SUNO_SMOKE_DRY_RUN=1 pnpm tsx scripts/suno-smoke.ts 15  # prints prompt, no API call
pnpm tsx scripts/suno-smoke.ts 15                        # full smoke (uses ~10 credits)
```

The original smoke scripts lived under `apps/noesis-web/scripts/` and have been retired with the app.

## See also

- `SUNO_AUTH_RUNBOOK.md` — what to do when the cookie expires (it will, every 7-30 days)
- `../../raagaegnin/SUNO_INTEGRATION_PLAN.md` — full 52-task swarm-architect plan
- `../../raagaegnin/SUNO_README.md` — top-level user setup guide
