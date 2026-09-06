---
phase: 02-reproducible-gates-dependency-repair
reviewed: 2026-09-06T14:58:29Z
depth: deep
baseline: 2bcaf77f24fe08ac5ad959560b3dba421b77285d
head: 009ab1fdacec87eb3311e489a85d47cdc8b40a93
files_reviewed: 3
files_reviewed_list:
  - scripts/validate_contracts.py
  - tests/scripts/test_validate_contracts.py
  - .planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md
findings:
  critical: 0
  warning: 5
  info: 0
  total: 5
status: issues_found
wr_r01_verdict: not_fully_fixed
production_promotion: HOLD
---

# Phase 02: WR-R01 Fix Recheck

**Reviewed:** 2026-09-06T14:58:29Z
**Depth:** deep
**Diff:** `2bcaf77f24fe08ac5ad959560b3dba421b77285d..009ab1fdacec87eb3311e489a85d47cdc8b40a93`
**Status:** issues_found
**WR-R01 verdict:** not fully fixed
**Production promotion:** HOLD

## Summary

Commit `009ab1f` closes the originally demonstrated ordinary comment/string bypasses, rejects wrong explicit qualifiers, rejects unsupported source suffixes, and preserves all current registry references. It does not yet provide a reliable declaration resolver across every suffix it claims to support. Handwritten regular expressions remain scope-insensitive, the Rust sanitizer mishandles valid lifetime and raw C-string syntax, TSX text is scanned as code, and Markdown headings inside non-rendered regions count as real anchors.

The committed focused suite is green (**45 passed**) and the canonical validator is green (`schemas=6 fixtures=5 registries=1 engines=19`). Those tests do not cover the residual cases below. A separate adversarial matrix exercised comments, conventional strings, templates, Rust raw/byte/character literals, raw C strings, wrong Rust/Python/TypeScript qualifiers, valid generic Rust impls, nested/local declarations, TSX text, unsupported suffixes, and Markdown non-heading regions. It reproduced eleven mismatches across four parser behaviors.

All **158** anchored registry-reference occurrences (**7 unique**) validate at this exact head: four Rust anchors, one TypeScript anchor, and two Markdown anchors. Unsupported `.go`, `.sh`, and `.json` source anchors reject. These current positives do not remove the false-green paths available to future or stale evidence rows.

## Narrative Findings (AI reviewer)

## Warnings

### WR-R2-01: Nested and class-scoped decoys satisfy broader evidence anchors

**Classification:** WARNING
**File:** `/Volumes/madara/2026/Projects/thoughtseed/.superset-worktrees/b7b13f10-dbc6-4dc5-b24a-4b03607ae607/codex/selemene-phase-recovery-20260905/scripts/validate_contracts.py:245-253,286-342,345-380`

**Issue:** Rust and TypeScript unqualified checks search every source line without tracking module scope. Their qualified checks likewise scan every `impl`, class, interface, or namespace regardless of whether it is nested inside a function. Python avoids function-body recursion, but `add_name` always registers both the qualified and unqualified form of class members. The following syntactically valid decoys were therefore accepted: a Rust local function as `#phantom`, a Rust function-local `impl Local` as `#Local::phantom`, a TypeScript nested function as `#phantom`, a TypeScript function-local class as `#Local::phantom`, and a Python class method as unqualified `#phantom`. Removing an intended module/runtime symbol while leaving any such decoy can keep its evidence row green.

**Fix:** Parse declarations with a language AST or tree-sitter grammar and retain their lexical scope. Require unqualified anchors to identify module-level declarations. Require methods and associated functions to use an exact qualified owner; update the current Rust method reference to `WorkflowOrchestrator::register_native_runtime_engines` before enforcing that rule. In the Python fallback, add an unqualified name only when `qualifiers` is empty. Add negative tests for nested functions, local `impl` blocks/classes, and unqualified class members in every supported language.

### WR-R2-02: Rust literal stripping false-greens raw C strings and rejects lifetime impls

**Classification:** WARNING
**File:** `/Volumes/madara/2026/Projects/thoughtseed/.superset-worktrees/b7b13f10-dbc6-4dc5-b24a-4b03607ae607/codex/selemene-phase-recovery-20260905/scripts/validate_contracts.py:185-207,256-306,391-396`

**Issue:** The raw-literal matcher recognizes `r` and `br` prefixes but omits Rust's valid `cr` raw C-string prefix. An embedded quote then terminates the generic quote scanner early, leaving later literal text to be matched as code. A valid, successfully compiled `cr#"embedded " quote\nfn phantom() {}\n"#` literal was accepted as declaration `#phantom`. Separately, after checking valid character literals, the generic branch treats every remaining apostrophe as a quoted literal. This consumes Rust lifetimes and causes valid, successfully compiled `impl<'a> Owner<'a>` and `impl<'a> Trait for Owner<'a>` blocks to reject `#Owner::method`. The ledger's claim that raw and character/lifetime-adjacent syntax is safely stripped is therefore broader than the implementation.

**Fix:** Prefer a Rust parser. For the bounded scanner, recognize `cr` alongside `r` and `br`, handle current Rust literal prefixes explicitly, and never pass an unmatched Rust apostrophe to the generic quoted-string path. Preserve lifetime and label tokens after the valid character-literal check. Add compile-backed tests for embedded-quote raw C strings, byte/raw strings, characters, lifetimes, labels, generic impls, and trait impls.

