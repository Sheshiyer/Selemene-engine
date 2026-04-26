# Issue #12 Deliverable: Admin RBAC Matrix (2026-02-25)

## 0) Deployment Topology Decision

| Component | Hosting | Contract |
|---|---|---|
| Admin dashboard frontend (`apps/admin-web`) | `Vercel` | Admin UI is deployed separately from backend services. |
| API and engine services (`/api/v1/*`) | `Railway` | Existing API + bridge engine deployment remains on Railway. |
| Frontend -> backend integration | HTTPS API base URL | Admin frontend points to Railway API origin via environment configuration. |

## 1) Role Definitions

| Role | Purpose | Allowed Scope | Explicit Constraints |
|---|---|---|---|
| `viewer` | Read-only operational visibility | Usage analytics, system operations visibility, audit trail read | No write actions, no API key lifecycle, no user mutations |
| `support` | Customer/user support operations | `viewer` scope + user list/detail/suspend + history sync diagnostics read | No API key lifecycle, no user role changes, no tier changes |
| `admin` | Operational administration | `support` scope + API key lifecycle + user tier changes + history sync retry | No user role changes |
| `platform-admin` | Platform governance and privileged access | Full admin portal scope in this document | Must be required for role assignment changes and any break-glass sensitive action |

## 2) Permission Naming Scheme

| Item | Standard |
|---|---|
| Pattern | `admin:<domain>:<action>` |
| Domain examples | `keys`, `users`, `history-sync`, `analytics`, `system`, `audit` |
| Action examples | `list`, `read`, `create`, `revoke`, `rotate`, `suspend`, `update`, `retry` |
| Rule | Use exact, non-hierarchical permission keys. Backend enforces each key explicitly. |

### Permission Catalog (Admin Portal Scope)

| Permission Key | Description |
|---|---|
| `admin:keys:list` | List API keys metadata (never secret material) |
| `admin:keys:create` | Create API keys |
| `admin:keys:revoke` | Revoke active API keys |
| `admin:keys:rotate` | Rotate API keys |
| `admin:users:list` | List users |
| `admin:users:read` | View user detail |
| `admin:users:suspend` | Suspend or unsuspend user |
| `admin:users:tier:update` | Change user plan/tier |
| `admin:users:roles:update` | Assign or remove user roles |
| `admin:history-sync:read` | View history sync diagnostics |
| `admin:history-sync:retry` | Retry failed history sync job |
| `admin:analytics:read` | View usage analytics |
| `admin:system:read` | View system operations status/visibility data |
| `admin:audit:list` | List audit events |
| `admin:audit:read` | View audit event detail |

### Legacy Compatibility (Current `noesis-auth` Aliases)

| Existing permission | Temporary mapping |
|---|---|
| `admin:users` | implies `admin:users:list`, `admin:users:read` |
| `admin:analytics` | implies `admin:analytics:read` |

## 3) Role -> Permission Set Mapping

| Role | Granted Permissions |
|---|---|
| `viewer` | `admin:analytics:read`, `admin:system:read`, `admin:audit:list`, `admin:audit:read` |
| `support` | `viewer` + `admin:users:list`, `admin:users:read`, `admin:users:suspend`, `admin:history-sync:read` |
| `admin` | `support` + `admin:keys:list`, `admin:keys:create`, `admin:keys:revoke`, `admin:keys:rotate`, `admin:users:tier:update`, `admin:history-sync:retry` |
| `platform-admin` | `admin` + `admin:users:roles:update` |

## 4) Operations Matrix (Roles, Permissions, Endpoints, UI Pages)

