# Admin Portal Information Architecture and Route Contracts

- Issue: `#13` (admin IA + route contracts)
- Date: `2026-02-25`
- Scope: `apps/admin-web` frontend routes and required `/api/v1/admin/*` backend contracts
- Source alignment: `docs/planning/admin-panel-taskmaster-plan-2026-02-25.json` (`P0-S1-04`)

## 0) Deployment Topology (Decision)

| Surface | Platform | Contract |
| --- | --- | --- |
| Admin dashboard frontend (`apps/admin-web`) | `Vercel` | Deploy admin UI independently from backend services. |
| API and engine services (`/api/v1/*`) | `Railway` | Keep existing API and engine runtime on Railway. |
| Admin web -> API connection | HTTPS base URL (`NEXT_PUBLIC_API_BASE_URL`) | Restrict CORS/session origins to approved Vercel domains. |

## 1) Canonical Route Tree

```text
/admin
  /login                               (public)
  /                                    (protected; redirects to /admin/dashboard)
  /dashboard                           (protected)
  /users                               (protected)
  /api-keys                            (protected)
  /history-sync                        (protected)
  /analytics                           (protected)
  /system                              (protected)
  /audit                               (protected)
```

Route rules:

| Rule | Contract |
| --- | --- |
| Auth boundary | `/admin/login` is the only public route. All other `/admin/*` routes require a valid admin session. |
| Default landing | `/admin` must redirect to `/admin/dashboard` after session validation. |
| Unauthorized behavior | If unauthenticated: redirect to `/admin/login?redirect=<encoded_target>`. If authenticated but unauthorized: render `403` state (no redirect loop). |
| Detail views | Detail panels are drawer-based via query params (not separate routes) for users, keys, and audit events. |

## 2) Permission Namespace (Required by Route Guards)

| Permission | Purpose | Required on pages |
| --- | --- | --- |
| `admin:keys:list` | Read key inventory and metadata | api-keys |
| `admin:keys:create` | Create API keys | api-keys |
| `admin:keys:revoke` | Revoke active keys | api-keys |
| `admin:keys:rotate` | Rotate keys | api-keys |
| `admin:users:list` | Read user list | users |
| `admin:users:read` | Read user detail | users, history-sync (user context) |
| `admin:users:suspend` | Suspend/reactivate users | users |
| `admin:users:tier:update` | Update user tier | users |
| `admin:users:roles:update` | Assign/remove admin roles | users |
| `admin:history-sync:read` | Read sync cursor/device/ingestion diagnostics | history-sync |
| `admin:history-sync:retry` | Retry failed sync jobs | history-sync (Phase-2 action path) |
| `admin:analytics:read` | Read usage analytics and aggregates | dashboard, analytics |
| `admin:system:read` | Read service health and operational status | system, dashboard health cards |
| `admin:audit:list` | Read audit event list | dashboard (recent), audit |
| `admin:audit:read` | Read audit event detail | audit |

Legacy compatibility (already present in codebase):

| Existing permission | Temporary mapping |
| --- | --- |
| `admin:users` | implies `admin:users:list`, `admin:users:read` |
| `admin:analytics` | implies `admin:analytics:read` |

## 3) Navigation Structure and Breadcrumbs

### 3.1 Sidebar Navigation

| Group | Item label | Route | Visibility gate |
| --- | --- | --- | --- |
| Overview | Dashboard | `/admin/dashboard` | `admin:analytics:read` |
| Operations | Users | `/admin/users` | `admin:users:list` |
| Operations | API Keys | `/admin/api-keys` | `admin:keys:list` |
| Operations | History Sync | `/admin/history-sync` | `admin:history-sync:read` |
| Insights | Usage Analytics | `/admin/analytics` | `admin:analytics:read` |
| Insights | Audit Trail | `/admin/audit` | `admin:audit:list` |
| Platform | System Operations | `/admin/system` | `admin:system:read` |

### 3.2 Breadcrumb Contracts

| Route state | Breadcrumb |
| --- | --- |
| `/admin/dashboard` | `Admin / Dashboard` |
| `/admin/users` | `Admin / Users` |
| `/admin/users?drawer=user:<id>` | `Admin / Users / <id-or-email>` |
| `/admin/api-keys` | `Admin / API Keys` |
| `/admin/api-keys?drawer=key:<id>` | `Admin / API Keys / <key-label-or-id>` |
| `/admin/history-sync` | `Admin / History Sync` |
| `/admin/analytics` | `Admin / Usage Analytics` |
| `/admin/system` | `Admin / System Operations` |
| `/admin/audit` | `Admin / Audit Trail` |
| `/admin/audit?drawer=event:<id>` | `Admin / Audit Trail / Event <id>` |

## 4) Page-Level Contract Matrix (MVP)

