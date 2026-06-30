# FMRL → Web Parity Map

**Date:** 2026-06-02
**Status:** Living checklist (planning)
**Archive note (2026-06-30):** `apps/biofield-web` and `apps/noesis-web` have been retired from this repo. Web parity work continues in [Sankalpa](../../sankalpa/).
**Decision:** Keep **two web apps** — enrich `apps/biofield-web` and `apps/noesis-web` **separately**. Do NOT unify into one fmrl-style app.

## Context

`fmrl` (`thoughtseed/fmrl`) is the rich **desktop blueprint** (Tauri 2 + React 19) — "Frequency Modulated Reality Lens", the full-featured frontend for Selemene's 16 engines. The web apps in this repo (`apps/biofield-web`, `apps/noesis-web`) are the **web version**, currently showing a fraction of fmrl's depth.

**Key reframe:** the web is NOT at 10%. In *breadth* it's ~half of fmrl (all engines present, auth works, reading flow exists). The gap is **depth, workflows, export, and design polish** — not missing functionality wholesale.

**Strategy:** PORT/ENRICH the existing web apps wave-by-wave (not adapt fmrl wholesale). fmrl is the reference to pull components/UX from.

## fmrl's live biofield compute is web-native

fmrl's PIP biofield = **camera → WebGL2 shader → metrics**, computed in-browser (+ optional Rust/Python acceleration). The in-browser path runs on the web with no backend. Tauri-only pieces to SKIP when porting: `RustComputeService`, `useTauriNative`, backend subprocess spawning, `open_oauth_window` (web uses redirect — already done), `open_url_in_browser`, camera-permission repair (tccutil).

## Parity table

| Area | fmrl (blueprint) | Web now | Status |
|---|---|---|---|
| Engine renderers | 12, decomposed (HumanDesign=BodyGraph+Center/Channel/Type; GeneKeys=Hologenetic+Key+Sequence) | noesis-web has 18 (incl. FaceReading, Nadabrahman, Raaga, Transits) but flatter/single-file | ✅ breadth · ⚠️ richness |
| Live Biofield/PIP | PIPShader, VideoPanel (AnalysisModeSelector, CaptureButton, PIPControlPanel, ResultModal), MetricsPanel/ScoreCard, Cards (LiveMetrics/ScoreTile/SymmetrySnapshot), GraphPanel, metrics.worker, segmentation zones | biofield-web: Cosmogram, LiveMetrics, PIPViewerPanel (core only) | ⚠️ PARTIAL |
| Workflows / synthesis | BioCorrelation, SynthesisView, EngineResultGrid, Workflow Card/Progress, useWorkflow | noesis-web: WitnessLayer + /get-reading flow | ❌ rich UI MISSING |
| Export | CSV/JSON/PDF/XLSX services | — | ❌ MISSING |
| Shell / design system | Shell, TimelineStrip, BreathNav, Onboarding, ShortcutsHelp, GlassmorphicCard, SacredGeometryOverlay, PageTransition | noesis-web NavBar (utilitarian) | ⚠️ PARTIAL |
| Pages | Dashboard/Today, DetailedAnalysis, MetricsGuide, EngineDirectory, Workflow, Settings, Account, Onboarding | engines, get-reading, readings, viewer, history, billing, pricing | ⚠️ PARTIAL |
| Auth | AuthGuard, AuthModal, SelemeneAuthBridge | Discord OAuth + JWT handoff (fixed 2026-06-01) | ✅ HAVE |
| Compute services | ComputeRouter (Rust/Python/remote), RustCompute, ScoreCalculator | api.ts, SDK, MetricsCalculator | ✅ HAVE (web = in-browser + remote path) |

## Port order (waves)

### Wave 1 — Biofield depth → `apps/biofield-web` (highest value)
Port from `fmrl/frontend/src/components/`:
- [ ] VideoPanel/ → AnalysisModeSelector, CaptureButton, PIPControlPanel, AnalysisResultModal
- [ ] MetricsPanel/ScoreCard + MetricsPanel/index
- [ ] Cards/ → LiveMetricsCard, ScoreTile, SymmetrySnapshotCard, GlassCard
- [ ] GraphPanel/ (recharts time-series of metrics)
- [ ] Modals/CaptureModal
- [ ] workers/metrics.worker.ts + hooks/useMetricsWorker (off-main-thread metrics)
- [ ] services/segmentation/ (BodySegmenter, FaceSegmenter, ZoneCreator) + useSegmentation
- [ ] pages: DetailedAnalysis, MetricsGuide
- [ ] hooks: useRealTimeMetrics, useFrameCapture, useCameraPermission (web: navigator.permissions/getUserMedia)
- Swap Tauri compute → in-browser WebGL/TF.js (already works) or Python sidecar HTTP.

### Wave 2 — Engine richness → `apps/noesis-web`
- [ ] HumanDesign.tsx → fmrl HumanDesign/ (BodyGraph, CenterDetail, ChannelDetail, TypeCard)
- [ ] GeneKeys.tsx → fmrl GeneKeys/ (HologeneticProfile, KeyDetail, SequenceView)
- [ ] Spot-check + upgrade where flatter: Tarot, IChing, Panchanga, Vimshottari, VedicClock, Numerology, Biorhythm, Enneagram, SacredGeometry, SigilForge
- Keep web-only engines fmrl lacks: FaceReading, Nadabrahman, Raaga, Transits.

### Wave 3 — Workflows / synthesis → `apps/noesis-web`
- [ ] Workflows/ → SynthesisView, BioCorrelation, EngineResultGrid, WorkflowCard, WorkflowProgress + useWorkflow, useBioCorrelation
- Extend existing WitnessLayer + /get-reading into the rich multi-engine synthesis UI.

### Wave 4 — Shell / design polish → both apps
- [ ] Layout/ (Shell, Header, TimelineStrip), UI/ (BreathNav, GlassmorphicCard, MetricsTooltip, SacredGeometryOverlay, SelemeneStatusBadge, ShortcutsHelp), Animations/PageTransition, Onboarding/NativeOnboarding
- Consider a shared web design-system package to avoid duplicating ports across both apps.

### Wave 5 — Export → both apps
- [ ] services Export → CSV/JSON/PDF/XLSX (jspdf already used by fmrl)

## Open questions (decide later)
- Shared component/design-system package between biofield-web + noesis-web (Wave 4) to avoid double-porting?
- Biofield web compute: in-browser WebGL/TF.js (fmrl's path, zero backend) vs Python sidecar (`PYTHON_BIOFIELD_URL`)? Lean in-browser for live metrics.

## Source references
- Blueprint: `thoughtseed/fmrl/frontend/src/{components,services,hooks,pages,workers}`
- Web targets: `apps/biofield-web`, `apps/noesis-web`
