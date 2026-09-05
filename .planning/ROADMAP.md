# Selemene continuation roadmap

Original wave scope remains in `docs/plans/selemene-engine/ROADMAP.md`; phase numbers below are a GSD execution adapter. Master GitHub issue #893 remains open. None of the 570 W3E issues is declared complete by import.

## Phases

- [x] **Phase 1: Authority and infrastructure recovery** — original Wave 0; #896. (completed 2026-09-05)
- [ ] **Phase 2: Reproducible gates and dependency repair** — original Wave 1; #901.
- [ ] **Phase 3: Capability and contract closure** — original Wave 2; #894.
- [ ] **Phase 4: Engine and media truth** — original Wave 3; #897.
- [ ] **Phase 5: State auth and durability** — original Wave 4; #913.
- [ ] **Phase 6: Distribution compatibility** — original Wave 5; #908.
- [ ] **Phase 7: Deployment and operational proof** — original Wave 6; #914.

## Phase details

### Phase 1: Authority and infrastructure recovery

**Goal:** Recover the planning authority, reconcile the existing issue corpus, map current provider state and enable CodeGraph. This closes the recovery slice; original Wave 0 registry/asset exit conditions continue in Phase 2.
**Depends on:** Nothing (recovery entry point)
**Requirements:** FND-01, FND-02, FND-03, FND-04, FND-05
**Original wave:** 0
**GitHub:** [#896](https://github.com/Sheshiyer/Selemene-engine/issues/896)
**Canonical refs:** `ISA.md`, `docs/plans/selemene-engine/ROADMAP.md`, `docs/plans/selemene-engine/RECOVERY-2026-09-05.md`
**Success Criteria:**
1. Restore canonical planning files without losing preserved state.
2. Reconcile master/wave controls and all 570 engine issue IDs.
3. Record verified Railway/Cloudflare ownership and explicit inventory gaps.
4. Initialize, sync and functionally query CodeGraph.
5. Provide usable GSD context, research, plans and evidence state.

**Plans:** 2/2 plans complete

### Phase 2: Reproducible gates and dependency repair

**Goal:** Close remaining Wave 0 registry/release-receipt authority and Wave 1 fail-closed gates, repair CI and dependency findings, and prepare exact reviewable artifacts before production changes.
**Depends on:** Phase 1
**Requirements:** GATE-01, GATE-02, GATE-03, GATE-04, GATE-05, GATE-06
**Original wave:** 1
**GitHub:** [#901](https://github.com/Sheshiyer/Selemene-engine/issues/901)
**Canonical refs:** `ISA.md`, `docs/plans/selemene-engine/ROADMAP.md`, `docs/plans/selemene-engine/RECOVERY-2026-09-05.md`
**Success Criteria:**
1. Make TS formatter/lint/typecheck/tests and the required CI matrix pass.
2. Remove high production dependency findings or record an explicit unresolved compatibility gate.
3. Prove clean installs and lockfile/toolchain reproducibility including Python.
4. Validate source Railway build/watch and Cloudflare account binding configuration.
5. Complete remaining registry/release receipt authority and immutable action pins.
6. Prepare protected release and promotion decisions before any production-triggering merge.

**Plans:** 5 validated plans; source repairs and verification in progress

### Phase 3: Capability and contract closure

**Goal:** Complete native, conditional and Python capability boundaries, then prove catalogue/API/bridge/SDK/CLI contract parity.
**Depends on:** Phase 2
**Requirements:** CON-01, CON-02
**Original wave:** 2
**GitHub:** [#894](https://github.com/Sheshiyer/Selemene-engine/issues/894)
**Canonical refs:** `ISA.md`, `docs/plans/selemene-engine/ROADMAP.md`, `docs/plans/selemene-engine/RECOVERY-2026-09-05.md`
**Success Criteria:**
1. Report native/TS/Python/conditional capabilities from actual runtime state.
2. Prove schemas, errors, auth, routing and catalogue parity across repository boundaries.

**Plans:** Pending phase discussion and research

### Phase 4: Engine and media truth

**Goal:** Execute existing per-engine issues with real semantic fixtures, explicit provenance and reduced-scope decisions; use the pilot before parallel engine expansion.
**Depends on:** Phase 3
**Requirements:** ENG-01, ENG-02
**Original wave:** 3
**GitHub:** [#897](https://github.com/Sheshiyer/Selemene-engine/issues/897)
**Canonical refs:** `ISA.md`, `docs/plans/selemene-engine/ROADMAP.md`, `docs/plans/selemene-engine/RECOVERY-2026-09-05.md`
**Success Criteria:**
1. Preserve and execute the 570 existing engine issues using per-engine semantic fixtures.
2. Distinguish deterministic/generated/fallback/unavailable results through real integration paths.

**Plans:** Pending phase discussion and research

### Phase 5: State auth and durability

**Goal:** Prove migration, auth, billing, cache, invitation and durable media boundaries in disposable environments before production operations.
**Depends on:** Phase 4
**Requirements:** STATE-01, STATE-02
**Original wave:** 4
**GitHub:** [#913](https://github.com/Sheshiyer/Selemene-engine/issues/913)
**Canonical refs:** `ISA.md`, `docs/plans/selemene-engine/ROADMAP.md`, `docs/plans/selemene-engine/RECOVERY-2026-09-05.md`
**Success Criteria:**
1. Prove disposable migration, auth, billing, cache and durable storage failure paths.
2. Obtain exact authorization before production schema/data changes.

**Plans:** Pending phase discussion and research

### Phase 6: Distribution compatibility

**Goal:** Produce independently installable verified packages, SDK/CLI/TUI/admin and consumer compatibility evidence.
**Depends on:** Phase 5
**Requirements:** DIST-01, DIST-02
**Original wave:** 5
**GitHub:** [#908](https://github.com/Sheshiyer/Selemene-engine/issues/908)
**Canonical refs:** `ISA.md`, `docs/plans/selemene-engine/ROADMAP.md`, `docs/plans/selemene-engine/RECOVERY-2026-09-05.md`
**Success Criteria:**
1. Pack/install/version/test each supported distribution surface.
2. Verify Selemene-owned compatibility fixtures without editing consumer repositories.

**Plans:** Pending phase discussion and research

### Phase 7: Deployment and operational proof

**Goal:** Map verified immutable artifacts to deployed source/schema versions, repair approved topology gaps and prove monitoring, assets and rollback.
**Depends on:** Phase 6
**Requirements:** OPS-01, OPS-02, OPS-03
**Original wave:** 6
**GitHub:** [#914](https://github.com/Sheshiyer/Selemene-engine/issues/914)
**Canonical refs:** `ISA.md`, `docs/plans/selemene-engine/ROADMAP.md`, `docs/plans/selemene-engine/RECOVERY-2026-09-05.md`
**Success Criteria:**
1. Prove deployed source and schema revisions for every service.
2. Resolve DNS access and pattern-memory support decisions explicitly.
3. Exercise approved rollback, observability and asset provenance checks.

**Plans:** Pending phase discussion and research

## Progress

| Phase | Plans Complete | Status | Completed |
|---|---|---|---|
| 1. Authority and infrastructure recovery | 2/2 | Complete    | 2026-09-05 |
| 2. Reproducible gates and dependency repair | 3/5 source tasks | Executing / verifying | - |
| 3. Capability and contract closure | 0/TBD | Pending | - |
| 4. Engine and media truth | 0/TBD | Pending | - |
| 5. State auth and durability | 0/TBD | Pending | - |
| 6. Distribution compatibility | 0/TBD | Pending | - |
| 7. Deployment and operational proof | 0/TBD | Pending | - |
