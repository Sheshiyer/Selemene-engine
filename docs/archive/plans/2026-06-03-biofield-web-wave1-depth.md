# Biofield-Web Wave 1 — Analysis Depth Implementation Plan

> **Archive note (2026-06-30):** `apps/biofield-web` has been retired from this
> repo. This plan is preserved for historical context; biofield depth work has
> moved to [Sankalpa](../../sankalpa/).

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bring `apps/biofield-web`'s `/viewer` from a basic live view up to fmrl's analysis depth — persistent score panel, time-series graph, rich capture controls, result modal, and detail pages — wired to biofield-web's existing data layer.

**Architecture:** PORT fmrl components as the **UX/visual reference only**; wire them to biofield-web's **existing** hooks/data (`components/pip/useLiveMetrics`, `PIPViewerPanel`, `CompositeScores`). Restyle to biofield-web design tokens. Do NOT import fmrl's data layer (it's Tauri/`useAppState`-coupled).

**Tech Stack:** Next.js (app router), React 19, TypeScript (strict), recharts (graph), existing `@selemene/*` packages. No Tailwind (biofield-web uses CSS vars + `biofield-*` classes + inline styles).

---

## What biofield-web ALREADY has (do NOT rebuild)
- Compute pipeline: `components/pip/{useCamera,useMediaPipe,usePIPRenderer,useLiveMetrics,MetricsCalculator,PIPRenderer}`
- `PIPViewerPanel` (`onCapture`, `onMetrics`), `BiofieldLiveMetrics` (overlay), `BiofieldCosmogram`
- Session lifecycle (start/hydrate/close), manual + PIP capture upload, capture-result tiles, DyadChamber banner, Noesis synthesis, `/history`, `/readings/[id]`
- Live data shape: `useLiveMetrics → { frame: FrameMetrics, composite: CompositeScores }`

## Three cross-cutting adaptations (resolve FIRST — Tasks 1–2)
1. **Metric vocabulary mismatch.** fmrl ScoreCards use `energy/symmetry/coherence/complexity/regulation/colorBalance`; biofield-web `CompositeScores` uses `lightQuantaDensity/normalizedArea/bodySymmetry/patternRegularity/overallCoherence` (read `components/pip/types.ts` for the authoritative shape). biofield-web's vocabulary is the source of truth. Build a `scoreModel` adapter, not fmrl's keys.
2. **State coupling.** fmrl components consume `useAppState` (AppContext reducer). biofield-web uses local viewer state. Port components as **props-driven** (pass `scores`, `history`, `onCapture`), no global context.
3. **Styling.** fmrl uses Tailwind classes + lucide-react. Restyle ported components to biofield-web tokens (`var(--panel)`, `var(--accent)`, `biofield-*` classes). Confirm whether `lucide-react` is a dep; if not, add it or use existing icon approach.

---

## Task 0 — Worktree baseline
**Files:** none (env only)
- Step 1: `cd` to this worktree; `npm install` at repo root (monorepo) — deferred from worktree setup.
- Step 2: `npm --prefix apps/biofield-web run typecheck` (or root equivalent) → expect 0 errors (clean baseline).
- Step 3: Detect test tooling: `cat apps/biofield-web/package.json` → is there `vitest`/`test`? Record the test command. If none, logic-only tasks (1, 3) get a lightweight `vitest` setup or a one-off node test; UI tasks verify via typecheck + visual.
- Step 4: `npm --prefix apps/biofield-web run dev` once → confirm `/viewer` renders today (baseline screenshot).

## Task 1 — Score model adapter (foundation)
**Files:** Create `apps/biofield-web/src/components/pip/scoreModel.ts`; Test `…/scoreModel.test.ts`
- Read `components/pip/types.ts` for the real `CompositeScores`/`FrameMetrics` fields.
- Define a `DisplayScore[]` model: `{ key, label, value0to1, description }` mapping biofield-web metrics → friendly labels (e.g. `overallCoherence→Coherence`, `bodySymmetry→Symmetry`, `patternRegularity→Regulation`, `lightQuantaDensity→Energy`, `fractalDimension→Complexity` if present, `normalizedArea→Field area`).
- TDD: failing test `mapsCompositeScoresToDisplay()` → implement `toDisplayScores(composite)` → pass → commit.

## Task 2 — ScoreCard + live MetricsPanel
**Files:** Create `src/components/metrics/ScoreCard.tsx`, `src/components/metrics/MetricsPanel.tsx`
- Reference: fmrl `components/MetricsPanel/index.tsx` + `MetricsPanel/ScoreCard.tsx` (read both).
- Adapt: props-driven (`scores: DisplayScore[]`), biofield-web tokens (no Tailwind), animated bar like the existing capture-result tiles.
- Integrate: in `app/(protected)/viewer/page.tsx`, feed `liveScores` (via Task 1 adapter) into `<MetricsPanel>` in the RIGHT column so the 6 scores are **always visible live** (not only post-capture).
- Verify: typecheck + dev render shows live-updating scores. Commit.

## Task 3 — Metrics history buffer + GraphPanel
**Files:** Create `src/components/pip/useMetricsHistory.ts` (+ test), `src/components/metrics/GraphPanel.tsx`
- Reference: fmrl `components/GraphPanel/index.tsx`.
- `useMetricsHistory(scores, capacity=60)`: ring buffer of recent composite scores (TDD the buffer: caps at capacity, newest last).
- `GraphPanel`: recharts line chart of selected metrics over the session window. Restyle to tokens.
- Integrate into viewer RIGHT column under MetricsPanel. Verify + commit.

## Task 4 — Live capture controls
**Files:** Create `src/components/pip/CaptureButton.tsx`, `AnalysisModeSelector.tsx`; Modify `PIPViewerPanel.tsx` + viewer.
- Reference: fmrl `VideoPanel/{CaptureButton,AnalysisModeSelector,index}.tsx`.
- Adapt: CaptureButton with progress/success/error feedback → calls existing `onCapture` (which already uploads via `client.uploadCapture`). AnalysisModeSelector sets a local `analysisMode` passed into capture metadata. Pause/Play wired to existing `usePIPRenderer` pause/resume. NO `useFrameCapture` (Tauri) — reuse biofield-web's capture path.
- Verify capture still uploads + shows result. Commit.

## Task 5 — PIP control panel (shader settings)
**Files:** Create `src/components/pip/PIPControlPanel.tsx`; Modify `usePIPRenderer.ts`/`PIPRenderer.ts` to expose adjustable params.
- Reference: fmrl `VideoPanel/PIPControlPanel.tsx`.
- Expose PIP shader controls (intensity/palette/threshold) bound to biofield-web's `PIPRenderer`. Collapsible. Verify live param changes. Commit.

## Task 6 — Analysis result modal + baseline
**Files:** Create `src/components/metrics/AnalysisResultModal.tsx`; Modify viewer capture-result section.
- Reference: fmrl `VideoPanel/AnalysisResultModal.tsx`.
- Show capture result in a modal with per-metric vs **baseline** deltas (baseline = first capture of session, stored locally). Replace/augment the inline capture-result tiles. Verify. Commit.

## Task 7 — Rich cards
**Files:** Create `src/components/metrics/{ScoreTile,SymmetrySnapshotCard,GlassCard}.tsx`
- Reference: fmrl `Cards/*`. Restyle to tokens. Use in MetricsPanel/result modal where they improve density. Verify + commit.

## Task 8 — Off-thread metrics (perf, optional)
**Files:** Create `src/components/pip/metrics.worker.ts`; Modify `useLiveMetrics.ts` to offload.
- Reference: fmrl `workers/metrics.worker.ts` (343 lines) — port the heavy math off the main thread. Keep the existing synchronous path as fallback. Verify metrics still match (parity) + no UI jank. Commit.

## Task 9 — Detail pages
**Files:** Create `app/(protected)/analysis/page.tsx`, `app/(protected)/metrics-guide/page.tsx`; add nav links.
- Reference: fmrl `pages/{DetailedAnalysis,MetricsGuide}.tsx`. Adapt to app-router + biofield-web data/auth. Verify routes render. Commit.

## Task 10 — Viewer integration + polish
- Recompose the RIGHT column: DyadChamber → session strip → **MetricsPanel** → **GraphPanel** → capture controls → result; keep LEFT = PIP + cosmogram.
- Full `typecheck` (0 errors) + dev walkthrough screenshot. Final commit.

---

## Verification gate (every task)
- `npm --prefix apps/biofield-web run typecheck` → 0 errors
- Logic tasks (1, 3, 8): unit tests pass
- UI tasks: `/viewer` renders + the new piece works against live camera data
- Commit after each task (conventional `feat(biofield-web): …`)

## Out of scope (later waves)
- Engine renderer richness → noesis-web (Wave 2)
- Workflows/synthesis (Wave 3), shell/design system (Wave 4), export (Wave 5)

## Source references
- Blueprint: `thoughtseed/fmrl/frontend/src/{components,hooks,services,workers,pages}`
- Target: `apps/biofield-web/{app/(protected)/viewer,src/components}`
- Parity map: `docs/plans/2026-06-02-fmrl-web-parity.md`
