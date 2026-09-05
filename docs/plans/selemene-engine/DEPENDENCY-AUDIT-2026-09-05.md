# Dependency audit — 2026-09-05

Scope: recovery source based on `9a05f5cc90777f083d7065e491484ef727aeada1`. Audit results describe candidate lockfiles and a newly resolved local Python environment. Installed production packages and image digests still require deployment evidence.

| Surface | Baseline | Repaired candidate |
|---|---|---|
| Rust | 0 known vulnerabilities; chacha20 0.10.1 yanked warning | 0 known vulnerabilities; explicit disposition below; Cargo.lock unchanged |
| Node production | 20 high, 11 moderate, 2 low | All severity counts zero |
| Node production + development | 2 critical, 17 high, 19 moderate, 1 low in first complete-graph audit | All severity counts zero after final frozen installation |
| Python | No project lock; old local development pip 26.1.2 advisory | uv.lock committed for repeatable resolution; 54 packages in fresh locked environment audited, 0 vulnerabilities |

Advisory counts are not counts of distinct packages. Node baseline audits were taken at different points before the complete repair; production and full-graph baselines must not be added together.

## Changes and compatibility evidence

Effective overrides moved from npm-style root `overrides` into `pnpm.overrides`. Next and eslint-config-next converge on 16.3.4. shadcn moves to 4.21.0. The SDK, verification and witness packages move from Vitest 2.1.8 to 3.2.6. Affected transitive majors retain scoped override ranges; no advisory ignore list was added. Root package.json and pnpm-lock.yaml record the exact ranges/resolution.

`pnpm install --frozen-lockfile` passed. The canonical repository gate passed: 53 script checks, 26 Rust contract/OpenAPI/calculate checks, engine SDK 35, general SDK 11, verification 36 and TS engines 93. Witness passed 104 tests across 31 files, including real browser PDF output and temporary migration fixtures. Admin lint, production build, typecheck and its three existing Node tests passed. These counts precede additional independent-review regression tests; the final verification receipt records the final count.

The previous admin package had no `test` script; the recovered gate now invokes three real existing tests. Witness fixtures no longer depend on a private brand directory. Runbook tests create and clean their own temporary directory, and the migration script validates all input files before creating a destination.

Python uses uv 0.7.13 with `uv sync --locked`. The fresh local environment passed all 61 sidecar tests and its audit found no vulnerabilities. Both Dockerfiles consume uv.lock; Linux image builds/imports and both Python 3.11/3.12 contract suites are required in candidate CI. A local Docker daemon was unavailable, so local tests are not reported as image verification.

## Yanked Rust dependency disposition

The unchanged lockfile contains `chacha20 0.10.1 <- rand 0.10.2 <- quinn-proto 0.11.16 <- quinn 0.11.9 <- reqwest 0.12.28`. The current default macOS feature graph does not activate this optional HTTP/3 branch (`cargo tree -i chacha20@0.10.1 --locked` reports nothing to print); source does not enable reqwest HTTP/3. This is a retained lockfile warning, not a remediated or suppressed yank. Re-evaluate it before enabling HTTP/3 or a target/feature set that activates the branch. Zero known advisories does not certify every target's supply chain.

## Release evidence still required

GSD 02 owns candidate CI and immutable GitHub Action references. The pin validator covers `uses:` references, not floating Docker base/service tags or operating-system package repositories; full image reproducibility remains explicit work in GSD 07. GSD 07 must attach source/schema revisions and runtime probes to each rebuilt production digest. No audit here proves the running Railway images were upgraded.
