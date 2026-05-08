# Noesis Platform — Full Drift Remediation + Expansion Plan
**Generated:** 2026-05-08  
**Status:** Active  
**Scope:** Drift audit → Doc currency → Noesis Web Stage 2 → Integrations → Doc CI

---

## Discovery Summary
- **Planning depth:** Deeply detailed  
- **Delivery mode:** Production  
- **CI/CD:** Production-grade (doc currency gate on release)  
- **Release model:** Phased rollout (P1 → P2 → P3 → P4)  
- **Quality bar:** All endpoints documented, all consumers covered, no 404 dark space  
- **Team topology:** Solo  
- **External constraints:** gematria naming (3437 = Noesis), naming must be sacred-value consistent

## System Map (as of v3.3.0)
- **API:** selemene.tryambakam.space (Rust/Axum, 16 engines)
- **Admin Web:** 144.tryambakam.space (Next.js, Vercel)
- **Witness Agents:** 48.tryambakam.space (TS sidecar, Railway)
- **Noesis Web (target):** noesis.tryambakam.space + 3437.tryambakam.space (DARK — needs deploy)
- **Consumers:** Raycast extension, TUI, SDK (noesis-sdk-ts), OpenClaw, Hermes (new), Website

## Assumptions
1. biofield-web becomes the Noesis Web viewer (all 16 engines) — domain rename, not code rewrite
2. 3437 is gematria of "Noesis" — intentional sacred naming artifact, not a port
3. Hermes agent integration mirrors OpenClaw pattern (API key auth, REST calls)
4. TUI is an existing consumer project (separate repo or planned), needs API contract docs
5. TOI = Tool of Intent = specific integration surface in the ecosystem
6. Doc-as-code system uses OpenAPI diff from `/api/openapi.json` as source of truth
7. All docs use "Noesis" product name; "Selemene Engine" is the repo/internal name

---

## Phase Map

| Phase | Name | Wave Count | Task Count | Goal |
|-------|------|-----------|-----------|------|
| P1 | Drift Remediation | 4 | ~24 | Fix all doc drift from v3.0→v3.3 |
| P2 | Noesis Web Stage 2 | 5 | ~31 | Deploy full 16-engine viewer |
| P3 | Integrations | 3 | ~18 | Hermes + SDK + TUI/Raycast/TOI |
| P4 | Doc CI/CD | 2 | ~8 | Automated doc currency on release |

---

## Phase 1 — Drift Remediation

### Wave 1: API Surface Audit
**Goal:** Know exactly what has drifted before touching docs.

#### Swarm 1A — Route Audit
- [ ] P1-W1-S1A-01: Audit all registered API routes vs docs coverage
- [ ] P1-W1-S1A-02: Audit noesis-sdk-ts vs current API surface (missing methods)
- [ ] P1-W1-S1A-03: Audit llms.txt accuracy vs live API + naming
- [ ] P1-W1-S1A-04: Audit OpenAPI spec completeness vs registered routes

#### Swarm 1B — Drift Report
- [ ] P1-W1-S1B-05: Create drift report (master artifact listing all gaps)

### Wave 2: Core Doc Updates
**Goal:** Surgically update primary doc files.

#### Swarm 2A — API Docs
- [ ] P1-W2-S2A-06: Update docs/API_QUICKSTART.md for v3.3.0
- [ ] P1-W2-S2A-07: Update docs/api/README.md — add billing/admin sections
- [ ] P1-W2-S2A-08: Write docs/api/billing.md — complete billing endpoint reference
- [ ] P1-W2-S2A-09: Write docs/api/admin-analytics.md — usage/summary, analytics/summary
- [ ] P1-W2-S2A-10: Write docs/api/admin-reconcile.md — reconciliation endpoints

