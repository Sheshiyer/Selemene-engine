# Selemene × Suno — 72-Melakarta Audio Library

**Document type:** Swarm Architect phase→wave→swarm plan
**Companion:** `RAAGA_ENGINE.md`, `V2_AUDIO_RICHNESS_PLAN.md`
**Predecessor:** Nādashakti V2.5 (real-time Strudel synthesis — shipped on `nadashakti/v2.5-acid-ritual` branch)
**Successor:** "Curated audio library" — pre-rendered Suno clips per melakarta, served via Selemene API

---

## 1. Discovery Summary

| Field | Decision |
|---|---|
| Planning depth | **standard** (40-50 tasks; bounded scope vs the V2.5 audio engine) |
| Delivery mode | **prototype → production** rollout (one-time bulk gen, then incremental) |
| Release model | **phased** — generate → store → serve → integrate → expand-styles |
| CI/CD expectation | basic — env-driven, no new infra |
| Quality bar | each clip auditioned manually; only approved clips reach prod |
| Team/agent topology | **solo-with-swarm** — Claude orchestrates; sub-agents per phase |
| Constraints | • Suno **Pro = 2,500 credits/mo = ~250 generations**<br>• Each gen = 10 credits, returns 2 variations<br>• 72 melakartas × 1 take = ~144 credits = **6% of monthly quota**<br>• Generation is **async, ~30-90s per call**<br>• Cookie-based auth (gcui-art wrapper requirement) |

## 2. Assumptions and Constraints

### Assumptions
- **A1.** Suno will accept prompts that describe ragas in natural language (e.g., "ambient instrumental in a slow Lydian-like Indian classical mode, sustained drone, no vocals"). It will NOT accept structured swara data.
- **A2.** Generation quality varies — we treat first generation as candidate, audition before promoting.
- **A3.** Cloudflare R2 is the storage of choice (zero egress, S3-compatible, cheap). Fallback: Selemene's existing storage if any.
- **A4.** The Suno wrapper (`gcui-art/suno-api`) deploys cleanly to Vercel/Railway alongside `noesis-web`.
- **A5.** Each clip is **30-60 seconds** — long enough to convey the raga's character, short enough to embed inline in product UX without large bandwidth.
- **A6.** A single "canonical" clip per raga lands first; multiple style variants (ambient / meditative / cinematic / acid) come in later phases.

### Constraints
- **C1.** Cannot exceed 250 gens/month without exhausting Pro quota. Plan caps Phase 2 bulk run at 144 credits (72 ragas × 2 takes).
- **C2.** Suno cookie expires periodically — must handle re-auth gracefully.
- **C3.** Each generation is non-deterministic; no two generations of the same prompt produce identical audio. Re-rolling for quality is normal.
- **C4.** **No streaming Suno → user**. Always: generate → store in R2 → serve from R2. Suno URLs expire; R2 doesn't.
- **C5.** Must not break the existing real-time Strudel V2.5 path — Suno clips are *additive*, not replacement.

## 3. Agent Ownership Model

| Concern | Primary owner | Secondary | Notes |
|---|---|---|---|
| Planning / orchestration | Claude (planner) | Human lead | This document |
| Suno wrapper deploy + auth | `suno-bridge` sub-agent | Claude | Vercel deploy of gcui-art wrapper, cookie env, smoke test |
| Prompt design (per-raga) | `prompt-engineer` sub-agent | Claude | Maps melakarta metadata → Suno-compatible natural-language prompt |
| Bulk generation runner | `gen-runner` sub-agent | Claude | Node CLI: walks 1..72, calls Suno, polls, downloads, uploads to R2 |
| Storage + DB schema | `storage-engineer` sub-agent | Claude | R2 bucket, `raga_clips` table, signed-URL generator |
| API delivery | `api-engineer` sub-agent | Claude | `/api/v1/raga/:num/clip` route in noesis-orchestrator |
| Web integration | `ui-integrator` sub-agent | Claude | Nadabrahman.tsx clip preview + fallback to V2.5 Strudel |
| Audition / QA | `audition-validator` sub-agent | Human (ear) | Listens to each candidate, marks approved/rejected/regenerate |

