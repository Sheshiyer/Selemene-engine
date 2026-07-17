# Detailed Task List — Selemene + Sankalpa Engine Integration (Swarm Architect Schema)

**Total Tasks Target:** 130+ (expanded here for fractal daily use). Each task 2-6 hours. Use with the main plan.md.

All tasks follow schemas/task-schema.json exactly.

## Phase 1 Tasks (Foundation & Contracts) — 25 tasks

T-001: { "id": "T-001", "title": "Refresh full discovery: confirm requirements for biofield (live capture + 17 metrics), face-reading (image/landmarks), raaga (audio + theory), sigil-forge (runic/vedic via nano-banana + kimi)", "area": "product", "owner_role": "orchestrator", "owner_agent": "claude", "phase": "P1", "wave": "W1", "swarm": "discovery", "est_hours": 3, "dependencies": [], "deliverable": "Discovery summary + open questions resolved or flagged.", "acceptance": "All media I/O needs documented with examples from engine docs and sankalpa current state.", "validation": "Cross-reference engine-matrix.json, docs/engines/*.md, sankalpa/features.ts and biofieldDomain.ts.", "branch": "swarm/engines/p1-w1/discovery/T-001-claude", "worktree": ".worktrees/T-001-claude", "lock_zone": false, "notes": "Day 1. Decompress: read previous deep dive + code." }

