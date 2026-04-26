# Python Sidecar Services — Integration Guide

These sidecars extend the reflection stack with specialized perception capabilities while keeping the core orchestration model consistent.

## Architecture

Selemene Engine uses a **sidecar pattern** for computer vision tasks that require Python libraries (MediaPipe, OpenCV). The Rust core communicates with Python services via HTTP on localhost.

```
┌─────────────────────────────┐     ┌──────────────────────────────┐
│   Rust (noesis-server)      │     │   Python Sidecar Services    │
│   Port 8080                 │     │                              │
│                             │     │   ┌──────────────────────┐   │
│   engine-face-reading ──HTTP──────>   │ MediaPipe Face Mesh  │   │
│                             │     │   │ Port 8001            │   │
│                             │     │   └──────────────────────┘   │
│                             │     │                              │
│   engine-biofield    ──HTTP──────>│   ┌──────────────────────┐   │
│                             │     │   │ Biofield CV          │   │
│                             │     │   │ Port 8002            │   │
│                             │     │   └──────────────────────┘   │
└─────────────────────────────┘     └──────────────────────────────┘
```

This is the same pattern used by `noesis-bridge` (Rust <-> TypeScript engines on port 3001).

## Services

| Service | Port | Purpose | Dependencies |
|---------|------|---------|-------------|
| MediaPipe Face Mesh | 8001 | 468 facial landmark detection | mediapipe, opencv-python-headless |
| Biofield CV | 8002 | Spatial biofield image analysis | opencv-python-headless, numpy, scipy |

## API Contracts

OpenAPI specs live in `python-services/openapi/`:
- `mediapipe-service.yaml` — Face Mesh service (POST /analyze)
- `biofield-cv-service.yaml` — Biofield CV service (POST /analyze)

Both services expose:
- `GET /health` — Health check with dependency availability flags
- `POST /analyze` — Image analysis (multipart/form-data)

## Rust Integration

### Using BridgeEngine (Existing Pattern)

The `noesis-bridge` crate provides `BridgeEngine` which wraps HTTP calls to external services. For Python services, create similar bridge engines:

```rust
use noesis_bridge::BridgeEngine;

// Create a bridge to the MediaPipe service
let mediapipe = BridgeEngine::new(
    "face-reading",      // engine_id
    "Face Reading",      // engine_name
    1,                   // required_phase
    "http://localhost:8001"  // base_url
);
```

### Custom HTTP Client (For Image Upload)

Since Python services accept multipart/form-data (not JSON), engines need a custom HTTP client:

```rust
use reqwest::multipart;

pub struct PythonServiceClient {
    client: reqwest::Client,
    base_url: String,
    timeout: Duration,
}

impl PythonServiceClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(2))
                .build()
                .expect("Failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(10),
        }
    }

    pub async fn analyze_image(&self, image_bytes: &[u8]) -> Result<Value, EngineError> {
        let part = multipart::Part::bytes(image_bytes.to_vec())
            .file_name("capture.jpg")
            .mime_str("image/jpeg")
            .map_err(|e| EngineError::BridgeError(e.to_string()))?;

        let form = multipart::Form::new().part("image", part);

        let response = self.client
            .post(format!("{}/analyze", self.base_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| EngineError::BridgeError(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(EngineError::BridgeError(
                format!("Service returned {}", response.status())
            ));
        }

        response.json().await
            .map_err(|e| EngineError::BridgeError(format!("JSON parse error: {}", e)))
    }

    pub async fn health_check(&self) -> Result<bool, EngineError> {
        let response = self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map_err(|e| EngineError::BridgeError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}
```

### Configuration

Add to `ApiConfig`:

```rust
pub python_mediapipe_url: Option<String>,  // default: http://localhost:8001
pub python_biofield_url: Option<String>,   // default: http://localhost:8002
```

Load from env vars:
- `PYTHON_MEDIAPIPE_URL` (default: `http://localhost:8001`)
- `PYTHON_BIOFIELD_URL` (default: `http://localhost:8002`)

## Error Handling & Graceful Degradation

When Python services are unavailable, Rust engines should fall back to mock data:

```rust
async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
    match self.python_client.analyze_image(&image_bytes).await {
        Ok(cv_result) => {
            // Use real CV data
            self.build_output_from_cv(cv_result)
        }
        Err(e) => {
            tracing::warn!("Python service unavailable, using mock data: {}", e);
            // Fall back to mock data
            self.build_mock_output()
        }
    }
}
```

Health checks should report degraded mode:
```json
{
  "status": "degraded",
  "message": "Python CV services unavailable — using mock data",
  "services": {
    "mediapipe": false,
    "biofield_cv": false
  }
}
```

## Development Workflow

### Running Locally

```bash
# Terminal 1: Rust server
cargo run

# Terminal 2: MediaPipe service
cd python-services
pip install -e ".[mediapipe,dev]"
python -m mediapipe_service.main

# Terminal 3: Biofield CV service
cd python-services
python -m biofield_cv_service.main
```

### Docker Compose

```bash
docker-compose up  # Starts Rust + Redis + Postgres + Python services
```

### Testing

```bash
# Python tests
cd python-services
pytest

# Rust integration tests (requires Python services running)
cargo test --package noesis-api -- --test python_integration
```

## Deployment (Railway)

Python services deploy as a separate Railway service:
1. Build: `pip install -e ".[mediapipe]"`
2. Start: `uvicorn mediapipe_service.main:app --host 0.0.0.0 --port 8001`
3. Health check: `GET /health`

The Rust service connects via Railway's internal networking (`<service-name>.railway.internal`).

## Performance Considerations

| Metric | Target | Notes |
|--------|--------|-------|
| MediaPipe landmark detection | < 800ms p95 | CPU-only on Railway |
| Biofield CV analysis | < 500ms p95 | Depends on algorithm selection |
| Health check | < 50ms | Instant response |
| Connection timeout | 2s | Fail fast if service is down |
| Request timeout | 10s | Allow for heavy CV processing |

## Monitoring

Wrap Python service calls with Sentry spans:

```rust
let span = sentry::start_span(sentry::SpanDescription {
    op: "http.client".into(),
    description: Some("python.mediapipe.analyze".into()),
});
let result = self.python_client.analyze_image(&bytes).await;
span.finish();
```
