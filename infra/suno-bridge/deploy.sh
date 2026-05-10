#!/usr/bin/env bash
# Deploy the gcui-art/suno-api wrapper to Vercel.
#
# Why this is a script you run yourself (not Claude):
#   - Pulls external community code that will execute with your Suno cookie.
#     YOU should review the upstream commit before deploying.
#   - Cookie value should never traverse chat history.
#
# Usage:
#   1. Cookie capture: open https://suno.com/create → DevTools → Network → click any
#      request → Headers → Request Headers → copy the entire `cookie:` value.
#   2. cd infra/suno-bridge
#   3. cp .env.template .env  # then edit .env and paste your cookie
#   4. bash deploy.sh
#
# Output: a Vercel URL serving /api/get_limit, /api/custom_generate, /api/get.
#
# Safety:
#   - This script never echoes the cookie value to stdout.
#   - It only piped through `vercel env add` which reads from stdin.

set -euo pipefail

cd "$(dirname "$0")"

# ── 0. Sanity ─────────────────────────────────────────────────────────────
if [[ ! -f .env ]]; then
  echo "❌ .env not found. Run: cp .env.template .env, then edit .env"; exit 1
fi
# shellcheck disable=SC1091
source .env
if [[ -z "${SUNO_COOKIE:-}" ]]; then
  echo "❌ SUNO_COOKIE empty in .env. Capture it from suno.com DevTools first."; exit 1
fi
PIN="${SUNO_API_PIN:-v1.0.0}"

# ── 1. Clone gcui-art/suno-api into ./suno-api-build (gitignored) ─────────
if [[ ! -d suno-api-build ]]; then
  echo "▸ Cloning gcui-art/suno-api at ${PIN}…"
  git clone --depth 1 --branch "${PIN}" https://github.com/gcui-art/suno-api.git suno-api-build || {
    echo "⚠ Tag ${PIN} not found, falling back to main HEAD"
    git clone --depth 1 https://github.com/gcui-art/suno-api.git suno-api-build
  }
fi
cd suno-api-build

# ── 2. Vercel link + deploy ───────────────────────────────────────────────
echo "▸ Linking + deploying to Vercel as 'selemene-suno-bridge'…"
# Project name must be unique under your account; change below if it clashes.
vercel --yes --name selemene-suno-bridge link 2>&1 | tail -3 || true
vercel --prod --yes 2>&1 | tail -10

# ── 3. Set the SUNO_COOKIE env var (production scope) ─────────────────────
echo "▸ Setting SUNO_COOKIE in Vercel production env…"
# `vercel env add` reads value from stdin when a value is piped to it.
printf '%s' "$SUNO_COOKIE" | vercel env add SUNO_COOKIE production --yes 2>&1 | tail -2

# ── 4. Trigger a fresh deploy to bake the env var into the runtime ────────
echo "▸ Redeploying with cookie env baked in…"
vercel --prod --yes 2>&1 | grep -E "Production|Inspect|https" | tail -5

# ── 5. Verify with a quota check ──────────────────────────────────────────
URL=$(vercel ls 2>&1 | grep selemene-suno-bridge | head -1 | awk '{print $2}')
if [[ -z "$URL" ]]; then
  echo "⚠ Could not auto-detect deployment URL — check 'vercel ls' output above."
else
  echo "▸ Verifying via /api/get_limit on $URL…"
  curl -sf "https://${URL}/api/get_limit" | head -1 || echo "❌ Quota check failed — cookie may be invalid"
fi

echo ""
echo "✅ Deploy complete."
echo "   Bridge URL:  https://${URL:-<see vercel ls>}"
echo "   Next step:   add SUNO_BRIDGE_URL=https://${URL:-<...>} to apps/noesis-web/.env.local"
echo "   Custom domain (optional): vercel domains add suno-bridge.tryambakam.space selemene-suno-bridge"
