#!/usr/bin/env bash
# Dodo Payments smoke test — drives a real subscription.active event from
# Dodo's relay through Next.js → Rust → Postgres and verifies tier flips.
#
# Reviewer's manual gate before merging PR #683. Run on a workstation with
# the dev stack (Postgres + admin-web dev server + noesis-api).
#
# Prerequisites:
#   1. Postgres running:  docker compose up -d postgres
#   2. Migrations applied: see runbooks/dodo-dashboard-setup.md
#   3. .env populated with DODO_PAYMENTS_API_KEY (test mode), webhook key,
#      forward secret, product/entitlement/meter IDs.
#   4. `dodo` CLI installed and `dodo login` completed.
#   5. Both admin-web (port 3001) and noesis-api (port 8080) running:
#        Terminal A: cd apps/admin-web && npm run dev
#        Terminal B: cargo run --bin noesis-server
#
# Usage: bash runbooks/scripts/dodo-smoke-test.sh
#
# What it does:
#   1. Verifies the stack is up (Postgres, Rust /health, Next.js /api/webhook/dodo-payments)
#   2. Seeds a fresh test user in Postgres
#   3. Records baseline tier
#   4. Prompts you to fire a `subscription.active` event from the Dodo
#      dashboard (or via `dodo wh trigger`) targeting the seeded user
#   5. Polls until users.tier flips OR 60s timeout
#   6. Verifies billing_subscriptions row landed
#   7. Cleans up the test user
#
# Exits 0 on success, non-zero on any failure with a clear message.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && cd .. && pwd)"
cd "$REPO_ROOT"

# -- Helpers --
log()   { printf "▶ %s\n" "$*"; }
fail()  { printf "✗ %s\n" "$*" >&2; exit 1; }
ok()    { printf "✓ %s\n" "$*"; }

# -- 1. Stack health --
log "Checking stack health…"

# Load env (needed for DODO_PAYMENTS_* and DATABASE_URL).
if [[ ! -f .env ]]; then fail ".env missing — run runbooks/dodo-dashboard-setup.md fast-path first"; fi
set -a; source .env; set +a

DATABASE_URL="${DATABASE_URL:-postgresql://noesis_user:noesis_password@localhost:5432/noesis}"
PSQL="${PSQL:-/opt/homebrew/opt/libpq/bin/psql}"
[[ -x "$PSQL" ]] || PSQL="psql"

# Postgres
if ! docker exec noesis-postgres pg_isready -U noesis_user -d noesis > /dev/null 2>&1; then
  fail "Postgres not ready — run: docker compose up -d postgres"
fi
ok "Postgres ready"

# Rust /health
if ! curl -sf http://localhost:8080/health/live > /dev/null; then
  fail "Rust noesis-server not responding on :8080 — run: cargo run --bin noesis-server"
fi
ok "Rust noesis-server responding"

# Next.js webhook route
if ! curl -sf -X POST http://localhost:3001/api/webhook/dodo-payments \
       -H "Content-Type: application/json" -d '{}' > /dev/null 2>&1; then
  # The route should reject without proper signature, but it should at
  # least respond. Anything other than network refusal is fine.
  curl -s -X POST http://localhost:3001/api/webhook/dodo-payments \
       -H "Content-Type: application/json" -d '{}' > /dev/null \
    || fail "admin-web not responding on :3001 — run: cd apps/admin-web && npm run dev"
fi
ok "admin-web responding on :3001"

# Required env
[[ -n "${DODO_PAYMENTS_API_KEY:-}" ]] || fail "DODO_PAYMENTS_API_KEY not set in .env"
[[ -n "${DODO_PAYMENTS_WEBHOOK_KEY:-}" ]] || fail "DODO_PAYMENTS_WEBHOOK_KEY not set in .env"
[[ -n "${DODO_INTERNAL_FORWARD_SECRET:-}" ]] || fail "DODO_INTERNAL_FORWARD_SECRET not set in .env"
[[ -n "${DODO_PRODUCT_BASIC_ID:-}" ]] || fail "DODO_PRODUCT_BASIC_ID not set in .env"
ok "All required env vars present"

# -- 2. Seed test user --
TEST_UUID="$(uuidgen | tr '[:upper:]' '[:lower:]')"
TEST_EMAIL="smoke-${TEST_UUID}@selemene.test"

log "Seeding test user $TEST_UUID …"
PGPASSWORD="$POSTGRES_PASSWORD" "$PSQL" "$DATABASE_URL" -v ON_ERROR_STOP=1 -q <<SQL
INSERT INTO users (id, email, password_hash, full_name, tier,
                   consciousness_level, experience_points)
