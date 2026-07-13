# OpenClaw + Hermes Integration (Updated 2026-07)

This document now points to the **current agent flow** after the relationship contract, language, Folio header, and NotebookLM updates.

**Primary reference:** `docs/api/AGENT_FLOW.md`

It contains:
- The 3 canonical shapes (ReportGenerationRequest, OrchestratorInput, OrchestratorOutput)
- The 8-step guided flow
- All required variables and env
- Post-processing (source pack, NotebookLM)
- Non-prescriptive guardrails

---

## Quick Map (old → new)

| Old concept in this file          | Current reality (2026-07) |
|-----------------------------------|---------------------------|
| Only engine + workflow calls      | Rich narrative witness via witness-pipeline (`/api/v1/assets/generate`) |
| Flat `birth_data` only            | `subjects[]` + `relationship_context` + `language` + `report_level` |
| No relationship semantics         | Explicit `relationship_context.type` + `relationship_header` prepended to output |
| No language field                 | `language` is first-class and injected |
| No Folio header contract          | Header is mandatory at top of assembled when relationship present |
| No NotebookLM readiness           | `selemene-notebooklm` skill + `generateSlidesPrompt` |

---

## For OpenClaw Agents

1. Load `selemene-core` + `selemene-cheatsheet.md` first.
2. Use the guided flow in `selemene-report` (one INPUT BOX at a time).
3. After getting an `OrchestratorOutput`, optionally hand it to `selemene-notebooklm`.
4. Always send `X-API-Key`.
5. Expect `relationship_header` at the top of the assembled reading when relationship context was supplied.

**Env still required:**
- `NOESIS_API_KEY`

---

## For Hermes Agents

1. Same flow as above.
2. The bridge still exposes the old 22 tools for engines/workflows.
3. For rich narrative readings, call the assets endpoint (or the local witness-pipeline) using the shapes in `AGENT_FLOW.md`.
4. After the call, you can feed the `OrchestratorOutput` into the NotebookLM shaper.

---

## Recommended Agent Skills (global)

- `selemene-core`
- `selemene-report`
- `selemene-notebooklm`
- `selemene-cheatsheet.md` (ultra-minimal)

---

## Still Valid from Old Docs

- Base URL, auth header, health check
- Engine list (17) and workflow list (6)
- `EngineInput` for direct engine/workflow calls
- Bridge tooling (`@selemene/bridge`, Hermes bridge, etc.)

Everything else for **narrative witness reports** has moved to the rich contract described in `AGENT_FLOW.md`.

---

Read `docs/api/AGENT_FLOW.md` before writing new OpenClaw/Hermes prompts or skills.