## 4. Phase Map

### Phase 1 — Foundations (Suno bridge + storage + schema)
- **Goal:** Suno wrapper deployed, R2 bucket live, DB table migrated, prompt template frozen.
- **Exit criteria:** Single test raga (Mayamalavagaula) generated end-to-end → stored in R2 → served via signed URL → audible in browser.
- **Waves:** 1.1 Suno bridge deploy · 1.2 Storage + DB · 1.3 Prompt template + smoke test

### Phase 2 — Bulk generation + audition
- **Goal:** All 72 melakartas have 1 approved canonical clip in R2, indexed in DB.
- **Exit criteria:** 72/72 ragas have `status=approved` in `raga_clips` table; total Suno credits used ≤ 200.
- **Waves:** 2.1 Bulk gen runner (parallel batches of 10) · 2.2 Audition harness · 2.3 Re-roll workflow

### Phase 3 — API delivery + Selemene integration
- **Goal:** Engine output (raga number) auto-resolves to a clip URL; Nadabrahman component plays it natively with V2.5 Strudel fallback.
- **Exit criteria:** Clicking ▶ on a Nadabrahman recommendation streams the Suno clip from R2 within 500ms; fallback to Strudel if R2 unreachable.
- **Waves:** 3.1 `/api/v1/raga/:num/clip` route · 3.2 Nadabrahman.tsx clip player · 3.3 Cache + edge optimization

### Phase 4 — Style variants + ongoing enrichment
- **Goal:** Each raga gains 3 additional style variants (ambient / meditative / cinematic / acid) for ~360 total clips. Library refresh cadence defined.
- **Exit criteria:** Library has ≥4 styles per raga; UI exposes style selector; quarterly refresh job documented.
- **Waves:** 4.1 Multi-style prompts · 4.2 Bulk regen for new styles · 4.3 Refresh cadence + telemetry

---

## 5. Detailed Phase 1 Wave Layout

### Wave 1.1 — Suno Bridge Deploy
**Goal:** `gcui-art/suno-api` wrapper running and reachable from `noesis-orchestrator`.

#### Swarm A — Wrapper deployment
- Owner: `suno-bridge`
- Inputs: gcui-art/suno-api repo, Suno Pro account cookie
- Outputs: deployed Vercel app at `suno-bridge.tryambakam.space`; env var `SUNO_COOKIE` set; `/api/get_limit` returns valid quota
- Validation: `curl https://suno-bridge.tryambakam.space/api/get_limit` returns `{credits_left: ~2500}`

#### Swarm B — Auth resilience
- Owner: `suno-bridge`
- Inputs: cookie expiry behavior (typically 7-30 days)
- Outputs: cookie-refresh runbook in `SUNO_AUTH_RUNBOOK.md`; alarm wired (Sentry / log) when 401 received
- Validation: simulated expired-cookie test triggers alarm

### Wave 1.2 — Storage + DB Schema
**Goal:** Where audio lives + how we look it up.

#### Swarm A — R2 bucket + lifecycle
- Owner: `storage-engineer`
- Inputs: Cloudflare account, R2 region preference
- Outputs: bucket `selemene-raga-clips` with public-via-CDN access; `R2_ACCESS_KEY_ID`/`R2_SECRET_ACCESS_KEY` in env
- Validation: `aws s3 cp test.mp3 s3://selemene-raga-clips/test/test.mp3 --endpoint-url=https://...r2.cloudflarestorage.com` succeeds + URL streams

