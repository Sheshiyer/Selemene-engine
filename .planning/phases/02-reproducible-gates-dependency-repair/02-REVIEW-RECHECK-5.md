---
phase: 02-reproducible-gates-dependency-repair
reviewed: 2026-09-06T16:17:58Z
depth: deep
reviewed_head: 4305265acee96461c40594fbb2689306d357f59d
files_reviewed: 10
files_reviewed_list:
  - .planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md
  - package.json
  - pnpm-lock.yaml
  - scripts/resolve_typescript_anchors.cjs
  - scripts/validate_contracts.py
  - tests/scripts/test_validate_contracts.py
  - contracts/v1/registries/engines.json
  - contracts/release/v1/manifest.json
  - contracts/release/v1/fixtures/eligible-source-redeploy.json
  - contracts/release/v1/fixtures/current-production-incomplete.json
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
wr_r4_verdict: closed
production_promotion: HOLD
---

# Phase 02: Fifth Independent Exact-Snapshot Recheck

**Reviewed:** 2026-09-06T16:17:58Z
**Exact committed source:** `4305265acee96461c40594fbb2689306d357f59d`
**Depth:** deep
**Status:** clean
**Production mutation disposition:** `HOLD`

## Summary

WR-R4-01 is closed at the exact committed snapshot. The shared Rust angle-close predicate now preserves `->` while both leading-generic and impl-body scans continue to count ordinary `>` and adjacent nested `>>` closes (`scripts/validate_contracts.py:324-341,360-396`). Compile-backed tests resolve `Fn`, `FnMut`, and `FnOnce` bounds, nested return generics, ordinary nested closes, and inherent and trait impl methods (`tests/scripts/test_validate_contracts.py:765-806`). An independent nine-case compile-and-resolve matrix also passed every valid spacing and adjacent-token variant tested.

The full cumulative resolver suite passed 141 contract tests, and the 40 release receipt tests also passed. All prior WR-R01, WR-R2, WR-R3, and WR-R4 counterexamples now behave as intended. The TypeScript compiler helper remains reproducible from a clean frozen installation, Markdown fence/comment ordering remains correct for the covered authority, and every current registry anchor resolves with matching release digests.

All reviewed files meet quality standards. No Critical, Warning, or Info findings remain in this exact review scope.

## Narrative Findings (AI reviewer)

No issues found.

## WR-R4-01 Closure

The committed compile-backed matrix passed these seven cases:

| Case | Impl form | Exact-head result |
|---|---|---|
| `F: Fn() -> bool` | inherent | Compiled and resolved |
| `F: FnMut() -> bool` | inherent | Compiled and resolved |
| `F: FnOnce() -> bool` | inherent | Compiled and resolved |
| `F: Fn() -> Option<Result<Vec<u8>, ()>>` | inherent, nested return and adjacent closes | Compiled and resolved |
| `F: Iterator<Item = Option<Vec<u8>>>` | inherent, ordinary nested `>>` closes | Compiled and resolved |
| `F: Fn() -> bool` | trait | Compiled and resolved |
| `F: FnOnce() -> Option<Vec<u8>>` | trait, nested return | Compiled and resolved |

The independent adjacent-token matrix then compiled each source with `rustc --edition 2021 --crate-type lib --emit metadata` and resolved `Owner::target` through the production validator. All nine passed:

1. `impl<F: Fn()->bool> Owner<F>`
2. `impl<F: Fn() ->bool> Owner<F>`
3. `impl<F: Fn()-> bool> Owner<F>`
4. A line-broken `Fn() -> Option<Result<Vec<u8>, ()>>` bound
5. `impl<F: for<'a> Fn(&'a str) -> &'a str> Owner<F>`
6. `impl<F: Fn() -> bool + Send> Owner<F>`
7. `impl<F: Fn() -> fn() -> bool> Owner<F>`
8. `impl<F: Fn() -> bool>/* owner */Owner<F>`
9. `impl<F> Owner<F> where F: Fn() -> Option<Vec<u8>>`

Result for every row:

```json
{"rustc": 0, "resolved": true, "diagnostic": ""}
```

The focused Rust selection also retained the surrounding safety controls:

- Function and impl tokens inside `()`, `[]`, and `{}` macro frames reject.
- Macro-definition bodies and macro input inside a real impl reject.
- A direct attributed `pub(crate)` impl method resolves.
- Lifetime/generic inherent and trait impl controls resolve.
- `r`, `br`, and `cr` raw strings, byte/C strings, characters, lifetimes, and labels preserve scope.
- Unbalanced delimiters and unterminated nested comments or literals fail closed.
- A brace-bearing const-generic impl header continues to fail closed with its bounded diagnostic.

The targeted command passed 24 cases with 117 deselected:

```text
$ python3 -m pytest tests/scripts/test_validate_contracts.py -q -k 'callable_generic_bounds or macro_token_tree or macro_definition or impl_scope or const_generic_brace or lifetime_generic or rust_raw_string or rust_byte_c_character or unterminated_rust'
24 passed, 117 deselected in 1.14s
```

## Prior Resolver Matrix

