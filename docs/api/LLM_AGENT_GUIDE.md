# LLM Agent Guide — Updated 2026-07

**Primary reference:** `docs/api/AGENT_FLOW.md`

This is the compact deterministic guide for Claude, OpenCode, Codex, Hermes, OpenClaw, etc.

---

## Current Reality (after July 2026 updates)

Narrative witness reports now use a **rich contract**:

- `subjects[]` (with roles + normalized_location)
- `relationship_context` (type + mapping_goal + sensitivity)
- `language` (first-class, default "en")
- `report_level` (L0–L5)
- Output includes `relationship_header` (prepended to `assembled` when relationship present)

See the three canonical shapes and 8-step flow in `AGENT_FLOW.md`.

---

## What Is Still True from This Document

- Base URL: `https://selemene.tryambakam.space`
- Auth: `X-API-Key` or Bearer JWT
- `EngineInput` for raw engine/workflow calls
- Health check, error schema, verification steps
- Precision values

---

## What Has Changed

- No longer "just call engines/workflows"
- Main path for rich reports: `POST /api/v1/assets/generate` (or local `IntegratedReadingOrchestrator`)
- Post-processing now includes Folio header + NotebookLM shaper
- Agents should follow the guided flow in `selemene-report`

---

## Recommended Agent Skills

- `selemene-core`
- `selemene-report` (step-by-step with INPUT BOXes)
- `selemene-notebooklm`
- `selemene-cheatsheet.md`

---

Read `docs/api/AGENT_FLOW.md` + the cheat sheet before prompting or building new agent logic.
