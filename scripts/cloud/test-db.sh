#!/usr/bin/env bash
# Start the container's local Postgres 16 and (re)create the noesis_test
# database with every migration applied, for the DB-backed cargo tests.
# Cloud sessions have no running services, so run this once per session:
#   bash scripts/cloud/test-db.sh && cargo test -p noesis-api --test billing_e2e_tests -- --test-threads=1
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."
db="${TEST_DB_NAME:-noesis_test}"

pg_ctlcluster 16 main start 2>/dev/null || true
for _ in $(seq 1 20); do
  pg_isready -q -h localhost && break
  sleep 0.5
done

# Trust local TCP connections so postgres://postgres@localhost works without a password.
hba=/etc/postgresql/16/main/pg_hba.conf
if ! grep -q "selemene-cloud-test" "$hba"; then
  sed -i '1i host all all 127.0.0.1/32 trust # selemene-cloud-test\nhost all all ::1/128 trust # selemene-cloud-test' "$hba"
  pg_ctlcluster 16 main reload
fi

psql -q -h localhost -U postgres -c "DROP DATABASE IF EXISTS $db" -c "CREATE DATABASE $db"
bash scripts/apply-migrations.sh --migrations-dir migrations -- -q -h localhost -U postgres -d "$db"
echo "ready: postgres://postgres@localhost/$db"
