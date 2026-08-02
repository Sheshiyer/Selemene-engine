-- Migration: 036_living_readings_archive
-- Description: Canonical, owner-scoped archive for imported living readings.
--
-- Account owners remain deliberately separate from corpus subjects. A subject
-- is never a users row and cannot acquire account authority through this
-- schema. Reading and artifact bodies remain in object storage; PostgreSQL
-- stores stable identifiers, checksums, locators, and editorial metadata.
--
-- Reverse: see the dependency-ordered DOWN block at the bottom.

-- ── UP ───────────────────────────────────────────────────────────────────

CREATE TABLE reading_import_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    manifest_id VARCHAR(128) NOT NULL,
    idempotency_key VARCHAR(128) NOT NULL,
    manifest_schema_version VARCHAR(32) NOT NULL,
    manifest_sha256 VARCHAR(64) NOT NULL,
    source_root_locator TEXT NOT NULL,
    state VARCHAR(24) NOT NULL DEFAULT 'pending'
        CHECK (state IN ('pending', 'running', 'completed', 'failed', 'rolled_back')),
    stats JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(stats) = 'object'),
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT reading_import_runs_manifest_sha256_format
        CHECK (manifest_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT reading_import_runs_manifest_id_nonempty
        CHECK (length(btrim(manifest_id)) > 0),
    CONSTRAINT reading_import_runs_idempotency_key_nonempty
        CHECK (length(btrim(idempotency_key)) > 0),
    CONSTRAINT reading_import_runs_source_root_nonempty
        CHECK (length(btrim(source_root_locator)) > 0),
    CONSTRAINT reading_import_runs_state_consistency CHECK (
        (state IN ('pending', 'running') AND finished_at IS NULL AND error_message IS NULL)
        OR (state = 'completed' AND finished_at IS NOT NULL AND error_message IS NULL)
        OR (
            state = 'failed'
            AND finished_at IS NOT NULL
            AND length(btrim(COALESCE(error_message, ''))) > 0
        )
        OR (state = 'rolled_back' AND finished_at IS NOT NULL)
    ),
    CONSTRAINT reading_import_runs_owner_idempotency_key
        UNIQUE (owner_user_id, idempotency_key),
    CONSTRAINT reading_import_runs_owner_manifest_id
        UNIQUE (owner_user_id, manifest_id),
    CONSTRAINT reading_import_runs_owner_manifest_sha256
        UNIQUE (owner_user_id, manifest_sha256),
    CONSTRAINT reading_import_runs_id_owner_key
        UNIQUE (id, owner_user_id)
);

CREATE INDEX idx_reading_import_runs_owner_started
    ON reading_import_runs(owner_user_id, started_at DESC);
CREATE INDEX idx_reading_import_runs_state_started
    ON reading_import_runs(state, started_at DESC);

