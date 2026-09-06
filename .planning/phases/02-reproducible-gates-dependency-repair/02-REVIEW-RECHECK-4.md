---
phase: 02-reproducible-gates-dependency-repair
reviewed: 2026-09-06T16:08:04Z
depth: deep
reviewed_head: f2e45efe3386692a6fdec6858dc6e00d47352e35
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
  warning: 1
  info: 0
  total: 1
status: issues_found
production_promotion: HOLD
---

# Phase 02: Fourth Independent Deep Recheck

**Reviewed:** 2026-09-06T16:08:04Z
**Exact committed source:** `f2e45efe3386692a6fdec6858dc6e00d47352e35`
**Depth:** deep
**Status:** issues found
**Production mutation disposition:** `HOLD`

## Summary

The parser-backed repair closes every previously reported false-green case. The exact-head suite passed 174 focused tests, including all WR-R01, WR-R2, and WR-R3 comment/literal, scope, qualifier, macro-token, TypeScript expression/regex, helper-failure, unsupported-source, and Markdown state regressions. A clean frozen install from a `git archive` of the exact SHA installed the pinned root `typescript@5.9.3`, left `pnpm-lock.yaml` unchanged, and ran the canonical contract validator successfully.

One new compile-backed Rust false negative remains. A valid qualified method in an `impl` whose leading generic bound contains a function return arrow is rejected because the handwritten generic scanner treats the arrow's `>` as the closing generic delimiter. This does not create a false-green mutation path, but it means the cumulative `repo://` resolver is not clean for supported Rust declarations. The fix ledger correctly remains `fixes_complete_pending_independent_recheck`; it must stay open until this warning is repaired and rechecked.

All 158 anchored registry-reference occurrences currently resolve, the seven unique anchors remain bound to their intended source or rendered headings, and the recomputed registry digest matches the release manifest and both fixtures. Both production mutation profiles remain disabled.

## Narrative Findings (AI reviewer)

## Warnings

### WR-R4-01: Function return arrows terminate leading Rust impl generics early

**Classification:** WARNING
**File:** `scripts/validate_contracts.py:324-351,354-390,393-415`
**Issue:** `_strip_leading_rust_generics` decrements angle depth for every `>` character. It therefore interprets the `>` in Rust's `->` return arrow as the end of the leading `impl<...>` generic list. `_rust_impl_body_opening` uses the same character-level angle accounting. The following valid source compiles, but `Owner::target` is absent from the resolver:

```rust
struct Owner<F>(F);

impl<F: Fn() -> bool> Owner<F> {
    fn target(&self) {}
}
```

Exact reproduction:

```text
$ rustc --edition 2021 --crate-type lib --emit metadata -o arrow-bound.rmeta arrow-bound.rs
warning: 2 warnings emitted
$ python3 -c '<load scripts/validate_contracts.py; call source_anchor_exists(path, source, "Owner::target", "probe")>'
False
$ python3 -c '<load scripts/validate_contracts.py; call _strip_leading_rust_generics("<F: Fn() -> bool> Owner<F> ")>'
bool> Owner<F>
```

The expected stripped header is `Owner<F>`, so `_rust_impl_target` should resolve `Owner`. Instead it resolves the incorrect prefix `bool` and rejects a real method. Function-trait bounds are ordinary Rust syntax, making this a material false negative for the advertised qualified Rust source-anchor support.

**Fix:** Replace character-level angle matching with a Rust-aware tokenizer/parser for `impl` items. If the bounded scanner is retained, both generic scanners must at minimum recognize `->` as a token whose `>` does not close generic depth, while preserving nested generic `>>` handling and existing fail-closed const-expression braces. Add a compile-backed regression such as `test_qualified_rust_anchor_accepts_function_trait_bound_in_impl_generics`, including an inherent impl and a trait impl with `Fn() -> bool` in the leading generic bounds.

## Prior Counterexample Recheck

| Boundary | Exact-head result |
|---|---|
| Comment-only, string/template-only, wrong-qualifier anchors | Pass; rejected |
| Rust `r`, `br`, `cr`, byte, C, character, lifetime, and label syntax | Pass |
| Rust macro input using `()`, `[]`, or `{}` | Pass; unqualified items and qualified impl methods rejected |
| Rust macro definitions and macro input inside a direct impl | Pass; rejected while the direct attributed `pub(crate)` method resolved |
| Rust mismatched/unclosed delimiters and unterminated nested comments/strings/raw strings | Pass; failed closed |
| Rust const-generic brace in an impl header | Pass; deliberately failed closed |
| Python unqualified class member and function-local class | Pass; rejected; fully qualified module-level member resolved |
| TypeScript/JavaScript named function and class expressions | Pass; rejected across parenthesized, array, assignment, argument, conditional, and arrow contexts |
| TypeScript/JavaScript regex, division, `/=`, `/=/`, control-header and post-block slash context | Pass through TypeScript compiler AST |
| TypeScript helper unavailable, timeout/runtime failure, missing dependency, parse diagnostic, malformed JSON, invalid shape/anchors | Pass; Python boundary failed closed |
| `.tsx`, `.jsx`, and other unsupported source suffixes | Pass; failed closed |
| Markdown backtick/tilde fences and closed/unclosed HTML comments | Pass with ordered fence-before-comment state |

