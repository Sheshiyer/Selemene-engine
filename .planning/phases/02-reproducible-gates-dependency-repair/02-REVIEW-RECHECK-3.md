---
phase: 02-reproducible-gates-dependency-repair
reviewed: 2026-09-06T15:19:23Z
depth: deep
diff_base: 009ab1fdacec87eb3311e489a85d47cdc8b40a93
reviewed_head: 5e3f2df52c3ab38775f1dc8e5a3863750bbb85c7
files_reviewed: 7
files_reviewed_list:
  - .planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md
  - contracts/release/v1/fixtures/current-production-incomplete.json
  - contracts/release/v1/fixtures/eligible-source-redeploy.json
  - contracts/release/v1/manifest.json
  - contracts/v1/registries/engines.json
  - scripts/validate_contracts.py
  - tests/scripts/test_validate_contracts.py
findings:
  critical: 0
  warning: 4
  info: 0
  total: 4
status: issues_found
wr_r2_verdict: not_fully_fixed
production_promotion: HOLD
---

# Phase 02: Final Independent Deep Recheck

**Reviewed:** 2026-09-06T15:19:23Z
**Depth:** deep
**Exact range:** `009ab1fdacec87eb3311e489a85d47cdc8b40a93..5e3f2df52c3ab38775f1dc8e5a3863750bbb85c7`
**Status:** issues found
**Production mutation disposition:** `HOLD`

## Summary

The requested WR-R2 regression matrix passes at the exact committed head: all 43 direct cases behaved as specified, the 112 focused contract and release tests passed, and both canonical validators passed. All 158 registry-reference occurrences resolve to seven unique anchors. Removing each canonical declaration or rendered heading makes its corresponding reference reject. The registry digest is also coherent across the registry, manifest, and both digest-bearing fixtures.

The cumulative resolver is still not a safe general declaration resolver. Three independently reproduced lexical edge classes produce false-green or false-red evidence, and the fix ledger therefore overstates the result as `all_fixed`. None of these residual cases currently invalidates the seven canonical anchors, and no production mutation profile was enabled by this commit.

## Narrative Findings (AI reviewer)

## Warnings

### WR-R3-01: Brace-only scope tracking accepts declarations inside expressions and Rust macro token trees

**Classification:** WARNING
**File:** `scripts/validate_contracts.py:284-303,347-409,451-489`
**Issue:** `_file_scope_source` and `_brace_depths` model scope with `{` and `}` only. They do not account for parentheses, brackets, expression context, or Rust macro token trees. The declaration regular expressions consequently treat tokens at brace depth zero as real file-scope declarations even when the language parser would not.

The following valid Rust source was accepted for `#phantom`, although `phantom` is only discarded macro input and no function item exists:

```rust
macro_rules! discard { ($($t:tt)*) => {}; }
discard!(
fn phantom() {}
);
```

The same construction accepted `#Owner::method` for an `impl Owner` placed inside the discarded token tree. Valid TypeScript and JavaScript named function expressions placed on a new line after `(` were accepted as unqualified file-scope declarations, and named class expressions similarly satisfied qualified `Owner::method` anchors:

```ts
const value = (
function phantom() {}
);
```

This is residual false-green repository evidence for WR-R2-01.

**Fix:** Resolve declarations with language-aware parsers and require the relevant AST node to be a module/file item. A Rust resolver must distinguish expanded items from inert macro token input; a TypeScript/JavaScript resolver must distinguish declarations from named expressions. If a parser cannot establish that boundary, fail closed. Add compile/parser-backed regressions for parenthesized function and class expressions plus macro token-tree decoys for both unqualified and qualified anchors.

### WR-R3-02: JavaScript and TypeScript regex literals corrupt brace depth

**Classification:** WARNING
**File:** `scripts/validate_contracts.py:194-250,284-303,451-489`
**Issue:** `_strip_c_like_comments_and_literals` removes quoted strings and templates but does not remove JavaScript or TypeScript regex literals. Braces inside a regex are then consumed as structural braces by both scope helpers. This causes both kinds of incorrect evidence:

- `const pattern = /\{/; export function real() {}` rejects the real file-scope `#real` anchor because the regex increments the retained brace depth.
- Inside `function outer()`, `const pattern = /\}/; function phantom() {}` accepts nested `#phantom` because the regex prematurely reduces brace depth to zero.

Both JavaScript examples pass `node --check`; the equivalent TypeScript cases follow the same sanitizer and scope path.

**Fix:** Prefer a JavaScript/TypeScript parser and derive declaration scope from its AST. A bounded lexical fallback would need a correct regex-literal tokenizer, including division-versus-regex context, escapes, character classes, and flags, before any delimiter accounting. Add regressions for opening and closing braces in regex literals at file and nested scope.

### WR-R3-03: HTML-comment removal before fence parsing hides rendered Markdown headings

**Classification:** WARNING
**File:** `scripts/validate_contracts.py:135-171`
**Issue:** `markdown_anchors` globally blanks every `<!-- ... -->` region before it recognizes fenced code blocks. An unclosed HTML-comment token that is literal text inside a fenced block therefore blanks the remainder of the document, including headings rendered after the fence:

````markdown
```text
<!-- literal
```
# Real Heading
````

The validator rejects `#real-heading`, even though the heading is outside the fence and is rendered. The same ordering defect applies to tilde fences. This is a false-negative repository-evidence result left by WR-R2-04.

