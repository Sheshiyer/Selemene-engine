# Biofield Python Sidecar Contract

Date: 2026-04-05
Status: frozen for Phase 1 implementation

## Purpose

This contract defines the private request and response boundary between Noesis API and `python-services/biofield_cv_service`.

It exists to stop the Rust handler and the Python service from drifting while the stub analysis path is replaced with real BV-PIP calculation logic.

## Network Boundary

- sidecar is private infrastructure
- browser never calls it directly
- Noesis API is the only caller

Default local runtime:

- service: `biofield_cv_service`
- port: `8002`
- endpoint: `POST /analyze`

## Request Contract

Transport:

- `multipart/form-data`

Required field:

- `image`
  - raw uploaded image bytes
  - accepted formats: implementation-defined image formats supported by the service

Optional fields:

- `algorithms`
  - JSON array of algorithm names to run
- `options`
  - JSON object for analysis options and feature switches
- `capture_metadata`
  - optional JSON object if the service later needs camera or viewer hints

Current compatibility note:

- Rust may send `file` or `image` to the public API
- the public API should normalize that to `image` for the sidecar request

## Required Response Shape

```json
{
  "contract_version": "biofield-cv/v1",
  "analysis_version": "string",
  "metrics": {},
  "quality_assessment": {
    "sharpness": 0.0,
    "contrast": 0.0,
    "noise_level": 0.0,
    "exposure": 0.0,
    "sufficient_quality": true
  },
  "algorithms_run": [],
  "processing_time_ms": 0.0
}
```

### Required top-level fields

- `contract_version`
- `analysis_version`
- `metrics`
- `quality_assessment`
- `algorithms_run`
- `processing_time_ms`

### `contract_version`

Phase 1 fixed value:

- `biofield-cv/v1`

### `analysis_version`

Rules:

- required on every successful response
- must change when extracted BV-PIP algorithm logic changes in a way that can affect persisted output

### `metrics`

Phase 1 metric group:

- `light_quanta_density`
- `normalized_area`
- `average_intensity`
- `inner_noise`
- `energy_analysis`
- `entropy_form_coefficient`
- `fractal_dimension`
- `correlation_dimension`
- `body_symmetry`
- `contour_complexity`
- `pattern_regularity`

The exact numeric definitions remain Python-owned, but field names are frozen here.

### `quality_assessment`

Required fields:

- `sharpness`
- `contrast`
- `noise_level`
- `exposure`
- `sufficient_quality`

If a capture fails quality gates, the service may either:

- return a normal response with `sufficient_quality = false`
- or return an explicit rejection response as defined below

The API layer must normalize that outcome into the public Noesis error model.

## Rejection And Error Contract

### Quality rejection

Preferred shape:

```json
{
  "contract_version": "biofield-cv/v1",
  "analysis_version": "string",
  "error_code": "BIOFIELD_CAPTURE_REJECTED_QUALITY",
  "error_message": "human-readable reason",
  "quality_assessment": {
    "sharpness": 0.0,
    "contrast": 0.0,
    "noise_level": 0.0,
    "exposure": 0.0,
    "sufficient_quality": false
  }
}
```

### Invalid payload

The sidecar should reject malformed multipart or malformed JSON option payloads with clear `4xx` responses.

### Service failure

Unexpected algorithm or runtime failures should result in `5xx` responses so Noesis can surface `BIOFIELD_ANALYSIS_UNAVAILABLE`.

## Versioning Rules

- changing route path, required field names, or removing metrics requires a contract version change
- adding optional fields does not require a contract version change
- changing algorithm behavior requires an `analysis_version` change even if `contract_version` remains `biofield-cv/v1`

## Stable Invariants

- Noesis API is responsible for auth and user ownership, not the sidecar
- sidecar responses must be deterministic for a given algorithm version and input
- persisted reading payloads must include enough metadata to trace which `analysis_version` produced them