### WR-R2-03: TSX JSX text is interpreted as a TypeScript declaration

**Classification:** WARNING
**File:** `/Volumes/madara/2026/Projects/thoughtseed/.superset-worktrees/b7b13f10-dbc6-4dc5-b24a-4b03607ae607/codex/selemene-phase-recovery-20260905/scripts/validate_contracts.py:203-207,309-342,397-402`

**Issue:** `.tsx` and `.jsx` are advertised as supported, but the sanitizer removes only comments and quoted/template literals. JSX child text remains unchanged. A valid JSX body containing a line of text `function phantom()` was accepted for `repo://case.tsx#phantom`, even though no function exists. A stale TypeScript/JavaScript evidence anchor can therefore remain green when its spelling survives only in rendered text.

**Fix:** Use the TypeScript compiler/tree-sitter AST for `.ts`, `.tsx`, `.js`, and `.jsx`. Until JSX-aware parsing exists, fail closed for anchored `.tsx`/`.jsx` references rather than applying the plain-source regex. Add JSX text, attribute string, expression, nested component, and fragment regression cases.

### WR-R2-04: Markdown code fences and HTML comments satisfy heading anchors

**Classification:** WARNING
**File:** `/Volumes/madara/2026/Projects/thoughtseed/.superset-worktrees/b7b13f10-dbc6-4dc5-b24a-4b03607ae607/codex/selemene-phase-recovery-20260905/scripts/validate_contracts.py:449-459`

**Issue:** Markdown validation treats every line beginning with `#` as a rendered heading. `## phantom-anchor` inside a fenced code block and inside an HTML comment both satisfied `#phantom-anchor`; neither creates a GitHub heading target. The two current Markdown anchors resolve to real headings, but a deleted heading left in an example or comment would false-green the evidence ledger.

**Fix:** Parse Markdown block structure and collect actual heading tokens, or at minimum track fenced code blocks and HTML comments before normalizing headings. Add negative tests for backtick/tilde fences and multiline HTML comments plus positive ATX-heading tests.

### WR-R2-05: The fix ledger marks WR-R01 complete despite residual false-greens

**Classification:** WARNING
**File:** `/Volumes/madara/2026/Projects/thoughtseed/.superset-worktrees/b7b13f10-dbc6-4dc5-b24a-4b03607ae607/codex/selemene-phase-recovery-20260905/.planning/phases/02-reproducible-gates-dependency-repair/02-REVIEW-FIXES.md:9-12,41-59,90-92`

**Issue:** The ledger records `recheck_fixed: 1`, `status: all_fixed`, and states that all raw/template/character literal forms are stripped and no Warning remains. The reproducible cases above contradict that release evidence. Downstream readers can therefore treat the review gate as clean while the source-evidence validator still has false-green paths.

**Fix:** Keep the ledger at `issues_found` until WR-R2-01 through WR-R2-04 are fixed and independently rechecked. Replace the broad parser claims with the exact tested syntax set, record the new regression commands, and change `all_fixed` only after the adversarial cases pass.

## Requested Probe Results

| Probe family | Result |
|---|---|
| Ordinary line/block comments in Rust, TS/JS, Python | Rejected as intended |
| Ordinary Rust/TS/JS/Python strings and TS/JS templates | Rejected as intended |
| Rust `r`/`br` raw strings and character literals | Rejected as intended |
| Rust `cr` raw C string with embedded quote/newline | **False-green accepted** |
| Wrong explicit Rust, TypeScript, and Python qualifiers | Rejected as intended |
| Rust/TypeScript nested/local declarations | **False-green accepted** |
| Python nested function | Rejected as intended |
| Unqualified Python class method | **False-green accepted** |
| Valid lifetime-parameterized Rust impl and trait impl | **Incorrectly rejected** |
| TSX JSX child text | **False-green accepted** |
| Unsupported `.go`, `.sh`, `.json` anchors | Rejected fail-closed |
| Markdown fenced/comment-only heading | **False-green accepted** |
| All current registry anchors | 158/158 occurrences, 7/7 unique references accepted |

## Verification Performed

- `python3 -m pytest tests/scripts/test_validate_contracts.py -q` — **45 passed in 41.31s**.
- `python3 scripts/validate_contracts.py` — passed: `schemas=6 fixtures=5 registries=1 engines=19`.
- `python3 -m py_compile scripts/validate_contracts.py tests/scripts/test_validate_contracts.py` — passed.
- `git diff --check 2bcaf77..009ab1f` — passed.
- Registry enumeration and direct `validate_repo_reference` calls — **158/158 occurrences, 7/7 unique anchors accepted**.
- Adversarial temporary-file matrix — 46 cases; eleven expectation mismatches grouped in WR-R2-01 through WR-R2-04.
- `rustc --edition=2021 --crate-type=lib --emit=metadata` — confirmed the raw C-string, lifetime impl, trait impl, local function, and local `impl` probes are valid Rust syntax.
- No source file, provider, repository setting, branch, tag, release, deployment, database, or external state was modified.

## Final Counts

- **Critical / BLOCKER:** 0
- **Warning:** 5
- **Info:** 0
- **Total:** 5

---

_Reviewed: 2026-09-06T14:58:29Z_
_Reviewer: gsd-code-reviewer (independent WR-R01 recheck)_
_Depth: deep_
