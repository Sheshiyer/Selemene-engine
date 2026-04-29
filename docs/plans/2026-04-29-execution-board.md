# Execution Board - 2026-04-29

## Scope

1. Mainline CI/CD unblocker
2. v2.2 closure sprint for issues #393 #394 #395 #407 #408 #409 #410 #416 #417 #421
3. Quick stale triage pass for P4/P5/v3.0
4. Choose next product lane

## Status Snapshot

- Mainline blocker reproduced from CI logs:
  - `engine-biorhythm` compile break
  - rustfmt check failures
- Local unblocker fix applied:
  - `crates/engine-biorhythm/src/lib.rs` repaired for duplicate symbol conflicts and missing secondary-cycle fields
  - `cargo check -p engine-biorhythm` passes
  - `cargo fmt --all -- --check` passes
- Stale triage pass executed:
  - Label created: `triage:stale-2026-04-29`
  - 71 open issues labeled across P4/P5/v3.0 stale set
- v2.2 issue kickoff comments posted on:
  - #393 #394 #395 #407 #408 #409 #410 #416 #417 #421

## Work Board

### Batch 0 - Mandatory Unblocker (Sequential)

- Issue: CI/CD mainline breakage
- Owner lane: Platform Rust
- Branch: `agent/fix-mainline-biorhythm-ci-unblocker`
- Done criteria:
  - CI `Test & Lint` green on main or hotfix PR
  - CD `Build & Deploy` green on main or hotfix PR

### Batch 1 - v2.2 Backend + Docs (Parallel Lane A/B)

- Lane A (backend workflows)
  - #393 branch `agent/issue-393-v22-w2-s5-01-daily-practice-secondary-synthesis`
  - #394 branch `agent/issue-394-v22-w2-s5-02-full-spectrum-compatibility-mode`
- Lane B (docs)
  - #395 branch `agent/issue-395-v22-w2-s5-03-biorhythm-docs-secondary-compatibility`

Done criteria:
- Daily-practice synthesis explicitly handles notable aesthetic/spiritual signals
- Full-spectrum workflow path passes `partner_birth_date` for compatibility mode
- Biorhythm API docs include secondary cycles + compatibility examples

### Batch 2 - v2.2 QA Hardening (Parallel Lane C/D)

- Lane C (engine QA)
  - #407 branch `agent/issue-407-v22-w3-s9-01-secondary-cycle-validation-tests`
  - #408 branch `agent/issue-408-v22-w3-s9-02-compatibility-validation-tests`
  - #409 branch `agent/issue-409-v22-w3-s9-03-biorhythm-benchmark-suite-validation`
  - #410 branch `agent/issue-410-v22-w3-s9-04-biorhythm-contract-test-hardening`
- Lane D (workflow e2e + release docs)
  - #416 branch `agent/issue-416-v22-w3-s11-01-daily-practice-4-engine-e2e`
  - #417 branch `agent/issue-417-v22-w3-s11-02-full-spectrum-expanded-e2e`
  - #421 branch `agent/issue-421-v22-w3-s11-06-v220-changelog-version-bump`

Done criteria:
- Engine-level test matrices satisfy acceptance criteria for secondary + compatibility
- Daily-practice and full-spectrum e2e tests validate required v2.2 outputs
- Changelog and versioning evidence documented per issue requirements

## Chosen Next Lane

Selected lane after sprint kickoff: **v3 security/hardening**

Rationale:
- Highest release risk reduction after CI/CD stability
- Directly aligned with stale P5/v3 release-readiness backlog
- Security hardening tasks are high-impact and minimally blocked by UI work

### v3 Lane Start Set

- #459 branch `agent/issue-459-v30-w3-s1-01-cargo-audit-remediation`
- #460 branch `agent/issue-460-v30-w3-s1-02-jwt-owasp-audit`
- #462 branch `agent/issue-462-v30-w3-s1-04-input-validation-hardening`

Done criteria:
- `cargo audit` clean (or explicit documented allowlist)
- JWT handling audited and remediated against OWASP checklist
- Input validation tightened with tests for malformed payload paths

## Notes

- This board is execution-first and intentionally parallelized.
- If CI on main turns red again, all lanes pause and return to Batch 0.
