# Deep-pass review: engines, layers, apps and agentic flows (2026-10-28)

**Scope:** `Selemene-engine` at `d7684b0` (#1508 merged), `urania-137` at
`35e8054` and `sankalpa` at `d58c5e4`.

**Method:** a read-only review of each repo, plus a full sweep of open issues
and PRs, and the receipt in `docs/plans/somatic-ralph-state.json` on
`ralph/s0-plan`.

**Evidence:** paths are `file:line` on those heads. "Inferred" marks anything
taken from receipts rather than code.

## 1. Where things stand

| Area | State |
|---|---|
| Engines | 19 registered: 13 native Rust and 6 TypeScript on Bun via the bridge. Two Python CV sidecars and one external astrology API sit behind them. |
| Provenance | Native engines carry the 7-field block since #1508, and ephemeris engines run on `swiss-files` in production. **Gaps:** i-ching, enneagram and sacred-geometry emit no block; the biofield Vedic path hard-codes `fallback_used:false`; defaulted time and timezone inputs are never signalled. |
| Production | Railway (`selemene.tryambakam.space`, plus `48.` serving the same build) with a full staging duplicate. Admin is on Vercel (`144.`). Urania v0.7.1 is on Cloudflare Pages behind Access. |
| CI | **None in any repo.** GitHub Actions was retired (#1493, Urania `35e8054`, Sankalpa `0becec6`). The gate is now local checks, which `docs/cloud/README.md` lists per repo. Urania's `scripts/release.mjs` still calls `gh workflow run release.yml`, so **the attested release path cannot run** until it is replaced. |
| Billing | Dodo is deferred. Production runs `BILLING_MODE=free` (inferred from the 3.3.1-free-mode receipt). As of this PR, nothing paid-tier is reachable (section 6). |
| Backlog | Selemene: 587 open issues, 570 of them the W3E grid (19 engines × 30 slots). Urania: 0. Sankalpa: 0 after the 2026-10-28 triage. 17 issues are labelled `cloud-ready`. |

## 2. The 19 engines

**Native Rust.** Registration: `crates/noesis-orchestrator/src/lib.rs:308-348`.
`biofield-capture` is registered only when a database is connected
(`noesis-api/src/lib.rs:4473`).

| Engine | Computes | Truth | Provenance and fallback | Open gap |
|---|---|---|---|---|
| panchanga | tithi, nakshatra, yoga, karana, vara | real | `native-rust`, with a fallback flag | noon default when time is missing (`engine-panchanga/src/lib.rs:519`); `/ghati/current` is unauthenticated |
| human-design | bodygraph (Swiss Ephemeris) | real | `swiss-ephemeris`, Moshier flagged | — |
| gene-keys | 64 keys from HD | derived | inherits HD | line data is fixed values (#1074) |
| vimshottari | 120-year dasha, full Lahiri | real | `…:swiss-files` in production | noon and UTC defaults (`engine.rs:459,464`) |
| transits | current transits | real | Moshier flagged | — |
| numerology | Pythagorean, Chaldean | real | never falls back | — |
| biorhythm | 23/28/33-day cycles | real | never falls back | — |
| vedic-clock | organ clock, doshas | rule-based | never falls back | an unknown timezone becomes UTC |
| nadabrahman | raga recommender | real | never falls back | — |
| biofield | chakra metrics | real (Vedic) or mock | `vedic-ephemeris` / `mock` | **Vedic path is always `fallback_used:false`** (`engine-biofield/src/engine.rs:173`) |
| face-reading | MediaPipe 468-landmark CV | real in production since 10-25 | `mediapipe-face-cv` or a fallback | upload route strips provenance; no real-face fixture |
| biofield-capture | reads back a persisted capture | database-backed | always false | old `stub-metrics/v1` rows are served as not-fallback |
| financial-biosensor | composes HD, GK, vimshottari, transits, biorhythm | composed | never falls back | a failed source silently lowers coverage (#1052) |

**TypeScript** (`ts-engines/src/engines/*`, through `noesis-bridge/src/lib.rs:307-357`):

| Engine | Truth | Provenance |
|---|---|---|
| tarot | seeded shuffle | block present |
| i-ching | seeded three-coin cast | **none** (#1341); changing lines are ignored (#1344) |
| enneagram | assessment | **none** (#1311) |
| sacred-geometry | symbolic | **none** (#1401) |
| sigil-forge | NVIDIA NIM FLUX images | block present; Kimi and nano-banana are honest placeholders |
| raaga | melakarta maths plus WAV synth, hosted on R2 | block present (`selemene-wav-synth`) |

**Behind the engines:**

- **Python sidecars:** `biofield_cv_service` (port 8002) and `mediapipe_service` (8001, `mediapipe==0.10.18`), both built from `Dockerfile.sidecar`.
- **External API:** FreeAstrologyAPI, through `noesis-vedic-api` and `noesis-western-api`, for `/charts/vedic`.

**Documentation drift:** `docs/ENGINES.md:6` and `tasks/todo.md` still say "17
engines". The uncounted two are `biofield-capture` and `financial-biosensor`.

## 3. How the layers stack

```
                     ┌───────────── consumers ──────────────┐
  Urania (CF Pages)  │ Folio · WitnessRun L0–L5 · Chat      │  Sankalpa (Electron)
  /api/selemene/* ───┤ narrator · Jev quality routing       │  SDK 0.2.0, 5 focus engines
                     └──────────────────┬───────────────────┘
                                        │ X-API-Key (shared server key)
  ┌─────────────────────────── Selemene API (Rust, Railway) ──────────────────────────┐
  │ auth: JWT · API key (engine scopes, enforce) · CF Access                          │
  │ L-c  /assets/generate ── witness-pipeline (12-pass, 6 rubric gates; L0/L4 modes) │
  │ L-b  /witness/interpret ─ noesis-witness LLM dyad → rule-based fallback          │
  │ L-a  witness_prompt on every engine output (rule-based)                           │
  │      workflows (6) ── synthesis per workflow ── orchestrator (phase gate 0–5)     │
  │      13 native engines · bridge → 6 TS engines · sidecars · FreeAstrologyAPI      │
  └───────────────────────────────────────────────────────────────────────────────────┘
```

- **Phase gating:** each engine's `required_phase()` (0–5) is compared with the
  user's `consciousness_level`, which comes from progression, not billing
  (`noesis-orchestrator/src/lib.rs:18,142,211`).
- **Workflows** (`workflow/registry.rs`):

  | Workflow | Phase | Engines |
  |---|---|---|
  | birth-blueprint | 0 | 5 engines |
  | daily-practice | 0 | 5 engines |
  | decision-support | 1 | 5 engines |
  | creative-expression | 1 | 5 engines |
  | self-inquiry | 2 | 4 engines |
  | full-spectrum | 3 | all |

  Within a workflow, engines run in parallel (`join_all`), then that workflow's
  synthesis step runs.
- **Witness:** a non-prescriptive mirror with two voices, Aletheios and Pichet,
  plus a synthesis.
  - **"L0–L5" are report depth levels**, not service layers. Only the L0 and L4
    mode documents exist; L1–L3 and L5 are planned.
  - The old witness-agents runtime on `48.` is retired to reference material
    (`docs/plans/2026-07-01-witness-agents-retirement-minimal-arch-design.md`).
    The witness/reading-repair split still lives only on
    `codex/selemene-capability-consumer-surface`.
- **Urania's interpretation stack** (`functions/lib/agents/*`):
  - The browser sends only IDs, the question and the depth. The server builds a
    ContextPacketV1 with `contextPacketHash` and `factLockHash`.
  - L0 returns the stored text verbatim. L1 and deeper call one of four route
    agents (pattern→aletheios, embodied→pichet, synthesis, navigate), which must
    return strict-JSON claims with evidence IDs.
  - Results are stored immutably in `reading_interpretations`.
  - **Gap:** `consciousnessLevel: 1` is hard-coded (`orchestration-client.ts:120`,
    `[[path]].ts:698`), so L2–L5 see the same evidence as L1.
    `layer-policy.ts:41` defines the richer layers, but nothing feeds them yet.
- **Jev** (TypeSafe `jev-latest`): validates engine output quality, and maps
  rich/adequate/partial/empty to the routes and depth a reading may use. This
  is advisory only.

## 4. AI and agentic chat flows

| Flow | Where | Provider path | Shape |
|---|---|---|---|
| Onboarding narrator | Urania `functions/lib/chat/turn.ts:129-195` | `NARRATOR_LLM_URL` → `workers/llm-proxy`, with Nebius and NVIDIA directly as fallbacks | one completion per turn; advisory `record_intake` tool, with the state machine as the only writer; SSE replays a persisted turn |
| WitnessRun interpretation | Urania `functions/lib/agents/model.ts` | same proxy | strict JSON, no tools, no streaming |
| Witness dyad | Selemene `noesis-witness/src/llm.rs` | per-tier models over NVIDIA NIM, then OpenRouter | system and user prompts per role |
| L0 report pipeline | `packages/witness-pipeline` | `multi-llm.ts` chain | 12 passes with rubric gates |
| Sigil images | ts-engines | NVIDIA FLUX | image generation |
| Embeddings | Urania patterns search | Workers AI `bge-small-en-v1.5` | Vectorize, 384-dim |

None of these is a multi-step agent loop. Each request is a single
completion. The ISA's AgentScope lab (M6, ISC-695..706) is decided but not
built.

**OmniRoute** (this PR):
- It is now first in the proxy, the Rust dyad and the pipeline. Each stays
  inert until both `OMNIROUTE_API_KEY` and `OMNIROUTE_MODEL` are set, and each
  keeps its existing fallbacks.
- Urania needs no code change, because it already calls the proxy.
- Cloud Claude Code sessions use OmniRoute as their external rail for
  ISC-659 patch provenance and cross-vendor audits. Their own reasoning model
  is host-managed.

## 5. The apps

**Urania 137:**
- **Front end:** React on a hash router. Routes: home graph, node runs, chat,
  threshold onboarding, Folio library, relationship readings, settings,
  admin-data.
- **Server:** one Pages Functions catch-all (`functions/api/[[path]].ts`) with
  D1 (11 migrations), R2 (53-reading corpus), Vectorize and Workers AI. There is
  no KV.
- **Auth:** Cloudflare Access JWT, with roles viewer, support, admin and
  platform-admin.
- **Selemene access:** 13 engines, 6 workflows and 3 witness modes. Not used:
  biofield, biofield-capture, face-reading, nadabrahman and raaga.
- **Milestones:**

  | Milestone | Status |
  |---|---|
  | M1 | done |
  | M2 (landing) | deployed, polish ongoing |
  | M3 (relationship UI) | API done, no UI caller |
  | M4 (723 import) | 53 of 723 imported |
  | M5 (Vectorize loop) | one-shot only |
  | M6 (AgentScope) | not built |
  | M7 | not started |

- **Open ISA residues:** ISC-143, 145, 146 and 189.
- **Bundle:** about 135 B under its budget.

**Sankalpa:**
- **Architecture:** a hardened Electron shell with 6 IPC channels. It reaches 5
  focus engines through `@selemene/engine-sdk` 0.2.0, with credentials only in
  main-process environment variables.
- **Biofield:** a webcam flow that computes metrics locally and uploads only
  after consent.
- **LLM:** none of its own. Witness text comes from templates.
- **Tests:** 145 pass and typecheck is clean, but the renderer budget fails
  (262,527 B against 256,000 B).
- **The R4 tombstone decision is still unrecorded,** although `main` keeps
  taking merges.

## 6. Paid tier: what this PR changes

**Before this PR:**
- Dodo checkout and portal already returned 503 outside `dodo` mode.
- Migration 038's admin toggle could only choose `free` or `disabled`.
- Urania never had a paid tier.

**What still acted paid-tier:**
- The **50-call monthly cap**, which returned **402 "Upgrade at /pricing"**
  whatever `BILLING_MODE` was (`noesis-api/src/lib.rs` `enforce_free_tier_quota`).
- The `dodo_reconcile` cron.
- Sankalpa's "Plan: Explorer" copy.

**Changed:**

| Change | Where |
|---|---|
| The quota applies only when the effective mode is `dodo`; the release-mode check avoids a database read on every call | `free_tier_quota_applies` |
| `dodo_reconcile` exits 0 and reports `skipped` unless `BILLING_MODE=dodo` | the reconcile binary |
| Sankalpa shows "Plan: Free" and "Tier: free", with a test that fails on any checkout or upgrade copy | Sankalpa account screen and tests |

All of it is reversible by setting `BILLING_MODE=dodo`.

**Still tier-shaped by design:**
- Per-minute and per-day rate limits (`middleware.rs:456-490`; note that
  `basic` is unmapped).
- Witness rule-based depth for premium tiers.
- Model choice per tier.

These are not upsells. Revisit them when the token-based entitlement model
lands.

**Leftovers to clean up when convenient:**
- The orphan `k8s/base/dodo-reconcile-cronjob.yaml`, which is not in
  `kustomization.yaml`.
- The admin-web billing pages.
- The SDK's `getBillingBalance`, checkout and portal methods.
- The 19 slot-21 W3E issues.

## 7. Seams for the Web3 layer (input to the planning pass)

There is no chain, wallet or NFT code in any repo. `contracts/v1/` holds JSON
Schemas for the API, not smart contracts. Planning should start from these
facts.

**Anchors that already exist:**

| Repo | Anchor |
|---|---|
| Urania | `src/lib/readings/canonical.ts:30-46` `canonicalReadingBytes`, a stable content digest per reading |
| Urania | `reading_interpretations` rows carry `context_packet_hash` and `fact_lock_hash` |
| Urania | synastry rows carry request and response sha256 |
| Selemene | `reading_import_runs.manifest_sha256` and `reading_sources.content_sha256` (migration 036) |
| Selemene | `readings.input_hash` |
| Selemene | the 7-field provenance schema `contracts/v1/schemas/provenance.schema.json` |

**Missing pieces:**
- **Identity:** wallets are absent everywhere. Urania users are keyed by the CF
  Access `sub` or an email hash. Selemene `users` has no public-key column.
  Sankalpa has no identity at all.
- **A mintable reading ID:** Selemene persists readings fire-and-forget
  (`tokio::spawn`, `lib.rs:3142`), and the calculate response does not return a
  reading ID (inferred). Minting needs a synchronous persist-and-return.
- **Canonical hashing:** `serde_json` output is not canonical, so a hash of a
  result needs RFC 8785 (JCS).
- **Entitlements:** the tier and billing model cannot express token
  entitlements. The SPL token is a new entitlement source, not a Dodo
  replacement.

**Hard constraints:**
- Birth data, reading bodies and interpretation text never go on chain. Mint
  only salted or committed digests.
- Sankalpa's consent tokens are client-made random strings and would need to
  become server-signed receipts.
- The obvious mint points are the Urania Folio (an authoritative D1 row plus R2
  body) and the Sankalpa Sigil export (a PNG plus guidance JSON).

## 8. Top risks

1. **No CI anywhere,** and the Urania release script depends on a deleted
   workflow. Every claim now rests on manual receipts. Restate #901 around
   local gates, or the cloud-session checks.
2. **Production Postgres has no point-in-time recovery or volume backups**
   (#914). The 10-26 `pg_dump` is the only snapshot.
3. **Provenance holes,** listed in Wave B of the queue, and silent input
   defaults.
4. **Thread-local Swiss Ephemeris state.** It is fixed per thread, but the
   negative-control test that proves the fix never finished.
5. **The shared `SELEMENE_API_KEY`** used by Urania still needs rotating, and a
   staging key with `engines:*` access expires 2026-10-28.
6. **Railway config-as-code is deprecated after 2026-12-01.**
7. **Interpretation depth L2–L5 is nominal** (section 3).
8. **PR #1488 is huge and stale.** It holds the 19-row registry that #896
   needs, so that decision blocks W3E closure.
9. **Bundle budgets** are effectively at their limits in Urania and over in
   Sankalpa.
10. **Documentation drift:** 17 vs 19 engines, a stale roadmap timeline, and
    `_PROJECT-STATUS.md` from July.

## 9. Next moves

- **Your decisions:**
  - the #1488 path;
  - Sankalpa R4;
  - restating #901 for a world without Actions;
  - backups on #914;
  - the OmniRoute key, model and deploy (`docs/cloud/README.md` §2).
- **Cloud sessions:** the 17 `cloud-ready` issues in the order of
  `docs/cloud/README.md` §5, one draft PR each.
- **Next planning pass:** Web3. Wallet identity, synchronous reading IDs with
  JCS digests, a soulbound mint of digests on Solana, and an SPL token as an
  entitlement source, all built on the seams in section 7.
