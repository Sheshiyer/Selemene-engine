# Biofield Capture Analysis Runbook

Date: 2026-04-09
Status: Phase 1 complete local verification path

## Purpose

This runbook covers the local verification path for the finished Phase 1 biofield web slice.

It proves that:

- the standalone `biofield-web` app boots
- the main Noesis API is reachable
- the private Python biofield sidecar is reachable
- a user can register and log in
- a biofield session can be created and closed
- a capture can be uploaded through Noesis to the Python sidecar
- a persisted reading appears in history
- the reading detail route returns the persisted analysis payload and artifact metadata

## Expected Local Ports

- `biofield-web`: `3002`
- `noesis-server`: `8080`
- `biofield_cv_service`: `8002`
- `postgres`: `5432`

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
cargo run --bin noesis-server
```

### 4. Start the web app

```bash
cd /Volumes/madara/2026/witnessos/Selemene-engine/apps/biofield-web
export NEXT_PUBLIC_API_BASE_URL="http://127.0.0.1:8080"
npm run dev
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

The smoke script can self-provision a user and generate a tiny PNG automatically.

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

The smoke path should prove all of the following:

- `/login`, `/viewer`, and `/history` return `200`
- `/health/live` returns `200`
- Python sidecar `/health` returns `200`
- auth register/login succeeds
- `POST /api/v1/biofield/sessions` returns `201`
- `POST /api/v1/biofield/sessions/:session_id/captures` returns `201`
- `GET /api/v1/biofield/readings` returns the new reading
- `GET /api/v1/biofield/readings/:reading_id` returns detail for the new reading
- `POST /api/v1/biofield/sessions/:session_id/close` returns `closed`

## What This Runbook Proves

This Phase 1 runbook proves the end-to-end browser-visible biofield slice:

- authenticated session lifecycle
- capture ingestion through Noesis
- Python-backed analysis response
- reading persistence
- artifact metadata linkage
- history list
- reading detail

## What It Does Not Prove Yet

This runbook does not yet prove later-phase scope such as:

- reading reprocess flow
- baselines
- exports
- downstream synthesis into non-biofield product surfaces
