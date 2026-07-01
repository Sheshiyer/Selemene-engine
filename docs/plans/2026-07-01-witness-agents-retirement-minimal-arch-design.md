# Witness Agents Retirement — Minimal Architectural Change Design

**Date:** 2026-07-01  
**Status:** Design (validated approach)  
**Goal:** Make Selemene the canonical service for rich witness dyads (Aletheios/Pichet) and premium asset generation while retiring witness-agents as a live runtime — with **zero** breaking changes to public contracts.

---

## Problem

witness-agents contains the authoritative personas (Aletheios + Pichet), multi-pass integrated reading modes, premium asset factory, and generation scripts. Only a small slice was ported (packages/witness-pipeline + Rust routing/dyad). 

Selemene already has:
- 16 engines + rule-based witness prompts
- `/api/v1/witness/interpret` (fixed 6-field dyad response)
- `noesis-orchestrator` workflows + synthesis
- Partial `witness-pipeline` (mode parser, orchestrator skeleton, factory/audit)

Consumers (SDK, TUI, API, workflows, reading persistence) depend on stable shapes. Any architectural change that mutates public envelopes, engine output contracts, or `/witness/interpret` risks breakage across the platform.

We need the rich personas and premium asset capability inside Selemene **without** touching existing surfaces.

---

## Vision

Selemene becomes the single source of truth for:
- Consciousness mirrors (engines + witness prompts)
- Witness Dyad interpretation (Aletheios + Pichet voices + synthesis)
- Premium integrated readings and asset generation

witness-agents is reduced to a read-only reference repository (personas, .premium-assets, scripts, historical batches). All live service traffic uses Selemene endpoints and the existing SDK surface.

All existing public contracts remain byte-for-byte compatible.

---

## Out of Scope (for this phase)

- Changing the shape of `POST /api/v1/witness/interpret` or `WitnessInterpretResponse`
- Changing any engine output envelope (`witness_prompt`, `witness_prompts`, result structure)
- Changing workflow result shapes or reading persistence models
- Removing or renaming existing routes
- Full inference stack migration (NVIDIA/OpenRouter adapters) — only personas + orchestration shape
- NotebookLM / slide-deck / PDF rendering pipelines (additive later)

---

## Principles

- **Public contract is sacred.** No existing client or integration may need to change.
- **Additive surfaces only.** New capability arrives via new paths or internal enrichment.
- **Selemene is canonical.** After this work, the runtime truth lives here.
- **witness-agents becomes source-of-record for artifacts, not runtime.**
- **Thin mirror adapter.** Any translation lives in witness-agents (compat shim) or in a new thin ingest layer — never mutates Selemene's core.
- **YAGNI on public API.** Do not add fields "just in case" to existing responses.

---

## Constraints

- `/api/v1/witness/interpret` response must remain exactly `{ aletheios, pichet, synthesis, witness_question, engines_used, llm_powered }`.
- Engine output contract (per-engine `witness_prompt` / `witness_prompts[]`) is frozen for v3.x.
- SDK `NoesisClient.interpretWitness(...)` signature and return type are frozen.
- All 16 engines continue to emit rule-based witness prompts as before.
- Tiered behavior (free/standard/premium/enterprise) must continue to work.
- Existing noesis-orchestrator workflows and synthesis must be unaffected.

---

## Goal

1. Rich Aletheios + Pichet personas (voice, tone, anti-dependency contract) are the **only** implementation used for dyad generation inside Selemene.
2. Premium asset generation (source-pack, mode-doc driven passes, register bands, chain audit) is available as a first-class Selemene capability.
3. All of the above is reachable via additive routes / SDK methods while **zero** existing public contracts change.
4. witness-agents can be safely retired from live service use (kept as reference + asset source).

---

## Design

### 1. Public Contract Invariants (Non-Negotiable)

