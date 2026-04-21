# Runbook: Discord OAuth Callback Policy

This note is the source of truth for Discord admin login callback handling.

## Canonical production callback URI

- `https://144.tryambakam.space/admin/login/discord-callback`

This is the canonical backend-configured `DISCORD_REDIRECT_URI` for production.
Stable production hosts must rely on this URI instead of forcing a browser-side override.

## Stable hosts that must not override

- `enantiodromia-engine-dashboard.vercel.app`
- `144.tryambakam.space`
- `selemene.tryambakam.space`

If the admin UI is loaded from one of these hosts, the frontend must not send a `redirect_uri`
override to the backend authorize or callback endpoints.

## Allowed local override behavior

Browser-side `redirect_uri` overrides are allowed only when all of the following are true:

1. The request comes from the same browser origin as the requested callback URI.
2. The callback path is one of the allowed admin callback routes:
   - `/admin/login/discord-callback`
   - `/admin/auth/discord/callback`
3. The callback URI uses `https`, or `http` only for localhost-style hosts.
4. The callback URI has no query string or fragment.

Only localhost-style origins are expected to use overrides:

- `localhost`
- `127.0.0.1`
- `0.0.0.0`

Preview domains (`*.vercel.app`, `*.railway.app`) now intentionally fall back to the canonical
backend redirect URI. This avoids Discord rejections caused by non-whitelisted ephemeral preview
callback URLs.

Trailing slashes on the callback path are accepted for validation purposes, but the allowed route
set is still only the two paths above.

## Verification

Use these checks after any auth or deployment change:

```bash
curl -sS "https://selemene.tryambakam.space/api/v1/auth/discord/authorize" \
  -H "Origin: https://enantiodromia-engine-dashboard.vercel.app" | jq -r .url

curl -sS "http://localhost:8080/api/v1/auth/discord/authorize?redirect_uri=http%3A%2F%2Flocalhost%3A3001%2Fadmin%2Flogin%2Fdiscord-callback%2F" \
  -H "Origin: http://localhost:3001" | jq -r .url
```

Expected result:

- Stable production origin resolves to the canonical `144.tryambakam.space` callback URI.
- Localhost origin can use a same-origin callback override, including a trailing slash variant.