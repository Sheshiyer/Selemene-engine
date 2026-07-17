# Goal & Overall Understanding — Selemene + Sankalpa Engine Integration (Deepened Extraction)

**Purpose:** Synthesize the true objective from all prior artifacts (plans, ISA, engine docs, roadmaps, issues, matrix). This is the "why" and "what done looks like" so we do not drift during execution.

## 1. Core Objective (One Sentence)
Deliver the four media/embodiment-heavy consciousness engines (biofield with BV-PIP capture, face-reading, raaga, sigil-forge) as first-class, end-to-end experiences inside the Sankalpa desktop instrument, powered by Selemene-engine as the authoritative backend, using contract-first two-pronged architecture.

## 2. Two-Pronged Model (Non-Negotiable)
- **Prong 1 — Selemene-engine (backend/core)**: Owns Rust analysis engines (biofield, face-reading), TS engines (raaga, sigil-forge via bridge), python CV sidecars, image generation providers (extend NVIDIA → nano-banana + kimi), noesis-core contracts, noesis-bridge, API surfaces, scaffolding for future engines.
- **Prong 2 — Sankalpa (frontend/instrument)**: Owns safe Electron shell (context isolation, consent, media perms), local-first previews (e.g. current biofield PIP + MetricsCalculator), input surfaces (camera, image upload, forms), consumption of engine results, visual system (frozen Kha/Ba/La), integration into Noesis readings/depth gallery where applicable. Never holds secrets or does heavy privileged compute.

Heavy lifting (CV, generative AI, full Vedic models) stays in Prong 1 or local-safe compute. Sankalpa is the "instrument" the user plays.

## 3. Focus Engines & Why These Four
- **biofield**: Embodiment + live field (capture + analysis). Dual paths already partially exist (server mock + client local). Highest existing scaffolding in Sankalpa.
- **face-reading**: Visual constitutional analysis. Most stubbed; needs image input + real CV path.
- **raaga**: Auditory/therapeutic. Strongest backend (full TS impl); needs frontend port + integration.
- **sigil-forge**: Generative ritual (intention → method → image). Functional TS + NVIDIA; needs provider expansion + UI.

These represent the spectrum: capture (biofield), CV/analysis (face), audio (raaga), generative image (sigil). Success here creates the pattern for the other 13 engines.

## 4. Success Looks Like (Extracted Criteria)
From swarm-plan + sankalpa/ISA + engine docs:

### Contract & Foundation (P1)
- Frozen, accurate EngineInput/Output + per-engine result schemas in noesis-core (fix all known mismatches).
- Media extensions for image refs, audio, generative outputs, consent metadata.
- Bridge + API can round-trip all four engines.
- engine-matrix.json treated as source of truth.

### Implementation (Waves)
- biofield: Real (or improved Vedic) server path + full capture flow through Selemene `engine-biofield-capture` + Sankalpa surfaces (live + consented capture).
- face-reading: Real image processing (at minimum heuristic + future CV) producing `FaceAnalysis`; wired in Sankalpa.
- raaga: Full Sankalpa player surface using the TS engine output (melakarta, swaras, Strudel, witness).
- sigil-forge: Multi-method UI + image gen via abstracted providers (nano-banana + kimi added); correct output shape (no phantom vector_path).

### Frontend Surfaces (Sankalpa)
- Routes or "Engine Lab" components for the four (matching frozen visual system).
- Explicit consent for every camera/image/gen operation.
- Ability to attach engine result to a Noesis reading / depth gallery.
- Local preview where possible; backend escalation only on opt-in.

### Quality & Hardening
- Tests (unit + contract + integration + adversarial for gen/CV).
- Observability (engine health, latency, quality scores).
- Performance budgets respected.
- Security: no secrets in renderer, permission model enforced, provenance on generative outputs.
- All new work passes wave-boundary validation gates.

### Scope Boundaries
- Not in this initiative: full 17-engine coverage (focus 4 + scaffolding), production packaging/signing, real OAuth/billing (deferred per ISA), MediaPipe parity.
- 2–3 month horizon with daily granular tasks and wave merges.

## 5. Overall Understanding of "What We Are Trying to Achieve"
We are not "porting old web apps". We are building a **desktop consciousness instrument** where:
- The user interacts locally and safely (Sankalpa).
- Authoritative, rich computation lives in a maintainable engine core (Selemene).
- Engines that involve the body (field, face), sound (raaga), or creative ritual (sigil) get first-class treatment because they are the most "embodied" and media-intensive.
- Everything is contract-driven so adding the next engine (or swapping a provider) is low-friction.
- Prior local biofield work in Sankalpa is the prototype pattern, not the final state.

Drift risks to avoid:
- Implementing UI before contracts.
- Conflating the two biofield paths.
- Hard-coding NVIDIA forever.
- Treating stubs as "good enough".
- Losing the local-first + explicit consent principle.

## 6. How Prior Artifacts Map to This Goal
- **sankalpa/ISA.md**: Defines the shell + biofield local foundation + visual contract that new engine surfaces must honor.
- **engine *.md files**: Precise spec for each engine's current state and exact output shapes to build against (and what to fix in contracts).
- **engine-matrix.json**: Inventory + phase plan.
- **Initial swarm plan**: The delivery vehicle (phases, waves, agent split, GitHub tracking).
- **Roadmap updates**: Public visibility of the two-prong integration inside larger consciousness roadmap.
- **GitHub #893/#894/#895**: Coordination surface with the exact tags we invented.

## 7. Unresolved / Needs Clarification (Carry into P1 W1)
- Exact interface + auth for kimi image/yantra generation.
- Whether we build per-engine pages or a unified "Engine Lab" + selector.
- Performance budgets for desktop CV + image gen.
- How deeply raaga/sigil results feed into Noesis witness readings (vs standalone).
- Final prod URLs / deployment model for TS server + python sidecars.

**Extraction sources for this understanding:**
- swarm-plan.md (objective, prongs, risks, quality bar)
- sankalpa/ISA.md + README (what already works, deferred boundaries)
- All four `docs/engines/*.md` (current vs target)
- engine-matrix.json + bridge code
- GitHub issue bodies + roadmap updates we performed
- Prior deep-dive session notes

This document + resources-and-assets.md + gaps-and-improvements.md together form the "pre-execution extraction pack". Any implementation task must reference at least one of these three.
