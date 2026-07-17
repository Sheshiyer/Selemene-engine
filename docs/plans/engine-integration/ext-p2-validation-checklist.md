# P2 Entry Validation Checklist — Selemene Core Engine Hardening

**Status:** DRAFTED 2026-07-17 (fail-open re-dispatch for ext-p2-validation-checklist from wave2-start-external.json). Pending execution before T-026+ / #897 hardening starts. Modeled 1:1 on p1-w1-validation-gate-checklist.md style, format, evidence rigor.
**Owner:** Validation swarm (Gemini-style) + orchestrator review.
**Cross-ref:** Fulfills the "ext-p2-validation-checklist" task. Canonical local to repo.
**References (MUST READ before validating):**
- Main plan: `docs/plans/engine-integration/selemene-sankalpa-full-integration-swarm-plan.md` (Phase 2 — Selemene Core Engine Hardening sections: Wave 1 Biofield+face, Wave 2 Raaga, Wave 3 Sigil; verification strategy, per-wave gate, risks)
- Detailed tasks: `docs/plans/engine-integration/detailed-task-list.md` (T-026+ P2: T-026 biofield Vedic+ capture mapping, T-027 face image input, T-031 raaga media +72, T-035 sigil abstraction + media; also T-028/029 for entry)
- Deepened extraction (anti-drift — every change must cite >=1):
  - `resources-and-assets.md`
  - `gaps-and-improvements.md`
  - `goal-understanding.md`