#### Swarm 2B — Identity + Release
- [ ] P1-W2-S2B-11: Update llms.txt — Noesis branding, v3.3.0, all 16 engines current
- [ ] P1-W2-S2B-12: Update docs/ENGINES.md — verify all 16 engines documented
- [ ] P1-W2-S2B-13: Update CHANGELOG.md and RELEASE_NOTES.md for v3.3.0

### Wave 3: Consumer Doc Updates
**Goal:** Every downstream consumer has accurate integration docs.

#### Swarm 3A — Agent/AI Consumers
- [ ] P1-W3-S3A-14: Update docs/api/OPENCLAW_INTEGRATION.md for v3.3.0
- [ ] P1-W3-S3A-15: Update docs/api/MCP_INTEGRATION.md for v3.3.0
- [ ] P1-W3-S3A-16: Update docs/api/LLM_AGENT_GUIDE.md for v3.3.0
- [ ] P1-W3-S3A-17: Update bridges/README.md — all consumers listed, Hermes placeholder

#### Swarm 3B — Portal Docs
- [ ] P1-W3-S3B-18: Update portal/docs/authentication.md
- [ ] P1-W3-S3B-19: Update portal/docs/sdk-quickstarts.md
- [ ] P1-W3-S3B-20: Verify all portal/docs/engines/*.md are accurate (16 engines)
- [ ] P1-W3-S3B-21: Update portal/docs/workflows/*.md — verify 6 workflows current

### Wave 4: Verification
**Goal:** Prove all docs are accurate against live production.

#### Swarm 4A — Link Validation
- [ ] P1-W4-S4A-22: Validate all doc endpoint links against live production API
- [ ] P1-W4-S4A-23: Verify OpenAPI /api/openapi.json includes all admin/billing endpoints

#### Swarm 4B — Naming Consistency
- [ ] P1-W4-S4B-24: Sweep all docs for Selemene→Noesis naming inconsistencies

---

## Phase 2 — Noesis Web Stage 2

### Wave 1: Architecture
- [ ] P2-W1-S1-25: Decide app rename strategy (biofield-web → noesis-web)
- [ ] P2-W1-S1-26: Design URL routing + deploy config for noesis.tryambakam.space
- [ ] P2-W1-S1-27: Design URL routing + deploy config for 3437.tryambakam.space
- [ ] P2-W1-S2-28: Design multi-engine data model and page structure
- [ ] P2-W1-S2-29: Design navigation/UX for 16-engine viewer

### Wave 2: Birth-Data Engine Views
- [ ] P2-W2-S1-30: Build Panchanga engine view component
- [ ] P2-W2-S1-31: Build Human Design bodygraph view
- [ ] P2-W2-S1-32: Build Gene Keys activation sequence view
- [ ] P2-W2-S1-33: Build Vimshottari dasha timeline view
- [ ] P2-W2-S1-34: Build Numerology numbers view
- [ ] P2-W2-S2-35: Build Biorhythm cycles view
- [ ] P2-W2-S2-36: Build Vedic Clock / TCM organ clock view
- [ ] P2-W2-S2-37: Build Transits planetary view
- [ ] P2-W2-S2-38: Build Biofield chakra view (expand existing PIP)

### Wave 3: TS Bridge Engine Views
- [ ] P2-W3-S1-39: Build Tarot spread view component
- [ ] P2-W3-S1-40: Build I-Ching hexagram view
- [ ] P2-W3-S1-41: Build Enneagram type/growth view
- [ ] P2-W3-S1-42: Build Sacred Geometry pattern view
- [ ] P2-W3-S2-43: Build Sigil Forge intent view
- [ ] P2-W3-S2-44: Build Nadabrahman sound frequency view
- [ ] P2-W3-S2-45: Build Face Reading view

### Wave 4: Full Spectrum + Witness Layer
- [ ] P2-W4-S1-46: Build full-spectrum workflow view (all engines unified)
- [ ] P2-W4-S1-47: Integrate witness prompt display layer
- [ ] P2-W4-S1-48: Build reading history (integrate /readings API)
- [ ] P2-W4-S2-49: Build birth data input form (shared across all engines)
- [ ] P2-W4-S2-50: Auth integration (API key + JWT for Noesis Web)

### Wave 5: Deployment
- [ ] P2-W5-S1-51: Deploy Noesis Web to Railway/Vercel
- [ ] P2-W5-S1-52: Map noesis.tryambakam.space → Noesis Web
- [ ] P2-W5-S1-53: Map 3437.tryambakam.space → same deployment (alias)
- [ ] P2-W5-S2-54: Health check + smoke test on both URLs
- [ ] P2-W5-S2-55: Performance check — all 16 engines load under 3s

---

## Phase 3 — Integrations

### Wave 1: Hermes Agent Bridge
- [ ] P3-W1-S1-56: Research Hermes agent API surface + auth model
- [ ] P3-W1-S1-57: Design Hermes bridge adapter (mirror OpenClaw pattern)
- [ ] P3-W1-S1-58: Implement bridges/hermes/ adapter
- [ ] P3-W1-S2-59: Write docs/api/HERMES_INTEGRATION.md
- [ ] P3-W1-S2-60: Test Hermes bridge against live production API
- [ ] P3-W1-S2-61: Add Hermes to bridges/README.md and llms.txt

### Wave 2: SDK Update
- [ ] P3-W2-S1-62: Audit noesis-sdk-ts vs current API — identify all gaps
- [ ] P3-W2-S1-63: Add billing balance + subscription endpoints to SDK
- [ ] P3-W2-S1-64: Add admin usage/summary + analytics to SDK
- [ ] P3-W2-S1-65: Add biofield readings + exports to SDK
- [ ] P3-W2-S2-66: Update SDK types for reading-object contract (v3.3.0 witness layer)
- [ ] P3-W2-S2-67: Bump SDK version to 3.3.0, update SDK changelog
- [ ] P3-W2-S2-68: Publish updated SDK

### Wave 3: TUI + Raycast + TOI
- [ ] P3-W3-S1-69: Document TUI consumer API contract
- [ ] P3-W3-S1-70: Write docs/api/TUI_INTEGRATION.md
- [ ] P3-W3-S1-71: Update Raycast extension docs for v3.3.0
- [ ] P3-W3-S2-72: Write docs/api/TOI_INTEGRATION.md
- [ ] P3-W3-S2-73: Verify all consumer docs link to correct production URLs

---

## Phase 4 — Doc CI/CD System

### Wave 1: Tooling
- [ ] P4-W1-S1-74: Build OpenAPI diff script (compare spec versions)
- [ ] P4-W1-S1-75: Build doc-coverage checker (flag undocumented endpoints)
- [ ] P4-W1-S2-76: Write GitHub Action: on-release doc currency check
- [ ] P4-W1-S2-77: Write doc-patch template system (surgical insert)

### Wave 2: Release Gates
- [ ] P4-W2-S1-78: Add doc-currency gate to release checklist template
- [ ] P4-W2-S1-79: Automate CHANGELOG entry from git log + OpenAPI diff
- [ ] P4-W2-S2-80: Write runbook: doc-currency check + patch procedure
- [ ] P4-W2-S2-81: Create tasks/lessons.md — all session lessons encoded

---

## Dependency Rationale
- P1 must complete before P3 (docs must be updated before consumer-specific layers)
- P2-W1 (architecture) must complete before P2-W2/W3 (engine views)
- P3-W1 (Hermes research) must complete before implementation
- P4 can run in parallel with P2/P3 (tooling build is independent)
- P1-W1 (audit) must complete before P1-W2 (doc updates)

## Verification Strategy
- Every Wave 4 in Phase 1 is pure verification
- Phase 2 deployment has dedicated health check tasks (54, 55)
- Phase 3 Hermes has dedicated test task (60)
- Phase 4 tooling proves itself (doc coverage check runs on its own output)