VALUES ('$TEST_UUID', '$TEST_EMAIL', 'placeholder', 'Smoke Test', 'free', 0, 0);
SQL
ok "User seeded (id=$TEST_UUID, tier=free)"

# Cleanup on exit (CASCADE removes subscriptions etc).
cleanup() {
  log "Cleaning up test user $TEST_UUID …"
  PGPASSWORD="$POSTGRES_PASSWORD" "$PSQL" "$DATABASE_URL" -q -c \
    "DELETE FROM users WHERE id = '$TEST_UUID';" 2>/dev/null || true
}
trap cleanup EXIT

# -- 3. Manual trigger prompt --
cat <<EOF

────────────────────────────────────────────────────────────────────────
Now fire a real \`subscription.active\` event from Dodo:

  Option A — Dashboard:
    1. Open Developers → Webhooks → click your webhook
    2. "Send test event"
    3. Select \`subscription.active\`
    4. **Critical:** edit the test payload's \`data.metadata.selemene_user_id\` to:
       $TEST_UUID
    5. Click "Send"

  Option B — `dodo wh listen` + `dodo wh trigger`:
    Terminal C: dodo wh listen  (point at http://localhost:3001/api/webhook/dodo-payments)
    Terminal D: dodo wh trigger → subscription.active, edit metadata.selemene_user_id

The event must reference user_id $TEST_UUID and product_id $DODO_PRODUCT_BASIC_ID.
────────────────────────────────────────────────────────────────────────

EOF

read -p "Press ENTER once you've fired the event…"

# -- 4. Poll for tier flip --
log "Polling users.tier (max 60s)…"
DEADLINE=$(($(date +%s) + 60))
while [[ $(date +%s) -lt $DEADLINE ]]; do
  CURRENT_TIER=$(PGPASSWORD="$POSTGRES_PASSWORD" "$PSQL" "$DATABASE_URL" -t -A -c \
    "SELECT tier FROM users WHERE id = '$TEST_UUID';")
  if [[ "$CURRENT_TIER" != "free" && -n "$CURRENT_TIER" ]]; then
    ok "users.tier flipped: free → $CURRENT_TIER"
    break
  fi
  sleep 2
done

if [[ "$CURRENT_TIER" == "free" || -z "$CURRENT_TIER" ]]; then
  log "Tier still '$CURRENT_TIER' after 60s. Diagnostics:"
  echo ""
  echo "Recent processed_webhook_events:"
  PGPASSWORD="$POSTGRES_PASSWORD" "$PSQL" "$DATABASE_URL" -c \
    "SELECT webhook_id, event_type, processed_at FROM processed_webhook_events ORDER BY processed_at DESC LIMIT 5;"
  echo ""
  echo "Recent billing_subscriptions:"
  PGPASSWORD="$POSTGRES_PASSWORD" "$PSQL" "$DATABASE_URL" -c \
    "SELECT user_id, status, provider, provider_subscription_id, created_at
     FROM billing_subscriptions WHERE user_id = '$TEST_UUID';"
  echo ""
  echo "Check noesis-api logs for 'subscription activated' or error lines."
  fail "Smoke test FAILED — tier did not flip within 60s"
fi

# -- 5. Verify subscription row --
SUB_COUNT=$(PGPASSWORD="$POSTGRES_PASSWORD" "$PSQL" "$DATABASE_URL" -t -A -c \
  "SELECT COUNT(*) FROM billing_subscriptions
   WHERE user_id = '$TEST_UUID' AND provider = 'dodo_payments' AND status = 'active';")
[[ "$SUB_COUNT" == "1" ]] || fail "Expected 1 active billing_subscriptions row, got $SUB_COUNT"
ok "billing_subscriptions row landed (status=active)"

# -- 6. Verify dodo_customer_id was populated --
CUSTOMER_ID=$(PGPASSWORD="$POSTGRES_PASSWORD" "$PSQL" "$DATABASE_URL" -t -A -c \
  "SELECT dodo_customer_id FROM users WHERE id = '$TEST_UUID';")
[[ -n "$CUSTOMER_ID" ]] || fail "users.dodo_customer_id not populated"
ok "users.dodo_customer_id populated: $CUSTOMER_ID"

cat <<EOF

────────────────────────────────────────────────────────────────────────
✅ SMOKE TEST PASSED

  user_id:           $TEST_UUID
  tier:              free → $CURRENT_TIER
  dodo_customer_id:  $CUSTOMER_ID
  active subs:       $SUB_COUNT

The full inbound pipeline (Standard Webhooks signature → Next.js verify →
Rust forward → idempotency → dispatch → DB mutation → tier mirror) is
verified against real Dodo crypto.

PR #683 is cleared for merge.
────────────────────────────────────────────────────────────────────────
EOF