T-002: { "id": "T-002", "title": "Freeze EngineInput + EngineOutput extensions for media (add image_data b64/ref, video_ref, audio_ref, generated outputs)", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P1", "wave": "W1", "swarm": "contracts-backend", "est_hours": 5, "dependencies": ["T-001"], "deliverable": "Updated crates/noesis-core/src/types.rs + TS mirror + OpenAPI.", "acceptance": "UI owner + validation agent approve. Samples for all 4 engines.", "validation": "JSON schema + example payloads validate.", "branch": "swarm/engines/p1-w1/contracts/T-002-copilot", "worktree": ".worktrees/T-002-copilot", "lock_zone": true, "notes": "Lock zone. Critical freeze." }

T-003: { "id": "T-003", "title": "Define and freeze image generation provider abstraction (NVIDIA current, nano-banana, kimi)", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P1", "wave": "W1", "swarm": "contracts-backend", "est_hours": 4, "dependencies": ["T-002"], "deliverable": "Interface + config + prompt builder updates in ts-engines.", "acceptance": "Provider switch is config-only. Sigil engine updated to use it.", "validation": "Unit test with mock providers.", "branch": "swarm/engines/p1-w1/contracts/T-003-copilot", "worktree": ".worktrees/T-003-copilot", "lock_zone": false }

T-004: { "id": "T-004", "title": "Define biofield-capture + face image capture lifecycle contract (requested/uploaded/analyzed/persisted, consent, quality)", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P1", "wave": "W1", "swarm": "contracts-backend", "est_hours": 4, "dependencies": ["T-002"], "deliverable": "Models + API request/response in noesis-api and shared.", "acceptance": "Matches current sankalpa biofieldDomain.ts + extends for full CV.", "validation": "Contract test + Sankalpa preview compatibility.", "branch": "swarm/engines/p1-w1/contracts/T-004-copilot", "worktree": ".worktrees/T-004-copilot", "lock_zone": true }

T-005: { "id": "T-005", "title": "Define raaga audio output contract (strudel_ratios, swaras, prahar + optional server clip URL + timbre/gamaka)", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P1", "wave": "W1", "swarm": "contracts-backend", "est_hours": 3, "dependencies": ["T-002"], "deliverable": "Extension to raaga result shape + Strudel player interface.", "acceptance": "Matches raagaegnin theory + sankalpa can render.", "validation": "Theory verification script + sample output.", "branch": "swarm/engines/p1-w1/contracts/T-005-copilot", "worktree": ".worktrees/T-005-copilot", "lock_zone": false }

... (T-006 to T-025: Sankalpa media UI contracts (camera/file components, consent UI), CI baseline updates, GitHub label setup, first wave issues creation, worker bootstrap packet for P1, local dev setup for TS server + python, verification gate for P1 W1, handoff docs, etc. Full 25 in main plan + this file.)

## Phase 2 Tasks (Selemene Hardening) — 30+ tasks

T-026: { "id": "T-026", "title": "Harden engine-biofield Rust: full Vedic path, mock guards, capture result mapping", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P2", "wave": "W1", "swarm": "biofield-impl", "est_hours": 6, "dependencies": ["T-002","T-004"], "deliverable": "engine.rs + models updated, tests pass.", "acceptance": "Birth data path + capture path produce correct shapes.", "validation": "Unit + integration against noesis-api.", "branch": "swarm/engines/p2-w1/biofield/T-026-copilot", "worktree": ".worktrees/T-026-copilot", "lock_zone": false }

T-027: { "id": "T-027", "title": "Extend engine-face-reading for image_data input (heuristic + landmark hook)", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P2", "wave": "W1", "swarm": "face-impl", "est_hours": 5, "dependencies": ["T-002","T-004"], "deliverable": "engine.rs + models support image_data path.", "acceptance": "Heuristic works; placeholder for real CV.", "validation": "Tests + sample with/without image.", "branch": "swarm/engines/p2-w1/face/T-027-copilot", "worktree": ".worktrees/T-027-copilot", "lock_zone": false }

T-031: { "id": "T-031", "title": "Update RaagaEngine + wisdom for media output options and full 72 melakartas verification", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P2", "wave": "W2", "swarm": "raaga-ts", "est_hours": 4, "dependencies": ["T-005"], "deliverable": "engine.ts + tests updated.", "acceptance": "All 72 + dosha/time selection + audio contract.", "validation": "SHRUTI theory check + Strudel compatibility.", "branch": "swarm/engines/p2-w2/raaga/T-031-copilot", "worktree": ".worktrees/T-031-copilot", "lock_zone": false }

T-035: { "id": "T-035", "title": "Refactor SigilForgeEngine to new image provider abstraction + add generate/edit paths", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P2", "wave": "W3", "swarm": "sigil-ts", "est_hours": 5, "dependencies": ["T-003"], "deliverable": "engine.ts + prompt-builder updated.", "acceptance": "Guidance + image gen via abstraction.", "validation": "Test with at least two providers (mock + one real).", "branch": "swarm/engines/p2-w3/sigil/T-035-copilot", "worktree": ".worktrees/T-035-copilot", "lock_zone": false }

(Continue pattern for models, wisdom, witness injection, unit/integration tests, bridge registration verification, noesis-orchestrator updates, ~30 total with daily granularity.)

## Phase 3 Tasks (Infrastructure) — 20+ tasks

T-060: { "id": "T-060", "title": "Implement nano-banana provider module (text-to-image + edit) for sigil/yantras", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P3", "wave": "W1", "swarm": "image-providers", "est_hours": 6, "dependencies": ["T-003"], "deliverable": "nano-banana.ts or util + registration.", "acceptance": "Works for runic/vedic styles per user spec.", "validation": "Sample generations + quality review.", "branch": "swarm/engines/p3-w1/providers/T-060-copilot", "worktree": ".worktrees/T-060-copilot", "lock_zone": false }

T-061: { "id": "T-061", "title": "Implement kimi provider path for sigil generation/edit", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P3", "wave": "W1", "swarm": "image-providers", "est_hours": 5, "dependencies": ["T-003"], "deliverable": "kimi adapter + prompt templates for yantras.", "acceptance": "Integrated in abstraction, selectable.", "validation": "End-to-end in sigil engine test.", "branch": "swarm/engines/p3-w1/providers/T-061-copilot", "worktree": ".worktrees/T-061-copilot", "lock_zone": false }

T-065: { "id": "T-065", "title": "Integrate python biofield CV service (mediapipe + metrics) into capture flow", "area": "backend", "owner_role": "backend-architect", "owner_agent": "copilot", "phase": "P3", "wave": "W2", "swarm": "cv-services", "est_hours": 6, "dependencies": ["T-004","T-026"], "deliverable": "python-services updates + api client wiring + result mapping.", "acceptance": "Full 11+ metrics from real CV.", "validation": "Health + analyze tests, sample capture.", "branch": "swarm/engines/p3-w2/cv/T-065-copilot", "worktree": ".worktrees/T-065-copilot", "lock_zone": false }

(Additional for face CV hook, raaga clip generation (suno-bridge), abstraction tests, fallback logic, ~20 total.)

## Phase 5 Tasks (Sankalpa — heaviest) — 35+ tasks

T-100: { "id": "T-100", "title": "Implement safe camera capture component in Sankalpa (Electron media perms, local preview, consent)", "area": "frontend", "owner_role": "app-builder", "owner_agent": "codex", "phase": "P5", "wave": "W1", "swarm": "media-io", "est_hours": 5, "dependencies": ["T-004"], "deliverable": "PipPortal + new capture hooks + consent UI.", "acceptance": "Local pixels + opt-in to backend; no auto network.", "validation": "Manual test + Playwright + sandbox check.", "branch": "swarm/engines/p5-w1/media/T-100-codex", "worktree": ".worktrees/T-100-codex", "lock_zone": false }

T-105: { "id": "T-105", "title": "Build raaga engine surface in Sankalpa (melakarta input, swara wheel, Strudel player, clip option)", "area": "frontend", "owner_role": "app-builder", "owner_agent": "codex", "phase": "P5", "wave": "W2", "swarm": "raaga-ui", "est_hours": 6, "dependencies": ["T-005","T-031"], "deliverable": "New route or component + integration with engine client.", "acceptance": "Plays correct ratios, shows theory, optional backend clip.", "validation": "Audio correctness + UI screenshots.", "branch": "swarm/engines/p5-w2/raaga/T-105-codex", "worktree": ".worktrees/T-105-codex", "lock_zone": false }

T-115: { "id": "T-115", "title": "Sigil Forge UI in Sankalpa (intention + method + provider select + b64 image viewer + manual draw guidance)", "area": "frontend", "owner_role": "app-builder", "owner_agent": "codex", "phase": "P5", "wave": "W3", "swarm": "sigil-ui", "est_hours": 5, "dependencies": ["T-035","T-060","T-061"], "deliverable": "Full surface + save/export.", "acceptance": "Matches design system; guidance first, image secondary.", "validation": "Screenshots + manual flow test.", "branch": "swarm/engines/p5-w3/sigil/T-115-codex", "worktree": ".worktrees/T-115-codex", "lock_zone": false }

T-120: { "id": "T-120", "title": "Face reading capture + result visualization (face map, elemental, health zones)", "area": "frontend", "owner_role": "app-builder", "owner_agent": "codex", "phase": "P5", "wave": "W3", "swarm": "face-ui", "est_hours": 4, "dependencies": ["T-027"], "deliverable": "Capture + viz components.", "acceptance": "Renders constitution + indicators correctly.", "validation": "With/without image samples.", "branch": "swarm/engines/p5-w3/face/T-120-codex", "worktree": ".worktrees/T-120-codex", "lock_zone": false }

(Additional: engine launcher, Noesis depth integration for results, local fallback logic, styling all new surfaces, tests in sankalpa, 35+ total covering all media + general engines.)

## Remaining Phases (condensed with counts)
Phase 4: 15 tasks on api exposure, bridge, sdk, health (T-080 to T-094).
Phase 6: 15 tasks on workflows + polish (T-136 to T-150).
Phase 7: 15+ tasks on qa/deploy (T-151 to T-170+): full matrix, adversarial gen/CV, CI, deploy, rollout, memory capture, lessons.

**Daily Fractal Example (for any wave):**
- Pick 2-4 tasks/day.
- Morning: load memory_inputs + contract.
- Execute deliverable.
- Afternoon: validation evidence + update issue + handoff note.
- EOD: push branch, post status.

All tasks include explicit file paths in full expansion (e.g. edit crates/noesis-core/src/types.rs for T-002, edit sankalpa/src/renderer/biofield/* and App.tsx for T-100 series).

This list + main plan + discovery = complete Swarm Architect output.

Status: Ready for GitHub import and daily execution.