#### Swarm B — `raga_clips` table migration
- Owner: `storage-engineer`
- Inputs: existing Selemene Postgres schema; raga_clips ER diagram
- Outputs: SQL migration `028_raga_clips.sql` adding table:
  ```sql
  CREATE TABLE raga_clips (
    id SERIAL PRIMARY KEY,
    melakarta_num SMALLINT NOT NULL CHECK (melakarta_num BETWEEN 1 AND 72),
    style VARCHAR(32) NOT NULL DEFAULT 'ambient',
    duration_sec SMALLINT NOT NULL,
    suno_song_id TEXT NOT NULL,
    suno_prompt TEXT NOT NULL,
    r2_key TEXT NOT NULL UNIQUE,
    cdn_url TEXT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'pending'
      CHECK (status IN ('pending','generated','approved','rejected','regenerate')),
    audition_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMPTZ,
    UNIQUE (melakarta_num, style)
  );
  CREATE INDEX idx_raga_clips_melakarta_status ON raga_clips(melakarta_num, status);
  ```
- Validation: migration applies cleanly + rolls back cleanly in test DB

### Wave 1.3 — Prompt Template + Smoke Test
**Goal:** Prove the end-to-end pipeline with one raga.

#### Swarm A — Prompt template per raga
- Owner: `prompt-engineer`
- Inputs: `RAAGA_ENGINE.md` (rasa, chakra, body zone, breath, time window per melakarta)
- Outputs: `lib/raga/suno-prompt.ts` with `buildSunoPrompt(melakartaNum, style)` returning natural-language prompt
- Example output for `buildSunoPrompt(15, 'ambient')`:
  > *Indian classical raga Mayamalavagaula, slow ambient instrumental, no vocals, played at 60 BPM with sustained tanpura drone, sitar lead. Mood: dawn meditation, śānta rasa. Approximately 45 seconds. No drums.*
- Validation: hand-review of prompts for 5 canonical ragas (Mayamalavagaula, Hanumatodi, Sankarabharanam, Kalyani, Kharaharapriya); each prompt ≤ 300 chars

#### Swarm B — Smoke generator
- Owner: `gen-runner`
- Inputs: prompt template, Suno bridge URL, R2 credentials
- Outputs: CLI `pnpm tsx scripts/suno-smoke.ts 15` that generates raga #15, polls until ready, downloads MP3, uploads to R2, inserts row into DB
- Validation: row appears in `raga_clips` with `status='generated'`; `cdn_url` plays in browser

---

## 6. Phase 2 Wave Layout (overview)

### Wave 2.1 — Bulk Generation Runner
- **Swarm A:** `scripts/suno-bulk-gen.ts` — walks 1..72, calls Suno in batches of 10 with 60s polling, max-credits-per-run cap
- **Swarm B:** Resumability — checkpoint after each successful gen so re-runs skip completed ragas

### Wave 2.2 — Audition Harness
- **Swarm A:** Internal admin page `/admin/audition` — lists pending clips, embedded `<audio>` players, approve/reject/regenerate buttons
- **Swarm B:** Audition log + reasoning capture (which prompts produced which outcomes)

### Wave 2.3 — Re-roll Workflow
- **Swarm A:** "Mark for regenerate" → next bulk-gen cycle picks up only `status='regenerate'` rows with prompt edits
- **Swarm B:** Cap re-rolls per raga at 3 (cost control); escalate to manual prompt revision after 3rd failure

---

## 7. Phase 3 Wave Layout (overview)

### Wave 3.1 — API Route
- **Swarm A:** `GET /api/v1/raga/:num/clip?style=ambient&duration=45` — returns signed CDN URL + metadata
- **Swarm B:** Cache-Control headers + edge cache rules (clips are immutable once approved)

### Wave 3.2 — Nadabrahman Player
- **Swarm A:** Nadabrahman.tsx — fetch clip URL on mount; render `<audio src controls>` if available
- **Swarm B:** Fallback chain: Suno clip → V2.5 Strudel ritual → V1 sine. User toggle for "live synthesis vs recording"

