# Embedding / Vectorize / Dyad Sync Architecture Memo

Date: 2026-07-09
Scope: Review existing NVIDIA embedding models, Cloudflare Vectorize usage in Selemene, and how embeddings could (or should not) connect to the dyad / sync flow.

## 1. NVIDIA Embedding Models Relevant to Selemene

NVIDIA NIM exposes several text-embedding families. The relevant options for a project like Selemene (English-only report patterns, small passages, commercially usable) are:

| Model | Family | Dim | Max tokens | License / Cost note | Fit for Selemene |
|-------|--------|-----|------------|---------------------|------------------|
| `nvidia/embed-qa-4` (NV-Embed-QA-4) | E5-Large fine-tuned | 1024 | 512 | Evaluation / commercial (NeMo license) | Good general-purpose QA retrieval |
| `nvidia/nv-embedqa-e5-v5` | E5-Large-Unsupervised fine-tuned | 1024 | 512 | NVIDIA AI Foundation + MIT, commercially ready | Same family, slightly older version |
| `nvidia/nv-embed-v1` | NVIDIA proprietary | varies | varies | Commercial | General embedding, less QA-tuned |
| `nvidia/llama-3.2-nv-embedqa-1b-v1/v2` | Llama-based small | 2048? | 512 | Commercial | Smaller/faster, useful at edge |
| `nvidia/llama-3.2-nemoretriever-300m-embed-v1/v2` | Nemoretriever | varies | varies | Commercial | Optimized for retrieval ranking |
| `nvidia/nv-rerankqa-mistral-4b-v3` | Reranker | N/A | N/A | Commercial | Could rerank top-k patterns |

Source: docs.api.nvidia.com NIM reference pages for retrieval models (fetched 2026-07-09).

### Recommendation

- **Do not introduce an NVIDIA embedding dependency for the L0 chart-fidelity gate.** Chart fidelity is a deterministic fact-matching problem (token inclusion against engine outputs). Adding a semantic embedding model would convert a hard verification into a soft similarity score, weakening the source-of-truth boundary.
- **Consider NVIDIA embeddings only when the project wants to upgrade pattern retrieval beyond the current Cloudflare Workers AI `@cf/baai/bge-small-en-v1.5` (384-dim).** The current BGE-small is adequate for anonymized report-pattern retrieval. If the project later needs higher-quality cross-report similarity or reranking, `nvidia/nv-embedqa-e5-v5` or `nvidia/embed-qa-4` are the natural candidates because they are explicitly QA/retrieval-tuned and commercially licensed.
- **NVIDIA NIM is a paid, externally hosted inference surface.** Routing birth-adjacent text to NVIDIA would require the same PII scrubbing already enforced by `containsPrivateBirthData()` in `cloudflare-vectorize.ts`. Treat it as a third-party provider, not a first-class deterministic component.

## 2. Cloudflare Vectorize — Current State

The existing implementation lives in `packages/witness-pipeline/src/patterns/cloudflare-vectorize.ts` and is documented in `docs/plans/2026-07-04-cloudflare-vectorize-pattern-memory.md`.

### What it does today

- **Embedding model**: Cloudflare Workers AI `@cf/baai/bge-small-en-v1.5` (384 dimensions, free-ish within Workers AI limits).
- **Vector store**: Cloudflare Vectorize index bound as `REPORT_PATTERNS`.
- **Durable full-text store**: R2 (`PATTERNS_BUCKET`) preferred, D1 (`PATTERNS_D1`) fallback, metadata-only fallback.
- **Privacy gate**: `containsPrivateBirthData()` regex-scans dates, times, lat/lng, birth language before upsert.
- **Upsert**: `CloudflareVectorizePatternStore.upsertPatterns(patterns)` embeds safe patterns, writes canonical record to R2/D1, writes vector + safe metadata to Vectorize.
- **Retrieve**: `CloudflareVectorizePatternRetriever.retrieveSimilar(query, filters, limit)` embeds the query, queries Vectorize with optional metadata filters, hydrates text from R2/D1.
- **Orchestrator integration**: `IntegratedReadingOrchestrator.run()` accepts an optional `retriever`; retrieved patterns are rendered via `renderRetrievedPatternsForPrompt()` with a non-deterministic warning, then added to the per-pass prompt and included in output.

### Where it fits in the data flow

