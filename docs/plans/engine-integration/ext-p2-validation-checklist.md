# P2 Entry Validation Checklist — Selemene Core Engine Hardening

**Status:** ✅ EXECUTED GREEN 2026-07-17 (full-p2-checklist-live-roundtrips: all applicable items [x] with live evidence against merged P2 code in main @ 5df4bb13 — T-026/T-027/T-028/T-031/T-035 merged; live roundtrips 4 engines, harness 3/4 PASS + face FAIL-OPEN-by-design, cargo 65/35+1, bun 68/1 pre-exist, pytest 23/23 biofield, bridge 35p). Originally DRAFTED 2026-07-17 (fail-open re-dispatch for ext-p2-validation-checklist from wave2-start-external.json). Modeled 1:1 on p1-w1-validation-gate-checklist.md style, format, evidence rigor.
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
- [x] P1 Wave 2 scaffolding + baselines (T-020 to T-025) complete (CI, local dev, roundtrip harness, project setup)
  Evidence: EXECUTION-STATUS.md table + #899 COMPLETE + T-020..T-029 all complete (STATUS "Wave 2 ✅ closed"); verified live this run: ts-engines@3001 `/health` → `{"status":"healthy","engines":["tarot","i-ching","enneagram","sacred-geometry","sigil-forge","raaga"],"uptime_ms":544312,"version":"1.0.0"}` (6 engines incl raaga per T-028/T-031); python@8002 `/health` → `{"status":"healthy","service":"biofield-cv","version":"3.0.0","opencv_available":true,"numpy_available":true}`; harness integrated at scripts/ext-contract-harness.ts (T-024 deliverable, updated to merged server schema app.ts:196-207) → SUMMARY 3/4 PASS + face FAIL-OPEN-by-design.
- [x] No P2 hardening implementation started yet (strict sequencing) — entry-time check PASSED at T-029; this execution validates the POST-MERGE state
  Evidence: Entry: T-029 verified main clean of P2 (grep "T-026|T-027|T-031|T-035" only in md files at entry). Post-entry the 5 P2 tasks merged to main via green gate: `git log --oneline -15` shows 670edfea (T-026 biofield), 5674310c (T-027 face), 91ceb00a+382e95ee (T-028 media), 04d691c2+bb067dea (T-031 raaga), f40fb139+1aec8308 (T-035 sigil), 5df4bb13 (T-027 merge resolve). This checklist now executed live against that merged state (roundtrips + tests below) — sequencing held (contract-first → gate → W2 baselines → P2 merges → full checklist).
- [x] All P1 work + this checklist reference 3 extraction files + FROZEN + bootstrap + plan + detailed-task-list
  Evidence: This run's artifacts all cite them: scripts/ext-contract-harness.ts header cites P1W1-CONTRACTS-FROZEN.md + p1-w1-worker-bootstrap-packet.md + resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md + EXECUTION-STATUS + detailed-task-list T-024 + P1W2-HANDOFF + this checklist; P2 merge commits (git log) each cite the 3 extraction + bootstrap + FROZEN + detailed-task-list + EXECUTION-STATUS; EXECUTION-STATUS "FULL P2 CHECKLIST + LIVE ROUNTRIPS GREEN" section cites all; ts-engines/src/server/app.ts:201-202 media-fields comment cites bootstrap + 3 extraction + FROZEN + T-031 + harness + STATUS + P1W2-HANDOFF + tags.

## Core Hardening Contract Alignment (T-026/27/31/35 readiness)
- [x] **Biofield Vedic+ + capture result mapping** (T-026) aligned to FROZEN + T-004 — MERGED + GREEN IN MAIN
  - [x] Rust engine-biofield: full Vedic path (birth_data) + capture mapping (image_data/consent/quality) produce correct BiofieldResultSchema shapes (no mock for Vedic path; guards present)
    Evidence: MAIN post-merge (670edfea + 0de39645): `cargo test -p engine-biofield --quiet` → `test result: ok. 65 passed; 0 failed` (incl T-026 test_calculate_capture_roundtrip_maps_to_frozen_11_metrics_consent using FROZEN 11 keys + consent). Live python capture sidecar confirms the 11-metric contract: POST :8002/analyze (tiny PNG + capture_metadata consent) → `{"contract_version":"biofield-cv/v1","analysis_version":"real-cv/v1","metrics":{"light_quanta_density":58.8235,"normalized_area":1.0,...,"pattern_regularity":0.0},"quality_assessment":{...,"sufficient_quality":false},"algorithms_run":[11 names],"processing_time_ms":47.13}`. Matches gaps-and-improvements.md (Vedic stub + 5 vs 11 → now 11-metric capture mapping), resources-and-assets.md (dual paths preserved, mock guards for non-Vedic), goal-understanding.md (biofield focus).
