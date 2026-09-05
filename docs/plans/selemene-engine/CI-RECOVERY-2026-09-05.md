# Clean-environment CI recovery

Draft candidate: [PR #1488](https://github.com/Sheshiyer/Selemene-engine/pull/1488). The initial run [33967732193](https://github.com/Sheshiyer/Selemene-engine/actions/runs/33967732193) checks source `747555bf1f8a679ef3e4ac3ce61c3ff066fed5d1` through GitHub's pull-request merge context. Passing local tests did not reveal all clean-runner and provider packaging gaps.

| Failure | Repair | Evidence available before the next CI run |
|---|---|---|
| Workspace Gate could not import `yaml` | Pin PyYAML in the gate requirements | A new Python 3.12 environment installed only those requirements and passed all 67 script tests |
| Admin SDK prerequisite could not resolve witness-pipeline output | Build witness-pipeline before SDK in the admin CI job | Same dependency order as the already successful Vercel application compilation; actionlint passes |
| MediaPipe Linux image could not load `libxcb.so.1` | Install its OpenCV and audio runtime libraries in the slim image | Initial Biofield image import check and both Python-version contract/smoke jobs pass; repaired MediaPipe image still requires remote verification |
| Vercel adapter could not find `.next/next-server.js.nft.json` after successful application compilation | Let Vercel own packaging while retaining standalone output for self-hosting | Both configuration branches retain the same base path and redirects; candidate preview must still prove provider completion |

The Next.js failure matches the [upstream standalone/adapter regression](https://github.com/vercel/next.js/issues/96646). The conditional output workaround avoids a dependency downgrade. Local builds alone cannot verify the Vercel adapter, so a successful preview receipt at the repaired revision is required.

The first run also passes the complete Node audit, Rust security audit, secret scan, TypeScript engines, Python 3.11/3.12 contracts and live sidecar smoke, Biofield Linux image imports and workflow registry parity. These results belong to the initial candidate. Read the latest PR checks for the repaired revision; do not treat this dated receipt as an assertion that every check or deployment is complete.

No production promotion, ruleset change, database mutation or live infrastructure edit is part of these repairs.

## Database-backed capability fixture

The next run, [33968504428](https://github.com/Sheshiyer/Selemene-engine/actions/runs/33968504428), passes the repaired workspace/admin checks, both Linux Python images, both Python contract/smoke versions, audits, Rust tests and release build. Integration tests expose a preserved capability test defect: its shared Router/PgPool outlives the separate Tokio runtimes created by three test attributes. A valid UUID admin request then times out resolving database permissions.

A new disposable local PostgreSQL 18.4 instance with repository migrations reproduces the same 500 failure using the original fixture. A test-binary-owned runtime keeps all three independently named cases alive in one runtime and passes 3/3 with the same database, UUID identities, real permission lookup and unchanged assertions. CI supplies PostgreSQL 16, so the complete remote rerun remains necessary. Production permission handling was not relaxed. The inaccurate comment claiming no database calls was corrected.

The `noesis-build` response returned an intention instead of code and was rejected. The bounded `noesis-fast` response resolved to `codex/gpt-5.6-luna`; its small fixture edits were reviewed. Its large replacement altered a field name and had no exact anchor, so that replacement was rejected. The original assertion body was wrapped mechanically instead. Compiler/test evidence applies to the integrated result, not the provider's unreviewed proposal.

## Generated route inventory

Run [33970102616](https://github.com/Sheshiyer/Selemene-engine/actions/runs/33970102616) proves all three database-backed capability route tests pass in CI. It then fails the strict API route inventory comparison because the earlier capability slice did not regenerate the baseline JSON. The repository's existing generator adds only GET `/admin/engines/capabilities`, changing 95 paths/100 methods to 96/101. The unchanged comparison test passes after generation; its generator remains an explicitly invoked ignored helper, not a skipped required assertion.

Integration CI now uses `--no-fail-fast` so all test binaries run and report failures while the overall job still fails. Actionlint and all 67 script tests pass after that command change. The current-source full rerun remains required; no failed CI run is waived.