| Domain | Operation | Required Permission | Allowed Roles | Backend Endpoint(s) | Frontend Page(s) |
|---|---|---|---|---|---|
| API keys lifecycle | List keys | `admin:keys:list` | `admin`, `platform-admin` | `GET /api/v1/admin/api-keys` | `/admin/api-keys` |
| API keys lifecycle | Create key | `admin:keys:create` | `admin`, `platform-admin` | `POST /api/v1/admin/api-keys` | `/admin/api-keys` |
| API keys lifecycle | Revoke key | `admin:keys:revoke` | `admin`, `platform-admin` | `POST /api/v1/admin/api-keys/{key_id}/revoke` | `/admin/api-keys` |
| API keys lifecycle | Rotate key | `admin:keys:rotate` | `admin`, `platform-admin` | `POST /api/v1/admin/api-keys/{key_id}/rotate` | `/admin/api-keys` |
| User management | List users | `admin:users:list` | `support`, `admin`, `platform-admin` | `GET /api/v1/admin/users` | `/admin/users` |
| User management | User detail | `admin:users:read` | `support`, `admin`, `platform-admin` | `GET /api/v1/admin/users/{user_id}` | `/admin/users/{user_id}` |
| User management | Suspend/unsuspend | `admin:users:suspend` | `support`, `admin`, `platform-admin` | `PATCH /api/v1/admin/users/{user_id}/state` | `/admin/users/{user_id}` |
| User management | Update tier | `admin:users:tier:update` | `admin`, `platform-admin` | `PATCH /api/v1/admin/users/{user_id}/tier` | `/admin/users/{user_id}` |
| User management | Update roles | `admin:users:roles:update` | `platform-admin` | `PUT /api/v1/admin/users/{user_id}/roles` | `/admin/users/{user_id}` |
| History sync diagnostics | View diagnostics | `admin:history-sync:read` | `support`, `admin`, `platform-admin` | `GET /api/v1/admin/history-sync/users` | `/admin/history-sync` |
| History sync diagnostics | Retry failed job | `admin:history-sync:retry` | `admin`, `platform-admin` | `POST /api/v1/admin/history-sync/jobs/{job_id}/retry` | `/admin/history-sync` |
| Usage analytics | View analytics dashboards | `admin:analytics:read` | `viewer`, `support`, `admin`, `platform-admin` | `GET /api/v1/admin/analytics/summary` | `/admin/analytics` |
| System operations visibility | View runtime/system state | `admin:system:read` | `viewer`, `support`, `admin`, `platform-admin` | `GET /api/v1/admin/system/health` | `/admin/system` |
| Audit trail | List audit events | `admin:audit:list` | `viewer`, `support`, `admin`, `platform-admin` | `GET /api/v1/admin/audit-events` | `/admin/audit` |
| Audit trail | Audit event detail | `admin:audit:read` | `viewer`, `support`, `admin`, `platform-admin` | `GET /api/v1/admin/audit-events/{event_id}` | `/admin/audit` |

## 5) Route Guard Recommendations (Backend + Frontend)

### Backend Guard Recommendations

| Route Pattern | Guard Requirement | Recommendation |
|---|---|---|
| `/api/v1/admin/**` | Authenticated session + explicit permission check | Apply `requirePermission('<key>')` per endpoint. Default deny on missing permission. |
| Mutating admin routes (`POST`, `PATCH`) | Permission + CSRF + audit emit | For every successful and denied mutation, emit audit event with actor, target, diff, and request id. |
| Sensitive mutations (`keys:rotate`, `keys:revoke`, `users:roles:update`) | Permission + step-up auth | Require recent MFA re-auth (for example, <=15 min freshness). |
| Role assignment route | `admin:users:roles:update` only | Enforce `platform-admin` level by permission; do not rely on role string checks in handlers. |

### Frontend Guard Recommendations

| UI Surface | Required Permission(s) | Recommendation |
|---|---|---|
| Admin route entry (`/admin/*`) | Any `admin:*` permission | Block route and show 403 page if none granted. |
| Page-level access | Page minimum permission from matrix | Wrap each page with `RouteGuard(requiredPerms)` and short-circuit data fetches when unauthorized. |
| Action controls (buttons/forms) | Action permission from matrix | Hide or disable controls with deterministic "Missing permission" reason text. |
| Permission source | Server-issued permission list | Fetch once via `/api/v1/admin/session`; treat backend as source of truth. |

## 6) Explicit Deny List (Sensitive Actions)

| Sensitive Action | Denied Roles | Enforcement Rule |
|---|---|---|
| Update user roles (`admin:users:roles:update`) | `viewer`, `support`, `admin` | Allow only `platform-admin` via permission check. |
| Rotate API keys (`admin:keys:rotate`) | `viewer`, `support` | Deny at backend even if UI payload is forged. |
| Revoke API keys (`admin:keys:revoke`) | `viewer`, `support` | Deny at backend even if UI payload is forged. |
| Create API keys (`admin:keys:create`) | `viewer`, `support` | Deny at backend; UI must not render create form. |
| View raw API key secret after creation | `viewer`, `support`, `admin`, `platform-admin` | Never return stored secret; show only one-time creation token. |
| Delete or mutate audit events | `viewer`, `support`, `admin`, `platform-admin` | No endpoint exposed; hard deny globally. |
| Execute system control actions (restart/backfill/kill) via admin portal | `viewer`, `support`, `admin` | Keep out of portal scope; if added later, gate to dedicated break-glass permission only. |

## Implementation Notes

| Rule | Requirement |
|---|---|
| Authorization model | Backend permission checks are authoritative; frontend guards are UX only. |
| Policy default | Deny by default for every endpoint not mapped in this document. |
| Auditing | Log both allow and deny decisions for all mutating admin endpoints. |
| Backward compatibility | Add permissions additively; never infer permissions from legacy role names in handlers. |