| Page route | Purpose | Required backend endpoints | Required permissions | Primary components | Loading / empty / error states |
| --- | --- | --- | --- | --- | --- |
| `/admin/login` | Admin authentication entry point and redirect recovery. | `POST /api/v1/auth/login`; `GET /api/v1/admin/session` (post-login validation). | none (public) | login form, password field, submit CTA, inline validation. | Loading: submit button spinner and form lock.<br>Empty: n/a.<br>Error: invalid credentials inline, network error toast, lockout message when applicable. |
| `/admin/dashboard` | At-a-glance health, usage, and recent admin activity. | `GET /api/v1/admin/dashboard/summary`; `GET /api/v1/admin/system/health`; `GET /api/v1/admin/audit-events?limit=10`. | `admin:analytics:read`, `admin:system:read`, `admin:audit:list` | KPI cards, health status list, mini charts, recent-activity table. | Loading: page skeleton + card/chart placeholders.<br>Empty: render zero-value cards and "No recent activity".<br>Error: partial widget-level error cards; page must still render available widgets. |
| `/admin/users` | User search + lifecycle management (status, tier, roles). | `GET /api/v1/admin/users`; `GET /api/v1/admin/users/{user_id}`; `PATCH /api/v1/admin/users/{user_id}/state`; `PATCH /api/v1/admin/users/{user_id}/tier`; `PUT /api/v1/admin/users/{user_id}/roles`. | `admin:users:list`, `admin:users:read` (+ `admin:users:suspend` for state changes, `admin:users:tier:update` for tier changes, `admin:users:roles:update` for role changes) | filter bar, table, pagination, detail drawer, edit form, confirmation modal. | Loading: table skeleton + drawer skeleton.<br>Empty: "No users match current filters" + clear-filter action.<br>Error: table-level retry, drawer-level retry, mutation failure toast with request id. |
| `/admin/api-keys` | Full key lifecycle operations with one-time reveal handling. | `GET /api/v1/admin/api-keys`; `GET /api/v1/admin/api-keys/{key_id}`; `POST /api/v1/admin/api-keys`; `POST /api/v1/admin/api-keys/{key_id}/revoke`; `POST /api/v1/admin/api-keys/{key_id}/rotate`. | `admin:keys:list` (+ `admin:keys:create`, `admin:keys:revoke`, `admin:keys:rotate` for mutations) | filter bar, table, create-key modal form, detail drawer, reveal-once panel, confirmation modal. | Loading: table/form skeletons and action-level spinners.<br>Empty: "No keys found" with create-key CTA.<br>Error: failed reveal/revoke/rotate shown inline + toast; preserve current table state. |
| `/admin/history-sync` | Diagnose user history sync state (cursor drift, device state, ingestion failures). | `GET /api/v1/admin/history-sync/users`; `GET /api/v1/admin/history-sync/users/{user_id}`; `GET /api/v1/admin/history-sync/devices`; `GET /api/v1/admin/history-sync/events`. | `admin:history-sync:read`, `admin:users:read` | filter bar, diagnostics table, status badges, user detail drawer, event log table. | Loading: split-panel skeletons.<br>Empty: "No out-of-sync users/devices".<br>Error: endpoint-specific error panels with retry; preserve filter params. |
| `/admin/analytics` | Inspect usage trends by time, engine, tier, and key/user segments. | `GET /api/v1/admin/analytics/summary`; `GET /api/v1/admin/analytics/usage-timeseries`; `GET /api/v1/admin/analytics/usage-breakdown`; `GET /api/v1/admin/analytics/top-consumers`. | `admin:analytics:read` | global date filter, KPI cards, line/bar charts, breakdown table, segment tabs. | Loading: chart shimmer + KPI skeleton.<br>Empty: "No usage for selected window".<br>Error: chart/table level errors with retry; keep selected date window in URL. |
| `/admin/system` | Operational visibility for API, bridges, workflows, and cache health. | `GET /api/v1/admin/system/health`; `GET /api/v1/admin/system/services`; `GET /api/v1/admin/system/workflows`; `GET /api/v1/admin/system/cache`. | `admin:system:read` | service status table, uptime cards, incident list, dependency detail drawer. | Loading: card/table skeletons.<br>Empty: "No incidents in selected window".<br>Error: per-widget error state; never blank the whole page when one source fails. |
| `/admin/audit` | Forensic audit event search with actor/action/target filters. | `GET /api/v1/admin/audit-events`; `GET /api/v1/admin/audit-events/{event_id}`; `GET /api/v1/admin/audit-events/actions`. | `admin:audit:list`, `admin:audit:read` | advanced filter bar, event table, detail drawer (JSON payload). | Loading: table skeleton + drawer spinner.<br>Empty: "No events match filters".<br>Error: query error banner with retry and preserved filters. |

## 5) Backend Endpoint Contract (MVP Required)

