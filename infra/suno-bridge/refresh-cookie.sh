#!/usr/bin/env bash
# Refresh the Suno session cookie and redeploy.
#
# Usage:
#   1. Go to suno.com in Arc → DevTools → Network → click any request
#      → right-click → "Copy as cURL"
#   2. Save to curl.md at repo root (overwrite the old one)
#   3. Run: bash infra/suno-bridge/refresh-cookie.sh
#
# What this does:
#   - Extracts the fresh cookie from curl.md
#   - Updates infra/suno-bridge/.env
#   - Updates Vercel env var and redeploys selemene-suno-bridge
#   - Verifies /api/get_limit returns quota data

set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BRIDGE_DIR="$(cd "$(dirname "$0")" && pwd)"

# ── 1. Extract cookie from curl.md ──────────────────────────────────────────
CURL_MD="${REPO_ROOT}/curl.md"
if [[ ! -f "$CURL_MD" ]]; then
  echo "❌ curl.md not found at $REPO_ROOT. Paste the cURL export there first."
  exit 1
fi

# Grab the -b '...' value
COOKIE=$(python3 -c "
import re, sys
text = open('${CURL_MD}').read()
m = re.search(r\"-b '([^']+)'\", text)
if not m:
    print('ERROR: no -b cookie found', file=sys.stderr); sys.exit(1)
print(m.group(1))
")

if [[ -z "$COOKIE" ]]; then
  echo "❌ Could not extract cookie from curl.md"
  exit 1
fi
echo "✔ Cookie extracted (${#COOKIE} chars)"

# ── 2. Update .env ───────────────────────────────────────────────────────────
python3 - << PYEOF
import re
env = open('${BRIDGE_DIR}/.env').read()
# Replace or insert SUNO_COOKIE line
if 'SUNO_COOKIE=' in env:
    env = re.sub(r"SUNO_COOKIE='[^']*'", "SUNO_COOKIE='${COOKIE}'", env)
else:
    env = "SUNO_COOKIE='${COOKIE}'\n" + env
open('${BRIDGE_DIR}/.env', 'w').write(env)
PYEOF
echo "✔ .env updated"

# ── 3. Update Vercel env var and redeploy ────────────────────────────────────
cd "${BRIDGE_DIR}/suno-api-build"
echo "▸ Updating Vercel SUNO_COOKIE…"
printf '%s' "$COOKIE" | npx vercel env rm SUNO_COOKIE production --yes 2>/dev/null || true
printf '%s' "$COOKIE" | npx vercel env add SUNO_COOKIE production --yes 2>&1 | tail -2

echo "▸ Redeploying…"
npx vercel --prod --yes 2>&1 | grep -E "Production:|Aliased:|https://suno-api-build" | head -5

# ── 4. Verify ────────────────────────────────────────────────────────────────
echo "▸ Verifying /api/get_limit…"
sleep 5
RESULT=$(curl -sf "https://suno-api-build.vercel.app/api/get_limit" 2>&1 || echo "FAILED")
echo "$RESULT" | head -1
if echo "$RESULT" | grep -q '"credits_left"'; then
  echo "✅ Bridge live with fresh cookie"
else
  echo "⚠  Unexpected response — cookie may still be propagating or expired"
fi

echo ""
echo "Next: paste your SUPABASE_SERVICE_ROLE_KEY into .env, then run:"
echo "  SUNO_BRIDGE_URL=https://suno-api-build.vercel.app bun ts-engines/scripts/suno-smoke.ts 15"
