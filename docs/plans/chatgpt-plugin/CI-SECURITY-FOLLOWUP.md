# PR 1492 security audit failure analysis

Observed 2026-09-30. Initial investigation was read-only. A subsequent explicit
assignment authorized the narrowly scoped Cargo.lock repair, validation, commit
and push described below. No audit suppression, merge or deployment is included.

## Finding

The fatal finding is **rustls 0.23.36, RUSTSEC-2026-0285**, a medium-severity
TLS 1.3 handshake validation vulnerability. The patched release is
**0.23.45 or later**. Cargo audit installed and ran successfully; it exited 1
because it found this vulnerability. This is not an installer, network or MCP
JavaScript dependency failure.

The same audit reported **chacha20 0.10.1 as yanked**, explicitly as one allowed
warning. That warning did not cause this job's failure and does not require
expanding the immediate fix.

Primary advisory: [RustSec RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285).
The issue concerns accepting handshake messages at the wrong encryption level.
The advisory says the transcript remains authenticated; the investigation does
not claim an observed compromise or exploit against Selemene.

## Exact CI evidence

- PR: [1492](https://github.com/Sheshiyer/Selemene-engine/pull/1492).
- Head: `45304412d6f907b652e77c6a2fdf21098721e7b7`.
- Base: `ebd97fe940a3454b0425ff60d56003b07337564e` (`main`).
- Workflow run: [36644962615](https://github.com/Sheshiyer/Selemene-engine/actions/runs/36644962615).
- Security job: [109665682580](https://github.com/Sheshiyer/Selemene-engine/actions/runs/36644962615/job/109665682580).
- Job checkout SHA in logs: `ed571519cb11b8df0796d096def86c93be013b89`
  (the workflow's `${{ github.sha }}` checkout, distinct from the PR head).
- Installed `cargo-audit 0.22.2`; loaded 1,277 advisories and scanned 634
  locked crate dependencies. The fatal audit output identified only rustls.
- Security audit failed at 2026-09-29T23:26:14Z.
- [CI Gate job 109671777215](https://github.com/Sheshiyer/Selemene-engine/actions/runs/36644962615/job/109671777215)
  explicitly recorded `SECURITY=failure`, while repository gate, lint,
  workflow parity, test, integration, secret scanning, build, TS engines and
  Python sidecars all recorded success. Its exit 1 is the aggregate consequence
  of the security lane, not an additional independent test failure.

Only relevant failure details are retained here; no credential values or raw
runner environment dumps are included.

## Changed versus inherited

`git diff` between the exact base and head found no changes to `Cargo.lock`,
`Cargo.toml`, `.cargo/audit.toml` or `.github/workflows/test.yml`.
Both revisions reference the identical Cargo.lock Git blob:

`31510d612b560b77534dceef379b720d3a939651`

The PR's changed-file list contains no Rust dependency manifest or lockfile
change. The security job runs root `cargo audit`; the new MCP package's npm
lockfile is not its input.

The base commit is dated 2026-09-09T00:13:44+05:30. The advisory was issued on
2026-09-14. Thus the vulnerable locked dependency is inherited from the base,
and the advisory postdates that base commit. This does not establish what any
earlier CI run concluded, and is not a reason to waive the current gate.

`cargo tree --locked --offline --invert rustls@0.23.36 --prefix none --depth 3`
successfully confirmed existing runtime dependency paths through:

- `hyper-rustls 0.27.7` and `tokio-rustls 0.26.4` to `reqwest 0.12.28`, used
  by the API, bridges and several engine clients.
- `sqlx-core 0.8.6`, including SQLx runtime/Postgres and macro dependencies.

This is a shared Rust workspace dependency rather than an isolated test-only
package. Source does not directly pin rustls in the inspected workspace
manifests; its present version is selected transitively in Cargo.lock.

## Smallest proposed repair

After source-edit ownership is assigned, perform a targeted lockfile update:

```sh
cargo update -p rustls@0.23.36 --precise 0.23.45
```

Inspect the resulting diff and retain only rustls and solver-required compatible
transitive changes. Do not run an unrestricted workspace update, remove the
security gate, or add an advisory ignore. The exact number of lockfile entries
that must change has not been solver-tested during this read-only assignment.

Validation after the update should include:

1. `cargo audit` against the current advisory database, confirming the fatal
   rustls finding is gone; distinguish any remaining allowed yanked warning.
2. `cargo tree --locked --invert rustls` to confirm the vulnerable version is
   no longer selected and inspect the affected dependency paths.
3. A focused compile/test pass for the affected API/auth/network dependency
   consumers using existing build caches, followed by the required PR CI lanes
   at the new head. A same-series dependency patch still needs build evidence.

## Authorized repair and local verification

The targeted update was applied in the isolated implementation worktree.
Final source scope is only Cargo.lock: four lines added and four removed,
changing versions/checksums for these two packages:

| Package | Before | After | Reason |
| --- | --- | --- | --- |
| rustls | 0.23.36 | 0.23.45 | Patched version for RUSTSEC-2026-0285 |
| rustls-webpki | 0.103.13 | 0.103.15 | rustls 0.23.45 requires webpki >=0.103.14 |

The solver initially rewired five unrelated Windows dependency references;
those references were restored to their prior values. The final locked graph
was accepted by Cargo with `--locked`, and contains no such unrelated changes.
No manifest, application source or security-policy change was needed.

Actual local verification:

- `cargo audit`: exit 0 against 1,277 freshly loaded advisories. No vulnerability
  finding; only the pre-existing allowed chacha20 0.10.1 yanked warning remains.
  A final `cargo audit --no-fetch` also exited 0 against the same database after
  the lockfile was minimized.
- `cargo tree --locked --offline --invert rustls --depth 1`: only rustls 0.23.45,
  used by hyper-rustls, reqwest, sqlx-core and tokio-rustls.
- Focused existing API tests, run with `--locked` and the shared build cache:

  ```sh
  CARGO_TARGET_DIR=/Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine/target \
    cargo test --locked -p noesis-api \
      --test report_phase_authority_tests \
      --test bridge_readiness_tests \
      --test living_readings_auth_tests
  ```

  Compilation completed in 1m 41s, including updated TLS, reqwest, SQLx,
  auth/data, bridge and API dependencies. **Five tests passed**: two bridge
  readiness tests, two living-reading authorization tests and the report-phase
  test covering ten authenticated HTTP scenarios. No test failed or was ignored.
- `git diff --check` passed for the lockfile and this report.

The repair is ready for its authorized commit/push. Replacement CI must be
observed against the exact resulting head; these local passes do not themselves
establish a green GitHub run. No merge or deployment is claimed.