### Wave 3.3 — Edge Optimization
- **Swarm A:** Cloudflare cache rules (1y immutable for `/raga-clips/*`)
- **Swarm B:** Bandwidth telemetry; alert if any single raga consumes > 5GB/day

---

## 8. Full Task List (52 tasks)

> Schema per `task-schema.json`: `{id, title, area, owner, est_hours, dependencies, deliverable, acceptance, validation}`. Compacted as a table.

### Phase 1 — Foundations (18 tasks)

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| S-001 | Fork or pin gcui-art/suno-api version | infra | suno-bridge | 0.5 | — | pinned tag in `infra/suno-bridge/` | Reproducible deploy | git tag visible |
| S-002 | Deploy suno-api to Vercel | infra | suno-bridge | 1 | S-001 | live Vercel URL | `/api/get_limit` returns 200 | curl + JSON shape check |
| S-003 | Set custom domain `suno-bridge.tryambakam.space` | infra | suno-bridge | 0.5 | S-002 | DNS resolves | TLS valid | dig + browser |
| S-004 | Capture Suno cookie + set `SUNO_COOKIE` env | infra | suno-bridge | 0.5 | S-002 | Vercel env populated | quota call returns ≥ 2400 credits | logs |
| S-005 | Cookie-refresh runbook | product | suno-bridge | 1 | S-004 | `SUNO_AUTH_RUNBOOK.md` | Steps to re-extract + redeploy | manual walkthrough |
| S-006 | Sentry/log alarm on 401 | infra | suno-bridge | 1 | S-004 | wired in `noesis-orchestrator` | Test 401 triggers alarm | manual injection |
| S-007 | Provision R2 bucket `selemene-raga-clips` | infra | storage-engineer | 0.5 | — | bucket exists | `s3 ls` succeeds | curl |
| S-008 | R2 lifecycle policy (no expiry, public-via-CDN) | infra | storage-engineer | 0.5 | S-007 | policy applied | objects served via CDN URL | streams in browser |
| S-009 | R2 credentials in env | infra | storage-engineer | 0.5 | S-007 | `R2_ACCESS_KEY_ID`/`R2_SECRET_ACCESS_KEY` set | `aws s3 cp` works | manual upload |
| S-010 | Migration `028_raga_clips.sql` | data | storage-engineer | 1 | — | SQL file added | applies cleanly | `sqlx migrate run` |
| S-011 | Add `style` enum + indices | data | storage-engineer | 0.5 | S-010 | indices live | EXPLAIN plan uses idx_raga_clips_melakarta_status | psql |
| S-012 | Rust model `RagaClip` in `noesis-core` | backend | storage-engineer | 1 | S-010 | struct + diesel/sqlx mapping | Compiles + insert/select round-trips | cargo test |
| S-013 | Prompt template `buildSunoPrompt(num, style)` | frontend | prompt-engineer | 2 | — | TS module | All 72 melakartas produce non-empty prompts | unit test |
| S-014 | Hand-review 5 canonical prompts | qa | prompt-engineer | 1 | S-013 | review notes | All prompts ≤ 300 chars + describe rasa+style | sign-off |
| S-015 | Smoke gen script `suno-smoke.ts` | backend | gen-runner | 2 | S-002, S-007, S-012, S-013 | CLI executable | Generates raga #15 end-to-end | manual run |
| S-016 | Verify CDN URL plays in browser | qa | audition-validator | 0.5 | S-015 | screenshot of `<audio>` playing | Audible audio | listening test |
| S-017 | Document smoke-test in `SUNO_README.md` | product | claude | 0.5 | S-015 | docs file | Repro steps clear | external read |
| S-018 | Phase-1 sign-off | qa | audition-validator | 0.5 | S-001..S-017 | sign-off doc | All P1 contracts frozen | sign-off |