```
Report generation (L0-L5)
  → extractReportPatterns()  ──safe/anonymized──┐
  → CloudflareVectorizePatternStore.upsertPatterns()
       ├─ AI.run('@cf/baai/bge-small-en-v1.5') → embedding
       ├─ R2/D1 → canonical text + provenance
       └─ Vectorize → vector + safe metadata

Future report generation (same mode / report level / systems filter)
  → CloudflareVectorizePatternRetriever.retrieveSimilar(query, filters)
       ├─ AI.run(...) → query embedding
       ├─ Vectorize.query(...) → top-k IDs + metadata
       └─ R2/D1.get(...) → hydrate full text
  → renderRetrievedPatternsForPrompt() → injected as analogy/context
  → LLM pass output
```

## 3. Dyad / Sync Flow — Where Embeddings Do and Do Not Belong

### Dyad flow today

- The dyad UI is `apps/noesis-web/src/lib/integrated/DyadChamber.tsx` (and related pages: auth, engines, get-reading).
- Dyad mode docs use topology `dyad-arc` and composite subjects (`subject_count: 2`).
- Engine routing in `packages/witness-pipeline/src/selemene/types.ts` marks some engines as `dyad-synthesis`.
- The actual dyad calculation/reading path is the same orchestrator (`IntegratedReadingOrchestrator`) with `engineResultsBySubject` containing two subject arrays.
- There is **no current use of embeddings in the dyad compatibility calculation**. Compatibility is deterministic: compare the two engine result sets.

### Proposed use-cases for embeddings / Vectorize in dyad/sync

1. **Cross-report pattern retrieval for dyad synthesis** (future)
   - When generating a dyad reading, query Vectorize for patterns from previous solo reports of the two subjects (or anonymized similar dyads) to help the LLM frame convergence/tension language.
   - Must use the same PII scrubbing and non-deterministic warning.
   - Use: **retrieval only**, not compatibility scoring.

2. **Solo-to-dyad sync for shared themes** (future)
   - Store patterns from a solo report; when the same person later creates a dyad report, retrieve their solo patterns to maintain thematic continuity.
   - Implementation: pass the same `retriever` with a filter on `mode` + report subject provenance.

3. **Compatibility similarity score** (recommend: do not do this)
   - Do not use cosine similarity between two birth-chart embeddings as a "compatibility score." That would replace deterministic engine comparison with a black-box similarity and conflict with the project's principle of grounding in deterministic data.
   - If a numeric compatibility metric is ever needed, derive it from deterministic engine outputs (e.g., gate/channel overlays, planetary aspects) with transparent rules.

4. **Dyad pattern memory** (future)
   - After a dyad report passes guardrails, extract dyad-specific patterns and store them in Vectorize with metadata `subject_count: 2` and `mode: composite-dyad`.
   - This enables retrieval for future triad/pentagon or longitudinal dyad readings.

### Trust boundaries

- **No birth data in Vectorize**: `containsPrivateBirthData()` must gate every upsert. Vectorize metadata contains only mode, report_level, kind, version, systems.
- **Full text in R2/D1**: canonical records live in project-owned durable storage, not in Vectorize metadata.
- **Retrieved patterns are non-deterministic**: `renderRetrievedPatternsForPrompt()` already enforces this warning.
- **Embeddings are content-addressable, not identity-addressable**: never use person name, birth date, or location as a Vectorize query; use thematic query text and provenance filters.

## 4. Integration Points Checklist

| Integration | Status | Notes |
|-------------|--------|-------|
| Cloudflare Vectorize pattern store/retrieve | Implemented in library, not yet bound in a deployed Worker | Needs `REPORT_PATTERNS`, `AI`, optional `PATTERNS_BUCKET`/`PATTERNS_D1` |
| Worker surface for pattern memory | Designed in plan, not deployed | `workers/llm-proxy/` currently only handles LLM secrets, not pattern memory |
| Dyad embedding consumer | Not implemented | Could be added later as a retrieval input to dyad mode docs |
| NVIDIA embedding backend | Not used | Optional future upgrade; not required |
| PII scrubbing | Implemented in `cloudflare-vectorize.ts` | Must remain in place for any new embedding backend |

## 5. Recommendations

1. **Do not add embeddings to the L0 fidelity gate.** Keep chart fidelity as deterministic token inclusion.
2. **Keep the current Cloudflare Vectorize/BGE-small implementation as the default pattern-memory backend.** It is cost-effective, co-located with Workers, and privacy-gated.
3. **If retrieval quality becomes a bottleneck, evaluate NVIDIA `nvidia/nv-embedqa-e5-v5` or `nvidia/embed-qa-4`** as an alternative embedding generator, but route through the same PII filter and durable-store architecture.
4. **For dyad/sync, add embeddings only as a retrieval-augmentation layer**, never as a compatibility or sync truth source. Truth remains deterministic engine comparison.
5. **Deploy the pattern-memory Worker surface before using Vectorize in production.** The library code is ready; it needs a Worker with the bindings and a smoke test against a real Vectorize index.