The endpoints below are the minimum backend surface needed for frontend completion of the routes above.

| Method + path | Required request/query contract | Minimum response fields | Used by pages |
| --- | --- | --- | --- |
| `GET /api/v1/admin/session` | bearer token | `user_id`, `email`, `roles[]`, `permissions[]`, `expires_at` | all protected routes |
| `GET /api/v1/admin/dashboard/summary` | `from`, `to` optional | `kpis{active_users,active_keys,requests_24h,error_rate}`, `updated_at` | dashboard |
| `GET /api/v1/admin/users` | `q`, `status`, `tier`, `role`, `page`, `page_size`, `sort_by`, `sort_order` | `items[]`, `page`, `page_size`, `total_items` | users |
| `GET /api/v1/admin/users/{user_id}` | path `user_id` | `user`, `roles[]`, `account_state`, `latest_activity_at` | users, history-sync |
| `PATCH /api/v1/admin/users/{user_id}/state` | body `{state, reason}` | updated `account_state`, `updated_at` | users |
| `PATCH /api/v1/admin/users/{user_id}/tier` | body `{tier}` | updated `tier`, `updated_at` | users |
| `PUT /api/v1/admin/users/{user_id}/roles` | body `{roles[]}` | updated `roles[]`, `updated_at` | users |
| `GET /api/v1/admin/api-keys` | `q`, `owner_user_id`, `is_active`, `tier`, `page`, `page_size`, `sort_by`, `sort_order` | `items[]`, `page`, `page_size`, `total_items` | api-keys |
| `GET /api/v1/admin/api-keys/{key_id}` | path `key_id` | key metadata, recent usage summary, last events | api-keys |
| `POST /api/v1/admin/api-keys` | body `{user_id,tier,permissions[],expires_at}` | created key metadata + `plaintext_key` (returned once) | api-keys |
| `POST /api/v1/admin/api-keys/{key_id}/revoke` | body `{reason}` | key state `revoked`, `revoked_at` | api-keys |
| `POST /api/v1/admin/api-keys/{key_id}/rotate` | body `{reason,carry_permissions=true}` | new key metadata + one-time `plaintext_key`, replaced key id | api-keys |
| `GET /api/v1/admin/history-sync/users` | `q`, `sync_status`, `stale_minutes_gte`, `page`, `page_size`, `sort_by`, `sort_order` | `items[]`, `total_items`, status counters | history-sync |
| `GET /api/v1/admin/history-sync/users/{user_id}` | path `user_id` | user sync cursor timeline, lag metrics | history-sync |
| `GET /api/v1/admin/history-sync/devices` | `q`, `platform`, `sync_status`, `page`, `page_size`, `sort_by`, `sort_order` | `items[]`, `total_items` | history-sync |
| `GET /api/v1/admin/history-sync/events` | `user_id`, `event_type`, `result`, `page`, `page_size`, `sort_by`, `sort_order` | `items[]`, `total_items` | history-sync |
| `GET /api/v1/admin/analytics/summary` | `from`, `to` | KPI summary object | analytics, dashboard |
| `GET /api/v1/admin/analytics/usage-timeseries` | `from`, `to`, `interval`, `tier`, `engine_id` | `series[]` with timestamp + value | analytics |
| `GET /api/v1/admin/analytics/usage-breakdown` | `from`, `to`, `group_by`, `tier` | grouped totals list | analytics |
| `GET /api/v1/admin/analytics/top-consumers` | `from`, `to`, `dimension`, `limit`, `sort_by`, `sort_order` | ranked list with totals | analytics |
| `GET /api/v1/admin/system/health` | optional `scope` | global status + per-subsystem status | system, dashboard |
| `GET /api/v1/admin/system/services` | `status`, `page`, `page_size`, `sort_by`, `sort_order` | services list with health and latency | system |
| `GET /api/v1/admin/system/workflows` | `status`, `page`, `page_size`, `sort_by`, `sort_order` | workflow run snapshots | system |
| `GET /api/v1/admin/system/cache` | optional `namespace` | cache hit rate + capacity + stale entries | system |
| `GET /api/v1/admin/audit-events` | `actor`, `action`, `target_type`, `target_id`, `request_id`, `result`, `from`, `to`, `page`, `page_size`, `sort_by`, `sort_order` | `items[]`, `total_items` | audit, dashboard |
| `GET /api/v1/admin/audit-events/{event_id}` | path `event_id` | full immutable event payload + metadata | audit |
| `GET /api/v1/admin/audit-events/actions` | none | allowed action names for filter autocomplete | audit |

## 6) URL Query Contract (Frontend Route State)

All filter/sort/pagination state for list and analytics pages must be URL-driven.

### 6.1 Shared Query Parameters