**Fix:** Parse Markdown in one block-aware pass, so HTML comment state begins only outside a fence and fence state is resolved before interpreting comment markers. A CommonMark/GFM parser is preferable. Add backtick- and tilde-fence regressions where literal closed and unclosed comment markers precede a rendered heading.

### WR-R3-04: The fix ledger asserts all findings are closed despite reproducible residual defects

**Classification:** WARNING
**File:** `.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md:12-15,66-112,143-145`
**Issue:** The ledger records all five second-recheck findings as fixed and sets `status: all_fixed`. It specifically says brace-scoped decoys can no longer satisfy anchors and that Markdown comment/fence handling is fixed. WR-R3-01 and WR-R3-03 provide counterexamples, while WR-R3-02 exposes an additional scope corruption in the same resolver. The closing claim that no Critical or Warning finding remains is therefore not supported by the exact committed implementation.

**Fix:** Change the ledger to `issues_found`, record this recheck and its exact head, and leave WR-R2-01/WR-R2-04 open until the residual cases have red/green tests and an independent committed-snapshot recheck. Preserve the production `HOLD` and disabled mutation dispositions.

## Requested WR-R2 Matrix

| Area | Exact-head result | Evidence |
|---|---:|---|
| Comment-only and string/template/raw/character literal tokens | Pass | Rust, TypeScript, JavaScript, and Python decoys rejected; `r`, `br`, and `cr` raw strings rejected |
| Wrong qualifiers and nested/local braces | Pass | Wrong Rust/Python/TypeScript/JavaScript owners rejected; requested nested-body cases rejected |
| Python class boundary | Pass | Unqualified class member rejected; fully qualified module-level class member accepted; function-local class rejected |
| Rust lifetime, label, generic, and trait syntax | Pass | Compile-backed lifetime/generic inherent and trait impl anchors accepted; byte/C string decoys rejected |
| TSX/JSX and unsupported suffixes | Pass, fail closed | `.tsx`, `.jsx`, `.go`, `.sh`, and `.json` references rejected as unsupported |
| Markdown fenced/comment-only headings | Pass for requested cases | Headings wholly inside backtick/tilde fences or HTML comments rejected; ordinary rendered heading accepted |
| Canonical registry authority | Pass | 158/158 occurrences resolved; seven unique anchors; declaration-removal probes rejected every canonical anchor |
| Registry digest and release fixtures | Pass | Recomputed digest matched manifest and both fixtures; release fixture validator passed |

The requested cases are useful regression coverage, but their passing result does not cover the expression, macro-token, regex-literal, or mixed fence/comment states described above.

## Canonical Authority Evidence

The 158 registry references reduce to these seven unique anchors, all of which resolved at `5e3f2df`:

1. `repo://crates/noesis-api/src/lib.rs#register_database_conditional_engines`
2. `repo://crates/noesis-bridge/src/lib.rs#BridgeManager::new`
3. `repo://crates/noesis-orchestrator/src/lib.rs#SUPPORTED_ENGINE_IDS`
4. `repo://crates/noesis-orchestrator/src/lib.rs#WorkflowOrchestrator::register_native_runtime_engines`
5. `repo://docs/plans/selemene-engine/CAPABILITY-LEDGER.md#engine-runtime-ledger`
6. `repo://docs/plans/selemene-engine/CAPABILITY-LEDGER.md#fresh-probe-receipt`
7. `repo://ts-engines/src/server/registry.ts#registerTypeScriptRuntimeEngines`

For each of the five source anchors, renaming only its intended declaration in a temporary exact-head copy caused validation to reject even when calls, comments, or similarly named members remained. Renaming each of the two rendered headings likewise caused its Markdown reference to reject.

The recomputed registry digest was:

```text
sha256:c801fb49332f5c78cdf78417b6efd343cfcf30dc26e6ce7259cfc6157b707896
```

It matches `contracts/release/v1/manifest.json:6-10`, `contracts/release/v1/fixtures/eligible-source-redeploy.json:24-31`, and `contracts/release/v1/fixtures/current-production-incomplete.json:24-31`.

## Verification Receipts

All committed-snapshot checks below used `git show 5e3f2df:<path>` or a clean temporary copy of that exact tree, so concurrent uncommitted edits were excluded.

| Check | Result |
|---|---|
| `python3 -m pytest tests/scripts/test_validate_contracts.py tests/scripts/test_validate_release_receipt.py -q` | 112 passed in 52.56s (72 contract, 40 release) |
| Direct requested adversarial matrix | 43/43 expected outcomes passed |
| Additional expression/macro/regex/fence probes | Every listed counterexample independently reproduced an unexpected outcome across the three parser findings |
| `python3 scripts/validate_contracts.py` | Passed: `schemas=6 fixtures=5 registries=1 engines=19` |
| `python3 scripts/validate_release_receipt.py --validate-fixtures` | Passed: `receipts=2 mutation_cases=9` |
| Python bytecode compilation | Passed |
| `git diff --check 009ab1f..5e3f2df` | Passed |
| Registry reference traversal | 158/158 occurrences passed; seven unique anchors |
| Canonical declaration-removal probes | Seven of seven rejected after intended declaration/heading removal |
| Registry SHA-256 linkage | Manifest and two fixtures matched recomputed registry digest |

No external provider command, deployment, push, release, or other mutation was performed. Production mutation profiles remain disabled and fail closed; this recheck does not authorize promotion.

---

_Reviewed: 2026-09-06T15:19:23Z_
_Reviewer: independent Phase 02 adversarial recheck_
_Depth: deep_
