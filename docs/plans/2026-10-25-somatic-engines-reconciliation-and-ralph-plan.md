# Selemene Engine — Somatic Engines Reconciliation and Ralph-Loop Plan

**Date:** 2026-10-25
**Branch at scan:** `codex/selemene-capability-consumer-surface` (44 ahead / 11 behind `origin/main`)
**Live production:** `https://selemene.tryambakam.space` v3.3.1, 19 engines loaded, uptime ~29 h
**Scope:** Deep pass over engine inventory, GitHub roadmap, the four somatic engines, and how the 11 sibling consumer projects actually use Selemene. Every claim below is tagged with the evidence axis it was read from: **Declared / Implemented / Executable / Integrated / Deployed / Operational** (the six-axis model from issue #896).

---

## 0. The one-paragraph truth

The repo is not "most of the way to four working somatic engines." It is most of the way on *plumbing* (contracts, API routes, SDK, health, bridge, Sankalpa surfaces) and roughly zero of the way on *somatic truth*: the parts that turn a camera frame, a voice, or an intention into something a reader can trust. Face reading has no CV in production (the MediaPipe sidecar is not deployed, so production falls back to a byte-heuristic). Server biofield is still `is_mock_data: true` always. Raaga produces real raga math but no hosted audio (clip mode defaults to `off`, suno-bridge is down). Sigil Forge produces real method guidance and can call NVIDIA for a PNG, but exposes no provenance for mock/fallback/provider and still ships a fictional OpenAPI stub. NadaBrahman is a raga *recommender* keyed on time/dosha/chakra and does not analyze audio, although `docs/ENGINES.md` says it does. The 570 open `[W3E:*]` issues are a 19×30 backlog grid, not a progress signal; zero of them have been closed.

---

## 1. Engine inventory and per-engine completion

19 runtime engine IDs (13 Rust crates, 6 TypeScript). Public docs say "17 engines"; the runtime loads 19 because `biofield-capture` and `financial-biosensor` are operational/composed IDs. That public-vs-runtime conflation is exactly what issue #896 exists to fix.

Legend: ✅ proven · 🟡 partial / caveat · ❌ absent · ❔ not verifiable from here

| Engine | Runtime | Implemented | Executable (tests) | Integrated (API/bridge/SDK) | Deployed | Operational truth | Consumer live use |
|---|---|---|---|---|---|---|---|
| panchanga | Rust | ✅ | ❔ (no local cargo) | ✅ | ✅ | 🟡 mean-longitude fallback provenance not exposed (#897) | urania, raycast, book scripts |
| human-design | Rust | ✅ | ❔ | ✅ | ✅ | ✅ (reference reports ship on it) | book scripts, witness-pipeline |
| gene-keys | Rust | ✅ | ❔ | ✅ | ✅ | 🟡 line data replaced by fixed values (#897) | book scripts |
| vimshottari | Rust | ✅ | ❔ | ✅ | ✅ (full Lahiri fix merged #1495) | ✅ | raycast, book scripts |
| numerology | Rust | ✅ | ❔ | ✅ | ✅ | ✅ | urania, raycast, book |
| biorhythm | Rust | ✅ | ❔ | ✅ | ✅ | ✅ | raycast, **mobile app (only engine it calls)** |
| vedic-clock | Rust | ✅ | ❔ | ✅ | ✅ | ✅ | raycast |
| transits | Rust | ✅ | ❔ | ✅ | ✅ | ✅ | urania, raycast, book |
| financial-biosensor | Rust (composed) | ✅ | ❔ | ✅ | ✅ | 🟡 silently renormalizes when a source engine fails (#897) | none |
| **biofield** (server, birth-data) | Rust | 🟡 Vedic analyzer exists, **main path returns mock, `is_mock_data: true` always** | ❔ | ✅ FROZEN `image_data` + consent accepted | ✅ | ❌ mock | sankalpa surface, urania artifact render |
| **biofield-capture** (camera) | Rust + Python `biofield-cv` | ✅ 11-metric CV sidecar | Python 61/61 green (prior receipt) | ✅ sessions/captures/readings routes | ✅ sidecar `ready` on Railway | 🟡 end-to-end camera→reading never proven from a consumer | sankalpa capture UI (consent-gated) |
| **face-reading** | Rust + Python `mediapipe` | 🟡 landmark hook wired to sidecar, **heuristic fallback when sidecar absent** | ❔ | ✅ `/engines/face-reading/upload` multipart | 🟡 Rust yes, **`mediapipe-face-mesh` sidecar: `unavailable` in prod; no Railway service exists** | ❌ production is byte-heuristic, labelled `is_mock_data:false` | sankalpa surface, mock-backed |
| **nadabrahman** | Rust | ✅ 72-melakarta recommender (time/dosha/rasa/chakra) | ❔ | ✅ | ✅ | 🟡 works, but **docs claim audio/voice analysis that does not exist** | **raycast dashboard (live)** |
| **raaga** | TS | ✅ melakarta math, Strudel ratios | ✅ (ts-engines 93/93 green today) | ✅ bridge + `audio_ref` FROZEN | ✅ | 🟡 **`RAAGA_CLIP_MODE` defaults `off` → `clip_url` null; suno-bridge host unreachable; `/clips/raaga/` 404** | sankalpa (with WebAudio demo fallback), urania `RaagaPlayer` |
| **sigil-forge** | TS | ✅ methods + NVIDIA PNG via ImageProvider | ✅ | ✅ `generated_image` FROZEN | ✅ NVIDIA key configured | 🟡 **no provider/fallback/mock provenance; nano-banana + kimi are placeholders; OpenAPI stub (`vector_path`) is fiction** | sankalpa (with fallback), urania artifact |
| tarot | TS | ✅ | ✅ | ✅ | ✅ | 🟡 provenance slice landed locally (PR #1487), replay semantics open | urania, raycast |
| i-ching | TS | ✅ | ✅ | ✅ | ✅ | 🟡 changing-line semantics gap (#897) | urania |
| enneagram | TS | ✅ | ✅ | ✅ | ✅ | ✅ | book scripts |
| sacred-geometry | TS | ✅ | ✅ | ✅ | ✅ | 🟡 symbolic vs generated artifact not distinguished | urania, sankalpa |

**Honest completion numbers**

| Lens | Count |
|---|---|
| Engines declared | 19 runtime / 17 public |
| Engines deployed and answering health | 19 |
| Engines with real (non-mock, non-heuristic) output in production | 14 |
| Somatic engines (biofield, face-reading, raaga/nada, sigil) with proven end-to-end somatic truth in production | **0** |
| W3E backlog issues open / closed | 570 / 0 |
| Non-W3E roadmap issues open | 17 |

---

## 2. GitHub roadmap and issues: what the corpus actually says

- **Milestones P1–P5, v2.2.0, v3.0.0, P5/P6 Noesis-Web:** all 100% closed. These represent the platform and plumbing eras. They are done and should not be reopened.
- **The W3E grid (#1285–#1484, 570 issues):** 19 engines × 30 identical slots (`01 six-axis baseline` … `30 deployment/rollback`). Created 2026-08-27 as a backlog *template*, not a tracked plan. Nothing has been closed. Treating it as "most work done" is the trap issue #896 and the next-wave plan explicitly warn against.
- **Control issues that matter (17):** #893 roadmap anchor (gates → contracts → engines), #896 six-axis authority ledger, #901 fail-closed gates, #894 contract convergence, #897 engine/media truth gaps, #913 state/auth, #908 distribution, #914 ops receipts. Plus P1-era leftovers #898, #899, #900, #902, #895 and L0-report issues #887–#890.
- **Open PRs:** #1486 → #1487 → #1491 is a three-deep stack (capabilities endpoint → Rust/Python parity + tarot provenance → consumer-reachable capability surface). #1486 is the base and is still unmerged. #1488 (phase recovery) is draft and conflicting. #1492 (ChatGPT MCP connector) is draft. #886 and #907 are stale and conflicting.
- **CI:** last green on `main` 2026-09-08 (`ebd97fe9`). Production health shows ts-engines now serves `/engines/capabilities` (live), but the Rust `/api/v1/engines/capabilities` route returns 404 and the admin variant redirects, so PR #1487/#1491 are **not** deployed.
- **The current branch** is 44 commits ahead and 11 behind `origin/main`, with a large uncommitted witness-pipeline diff. The ISA is stale (last task: "Free Selemene plugin public publication", progress 291/339, updated 2026-10-09).

---

## 3. The four somatic engines: exact blockers

### 3.1 Face Reading (most blocked)
- **Code:** `crates/engine-face-reading` calls the MediaPipe sidecar at `SELEMENE_FACE_CV_URL` (default `127.0.0.1:8001`) with a 2 s timeout and silently degrades to `heuristic_from_seed` on any error, while setting `is_mock_data: false` and backend `heuristic-image-landmark-hook`.
- **Deploy:** `python-services/railway.toml` builds **only** `Dockerfile.biofield` and explicitly excludes `mediapipe_service/**`. No Railway service named `mediapipe-service` exists. Production `/health/dependencies` reports `mediapipe-face-mesh: unavailable`. The root service has no `SELEMENE_FACE_CV_URL` variable.
- **Consumer:** Sankalpa `FaceReadingSurface.tsx` is wired but mock-backed (`is_mock_data: !hasSeed`).
- **Net:** every production face reading today is a deterministic hash of the image bytes dressed as analysis. This is the "confident output from unavailable input" anti-pattern #897 forbids.

### 3.2 Biofield Viewer
- **Two paths that must not be conflated:** (A) server `biofield` birth-data engine = mock always; (B) `biofield-capture` = camera frame → Python `biofield-cv` sidecar (deployed, `ready`) → persisted reading.
- **Consumer:** Sankalpa has the only capture UI and defaults to `https://48.tryambakam.space` (a second production host that answers identically to `selemene.tryambakam.space`; same uptime, same build). `bv-pip` (biofield.live) is a completely separate self-hosted FastAPI + WebGL stack with its own segmentation/metrics and **never talks to Selemene**.
- **Net:** the CV pipeline exists and is deployed, but no consumer has been proven to complete capture → analyze → reading → render against production. Path (A) should either be wired to `VedicBiofieldAnalyzer` or honestly labelled.

### 3.3 Nada Brahman / Raaga
- **Two engines, one brand.** `nadabrahman` (Rust) recommends ragas from prahar/dosha/rasa/chakra. `raaga` (TS) does melakarta math and optionally produces a clip. Neither analyzes incoming audio. `docs/ENGINES.md` §10 ("Analyzes vocal patterns, frequency signatures", input `audio_data`) is false and should be corrected or the capability built.
- **Audio out:** `RAAGA_CLIP_MODE` is unset in Railway (defaults `off`), so `generated_audio.clip_url` is always null. `local` mode writes PCM WAV to tmp (not durable on Railway). `service` mode points at a suno-bridge, whose host `suno-bridge.tryambakam.space` does not resolve; `SUNO_COOKIE` is set on two services, with a manual cookie-refresh runbook. R2 bucket `selemene-raga-clips` exists and R2 credentials are set on the root service, but nothing uploads to it in the engine path.
- **Consumer:** Raycast uses `nadabrahman` live (the one somatic-adjacent engine actually in daily use). Sankalpa's Raaga surface falls back to local WebAudio synthesis.

### 3.4 Sigil Forge
- **Works:** method guidance, word-elimination, witness prompts, NVIDIA FLUX PNG when `generate_image: true` (key present in prod).
- **Broken truth:** no `provenance` block (provider used, fallback, mock); `kimi.ts` and `nano-banana.ts` are placeholder/mock providers; the Rust OpenAPI stub `SigilForgeResultSchema` (`sigil_id`, `vector_path`) describes an engine that does not exist. No artifact ID, retention, or R2 persistence for generated PNGs.

---

## 4. How Selemene is used across the 11 sibling projects

| Project | Reaches Selemene via | Engines used | Somatic UI | Last commit |
|---|---|---|---|---|
| **sankalpa** (Electron) | `@selemene/engine-sdk` 0.2.0 vendored tgz + main-process gateway → `48.tryambakam.space` | biofield, raaga, sigil, face-reading, witness | **all four**, face mock-backed, raaga/sigil with local fallbacks | 2026-09-27 |
| **urania-137** (report console) | Pages Function proxy → `selemene.tryambakam.space` | all 18 | renders raaga/sigil/biofield/face **artifacts**; capture deferred to Sankalpa | 2026-09-27 |
| **noesis-raycast** | direct HTTP `selemene.tryambakam.space` | witness, biorhythm, **nadabrahman**, vimshottari, vedic-clock, tarot… | NadaBrahman live on dashboard | 2026-09-27 |
| **antahkarana** (CLI/cockpit) | `selemene-gw` worker → prod | daily-practice workflow; roster of 18 | none | 2026-10-23 |
| **somatic-canticles-mobile-app** | direct HTTP, hardcoded | biorhythm only | none | 2026-08-08 |
| **somatic-canticles-book** | Python research scripts, direct HTTP | HD, gene-keys, vimshottari, numerology, biorhythm, transits, enneagram | none | not a git repo |
| **somatic-canticles-webapp** | **none** (local biorhythm lib) | — | "Biofield Viewer" lore entry only | not a git repo |
| **bv-pip** | **none** (own FastAPI + WebGL, biofield.live) | — | the only *working* biofield capture, outside Selemene | not a git repo |
| **synchronocities-blog** | none | — | none | 2026-10-09 |
| **noesismirror-web-falseearth** | offline sync script only | — | none | not a git repo |
| **witness0s-webshore** | none | — | decorative only | not a git repo |

**Cross-cutting findings**
1. **SDK adoption is one consumer.** Only Sankalpa uses `@selemene/engine-sdk`, and it is a vendored tarball. Everyone else hardcodes `selemene.tryambakam.space` and `X-API-Key`. Issue #908 (distribution) is real.
2. **Two production hostnames** (`selemene.` and `48.tryambakam.space`) serve the same build. Sankalpa's default is the odd one out. Pick one canonical host and alias the other.
3. **The only working biofield camera pipeline in the whole portfolio is bv-pip, which bypasses Selemene.** That is the fastest path to a "live" Biofield Viewer: either point bv-pip's capture at `/api/v1/biofield/sessions/*/captures`, or acknowledge biofield.live as the viewer and have Selemene ingest its readings.
4. Four of the eleven folders are not git repositories. They cannot carry roadmap evidence until they are.

---

## 5. Reconciliation: have vs. don't have

**Have (proven):**
- 19 engines deployed, healthy, and answering through API, bridge, SDK types, health, and `/health/dependencies`.
- FROZEN media contracts (`image_data`, `audio_ref`, `generated_image`, `generated_audio`, consent) accepted end-to-end in API + bridge + SDK tests (ISA ISC-11…27).
- Capability discovery on ts-engines live; Rust/Python parity written and tested locally (PR #1487), not deployed.
- Python `biofield-cv` sidecar deployed and `ready`.
- NVIDIA image key configured in production.
- Sankalpa has consent-gated UI surfaces for all four engines.
- ts-engines test suite green (93/93, this session).

**Don't have (proven absent):**
- A deployed MediaPipe face sidecar, or any production face-reading output that is not a byte-heuristic.
- A non-mock server biofield analysis.
- Any hosted raaga audio (`clip_url` always null in production); a reachable suno-bridge.
- Provenance (provider / fallback / mock) on sigil-forge, raaga, face-reading, biofield envelopes.
- A single consumer-proven camera→reading→render round-trip against production biofield-capture.
- A Rust toolchain on this workstation (cargo absent), so Rust engine tests cannot be run locally here; CI is the only Rust evidence source.
- A truthful `docs/ENGINES.md` entry for nadabrahman.
- Any closed W3E issue; any six-axis ledger row populated per #896.

---

## 6. Plan: make the four somatic engines live

Sequenced to follow #893 (gates → contracts → engines) but narrowed to the four engines, since all gates and contracts they need already exist locally. Each step names the evidence axis it moves and the probe that proves it.

### Wave S0 — Land what is already written (1–2 days)
1. Rebase and merge the stack #1486 → #1487 → #1491 through the gated PR flow. Moves capability parity from Implemented to Deployed. Probe: `GET /api/v1/engines/capabilities` returns 200 on prod.
2. Reconcile this branch with `origin/main` (44/11 drift) via PR, not push. Commit or stash the witness-pipeline diff separately.
3. Close or supersede stale PRs #886, #907, #1488 with written reasons.
4. Decide canonical host; update Sankalpa default from `48.` to `selemene.`.

### Wave S1 — Face Reading becomes real (3–5 days)
1. Add a Railway service `mediapipe-service` from `python-services` with `Dockerfile.mediapipe` (new `railway.mediapipe.toml`, root `/python-services`). Set `SELEMENE_FACE_CV_URL` on the root service to the private Railway URL.
2. Make fallback fail-closed-or-honest: when the sidecar is unreachable, return `availability: degraded`, `provenance.fallback_used: true`, `backend: byte-heuristic`, and `is_mock_data: true`. Never label a heuristic as a real reading. (Slots `W3E:face-reading:07, 09`.)
3. Golden fixtures: one real face PNG through MediaPipe, one sidecar-down path. (Slot 27.)
4. Probe: `/health/dependencies` shows `mediapipe-face-mesh: ready`; a multipart upload returns `landmark_source: mediapipe-facemesh`.
5. Sankalpa: remove `is_mock_data: !hasSeed` shim; render the provenance badge.

### Wave S2 — Biofield Viewer goes end-to-end (3–5 days)
1. Prove capture → `/biofield/sessions` → `/captures` → sidecar → `/readings/{id}` → `biofield-capture` engine from Sankalpa against prod. Record the full receipt. (Slot 28.)
2. Wire server `biofield` (A) to `VedicBiofieldAnalyzer` when birth data is present; keep mock only when it is absent and label it. Flip the `is_mock_data` invariant in `engine.rs:309` and the doc.
3. Decide bv-pip's role: (a) fold its capture into Selemene's session API, or (b) declare biofield.live the viewer and add an ingest route. Recommendation: (a), since the sidecar's 11 metrics are already authoritative and bv-pip already POSTs frames to a `/capture` endpoint of identical shape.
4. Probe: a reading ID produced from a real frame renders in both Sankalpa and urania's `ReadingArtifact`.

### Wave S3 — Nada Brahman / Raaga produce sound (3–4 days)
1. Fix the docs lie: rewrite `docs/ENGINES.md` §10 to describe the recommender; open a separate issue if voice analysis is actually wanted.
2. Set `RAAGA_CLIP_MODE=service` or implement `r2` mode: generate WAV (existing `clip.ts`), upload to `selemene-raga-clips` with the R2 credentials already present on the root service, return a public `R2_PUBLIC_BASE_URL` clip URL. This removes the dependency on the dead suno-bridge for the first live clip. (Slots `raaga:09, 14`.)
3. Keep suno as an optional provider behind `provenance.provider`; repair or retire `suno-bridge.tryambakam.space`.
4. Probe: `generated_audio.clip_url` returns 200 audio/wav from a public URL; Sankalpa's demo fallback is no longer triggered; urania `RaagaPlayer` plays it.

### Wave S4 — Sigil Forge tells the truth (2–3 days)
1. Add `provenance { provider, fallback_used, mock, artifact_id }` to the envelope; make kimi/nano-banana report `mock: true` or be removed. (Slot `sigil-forge:07`.)
2. Delete or rewrite `SigilForgeResultSchema` in `noesis-core/src/types.rs` to match runtime; drop `vector_path`/`sigil_id`. (Slot 03.)
3. Persist generated PNGs to R2 with an artifact ID and retention note instead of inline b64 only. (Cross-cutting media requirement in #897.)
4. Probe: a `generate_image: true` call returns a provenance block naming `nvidia` and an artifact URL.

### Wave S5 — Ledger and closeout (1 day, repeats per wave)
- Populate the six-axis ledger row for each of the four engines (#896). Close the specific W3E slots touched, each with the PR, SHA, and live probe. Do not bulk-close.
- Update `docs/engines/{face-reading,biofield,raaga,sigil-forge,nadabrahman}.md` status headers.
- Refresh ISA.md to this task.

**Estimated total:** 13–20 working days single-threaded; S1–S4 are independent after S0 and can run in parallel lanes.

---

## 7. Ralph loop: how to run this autonomously

A Ralph loop (fixed prompt re-fed every iteration; the agent reads the plan, picks the next unchecked item, does it, commits, exits; a stop hook or outer shell restarts it until a completion sentinel appears) fits this work **only if the loop is fed receipts, not vibes.** The failure mode this repo has already lived through is agents closing "declared" work as "done." The design below is built to make that impossible.

### 7.1 Loop contract
- **Prompt file:** `docs/plans/2026-10-25-somatic-ralph-prompt.md` (to be written at kickoff) containing: this plan's §6 as the task list, the six-axis rule, the forbidden list, and the completion sentinel.
- **State file:** `docs/plans/somatic-ralph-state.json` — per task: `status`, `pr`, `sha`, `probe_output`, `axis_reached`. The loop may only tick a box by writing a probe output into this file.
- **One task per iteration.** The agent reads state, picks the first task whose dependencies have `axis_reached >= Integrated`, works it in a worktree branch `ralph/<wave>-<slug>`, runs the named probe, writes state, opens (never merges) a PR, exits.
- **Stop sentinel:** state file has every S1–S4 task at `Operational`, or `blocked_on_human` set on any task. Blocked tasks stop the loop; they do not get skipped.

### 7.2 Hard fences (hook-enforced, not prompt-enforced)
- No `git push --force`, no merge to `main`, no issue close, no paid provider call from a probe. These match the Global Constraints already in `GH-ISSUE-NEXT-WAVE-PLAN.md`.
- Railway service creation (S1.1), Railway variable writes, and R2 public-URL config (S3.2) are **operator-gated**: the loop prepares the exact `railway` / `wrangler` command and runbook and sets `blocked_on_human`. The operator runs it; the loop reads it back.
- **GitHub CI is disabled and is not an evidence source.** Every gate runs locally or on Railway/Cloudflare via CLI (see §7.6). A PR exists for review and merge ancestry only; its checks are ignored.

### 7.6 Verification without GitHub CI

Three evidence tiers, each with a CLI the operator already has (`railway`, `wrangler`, `bun`, `python3`, `docker`, `codex`, `temperance-batch`):

| Tier | What it proves | How it runs | Prereq on this Mac |
|---|---|---|---|
| **T1 local gate** | Implemented + Executable | `pnpm run gate` = `gate:scripts` (Python contract/migration/docker validators) + `gate:contracts` (cargo noesis-core/noesis-api + SDK tests) + `gate:verification` + `gate:ts` (bun 93/93 today) | **Rust toolchain missing.** Either `rustup` install (fastest) or run cargo in Docker (`docker-compose.yml` already defines `noesis-api`, `ts-engines`, `mediapipe-service`, `biofield-cv-service`). Docker daemon is currently down. |
| **T2 Railway staging** | Integrated + Deployed | `railway up --service <svc> --environment staging` into a new `staging` environment cloned from `production`, then `railway logs`, `railway run -- curl /health/dependencies`, deploy status via `railway service list --json` | Create the `staging` environment once (operator). Add `mediapipe-service` there first. |
| **T3 prod readback** | Operational | Read-only probes: `/health/live`, `/health/ready`, `/health/dependencies`, `/engines/capabilities`, `wrangler deployments list`, `wrangler kv/r2/d1 list`, Railway service list and var *names* | Already works (used for this report). |

Rules:
- A task reaches `Executable` only on a T1 receipt, `Deployed` only on a T2 receipt, `Operational` only on a T3 receipt. Receipts are command + exit code + trimmed output written into the state file.
- Rust changes without a working local cargo are **blocked**, not deferred to CI. Installing Rust is therefore an S0 prerequisite.
- The TypeScript and Python lanes (S3, S4, half of S1) are unblocked today: `bun test` and `pytest python-services/tests` run locally now.

### 7.7 Fan-out through Temperance

The native session babysits. Workers go to the combos per the rail contract:

| Phase | Combo | Used for |
|---|---|---|
| Observe/Think | `noesis-observe` | per-lane re-read of engine source and docs at lane start |
| Plan | `noesis-plan` | turning each wave in §6 into a lane task graph with probes |
| Build | `noesis-build` | the code edits in each lane's worktree |
| Execute (fleet) | `temperance-batch … model noesis-execute` | the four lanes S1–S4 in parallel, one worktree each |
| Verify | `noesis-verify` | adversarial re-run of every T1/T2/T3 probe by a *different* model than the one that built it |

Lock zones (serialized, one worker at a time): `crates/noesis-core/src/types.rs`, `Cargo.lock`, `python-services/railway.toml`, root `railway.toml`, Sankalpa `engineGateway.ts`. Everything else is per-lane disjoint.

After each worker returns, the babysitter announces the resolved provider and model, as the rail requires.

### 7.3 Lanes
Four lanes (S1 face, S2 biofield, S3 audio, S4 sigil) can run as four independent Ralph loops after S0, each in its own worktree and branch, because they touch disjoint crates/dirs: `engine-face-reading` + `python-services/mediapipe_service`; `engine-biofield` + `noesis-api/handlers/biofield.rs` + Sankalpa biofield dir; `ts-engines/src/engines/raaga` + `infra/suno-bridge`; `ts-engines/src/engines/sigil-forge` + `noesis-core/src/types.rs`. The one shared file is `noesis-core/src/types.rs` (S4 edits it); sequence S4's contract step before any lane touches it, or give S4 the lock.

### 7.4 Why not Ralph for everything
S0 (PR stack landing, host decision, stale-PR triage) is judgment work with external side effects. Do it by hand in one session first. Then start the loops.

### 7.5 Iteration budget
Cap each lane at 12 iterations. If a lane hits the cap without reaching `Deployed`, it stops and reports the delta rather than continuing to grind. Expect S1 to spend 2–3 iterations blocked on the human Railway step.

---

## 8. Decisions needed before execution (discussion gate)

1. **Canonical host:** `selemene.tryambakam.space` recommended; `48.` becomes an alias and Sankalpa's default changes.
2. **bv-pip's role:** fold its capture into Selemene's session API (recommended) vs. keep biofield.live separate and ingest its readings.
3. **Rust locally:** `rustup` on this Mac (recommended, one command) vs. cargo-in-Docker. Without one of these, every Rust lane is blocked by rule.
4. **Railway staging environment:** create one (recommended) vs. test directly against production. Staging is where `mediapipe-service` gets its first deploy.
5. **Raaga first audio path:** R2-direct WAV (recommended, uses credentials already present) vs. repair suno-bridge first.
6. **Nada Brahman scope:** correct the docs to "recommender" (recommended) vs. actually build voice analysis as a new engine lane.
7. **W3E handling:** close only touched slots with receipts (recommended) vs. bulk-reorganize the 570-issue grid.
8. **Lane parallelism:** four Ralph lanes at once vs. two (face + biofield first, since they share the Python sidecar deploy pattern).

---

## 9. Decisions taken (2026-10-25)

All eight section-8 recommendations accepted by the operator:

1. Canonical host `selemene.tryambakam.space`; `48.tryambakam.space` becomes an alias; Sankalpa default changes.
2. bv-pip capture folds into Selemene's `/api/v1/biofield/sessions/*` API.
3. Rust via `rustup` on this Mac.
4. A Railway `staging` environment duplicated from `production`; `mediapipe-service` deploys there first.
5. Raaga first audio path is R2-direct WAV; suno-bridge stays optional.
6. `docs/ENGINES.md` describes Nada Brahman as a recommender; voice analysis is a separate future issue.
7. W3E slots close one at a time with receipts; no bulk reorganization.
8. Four Ralph lanes after S0, with the five lock-zone files serialized.

## 10. S0 operator runbook

Operator runs these by hand; the agent reads back results. Nothing here pushes to `main`, closes an issue, or writes a secret value into a file.

### S0.1 Rust toolchain (unblocks every Rust lane)
Download the installer to a file first, inspect it, then run it (no pipe-to-shell):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
```
```bash
sh /tmp/rustup-init.sh -y --profile minimal
```
```bash
source "$HOME/.cargo/env" && rustc --version && cargo --version
```
Agent readback: `cargo test -p engine-face-reading -p engine-biofield -p engine-nadabrahman -p engine-biofield-capture --locked`, then `pnpm run gate:contracts`.

### S0.2 Docker daemon (compose-based sidecar tests)
```bash
open -a Docker
```
Agent readback: `docker info` succeeds, then `docker compose build mediapipe-service biofield-cv-service`.

### S0.3 Railway staging environment
```bash
railway link --project 11eedde4-41e6-4f51-b86b-cf77111cf592 --environment production
```
```bash
railway environment new staging --duplicate production
```
Agent readback: `railway service list --environment staging --json`; expect `Selemene-engine`, `ts-engines`, `biofield-cv-service`, `Postgres`, `Redis`, with variable names copied.

### S0.4 mediapipe-service in staging
```bash
railway add --service mediapipe-service --repo Sheshiyer/Selemene-engine --branch main --environment staging
```
In the Railway dashboard for `mediapipe-service`: root directory `/python-services`, config path `/python-services/railway.mediapipe.toml` (agent adds this file in S1 before you set it), healthcheck `/health`, private networking on.
```bash
railway variables --service Selemene-engine --environment staging --set 'SELEMENE_FACE_CV_URL=http://mediapipe-service.railway.internal:8001'
```

### S0.5 Raaga clip mode in staging
```bash
railway variables --service ts-engines --environment staging --set 'RAAGA_CLIP_MODE=local'
```
R2 mode variables follow in S3 once `clip.ts` has an `r2` mode; the five `R2_*` values already on `Selemene-engine` get mirrored to `ts-engines` then.

### S0.6 Cloudflare read-only check
```bash
wrangler whoami --profile 9d9d
```
```bash
wrangler r2 bucket list --profile 9d9d
```
Expect `selemene-raga-clips` present.

### S0.7 Git lane prep (agent does, operator approves PRs)
- Commit the witness-pipeline diff on its own branch.
- Open a review-only PR of this branch against `main`; GitHub checks are ignored.
- Comment on PRs #886, #907, #1488 as superseded; no close until operator says.
- Create worktrees `ralph/s1-face`, `ralph/s2-biofield`, `ralph/s3-audio`, `ralph/s4-sigil` from `origin/main`.
- Initialize `docs/plans/somatic-ralph-state.json`.

### S0 receipts (live log)
- 2026-10-25 S0.1 ✅ rustup stable 1.99.1 installed. T1: `cargo test -p engine-face-reading -p engine-nadabrahman -p engine-biofield -p engine-biofield-capture --locked` → biofield 65/65, biofield-capture 6/6, face-reading 46/46, nadabrahman 30/30, 0 failed.
- 2026-10-25 S0.2 ✅ Docker via Colima (`colima start`), `docker info` ok. Note: no Docker Desktop on this Mac.
- 2026-10-25 S0.3 ❌ staging delete succeeded; recreate hit a name clash. Environment does not exist. Rerun `railway environment new staging --duplicate production`.
- 2026-10-25 S0.7 ✅ witness diff parked on `wip/witness-reference-tooling` (2076eb6b); plan + state on `ralph/s0-plan`; lane worktrees `ralph/s1-face s2-biofield s3-audio s4-sigil` from origin/main cb8ce56a; PRs #886 #907 #1488 commented superseded. This branch already has PR #1491, so no new review PR opened.

### S0 exit criteria
- `cargo --version` works and the four somatic crates' tests pass locally (T1).
- `staging` answers `/health/live` on its own URL (T2).
- `mediapipe-service` exists in staging (may be red until S1 adds its toml).
- Four lane worktrees exist and the state file is initialized.

## Appendix A — Live probes taken 2026-10-25

| Probe | Result |
|---|---|
| `selemene.tryambakam.space/health/live` | ok, v3.3.1, 19 engines, 6 workflows |
| `…/health/ready` | redis ok, postgres ok, bridge available, 6 TS engines healthy |
| `…/health/dependencies` | status **unavailable**; biofield-cv ready; **mediapipe-face-mesh unavailable**; nvidia configured |
| `…/api/v1/engines/capabilities` | **404** (PR #1487 not deployed) |
| `ts-engines-production.up.railway.app/engines/capabilities` | 200, 6 capabilities (deployed since the 08-27 readback) |
| `48.tryambakam.space/health/live` | identical build, same uptime |
| `selemene-gw.…workers.dev/api/v1/health/live` | 401 (auth-protected gateway) |
| `suno-bridge.tryambakam.space/api/get_limit` | **no response** |
| `…/clips/raaga/` | 404 |
| Railway services | Selemene-engine, ts-engines, biofield-cv-service, suno-bridge, Postgres, Redis; **no mediapipe service** |
| Railway var names (root) | NVIDIA_API_KEY, PYTHON_BIOFIELD_URL, R2_* (5), SUNO_COOKIE, TS_ENGINES_URL; **no SELEMENE_FACE_CV_URL, no RAAGA_CLIP_MODE** |
| `cd ts-engines && bun test` | 93 pass, 0 fail |
| `cargo test` (4 somatic crates) | **cargo not installed on this host** |

## Appendix B — Source documents read
`ISA.md`, `_PROJECT-STATUS.md`, `docs/ENGINES.md`, `docs/engines/{face-reading,biofield,sigil-forge,nadabrahman}.md`, `docs/plans/selemene-engine/{GH-ISSUE-NEXT-WAVE-PLAN,RUNTIME-CAPABILITY-EVIDENCE}.md`, `docs/plans/engine-integration/{gaps-and-improvements,selemene-sankalpa-full-integration-swarm-plan,EXECUTION-STATUS}.md`, `docs/deployment/RAILWAY.md`, `infra/suno-bridge/SUNO_AUTH_RUNBOOK.md`, GitHub issues #893 #896 #897 and the W3E index, PRs #886–#1492, source of `engine-face-reading`, `engine-biofield`, `engine-nadabrahman`, `ts-engines/src/engines/{raaga,sigil-forge}`, `ts-engines/src/providers/*`.
