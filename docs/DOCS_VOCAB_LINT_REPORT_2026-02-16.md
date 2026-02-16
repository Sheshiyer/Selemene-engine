# Docs Vocabulary Lint Report (2026-02-16)

## Scope

Audited Tier 1–3 docs listed in `docs/MESSAGING_CONSISTENCY_SCOPE_2026-02-16.md`, focusing on intro regions (first ~25 lines per file for user-facing framing checks).

## Rules

- Preferred terms: `authorship`, `reflection`, `inquiry`, `witness`, `synthesis`, `mirrors`
- Disallowed in user-facing intros: `journey`, `manifesting`, `vibration`, prescriptive authority framing

## Findings

- Disallowed term matches (`journey`, `manifesting`, `vibration`) in scoped intros: **0**
- Prescriptive-language matches in intro regions: found only in technical contexts (integration requirements, auth/runbook instructions)
- Tier 1/Tier 2/Tier 3 intro framing status: **PASS**

## Contextual exceptions (acceptable)

- `docs/api/LLM_AGENT_GUIDE.md` uses imperative wording for deterministic agent behavior.
- `docs/api/OPENCLAW_INTEGRATION.md` uses imperative wording for integration steps.
- `docs/deployment/RAILWAY.md` uses imperative wording for deployment constraints.

These are technical requirements, not product-positioning violations.

## Verdict

**PASS** — no disallowed messaging terms remain in user-facing intros within scoped docs.
