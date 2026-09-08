# External release candidate checklist

Status: production promoted; package publication remains held
Release record: [3.3.1 — Dodo-free mode production release](https://github.com/Sheshiyer/Selemene-engine/releases/tag/release-3.3.1-free-mode-20260908)
Main merge: `561c3e476e134eeac424c77b092b95b9749e0215`
Review: [PR #1489](https://github.com/Sheshiyer/Selemene-engine/pull/1489)

This document is the external-release handoff for the Dodo-free billing and
Tarot truth work. Production promotion is complete through the connected
workflow; it records what shipped and what remains intentionally blocked.

## Public contract

- The public API retains the billing routes and SDK methods.
- `BILLING_MODE=free` is the safe default when the variable is absent.
- `BILLING_MODE=disabled` is an operator hard ceiling.
- Paid Dodo behavior requires explicit mode, complete credentials, and current
  provider evidence.
- Historical Dodo subscription and promotion rows remain audit data; they do
  not prove current entitlement or extend the prior promotion duration.
- Tarot has a deterministic local fixture. Generated, fallback, and
  unavailable Tarot states are not claimed because this implementation does
  not expose those paths.

## Distribution surfaces

| Surface | Prepared state | Release gate |
| --- | --- | --- |
| `noesis-core` | Manifest and README carry repository metadata | Registry receipt at candidate version |
| `noesis-sdk` | Manifest and README carry repository metadata | `noesis-core` receipt, then SDK receipt |
| `noesis-tui` | Manifest and README added; source install documented | Core and SDK receipts, then TUI receipt |
| `@noesis/sdk` | README and billing caveat updated | Build, pack, registry receipt |
| `@selemene/bridge` | Package provenance metadata updated | Build, pack, registry receipt |
| `@selemene/engine-sdk` | Package provenance metadata updated | Build, pack, registry receipt |

The current local evidence still holds Rust SDK/TUI publication because the
registry does not expose the required `noesis-core` dependency line. No package
has been published or installed from a candidate registry version.

## External receipts

- [x] Exact merged source has a fresh green canonical CI Gate: [run 34258440332](https://github.com/Sheshiyer/Selemene-engine/actions/runs/34258440332).
- [ ] Package dry-runs pass from a clean candidate commit.
- [ ] Registry pages show the dependency chain in publish order.
- [x] Workflow checked out and validated source SHA `561c3e47`; Railway
      production deployment `ee9cb602-d3d3-4db3-a8d6-d06fc75090a8` succeeded,
      `BILLING_MODE=free` is explicitly set, and live health/readiness pass.
- [ ] Railway schema revision and deployed-image-to-GHCR digest equivalence
      still need a provider-level receipt. The Railway-built digest and GHCR
      provenance digest are recorded separately.
- [ ] Cloudflare source/account bindings are read back using the explicitly
      scoped account session.
- [ ] Vercel production protection and same-origin admin proxy checks pass.
- [ ] Rollback target and schema rollback path are recorded and exercised.
- [x] Production deploy and release record are published; package publication
      remains held until the package and provider receipts above exist.

## Current holds

- Advisor verification is unavailable because the local OAuth session expired.
- The production workflow [34258440332](https://github.com/Sheshiyer/Selemene-engine/actions/runs/34258440332)
  passed Security Audit, Secret Scanning, Workspace Gate, TS Engines, Lint,
  Python Sidecars, Workflow Registry Parity, Test, Integration Tests, Build,
  source validation, both image builds, Railway deploy, API smoke, and admin
  smoke. Kubernetes and release-artifact publication were intentionally
  skipped.
- Local Postgres lacks the configured `noesis_user` role; local DB-backed billing
  replay is not a production authorization receipt.
- Cloudflare scoped-account verification, Vercel production protection, exact
  Railway schema/image equivalence, package publication, and rollback exercise
  remain explicit holds.

## Rollback posture

Do not delete tags, revert deployments, alter production schema, or publish a
replacement package as an unplanned rollback. If a candidate is released and
fails, record the prior healthy source/image/schema tuple first, then use the
scoped provider runbook and a new patch version for immutable registries.
