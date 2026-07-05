# Release v0.1.1 — witness-pipeline (Patch)

> **Tryambakam Noesis · Selemene Engine**
> Patch release for the report quality and learning infrastructure.

**Commit:** `7eb2773f`  
**Tag:** `v0.1.1-witness-pipeline`

---

## What's New

### Universal Per-Section Rubric Matrix
- Added `SectionRubric` + `auditSectionOutput()` with deterministic checks:
  - Word-count fit (80-125% pass)
  - Deterministic fact grounding (Vedic/HD/Gene Keys/panchanga terms)
  - Integrated layering (distinct systems counted)
  - Guardrail gates for wealth, love-marriage, health, family-lineage
  - Model requested/used + latency per pass
- Kundali-L0 specific thresholds (e.g. master-timeline: 8 facts / 3 layers min)
- Rubrics persisted to source-pack `manifest.quality.sections`

### Post-Report Pattern Extraction + Vectorize Safety
- `ExtractedPattern` type + `extractReportPatterns()`
- Filters: only sections passing guardrail + layering gates
- Anonymization of subject names
- `NoopPatternVectorStore` (default)
- `renderRetrievedPatternsForPrompt()` labels patterns as **non-deterministic context** (never facts)
- Patterns attached to `OrchestratorOutput.patterns[]`

### Report Intake Schema (Required Before Generation)
- `ReportGenerationRequest`, `ReportSubjectInput`
- `NormalizedLocation` (display_name, lat, lng, IANA timezone, provider, confidence)
- `isCompleteReportRequest()` — blocks generation until every subject has confirmed normalized location
- Manual coordinate fallback + question builder for gender + birthplace confirmation
- Gender vs `sex_for_external_chart_source` separation

### Report Levels (L0-L5) Separate from Consciousness Register
- `report_level` metadata in mode frontmatter
- Parser support + validation
- Annotated: `integrated-kundali-l0` (L0), `birth-blueprint` (L1), `integrated-reading` (L3)

### Source Pack Learning Provenance
- `pattern_learning: { extracted, upserted, skipped }` in `manifest.quality`
- Server-side `build_section_rubrics` for Rust API parity (kundali-l0 contract test asserts 12 sections)

### Version Bump
- `@noesis/witness-pipeline`: `0.1.0` → `0.1.1`

---

## Verification
- witness-pipeline: 36/36 green
- @noesis/sdk: 9/9 green
- noesis-api contract tests: 4/4 (including `assets_generate_supports_integrated_kundali_l0_mode` asserting sections rubric matrix)
- Typechecks: clean (witness-pipeline + sdk)

---

## Notes for Future Releases
- This is a **patch** addition. No breaking changes to existing `/witness/interpret` or public SDK surfaces.
- Retrieved patterns are **synthesis aids only** — current chart data always overrides.
- Next: real Vectorize binding + retrieval in Cloudflare worker (see `docs/plans/2026-07-04-cloudflare-vectorize-pattern-memory.md`).

---

**Full plan:** `docs/plans/2026-07-04-universal-report-rubric-vector-hardening.md`