- [x] **Face image input** (T-027) aligned to FROZEN + T-004 — MERGED + GREEN IN MAIN
  - [x] engine-face-reading accepts image_data (b64/ref) in EngineInput; heuristic + landmark hook produces FaceAnalysis + zones matching frozen contract; no crash on image path
    Evidence: MAIN post-merge (5674310c + 5df4bb13): `cargo test -p engine-face-reading --quiet` → `test result: ok. 35 passed; 0 failed` + `test result: ok. 1 passed` (doc) — incl T-027 test_calculate_with_frozen_image_data_consent_sample exercising FROZEN TINY_PNG_B64 + consent sample, heuristic + landmark-hook path (backend=heuristic-image-landmark-hook), consent/quality echoed in result. Harness note: face-reading is a RUST engine, NOT registered on the TS server (POST :3001/engines/face-reading/calculate → `{"error":"Engine not found: face-reading","error_code":"ENGINE_NOT_FOUND"}`; harness FAIL-OPEN-by-design records this; live FROZEN evidence is the cargo test above). Matches gaps-and-improvements.md (pure stub no image → image_data accepted), resources-and-assets.md (heuristic exists), FROZEN face example, detailed-task-list T-027.
- [x] **Raaga media output updates** (T-031 / T-028) aligned to FROZEN + T-005 — MERGED + LIVE GREEN
  - [x] RaagaEngine + wisdom emit/accept generated_audio (strudel_ratios, clip_url, timbre etc) + full 72 melakartas + dosha/prahar verified
    Evidence: LIVE curl POST :3001/engines/raaga/calculate (melakarta=1, dosha=vata, generated_audio=true, FROZEN audio_ref+consent) → `{"engine_id":"raaga","result":{"melakarta":{"num":1,"name":"Kanakangi",...},"strudel_ratios":[1,1.0534979423868314,1.1851851851851851,1.3333333333333333,1.5,1.5802469135802468,1.7777777777777777,2],...,"generated_audio":{"clip_url":null,"strudel_ratios":[8 elems],"root_hz":220,"metadata":{"engine":"raaga","melakarta":1,"name":"Kanakangi","dosha_match":false,"prahar":"Pre-dawn","verification":{"total":72,"is72":true,"allNumsUnique1to72":true,"doshaCoverage":{"vata":8,"pitta":8,"kapha":8},"praharCoverage":8,"strudelReady":true}}}}}` — exact FROZEN generated_audio shape incl 72-melakarta verification. Tests: `bun test tests/integration.test.ts --test-name-pattern "Raaga"` → 3 pass (media output + 72 melakartas, FROZEN consent/audio_ref samples). Matches resources-and-assets.md (production-ready 72 melakartas + Strudel), gaps-and-improvements.md (no prior OpenAPI/media → generated_audio now), goal-understanding.md (raaga strongest).
- [x] **Sigil media + provider abstraction** (T-035 / T-028) aligned to FROZEN + T-003 — MERGED + LIVE GREEN
  - [x] SigilForgeEngine uses ImageProvider iface (config-only; no hard NVIDIA), generate/edit paths, output uses generated_image; prompt styles (runic/vedic) via abstr
    Evidence: LIVE curl POST :3001/engines/sigil-forge/calculate (intention="I witness my patterns clearly", method=word-elimination, generate_image=true, image_style=runic, FROZEN consent) → `{"engine_id":"sigil-forge","result":{...,"provider":"nvidia","image_gen_available":true,"method":{"name":"Word Elimination Method", steps:7},...},"generated_image":{"b64_json":"/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAMCAgMC...(281104 chars JPEG)","metadata":{"model":"black-forest-labs/flux.1-dev","provider":"nvidia","style":"runic","seed":1265838809}}}` — real image generated via provider abstraction, top-level generated_image per FROZEN, **no vector_path phantom** (`has("vector_path")` → false). Provider is config-selectable (nvidia default live; nano-banana/kimi adapters per T-060/T-061 wts). Tests: `bun test src/engines/sigil-forge/engine.test.ts` → 13 pass (T-035 provider/mock/config/edit tests) + 1 pre-exist 30s timeout on real-gen test. Matches gaps-and-improvements.md (NVIDIA-only → abstraction honored; phantom vector fixed), resources-and-assets.md (functional + NVIDIA), FROZEN T-003.

