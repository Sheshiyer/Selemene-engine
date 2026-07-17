# Selemene Python Services

Python sidecar services for Selemene Engine. Two FastAPI services:

- **mediapipe_service** (port 8001) -- MediaPipe Face Mesh landmark detection
- **biofield_cv_service** (port 8002) -- Biofield spatial image analysis via OpenCV

## Setup

```bash
cd python-services
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"

# Optional: install mediapipe + opencv
pip install -e ".[mediapipe]"
```

## Run

```bash
# Run individually
mediapipe-service
biofield-cv-service

# Or directly
uvicorn mediapipe_service.main:app --port 8001 --reload
uvicorn biofield_cv_service.main:app --port 8002 --reload
```

## Test

```bash
pytest
```

## OpenAPI Specs

See `openapi/` for the contract definitions consumed by the Rust engine.

## P1 W1: Local Dev for Biofield CV Sidecar (Media/Capture Contracts)

**Focus:** Enable end-to-end testing of biofield-capture + face image lifecycle contracts (T-004) and media extensions (T-002) before full engine hardening.

**MUST REFERENCE (anti-drift):**
- `docs/plans/engine-integration/p1-w1-worker-bootstrap-packet.md`
- `docs/plans/engine-integration/p1-w1-validation-gate-checklist.md` (local dev + Sankalpa compat section)
- `docs/plans/engine-integration/EXECUTION-STATUS.md`
- `docs/plans/engine-integration/resources-and-assets.md` (dual biofield paths)
- `docs/plans/engine-integration/gaps-and-improvements.md` (schema mismatch, no e2e wiring)
- `docs/plans/engine-integration/goal-understanding.md` (two-prong, local-first + explicit consent, do not conflate server CV vs client local PIP)
- `docs/plans/engine-integration/detailed-task-list.md` (T-004, T-065)
- Full plan + `docs/engines/biofield.md`
- Sankalpa: `sankalpa/src/renderer/biofield/biofieldDomain.ts` (11 metrics + consent model)

**Run biofield-cv-service (authoritative 11-metric capture path):**

```bash
cd python-services
python -m venv .venv && source .venv/bin/activate
pip install -e ".[dev]"

# Start sidecar (default port 8002)
biofield-cv-service
# or
uvicorn biofield_cv_service.main:app --host 0.0.0.0 --port 8002 --reload
```

**Verify (standalone contract test):**
```bash
curl -s http://localhost:8002/health | jq
# POST /analyze with image (see tests/test_biofield_analyze.py for payloads)
```

**How it integrates with noesis-api (Rust):**
- `crates/noesis-api/src/biofield_client.rs` → `BiofieldClient::from_config` uses `PYTHON_BIOFIELD_URL` (default `http://localhost:8002`)
- Called from handlers/biofield.rs and capture flow for `analyze_capture`
- Bridge: `noesis_bridge::PythonServiceClient`
- Config in `crates/noesis-api/src/config.rs` (env `PYTHON_BIOFIELD_URL`, `PYTHON_BIOFIELD_TIMEOUT_MS`)
- OpenAPI contracts in `python-services/openapi/biofield-cv-service.yaml`

**For full e2e with noesis-api (contracts only):**
```bash
# Terminal 1
cd python-services && ... (start biofield-cv-service)

# Terminal 2
export PYTHON_BIOFIELD_URL=http://127.0.0.1:8002
export JWT_SECRET=dev-secret-at-least-32-characters
# (DB optional for pure engine contract smoke; see docs/runbooks/biofield-capture-analysis.md)
cargo run --bin noesis-server
# then use /api/v1/biofield/... or engine calculate paths
```

**Local-first + consent alignment (goal-understanding.md + Sankalpa):**
- This sidecar is **Prong 1 heavy CV**.
- Sankalpa (Prong 2) must do local preview + explicit opt-in (camera permission + submit) **before** sending image + `consent_token`.
- Never auto-call from renderer. See biofieldDomain.ts capture flow.
- Dual paths preserved: client local MetricsCalculator (preview) vs server CV (authoritative).

Do **not** start full engine impl (P2+). Use for contract shape validation + roundtrips only.

## Test Matrix for Contracts
- `pytest` (local shapes, 11 metrics, quality, algorithms filter)
- Cross with ts-engines for other media (raaga audio, sigil image)
- See validation checklist for required evidence.

**P1 W1 post-gate verification (local + CI):**
- Setup: `cd python-services && python -m venv .venv && source .venv/bin/activate && pip install -e ".[dev]"`
- Run: `uvicorn biofield_cv_service.main:app --port 8002` or `biofield-cv-service`
- Verify: `curl http://localhost:8002/health` → {"status":"healthy","service":"biofield-cv",...,"opencv_available":true,"numpy_available":true}
- Contract test: `pytest tests/test_biofield_analyze.py -q` (11 metrics, quality_assessment, contract_version="biofield-cv/v1")
- Sample roundtrip evidence (2026-07-17): health OK, pytest 1+ tests for required fields + 11 metrics PASS. Integrates per biofield_client.rs when PYTHON_BIOFIELD_URL set. Full harness (biofield-capture image+consent): docs/plans/engine-integration/ext-contract-harness.md (refs FROZEN + goal-understanding local-first).
- For full with TS: start both sidecars + noesis-api (mocks for DB-free contract smoke).
- Refs: p1-w1-worker-bootstrap-packet.md, p1-w1-validation-gate-checklist.md, EXECUTION-STATUS.md, detailed-task-list.md (T-004/T-065), resources-and-assets.md (dual biofield), gaps-and-improvements.md (schema), goal-understanding.md (local-first, do not conflate), P1W1-CONTRACTS-FROZEN.md (worktree), .github/workflows/test.yml (new python-sidecars job for smoke).

**Last updated:** 2026-07-17 (P1 W1 local dev bootstrap + CI baseline update #899; cites 3 extraction files)
