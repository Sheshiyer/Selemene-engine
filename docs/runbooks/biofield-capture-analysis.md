# Biofield Capture Analysis Runbook

Date: 2026-04-09
Status: **ARCHIVED** — `biofield-web` was retired on 2026-06-30. The API endpoints and Python sidecar workflow below are still valid, but the web app consumer has moved to [Sankalpa](../../sankalpa/).

## Purpose

This runbook covers the local verification path for the biofield vertical slice.

It proves that:

- the main Noesis API is reachable
- the private Python biofield sidecar is reachable
- a user can register and log in
- a biofield session can be created and closed
- a capture can be uploaded through Noesis to the Python sidecar
- a persisted reading appears in history
- the reading detail route returns the persisted analysis payload and artifact metadata
- a stored source artifact can be reprocessed into a new reading
- a baseline can be created from selected readings and listed back to the user
- reading detail can return baseline comparison deltas
- an export bundle can be created, persisted, and downloaded locally

## Expected Local Ports

- `noesis-server`: `8080`
- `biofield_cv_service`: `8002`
- `postgres`: `5432`

If local `8080` is occupied, pick another API port and pass it consistently to the API process, web env, and smoke command.

## Required Environment

### PostgreSQL

Provide a running database compatible with:

- `postgresql://noesis_user:noesis_password@localhost:5432/noesis`

The repo’s documented local path is:

```bash
docker-compose up -d postgres
```

Then verify:

```bash
PGPASSWORD=noesis_password psql 'postgresql://noesis_user@localhost:5432/noesis' -c 'select 1'
```

### Noesis API

- `JWT_SECRET`
- `DATABASE_URL`
- `PYTHON_BIOFIELD_URL`
  - default: `http://localhost:8002`
- `PYTHON_BIOFIELD_TIMEOUT_MS`
  - default: `10000`
- optional `BIOFIELD_ARTIFACTS_DIR`
  - default: `./.runtime/biofield-artifacts`

### Biofield Web

- `NEXT_PUBLIC_API_BASE_URL`
  - for local dev: `http://127.0.0.1:8080`

## Local Bootstrap

### 1. Start PostgreSQL

```bash
cd /Volumes/madara/2026/witnessos/Selemene-engine
docker-compose up -d postgres
```

### 2. Start the Python sidecar

```bash
cd python-services
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
uvicorn biofield_cv_service.main:app --host 0.0.0.0 --port 8002 --reload
```

### 3. Start the Rust API

```bash
cd /Volumes/madara/2026/witnessos/Selemene-engine
export JWT_SECRET="dev-secret-at-least-32-characters"
export DATABASE_URL="postgresql://noesis_user:noesis_password@localhost:5432/noesis"
export PYTHON_BIOFIELD_URL="http://127.0.0.1:8002"
export PYTHON_BIOFIELD_TIMEOUT_MS="10000"
# optional override if you want artifact files elsewhere
# export BIOFIELD_ARTIFACTS_DIR="/absolute/path/to/biofield-artifacts"
cargo run --bin noesis-server
```

### 4. Start the desktop client

```bash
cd /Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/sankalpa
export VITE_BIOFIELD_API_URL="http://127.0.0.1:8080"
npm run electron:dev
```

## Smoke Check

Run:

```bash
BIOFIELD_WEB_URL=http://127.0.0.1:3002 \
API_BASE_URL=http://127.0.0.1:8080 \
PYTHON_BIOFIELD_URL=http://127.0.0.1:8002 \
bash /Volumes/madara/2026/witnessos/Selemene-engine/scripts/smoke_biofield_web.sh
```

### Optional Overrides

The smoke script can self-provision a user and generate a capture automatically.

Override only if you need fixed values:

```bash
BIOFIELD_EMAIL=qa@example.com \
BIOFIELD_PASSWORD=SmokePass123 \
BIOFIELD_FULL_NAME="QA Smoke" \
BIOFIELD_CAPTURE_IMAGE=/absolute/path/to/image.png \
BIOFIELD_WEB_URL=http://127.0.0.1:3002 \
API_BASE_URL=http://127.0.0.1:8080 \
PYTHON_BIOFIELD_URL=http://127.0.0.1:8002 \
bash /Volumes/madara/2026/witnessos/Selemene-engine/scripts/smoke_biofield_web.sh
```

## Expected Results

The BF3 smoke path should prove all of the following:

- `/login`, `/viewer`, and `/history` return `200`
- `/health/live` returns `200`
- Python sidecar `/health` returns `200`
- auth register/login succeeds
- `POST /api/v1/biofield/sessions` returns `201`
- `POST /api/v1/biofield/sessions/:session_id/captures` returns `201`
- `GET /api/v1/biofield/readings` returns the new reading
- `GET /api/v1/biofield/readings/:reading_id` returns detail for the new reading
- `POST /api/v1/biofield/readings/:reading_id/reprocess` returns a new reading
- `GET /api/v1/biofield/readings/:reprocessed_reading_id` resolves
- `POST /api/v1/biofield/baselines` creates a baseline from the original and reprocessed readings
- `GET /api/v1/biofield/baselines` returns the created baseline
- `GET /api/v1/biofield/readings/:reading_id?baseline_id=...` returns comparison deltas
- `POST /api/v1/biofield/exports` returns persisted export metadata and a bundle payload
- the returned export `storage_path` exists under the configured biofield artifact root
- `POST /api/v1/biofield/sessions/:session_id/close` returns `closed`

## What This Runbook Proves

This BF3 runbook proves the current storage-backed slice beyond BF2:

- authenticated session lifecycle
- capture ingestion through Noesis
- Python-backed analysis response
- reading persistence
- artifact metadata linkage
- real source-artifact file persistence
- reprocess from stored source artifact
- baseline creation and list
- reading-vs-baseline comparison deltas
- synchronous persisted JSON export bundles
- history list
- reading detail

## What It Does Not Prove Yet

This runbook does not yet prove later-phase scope such as:

- richer baseline comparison visualization beyond the thin reading-detail surface
- alternate export formats
- downstream synthesis into non-biofield product surfaces
- real computer-vision upgrade beyond the current Python stub