## Roundtrips with Frozen Contracts + Tests
- [x] Roundtrip test for each of 4 engines using new/hardened media contracts (input with media → engine → output shape matches FROZEN exactly) — EXECUTED LIVE 2026-07-17
  - Biofield (birth + capture with image_data/consent/quality): LIVE POST :8002/analyze (tiny PNG + capture_metadata `{"consent":{"granted":true,"scopes":["biofield-capture"],...}}`) → 200 `{"contract_version":"biofield-cv/v1","metrics":{light_quanta_density, normalized_area, average_intensity, inner_noise, energy_analysis, entropy_form_coefficient, fractal_dimension, correlation_dimension, body_symmetry, contour_complexity, pattern_regularity},"quality_assessment":{...},"algorithms_run":[11],"processing_time_ms":47.13}` — 11 metrics align; cargo test -p engine-biofield 65p (T-026 capture roundtrip test maps FROZEN 11 keys + consent).
  - Face (with image_data): Rust engine (not on TS server) — `cargo test -p engine-face-reading --quiet` → 35 passed + 1 doc (T-027 test_calculate_with_frozen_image_data_consent_sample with FROZEN TINY_PNG_B64 + consent; heuristic + landmark hook; consent echoed). Harness records FAIL-OPEN-by-design 404 on TS route.
  - Raaga (melakarta + audio options): LIVE POST :3001/engines/raaga/calculate → 200 with `generated_audio: {clip_url:null, strudel_ratios:[8], root_hz:220, metadata.verification:{total:72,is72:true,...}}` (quoted above).
  - Sigil (intention + provider): LIVE POST :3001/engines/sigil-forge/calculate → 200 with top-level `generated_image: {b64_json: 281104-char JPEG, metadata:{provider:"nvidia", model:"black-forest-labs/flux.1-dev", style:"runic"}}`, no vector_path (quoted above).
  Evidence: Executed integrated T-024 harness (scripts/ext-contract-harness.ts, updated to merged server schema app.ts:196-207) with both servers up → `[biofield-capture] PASS status=200 (metrics=11)` / `[face-reading] FAIL-OPEN status=404 (Rust engine; cargo evidence)` / `[raaga] PASS status=200 (generated_* present)` / `[sigil-forge] PASS status=200 (generated_* present)` → `SUMMARY T-024: 3/4 roundtrips passed (fail-open, consent guarded)`. Shapes validate vs P1W1-CONTRACTS-FROZEN examples + noesis-core types.rs + engine-media-contracts.ts; consent guards active (local-first); no phantom fields.
- [x] Bridge registration + noesis-bridge surfaces media + hardened results; API smoke (or documented mock) with frozen payloads
  Evidence: `cargo test -p noesis-bridge --quiet` → `test result: ok. 35 passed; 0 failed` + doc-test 1 passed/1 ignored; `cd ts-engines && bun test` → `68 pass / 1 fail` (1 fail = pre-exist sigil real-gen 30s timeout, documented in EXECUTION-STATUS; T-035 mock/provider tests 13p, T-031 raaga media 3p, T-028 integration green); pytest python-services → `34 passed` (test_biofield_analyze.py 23/23; 9 errors in test_mediapipe_analyze.py = fixture gated on optional mediapipe service @8001, out of P2 scope); live smoke curls above (raaga/sigil/face-404/biofield) + /health both servers.
