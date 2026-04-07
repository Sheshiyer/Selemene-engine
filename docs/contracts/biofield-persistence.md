# Biofield Persistence Contract

Date: 2026-04-05
Status: frozen for Phase 1 Wave 1.2

## Purpose

This contract defines the persistence shape for the first biofield capture flow.

It exists to keep the root migration tree, the Supabase migration tree, and the Rust data layer aligned before repository code lands.

## Design Goals

- keep the canonical biofield analysis payload in the shared `readings` table
- preserve capture-native semantics without forcing new nullable columns into `readings`
- support session lifecycle, artifact metadata, and reading linkage for the beta flow
- make the root and Supabase migrations byte-identical so drift is obvious in review

## Table Design

### `biofield_sessions`

Purpose:

- record the authenticated viewer session lifecycle
- keep capture attempts grouped under a user-owned session

Columns:

- `id` UUID primary key
- `user_id` UUID foreign key to `users(id)`
- `status` with Phase 1 values:
  - `active`
  - `closed`
  - `abandoned`
- `client_device_id` optional browser/device identifier
- `viewer_version` optional web build or viewer version string
- `notes` optional operator or repair note field
- `started_at` lifecycle start timestamp
- `closed_at` lifecycle end timestamp when session leaves `active`
- `created_at`
- `updated_at`

Rules:

- `active` sessions must keep `closed_at = NULL`
- `closed` and `abandoned` sessions must set `closed_at`
- `closed_at` may never be earlier than `started_at`

Indexes:

- by `user_id`
- by `(user_id, started_at DESC)`
- by `(user_id, status, started_at DESC)`

### `biofield_capture_artifacts`

Purpose:

- store metadata for source and derived capture files
- link stored artifacts back to the owning session and, when analysis succeeds, the persisted reading

Columns:

- `id` UUID primary key
- `session_id` UUID foreign key to `biofield_sessions(id)`
- `reading_id` nullable UUID foreign key to `readings(id)`
- `artifact_kind` with Phase 1 values:
  - `source-image`
  - `segmentation-mask`
  - `analysis-overlay`
  - `thumbnail`
- `storage_path` canonical storage key or path
- `mime_type`
- `byte_size`
- `capture_metadata` JSONB metadata bag
- `created_at`

Rules:

- every artifact belongs to exactly one session
- `reading_id` stays nullable so rejected or not-yet-persisted captures can still keep artifact metadata
- `storage_path` must be non-empty and globally unique
- `byte_size` must be non-negative

Indexes:

- by `(session_id, created_at DESC)`
- by `(reading_id, created_at DESC)` for linked-reading lookups
- by `(session_id, artifact_kind, created_at DESC)`
- unique on `storage_path`

## Reading Linkage

Phase 1 keeps the canonical analysis result in `readings` with:

- `engine_id = "biofield-capture"`
- normalized capture request metadata in `input_data`
- normalized analysis output in `result_data`

Session linkage is derived through `biofield_capture_artifacts.reading_id`, not by adding a new `session_id` column to `readings` in this wave.

This keeps the shared readings contract stable while still letting biofield history and detail routes recover:

- owning session
- source artifact metadata
- derived artifact metadata

## Migration Sync Rule

The root migration file and the Supabase migration file for this wave must remain byte-identical apart from file name.

Phase 1 file names:

- `migrations/017_biofield_sessions.sql`
- `supabase/migrations/20260405000017_017_biofield_sessions.sql`

## Review Checklist

- session status values match the frozen domain contract
- artifact linkage supports successful and rejected capture paths
- reading linkage is preserved without changing the shared `readings` table shape
- index coverage supports user history, session lookups, and reading detail joins
- root and Supabase migration contents are kept in sync
