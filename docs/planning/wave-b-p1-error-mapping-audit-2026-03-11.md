# Wave B P1 Error Mapping Audit

Date: 2026-03-11  
Scope: `#45` through `#57` in `P1-Stabilization`

## Implementation Update

Later on `2026-03-11`, the follow-up implementation for `#47` and `#50` landed.

- `#47` is now closeable:
  - `ErrorResponse` moved into [`crates/noesis-api/src/error_mapper.rs`](../../crates/noesis-api/src/error_mapper.rs)
  - schema now includes `status`, `error_code`, `message`, legacy `error`, `details`, and `trace_id`
  - route-level error shape assertions were expanded in:
    - [`crates/noesis-api/tests/error_handling_tests.rs`](../../crates/noesis-api/tests/error_handling_tests.rs)
    - [`crates/noesis-api/tests/rate_limit_tests.rs`](../../crates/noesis-api/tests/rate_limit_tests.rs)
- `#50` is now closeable:
  - dedicated [`crates/noesis-api/src/error_mapper.rs`](../../crates/noesis-api/src/error_mapper.rs) module exists
  - `engine_error_to_response()` was removed from [`crates/noesis-api/src/lib.rs`](../../crates/noesis-api/src/lib.rs)
  - `ApiError`, inline handler call sites, auth middleware, rate-limit middleware, and admin helpers now route through `ErrorMapper`
- Issues that remain open after this implementation pass:
  - `#48`, `#51`, `#52`, `#53`, `#54`, `#55`, `#56`, `#57`

## Summary

This audit reviewed the current `noesis-api` error response path, the engine-level error variants used across native Rust engines and TS bridge engines, and the handler-level response patterns in `noesis-api`.

Current closeable issues:

- `#45` Catalog all `EngineError` variants and their current HTTP mappings
- `#46` Identify inconsistent error handling across engine crates
- `#49` Map error handling patterns across all handler functions in `noesis-api`

Current non-closeable issues:

- `#47` `ErrorResponse` is unified but does not yet include `status` or `trace_id`
- `#48` current bridge propagation is partially documented here, but the issue assumes a dedicated `BridgeError` variant taxonomy that does not exist in the current codebase
- `#50` through `#57` remain open implementation work

## Issue Status Matrix

| Issue | Status | Reason |
| --- | --- | --- |
| `#45` | closable | Current taxonomy and HTTP mapping are fully documented below |
| `#46` | closable | Current engine inconsistency audit is documented below across native and TS engines |
| `#47` | open | [`ErrorResponse`](../../crates/noesis-api/src/lib.rs) only has `error`, `error_code`, `details`; missing `status` and `trace_id` |
| `#48` | open / re-scope | Current bridge layer maps directly to `EngineError::{ValidationError,BridgeError}` without a distinct bridge-error enum |
| `#49` | closable | Handler-level audit is documented below, including `ApiError` usage vs ad-hoc responses |
| `#50` | open | No dedicated `error_mapper.rs`; mapping still lives in `engine_error_to_response()` |
| `#51` | open | 5xx Sentry capture exists inline, but no breadcrumb split and no request trace correlation |
| `#52` | open | Exhaustive `match` exists implicitly, but there is no dedicated exhaustiveness test or `ErrorMapper` module |
| `#53` | open | Bridge error translation is still string-based, not structured |
| `#54` | open | Error JSON does not include `trace_id` |
| `#55` | open | No snapshot suite for error response shapes |
| `#56` | open | No per-`error_code` Prometheus counter at the API response layer |
| `#57` | open | Workflow partial failures do not return `207 Multi-Status` |

## `#45` EngineError Taxonomy and HTTP Mapping

Current source of truth:

- [`crates/noesis-core/src/error.rs:4`](../../crates/noesis-core/src/error.rs#L4)
- [`crates/noesis-api/src/lib.rs:1392`](../../crates/noesis-api/src/lib.rs#L1392)

The issue body says “11 EngineError variants,” but the current code has `12`.

| EngineError variant | HTTP status | `error_code` | user-facing `error` field | `details` shape |
| --- | --- | --- | --- | --- |
| `EngineNotFound(String)` | `404` | `ENGINE_NOT_FOUND` | passthrough `err.to_string()` | `{ "engine_id": id }` |
| `WorkflowNotFound(String)` | `404` | `WORKFLOW_NOT_FOUND` | passthrough `err.to_string()` | `{ "workflow_id": id }` |
| `PhaseAccessDenied { required, current }` | `403` | `PHASE_ACCESS_DENIED` | passthrough `err.to_string()` | `{ "required_phase": required, "current_phase": current }` |
| `AuthError(String)` | `401` | `AUTH_ERROR` | passthrough `err.to_string()` | `{ "reason": msg }` |
| `RateLimitExceeded` | `429` | `RATE_LIMIT_EXCEEDED` | passthrough `err.to_string()` | `null` |
| `ValidationError(String)` | `422` | `VALIDATION_ERROR` | passthrough `err.to_string()` | `{ "validation_message": msg }` |
| `CalculationError(String)` | `500` | `CALCULATION_ERROR` | `"An internal calculation error occurred"` | `null` |
| `CacheError(String)` | `500` | `CACHE_ERROR` | `"An internal cache error occurred"` | `null` |
| `ConfigError(String)` | `500` | `CONFIG_ERROR` | `"An internal configuration error occurred"` | `null` |
| `BridgeError(String)` | `500` | `BRIDGE_ERROR` | `"An internal bridge error occurred"` | `null` |
| `SwissEphemerisError(String)` | `500` | `SWISS_EPHEMERIS_ERROR` | `"An internal ephemeris error occurred"` | `null` |
| `InternalError(String)` | `500` | `INTERNAL_ERROR` | `"An internal error occurred"` | `null` |

Notes:

- Current `ErrorResponse` schema lives at [`crates/noesis-api/src/lib.rs:519`](../../crates/noesis-api/src/lib.rs#L519).
- Current Sentry capture for 5xx mappings is inline at [`crates/noesis-api/src/lib.rs:1475`](../../crates/noesis-api/src/lib.rs#L1475).

## `#46` Engine Crate Error Pattern Audit

### Native Rust engines

| Engine | Primary error patterns | Source refs | Consistency note |
| --- | --- | --- | --- |
| `panchanga` | `CalculationError` for compute/serialize, `ValidationError` for deserialize | [`crates/engine-panchanga/src/lib.rs:478`](../../crates/engine-panchanga/src/lib.rs#L478), [`crates/engine-panchanga/src/lib.rs:518`](../../crates/engine-panchanga/src/lib.rs#L518) | Reasonable split, but input parse failures are not strongly distinguished from compute failures |
| `vimshottari` | invalid date/time and downstream failures all use `CalculationError` | [`crates/engine-vimshottari/src/engine.rs:84`](../../crates/engine-vimshottari/src/engine.rs#L84) | Inconsistent with `human-design` and `transits`, which treat bad user input as `ValidationError` |
| `human-design` | bad birth input/timezone uses `ValidationError`; chart generation uses `CalculationError` | [`crates/engine-human-design/src/engine.rs:36`](../../crates/engine-human-design/src/engine.rs#L36), [`crates/engine-human-design/src/engine.rs:162`](../../crates/engine-human-design/src/engine.rs#L162) | Strongest native pattern separation |
| `gene-keys` | malformed options use `ValidationError`; upstream HD/output failures use `CalculationError` | [`crates/engine-gene-keys/src/engine.rs:56`](../../crates/engine-gene-keys/src/engine.rs#L56), [`crates/engine-gene-keys/src/engine.rs:207`](../../crates/engine-gene-keys/src/engine.rs#L207) | Mostly good; dependent upstream data failures are treated as compute failures |
| `transits` | invalid date/time/timezone and missing birth data use `ValidationError`; transit compute failures use `CalculationError` | [`crates/engine-transits/src/engine.rs:45`](../../crates/engine-transits/src/engine.rs#L45), [`crates/engine-transits/src/engine.rs:209`](../../crates/engine-transits/src/engine.rs#L209) | Strong pattern, aligned with `human-design` |
| `numerology` | missing `birth_data`/`name` uses `ValidationError`; invalid date components still use `CalculationError`; serialization uses `InternalError` | [`crates/engine-numerology/src/lib.rs:186`](../../crates/engine-numerology/src/lib.rs#L186), [`crates/engine-numerology/src/lib.rs:291`](../../crates/engine-numerology/src/lib.rs#L291), [`crates/engine-numerology/src/lib.rs:351`](../../crates/engine-numerology/src/lib.rs#L351) | Mixed classification for user-supplied date errors |
| `biorhythm` | invalid date strings often use `CalculationError`; some downstream parsing uses `ValidationError` | [`crates/engine-biorhythm/src/lib.rs:313`](../../crates/engine-biorhythm/src/lib.rs#L313), [`crates/engine-biorhythm/src/lib.rs:433`](../../crates/engine-biorhythm/src/lib.rs#L433) | Similar inconsistency to `vimshottari`/`numerology` |
| `biofield` | top-level engine failures use `CalculationError`; Vedic helper parsing uses `ValidationError` | [`crates/engine-biofield/src/engine.rs:231`](../../crates/engine-biofield/src/engine.rs#L231), [`crates/engine-biofield/src/vedic/mod.rs:142`](../../crates/engine-biofield/src/vedic/mod.rs#L142) | Helper layer is stricter than outer engine layer |
| `vedic-clock` | main failures use `CalculationError` | [`crates/engine-vedic-clock/src/engine.rs:276`](../../crates/engine-vedic-clock/src/engine.rs#L276) | No fine-grained validation taxonomy exposed at engine boundary |
| `face-reading` | failures use `CalculationError` | [`crates/engine-face-reading/src/engine.rs:144`](../../crates/engine-face-reading/src/engine.rs#L144) | Coarse but internally consistent |
| `nadabrahman` | failures use `CalculationError` | [`crates/engine-nadabrahman/src/engine.rs:260`](../../crates/engine-nadabrahman/src/engine.rs#L260) | Coarse but internally consistent |

### TS bridge engines

All TS engines share one HTTP wrapper and one Rust bridge adapter:

- TS sidecar response surface: [`ts-engines/src/server/app.ts:141`](../../ts-engines/src/server/app.ts#L141)
- Rust bridge translation: [`crates/noesis-bridge/src/lib.rs:218`](../../crates/noesis-bridge/src/lib.rs#L218)

| Engine | Current effective error pattern | Source refs | Consistency note |
| --- | --- | --- | --- |
| `tarot` | thrown exceptions become sidecar `500 CALCULATION_ERROR` | [`ts-engines/src/server/app.ts:169`](../../ts-engines/src/server/app.ts#L169) | No engine-specific error taxonomy |
| `i-ching` | same shared wrapper | [`ts-engines/src/server/app.ts:169`](../../ts-engines/src/server/app.ts#L169) | No engine-specific error taxonomy |
| `enneagram` | same shared wrapper | [`ts-engines/src/server/app.ts:169`](../../ts-engines/src/server/app.ts#L169) | No engine-specific error taxonomy |
| `sacred-geometry` | same shared wrapper | [`ts-engines/src/server/app.ts:169`](../../ts-engines/src/server/app.ts#L169) | No engine-specific error taxonomy |
| `sigil-forge` | bridge adapter adds pre-validation, sidecar still wraps thrown exceptions as `500 CALCULATION_ERROR` | [`crates/noesis-bridge/src/lib.rs:219`](../../crates/noesis-bridge/src/lib.rs#L219), [`ts-engines/src/server/app.ts:169`](../../ts-engines/src/server/app.ts#L169) | Only TS engine with extra Rust-side pre-validation today |

### Highest-signal inconsistencies

1. Invalid user-supplied date/time input is still classified as `CalculationError` in `vimshottari`, `numerology`, and `biorhythm`, but as `ValidationError` in `human-design` and `transits`.
2. TS bridge engines collapse all thrown engine exceptions into `500 CALCULATION_ERROR` at the sidecar layer, losing granularity before Rust receives the error.
3. `noesis-bridge` now cleanly maps sidecar 4xx to `ValidationError` and 5xx/transport faults to `BridgeError`, but the preserved context is still string-only rather than structured.
4. Serialization failures are inconsistently classified:
   - `numerology` uses `InternalError`
   - `panchanga` and `biorhythm` use `CalculationError`
5. Helper modules sometimes use stricter validation categories than their parent engines, especially in `biofield`.

### Recommended fix categories

- Normalize input parsing failures to `ValidationError` across all user-input-driven native engines.
- Preserve structured bridge context instead of string-only `BridgeError` messages.
- Move TS engine error responses to a typed `error_code` taxonomy instead of a blanket `CALCULATION_ERROR`.
- Normalize serialization failures to `InternalError` where the failure is not user-caused.

## `#48` Current Bridge Propagation State

Current path:

- TS sidecar returns `404 ENGINE_NOT_FOUND`, `403 PHASE_ACCESS_DENIED`, or `500 CALCULATION_ERROR` from [`ts-engines/src/server/app.ts:125`](../../ts-engines/src/server/app.ts#L125)
- Rust bridge adapter maps transport failures to `EngineError::BridgeError` and sidecar 4xx to `EngineError::ValidationError` in [`crates/noesis-bridge/src/lib.rs:247`](../../crates/noesis-bridge/src/lib.rs#L247) and [`crates/noesis-bridge/src/lib.rs:277`](../../crates/noesis-bridge/src/lib.rs#L277)
- API mapping then converts those `EngineError` values to HTTP responses in [`crates/noesis-api/src/lib.rs:1392`](../../crates/noesis-api/src/lib.rs#L1392)

This is enough to describe the live propagation path, but it does not satisfy the issue body as written because there is no dedicated bridge error enum with six variants in the current codebase.

## `#49` noesis-api Handler Error Pattern Audit

### Inline handlers in `lib.rs`

| Handler | Pattern | Source refs |
| --- | --- | --- |
| `calculate_handler` | centralized `engine_error_to_response()` | [`crates/noesis-api/src/lib.rs:706`](../../crates/noesis-api/src/lib.rs#L706), [`crates/noesis-api/src/lib.rs:834`](../../crates/noesis-api/src/lib.rs#L834) |
| `validate_handler` | mixed: ad-hoc `404` for missing engine, then centralized mapper for engine validation errors | [`crates/noesis-api/src/lib.rs:857`](../../crates/noesis-api/src/lib.rs#L857), [`crates/noesis-api/src/lib.rs:867`](../../crates/noesis-api/src/lib.rs#L867) |
| `engine_info_handler` | ad-hoc `404` response | [`crates/noesis-api/src/lib.rs:901`](../../crates/noesis-api/src/lib.rs#L901) |
| `workflow_execute_handler` | centralized `engine_error_to_response()` | [`crates/noesis-api/src/lib.rs:968`](../../crates/noesis-api/src/lib.rs#L968), [`crates/noesis-api/src/lib.rs:1098`](../../crates/noesis-api/src/lib.rs#L1098) |
| `workflow_info_handler` | ad-hoc `404` response | [`crates/noesis-api/src/lib.rs:1149`](../../crates/noesis-api/src/lib.rs#L1149), [`crates/noesis-api/src/lib.rs:1157`](../../crates/noesis-api/src/lib.rs#L1157) |
| `list_readings_handler` | ad-hoc DB and UUID errors | [`crates/noesis-api/src/lib.rs:1207`](../../crates/noesis-api/src/lib.rs#L1207) |
| `get_reading_handler` | ad-hoc DB, UUID, and not-found errors | [`crates/noesis-api/src/lib.rs:1274`](../../crates/noesis-api/src/lib.rs#L1274) |
| `readings_stats_handler` | ad-hoc DB and UUID errors | [`crates/noesis-api/src/lib.rs:1328`](../../crates/noesis-api/src/lib.rs#L1328) |
| `health_handler`, `readiness_handler`, `status_handler`, `list_engines_handler`, `list_workflows_handler`, `legacy_panchanga_handler`, `legacy_ghati_current_handler` | no `ErrorResponse` path at handler boundary | [`crates/noesis-api/src/lib.rs:539`](../../crates/noesis-api/src/lib.rs#L539), [`crates/noesis-api/src/lib.rs:563`](../../crates/noesis-api/src/lib.rs#L563), [`crates/noesis-api/src/lib.rs:667`](../../crates/noesis-api/src/lib.rs#L667), [`crates/noesis-api/src/lib.rs:940`](../../crates/noesis-api/src/lib.rs#L940), [`crates/noesis-api/src/lib.rs:1116`](../../crates/noesis-api/src/lib.rs#L1116), [`crates/noesis-api/src/lib.rs:1532`](../../crates/noesis-api/src/lib.rs#L1532), [`crates/noesis-api/src/lib.rs:1624`](../../crates/noesis-api/src/lib.rs#L1624) |

### `handlers/auth.rs`

All route handlers return `Result<Response, ApiError>`, which delegates to `engine_error_to_response()` through [`crates/noesis-api/src/error.rs:7`](../../crates/noesis-api/src/error.rs#L7).

Handlers:

- [`register`](../../crates/noesis-api/src/handlers/auth.rs#L120)
- [`login`](../../crates/noesis-api/src/handlers/auth.rs#L172)
- [`forgot_password`](../../crates/noesis-api/src/handlers/auth.rs#L275)
- [`reset_password`](../../crates/noesis-api/src/handlers/auth.rs#L314)
- [`change_password`](../../crates/noesis-api/src/handlers/auth.rs#L364)

### `handlers/users.rs`

Both route handlers return `Result<Response, ApiError>` and use centralized mapping:

- [`get_me`](../../crates/noesis-api/src/handlers/users.rs#L63)
- [`update_me`](../../crates/noesis-api/src/handlers/users.rs#L180)

### `handlers/admin.rs`

Most route handlers return `Result<Response, ApiError>`, but this file also contains several helper functions that build ad-hoc `Response` values directly:

- [`json_error_response`](../../crates/noesis-api/src/handlers/admin.rs#L636)
- [`service_unavailable_response`](../../crates/noesis-api/src/handlers/admin.rs#L653)
- [`forbidden_response`](../../crates/noesis-api/src/handlers/admin.rs#L662)
- [`parse_uuid_or_422`](../../crates/noesis-api/src/handlers/admin.rs#L674)
- [`require_permission_or_forbidden`](../../crates/noesis-api/src/handlers/admin.rs#L887)
- [`admin_repo_or_503`](../../crates/noesis-api/src/handlers/admin.rs#L896)

Route handlers using `ApiError`:

- [`get_session`](../../crates/noesis-api/src/handlers/admin.rs#L918)
- [`list_users`](../../crates/noesis-api/src/handlers/admin.rs#L965)
- [`update_user_state`](../../crates/noesis-api/src/handlers/admin.rs#L1033)
- [`update_user_tier`](../../crates/noesis-api/src/handlers/admin.rs#L1114)
- [`update_user_roles`](../../crates/noesis-api/src/handlers/admin.rs#L1187)
- [`list_api_keys`](../../crates/noesis-api/src/handlers/admin.rs#L1286)
- [`create_api_key`](../../crates/noesis-api/src/handlers/admin.rs#L1358)
- [`revoke_api_key`](../../crates/noesis-api/src/handlers/admin.rs#L1479)
- [`rotate_api_key`](../../crates/noesis-api/src/handlers/admin.rs#L1538)
- [`history_sync_users`](../../crates/noesis-api/src/handlers/admin.rs#L1618)
- [`history_sync_devices`](../../crates/noesis-api/src/handlers/admin.rs#L1691)
- [`history_sync_events`](../../crates/noesis-api/src/handlers/admin.rs#L1765)
- [`analytics_summary`](../../crates/noesis-api/src/handlers/admin.rs#L1840)
- [`analytics_timeseries`](../../crates/noesis-api/src/handlers/admin.rs#L1902)
- [`analytics_breakdown`](../../crates/noesis-api/src/handlers/admin.rs#L1975)
- [`analytics_top_consumers`](../../crates/noesis-api/src/handlers/admin.rs#L2047)
- [`system_health`](../../crates/noesis-api/src/handlers/admin.rs#L2104)
- [`system_services`](../../crates/noesis-api/src/handlers/admin.rs#L2176)
- [`system_workflows`](../../crates/noesis-api/src/handlers/admin.rs#L2277)
- [`system_cache`](../../crates/noesis-api/src/handlers/admin.rs#L2405)
- [`list_audit_events`](../../crates/noesis-api/src/handlers/admin.rs#L2456)
- [`get_audit_event`](../../crates/noesis-api/src/handlers/admin.rs#L2540)
- [`list_audit_actions`](../../crates/noesis-api/src/handlers/admin.rs#L2597)

### `#49` conclusion

Current state is mixed, not fully unified:

- `auth.rs`, `users.rs`, and most of `admin.rs` already use centralized mapping through `ApiError`
- `calculate_handler` and `workflow_execute_handler` use `engine_error_to_response()` directly
- several inline handlers in `lib.rs` and several admin helpers still construct ad-hoc `ErrorResponse` values

That means the audit issue is complete, but the full unification issue is still `#50`, not `#49`.