The only failed supported-language control was the leading Rust function-trait generic shown in WR-R4-01.

## TypeScript Dependency and Helper Evidence

The helper loads the compiler through `require("typescript")` and rejects compiler parse diagnostics before collecting only direct source/module statements and direct class/interface members (`scripts/resolve_typescript_anchors.cjs:11-16,35-75,77-154`). The Python boundary enforces a 15-second timeout, nonzero-exit rejection, JSON parsing, an exact response shape, valid symbol grammar, and unique anchors (`scripts/validate_contracts.py:419-462`). Named expressions and regex/division ambiguity were exercised through the actual helper.

The dependency is exact at `package.json:15-18`, the root importer resolves the same version at `pnpm-lock.yaml:23-32`, and the lock records its integrity at `pnpm-lock.yaml:3629-3632`.

Fresh exact-SHA install receipt:

```text
$ git archive f2e45efe3386692a6fdec6858dc6e00d47352e35 | tar -x -C <empty-temp-directory>
$ pnpm install --offline --ignore-scripts --frozen-lockfile
Scope: all 8 workspace projects
Packages: +715
Done in 6.5s using pnpm v10.33.0
$ node -e "const ts=require('typescript'); process.stdout.write(ts.version)"
5.9.3
$ python3 scripts/validate_contracts.py
contract authority v1 valid: schemas=6 fixtures=5 registries=1 engines=19
```

The pre/post lock SHA-256 remained `ba930dc5814352f0d47fbba825b10ecbef57d5728a5f5cf1a15f182de257f67d`.

## Registry and Release Authority Evidence

The registry contains 177 `repo://` references, of which 158 carry anchors and reduce to seven unique targets:

1. `repo://crates/noesis-api/src/lib.rs#register_database_conditional_engines`
2. `repo://crates/noesis-bridge/src/lib.rs#BridgeManager::new`
3. `repo://crates/noesis-orchestrator/src/lib.rs#SUPPORTED_ENGINE_IDS`
4. `repo://crates/noesis-orchestrator/src/lib.rs#WorkflowOrchestrator::register_native_runtime_engines`
5. `repo://docs/plans/selemene-engine/CAPABILITY-LEDGER.md#engine-runtime-ledger`
6. `repo://docs/plans/selemene-engine/CAPABILITY-LEDGER.md#fresh-probe-receipt`
7. `repo://ts-engines/src/server/registry.ts#registerTypeScriptRuntimeEngines`

All 158 resolved through the canonical validator. The registry digest is:

```text
sha256:c801fb49332f5c78cdf78417b6efd343cfcf30dc26e6ce7259cfc6157b707896
```

It matches `contracts/release/v1/manifest.json:6-10`, `contracts/release/v1/fixtures/eligible-source-redeploy.json:24-31`, and `contracts/release/v1/fixtures/current-production-incomplete.json:24-31`. Release fixture validation accepted both receipts and all nine mutation cases. `deploy-production` and `release-production` remain disabled at `contracts/release/v1/manifest.json:91-123`; this review does not authorize either profile.

## Verification Receipts

| Command | Exact result |
|---|---|
| `git rev-parse HEAD` | `f2e45efe3386692a6fdec6858dc6e00d47352e35` |
| `python3 -m pytest tests/scripts/test_validate_contracts.py tests/scripts/test_validate_release_receipt.py -q` | 174 passed in 80.91s: 134 contract and 40 release tests |
| `python3 scripts/validate_contracts.py` | Passed: `schemas=6 fixtures=5 registries=1 engines=19` |
| `python3 scripts/validate_release_receipt.py --validate-fixtures` | Passed: `receipts=2 mutation_cases=9` |
| `python3 -m py_compile scripts/validate_contracts.py tests/scripts/test_validate_contracts.py` | Passed |
| `node --check scripts/resolve_typescript_anchors.cjs` | Passed |
| Clean archive `pnpm install --offline --ignore-scripts --frozen-lockfile` | Passed; exact `typescript@5.9.3`; lock unchanged |
| Registry traversal | 177 total references; 158 anchored occurrences; seven unique anchored targets; canonical validation passed |
| Registry SHA-256 linkage | Registry, manifest, and both digest-bearing fixtures matched |
| `git diff --check 5e3f2df..f2e45efe` | Passed |
| Compile-backed `Fn() -> bool` impl probe | `rustc` exit 0; resolver returned `False` for `Owner::target` |

No source, provider, repository setting, deployment, release, tag, DNS, schema, data, or secret mutation was performed. Production remains `HOLD`.

---

_Reviewed: 2026-09-06T16:08:04Z_
_Reviewer: independent Phase 02 adversarial recheck_
_Depth: deep_