| Boundary | Exact-head verdict |
|---|---|
| Comment-only and string/template/raw/character-literal decoys | Rejected |
| Wrong Rust, Python, TypeScript, and JavaScript qualifiers | Rejected |
| Nested/local Rust, Python, TypeScript, and JavaScript declarations | Rejected |
| Python unqualified class member | Rejected; fully qualified module-level member resolved |
| Rust macro token trees and non-direct impl frames | Rejected |
| TypeScript/JavaScript named function and class expressions | Rejected across parenthesized, array, assignment, argument, conditional, and arrow contexts |
| TypeScript/JavaScript regex, division, `/=`, `/=/`, control-header, and post-block slash cases | Correctly resolved through the compiler AST |
| TypeScript helper failure, timeout/unavailability, missing dependency, diagnostics, malformed output, invalid shapes, duplicates, and unsupported anchors | Failed closed |
| `.tsx`, `.jsx`, and unsupported source suffixes | Failed closed |
| Markdown backtick/tilde fences and closed/unclosed HTML-comment ordering | Correct |

## TypeScript Clean-Install and Helper Boundary

The direct root dependency remains exact at `package.json:15-18`; the root lock importer binds `5.9.3` at `pnpm-lock.yaml:23-32`, with its integrity at `pnpm-lock.yaml:3629-3632`. The helper continues to use the TypeScript compiler API and fail on parse diagnostics (`scripts/resolve_typescript_anchors.cjs:11-16,35-75`). The Python boundary retains its timeout, process-failure, JSON-shape, symbol-grammar, and duplicate checks (`scripts/validate_contracts.py:425-468`).

Fresh exact-SHA receipt:

```text
$ git archive 4305265acee96461c40594fbb2689306d357f59d | tar -x -C <empty-temp-directory>
$ pnpm install --offline --ignore-scripts --frozen-lockfile
Scope: all 8 workspace projects
Packages: +715
Done in 6.4s using pnpm v10.33.0
$ node -e "const ts=require('typescript'); process.stdout.write(ts.version)"
5.9.3
$ python3 scripts/validate_contracts.py
contract authority v1 valid: schemas=6 fixtures=5 registries=1 engines=19
```

The lockfile SHA-256 remained `ba930dc5814352f0d47fbba825b10ecbef57d5728a5f5cf1a15f182de257f67d` before and after installation.

## Registry, Release, and Ledger Evidence

The engine registry contains 177 `repo://` references. Its 158 anchored occurrences reduce to seven unique targets, and all 158 resolve through the canonical validator:

1. `repo://crates/noesis-api/src/lib.rs#register_database_conditional_engines`
2. `repo://crates/noesis-bridge/src/lib.rs#BridgeManager::new`
3. `repo://crates/noesis-orchestrator/src/lib.rs#SUPPORTED_ENGINE_IDS`
4. `repo://crates/noesis-orchestrator/src/lib.rs#WorkflowOrchestrator::register_native_runtime_engines`
5. `repo://docs/plans/selemene-engine/CAPABILITY-LEDGER.md#engine-runtime-ledger`
6. `repo://docs/plans/selemene-engine/CAPABILITY-LEDGER.md#fresh-probe-receipt`
7. `repo://ts-engines/src/server/registry.ts#registerTypeScriptRuntimeEngines`

The recomputed registry digest is:

```text
sha256:c801fb49332f5c78cdf78417b6efd343cfcf30dc26e6ce7259cfc6157b707896
```

It matches `contracts/release/v1/manifest.json:6-10`, `contracts/release/v1/fixtures/eligible-source-redeploy.json:24-31`, and `contracts/release/v1/fixtures/current-production-incomplete.json:24-31`. Release validation accepted both fixtures and all nine mutation cases.

The fix ledger accurately records the prior exact head, one WR-R4 finding, its applied fix, the callable-bound regression coverage, and `fixes_complete_pending_independent_recheck` (`02-REVIEW-FIXES.md:18-26,234-276`). This independent report supplies that clean recheck. The ledger does not claim production readiness, and its production section preserves the external-authority boundary.

`deploy-production` and `release-production` both remain `disabled` at `contracts/release/v1/manifest.json:91-123`. Production promotion remains `HOLD`.

## Verification Receipts

| Command | Exact result |
|---|---|
| `git rev-parse HEAD` | `4305265acee96461c40594fbb2689306d357f59d` |
| `python3 -m pytest tests/scripts/test_validate_contracts.py tests/scripts/test_validate_release_receipt.py -q` | 181 passed in 72.91s: 141 contract and 40 release tests |
| Focused Rust boundary selection | 24 passed, 117 deselected in 1.14s |
| Independent compile-backed Rust spacing/token matrix | 9/9 compiled and resolved |
| `python3 scripts/validate_contracts.py` | Passed: `schemas=6 fixtures=5 registries=1 engines=19` |
| `python3 scripts/validate_release_receipt.py --validate-fixtures` | Passed: `receipts=2 mutation_cases=9` |
| `python3 -m py_compile scripts/validate_contracts.py tests/scripts/test_validate_contracts.py` | Passed |
| `node --check scripts/resolve_typescript_anchors.cjs` | Passed |
| Clean archive `pnpm install --offline --ignore-scripts --frozen-lockfile` | Passed; exact root `typescript@5.9.3`; lock unchanged |
| Registry traversal | 177 total references; 158 anchored occurrences; seven unique targets; validation passed |
| Registry SHA-256 linkage | Registry, manifest, and both digest-bearing fixtures matched |
| Production mutation policy | `deploy-production=disabled`; `release-production=disabled` |
| `git diff --check f2e45efe..4305265` | Passed |

No source, provider, repository setting, deployment, release, tag, DNS, schema, data, or secret mutation was performed. Production remains `HOLD`.

---

_Reviewed: 2026-09-06T16:17:58Z_
_Reviewer: independent Phase 02 adversarial recheck_
_Depth: deep_