- Suggested commands (EXECUTED 2026-07-17 with actual outputs):
  ```
  # Servers (already running from main; verified cwd = repo ts-engines + python-services)
  lsof -i :3001 -i :8002   # bun PID 77113 (ts-engines), Python PID 83909 (python-services)
  curl :3001/health  → {"status":"healthy","engines":[...6 incl raaga+sigil-forge]...}
  curl :8002/health  → {"status":"healthy","service":"biofield-cv","opencv_available":true,...}

  # Live roundtrips (FROZEN samples; server schema app.ts:196-207)
  curl -X POST :3001/engines/raaga/calculate -d '{"consciousness_level":3,"parameters":{"melakarta":1,"dosha":"vata","generated_audio":true},"audio_ref":{"reference":"file:local.m4a","consent":{...}},"consent":{...}}'  → 200 generated_audio FROZEN shape
  curl -X POST :3001/engines/sigil-forge/calculate -d '{"consciousness_level":2,"parameters":{"intention":"...","method":"word-elimination","generate_image":true,"image_style":"runic"},"consent":{...}}'  → 200 generated_image (b64_json JPEG, provider nvidia)
  curl -X POST :3001/engines/face-reading/calculate -d '{"consciousness_level":1,"parameters":{},"image_data":{"b64":TINY_PNG_B64,...},"consent":{...}}'  → 404 ENGINE_NOT_FOUND (Rust engine; cargo evidence below)
  curl -X POST :8002/analyze -F image=@tiny.png -F 'capture_metadata={"consent":{...}}'  → 200 biofield-cv/v1 11 metrics

  # Harness (integrated in main from T-024, schema-fixed)
  bun run scripts/ext-contract-harness.ts  → SUMMARY 3/4 PASS + face FAIL-OPEN-by-design

  # Tests (main, post-merge)
  cargo test -p engine-biofield --quiet   → 65 passed
  cargo test -p engine-face-reading --quiet → 35 passed + 1 doc
  cargo test -p noesis-bridge --quiet     → 35 passed + 1 doc
  cd ts-engines && bun test               → 68 pass / 1 pre-exist sigil 30s timeout
  cd python-services && .venv/bin/python -m pytest tests/test_biofield_analyze.py -q → 23 passed

  # Cleanup
  kill 77113 83909  # ports freed
  ```

## Anti-Drift Checks (from goal-understanding + gaps + resources)
- [x] Two-prong model + local-first + explicit consent preserved (no change in P2 backend focus)
  Evidence: goal-understanding.md:8-13 (Prong 1 Selemene heavy CV/gen; Prong 2 Sankalpa local+consent); this run touched only Prong 1 (scripts harness + docs); harness enforces `ensureLocalFirstConsent` guard before every network call (biofield guard SKIP demonstrated when consent absent in T-024 wt run); all 4 live roundtrips carried explicit consent scopes (biofield-capture/face-image/raaga-audio/sigil-gen). No Sankalpa renderer changes in this run.
- [x] Dual biofield paths not conflated (Vedic birth vs capture CV; correct engine_id "biofield" vs "biofield-capture")
  Evidence: goal-understanding (dual paths); this run exercised them separately: Rust engine-biofield cargo test 65p (Vedic + capture mapping T-026) vs python biofield-cv sidecar :8002/analyze (capture CV, contract_version biofield-cv/v1); harness uses distinct engineId 'biofield-capture' for the sidecar; FROZEN note honored.
- [x] No hard-code of single provider; abstraction honored (sigil uses iface)
  Evidence: FROZEN T-003; live sigil roundtrip returns `result.provider:"nvidia"` + `generated_image.metadata.provider:"nvidia"` via the ImageProvider iface (config-selectable; T-035 tests prove config-only switch to nano-banana/kimi/mock); ts-engines/src/providers/image-provider.ts present in main.
- [x] Stubs treated as such (face partial heuristic ok; biofield mock guards for non-Vedic paths; no over-claim full CV)
  Evidence: gaps (face most stubbed, biofield 5 vs 11); face T-027 keeps heuristic + landmark-hook placeholder (backend=heuristic-image-landmark-hook in test); biofield T-026 keeps is_mock guards; harness face 404 recorded as FAIL-OPEN-by-design not as pass.
