# GitHub Issue Update Packets

> **DRAFT — for human review, not yet posted.** Nothing here has been sent to GitHub. Per `GH-ISSUE-NEXT-WAVE-PLAN.md` Global Constraints, do not run `gh issue comment` on these until the user explicitly authorizes remote GitHub updates.

Produced 2026-08-31 as Task 5 of `GH-ISSUE-NEXT-WAVE-PLAN.md`, drafted and adversarially verified against live `gh`/`git` state in the same session (all issue numbers/titles/states re-confirmed via `gh issue view`; all commit shas and test counts re-confirmed against local repo state).

## #893 — [roadmap] Evidence-led Selemene completion plan: gates -> contracts -> engines

DRAFT — for human review, not yet posted.

Gate status update. Security-audit gate is green on main: PR #1485 (07fb220, 1348460, 3252b79) merged as ae3e2cef; CI "Test & Lint" run 33048319281 succeeded at that sha. The prior failing run (32248132223, sha b0827e1a) predates the fix and is resolved, not open.

New evidence slice (local, uncommitted). A GET /engines/capabilities endpoint has landed locally on top of ae3e2cef (ts-engines/src/server/registry.ts EngineRegistry.listCapabilities(), ts-engines/src/server/app.ts route wiring). It returns canonical v1 ContractEngineCapability records (contract_version, engine_id, display_name, availability, runtime_kind, dependencies, required_phase, implementation_version) for the 6 registered TS engines (tarot, i-ching, enneagram, sacred-geometry, sigil-forge, raaga), with availability derived from each engine's live self-check (available/unavailable) rather than static claims. No provider/DB/remote calls are made.

Tests: bun test tests/baseline_registry.test.ts tests/health.test.ts -> 9 pass, 0 fail. Full suite: bun run typecheck && bun test -> typecheck clean, 92/92 pass.