CREATE TRIGGER update_reading_import_runs_updated_at
    BEFORE UPDATE ON reading_import_runs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE reading_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    import_run_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    stable_source_id VARCHAR(160) NOT NULL,
    source_kind VARCHAR(24) NOT NULL
        CHECK (source_kind IN ('manifest', 'file', 'directory', 'generated', 'external')),
    locator TEXT NOT NULL,
    content_sha256 VARCHAR(64),
    byte_size BIGINT,
    media_type VARCHAR(255),
    observed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(metadata) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT reading_sources_import_run_owner_fk
        FOREIGN KEY (import_run_id, owner_user_id)
        REFERENCES reading_import_runs(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT reading_sources_stable_id_nonempty
        CHECK (length(btrim(stable_source_id)) > 0),
    CONSTRAINT reading_sources_locator_nonempty
        CHECK (length(btrim(locator)) > 0),
    CONSTRAINT reading_sources_content_sha256_format
        CHECK (content_sha256 IS NULL OR content_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT reading_sources_byte_size_nonnegative
        CHECK (byte_size IS NULL OR byte_size >= 0),
    CONSTRAINT reading_sources_run_stable_id_key
        UNIQUE (import_run_id, stable_source_id),
    CONSTRAINT reading_sources_run_locator_key
        UNIQUE (import_run_id, locator),
    CONSTRAINT reading_sources_id_run_owner_key
        UNIQUE (id, import_run_id, owner_user_id),
    CONSTRAINT reading_sources_id_owner_key
        UNIQUE (id, owner_user_id)
);

CREATE INDEX idx_reading_sources_owner_kind
    ON reading_sources(owner_user_id, source_kind);
CREATE INDEX idx_reading_sources_content_sha256
    ON reading_sources(content_sha256)
    WHERE content_sha256 IS NOT NULL;

CREATE TABLE corpus_subjects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subject_key VARCHAR(160) NOT NULL,
    subject_type VARCHAR(24) NOT NULL DEFAULT 'person'
        CHECK (subject_type IN ('person', 'group', 'organization', 'place', 'event', 'other')),
    canonical_name TEXT NOT NULL,
    sort_name TEXT,
    reconciliation_state VARCHAR(24) NOT NULL DEFAULT 'candidate'
        CHECK (reconciliation_state IN ('candidate', 'verified', 'ambiguous', 'rejected', 'archived')),
    profile JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(profile) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ,
    CONSTRAINT corpus_subjects_subject_key_nonempty
        CHECK (length(btrim(subject_key)) > 0),
    CONSTRAINT corpus_subjects_canonical_name_nonempty
        CHECK (length(btrim(canonical_name)) > 0),
    CONSTRAINT corpus_subjects_archive_consistency CHECK (
        (reconciliation_state = 'archived' AND archived_at IS NOT NULL)
        OR (reconciliation_state <> 'archived' AND archived_at IS NULL)
    ),
    CONSTRAINT corpus_subjects_owner_subject_key
        UNIQUE (owner_user_id, subject_key),
    CONSTRAINT corpus_subjects_id_owner_key
        UNIQUE (id, owner_user_id)
);

CREATE INDEX idx_corpus_subjects_owner_state_name
    ON corpus_subjects(owner_user_id, reconciliation_state, canonical_name);
CREATE INDEX idx_corpus_subjects_owner_sort_name
    ON corpus_subjects(owner_user_id, sort_name)
    WHERE sort_name IS NOT NULL;

CREATE TRIGGER update_corpus_subjects_updated_at
    BEFORE UPDATE ON corpus_subjects
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE corpus_subject_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL,
    source_id UUID,
    owner_user_id UUID NOT NULL,
    alias TEXT NOT NULL,
    normalized_alias TEXT NOT NULL,
    alias_kind VARCHAR(24) NOT NULL DEFAULT 'name'
        CHECK (alias_kind IN ('name', 'nickname', 'transliteration', 'path_segment', 'identifier', 'other')),
    review_state VARCHAR(16) NOT NULL DEFAULT 'candidate'
        CHECK (review_state IN ('candidate', 'verified', 'rejected')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT corpus_subject_aliases_subject_owner_fk
        FOREIGN KEY (subject_id, owner_user_id)
        REFERENCES corpus_subjects(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT corpus_subject_aliases_source_owner_fk
        FOREIGN KEY (source_id, owner_user_id)
        REFERENCES reading_sources(id, owner_user_id)
        ON DELETE RESTRICT,
    CONSTRAINT corpus_subject_aliases_alias_nonempty
        CHECK (length(btrim(alias)) > 0),
    CONSTRAINT corpus_subject_aliases_normalized_nonempty
        CHECK (length(btrim(normalized_alias)) > 0),
    CONSTRAINT corpus_subject_aliases_subject_normalized_key
        UNIQUE (subject_id, normalized_alias)
);

CREATE INDEX idx_corpus_subject_aliases_normalized
    ON corpus_subject_aliases(normalized_alias);
CREATE INDEX idx_corpus_subject_aliases_source
    ON corpus_subject_aliases(source_id)
    WHERE source_id IS NOT NULL;

CREATE TABLE corpus_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    relationship_key VARCHAR(160) NOT NULL,
    relationship_kind VARCHAR(64) NOT NULL,
    label TEXT,
    reconciliation_state VARCHAR(24) NOT NULL DEFAULT 'candidate'
        CHECK (reconciliation_state IN ('candidate', 'verified', 'ambiguous', 'rejected', 'archived')),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(metadata) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ,
    CONSTRAINT corpus_relationships_key_nonempty
        CHECK (length(btrim(relationship_key)) > 0),
    CONSTRAINT corpus_relationships_kind_nonempty
        CHECK (length(btrim(relationship_kind)) > 0),
    CONSTRAINT corpus_relationships_archive_consistency CHECK (
        (reconciliation_state = 'archived' AND archived_at IS NOT NULL)
        OR (reconciliation_state <> 'archived' AND archived_at IS NULL)
    ),
    CONSTRAINT corpus_relationships_owner_key
        UNIQUE (owner_user_id, relationship_key),
    CONSTRAINT corpus_relationships_id_owner_key
        UNIQUE (id, owner_user_id)
);

