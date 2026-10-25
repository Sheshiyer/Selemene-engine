# Somatic lanes S2–S4: plan, then fan out

## Context

On 2026-10-25 the reconciliation found zero somatic engines producing real output in production. Lane S1 fixed face reading (PR #1496: MediaPipe sidecar, provenance, honest health) and lane S5 added engine permission scopes (PR #1497, audit mode in production, enforce in staging). Three somatic engines remain. Exploration of the actual code corrected one earlier belief: the server biofield engine already runs the Vedic analyzer whenever birth data is present; only the no-birth-data and capture-shaped inputs are mock.

What is actually wrong today:

- **Biofield**: `POST /biofield/sessions/{id}/captures` never checks consent server-side; the engine's "capture" branch returns mock metrics labelled as capture; no consumer has proven camera → session → capture → reading → `biofield-capture` engine against Selemene; Sankalpa loads readings from the wrong route.
- **Raaga / Nada Brahman**: `clip_url` is always null in production (`RAAGA_CLIP_MODE` defaults off; local mode writes to an ephemeral disk; the R2 uploader uses a non-SigV4 auth that R2 rejects; suno-bridge is dead and the `raga_clips` table has 0 approved rows); no provenance; `docs/ENGINES.md` describes a voice-analysis engine that does not exist.
- **Sigil Forge**: no provenance; the kimi provider and, without a token, nano-banana return a hard-coded 1x1 PNG as if it were a real image; `finish_reason` is dropped; the guidance note claims an image was included even on error; `parameters.provider` is ignored; the Rust OpenAPI schema (`sigil_id`, `vector_path`) describes an engine that never existed.

Outcome: each engine reaches the Operational axis in staging and is promoted to production with receipts, via the same flow S1 and S5 used (local gate → Railway staging built from the lane branch → PR → merge → production readback). No GitHub Actions.

## Execution model (shared)

- **One session per lane**, in the existing worktrees, each rebased onto `origin/main` (0f39cd07) at kickoff: `.worktrees/ralph-s2-biofield` → `ralph/s2-biofield`, `.worktrees/ralph-s3-audio` → `ralph/s3-audio`, `.worktrees/ralph-s4-sigil` → `ralph/s4-sigil`.
- **This session babysits**: reviews each PR, drives staging with the Railway MCP (staged source switch + `accept-deploy`, probes with the staging keys), merges one PR per lane pinned to the verified head, confirms production, and writes receipts to `docs/plans/somatic-ralph-state.json` on `ralph/s0-plan`. Lane sessions commit and push to their branch and open the PR; they do not merge or touch Railway.
- **Verification tiers**: T1 local (`cargo test --locked`, `bun test`, pytest inside the sidecar image), T2 staging (service built from the lane branch; probes below), T3 production readback after merge.
- **Lock zones** (one owner each): `crates/noesis-core/src/types.rs` and `contract.rs` → S4; `python-services/**` → S2 (none planned); `ts-engines/src/utils/r2.ts` and `ts-engines/package.json` → S3; `ts-engines/src/types/engine.ts` → additive optional fields only, S4 merges before S3. `Cargo.lock` changes only if a lane adds a Rust dependency (none planned).
- **Consumer repos**: no lane edits Sankalpa, Urania, or Raycast. All consumer changes are collected into a follow-up lane **S6** after S2–S4 merge (see end).
- **Staging keys** exist: `ralph-s1-staging` has biofield, raaga, sigil-forge, nadabrahman, panchanga, numerology, face-reading scopes; staging runs `ENGINE_SCOPE_MODE=enforce`, which also proves scopes on these routes.
- **Operator-gated** (by hand, at promotion): any production Railway variable write. Staging variable writes are done by this session at kickoff.
- **Provenance rule**: every output carries the seven-field contract block (`runtime_kind, implementation_version, cached, fallback_used, backend_id?, provider_id?, confidence?`), exactly those fields, because the API parser in `crates/noesis-api/src/lib.rs:1531-1557` drops the block on any unknown field. `fallback_used: true` whenever the result is mock, placeholder, skipped, or errored.

## Lane S2 — Biofield Viewer end-to-end

**Decisions**
- Enforce consent on capture server-side by reusing `require_consent_scope` (`crates/noesis-api/src/lib.rs:1823`, make it `pub(crate)`) with scope `biofield-capture`, read from the `capture_metadata.consent` multipart field that the SDK and Sankalpa already send. Missing or wrong scope returns the same 422 `CONSENT_REQUIRED` face reading returns, before the sidecar is called.
- Make the engine's capture branch honest: in `crates/engine-biofield/src/engine.rs` `analyze` (l.119-172), if `options.biofield_capture.reading_id` is present, resolve the real persisted reading (the lookup `crates/engine-biofield-capture` already does) and return its metrics with `fallback_used: false`; otherwise keep mock but set `is_mock_data: true`, a `mock_reason`, and `fallback_used: true`. Add the provenance block on all three paths (`backend_id`: `vedic-ephemeris` | `biofield-capture` | `mock`).
- bv-pip stays separate. Its response contract (composite scores, processed images, PIP settings) is a different product; an adapter would freeze a second schema. Record as one line in `docs/engines/biofield.md`.
- Commit a small synthetic test image (`tests/fixtures/images/biofield-portrait-256.jpg`, under 60 KB, downscaled from the Higgsfield portrait, origin noted in a README) so the capture probe is reproducible.

**Changes**: `crates/noesis-api/src/lib.rs` (visibility), `crates/noesis-api/src/handlers/biofield.rs` `create_capture` (~l.1659, parse `capture_metadata` early, consent check), `crates/engine-biofield/src/engine.rs` (split `analyze_vedic` / `analyze_capture` / `analyze_mock`, provenance), `docs/engines/biofield.md`.

**Reuse**: `require_consent_scope`; test helpers `build_multipart_body`, `create_session_for_user` in `crates/noesis-api/tests/biofield_capture_proxy.rs`; the `sufficient_quality` gate (l.1000-1036) unchanged.

**Tests**: capture_proxy: `capture_rejects_missing_consent`, `capture_rejects_wrong_scope`, `capture_accepts_frozen_consent` (and add consent to the shared helper so the existing 28 keep passing); engine-biofield unit tests for provenance on all three paths and `fallback_used` on mock; refresh `tests/fixtures/expected_outputs/biofield/*.json` if the provenance block changes snapshots.

**Staging probes**: create session → upload the fixture with consent → 200 `reading_id`; same upload without consent → 422; `GET /api/v1/biofield/readings/{id}` → 200 with metrics; calculate `biofield` with `options.biofield_capture.reading_id` → `provenance.fallback_used: false`; without it → `true`.

**Risks**: the sidecar's quality gate may reject a portrait (it expects a full-body frame); if so the probe uses a frame the operator supplies and the fixture is replaced. Session tests need the staging database (migrated).

## Lane S3 — Raaga / Nada Brahman hosted audio

**Decisions**
- Add an `r2` clip mode: render the existing WAV (`renderRaagaClipWav` in `ts-engines/src/engines/raaga/clip.ts`), upload to R2 with proper SigV4 via **aws4fetch** (small, fetch and WebCrypto based, runs under Bun; the AWS SDK has Node-stream problems under Bun). Key `raaga/<sha256>.wav`, `HEAD` before `PUT` so repeats are idempotent, `Content-Type: audio/wav`, immutable cache headers, URL `${R2_PUBLIC_BASE_URL}/${key}`.
- S3 **owns** the shared helper `ts-engines/src/utils/r2.ts` (`putIfAbsent(key, body, contentType) → {url, existed}`, `publicUrl(key)`) and the `aws4fetch` dependency; it rewrites `ts-engines/scripts/r2-upload.ts` on top of it with content type inferred from extension.
- Provenance on every raaga output: `runtime_kind: typescript`, `provider_id: selemene-wav-synth`, `fallback_used: true` whenever a clip was requested and not delivered, `confidence: 1` for the deterministic ratios.
- Local mode returns an absolute URL when a public base is known; otherwise documented as ephemeral.
- Suno path: leave the pipeline alone; fix only the two trivial mismatches in `crates/noesis-api/src/handlers/raga.rs` and a new migration (`style` CHECK gains `traditional`; `duration_sec` typed `Option<i16>`), or defer with a note if the cast is unclear.
- Docs: `docs/ENGINES.md` §10 and `docs/engines/nadabrahman.md` describe the recommender truthfully; `docs/engines/raaga.md` gains the clip modes.

**Changes**: `clip.ts` (pure `renderWav()`, `r2` branch), `raaga/engine.ts` (l.236-285, attach provenance the way tarot does at `tarot/engine.ts:119-125`), `utils/r2.ts` (new), `scripts/r2-upload.ts`, `package.json`, docs, optional `raga.rs` + migration.

**Tests**: `engine.test.ts` r2 mode with fetch mocked (HEAD 404 → PUT; HEAD 200 → no PUT; PUT 500 → status failed and `fallback_used`); provenance present for all modes; absolute-URL test for local mode; `r2.ts` unit tests; a bridge test in `crates/noesis-bridge` that raaga provenance survives `__contract_provenance`.

**Infra**: this session copies `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_PUBLIC_BASE_URL`, `R2_RAGA_CLIPS_BUCKET` from the root service to `ts-engines` in **staging** at kickoff and sets `RAAGA_CLIP_MODE=r2` there; production copies are operator-gated at promotion. Confirm the bucket serves public reads at `R2_PUBLIC_BASE_URL`.

**Staging probes**: calculate raaga with `request_clip: true` and consent → `generated_audio.clip_url` is `https://…`, `metadata.clip.mode: r2`; `curl -I` the URL → 200 `audio/wav`; repeat call → same URL; without consent → null and `fallback_used: true`.

**Risks**: R2 public-access configuration; SigV4 clock skew; secrets in logs (redact).

## Lane S4 — Sigil Forge provenance

**Decisions**
- Provenance on every output: `provider_id` is the provider that actually produced the image (`nvidia` | `nano-banana` | `kimi` | `mock`), or omitted when no image was generated; `fallback_used: true` for placeholder, mock, error, or generation not requested; `confidence` 0.9 on real success, 0 on fallback.
- Honest providers: `kimi.ts` and `nano-banana.ts` report `isAvailable()` false without credentials and return `{error, mock: true}` on failure, never the 1x1 PNG as a success. `MockImageProvider` stays for tests. The engine omits top-level `generated_image` whenever `fallback_used`, carries `finish_reason` into `generated_image.metadata`, fixes the guidance-note bug (l.306-310), and honours `parameters.provider` only when that provider is configured and available (otherwise default plus a metadata note). Remove the duplicate `NvidiaImageProvider` in `utils/nvidia-image.ts`.
- Replace the fictional `SigilForgeResultSchema` (`crates/noesis-core/src/types.rs:361-369`) with one matching runtime (`intention`, `method`, `svg_preview`, `generated_image?`, `provenance?`), keeping the type name so `openapi_schema_tests.rs:27` passes.
- Artifact persistence to R2 is **not** in this lane; it becomes a follow-up iteration after S3's `r2.ts` lands, opt-in via `SIGIL_ARTIFACT_MODE`.

**Changes**: `ts-engines/src/engines/sigil-forge/engine.ts` (provider registry replacing l.45-53, provenance at l.284-358, guidance fix), `providers/kimi.ts`, `providers/nano-banana.ts`, `utils/nvidia-image.ts`, `ts-engines/src/types/engine.ts` (additive `finish_reason?`), `crates/noesis-core/src/types.rs`, `docs/engines/sigil-forge.md`.

**Tests**: `engine.test.ts`: mocked nvidia success → `provider_id: nvidia`, `fallback_used: false`; no key → `fallback_used: true`, no top-level image; `MockImageProvider` → `provider_id: mock`; update l.263 (nano-banana availability without token is now false). Rust: bridge test that a seven-field sigil provenance passes through intact; `openapi_schema_tests` still green; `p4_media_engines_tests` and `fixture_snapshot_tests` still green.

**Staging probes**: calculate sigil-forge with `generate_image: true` (NVIDIA key present on staging ts-engines) → `provenance.provider_id: nvidia`, `fallback_used: false`, image present; with `parameters.provider: kimi` → falls back to nvidia with a note; no-key case proven locally.

**Risks**: consumers that rendered the kimi placeholder as success now see a declared failure (this is the point); the schema change ripples into OpenAPI clients.

## Ordering and merge sequence

1. All three lanes start together after kickoff (rebase, staging R2 vars).
2. **S4 merges first** (owns `types.rs`; additive `engine.ts` types).
3. **S3 rebases on S4** and merges second (owns `r2.ts`; additive `engine.ts` types).
4. **S2 merges whenever ready**; it touches neither `types.rs` nor `contract.rs`.
5. Production promotion per lane: merge → production rebuild → T3 readback with the same probes against `selemene.tryambakam.space` (production key minted and revoked per proof, as before). S3's production step includes the operator R2 variable copy.
6. **Follow-up lane S6 (consumers)**, one session after the three merge: Sankalpa `loadReading` route fix and default host `selemene.tryambakam.space`, provenance badges on the sigil and face surfaces, `<audio>` player when `clip_url` is absolute, and the optional sigil artifact mode once R2 is available.

## Verification (end to end)

- T1 per lane as listed; all three must also keep `cargo test -p noesis-api --locked --test integration_tests --test p4_media_engines_tests --test fixture_snapshot_tests` and `cd ts-engines && bun run typecheck && bun test` green.
- T2: staging `Selemene-engine` (S2) and `ts-engines` (S3, S4) built from the lane branch via staged source switch and `accept-deploy`; probes above with `ralph-s1-staging`; receipts written to the state file.
- T3: after each merge, production deployment on the merge commit, health ready, and the same probes once with a scoped production key.
- Done means, for each engine, a production response whose provenance says the real path ran: biofield `backend_id: biofield-capture` from a real capture, raaga an `https` clip that plays, sigil `provider_id: nvidia` with a real image, and every fallback path declaring itself.
