# ADR-0002: Face Analysis Backend Selection for `engine-face-reading`

- **Status:** Accepted
- **Date:** 2026-03-03
- **Related Issues:** #422, #423, #424, #425

## Context

Wave W1 requires replacing pure mock behavior in face-reading with a production-safe path that can:
1. Accept uploaded image input,
2. Produce deterministic non-mock outputs,
3. Fall back to birth-data-derived analysis when image input is unavailable,
4. Preserve API stability and avoid blocking v3.0.0 launch on heavy CV model rollout.

## Decision

Adopt a **phased backend strategy**:

1. **Phase A (current, launch-safe):**
   - Implement a deterministic heuristic analysis backend in Rust:
     - image-bytes-derived seed path,
     - birth-data physiognomy fallback path,
     - explicit mock-only fallback when neither input exists.
   - Mark metadata/backend precisely (`heuristic-image-backend`, `birth-physiognomy-fallback`, `mock-stub`).
   - Add multipart upload endpoint in `noesis-api` for face-reading.

2. **Phase B (post-launch hardening):**
   - Integrate landmark extraction using MediaPipe/ONNX runtime.
   - Map landmark vectors to constitutional model features.
   - Keep current API contract unchanged while replacing internals.

## Alternatives Considered

### 1) Immediate full MediaPipe/ONNX rollout
- **Pros:** Most realistic facial feature extraction.
- **Cons:** Heavier dependencies, deployment surface increase, slower launch unblock.
- **Result:** Deferred to Phase B.

### 2) Third-party hosted face-analysis API
- **Pros:** Fast implementation.
- **Cons:** Privacy/compliance concerns, external latency and reliability dependency.
- **Result:** Rejected for launch-critical path.

### 3) Keep mock-only behavior
- **Pros:** Lowest implementation effort.
- **Cons:** Fails Wave W1 acceptance criteria and undermines production confidence.
- **Result:** Rejected.

## Consequences

### Positive
- Unblocks Wave W1 launch requirements with deterministic non-mock behavior.
- Preserves stable interfaces for future backend upgrades.
- Enables immediate API upload integration and validation coverage.

### Trade-offs
- Heuristic backend is not equivalent to full landmark-driven analysis.
- Phase B remains required for maximum fidelity.

## Validation

- `cargo test -p engine-face-reading`
- `cargo test -p noesis-api --lib`
- `npm --prefix ts-engines test`
