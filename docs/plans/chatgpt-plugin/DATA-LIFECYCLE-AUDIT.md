# Selemene data lifecycle evidence for the submission draft

Observed 2026-09-30. This is a bounded source and provider-metadata audit, not a
published privacy policy or a guarantee of production behavior. No database
contents, credentials, user records, request bodies or backup files were read.
No provider settings changed.

## Source scope

Reviewed the isolated plugin worktree's API, auth, data repositories, cache and
migrations; compared relevant files with the original engine checkout at
`/Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine`.
Main/auth/users/middleware and the readings/key schema files match byte-for-byte.
The API lib and admin repository differ between branches; the relevant routing,
asynchronous persistence, cache configuration and revoked-key purge behavior were
also confirmed directly in the original checkout. Current source remains separate
from the older production Rust deployment documented in `RAILWAY-AUDIT.md`.

## Facts available for drafting

| Data/control | Source evidence | What can actually be said |
| --- | --- | --- |
| Reading inputs and outputs | `migrations/005_readings.sql`; `crates/noesis-api/src/lib.rs` calculate and workflow handlers | The system attempts to store complete normalized input JSON, result JSON, witness text, engine/workflow IDs, user ID, phase and calculation/timestamp metadata. Persistence is asynchronous and best-effort; an HTTP success is not a storage receipt. |
| Birth details and profile | `crates/noesis-api/src/lib.rs` and `user_repository.rs::ensure_profile_from_birth_data` | Supplied birth name/date/time/location/timezone can populate the account profile on first use. Calculation can also create usage records and update XP/phase. |
| Profile controls | API router and `handlers/users.rs` | Authenticated `GET /api/v1/users/me` and `PATCH /api/v1/users/me` exist. Email is intentionally excluded from that update API. An update endpoint is not an account-deletion capability. |
| Reading access | API router, `list_readings_handler`, `get_reading_handler` | User-scoped paginated reading list/detail APIs exist. No general account-data export or self-service account/reading deletion endpoint was found in the scoped route/source search. A separate biofield-specific export API exists, outside this plugin's scope. |
| Persistent retention | Reading/profile/usage migrations and production source scans | No timed deletion policy or scheduled reading/profile/usage purge was found. Usage partitions support future operations but creation/partitioning does not implement deletion. Retention duration remains a business and implementation decision. |
| User deletion dependencies | `migrations/004_api_keys.sql`, `005_readings.sql` and later dependent schemas | Some rows cascade on user deletion, but `usage_logs.user_id` has a non-cascading foreign key in the inspected schema. Cascades do not prove a working account deletion operation or deletion from backups/logs. |
| API key storage | `migrations/004_api_keys.sql`; `noesis-auth/src/lib.rs` | The Rust database stores a SHA-256 key digest and key metadata, not the raw key in the inspected table. Postgres-backed validation checks active status and expiry on each lookup. |
| Key revocation/deletion | Admin routes, `handlers/admin.rs`, `admin_repository.rs` | Admin operations can revoke or delete API keys. These are not presently user-facing account deletion controls. |
| Revoked-key retention | `noesis-api/src/main.rs:75`; `admin_repository.rs::purge_revoked_api_keys`; migration 030 | Source schedules a daily purge of inactive keys older than 30 days by `revoked_at`, falling back to `created_at` for legacy rows. It requires an available admin repository. This is not a verified live-job receipt or a 30-day policy for readings. |
| Logout | `handlers/auth.rs::logout`; `noesis-auth/src/lib.rs::revoke_token` | JWT logout adds the current token identifier to an in-process revocation map until token expiry. This is not durable cross-instance revocation. API-key logout is explicitly a no-op; the key must be revoked/deleted separately. |
| Calculation cache | `build_app_state`, `noesis-cache/src/l1_cache.rs`, `l2_cache.rs` | Source configures a 100 MB size-bounded L1 cache, an L2 Redis TTL of one hour, and disabled L3 disk caching. L1 get does not enforce a one-hour TTL. These cache settings do not define database/log retention and do not prove every calculation uses a cache. |
| API request logging | `crates/noesis-api/src/middleware.rs::request_logging_middleware` | This middleware records method, URL path, trace ID, user ID when available, status and elapsed time. It does not explicitly log request/response bodies or auth headers. Other error/provider logs are not covered by that narrow statement. |
| Error/telemetry integrations | `noesis-api/src/main.rs`, `logging.rs` | Source supports Sentry when configured and tracing/OpenTelemetry paths. Live enablement, destination, redaction and provider retention were not established. Do not claim there are no other processors or that all logs exclude personal data. |

## Fresh Railway metadata: backup boundaries

Used existing CLI authentication with explicit project
`11eedde4-41e6-4f51-b86b-cf77111cf592` (robust-adventure), environment
`702b945e-2c66-4d5a-bae1-4c67ea14c3bb` (production), and Postgres service
`1ba4a991-ca9a-4831-981e-aa3470101ef1`.

Read-only `railway postgres pitr status --json` reported:

```json
{"isHaCluster":false,"enabled":false,"bucketWired":false}
```

The current environment's volume-instance metadata identified Postgres instance
`f4a888d8-e035-404c-a547-8357e30cd9d7` and Redis instance
`bb3d3813-e57d-47c8-881c-9d577b85ba0b`. GraphQL query-only reads returned:

```json
{
  "postgresSchedules": [],
  "redisSchedules": [],
  "postgresBackups": [{
    "createdAt": "2026-08-23T11:44:09.963Z",
    "expiresAt": "2026-09-22T11:44:09.644Z",
    "scheduleId": null
  }]
}
```

There is no configured backup schedule in those returned arrays. The single
listed manual Postgres backup has an expiry timestamp in the past; its presence
in metadata does not establish restorability or physical erasure. Do not turn
this one historical 30-day expiry into a retention promise. Provider internal
copies, other manual exports, restoration behavior and log retention remain
unknown. No backup was created, opened, restored, locked or deleted.

## Commitments still needed before policy publication

- Name the actual data controller/publisher, contact channel and request owner.
- Decide justified retention periods for profiles, readings, usage, security logs,
  revoked credentials and backup copies; implement and verify the promised policy.
- Define authenticated deletion and access/export handling, including shared
  readings, dependencies, backups, logs, exceptions and response timing. No SLA
  or one-click control is established by this audit.
- Verify actual Cloudflare OAuth/KV grant expiry, unlink/revocation behavior and
  raw-key handling once the MCP implementation is finalized. Disconnecting a
  connector must not be described as deleting existing readings or the account.
- Confirm the final processor/subprocessor inventory, regions, access controls
  and actual observability configuration. The intended Cloudflare/Railway boundary
  does not alone prove all runtime data destinations.
- Keep ChatGPT/OpenAI's own conversation/account data handling distinct from
  Selemene's storage; do not promise deletion from those systems through Selemene.

Safe interim wording: "Calculations can save supplied reading inputs and results
to your Selemene account and may update your profile and usage history. Account
linking and stored reading data are separate. Retention and deletion procedures
must be finalized before this draft is published as the service policy."

## Parent follow-up: telemetry configuration presence

A later read-only CLI query against the same explicit production API service returned `SENTRY_DSN` and `RUST_LOG` as configured. Only variable presence was printed; no values were saved or exposed. Sentry configuration is therefore present in the root service. Event delivery, payload/redaction behavior, account destination and retention remain unverified. Final processor disclosures must account for this configured error-monitoring integration rather than implying that Cloudflare and Railway are the entire processor inventory.