- `POST /api/v1/witness/interpret` shape is **frozen**.
- `EngineOutput` envelope (`witness_prompt`, `witness_prompts?`, result) is **frozen**.
- `WorkflowResult`, `Reading`, `ReadingDetail` shapes are **frozen**.
- SDK `interpretWitness`, `calculate`, `workflow` signatures are **frozen**.
- Any new capability must arrive via new routes (e.g. `/api/v1/assets/*`) or purely internal enrichment.

**Verification:** Existing integration tests and contract tests must continue to pass with identical payloads.

### 2. Internal Enrichment — noesis-witness (Zero Public Surface Change)

Location: `crates/noesis-witness/src/`

- Replace / extend the current `ALETHEIOS_SYSTEM` and `PICHET_SYSTEM` prompts (and rule-based fallbacks in `rule_based_dyad`) with the full persona language from witness-agents (`agents/aletheios/IDENTITY.md`, `agents/pichet/IDENTITY.md`).
- Preserve exact output contract: still return `WitnessDyadLlm { aletheios, pichet, synthesis, witness_question, engines_used }`.
- Keep existing routing (`aletheios-primary` / `pichet-primary` / `dyad-synthesis`) and tier gating.
- Keep the existing `build_context_message` structure; only the system prompts and fallback text become richer.
- Anti-dependency language is added to the personas (user should need the mirror less over time).

**Impact:** Only internal strings and logic. No new fields, no new routes.

**Verification:** 
- All existing `/witness/interpret` tests still pass with identical response shape.
- Manual spot checks show richer, more precise language while staying non-prescriptive.

### 3. Additive Surface — Premium Assets (New Capability)

New optional surface (not required for existing clients):

- `POST /api/v1/assets/generate` (or behind feature flag `/api/v1/assets/premium`)
  - Input: birth_data + mode (e.g. "integrated-reading", "birth-blueprint", etc.) + consciousness_level + options
  - Output: source-pack style structure (passes, assembled text, audit metadata) — defined in `witness-pipeline` types.
- SDK addition (additive method only): `generatePremiumAsset(...)` — does not affect any existing method.
- Implementation: `packages/witness-pipeline` becomes the canonical engine here.
  - Use the already-ported `IntegratedReadingOrchestrator`, mode parser, factory, and audit.
  - LLM is injected (same pattern as today).
- This route is **additive**. Existing `/witness/interpret` and workflows are untouched.

**Why separate?** Premium asset generation is a heavier, multi-pass artifact flow. It does not belong in the lightweight dyad endpoint.

**Verification:**
- New endpoint has its own contract tests.
- Existing dyad and workflow tests are unchanged.

### 4. witness-pipeline as Canonical (Internal + New Surface)

Current state: partially ported, lives in `packages/witness-pipeline`.

Actions:
- Promote it from "ported slice" to the **official** location for:
  - Mode document parsing + register bands
  - Integrated reading orchestration (multi-pass)
  - Source-pack factory
  - Asset chain audit
- Wire the new `/assets/generate` (or equivalent) through this package.
- Keep it decoupled from the core engine calculation path. It consumes engine results (via existing fetcher or internal calls).

No changes to how engines themselves are called.

### 5. Thin Mirror Adapter + Ingest Layer (witness-agents → Selemene)

witness-agents will **not** call Selemene in a way that changes Selemene.

Instead:
- A thin "mirror" layer (small scripts or a new small package) can be added **inside witness-agents** later for backward compatibility with any old direct consumers.
- The real flow is: assets and persona definitions are **ingested** into Selemene (or referenced at build/docs time).
- `.premium-assets/` and historical batches stay in witness-agents as the long-term source of truth for that data.
- Selemene does not take a dependency on the full witness-agents runtime.

This keeps the arrow of authority pointing to Selemene for live service while preserving the original artifacts.

### 6. SDK / TUI / Client Impact

- `NoesisClient` gains **only additive** methods (e.g. `generatePremiumAsset`).
- Existing `interpretWitness`, `calculate`, `workflow`, auth, billing, readings methods are **untouched**.
- TUI / CLI clients that only use the current surfaces see zero change.
- Clients that want the new power opt into the new methods.

### 7. Retirement Path for witness-agents

