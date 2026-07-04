# Cloudflare Vectorize Pattern Memory Plan

## Purpose

Post-report pattern memory should store reusable, anonymized synthesis patterns for future report generation. Retrieved patterns may shape wording, analogical structure, and cross-system synthesis, but they must never override deterministic chart facts from the current report request.

This plan defines the Cloudflare Worker surface for embedding and retrieving report patterns without mixing Cloudflare bindings into the core witness-pipeline library.

## Worker Bindings

The Worker should expose two required bindings:

- `REPORT_PATTERNS`: Cloudflare Vectorize index for report-pattern embeddings.
- `AI`: Cloudflare Workers AI binding for embeddings.

Target `wrangler.jsonc` shape:

```jsonc
{
  "vectorize": [
    {
      "binding": "REPORT_PATTERNS",
      "index_name": "selemene-report-patterns"
    }
  ],
  "ai": {
    "binding": "AI"
  }
}
```

## Canonical Storage

Vectorize should store vectors and lookup metadata only. Full pattern records need canonical storage in one of the durable stores already aligned with the platform:

- R2 for JSON pattern documents and versioned pattern snapshots.
- D1 for indexed Worker-local metadata and lightweight retrieval joins.
- Postgres for canonical application records when the report platform needs relational ownership, approval, or audit workflows.

The canonical record should hold the full anonymized pattern text, kind, source section, source rubric score, systems, version, creation timestamp, approval status, and source provenance. Vectorize IDs should point back to these canonical records.

## Metadata Indexes

Every Vectorize upsert should include only non-private retrieval metadata:

- `mode`
- `report_level`
- `kind`
- `version`

Optional safe metadata may include `systems`, `source_section_id`, and `rubric_score_bucket` if those fields contain no private birth data.

## Privacy Boundary

No private birth data may be written to vector text or Vectorize metadata. This includes:

- Names.
- Birth dates.
- Birth times.
- Birthplace text.
- Latitude or longitude.
- Timezone.
- Relationship labels that identify a private person.
- Raw chart payloads that can reconstruct a private birth profile.

The only text eligible for embedding is an anonymized reusable synthesis pattern that has passed report guardrails and pattern eligibility checks.

## Write Flow

1. Receive extracted pattern candidates from the report pipeline.
2. Reject any candidate containing private birth data or failed guardrail provenance.
3. Persist the full anonymized pattern to R2, D1, or Postgres as canonical storage.
4. Generate an embedding through the `AI` binding.
5. Upsert the vector into `REPORT_PATTERNS` with safe metadata indexes.
6. Return counts for extracted, upserted, and skipped patterns.

## Retrieval Flow

1. Embed the current section intent or synthesis query with the `AI` binding.
2. Query `REPORT_PATTERNS` using metadata filters for `mode`, `report_level`, `kind`, and `version` where relevant.
3. Fetch full anonymized pattern records from canonical storage by Vectorize ID.
4. Provide retrieved patterns to the report generator as optional synthesis context.

Retrieved patterns are not deterministic facts. Current chart data and source-pack facts remain authoritative.
