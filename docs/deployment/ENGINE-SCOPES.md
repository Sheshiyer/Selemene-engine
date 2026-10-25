# Engine permission scopes

Engine routes check that an **API-key principal** holds `"<engine_id>:read"`
(or `engines:*`, `admin:*`, `*`) before executing an engine:

- `POST /api/v1/engines/{engine_id}/calculate`
- `POST /api/v1/engines/face-reading/upload`
- `POST /api/v1/workflows/{workflow_id}/execute` (every engine the workflow composes)

An API-key principal is one that authenticated through the `X-API-Key` header;
the middleware marks that request explicitly (`ApiKeyPrincipal`). JWT and
Cloudflare Access principals carry only `basic:access` by design and are
governed by tier and quota; the scope rule does not apply to them, and the
check never infers the auth method from token fields such as `jti`.

## Mode

`ENGINE_SCOPE_MODE` on the `Selemene-engine` service:

| value | behaviour |
|---|---|
| `off` | no check |
| `audit` (default) | missing scope is logged (`engine scope missing on API key`, with `user_id`, `engine_id`, `required_permission`) and counted as API error code `ENGINE_SCOPE_AUDIT`; the request proceeds |
| `enforce` | missing scope returns `403 ENGINE_SCOPE_FORBIDDEN` with `details.engine_id` and `details.required_permission`; counted as `ENGINE_SCOPE_FORBIDDEN` |

The check runs before the free-tier quota, so a forbidden call never consumes an allowance.

## Rollout

1. Deploy with the default `audit` mode. Read the warning lines for a few days to see which keys and engines would be blocked.
2. Backfill scopes that existing premium/enterprise keys were never issued (raaga, transits, biofield-capture, financial-biosensor, sigil-forge) with `scripts/sql/backfill-engine-scopes.sql`. Free and basic keys keep the engines they were issued.
3. Set `ENGINE_SCOPE_MODE=enforce` and redeploy. Verify with a key scoped to one engine: another engine must return 403.

## Why the finding existed

Keys were issued with explicit engine lists, but `calculate` only ever checked that a principal was authenticated. A premium key scoped to `face-reading:read` executed `panchanga` in production on 2026-10-25.
