# Release Checklist Template

Use this template for future Selemene releases. Copy the checklist, fill in the
placeholders, and keep the completed version with the release artifacts.

## Release Header

- Release version: `vX.Y.Z`
- Release date: `YYYY-MM-DD`
- Release owner: `<name>`
- Change window: `<start/end>`
- Stability window: `<hours or days>`
- Release branch / tag: `<branch-or-tag>`

## Version Matrix

| Surface | Target version | Verified by | Notes |
| --- | --- | --- | --- |
| `noesis-api` | `vX.Y.Z` | `<owner>` |  |
| `ts-engines` | `vX.Y.Z` | `<owner>` |  |
| Admin web / companion surface | `vX.Y.Z` | `<owner>` |  |
| Monitoring dashboards / alert rules | `vX.Y.Z` | `<owner>` |  |
| SDK / client surfaces | `vX.Y.Z` | `<owner>` | optional |

## Release Gates

- [ ] Scope is frozen for this release.
- [ ] All must-ship GitHub issues are closed or explicitly deferred.
- [ ] Release notes are updated in [docs/RELEASE_NOTES.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/RELEASE_NOTES.md).
- [ ] Docs impacted by the release are updated and linked.

## Verification

### Core API

- [ ] `GET /health/live` returns `200`
- [ ] `GET /ready` returns acceptable dependency state for the target environment
- [ ] `GET /metrics` responds and exposes expected metric families
- [ ] Representative engine calculation succeeds
- [ ] Representative workflow execution succeeds

### Smoke / Release Gate

- [ ] `scripts/smoke-test-runner.sh` passes against the intended target
- [ ] Any release-specific smoke checks are attached to the release artifact
- [ ] Rollback command/path is documented and tested in dry-run mode if available

### Monitoring / Observability

- [ ] Prometheus config and alert rules match the deployed topology
- [ ] Grafana dashboard JSON parses cleanly
- [ ] Alertmanager receiver routes are reviewed for this release
- [ ] Sentry DSN / environment wiring is verified if Sentry is enabled
- [ ] Trace/log correlation plan is confirmed if Jaeger/Loki are enabled

Reference:

- [docs/monitoring/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/monitoring/README.md)
- [docs/runbooks/README.md](/Volumes/madara/2026/witnessos/Selemene-engine/docs/runbooks/README.md)

## Security and Secrets

- [ ] `cargo audit` or equivalent dependency audit completed
- [ ] Required secrets exist in the target environment
- [ ] No temporary credentials, debug bypasses, or test DSNs remain enabled
- [ ] Auth and rate-limit assumptions were reviewed for release impact

## Data / Infrastructure

- [ ] Database migrations are identified and ordered
- [ ] Migration/rollback path is documented
- [ ] Redis / cache degradation behavior is acceptable for the release
- [ ] Railway / hosting environment variables are reviewed if applicable

## Documentation Freeze

- [ ] Public docs are updated
- [ ] Deployment docs reflect the current runtime
- [ ] Monitoring docs reflect the current repo-visible config
- [ ] Any stale provider/runtime wording has been removed or explicitly noted

## Stability Window

- [ ] Assigned owner for the post-release watch window
- [ ] Alert channels and runbooks are linked in the release handoff
- [ ] Success criteria for ending the watch window are defined
- [ ] Rollback threshold is defined before deployment starts

## Sign-Off

| Role | Name | Status | Timestamp | Notes |
| --- | --- | --- | --- | --- |
| Tech Lead |  |  |  |  |
| Release Owner |  |  |  |  |
| Ops / SRE |  |  |  |  |
| Product / Docs |  |  |  |  |

## Post-Release Review

- [ ] Actual deploy time recorded
- [ ] Incidents or anomalies recorded
- [ ] Follow-up issues created for anything deferred
- [ ] Completed checklist archived with release artifacts
