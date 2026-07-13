# Selemene / Noesis — Agent Flow (2026-07)

This document is the **current canonical reference** for coding agents (Claude, OpenCode, Codex, Hermes, OpenClaw, etc.) that need to generate Selemene reports and artifacts.

It reflects the rich contract after the July 2026 updates: language, relationship_context, Folio headers, witness-pipeline `OrchestratorOutput`, and the NotebookLM first artifact.

---

## Two Surfaces

| Surface              | Path                                      | When to use |
|----------------------|-------------------------------------------|-------------|
| Deterministic (Rust) | `@selemene/bridge` or direct `POST /api/v1/workflows/{id}/execute` | Birth, compatibility, transit reports |
| Narrative Witness    | `packages/witness-pipeline` → `IntegratedReadingOrchestrator` (live via Rust `/api/v1/assets/generate`) | Rich multi-subject readings, relationship-aware, language-aware, L0-L5, synastry |

Most agent work now goes through the **narrative witness** surface.

---

## 3 Key Shapes (copy these)

### 1. ReportGenerationRequest (recommended for most agents)
```json
{
  "report_level": "L2",
  "report_mode": "synastry",
  "subjects": [
    {
      "role": "mother",
      "name": "Aarav",
      "birth_date": "1970-01-01",
      "birth_time": "10:30",
      "birth_time_confidence": "exact",
      "birth_location_query": "Bengaluru, India",
      "normalized_location": {
        "display_name": "Bengaluru, Karnataka, India",
        "latitude": 12.9716,
        "longitude": 77.5946,
        "timezone": "Asia/Kolkata",
        "provider": "manual",
        "confidence": "manual"
      }
    }
  ],
  "relationship_context": {
    "type": "family",
    "mapping_goal": "understand lineage transmission patterns without outcome prediction",
    "sensitivity_level": "high"
  },
  "language": "en",
  "output": {
    "format": "markdown",
    "include_rubric": true,
    "include_pattern_extraction": true
  }
}
```

### 2. OrchestratorInput (internal / direct calls)
```json
{
  "subjectNames": ["Aarav", "Vikram"],
  "subjectRoles": [
    { "role": "mother", "name": "Aarav" },
    { "role": "son", "name": "Vikram" }
  ],
  "relationshipContext": { "type": "family", "mapping_goal": "...", "sensitivity_level": "high" },
  "language": "en",
  "consciousnessLevel": 2,
  "engineResultsBySubject": [ /* ... */ ]
}
```

### 3. OrchestratorOutput (what you receive back)
```json
{
  "mode": "mother-son-lineage",
  "subject_names": ["Aarav", "Vikram"],
  "register": "l1_l3",
  "relationship_header": "Mother-Son Lineage Mapping — non-predictive pattern witness",
  "passes": [{ "id": "...", "title": "...", "output": "...", "rubric": { ... } }],
  "assembled": "# Mother-Son... \n\n full reading",
  "patterns": [],
  "retrieved_patterns": []
}
```

**Important:** When `relationship_context` is present, `relationship_header` is prepended to `assembled`.

---

## 8-Step Guided Flow (for agents)

Use the `selemene-report` skill (or follow this sequence manually):

1. **Surface** — `witness` (narrative) or `deterministic`
2. **Subjects** — one block per person (role + birth + normalized_location)
3. **Relationship** — `relationship_context` or `null` for solo
4. **Language + Level** — `language` + `consciousness_level` (0-5)
5. **Mode + Level** — `report_level` + `report_mode` (see modes below)
6. **Assemble** — output under `### FINAL ASSEMBLED REQUEST`
7. **Run** — get `OrchestratorOutput`
8. **Post-process**:
   - Source pack → `createSourcePack`
   - NotebookLM slides → hand output to `selemene-notebooklm`

---

## Current Variables & Environment

| Variable                  | Purpose                                      | Default / Example                  |
|---------------------------|----------------------------------------------|------------------------------------|
| `NOESIS_API_KEY`          | Auth header `X-API-Key`                      | `nk_...` (required for remote)     |
| `SELEMENE_RUST_URL`       | Base for deterministic + witness calls       | `http://localhost:8080`            |
| `CF_DEV_BYPASS_TOKEN`     | Dev bypass header `x-noesis-dev-auth`        | Set only in local dev              |
| `language`                | First-class field on request                 | `"en"` (injected into prompts)     |
| `report_level`            | L0–L5                                        | `"L2"`                             |
| `relationship_context.type` | `family`, `business-partners`, etc.        | Required for non-solo              |

---

## Key Modes (2026-07)

- `birth-blueprint`
- `integrated-reading` / `integrated-reading-l4`
- `mother-son-lineage`
- `business-partners`
- `family-penta`
- `unmarried-partners`
- `married-partners`

Modes declare `relationship_types`, `roles`, `bridge_mandates`, etc.

---

## Post-Processing Artifacts

- **Folio header** — already inside `assembled` when relationship present
- **Source pack** — `createSourcePack(...)` from `packages/witness-pipeline/src/assets/factory.ts`
- **NotebookLM slides prompt** — `generateSlidesPrompt(output, { language, bridgeMandates })` (via `selemene-notebooklm` skill)

---

## Non-Prescriptive Rules (enforce in every prompt/output)

- "Facts only. No prediction. No diagnosis."
- Use `relationship_header` verbatim
- Respect `sensitivity_level` and relationship type in guardrails
- Never promise outcomes

---

## Agent Skills (recommended)

- `selemene-core` — current contract + taxonomy
- `selemene-report` — guided Q&A flow (copy-paste INPUT BOXes)
- `selemene-notebooklm` — turn `OrchestratorOutput` into NotebookLM prompt
- `selemene-cheatsheet.md` — ultra-minimal 3-shapes + 8-steps

---

## Quick Verification (after changes)

```bash
# Health
curl -s https://selemene.tryambakam.space/health/live

# Engines
curl -s https://selemene.tryambakam.space/api/v1/engines -H "X-API-Key: $NOESIS_API_KEY"

# Rich witness call (example)
curl -s -X POST https://selemene.tryambakam.space/api/v1/assets/generate \
  -H "X-API-Key: $NOESIS_API_KEY" \
  -H "Content-Type: application/json" \
  -d @request.json
```

`request.json` should use the `ReportGenerationRequest` shape above with `relationship_context` and `language`.

---

This doc supersedes older fragments in `OPENCLAW_INTEGRATION.md`, `HERMES_INTEGRATION.md`, and `LLM_AGENT_GUIDE.md` for the narrative witness path.
