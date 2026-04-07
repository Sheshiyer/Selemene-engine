# Biofield Capture Analysis Bootstrap

Date: 2026-04-05
Status: Phase 1 / Wave 1.1 bootstrap runbook

## Purpose

This runbook covers the minimum local bootstrap path for the native biofield web slice before any session or capture routes are implemented.

It verifies that:

- the standalone `biofield-web` app boots
- the main Noesis API is reachable
- the private Python biofield sidecar is reachable

## Expected Local Ports

- `biofield-web`: `3002`
- `noesis-server`: `8080`
- `biofield_cv_service`: `8002`

## Required Environment

### Noesis API

- `JWT_SECRET`
- `PYTHON_BIOFIELD_URL`
  - default: `http://localhost:8002`
- `PYTHON_BIOFIELD_TIMEOUT_MS`
  - default: `10000`

### Biofield Web

- whatever auth and API base variables are needed for the local app shell later
- in Wave 1.1 the shell can still be built and opened without the full auth route wiring

## Local Bootstrap

### 1. Start the Python sidecar

```bash
cd python-services
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
uvicorn biofield_cv_service.main:app --host 0.0.0.0 --port 8002 --reload
```

### 2. Start the Rust API

```bash
cd /Volumes/madara/2026/witnessos/Selemene-engine
export JWT_SECRET="dev-secret-at-least-32-characters"
export PYTHON_BIOFIELD_URL="http://127.0.0.1:8002"
export PYTHON_BIOFIELD_TIMEOUT_MS="10000"
cargo run --bin noesis-server
```

### 3. Start the web app

```bash
cd /Volumes/madara/2026/witnessos/Selemene-engine/apps/biofield-web
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

Expected results:

- `/login`, `/viewer`, and `/history` return `200`
- `/health/live` returns `200`
- Python sidecar `/health` returns `200`

## What This Runbook Does Not Prove Yet

This Wave 1.1 runbook does not yet prove:

- session creation
- capture upload
- persisted readings
- history data
- reading detail data

Those validations belong to later Phase 1 waves once the API routes and repository layer exist.
