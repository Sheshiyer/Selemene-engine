# Biofield Domain Contract

Date: 2026-04-05
Status: frozen for Phase 1 implementation

## Purpose

This contract defines the user-facing and backend-owned nouns for the native biofield product surface.

It exists to keep web, backend, data, and Python analysis work aligned while the product moves from standalone BV-PIP assets into Selemene.

## Primary Nouns

### Session

A `session` is the authenticated, time-bounded viewer context in which a user opens the biofield surface, initializes local camera and rendering state, and may capture one or more artifacts.

Properties:

- owned by exactly one user
- created and closed by Noesis API
- may contain zero or more captures
- may remain useful even when no persisted reading is produced

Phase 1 status values:

- `active`
- `closed`
- `abandoned`

### Capture

A `capture` is a user-initiated still artifact produced during a session and submitted for deeper analysis.

Properties:

- always belongs to one session
- may have one source image artifact and zero or more derived artifacts
- may succeed, fail quality checks, or be reprocessed later

`capture` is a domain noun, but in persistence the successful analysis result is stored as a reading plus linked artifact rows.

### Reading

A `reading` is the canonical persisted analysis result of a completed capture.

Properties:

- stored in the shared `readings` table
- uses `engine_id = "biofield-capture"`
- contains normalized input metadata in `input_data`
- contains analysis output in `result_data`
- is the durable object used for history, detail views, comparisons, and later synthesis

### Baseline

A `baseline` is a user-owned reference aggregate assembled from one or more prior readings for later comparison.

Properties:

- not required for Phase 1 shipping
- route namespace is frozen now so later implementation does not force contract churn

## Ownership Rules

- Browser owns:
  - camera stream
  - local PIP renderer
  - local segmentation
  - lightweight live metrics
  - transient viewer state
- Noesis API owns:
  - authentication
  - session lifecycle
  - capture ingestion
  - persisted readings
  - artifact metadata
  - history, baselines, exports
- Python sidecar owns:
  - deep capture analysis
  - quality assessment
  - algorithm execution details

## Persistence Rules

### Reused persistence surface

`readings`

- `engine_id` is fixed to `biofield-capture`
- `input_data` stores capture request metadata and processing settings
- `result_data` stores the normalized biofield analysis payload

### Dedicated persistence surfaces

`biofield_sessions`

- authoritative record for session lifecycle
- links user to capture attempts

`biofield_capture_artifacts`

- authoritative record for source and derived artifact metadata
- links session and reading to storage-backed files

### Deferred persistence surfaces

- `biofield_baselines`
- `biofield_baseline_readings`
- optional export-job records if exports require asynchronous execution

## State Transitions

### Session lifecycle

`active -> closed`

Normal completed session.

`active -> abandoned`

Viewer leaves or errors before an explicit close.

Closed and abandoned sessions are immutable except for internal repair or metadata backfill.

### Capture lifecycle

`requested -> uploaded -> analyzed -> persisted`

Successful capture path.

`requested -> uploaded -> rejected`

Quality gate or validation failure.

`persisted -> reprocessed`

Later explicit re-analysis using the same stored source artifact, subject to policy.

## Beta Scope

Phase 1 must support:

- authenticated session create and close
- capture upload
- Python-backed analysis
- reading persistence
- history list
- reading detail

Phase 1 does not require:

- baselines
- exports
- workflow-level somatic synthesis

## Boundary With `engine-biofield`

This product surface is not the same thing as the existing `engine-biofield` workflow-facing engine.

Rules:

- live session and capture semantics stay in the dedicated biofield namespace
- `engine-biofield` remains stable in Phase 1
- future synthesis may consume persisted `biofield-capture` readings, but that is a later integration step

## Stable Invariants

- browser never calls the Python sidecar directly
- every persisted biofield result is user-scoped
- every persisted biofield result is represented as a reading
- session and artifact tables exist to preserve capture-native semantics without distorting the shared readings model