| Param | Type | Rules |
| --- | --- | --- |
| `q` | string | free-text search; trim whitespace; max 120 chars. |
| `page` | integer | `>=1`; default `1`. |
| `page_size` | integer | allowed: `10`, `25`, `50`, `100`; default `25`. |
| `sort` | string | format `<field>:<asc|desc>`; single sort in MVP. |
| `from` | ISO datetime | UTC timestamp (inclusive lower bound). |
| `to` | ISO datetime | UTC timestamp (exclusive upper bound). |
| `drawer` | string | UI-only detail drawer state. Allowed formats: `user:<id>`, `key:<id>`, `event:<id>`. |

Normalization rules:

| Rule | Behavior |
| --- | --- |
| Invalid `page`/`page_size` | Replace with defaults and update URL silently. |
| Filter change | Reset `page=1` and remove `drawer`. |
| Sort change | Reset `page=1`. |
| Unknown query keys | Preserve in URL but do not forward to API calls. |
| API mapping | `page/page_size/sort` map to `page,page_size,sort_by,sort_order` API params. |

### 6.2 Per-Route Query Keys

| Route | Allowed filter keys | Allowed sort fields | Pagination |
| --- | --- | --- | --- |
| `/admin/users` | `q`, `status`, `tier`, `role`, `created_from`, `created_to`, `drawer` | `created_at`, `last_login_at`, `email`, `tier`, `status` | `page`, `page_size` |
| `/admin/api-keys` | `q`, `owner_user_id`, `is_active`, `tier`, `expires_before`, `created_from`, `created_to`, `drawer` | `created_at`, `last_used_at`, `expires_at`, `tier`, `is_active` | `page`, `page_size` |
| `/admin/history-sync` | `q`, `sync_status`, `platform`, `event_type`, `result`, `user_id`, `drawer` | `lag_minutes`, `updated_at`, `last_event_at`, `platform`, `sync_status` | `page`, `page_size` |
| `/admin/analytics` | `from`, `to`, `interval`, `group_by`, `tier`, `engine_id`, `dimension` | `total_requests`, `error_rate`, `p95_ms` (for tabular segments) | table sections only use `page`, `page_size` |
| `/admin/system` | `status`, `service_type`, `region` | `status`, `latency_ms`, `updated_at`, `error_rate` | `page`, `page_size` for services/workflows tables |
| `/admin/audit` | `actor`, `action`, `target_type`, `target_id`, `request_id`, `result`, `from`, `to`, `drawer` | `created_at`, `actor`, `action`, `result` | `page`, `page_size` |

## 7) UI State Contract (Cross-Page)

| Component type | Loading state | Empty state | Error state |
| --- | --- | --- | --- |
| Table | skeleton rows matching current page size | explicit empty-copy + clear-filters action | inline alert above table + retry button |
| Filter bar | controls disabled during initial load only | keep controls interactive | keep last values; show non-blocking error chip |
| Form/modal | submit spinner + disabled controls | n/a | field-level errors plus top-level request error |
| Detail drawer | placeholder skeleton sections | "No detail available" when entity removed | inline drawer error with close + retry |
| Charts | chart skeleton | "No data in selected window" | chart fallback card with retry |

## 8) MVP vs Phase-2 Scope Notes

| Area | MVP (required now) | Phase-2 (defer) |
| --- | --- | --- |
| Auth/session | Login + session validation + protected routes | refresh token endpoint, SSO/SAML/OIDC, step-up auth |
| Dashboard | KPI cards + health snapshot + recent audit events | custom widget layout, saved dashboard views |
| Users | search/detail, suspend/reactivate, tier update, role assignment | bulk actions, impersonation mode, user timeline diff |
| API Keys | list/create/revoke/rotate, one-time reveal UX | scoped key templates, scheduled rotation, key usage anomaly alerts |
| History Sync | read-only diagnostics for users/devices/events | repair actions (`resync`, `replay`, `cursor reset`) gated by `admin:history-sync:retry` |
| Analytics | date-windowed usage charts and top consumer tables | cohort retention, anomaly detection, scheduled exports |
| System | health/services/workflows/cache visibility | admin-triggered maintenance actions, runbook automation |
| Audit | immutable event list/detail with rich filters | signed exports, alert subscriptions, SIEM push integration |

## 9) Implementation Checklist for Teams

| Team | Action |
| --- | --- |
| Frontend | Implement exact route tree and query contracts in router/state layer; no hidden local filter state. |
| Frontend | Gate nav visibility by permission and enforce route-level permission checks before data fetch. |
| Backend | Implement all MVP endpoints in Section 5 under `/api/v1/admin/*` with explicit permission checks and audit emission for mutations. |
| Backend | Return deterministic pagination envelopes (`items`, `page`, `page_size`, `total_items`) for all list endpoints. |
| Backend | Include `request_id` in error payloads to support operator troubleshooting UI. |
| Security | Validate legacy permission alias mapping until granular permissions are fully rolled out. |