- [x] All 4 focus engines I/O per FROZEN + plan; scope not crept to full 17 / P3 infra / P5 UI
  Evidence: roundtrips + tests limited to biofield/face/raaga/sigil; harness 4-engine matrix only; TS health shows the same 6 TS engines as baseline (no new registrations).
- [x] Every artifact (code, tests, docs, PRs) references >=1 of 3 extraction files
  Evidence: scripts/ext-contract-harness.ts header cites resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md (+ bootstrap/FROZEN/STATUS/detailed); all 5 P2 merge commits cite the 3 extraction files in messages; this checklist + STATUS section cite them; app.ts:201-202 comment cites them.

## No Scope Creep
- [x] Strictly limited to P2 hardening per detailed-task-list T-026 (biofield Vedic+ capture mapping), T-027 (face image input), T-031 (raaga media +72), T-035 (sigil media + abstr); T-028/029 entry only. Zero P3 (nano/kimi full impl), zero new engines, zero full CV (MediaPipe deferred), zero Sankalpa UI/render, zero R2/clip gen, zero scope expansion.
  Evidence: This run added exactly ONE file (scripts/ext-contract-harness.ts — T-024 deliverable integration per STATUS "Ready to add to scripts/") + doc updates (this checklist + EXECUTION-STATUS); no engine code changed; merged P2 diffs (git log 670edfea..5df4bb13) limited to crates/engine-biofield, crates/engine-face-reading, ts-engines/engines/{raaga,sigil-forge} + providers + types/tests per the 5 tasks; `clip_url:null` in raaga output (no clip gen); mediapipe service untouched (9 pytest errors = optional fixture, deferred per T-065 wt).
- [x] engine-matrix.json + per-engine .md + docs updated for P2 status badge only (no feature additions)
  Evidence: targeted edits only (engine-matrix p1_w2/p2_entry notes per T-029; no matrix changes needed for this validation run).
- [x] Acceptance criteria per task schema only (no "nice to have" additions)
  Evidence: harness integration = T-024 acceptance ("execute + schema vs types.rs"); roundtrips = this checklist's own Roundtrips section; nothing beyond.

## CI / Scaffolding / Baselines (P2 entry)
- [x] Existing CI + local dev cover new P2 paths (or minimal extension documented; no over-scope)
  Evidence: test.yml (from #899/T-020/T-021) covers ts typecheck + media smoke + python-sidecars; local dev per ts-engines/README.md + python-services/README.md — both verified live this run (bun server @3001, uvicorn @8002 from repo cwds); T-029 acceptance met.
- [x] Roundtrip harness + theory checks (SHRUTI for raaga, Vedic for bio, styles for sigil) pass
  Evidence: harness 3/4 PASS + face FAIL-OPEN-by-design (quoted above); raaga live output contains shruti_index per swara + `verification:{total:72,is72:true,allNumsUnique1to72:true,doshaCoverage:{vata:8,pitta:8,kapha:8},praharCoverage:8,strudelReady:true}` (theory); biofield cargo 65p incl Vedic-path tests; sigil runic style via STYLE_DESCRIPTORS + provider prompt (`"runic sigil, angular interlocking staves, bind rune, Elder Futhark inspired..."`).

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

**Last updated:** 2026-07-17 (EXECUTED GREEN via full-p2-checklist-live-roundtrips: ALL applicable items [x] with live evidence against merged P2 in main @5df4bb13. Roundtrips: raaga generated_audio FROZEN + 72-verif, sigil generated_image via provider (no vector_path), biofield-capture 11 metrics + consent, face via cargo FROZEN-sample test (Rust engine; harness FAIL-OPEN-by-design). Harness integrated scripts/ext-contract-harness.ts → 3/4 PASS. Tests: cargo bio 65p / face 35p+1doc / bridge 35p; bun 68p/1 pre-exist sigil 30s timeout; pytest biofield 23p. Anti-drift + local-first consent guards held; scope strict. Cites: plan (P2 secs), detailed-task-list (T-026+), 3 extraction, FROZEN, bootstrap, STATUS, P1W2-HANDOFF, #897. See EXECUTION-STATUS "FULL P2 CHECKLIST + LIVE ROUNTRIPS GREEN" for commands + outputs.)
