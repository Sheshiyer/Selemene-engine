# Cross-App Authentication — Fragment Token Handoff

> **ARCHIVED / RETIRED** — 2026-06-30. `noesis-web` and `biofield-web` have been
> removed from this repository; the fragment-token handoff between those two
> apps is no longer active. `admin-web` is now the only web frontend and uses
> direct Discord OAuth + session cookies on its own origin.
>
> This document is kept as a historical reference for the security model.

This document described the pattern used to pass an authenticated session from
`noesis-web` (the 17-engine dashboard) into `biofield-web` (the biofield viewer)
without requiring the user to re-authenticate via Discord.

## Motivation

`noesis-web` and `biofield-web` were deployed as **separate Vercel projects** with
separate origins. Browsers enforce the same-origin policy, so cookies and
localStorage values from one app are not readable in the other. Rather than
sharing a session cookie across origins (which requires a common parent domain
and careful SameSite configuration), we used a single JWT forwarded via the URL
fragment (`#`) — which is never sent to any server.

## Flow

```
noesis-web (3437.tryambakam.space)
        |
        |   user clicks "Biofield ↗" tab
        |   NavBar reads localStorage["noesis_api_key"]
        |   if value starts with "eyJ" (JWT), appends #token=<jwt>
        v
https://biofield.tryambakam.space/login#token=eyJ...

biofield-web login page (app/(public)/login/page.tsx)
        |
        |   useEffect reads window.location.hash
        |   clears fragment via history.replaceState BEFORE any async work
        |   calls verifyToken(token) → GET /api/v1/users/me
        |   on success: stores BiofieldAuthSession, redirects to /viewer
        |   on failure: silently shows normal Discord login form
        v
/viewer (authenticated)
```

## Security properties

| Property | How achieved |
|---|---|
| Token never sent to server via URL | HTTP spec: fragment is not included in requests |
| Token not in browser history | `history.replaceState` clears it synchronously before any `await` |
| Only JWTs forwarded, not API keys | NavBar checks `eyJ` prefix; `nk_` API keys are not forwarded |
| Invalid/expired tokens don't loop | Invalid tokens silently fall through to normal login — no redirect back |
| JWT validated before session created | `verifyToken()` calls `/api/v1/users/me`; only a valid Bearer token succeeds |
| No CSRF risk | No state change triggered by the fragment alone; requires server round-trip |

## Code locations (retired)

| File | Role | Status |
|---|---|---|
| `apps/noesis-web/src/components/NavBar.tsx` | Generated `getBiofieldHref()` — reads localStorage, appends `#token=<jwt>` if JWT | Removed with `noesis-web` |
| `apps/biofield-web/app/(public)/login/page.tsx` | Consumed `#token` fragment; cleared it; verified; redirected or fell through | Removed with `biofield-web` |
| `apps/biofield-web/src/lib/api.ts` | `verifyToken(token)` → called `/api/v1/users/me` with `Authorization: Bearer` | Removed with `biofield-web` |
| `apps/biofield-web/src/lib/discord-oauth.ts` | `STABLE_HOSTS` set — controlled whether callback override is sent | Removed with `biofield-web` |

## Discord OAuth for admin-web

`admin-web` has its own Discord OAuth flow. The Rust backend validates redirect URIs against:

```
ALLOWED_DISCORD_CALLBACK_PATHS = [
    "/admin/login/discord-callback",
    "/admin/auth/discord/callback",
    "/login/discord-callback",
    "/auth/discord/callback",
]
```

Session state is stored in an HTTP-only cookie on the `admin-web` origin.


For **stable production** hosts (`selemene.tryambakam.space`, `144.tryambakam.space`),
the server uses its configured `DISCORD_REDIRECT_URI` env var (currently pointing to
the admin dashboard).

For **biofield-web hosts** (`biofield.tryambakam.space`, `biofield-web.vercel.app`),
the client dynamically passes `redirect_uri=https://<host>/login/discord-callback`.
The Rust backend validates:
1. The path is in `ALLOWED_DISCORD_CALLBACK_PATHS` ✓
2. The `Origin` header matches the requested redirect URI host (same-origin check)

Both `https://biofield.tryambakam.space/login/discord-callback` and
`https://biofield-web.vercel.app/login/discord-callback` must be registered in
the **Discord Developer Portal** under OAuth2 → Redirects.

## Environment variables (Railway)

| Variable | Value |
|---|---|
| `ALLOWED_ORIGINS` | Comma-separated list including `https://biofield.tryambakam.space` and `https://biofield-web.vercel.app` |
| `DISCORD_REDIRECT_URI` | Points to the admin dashboard; biofield-web overrides this dynamically |

## Invariants (do not break)

1. `history.replaceState` is called **before** the first `await` in the token
   consumer `useEffect`. If moved after, the fragment appears in browser history.
2. The `getBiofieldHref()` function checks `eyJ` prefix. If the check is removed,
   API keys (`nk_...`) would be forwarded and rejected by `verifyToken()` — causing
   a confusing silent failure.
3. `biofield.tryambakam.space` must **not** be in `STABLE_HOSTS`. If added, the
   client will not send a redirect_uri override, and Discord will redirect to the
   admin dashboard instead of the biofield viewer.
