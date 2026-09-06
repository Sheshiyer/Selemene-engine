# Dependency audit refresh — 2026-09-06

Scope: recovery candidate `4305265acee96461c40594fbb2689306d357f59d`. These are current lockfile and local-environment results. They do not identify or certify the packages installed in the existing Railway images.

| Surface | Current result | Command boundary |
|---|---|---|
| Node production | 0 critical, high, moderate, low, or informational findings | `pnpm audit --prod --audit-level=high --json` |
| Node complete graph | 0 critical, high, moderate, low, or informational findings | `pnpm audit --audit-level=high --json` |
| Rust | 0 known vulnerabilities; one yanked-package warning | `cargo audit --json` |
| Python | 52 resolved packages audited; 0 vulnerabilities | locked `uv export` plus isolated `pip-audit` |

The Rust warning remains the documented inactive `chacha20 0.10.1` lockfile path through optional HTTP/3 dependencies. No advisory was ignored and no dependency changed during this refresh.

`uv lock --check` passed with uv 0.7.13 and CPython 3.11.15. The Python audit consumed a temporary requirements export from the committed lockfile and did not modify the repository.

After the final resolver repair made the already-locked TypeScript 5.9.3 parser a direct root development dependency, both Node audits were rerun with zero findings. `pnpm install --frozen-lockfile --offline --ignore-scripts` also passed across all eight workspace projects, proving the parser dependency is available from the committed lock without a network resolution step.

The full database-free repository gate remains the source-level compatibility proof. Remote CI must still pass on the final pushed candidate, and a future release receipt must bind rebuilt image digests before any production package claim.