CREATE INDEX idx_corpus_relationships_owner_state
    ON corpus_relationships(owner_user_id, reconciliation_state, created_at DESC);

CREATE TRIGGER update_corpus_relationships_updated_at
    BEFORE UPDATE ON corpus_relationships
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE corpus_relationship_members (
    relationship_id UUID NOT NULL,
    subject_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    member_role VARCHAR(24) NOT NULL
        CHECK (member_role IN ('primary', 'secondary', 'member', 'author', 'recipient', 'observer')),
    position SMALLINT NOT NULL CHECK (position > 0),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(metadata) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT corpus_relationship_members_relationship_owner_fk
        FOREIGN KEY (relationship_id, owner_user_id)
        REFERENCES corpus_relationships(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT corpus_relationship_members_subject_owner_fk
        FOREIGN KEY (subject_id, owner_user_id)
        REFERENCES corpus_subjects(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT corpus_relationship_members_pk
        PRIMARY KEY (relationship_id, subject_id, member_role),
    CONSTRAINT corpus_relationship_members_position_key
        UNIQUE (relationship_id, position)
);

CREATE UNIQUE INDEX uq_corpus_relationship_members_named_role
    ON corpus_relationship_members(relationship_id, member_role)
    WHERE member_role IN ('primary', 'secondary', 'author', 'recipient');
CREATE INDEX idx_corpus_relationship_members_subject
    ON corpus_relationship_members(subject_id, relationship_id);

CREATE TABLE archived_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL,
    import_run_id UUID NOT NULL,
    primary_source_id UUID NOT NULL,
    relationship_id UUID,
    stable_reading_id VARCHAR(160) NOT NULL,
    title TEXT NOT NULL,
    reading_type VARCHAR(64) NOT NULL,
    language_tag VARCHAR(35) NOT NULL DEFAULT 'en',
    producer_kind VARCHAR(24) NOT NULL
        CHECK (producer_kind IN ('human', 'engine', 'workflow', 'imported', 'unknown')),
    producer_ref TEXT,
    content_locator TEXT NOT NULL,
    content_sha256 VARCHAR(64) NOT NULL,
    summary TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(metadata) = 'object'),
    record_state VARCHAR(16) NOT NULL DEFAULT 'active'
        CHECK (record_state IN ('active', 'deleted')),
    captured_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT archived_readings_import_run_owner_fk
        FOREIGN KEY (import_run_id, owner_user_id)
        REFERENCES reading_import_runs(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT archived_readings_primary_source_fk
        FOREIGN KEY (primary_source_id, import_run_id, owner_user_id)
        REFERENCES reading_sources(id, import_run_id, owner_user_id)
        ON DELETE RESTRICT,
    CONSTRAINT archived_readings_relationship_owner_fk
        FOREIGN KEY (relationship_id, owner_user_id)
        REFERENCES corpus_relationships(id, owner_user_id)
        ON DELETE RESTRICT,
    CONSTRAINT archived_readings_stable_id_nonempty
        CHECK (length(btrim(stable_reading_id)) > 0),
    CONSTRAINT archived_readings_title_nonempty
        CHECK (length(btrim(title)) > 0),
    CONSTRAINT archived_readings_type_nonempty
        CHECK (length(btrim(reading_type)) > 0),
    CONSTRAINT archived_readings_language_nonempty
        CHECK (length(btrim(language_tag)) > 0),
    CONSTRAINT archived_readings_content_locator_nonempty
        CHECK (length(btrim(content_locator)) > 0),
    CONSTRAINT archived_readings_content_sha256_format
        CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT archived_readings_delete_consistency CHECK (
        (record_state = 'active' AND deleted_at IS NULL)
        OR (record_state = 'deleted' AND deleted_at IS NOT NULL)
    ),
    CONSTRAINT archived_readings_owner_stable_id_key
        UNIQUE (owner_user_id, stable_reading_id),
    CONSTRAINT archived_readings_owner_content_locator_key
        UNIQUE (owner_user_id, content_locator),
    CONSTRAINT archived_readings_id_owner_key
        UNIQUE (id, owner_user_id)
);

CREATE INDEX idx_archived_readings_owner_created
    ON archived_readings(owner_user_id, created_at DESC);
CREATE INDEX idx_archived_readings_owner_type_created
    ON archived_readings(owner_user_id, reading_type, created_at DESC);
CREATE INDEX idx_archived_readings_import_run
    ON archived_readings(import_run_id);
CREATE INDEX idx_archived_readings_source
    ON archived_readings(primary_source_id);
CREATE INDEX idx_archived_readings_content_sha256
    ON archived_readings(content_sha256);
CREATE INDEX idx_archived_readings_relationship
    ON archived_readings(relationship_id)
    WHERE relationship_id IS NOT NULL;
CREATE INDEX idx_archived_readings_active_browse
    ON archived_readings(owner_user_id, captured_at DESC, id)
    WHERE record_state = 'active';

CREATE TRIGGER update_archived_readings_updated_at
    BEFORE UPDATE ON archived_readings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE archived_reading_subjects (
    reading_id UUID NOT NULL,
    subject_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    subject_role VARCHAR(24) NOT NULL
        CHECK (subject_role IN ('primary', 'secondary', 'author', 'recipient', 'mentioned', 'observer')),
    confidence NUMERIC(5,4)
        CHECK (confidence IS NULL OR (confidence >= 0 AND confidence <= 1)),
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT archived_reading_subjects_reading_owner_fk
        FOREIGN KEY (reading_id, owner_user_id)
        REFERENCES archived_readings(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT archived_reading_subjects_subject_owner_fk
        FOREIGN KEY (subject_id, owner_user_id)
        REFERENCES corpus_subjects(id, owner_user_id)
        ON DELETE RESTRICT,
    CONSTRAINT archived_reading_subjects_pk
        PRIMARY KEY (reading_id, subject_id, subject_role)
);

CREATE UNIQUE INDEX uq_archived_reading_subjects_named_role
    ON archived_reading_subjects(reading_id, subject_role)
    WHERE subject_role IN ('primary', 'secondary', 'author', 'recipient');
CREATE INDEX idx_archived_reading_subjects_subject
    ON archived_reading_subjects(subject_id, reading_id);

CREATE TABLE archived_reading_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reading_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    artifact_key VARCHAR(160) NOT NULL,
    artifact_role VARCHAR(32) NOT NULL
        CHECK (artifact_role IN (
            'source_document',
            'rendered_reading',
            'attachment',
            'image',
            'audio',
            'video',
            'data',
            'other'
        )),
    storage_provider VARCHAR(24) NOT NULL
        CHECK (storage_provider IN ('r2', 's3', 'gcs', 'azure', 'https', 'filesystem', 'other')),
    object_locator TEXT NOT NULL,
    content_sha256 VARCHAR(64) NOT NULL,
    byte_size BIGINT NOT NULL CHECK (byte_size >= 0),
    media_type VARCHAR(255),
    availability_state VARCHAR(24) NOT NULL DEFAULT 'available'
        CHECK (availability_state IN ('pending', 'available', 'missing', 'quarantined', 'deleted')),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(metadata) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT archived_reading_artifacts_reading_owner_fk
        FOREIGN KEY (reading_id, owner_user_id)
        REFERENCES archived_readings(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT archived_reading_artifacts_key_nonempty
        CHECK (length(btrim(artifact_key)) > 0),
    CONSTRAINT archived_reading_artifacts_locator_nonempty
        CHECK (length(btrim(object_locator)) > 0),
    CONSTRAINT archived_reading_artifacts_sha256_format
        CHECK (content_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT archived_reading_artifacts_delete_consistency CHECK (
        (availability_state = 'deleted' AND deleted_at IS NOT NULL)
        OR (availability_state <> 'deleted' AND deleted_at IS NULL)
    ),
    CONSTRAINT archived_reading_artifacts_reading_key
        UNIQUE (reading_id, artifact_key),
    CONSTRAINT archived_reading_artifacts_owner_locator_key
        UNIQUE (owner_user_id, object_locator),
    CONSTRAINT archived_reading_artifacts_id_reading_owner_key
        UNIQUE (id, reading_id, owner_user_id),
    CONSTRAINT archived_reading_artifacts_id_owner_key
        UNIQUE (id, owner_user_id)
);

CREATE INDEX idx_archived_reading_artifacts_reading_role
    ON archived_reading_artifacts(reading_id, artifact_role);
CREATE INDEX idx_archived_reading_artifacts_availability
    ON archived_reading_artifacts(owner_user_id, availability_state);
CREATE INDEX idx_archived_reading_artifacts_sha256
    ON archived_reading_artifacts(content_sha256);

CREATE TRIGGER update_archived_reading_artifacts_updated_at
    BEFORE UPDATE ON archived_reading_artifacts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE archived_reading_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reading_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    source_id UUID,
    artifact_id UUID,
    evidence_key VARCHAR(160) NOT NULL,
    evidence_type VARCHAR(32) NOT NULL
        CHECK (evidence_type IN (
            'source_excerpt',
            'structured_fact',
            'citation',
            'calculation',
            'editorial_note'
        )),
    claim TEXT NOT NULL,
    excerpt TEXT,
    review_state VARCHAR(24) NOT NULL DEFAULT 'unreviewed'
        CHECK (review_state IN ('unreviewed', 'supported', 'contested', 'rejected', 'withheld')),
    confidence NUMERIC(5,4)
        CHECK (confidence IS NULL OR (confidence >= 0 AND confidence <= 1)),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(metadata) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT archived_reading_evidence_reading_owner_fk
        FOREIGN KEY (reading_id, owner_user_id)
        REFERENCES archived_readings(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT archived_reading_evidence_source_owner_fk
        FOREIGN KEY (source_id, owner_user_id)
        REFERENCES reading_sources(id, owner_user_id)
        ON DELETE RESTRICT,
    CONSTRAINT archived_reading_evidence_artifact_reading_owner_fk
        FOREIGN KEY (artifact_id, reading_id, owner_user_id)
        REFERENCES archived_reading_artifacts(id, reading_id, owner_user_id)
        ON DELETE RESTRICT,
    CONSTRAINT archived_reading_evidence_key_nonempty
        CHECK (length(btrim(evidence_key)) > 0),
    CONSTRAINT archived_reading_evidence_claim_nonempty
        CHECK (length(btrim(claim)) > 0),
    CONSTRAINT archived_reading_evidence_reference_consistency CHECK (
        evidence_type NOT IN ('source_excerpt', 'citation')
        OR source_id IS NOT NULL
        OR artifact_id IS NOT NULL
    ),
    CONSTRAINT archived_reading_evidence_reading_key
        UNIQUE (reading_id, evidence_key)
);

CREATE INDEX idx_archived_reading_evidence_reading_state
    ON archived_reading_evidence(reading_id, review_state);
CREATE INDEX idx_archived_reading_evidence_owner_state
    ON archived_reading_evidence(owner_user_id, review_state);
CREATE INDEX idx_archived_reading_evidence_source
    ON archived_reading_evidence(source_id)
    WHERE source_id IS NOT NULL;

CREATE TRIGGER update_archived_reading_evidence_updated_at
    BEFORE UPDATE ON archived_reading_evidence
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE archived_reading_editorial_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reading_id UUID NOT NULL,
    owner_user_id UUID NOT NULL,
    state VARCHAR(24) NOT NULL
        CHECK (state IN ('imported', 'needs_review', 'in_review', 'approved', 'rejected', 'published', 'archived')),
    visibility VARCHAR(24) NOT NULL DEFAULT 'owner_only'
        CHECK (visibility IN ('owner_only', 'admin_only', 'shared', 'public')),
    changed_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    change_role VARCHAR(24) NOT NULL
        CHECK (change_role IN ('importer', 'owner', 'editor', 'reviewer', 'system')),
    revision INTEGER NOT NULL CHECK (revision > 0),
    is_current BOOLEAN NOT NULL DEFAULT true,
    rationale TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT archived_reading_editorial_states_reading_owner_fk
        FOREIGN KEY (reading_id, owner_user_id)
        REFERENCES archived_readings(id, owner_user_id)
        ON DELETE CASCADE,
    CONSTRAINT archived_reading_editorial_states_reading_revision_key
        UNIQUE (reading_id, revision)
);

CREATE UNIQUE INDEX uq_archived_reading_editorial_states_current
    ON archived_reading_editorial_states(reading_id)
    WHERE is_current;
CREATE INDEX idx_archived_reading_editorial_states_owner_state
    ON archived_reading_editorial_states(owner_user_id, state, created_at DESC)
    WHERE is_current;
CREATE INDEX idx_archived_reading_editorial_states_changed_by
    ON archived_reading_editorial_states(changed_by_user_id)
    WHERE changed_by_user_id IS NOT NULL;

COMMENT ON TABLE corpus_subjects IS
    'Owner-scoped corpus identities. These are not accounts and intentionally have no foreign key that turns a users row into a subject.';
COMMENT ON TABLE archived_reading_artifacts IS
    'Object metadata only. object_locator and content_sha256 identify storage content; artifact bytes never belong in PostgreSQL.';
COMMENT ON COLUMN archived_readings.producer_ref IS
    'External producer identifier (human, engine, or workflow); never inferred from owner_user_id.';
COMMENT ON COLUMN archived_reading_evidence.excerpt IS
    'Optional short editorial excerpt used for review, not a copy of the source artifact.';

-- ── DOWN (manual rollback, statements remain commented for forward runners) ──
-- <down>
-- DROP TABLE IF EXISTS archived_reading_editorial_states;
-- DROP TABLE IF EXISTS archived_reading_evidence;
-- DROP TABLE IF EXISTS archived_reading_artifacts;
-- DROP TABLE IF EXISTS archived_reading_subjects;
-- DROP TABLE IF EXISTS archived_readings;
-- DROP TABLE IF EXISTS corpus_relationship_members;
-- DROP TABLE IF EXISTS corpus_relationships;
-- DROP TABLE IF EXISTS corpus_subject_aliases;
-- DROP TABLE IF EXISTS corpus_subjects;
-- DROP TABLE IF EXISTS reading_sources;
-- DROP TABLE IF EXISTS reading_import_runs;
-- </down>
