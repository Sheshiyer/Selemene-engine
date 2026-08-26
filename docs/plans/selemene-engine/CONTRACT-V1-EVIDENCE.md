# Selemene Contract Authority v1 — Completion Evidence

Date: 2026-08-26
Branch: `codex/selemene-contract-convergence`
Baseline: `01160e5`
Candidate: `0df208d`

## Boundary

This slice establishes the repository-local, language-neutral contract authority before engine semantic work. It changes schemas, fixtures, validators, additive Rust/TypeScript types, OpenAPI parity, and API-boundary validation. It does not change engine algorithms, provider selection, runtime routing, generated-media execution, external repositories, databases, cloud resources, deployments, package publication, remotes, or live state.

## Canonical authority

- `contracts/v1/manifest.json` declares the immutable `v1` authority.
- Six Draft 2020-12 schemas cover request, result, error, consent, provenance, and engine capability.
- Five golden fixtures cover canonical request, legacy-compatible request, result, error, and capability payloads.
- `scripts/validate_contracts.py` fail-closes on manifest/schema/fixture drift, malformed schemas, duplicate or missing fixtures, path escape, unresolved local references, network references, invalid fragments, and sensitive error/provenance keys.
- Rust core, OpenAPI, the engine SDK, the general TypeScript SDK, and TypeScript engine types prove additive parity against the same authority.

## TDD and adversarial receipts

The implementation began red: nine validator tests failed because the authority and validator did not exist; root gate wiring, Rust contract imports, OpenAPI `contract_version`, and TypeScript exports also failed before their production paths were added.

Independent review then exposed five material false-green paths, each converted into executable protection:

1. Canonical result optionality did not match valid singular-witness and provenance-absent adapters.
2. Versioned API requests did not initially enforce version, required canonical parameters, or consciousness bounds.
3. Validator coverage did not initially reject zero/duplicate fixtures, path traversal, or unresolved fragments.
4. The maintained general TypeScript SDK was not initially composed into the root gate.
5. Actual JSON boundary behavior did not initially align seed bounds, lowercase precision, media shape, unknown-field rejection, or extractor-error envelopes.

The final candidate closes those paths with schema parity, typed boundary structs, deny-unknown deserialization, mapped JSON rejection, and route-level negative tests.

## Fresh verification

`pnpm run gate` exited zero at `0df208d`:

| Surface | Receipt |
|---|---:|
| Python gate and validator behaviors | 47 passed |
| Rust canonical contract tests | 6 passed |
| OpenAPI parity tests | 4 passed |
| API calculate boundary tests | 16 passed |
| Engine SDK tests | 35 passed plus typecheck |
| General TypeScript SDK tests | 11 passed plus typecheck |
| Verification package tests | 36 passed plus typecheck |
| TypeScript engine tests | 90 passed plus typecheck |

`git diff --check` also passed. The changed-path audit is confined to `contracts/v1`, contract validators/gates/tests, additive shared contract types, OpenAPI/API compatibility fields and request validation, SDK parity surfaces, and this evidence/plan documentation.

## Remaining boundaries

- Runtime provenance remains optional because existing calculation paths do not yet have a truthful unified provenance producer; this slice does not fabricate one.
- Engine semantic migration, runtime capability-registry adoption, and per-engine completion repair remain the next separately authorized layer.
- Historical ISC-217 remains open: GitHub Actions still use floating version tags rather than immutable commit SHAs.
- This branch is isolated and unmerged. No push, publication, deployment, or external-repository change occurred.

## Review verdict

Final independent read-only re-review returned **GO** at `0df208d` with no remaining P0/P1 findings. The reviewer independently reran `gate:scripts`, `gate:contracts`, and `git diff --check 01160e5..HEAD`; confirmed canonical seed, precision, media, consent, quality, unknown-field, and structured extraction-error behavior; and found no algorithm, provider, routing, authentication, persistence, infrastructure, or live-state scope leak.