### Phase 2 — Bulk Gen + Audition (16 tasks)

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| S-019 | Bulk gen script `suno-bulk-gen.ts` | backend | gen-runner | 3 | S-015 | walks 1..72, batches of 10, polls every 5s | Resumable; respects credit cap | dry-run test |
| S-020 | Per-batch credit guard | backend | gen-runner | 1 | S-019 | hard stop at `credits_used >= 200` | Refuses to start if budget < 100 | unit test |
| S-021 | Resumability via checkpoint | backend | gen-runner | 2 | S-019 | `.suno-checkpoint.json` | Re-run skips completed ragas | manual interrupt |
| S-022 | First bulk run (72 ragas, 1 take each) | data | gen-runner | 4 | S-019, S-020, S-021 | 72 rows in `raga_clips` | All rows have `status='generated'` | DB query |
| S-023 | Audition page `/admin/audition` | frontend | ui-integrator | 3 | S-022 | Next.js admin route | Lists pending clips with `<audio>` controls + 3 buttons | manual click |
| S-024 | Approve/reject/regenerate handlers | backend | api-engineer | 2 | S-023 | API routes update `status` | DB transitions correct | test transitions |
| S-025 | Audition notes capture | frontend | ui-integrator | 1 | S-023 | textarea per clip | Saved to `audition_notes` | DB inspect |
| S-026 | First audition pass (all 72) | qa | audition-validator (HUMAN ear) | 6 | S-022..S-025 | each clip marked | `status` distribution: ≥60 approved, ≤12 regen | DB query |
| S-027 | Re-roll workflow trigger | backend | gen-runner | 1 | S-026 | bulk gen now picks up `status='regenerate'` only | Reruns only flagged | dry run |
| S-028 | Per-raga re-roll cap (3 attempts) | backend | gen-runner | 0.5 | S-027 | counter in DB | After 3rd reject → escalate | unit test |
| S-029 | Manual prompt revision UI | frontend | ui-integrator | 2 | S-028 | edit prompt inline in audition page | Saved + retried | manual test |
| S-030 | Second + third re-roll cycles | data | gen-runner | 4 | S-027..S-029 | all rejected ragas resolved | 72/72 status='approved' | DB query |
| S-031 | Cumulative credit usage report | qa | audition-validator | 0.5 | S-030 | report doc | ≤ 200 credits used | log analysis |
| S-032 | Library completeness audit | qa | audition-validator | 1 | S-030 | report | All 72 ragas have approved clip; no orphans in R2 | DB cross-check |
| S-033 | Phase-2 sign-off | qa | audition-validator | 0.5 | S-019..S-032 | sign-off | All approved | sign-off |
| S-034 | Library snapshot to repo (`raga-clips-manifest.json`) | data | storage-engineer | 0.5 | S-033 | committed manifest | 72 entries with `r2_key` + `cdn_url` | git diff |

### Phase 3 — API + Integration (12 tasks)

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| S-035 | API route `/api/v1/raga/:num/clip` | backend | api-engineer | 2 | S-033 | Rust handler in `noesis-orchestrator` | Returns `{cdn_url, duration, style, generated_at}` | curl test |
| S-036 | Style query param + 404 on unknown | backend | api-engineer | 0.5 | S-035 | `?style=ambient` filter | Defaults to `ambient`, 404 if no clip for style | tests |
| S-037 | Cache-Control headers (1y immutable) | backend | api-engineer | 0.5 | S-035 | response headers | Cloudflare caches at edge | dev tools |
| S-038 | Update Nadabrahman.tsx to fetch clip | frontend | ui-integrator | 2 | S-035 | `useRagaClip(num, style)` hook | Plays clip when present | manual test |
| S-039 | Fallback chain in Nadabrahman | frontend | ui-integrator | 2 | S-038 | Suno → V2.5 Strudel → V1 sine | If clip 404 → Strudel still plays | offline test |
| S-040 | UI toggle "live synthesis ↔ recording" | frontend | ui-integrator | 1 | S-039 | switch + label | Setting persists | localStorage test |
| S-041 | Latency budget test | qa | audition-validator | 1 | S-035..S-040 | report: time-to-first-byte | Median ≤ 500ms | DevTools network |
| S-042 | Bandwidth telemetry | infra | storage-engineer | 1 | S-035 | Cloudflare Analytics dashboard | Per-raga req/day visible | dashboard screenshot |
| S-043 | Rate-limit on API (100 req/min/IP) | backend | api-engineer | 1 | S-035 | middleware | 429 on overflow | load test |
| S-044 | Smoke: Nadabrahman result → audible clip | qa | audition-validator | 1 | S-038, S-039 | screen recording | Click ▶ → audio in <500ms | recording attached |
| S-045 | Update RAAGA_ENGINE.md with library section | product | claude | 0.5 | S-044 | docs section added | Lists endpoint, headers, fallback | docs review |
| S-046 | Phase-3 sign-off | qa | audition-validator | 0.5 | S-035..S-045 | sign-off | All P3 verified | sign-off |