- FROZEN: `.worktrees/T-002-copilot/docs/plans/engine-integration/P1W1-CONTRACTS-FROZEN.md` (media extensions, ImageProvider iface start, Capture*, generated_audio, no vector_path, 4-engine examples)
- Bootstrap: `p1-w1-worker-bootstrap-packet.md` (P1 context + extraction mandate + next after gate)
- EXECUTION-STATUS.md (P1 W1 gate closed, #899 complete, P1W2 breakdown T-020..T-029 incl P2 starts on #897, re-dispatch notes)
- GitHub: #897 (P2), #893 (epic), #901 (P1 gate), #898 (contracts)
- Engine sources (pre-P2 state): `crates/engine-biofield/`, `crates/engine-face-reading/`, `ts-engines/src/engines/{raaga,sigil-forge}/`, `noesis-core/src/types.rs` (frozen), `ts-engines/src/providers/image-provider.ts`
- Sankalpa compat (for roundtrips): `sankalpa/src/renderer/data/engine-media-contracts.ts`, `biofieldDomain.ts`

**Gate Purpose (P2 Entry per T-029 / plan Phase 2):** Confirm P1 contracts frozen + validated, P1W2 baselines/roundtrips/harnesses green, anti-drift vs 3 extraction files enforced, no scope creep (strictly biofield Vedic+ capture mapping, face image input, raaga/sigil media updates per T-026/27/28/31/35), before any P2 hardening code lands. Evidence package attached to #897 + #893. P2 swarms only on green. Contract-first invariant continues.

## Pre-P2 Checks (Orchestrator / All Swarms)
- [x] P1 W1 validation gate closed + T-002/T-003/4/5 validated in worktree + FROZEN 1:1 match
  Evidence: See `p1-w1-validation-gate-checklist.md` (all [x] + full evidence package); EXECUTION-STATUS.md (gate CLOSED section + worktree review); P1W1-CONTRACTS-FROZEN.md; T-002 worktree reviewed green (no merge yet). T-029 close confirmed baselines + P1W2.
- [ ] P1 Wave 2 scaffolding + baselines (T-020 to T-025) complete (CI, local dev, roundtrip harness, project setup)
  Evidence: EXECUTION-STATUS.md table + #899 COMPLETE (bun ts-engines@3001 health+raaga/sigil calc + 61p tests; python@8002 health+11-metric pytest; test.yml enhanced with typecheck+python-sidecars+smoke; READMEs+checklist+STATUS updated); T-024 harness (4 roundtrips per FROZEN); T-029 notes "baselines green".
- [ ] No P2 hardening implementation started yet (strict sequencing)
  Evidence: Per EXECUTION-STATUS ("No P2 started", "contract-first"); pre-state reads: engine-biofield still has mock guards + partial Vedic (T-026 not done); face-reading pure stub no image_data (T-027 not); sigil-forge still direct nvidia (T-035 not, abstr only from T-003); raaga no media generated_audio update (T-031 not). grep for "T-026|T-027|T-031|T-035" only in md files.
- [ ] All P1 work + this checklist reference 3 extraction files + FROZEN + bootstrap + plan + detailed-task-list
  Evidence: grep -r "resources-and-assets.md\|gaps-and-improvements.md\|goal-understanding.md" across docs/plans/engine-integration/ + recent commits; STATUS/FROZEN/bootstrap/checklist all cite; github-issue-mapping + roadmaps cite; this file header does.

## Core Hardening Contract Alignment (T-026/27/31/35 readiness)
- [ ] **Biofield Vedic+ + capture result mapping** (T-026) aligned to FROZEN + T-004
  - [ ] Rust engine-biofield: full Vedic path (birth_data) + capture mapping (image_data/consent/quality) produce correct BiofieldResultSchema shapes (no mock for Vedic path; guards present)
    Evidence: Read crates/engine-biofield/src/engine.rs (VedicBiofieldAnalyzer + generate_vedic... + is_mock_data path), models.rs (BiofieldMetrics 11+), lib.rs, vedic/*; vs FROZEN examples + noesis-core/types.rs BiofieldResult; gaps-and-improvements.md (Vedic stub + 5 vs 11); resources (mock + dual paths); goal-understanding (biofield focus).
- [ ] **Face image input** (T-027) aligned to FROZEN + T-004
  - [ ] engine-face-reading accepts image_data (b64/ref) in EngineInput; heuristic + landmark hook produces FaceAnalysis + zones matching frozen contract; no crash on image path
    Evidence: Read crates/engine-face-reading/src/engine.rs (stub + heuristic_from_seed), models.rs (FaceAnalysis, FaceZone, HealthIndicator); vs FROZEN image_data on Input + FaceReadingResultSchema; gaps (pure stub, no image, most incomplete); resources (heuristic exists); detailed-task-list T-027.
- [ ] **Raaga media output updates** (T-031 / T-028) aligned to FROZEN + T-005
  - [ ] RaagaEngine + wisdom emit/accept generated_audio (strudel_ratios, clip_url, timbre etc) + full 72 melakartas + dosha/prahar verified
    Evidence: Read ts-engines/src/engines/raaga/engine.ts + wisdom.ts; test outputs (Kanakangi + strudel); vs FROZEN generated_audio + raaga audio contract; resources (production-ready 72 melakartas + Strudel); gaps (no prior OpenAPI/media); goal-understanding (raaga strongest).
- [ ] **Sigil media + provider abstraction** (T-035 / T-028) aligned to FROZEN + T-003
  - [ ] SigilForgeEngine uses ImageProvider iface (config-only; no hard NVIDIA), generate/edit paths, output uses generated_image; prompt styles (runic/vedic) via abstr
    Evidence: Read ts-engines/src/engines/sigil-forge/engine.ts (current direct nvidia), prompt-builder.ts; ts-engines/src/providers/image-provider.ts (iface from FROZEN); vs FROZEN T-003 + sigil media; gaps (NVIDIA-only, phantom vector fixed in FROZEN); resources (functional + NVIDIA).

## Roundtrips with Frozen Contracts + Tests
- [ ] Roundtrip test for each of 4 engines using new/hardened media contracts (input with media → engine → output shape matches FROZEN exactly)
  - Biofield (birth + capture with image_data/consent/quality): Vedic path + 11+ metrics align
  - Face (with image_data): heuristic on ref → FaceAnalysis + zones
  - Raaga (melakarta + audio options): generated_audio strudel_ratios + clip
  - Sigil (intention + provider): generated_image via abstr
  Evidence: Execute T-024 harness + new P2 tests; shapes validate vs P1W1-CONTRACTS-FROZEN examples + noesis-core types.rs + engine-media-contracts.ts; bridge to_ts_request + registration green; no phantom fields.
- [ ] Bridge registration + noesis-bridge surfaces media + hardened results; API smoke (or documented mock) with frozen payloads
  Evidence: cargo test -p noesis-bridge; ts-engines bun test; curls per bootstrap/STATUS (raaga/sigil + media variants); python biofield sidecar for capture.
- Suggested commands (update with actual P2 evidence post-run):
  ```
  # Pre-P2 state verification (run now)
  cd /Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/Selemene-engine
  cargo test -p noesis-core --features openapi -- --quiet
  cargo test -p noesis-bridge -- --quiet
  cd ts-engines && bun test
  cargo test -p engine-biofield -- --quiet   # expect partial (pre-hardening)
  cargo test -p engine-face-reading -- --quiet

  # Roundtrips (use FROZEN samples + prior ext-contract examples)
  # TS server
  cd ts-engines && bun run dev &
  curl -X POST http://localhost:3001/engines/raaga/calculate -d '{"consciousness_level":2,"parameters":{"melakarta":1,"generated_audio":true}}' -H 'content-type:application/json'
  # Python capture
  cd python-services && ... uvicorn ... --port 8002 &
  curl -X POST http://localhost:8002/analyze -d '{"image_data":"...","consent":{...}}' ...

  # Post-hardening (T-026+)
  cargo test -p engine-biofield
  cargo test -p engine-face-reading
  cd ts-engines && bun test --grep "raaga|sigil|media"
  # Full 4-engine harness (from T-024 + ext)
  node scripts/contract-roundtrips.ts  # or equiv
  ```

## Anti-Drift Checks (from goal-understanding + gaps + resources)
- [ ] Two-prong model + local-first + explicit consent preserved (no change in P2 backend focus)
  Evidence: goal-understanding.md:8-13 (Prong 1 Selemene heavy CV/gen; Prong 2 Sankalpa local+consent); gaps (local-first invariant); no Sankalpa renderer changes in P2 scope.
- [ ] Dual biofield paths not conflated (Vedic birth vs capture CV; correct engine_id "biofield" vs "biofield-capture")
  Evidence: goal-understanding (dual paths); gaps (Vedic stub + capture 11-metric mismatch); resources (server mock + Sankalpa local PIP); FROZEN note.
- [ ] No hard-code of single provider; abstraction honored (sigil uses iface)
  Evidence: FROZEN T-003; image-provider.ts; current sigil still direct but P2 T-035 will switch.
- [ ] Stubs treated as such (face partial heuristic ok; biofield mock guards for non-Vedic paths; no over-claim full CV)
  Evidence: gaps (face most stubbed, biofield 5 vs 11); resources (heuristic + Vedic stub); detailed-task-list acceptance (heuristic placeholder; mock guards).
- [ ] All 4 focus engines I/O per FROZEN + plan; scope not crept to full 17 / P3 infra / P5 UI
  Evidence: examples + changes limited to 4; engine-matrix as truth.
- [ ] Every artifact (code, tests, docs, PRs) references >=1 of 3 extraction files
  Evidence: (enforce in review) grep + manual in commits; STATUS/FROZEN/bootstrap pattern.

## No Scope Creep
- [ ] Strictly limited to P2 hardening per detailed-task-list T-026 (biofield Vedic+ capture mapping), T-027 (face image input), T-031 (raaga media +72), T-035 (sigil media + abstr); T-028/029 entry only. Zero P3 (nano/kimi full impl), zero new engines, zero full CV (MediaPipe deferred), zero Sankalpa UI/render, zero R2/clip gen, zero scope expansion.
  Evidence: git diff --stat limited to listed crates/engine-biofield, engine-face-reading, ts-engines/engines/{raaga,sigil-forge} + tests + status docs; no new files outside hardening; plan Phase 2 ~30 tasks scoped; EXECUTION-STATUS "P2 start" + T-029 "first P2 tasks"; no creep in prior P1 boilerplate note.
- [ ] engine-matrix.json + per-engine .md + docs updated for P2 status badge only (no feature additions)
  Evidence: targeted edits only.
- [ ] Acceptance criteria per task schema only (no "nice to have" additions)

## CI / Scaffolding / Baselines (P2 entry)
- [ ] Existing CI + local dev cover new P2 paths (or minimal extension documented; no over-scope)
  Evidence: test.yml (from #899), READMEs, cargo/bun runs; T-029 acceptance.
- [ ] Roundtrip harness + theory checks (SHRUTI for raaga, Vedic for bio, styles for sigil) pass
  Evidence: T-024 + new; resources (raaga ready, theory tables).

## Evidence Package Requirements (attach to #897)
- Test logs: cargo test -p engine-biofield/face; bun test (raaga/sigil); pytest biofield (if capture)
- Diffs: git diff main -- crates/engine-biofield/... ts-engines/... (post hardening)
- Reads: full of 3 extraction files + FROZEN + plan P2 sections + detailed T-026+ + current engine.rs before/after + types.rs frozen
- gh: issue view #897 #893; labels (phase:integration-p2, wave:integration-w1, engine-biofield etc); comments with evidence + cites
- Roundtrip outputs: 4 JSON samples matching FROZEN + no drift
- Anti-drift: grep hits for extraction cites; dual-path checks
- Scope check: git diff --stat + manual review vs detailed-task-list
- Commands run + outputs in STATUS update + checklist

## Failure / Re-open Criteria
- Any schema/roundtrip mismatch vs FROZEN → block; re-validate contracts
- Drift vs goal-understanding/gaps/resources (e.g. biofield paths conflated, no extraction cite) → add gap + block
- Scope creep (P3 code, UI, new features) → revert + flag #897
- Missing tests or baselines green → fail entry

**Suggested P2 Harness Additions (for T-029 + waves):** per-engine unit in crates + ts, contract roundtrip matrix exercising media + hardening, adversarial for gen (sigil) + Vedic accuracy spot (bio), theory scripts (72 melakarta, Vedic planetary).

**When this gate is green:** P2 entry passed. Execute T-026+ swarms under #897 (biofield-impl, face-impl, raaga-ts, sigil-ts per plan/detailed). Update EXECUTION-STATUS + P2 handoff note + engine-matrix + gh comments on #897/#893 with full evidence. Reference 3 extraction + FROZEN + bootstrap + plan + detailed-task-list + this checklist always. Contract-first + no drift continues into hardening.

**Last updated:** 2026-07-17 (P2 entry checklist DRAFTED via fail-open re-dispatch for ext-p2-validation-checklist task. 8-10 core items + sub-bullets for hardening (biofield Vedic+ capture, face image, raaga/sigil media), tests, anti-drift vs 3 extraction, roundtrips w/ FROZEN contracts, no scope creep. Format: markdown checkboxes + evidence + commands. Cites: plan (P2 secs), detailed-task-list (T-026+), 3 extraction, FROZEN, bootstrap, STATUS, #897, P1W1 checklist style. Will execute + update STATUS + gh on #897 post P1W2 baselines. See EXECUTION-STATUS for handoff.)