Evidence-axis mapping: this satisfies Implemented (exact symbols above) and Executable (reproducible test commands + output) for the registry-level capability-discovery slice. It does not yet satisfy Deployed (live Railway/Cloudflare deployment is still on the pre-#1485 commit; GET /engines/capabilities 404s live) or Operational (no production health/trace evidence). The change is also still uncommitted locally, so it is not yet Integrated in the sense of a merged, reviewed change.

No action requested here beyond record-keeping; flagging for whoever updates the ledger in #896.

---

## #896 — [W0][authority] Reconcile engine inventory and completion evidence ledger

DRAFT — for human review, not yet posted.

Ledger update candidate:
- Security-audit gate: GREEN on main @ ae3e2cef (PR #1485: 07fb220, 1348460, 3252b79). CI run 33048319281 succeeded at this sha.
- New capability-discovery slice (uncommitted, local only): EngineRegistry.listCapabilities() (ts-engines/src/server/registry.ts) + GET /engines/capabilities (ts-engines/src/server/app.ts) covering tarot, i-ching, enneagram, sacred-geometry, sigil-forge, raaga. Response shape: v1 ContractEngineCapability[] with availability sourced from live per-engine self-check.
- Axis receipts for this slice: Implemented — yes (symbols cited above). Executable — yes (bun test tests/baseline_registry.test.ts tests/health.test.ts = 9/9 pass; full suite = 92/92 pass, typecheck clean). Declared — partial: response shape matches an existing internal ContractEngineCapability type (ts-engines/src/types/engine.ts), but no canonical cross-repo contract doc is cited here. Integrated — not yet (uncommitted, no PR). Deployed — no (live deployment predates PR #1485 and 404s on this route). Operational — no.
- Important caveat: this is a coarse, registry-wide availability signal (available/unavailable per engine), not the full per-engine capability schema (modes/inputs/outputs/versions/limitations) required by the W3E:04 issues (#1458, #1338, #1308, #1398, #1428, #1368). Do not mark those issues' acceptance criteria satisfied from this alone.

---

## #901 — [W1][gates] Fail-closed validation and release evidence gates

DRAFT — for human review, not yet posted.

Status note: security-audit release gate is now green on main (PR #1485, sha ae3e2cef; CI run 33048319281 succeeded). The previously-flagged failing run (32248132223 @ b0827e1a) is resolved/superseded, not open.

Separately, a new local (uncommitted) slice adds a self-check-backed GET /engines/capabilities endpoint (ts-engines/src/server/registry.ts listCapabilities(), ts-engines/src/server/app.ts route). It fails closed in the sense that availability is computed from each engine's live self-check result (healthy→'available', unhealthy→'unavailable') rather than a static/declared claim — relevant if this gate's scope includes capability-truth gating. Not yet integrated (no commit/PR) or deployed, so no release-gate evidence should be recorded for it until it lands on main and ships live.

---

## #894 — [W2][contracts] Canonical schemas, catalogues, bridge, SDK and CLI convergence

DRAFT — for human review, not yet posted.

Contract-convergence note: a v1 ContractEngineCapability type (ts-engines/src/types/engine.ts, CapabilityAvailability = 'declared' | 'available' | 'degraded' | 'unavailable') and a matching GET /engines/capabilities HTTP route now exist locally (uncommitted) on top of main @ ae3e2cef, returning capability records for all 6 registered TS engines. Fields: contract_version, engine_id, display_name, availability, runtime_kind, dependencies, required_phase, implementation_version.

This is a candidate building block for the canonical capability-discovery contract this issue tracks, but it is scoped to the TS-engine registry only — no SDK/CLI/bridge convergence work has been done against it yet, and it has not been committed, reviewed, or deployed. Tests (bun test tests/baseline_registry.test.ts tests/health.test.ts -> 9/9 pass; full suite 92/92 pass) prove the endpoint behaves as coded, not that it is the agreed canonical shape across bridge/SDK/CLI — flagging for contract owners to confirm before wider adoption.

---

## #897 — [W3][engines] Close engine and media truth gaps across partial lenses

DRAFT — for human review, not yet posted.

Cross-engine evidence note: a registry-level GET /engines/capabilities endpoint (local, uncommitted, on main @ ae3e2cef) now reports live availability (via self-check) for all 6 W3 engines — tarot, i-ching, enneagram, sacred-geometry, sigil-forge, raaga — via EngineRegistry.listCapabilities() (ts-engines/src/server/registry.ts) and the route in ts-engines/src/server/app.ts. Tests: 9/9 pass on the targeted files, 92/92 on the full suite, typecheck clean.

This is Implemented + Executable evidence for a coarse capability-discovery signal only. It explicitly does not close any per-engine:04 Publish capability discovery metadata issue (#1458 tarot, #1338 i-ching, #1308 enneagram, #1398 sacred-geometry, #1428 sigil-forge, #1368 raaga) — those require per-engine modes/inputs/outputs/versions/limitations plus boundary/negative/degraded fixtures, none of which this endpoint provides. Also not yet Deployed: live deployment predates PR #1485 and this route 404s there today. Flagging as a partial input to those child issues, not a substitute for their acceptance criteria.

---

## #913 — [W4][state-auth] Persistence, auth, billing, invitations and durability hardening

DRAFT — for human review, not yet posted.

Scope note (no direct action expected here): the new GET /engines/capabilities route (local, uncommitted, on main @ ae3e2cef; ts-engines/src/server/app.ts + registry.ts) reads only the in-process engine registry and each engine's local self-check — it makes no persistence, auth, billing, or invitation calls, and requires no auth today. If this endpoint is later exposed publicly, confirm with this issue's owners whether it needs auth/rate-limit treatment before deploy. No evidence-axis claim is made here beyond noting the endpoint has zero footprint on state/auth/billing surfaces.

---

## #908 — [W5][distribution] Release compatibility surfaces from canonical Selemene contracts

DRAFT — for human review, not yet posted.

Distribution-readiness note: a v1 ContractEngineCapability response (GET /engines/capabilities) now exists locally (uncommitted, on main @ ae3e2cef) — see ts-engines/src/server/registry.ts listCapabilities() and ts-engines/src/server/app.ts. This is a plausible input to a future bridge/SDK/CLI capability surface, but no such downstream distribution work has started, the change is not committed/merged, and it is not deployed (live ts-engines deployment predates PR #1485 and 404s on this route). Evidence today is Implemented + Executable only (tests: 9/9 targeted, 92/92 full suite). Flagging for awareness, not requesting action.

---

## #914 — [W6][ops-assets] Deployment, observability, asset governance and rollback receipts

DRAFT — for human review, not yet posted.

Deployment status note: main is at ae3e2cef (PR #1485 merged; CI run 33048319281 green). The live Railway/Cloudflare ts-engines deployment has NOT yet picked this up and remains on an older commit — confirmed via a live 404 on a route this session's local build exposes (GET /engines/capabilities, ts-engines/src/server/app.ts + registry.ts, uncommitted locally as of this writing). No observability/rollback receipts exist for this new endpoint because it has not shipped. Once committed, merged, and deployed, this endpoint would be a natural target for an operational health/observability probe under this issue's scope — flagging for the next deploy cycle, not requesting one now.

---

## #1458 — [W3E:tarot:04] Publish capability discovery metadata

DRAFT — for human review, not yet posted.

Partial, generic evidence only — does not close this issue. GET /engines/capabilities (ts-engines/src/server/app.ts, EngineRegistry.listCapabilities() in ts-engines/src/server/registry.ts; local/uncommitted on main @ ae3e2cef) now reports Tarot's availability ('available'/'unavailable') derived from its live self-check, plus implementation_version and required_phase from engine.metadata(). Covered by bun test tests/baseline_registry.test.ts tests/health.test.ts (9/9 pass) and the full suite (92/92 pass).

Gap against this issue's acceptance criteria: no supported-modes/inputs/outputs/limitations schema is exposed (only availability + version + phase), no boundary/negative/degraded fixtures specific to Tarot's spread positions/orientation/seed-replay are added, and the endpoint is not yet deployed (live deployment 404s on this route). This is Implemented + Executable evidence for a coarse availability signal, not a capability-discovery-metadata closure.

---

## #1338 — [W3E:i-ching:04] Publish capability discovery metadata

DRAFT — for human review, not yet posted.

Same generic slice as tarot's #1458, applied to I Ching: GET /engines/capabilities (ts-engines/src/server/app.ts + registry.ts, local/uncommitted on main @ ae3e2cef) reports I Ching's live-self-check availability plus implementation_version/required_phase. Tests: 9/9 targeted, 92/92 full suite, typecheck clean.

Does not close this issue: no modes/inputs/outputs/limitations schema, no I Ching-specific boundary/negative/degraded fixtures (hexagram casting, changing lines, etc.), not yet deployed. Flagging as partial Implemented + Executable evidence only.

---

## #1308 — [W3E:enneagram:04] Publish capability discovery metadata

DRAFT — for human review, not yet posted.

Same generic slice, applied to Enneagram: GET /engines/capabilities (ts-engines/src/server/app.ts + registry.ts, local/uncommitted on main @ ae3e2cef) reports Enneagram's live-self-check availability plus implementation_version/required_phase. Tests: 9/9 targeted, 92/92 full suite, typecheck clean.

Does not close this issue: no type/wing/instinct-specific modes/inputs/outputs/limitations schema, no Enneagram-specific boundary/negative/degraded fixtures, not yet deployed. Flagging as partial Implemented + Executable evidence only.

---

## #1398 — [W3E:sacred-geometry:04] Publish capability discovery metadata

DRAFT — for human review, not yet posted.

Same generic slice, applied to Sacred Geometry: GET /engines/capabilities (ts-engines/src/server/app.ts + registry.ts, local/uncommitted on main @ ae3e2cef) reports its live-self-check availability plus implementation_version/required_phase. Tests: 9/9 targeted, 92/92 full suite, typecheck clean.

Does not close this issue: no pattern/generation-mode-specific modes/inputs/outputs/limitations schema, no Sacred-Geometry-specific boundary/negative/degraded fixtures, not yet deployed. Flagging as partial Implemented + Executable evidence only.

---

## #1428 — [W3E:sigil-forge:04] Publish capability discovery metadata

DRAFT — for human review, not yet posted.

Same generic slice, applied to Sigil Forge: GET /engines/capabilities (ts-engines/src/server/app.ts + registry.ts, local/uncommitted on main @ ae3e2cef) reports its live-self-check availability plus implementation_version/required_phase. Tests: 9/9 targeted, 92/92 full suite, typecheck clean.

Does not close this issue: no sigil-generation-specific modes/inputs/outputs/limitations schema, no Sigil-Forge-specific boundary/negative/degraded fixtures, not yet deployed. Flagging as partial Implemented + Executable evidence only.

---

## #1368 — [W3E:raaga:04] Publish capability discovery metadata

DRAFT — for human review, not yet posted.

Same generic slice, applied to Raaga: GET /engines/capabilities (ts-engines/src/server/app.ts + registry.ts, local/uncommitted on main @ ae3e2cef) reports its live-self-check availability plus implementation_version/required_phase. Tests: 9/9 targeted, 92/92 full suite, typecheck clean. Note: Raaga also has clip-resolution helpers (resolveClipDir, resolveStoredClip) imported in app.ts, but neither is touched by this capability-endpoint change.

Does not close this issue: no raga/clip-generation-specific modes/inputs/outputs/limitations schema, no Raaga-specific boundary/negative/degraded fixtures, not yet deployed. Flagging as partial Implemented + Executable evidence only.

---
