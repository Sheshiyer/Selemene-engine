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
