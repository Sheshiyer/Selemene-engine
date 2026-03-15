# API Config Environment Parity

Baseline audit for `ApiConfig::from_env()` as of 2026-03-11.

Validation path:

```bash
cargo run -p noesis-api --bin validate_config -- --dry-run
```

The CI workflow now runs that dry-run audit in `.github/workflows/test.yml`.

## Parity Matrix

| Env var | Local `.env` / `.env.example` | CI (`.github/workflows/test.yml`) | Railway / deploy docs | Current status |
| --- | --- | --- | --- | --- |
| `RUST_ENV` | set to `development` | unset, defaults to development behavior | dashboard-managed / not represented in repo | documented |
| `HOST` | not set directly | unset, defaults to `0.0.0.0` | docs are mixed | documented |
| `SERVER_HOST` | set to `0.0.0.0` | unset | `docs/deployment/docker.md`, `docs/deployment/kubernetes.md` still use this legacy alias | supported alias |
| `PORT` | not set directly | unset in test workflow, defaults to `8080`; Railway injects runtime port | `docs/deployment/README.md` uses `PORT` | documented |
| `SERVER_PORT` | set to `8080` | unset | `docs/deployment/docker.md`, `docs/deployment/kubernetes.md` still use this legacy alias | supported alias |
| `JWT_SECRET` | set | unset explicitly in CI, dev default is used unless a test overrides it | expected as secret in production | documented |
| `DATABASE_URL` | set | set | dashboard-managed / secret in hosted environments | documented |
| `REDIS_URL` | set | set | add-on / dashboard-managed in Railway | documented |
| `ALLOWED_ORIGINS` | unset, defaults to localhost origins | unset, defaults to localhost origins | deploy docs describe CORS, but Railway value is not tracked in repo | documented |
| `RATE_LIMIT_REQUESTS` | set to `100` | unset, defaults to `100` | deploy docs cover the knob, Railway value not tracked in repo | documented |
| `RATE_LIMIT_WINDOW_SECS` | unset, defaults to `60` | unset, defaults to `60` | not tracked in Railway docs | documented |
| `REQUEST_TIMEOUT_SECS` | unset, defaults to `30` | unset, defaults to `30` | not tracked in Railway docs | documented |
| `RUST_LOG` | set to `info` | not set per-job, falls back to crate default | deploy docs mention it in Docker/Kubernetes | documented |
| `LOG_FORMAT` | unset, defaults to `pretty` | unset, defaults to `pretty` | deploy docs use `json` for hosted setups | documented |

## Drift Notes

1. `HOST` / `PORT` are the canonical config keys in code. `SERVER_HOST` / `SERVER_PORT` remain widely used in local and deployment examples, so `ApiConfig::from_env()` now accepts them as compatibility aliases.
2. Railway runtime configuration is partially dashboard-managed, so repo-visible parity is limited to deployment docs and workflow variables. This checklist marks those entries as "documented" rather than "repo-enforced".
3. CI uses explicit `DATABASE_URL` and `REDIS_URL`, but still relies on development defaults for several optional settings. That is intentional and now covered by the dry-run audit.
