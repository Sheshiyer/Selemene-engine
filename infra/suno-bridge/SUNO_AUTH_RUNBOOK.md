# Suno Cookie Refresh Runbook

**Trigger:** any of:
- `suno-bridge.tryambakam.space/api/get_limit` returns `401` or `403`
- bulk-gen script logs `custom_generate failed: 401`
- credits_left value drops to `0` despite no recent generation
- Sentry alarm `suno-bridge: auth-failed`

**Time to resolve:** ~5 minutes.

---

## Why this happens

Suno's web app uses session cookies that expire periodically (typically 7-30 days, sometimes faster after billing changes or password resets). The gcui-art wrapper has no auto-refresh — we must manually re-extract the cookie from a logged-in browser session.

## Steps

### 1. Open Suno + DevTools

1. Open https://suno.com/create in Chrome/Firefox.
2. Log in if not already (use your **Pro account**, not free).
3. Open DevTools (⌘⌥I on macOS, F12 on Windows).
4. Switch to the **Network** tab.
5. Click the **▶ Create** or any button that triggers a network request.

### 2. Extract the cookie

1. In DevTools Network tab, click any request to `suno.com` or `studio-api.suno.ai`.
2. Switch to the **Headers** tab.
3. Scroll to **Request Headers**.
4. Find `cookie:` (lowercase).
5. **Copy the entire value** — a long string starting with something like `__cf_bm=...`. It includes multiple `name=value` pairs separated by `; `.

### 3. Update the env var

**Vercel (preferred):**
1. Go to https://vercel.com/dashboard → your `selemene-suno-bridge` project.
2. Settings → Environment Variables → find `SUNO_COOKIE`.
3. Click ⋯ → Edit.
4. Paste the new cookie value, replacing the old one.
5. Save → click "Redeploy" on the most recent deployment (or run `vercel --prod` from the `suno-api` repo).

**Local-only (dev):**
```bash
# in suno-api/.env
SUNO_COOKIE="<paste new cookie here>"
# then restart npm run dev
```

### 4. Verify

```bash
curl https://suno-bridge.tryambakam.space/api/get_limit
```

Should return `200` with valid `credits_left`. If still `401`:
- Confirm you logged in to the **right** account
- Check the cookie didn't get truncated when pasting (it's long — easy to miss the trailing characters)
- Try logging out + back in to refresh server-side session, then re-extract

### 5. Resume any paused work

- If a `suno-bulk-gen.ts` run was interrupted: re-run with the same args. The `.suno-checkpoint.json` skips already-done ragas; fresh run picks up where it left off.
- If users hit cached `/api/v1/raga/:num/clip` responses, they're unaffected — those are served from R2, not from Suno.

---

## Prevention

- **Stay logged in** to suno.com in your everyday browser; don't clear cookies.
- **Watch the period header** — if `period: monthly` resets and `credits_left` looks wrong, check cookie validity before debugging quota math.
- **Set a Sentry alarm** on 401 from the bridge: any single 401 should page on-call.
- **Rotate proactively** every 14 days regardless — schedule a calendar reminder.

## Escalation

If cookie refresh doesn't resolve auth:
1. Check upstream `gcui-art/suno-api` issues for "401" / "auth broken" reports.
2. Check Suno's status page / Discord for service-side issues.
3. Worst case: temporarily fall back to Strudel V2.5 in the Nadabrahman UI by setting `NEXT_PUBLIC_SUNO_DISABLED=true`. Document in incident log.
