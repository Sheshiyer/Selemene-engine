# Gaps & Improvements — Selemene + Sankalpa Engine Integration (Deepened Extraction)

**Purpose:** Honest inventory of what is missing, broken, stubbed, or at risk, extracted from current code/docs/prior work. This prevents drift and informs contract-first P1 work. Prioritized for the 4 focus engines.

## 1. Implementation Maturity Gaps (per focus engine)

### Biofield (BV-PIP + server)
- **Server birth analysis (`engine-biofield`)**: Still returns mock data (`is_mock_data: true` always). Vedic analyzer path exists in `engine.rs:18` but not wired into main path. Only 5 metrics; live capture uses separate 11-metric shape.
- **Live capture path**: Python sidecar exists and produces authoritative `BiofieldMetrics` (11 fields) + QualityAssessment. Sankalpa has local in-browser `MetricsCalculator` (5 composite) + PIP UI. But: no end-to-end wiring from Sankalpa capture → Selemene `engine-biofield-capture` engine → reading.
- **Schema mismatch**: OpenAPI `BiofieldResultSchema` (noesis-core) does not match either struct accurately. Renderer historically read nested vs flat.
- **Gap vs goal**: Sankalpa biofield is "local preview only" today; full Selemene-powered capture + witness not integrated.

### Face Reading
- **Core engine**: Pure stub (`generate_mock_analysis`) + `heuristic_from_seed`. No image input, no landmark detection, no real CV.
- **Models exist** (`FaceAnalysis`, zones, wisdom) but unexercised.
- **Renderer**: Draft `FaceReading.tsx` exists in retired app location; explicitly "not yet wired into any router".
- **Input contract**: Needs image ref (file path / b64 / upload id) + consent. Currently zero media handling.
- **High priority gap**: This is the most incomplete of the four.

### Raaga
- **Engine**: Strongest of the four — full algorithmic 72 melakartas, Strudel ratios, wisdom, witness prompts. TS only.
- **Gaps**:
  - Renderer port from retired `apps/noesis-web` to Sankalpa (no current surface).
  - No integration into Noesis readings or unified engine lab yet.
  - OpenAPI stub absent entirely (not in `EngineResultData` enum).
  - Bridge registration exists but end-to-end call from Sankalpa desktop not present.
  - Audio output handling (play in Electron, export clips?).

### Sigil Forge
- **Engine**: Functional for method steps + intention. NVIDIA image gen works (b64 PNG).
- **Critical mismatches**:
  - OpenAPI stub claims `vector_path` / glyph output that **does not exist** in any code. Warned in `sigil-forge.md`.
  - Only NVIDIA provider; no abstraction for nano-banana or kimi yet.
  - Prompt builder is NVIDIA-specific.
- **Gaps**:
  - No Sankalpa renderer surface (was in noesis-web).
  - No charging ritual UI, no multi-method comparison.
  - Image gen not consent-gated in a desktop context.
  - No vector/SVG output path (if ever desired).
- **Provider expansion**: "kimi code on an api" details unresolved (auth, endpoint, yantra/runic style prompts).

## 2. Integration & Wiring Gaps (Cross-Prong)

- **No unified engine client in Sankalpa**: Current biofield is 100% local. No calls to `noesis-bridge` / Selemene API for any engine.
- **Feature catalog + routing**: `features.ts` + App.tsx know about biofield local + Noesis. No engine-abstracted surfaces for raaga/sigil/face.
- **Input surfaces missing**: Camera (biofield/face), image upload (face/sigil), intention form (sigil), melakarta selector (raaga) — only partial biofield live.
- **Output consumption**: No reading persistence that mixes local analysis + backend engine results. No "add engine result to current reading".
- **Bridge / API surface for Sankalpa**: Desktop needs safe proxy (via main process?) or direct consented HTTP to Selemene. Not designed yet.
- **Media contracts**: EngineInput/Output need explicit extensions for:
  - Image refs (for face, sigil, future)
  - Audio refs (raaga)
  - Generative b64/URL outputs
  - Consent tokens / opt-in metadata

## 3. Contract & Schema Gaps

- Inaccurate or missing OpenAPI schemas in `noesis-core` for all four (biofield mismatch, sigil vector hallucination, raaga absent, face incomplete).
- Dual biofield shapes not unified in a single envelope.
- No media-aware input schema (current inputs are mostly birth data + simple forms).
- Witness prompt integration not standardized across engines.

## 4. Provider & External Gaps

- Image generation: Hard-coded NVIDIA. Need pluggable `ImageProvider` (nano-banana via runcomfy, kimi).
- CV for face: None. (MediaPipe was in old web apps but deferred per Sankalpa ISA.)
- Audio for raaga: Client Strudel works, but server-side clip generation / export / R2 storage unclear.
- "Kimi code on an api" — no concrete endpoint/auth documented in repo.

## 5. Security / Consent / Privacy Gaps

- Camera permission handling exists in Sankalpa for biofield, but must be generalized + audited for face.
- Image upload paths (face/sigil) have zero current consent model in desktop context.
- Generative outputs (sigil images) need provenance + safety review before rendering.
- No renderer-side secret leakage (good so far), but must enforce for new engine calls.

## 6. Documentation & Maintenance Gaps

- Many engine docs note "retired apps" paths — need Sankalpa-specific renderer notes.
- engine-matrix.json is good but not yet used as runtime registry source.
- Prior plan (this dir) is the first formal integration plan; no previous end-to-end integration work existed.

## 7. Prior Work That Is Good (Avoid Re-do)
- Sankalpa shell + biofield local + visual system (50 ISCs signed off).
- Raaga full data model + TS engine.
- Sigil wisdom + process logic.
- Bridge registration pattern for TS engines.
- Detailed per-engine .md files (use them as spec).
- GitHub label taxonomy + issue skeleton already created.

## 8. Improvement Opportunities (High Leverage)

1. **Contract-first P1 W1**: Freeze media-extended EngineInput/Output + per-engine result schemas. Fix the known mismatches (sigil, biofield).
2. **Provider abstraction** for images early — unblocks sigil expansion.
3. **Sankalpa "Engine Lab" or per-engine routes** that can call real engines (start with raaga as it's ready).
4. **Local-first + opt-in** pattern: keep PIP-style live previews local; escalate to backend only on explicit capture/submit.
5. **Witness prompt standardization** so every engine can feed Noesis depth readings.
6. **Use engine-matrix + bridge registration** as single source; generate types/docs from it.
7. **Adversarial validation** for generative (sigil images) and CV (future face) as first-class in test strategy.

**Extraction sources:** All gaps pulled from:
- `docs/engines/*.md` (explicit "stub", "mock", "mismatch", "not wired" warnings)
- `sankalpa/ISA.md` (what is ported vs deferred)
- `ts-engines/.../engine.ts` + utils (provider hard-coding)
- `crates/.../engine.rs` + mock.rs (real vs generated)
- `noesis-core/src/types.rs` (schema stubs)
- `docs/plans/...` previous discovery (unresolved questions)
- engine-matrix.json

**Recommendation:** Do not start implementation waves until P1 contracts + these gaps have explicit tasks + owners.
