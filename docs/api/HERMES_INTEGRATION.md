# Hermes Integration (Noesis) — Updated 2026-07

**Primary reference:** `docs/api/AGENT_FLOW.md`

This file now redirects to the current rich contract for narrative witness readings.

---

## What Changed

- Old model: flat engine/workflow calls with only `birth_data`
- New model: rich `ReportGenerationRequest` / `OrchestratorOutput` with:
  - `language`
  - `relationship_context` + `relationship_header`
  - `subjectRoles`
  - L0-L5 + explicit modes (including unmarried/married-partners)
  - Folio B-surface header prepended to assembled
  - NotebookLM readiness (`generateSlidesPrompt`)

---

## For Hermes Users

1. Use the same 22 tools for direct engine + workflow calls (they still work).
2. For full narrative witness reports (the main new capability), follow the 8-step flow in `AGENT_FLOW.md` and the shapes in `selemene-cheatsheet.md`.
3. After getting an `OrchestratorOutput`, you can feed it to `selemene-notebooklm` for slides prompts.

---

## Still Valid

- Base URL + `X-API-Key`
- `EngineInput` for raw engine/workflow calls
- Hermes bridge code in `bridges/hermes/`
- Tool auto-detection (OpenAI vs XML)

---

## Recommended Reading Order for Agents

1. `docs/api/AGENT_FLOW.md`
2. `selemene-core` skill
3. `selemene-cheatsheet.md` (minimal)
4. `selemene-report` (guided Q&A)
5. `selemene-notebooklm` (when you want NotebookLM artifacts)

---

Everything else in this file about Hermes setup, models, and bridge code remains correct.