Phase 1 (this work):
- Rich personas live in Selemene (noesis-witness).
- Premium asset generation lives in Selemene (witness-pipeline + new additive route).
- All live traffic uses Selemene.

Phase 2 (later, after validation):
- witness-agents is marked read-only for runtime use.
- CI / docs / notebooks can still reference it for asset sources and history.
- Optional thin compat shim (in witness-agents) can translate old call sites to new Selemene endpoints if needed by external parties.

Selemene is now the service. witness-agents is the archive + asset vault.

---

## Data Flow (Unchanged + New)

**Unchanged paths:**
- Engine calculation → `witness_prompt` (rule-based) → returned in envelope.
- `/witness/interpret` → parallel engines → `WitnessContext` → `interpret_with_llm` (now using enriched personas) → same 6-field response.
- Workflows → synthesis + workflow witness prompts (no change).

**New additive path (example):**
- Client calls new `generatePremiumAsset(...)` or `POST /api/v1/assets/generate`
- Selemene fetches required engine results (or accepts pre-fetched)
- `witness-pipeline` IntegratedReadingOrchestrator runs mode passes
- Factory + audit produce source-pack + metadata
- Result returned in new shape (additive contract)

---

## Error Handling

- All existing error paths for `/witness/interpret` and engines remain identical.
- New asset generation surface has its own error taxonomy (e.g. `AssetGenerationError`, mode not found, LLM failure during pass, chain audit failure).
- Partial results policy for assets mirrors the existing "partial context is acceptable" stance in witness dyad.
- No change to how engine failures are tolerated inside the dyad path.

---

## Testing & Verification Strategy

**Invariants (must never regress):**
- All current `/witness/interpret` response shape tests.
- All engine `witness_prompt` presence + non-emptiness tests.
- All workflow + reading persistence contract tests.
- SDK contract tests for existing methods.

**New coverage:**
- Persona quality tests (language tone, anti-dependency, reference to specific data points) — can be golden-file or LLM-as-judge style.
- Mode parser + orchestrator tests (already partially exist in witness-pipeline).
- Factory + audit tests (already partially exist).
- New asset generation endpoint contract + error tests.
- End-to-end smoke for additive path using real mode docs.

**Retirement readiness check:**
- Run full test matrix with witness-agents services turned off (where possible) to prove Selemene is sufficient.
- Document any remaining hard dependencies on witness-agents (should be only asset source data and historical scripts).

---

## Decisions (to be appended during implementation)

- 2026-07-01: Chose additive surfaces + internal enrichment only. No public contract mutation.
- (future) Decision on exact route name for premium assets (`/assets/generate`, `/assets/premium`, etc.).
- (future) Decision on whether to expose a "mode" parameter on the existing dyad for light integrated readings or keep that strictly in the new asset path.

---

## Verification Checklist (for completion)

- [ ] `/witness/interpret` response shape tests still pass identically.
- [ ] Engine envelopes unchanged.
- [ ] SDK existing methods unchanged.
- [ ] New additive asset path has contract + integration tests.
- [ ] Enriched Aletheios/Pichet personas are the only ones used in dyad generation.
- [ ] `witness-pipeline` is wired as the canonical multi-pass / premium path.
- [ ] Documentation updated (ENGINES.md, API docs, SDK README) to position Selemene as canonical.
- [ ] witness-agents repo updated to note "runtime retired; reference + assets only".

---

## Next Steps (after design validation)

1. Write this design to `docs/plans/2026-07-01-witness-agents-retirement-minimal-arch-design.md`.
2. Create isolated worktree (using superpowers:using-git-worktrees).
3. Use writing-plans / task-master to break into implementation tasks.
4. Implement in this order:
   a. Internal persona enrichment in noesis-witness (no public change).
   b. Promote + complete witness-pipeline as canonical.
   c. Add additive premium asset route + SDK method.
   d. Update docs + retirement notes.
   e. Verification run with old surfaces frozen.

This keeps the architecture change boring, safe, and reversible while achieving the strategic goal.