### Phase 4 — Style Variants + Refresh (6 tasks)

| ID | Title | Area | Owner | Hrs | Deps | Deliverable | Acceptance | Validation |
|---|---|---|---|---|---|---|---|---|
| S-047 | Multi-style prompt templates (ambient/meditative/cinematic/acid) | frontend | prompt-engineer | 2 | S-013 | 4 prompt builders | Each style produces distinct character on test raga | listening test |
| S-048 | Bulk regen for 3 new styles × 72 = 216 clips | data | gen-runner | 8 | S-047 | DB rows for all styles | 4 styles × 72 = 288 approved clips | DB count |
| S-049 | Style selector in Nadabrahman UI | frontend | ui-integrator | 1 | S-048 | dropdown | All 4 styles selectable | manual test |
| S-050 | Quarterly refresh cadence runbook | product | claude | 0.5 | S-048 | `SUNO_REFRESH_RUNBOOK.md` | Schedule, budget, prompt-revision policy | docs read |
| S-051 | Cost telemetry (credits/month vs budget) | infra | storage-engineer | 1 | S-031 | dashboard | Credits-per-month visible vs 2500 cap | screenshot |
| S-052 | Phase-4 sign-off + library v1.0 tag | infra | claude | 0.5 | S-047..S-051 | git tag `raga-library-v1.0` | release tagged | tag visible |

**Total: 52 tasks · est. ~75 hours · 4 phases · 12 waves**

---

## 9. Dependency Rationale

- **Sequential entry:** S-001..S-017 (Phase 1) must complete before any bulk gen. The smoke test (S-015) is the contract gate — without it we don't know whether prompt → Suno → R2 → DB → playback actually works end-to-end.
- **Phase 2 is largely sequential** because of credit budget — we run small batches, audition, re-roll, repeat. Resumability (S-021) is critical so partial runs don't waste credits.
- **Phase 3 is parallel-friendly** after S-033 — API route, fallback wiring, and cache rules can all develop in parallel against the frozen library.
- **Phase 4 depends on Phase 1's prompt template generality** — if S-013 was hardcoded for one style, P4 needs a refactor. Design S-013 with style as first-class parameter from the start.
- **Lock-zone serialization:** `Nadabrahman.tsx`, `noesis-orchestrator/src/routes/raga.rs` are shared between V2.5 work and Suno work — coordinate via wave boundaries.

## 10. Verification Strategy

| Level | Gate | Evidence |
|---|---|---|
| **Task** | per-task `validation` field | curl/screenshot/DB query/listening test |
| **Wave** | wave-close runbook | All tasks closed + handoff doc to next wave |
| **Phase** | phase-exit sign-off | S-018 / S-033 / S-046 / S-052 |
| **Library** | bulk audit (S-032) | 72/72 approved + R2 cross-check + manifest committed |
| **End-to-end** | UAT (S-044) | Nadabrahman result → clip plays in <500ms with fallback verified |

