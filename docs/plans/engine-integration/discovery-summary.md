# Discovery Output — Selemene + Sankalpa Engine Integration

> **Superseded / Deepened:** See the three extraction files created 2026-07-17 in this directory:
> - `resources-and-assets.md`
> - `gaps-and-improvements.md`
> - `goal-understanding.md`
> These were produced explicitly to understand what has been done, extract resources, surface gaps, and lock goal understanding before any execution.

**Date:** 2026-07-16 (original); 2026-07-17 (deepened)
**Planning depth:** deeply detailed
**Delivery mode:** production + hardening
**Release model:** phased rollout (wave boundaries)
**CI/CD:** production-grade (GitHub Actions + existing + enhancements)
**Quality bar:** High (tests, contracts, media consent/security, performance, observability, adversarial for CV/gen)

## Confirmed Inputs
- Two-pronged architecture: Selemene (Rust/TS/Python backend + scaffolding) as core; Sankalpa (Electron) as frontend instrument.
- Focus engines: biofield (BV-PIP capture), face-reading, raaga, sigil-forge (with target nano-banana + kimi image gen).
- Other media-tagged engines to review/integrate where > form input/output.
- Evidence from deep-dive: sankalpa has scaffolding for biofield (local only), core engines in Selemene (some stubs/mocks), bridge for TS engines, etc.
- 2-3 month daily cadence, granular tasks.

## Assumptions Made
- Contracts for media (image/video/audio refs, generative outputs) can be frozen early in P1.
- Backend heavy lifting (CV, gen) stays in Selemene; Sankalpa does safe local + consented remote.
- Raaga/sigil remain TS (bridge); biofield/face have Rust + sidecar.
- Existing design system in Sankalpa frozen; new surfaces must match.
- GitHub for tracking; worktree/branch per task.

## Unresolved Questions (P1 W1 priority)
- Exact "kimi code on an api" details (endpoint, auth, prompt format for yantras/runic).
- Scope for non-4 engines (how many get dedicated Sankalpa surfaces vs unified lab).
- Performance budgets for desktop CV/gen.
- Prod URLs for TS-engines server and python sidecars.

## Proposed Agent Split
- Orchestrator/Planner: OpenCode/Claude
- UI/App (Sankalpa): Codex-style
- Backend/Selemene/Rust/Bridge/Python: Copilot-style
- Validation: Gemini-style

## Recommended Plan Shape
- 7 phases, ~3 waves each, 2-3 swarms per wave.
- 120-150+ schema tasks.
- Contract-first, wave-boundary merges.
- Full GitHub sync + worker bootstrap packets.
