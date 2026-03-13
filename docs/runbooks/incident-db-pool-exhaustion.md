# Runbook: Database Connection Pool Exhaustion

This runbook covers PostgreSQL pool exhaustion or database unavailability that impacts auth and DB-backed admin surfaces.

## Current Degraded-Mode Pattern

The current runtime intentionally tolerates missing or failed database connections:

- API boot path in [crates/noesis-api/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-api/src/lib.rs) creates an optional database pool
- auth service in [crates/noesis-auth/src/lib.rs](/Volumes/madara/2026/witnessos/Selemene-engine/crates/noesis-auth/src/lib.rs) uses `Option<PgPool>`
- when the pool is `None`, API key validation falls back to in-memory and DB-backed endpoints degrade

This is the runtime behavior the issue refers to as degraded mode.

## Symptoms

- auth endpoint failures
- admin/repository-backed routes fail
- boot logs report:
  - `Database connection failed (running without DB): ...`
  - `Database connection timed out after 5s (running without DB)`
  - `No DATABASE_URL configured — auth endpoints unavailable`
- high connection count or blocked sessions in Postgres

Treat this as the operational equivalent of a `NoesisPostgresConnectionsHigh` alert.

## Diagnosis

1. Check whether the API is running without DB:
   - inspect logs for the messages above
2. Confirm database connectivity:
   - `psql "$DATABASE_URL" -c 'select 1'`
3. Inspect active connections:
```sql
select pid, usename, application_name, state, wait_event_type, wait_event,
       now() - query_start as query_age, now() - state_change as state_age,
       query
from pg_stat_activity
where datname = current_database()
order by query_start asc;
```
4. Check connection counts by state:
```sql
select state, count(*)
from pg_stat_activity
where datname = current_database()
group by state
order by count(*) desc;
```
5. Check for long-idle transactions:
```sql
select pid, usename, application_name, state,
       now() - xact_start as xact_age, query
from pg_stat_activity
where xact_start is not null
order by xact_start asc;
```

## Mitigation

1. Preserve degraded mode if the API is otherwise serving.
   - non-DB routes can continue serving
   - auth / DB-backed routes may be unavailable
2. Kill obviously stuck idle sessions only if it is operationally safe:
```sql
select pg_terminate_backend(<pid>);
```
3. Verify pool settings in the boot path:
   - `max_connections(5)`
   - `acquire_timeout(5s)`
   - `idle_timeout(300s)`
4. If the database is healthy but pool acquisition still fails:
   - restart the API to clear broken client state
5. If the DB itself is unhealthy:
   - escalate to the platform / DB owner

## Recovery Verification

1. `psql "$DATABASE_URL" -c 'select 1'` succeeds
2. connection counts return to normal
3. auth routes recover
4. DB-backed admin surfaces recover
5. logs stop reporting DB timeout / unavailable messages

## Escalation Contacts

- database / platform owner
- auth owner if token/API-key flows remain degraded after DB recovery
- release owner if degradation overlaps a deployment