**Suno-specific verifications:**
- **Quota guard (S-020):** runner refuses to start if remaining credits < 100 — prevents quota exhaustion from runaway bug
- **Audition coverage (S-026):** human listens to every clip; only approved clips reach prod
- **Fallback test (S-039):** disable R2 in dev → Strudel V2.5 should still play — proves V2.5 is not regressed by Suno layer

## 11. GitHub Sync Strategy

- **Issue mapping:** one issue per task. Labels: `phase:S1`, `wave:1.2`, `swarm:storage`, `area:backend`, etc.
- **Dependencies:** "Blocked by: #M" lines mirroring `dependencies` array.
- **Wave summary comments:** at each wave close, post recap with credit usage + audit results.
- **Branch naming:** `suno/<wave-id>/<short-slug>` per worktree-strategy playbook.
- **PR linkage:** PR title `[S-NNN] <title>`; PRs auto-close their issue.
- **Wave merge:** single integration PR per wave (`integration/suno-wave-2.1`).

## 12. Risks & Fallback Plan

| Risk | Probability | Impact | Trigger | Fallback |
|---|---|---|---|---|
| Suno cookie expires mid-bulk-gen | H | M | runner sees 401 | Pause, run cookie-refresh runbook (S-005), resume from checkpoint |
| Generation quality too low for ≥50% of ragas | M | H | First audition pass (S-026) shows >36 rejects | Iterate prompts; if persistent, fall back to V2.5 Strudel-only for affected ragas |
| Suno API rate-limits beyond 250/mo | L | M | Pro account quota changes | Throttle to 5 gens/day; complete library over 14 days |
| R2 bandwidth spike → cost blow-up | L | M | Single raga > 5GB/day | Add Cloudflare-level rate limit; investigate hot-linking |
| Suno prompt template generates copyrighted-sounding music | L | H | Audition flags "this sounds like real song X" | Add rephrasing rules; legal review of prompt approach |
| `gcui-art/suno-api` upstream breaks Suno auth | M | H | Bridge returns 5xx | Pin to known-good version; hold Suno work; rely on V2.5 Strudel until upstream fix |
| Selemene engine raga numbering changes | L | H | Hypothetical post-V2.5 refactor | DB has raga_num as the source of truth — engine query becomes a join |

## 13. Cost Model

| Phase | Suno credits | R2 storage | API egress | Total est. |
|---|---|---|---|---|
| Phase 1 (smoke) | 10 | <1 MB | negligible | $0 (within Pro) |
| Phase 2 (72 ragas × 1 take + ~30 re-rolls) | ~200 | ~50 MB | negligible | $0 (within Pro monthly 2500) |
| Phase 3 (API + UI) | 0 | unchanged | depends on traffic | depends |
| Phase 4 (3 more styles × 72) | ~720 (over 3 months) | ~200 MB | depends | well within Pro |
| **Steady state (post-launch)** | ~50/mo for refreshes | ~250 MB | scales with traffic | <$10/mo R2 + Pro sub |

Pro account at $10/mo + R2 ~$5/mo = **$15/mo total** for the entire 288-clip library + delivery.

## 14. Definition of Done

The Suno integration ships when:
- [ ] All 52 tasks closed with evidence
- [ ] All 12 waves closed with `wave-close` checklist
- [ ] Phase 4 sign-off (S-052) + `raga-library-v1.0` git tag published
- [ ] Nadabrahman.tsx UAT passes: ▶ on any of 72 recommendations → audible clip in <500ms
- [ ] Fallback chain verified (Suno 503 → Strudel V2.5 → V1 sine)
- [ ] Quarterly refresh runbook in place
- [ ] Cost telemetry shows steady-state ≤ $15/mo

---

🎵 *"The synthesizer dreams the raga. The library remembers it. The API delivers it."*
