#!/usr/bin/env bash
# Start the container's local Postgres 16 and (re)create the noesis_test
# database with every migration applied, for the DB-backed cargo tests.
# Cloud sessions have no running services, so run this once per session:
#   bash scripts/cloud/test-db.sh && cargo test -p noesis-api --test billing_e2e_tests -- --test-threads=1
# The shared-identity RLS tests also need the bounded runtime role it prints:
#   TEST_SHARED_RUNTIME_URL=postgres://noesis_api_runtime@localhost/noesis_test
# The noesis-data witness catalog test needs its own isolated database:
#   WITNESS_TEST_DATABASE_URL=postgres://postgres@127.0.0.1:5432/noesis_identity_test \
#     cargo test -p noesis-data --test witness_catalog_integration
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

# Each database gets every migration plus the bounded API runtime role
# (NOSUPERUSER, NOBYPASSRLS) for the shared-identity RLS tests, provisioned
# with the same script operators run in production. Local trust auth means it
# needs no password here.
provision() {
  psql -q -h localhost -U postgres -c "DROP DATABASE IF EXISTS $1" -c "CREATE DATABASE $1"
  bash scripts/apply-migrations.sh --migrations-dir migrations -- -q -h localhost -U postgres -d "$1"
  psql -q -h localhost -U postgres -d "$1" -v ON_ERROR_STOP=1 -f scripts/identity/provision-runtime-role.sql
}

provision "$db"
# The witness catalog test accepts only an explicit, disposable database named
# noesis_identity_test on 127.0.0.1:<port>: it leaves immutable synthetic
# history behind and temporarily re-owns witness_versions, so it must never
# share the general test database.
witness_db=noesis_identity_test
[ "$db" = "$witness_db" ] || provision "$witness_db"
port="$(psql -tA -h localhost -U postgres -c 'SHOW port')"
echo "ready: postgres://postgres@localhost/$db"
echo "runtime: TEST_SHARED_RUNTIME_URL=postgres://noesis_api_runtime@localhost/$db"
echo "witness: WITNESS_TEST_DATABASE_URL=postgres://postgres@127.0.0.1:$port/$witness_db"
