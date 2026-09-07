---
phase: 03-capability-and-contract-closure
plan: "06"
subsystem: python-observations
tags: [python, fastapi, sidecar-health, dependency-observations]

requires:
  - phase: 03-capability-and-contract-closure
    plan: "04"
    provides: bounded resolver vocabulary and canonical dependency metadata
provides:
  - bounded Python dependency observation models
  - biofield and face-reading health projections attached to existing engine IDs
  - required/optional dependency test matrix in locked uv environment
affects: [03-07, 03-09, 03-15]

tech-stack:
  added: []
  patterns:
    - Pydantic observation models use the same bounded states/reason codes as v1
    - sidecar health remains local, non-generative and internal-only

key-files:
  created: []
  modified:
    - python-services/shared/models.py
    - python-services/biofield_cv_service/health.py
    - python-services/mediapipe_service/health.py
    - python-services/tests/test_capability_health.py
    - python-services/README.md

key-decisions:
  - "Python sidecars report observations for biofield and face-reading; they never add a twentieth engine identity."
  - "OpenCV and NumPy are required for the biofield-cv observation; MediaPipe is optional there and required for face-reading."
  - "Health endpoint status and legacy boolean fields remain compatible while capability_observations adds bounded truth."

patterns-established:
  - "Required-unavailable wins; optional MediaPipe absence degrades biofield capability."
  - "Import/check exceptions collapse to stable reason codes without raw paths, URLs or credentials."

requirements-completed: [CON-01, CON-02]
requirements-progressed: []

duration: 7min
completed: 2026-09-08
---

# Phase 3: Capability and contract closure — Plan 06 Summary

**Python sidecars now emit bounded dependency evidence under the existing canonical engine identities.**

## Accomplishments

- Added shared Pydantic models for capability observations, dependency kinds/requirements and bounded reason codes.
- Added `capability_observations` to both health responses while preserving service/status/version and existing dependency booleans.
- Biofield reports `python:biofield-cv` as required and `python:mediapipe-face-mesh` as optional; face-reading reports `python:mediapipe-face-mesh` as required.
- Added safe checker handling so local import exceptions become unavailable observations without leaking exception text.
- Documented the observation boundary and Phase 7 ingress/auth distinction in the Python service README.

## Verification Evidence

- `cd python-services && uv run --locked --extra dev python -m pytest tests/test_capability_health.py -q --tb=line` — 12 passed.
- `cd python-services && uv run --locked --extra dev python -m pytest tests/test_biofield_health.py tests/test_mediapipe_health.py -q --tb=line` — 12 passed.
- Only two existing Starlette/httpx deprecation warnings appeared; no test failure or raw diagnostic output.
- Tests cover required OpenCV/NumPy absence, optional MediaPipe absence, face-reading required absence, exception bounding and stable owning IDs.
- No sidecar start, provider request, database access, Railway/Cloudflare action, deployment or release mutation occurred.

## Deviations from Plan

None. The new observations are additive and remain local-only.

## User Setup Required

None.

## Next Phase Readiness

Plan 07 can join Rust, TypeScript and Python observations into the authenticated 19-row API capability envelope.

---
*Phase: 03-capability-and-contract-closure*
*Plan: 06*
