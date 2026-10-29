# Desktop sign-in (Sankalpa)

Password login is retired (`POST /api/v1/auth/login` returns 410). Desktop clients sign in through the system browser, using the same Cloudflare Access identity as Urania and admin. The flow follows RFC 8252 (native apps) with PKCE (RFC 7636).

## Flow

1. The app starts a one-shot HTTP listener on `127.0.0.1:<random port>` and generates a PKCE verifier and challenge plus a `state` value.
2. It opens this URL in the system browser:
   `GET /api/v1/desktop/authorize?code_challenge=…&code_challenge_method=S256&redirect_uri=http://127.0.0.1:<port>/callback&state=…`
3. Cloudflare Access authenticates the person. The handler accepts only a browser identity (Cloudflare Access, or the development bypass). A bearer JWT or API key gets `403 DESKTOP_AUTH_REQUIRES_BROWSER`, so a desktop token can never mint more sessions. The handler then issues a 60-second, single-use code and redirects (303) to the loopback callback.
4. The app calls `POST /api/v1/desktop/token` with `{grant_type: "authorization_code", code, code_verifier, redirect_uri}`. The response has:
   - an `access_token`: a Selemene JWT with `basic:access` only, valid for 1 hour, revocable by `jti`;
   - a `refresh_token`: opaque, valid for 30 days, rotated on every use.
5. To refresh, the app sends `{grant_type: "refresh_token", refresh_token}`. Presenting a refresh token that was already rotated revokes the whole session family (reuse detection).
6. To sign out, the app calls `POST /api/v1/desktop/revoke` with `{refresh_token}`, plus `POST /api/v1/auth/logout` with the access token.

Codes and refresh tokens are stored only as SHA-256 hashes (migration `042_desktop_sessions`). `/desktop/token` and `/desktop/revoke` are public and rate limited by IP.

## Operator setup (once per environment)

1. Apply migration 042: `bash scripts/apply-migrations.sh -- <psql args>`.
2. In the Cloudflare dashboard, open the Access application that protects the API host, or create one, and add the path `/api/v1/desktop/authorize`. Use the same policy as admin and Urania. Only the authorize path needs Access, because the token endpoints authenticate by code or refresh token.
3. Confirm that `CF_ACCESS_ISSUER` and `CF_ACCESS_AUDIENCE` on the API service match that Access application.
4. Smoke test:
   - Open `https://<api-host>/api/v1/desktop/authorize?code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&code_challenge_method=S256&redirect_uri=http://127.0.0.1:53682/callback&state=smoke-0123456789abcdef` in a browser.
   - After the Access login, the browser should land on `127.0.0.1:53682/callback?code=…`. A connection-refused page there is expected.

## Local and end-to-end runs

With `RUST_ENV=development` and `CF_DEV_BYPASS_TOKEN` set, a request carrying `x-noesis-dev-auth: <token>` counts as a browser identity. That stands in for Cloudflare Access locally. Production never sets `RUST_ENV=development`.
