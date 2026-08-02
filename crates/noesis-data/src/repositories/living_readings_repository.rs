use crate::models::living_reading::{
    LivingReadingArtifact, LivingReadingDetail, LivingReadingEditorialState, LivingReadingEvidence,
    LivingReadingImportRun, LivingReadingInvitation, LivingReadingInvitationResolutionRecord,
    LivingReadingListFilters, LivingReadingListItem, LivingReadingListPage,
    LivingReadingPublicSubject, LivingReadingPublicationArtifactCandidate,
    LivingReadingRelationship, LivingReadingRelationshipMember, LivingReadingSource,
    LivingReadingSubject, LivingReadingTypeFilter, NewLivingReadingInvitation,
};
use chrono::{DateTime, Utc};
use sqlx::{types::Json, Error, FromRow, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

const LIST_SELECT: &str = r#"
    SELECT
        r.id,
        r.owner_user_id,
        u.email AS owner_email,
        r.stable_reading_id,
        r.title,
        r.summary,
        r.reading_type,
        r.language_tag,
        r.producer_kind,
        r.producer_ref,
        s.id AS source_id,
        s.locator AS source_locator,
        s.content_sha256 AS source_sha256,
        ir.id AS import_run_id,
        ir.manifest_id AS import_manifest_id,
        rel.id AS relationship_id,
        rel.label AS relationship_label,
        rel.relationship_kind,
        editorial.state AS editorial_state,
        editorial.visibility AS editorial_visibility,
        CASE
            WHEN editorial.state IN ('approved', 'published')
                 AND EXISTS (
                     SELECT 1 FROM archived_reading_artifacts publication_artifact
                     WHERE publication_artifact.reading_id = r.id
                       AND publication_artifact.artifact_role = 'rendered_reading'
                       AND publication_artifact.availability_state = 'available'
                 )
            THEN 'metadata_available'
            WHEN editorial.state IN ('approved', 'published') THEN 'artifact_unavailable'
            ELSE 'not_completed'
        END AS publication_availability,
        COALESCE(subjects.items, '[]'::jsonb) AS subjects,
        r.captured_at,
        r.created_at
    FROM archived_readings r
    INNER JOIN users u ON u.id = r.owner_user_id
    INNER JOIN reading_sources s ON s.id = r.primary_source_id
    INNER JOIN reading_import_runs ir ON ir.id = r.import_run_id
    LEFT JOIN corpus_relationships rel ON rel.id = r.relationship_id
    LEFT JOIN LATERAL (
        SELECT es.state, es.visibility
        FROM archived_reading_editorial_states es
        WHERE es.reading_id = r.id AND es.is_current
        ORDER BY es.revision DESC
        LIMIT 1
    ) editorial ON true
    LEFT JOIN LATERAL (
        SELECT jsonb_agg(
            jsonb_build_object(
                'id', cs.id,
                'subject_key', cs.subject_key,
                'canonical_name', cs.canonical_name,
                'subject_type', cs.subject_type,
                'role', ars.subject_role,
                'confidence', ars.confidence::DOUBLE PRECISION,
                'aliases', COALESCE(
                    (
                        SELECT jsonb_agg(csa.alias ORDER BY csa.alias)
                        FROM corpus_subject_aliases csa
                        WHERE csa.subject_id = cs.id
                          AND csa.review_state <> 'rejected'
                    ),
                    '[]'::jsonb
                )
            )
            ORDER BY ars.subject_role, cs.canonical_name
        ) AS items
        FROM archived_reading_subjects ars
        INNER JOIN corpus_subjects cs ON cs.id = ars.subject_id
        WHERE ars.reading_id = r.id
    ) subjects ON true
    WHERE r.record_state = 'active'
"#;

#[derive(Debug, FromRow)]
struct LivingReadingListRow {
    id: Uuid,
    owner_user_id: Uuid,
    owner_email: String,
    stable_reading_id: String,
    title: String,
    summary: Option<String>,
    reading_type: String,
    language_tag: String,
    producer_kind: String,
    producer_ref: Option<String>,
    source_id: Uuid,
    source_locator: String,
    source_sha256: Option<String>,
    import_run_id: Uuid,
    import_manifest_id: String,
    relationship_id: Option<Uuid>,
    relationship_label: Option<String>,
    relationship_kind: Option<String>,
    editorial_state: Option<String>,
    editorial_visibility: Option<String>,
    publication_availability: String,
    subjects: Json<Vec<LivingReadingSubject>>,
    captured_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<LivingReadingListRow> for LivingReadingListItem {
    fn from(row: LivingReadingListRow) -> Self {
        Self {
            id: row.id,
            owner_user_id: row.owner_user_id,
            owner_email: row.owner_email,
            stable_reading_id: row.stable_reading_id,
            title: row.title,
            summary: row.summary,
            reading_type: row.reading_type,
            language_tag: row.language_tag,
            producer_kind: row.producer_kind,
            producer_ref: row.producer_ref,
            source_id: row.source_id,
            source_locator: row.source_locator,
            source_sha256: row.source_sha256,
            import_run_id: row.import_run_id,
            import_manifest_id: row.import_manifest_id,
            relationship_id: row.relationship_id,
            relationship_label: row.relationship_label,
            relationship_kind: row.relationship_kind,
            editorial_state: row.editorial_state,
            editorial_visibility: row.editorial_visibility,
            publication_availability: row.publication_availability,
            subjects: row.subjects.0,
            captured_at: row.captured_at,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct LivingReadingRelationshipRow {
    id: Uuid,
    relationship_key: String,
    relationship_kind: String,
    label: Option<String>,
    reconciliation_state: String,
    members: Json<Vec<LivingReadingRelationshipMember>>,
}

impl From<LivingReadingRelationshipRow> for LivingReadingRelationship {
    fn from(row: LivingReadingRelationshipRow) -> Self {
        Self {
            id: row.id,
            relationship_key: row.relationship_key,
            relationship_kind: row.relationship_kind,
            label: row.label,
            reconciliation_state: row.reconciliation_state,
            members: row.members.0,
        }
    }
}

pub struct LivingReadingsRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
struct InvitationArtifactCandidateRow {
    storage_provider: String,
    object_locator: String,
    content_sha256: String,
    byte_size: i64,
    media_type: Option<String>,
}

impl LivingReadingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        filters: &LivingReadingListFilters,
        limit: i64,
        offset: i64,
    ) -> Result<LivingReadingListPage, Error> {
        let limit = limit.clamp(1, 100);
        let offset = offset.clamp(0, 10_000);

        let mut items_query: QueryBuilder<Postgres> = QueryBuilder::new(LIST_SELECT);
        push_filters(&mut items_query, filters);
        items_query
            .push(" ORDER BY COALESCE(r.captured_at, r.created_at) DESC, r.id DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        let items = items_query
            .build_query_as::<LivingReadingListRow>()
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(LivingReadingListItem::from)
            .collect();

        let mut count_query: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT COUNT(*)::BIGINT
             FROM archived_readings r
             INNER JOIN users u ON u.id = r.owner_user_id
             LEFT JOIN corpus_relationships rel ON rel.id = r.relationship_id
             WHERE r.record_state = 'active'",
        );
        push_filters(&mut count_query, filters);
        let total = count_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await?;

        let mut completed_count_query: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT COUNT(*)::BIGINT
             FROM archived_readings r
             INNER JOIN users u ON u.id = r.owner_user_id
             LEFT JOIN corpus_relationships rel ON rel.id = r.relationship_id
             WHERE r.record_state = 'active'
               AND EXISTS (
                   SELECT 1
                   FROM archived_reading_editorial_states completed_editorial
                   WHERE completed_editorial.reading_id = r.id
                     AND completed_editorial.is_current
                     AND completed_editorial.state IN ('approved', 'published')
               )",
        );
        push_filters(&mut completed_count_query, filters);
        let completed_total = completed_count_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await?;

        Ok(LivingReadingListPage {
            items,
            total,
            completed_total,
            limit,
            offset,
        })
    }

    pub async fn get(&self, reading_id: Uuid) -> Result<Option<LivingReadingDetail>, Error> {
        let mut reading_query: QueryBuilder<Postgres> = QueryBuilder::new(LIST_SELECT);
        reading_query.push(" AND r.id = ").push_bind(reading_id);
        let Some(reading) = reading_query
            .build_query_as::<LivingReadingListRow>()
            .fetch_optional(&self.pool)
            .await?
            .map(LivingReadingListItem::from)
        else {
            return Ok(None);
        };

        let source = sqlx::query_as::<_, LivingReadingSource>(
            r#"
            SELECT
                id,
                stable_source_id,
                source_kind,
                locator,
                content_sha256,
                byte_size,
                media_type,
                observed_at,
                metadata
            FROM reading_sources
            WHERE id = $1
            "#,
        )
        .bind(reading.source_id)
        .fetch_one(&self.pool)
        .await?;

        let import_run = sqlx::query_as::<_, LivingReadingImportRun>(
            r#"
            SELECT
                id,
                manifest_id,
                manifest_schema_version,
                manifest_sha256,
                source_root_locator,
                state,
                stats,
                error_message,
                started_at,
                finished_at
            FROM reading_import_runs
            WHERE id = $1
            "#,
        )
        .bind(reading.import_run_id)
        .fetch_one(&self.pool)
        .await?;

        let relationships = sqlx::query_as::<_, LivingReadingRelationshipRow>(
            r#"
            SELECT
                rel.id,
                rel.relationship_key,
                rel.relationship_kind,
                rel.label,
                rel.reconciliation_state,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'subject_id', cs.id,
                            'canonical_name', cs.canonical_name,
                            'role', crm.member_role,
                            'position', crm.position
                        )
                        ORDER BY crm.position
                    ) FILTER (WHERE crm.subject_id IS NOT NULL),
                    '[]'::jsonb
                ) AS members
            FROM archived_readings r
            INNER JOIN corpus_relationships rel ON rel.id = r.relationship_id
            LEFT JOIN corpus_relationship_members crm ON crm.relationship_id = rel.id
            LEFT JOIN corpus_subjects cs ON cs.id = crm.subject_id
            WHERE r.id = $1
            GROUP BY rel.id
            "#,
        )
        .bind(reading_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(LivingReadingRelationship::from)
        .collect();

        let artifacts = sqlx::query_as::<_, LivingReadingArtifact>(
            r#"
            SELECT
                id,
                artifact_key,
                artifact_role,
                storage_provider,
                object_locator,
                content_sha256,
                byte_size,
                media_type,
                availability_state,
                metadata,
                created_at
            FROM archived_reading_artifacts
            WHERE reading_id = $1
            ORDER BY artifact_role, artifact_key
            "#,
        )
        .bind(reading_id)
        .fetch_all(&self.pool)
        .await?;

        let evidence = sqlx::query_as::<_, LivingReadingEvidence>(
            r#"
            SELECT
                id,
                source_id,
                artifact_id,
                evidence_key,
                evidence_type,
                claim,
                excerpt,
                review_state,
                confidence::DOUBLE PRECISION AS confidence,
                metadata,
                created_at
            FROM archived_reading_evidence
            WHERE reading_id = $1
            ORDER BY created_at, id
            "#,
        )
        .bind(reading_id)
        .fetch_all(&self.pool)
        .await?;

        let editorial_history = sqlx::query_as::<_, LivingReadingEditorialState>(
            r#"
            SELECT
                es.id,
                es.state,
                es.visibility,
                es.changed_by_user_id,
                u.email AS changed_by_email,
                es.change_role,
                es.revision,
                es.is_current,
                es.rationale,
                es.created_at
            FROM archived_reading_editorial_states es
            LEFT JOIN users u ON u.id = es.changed_by_user_id
            WHERE es.reading_id = $1
            ORDER BY es.revision DESC
            "#,
        )
        .bind(reading_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(LivingReadingDetail {
            access_reason:
                "Protected administrative archive access granted by admin:analytics:read"
                    .to_string(),
            reading,
            source,
            import_run,
            relationships,
            artifacts,
            evidence,
            editorial_history,
        }))
    }

    pub async fn create_invitation(
        &self,
        invitation: NewLivingReadingInvitation,
    ) -> Result<Option<LivingReadingInvitation>, Error> {
        sqlx::query_as::<_, LivingReadingInvitation>(
            r#"
            INSERT INTO archived_reading_invitations (
                reading_id, token_digest, created_by_user_id, expires_at
            )
            SELECT r.id, $2, $3, $4
            FROM archived_readings r
            WHERE r.id = $1
              AND r.record_state = 'active'
              AND EXISTS (
                  SELECT 1
                  FROM archived_reading_editorial_states es
                  WHERE es.reading_id = r.id
                    AND es.is_current
                    AND es.state IN ('approved', 'published')
              )
            RETURNING id, reading_id, created_by_user_id, expires_at,
                      revoked_at, revoked_by_user_id, created_at
            "#,
        )
        .bind(invitation.reading_id)
        .bind(invitation.token_digest)
        .bind(invitation.created_by_user_id)
        .bind(invitation.expires_at)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_invitable_publication(
        &self,
        reading_id: Uuid,
    ) -> Result<Option<LivingReadingPublicationArtifactCandidate>, Error> {
        let row = sqlx::query_as::<_, InvitationArtifactCandidateRow>(
            r#"
            SELECT
                artifact.storage_provider,
                artifact.object_locator,
                artifact.content_sha256,
                artifact.byte_size,
                artifact.media_type
            FROM archived_readings r
            INNER JOIN LATERAL (
                SELECT es.state
                FROM archived_reading_editorial_states es
                WHERE es.reading_id = r.id
                  AND es.is_current
                  AND es.state IN ('approved', 'published')
                LIMIT 1
            ) editorial ON true
            INNER JOIN LATERAL (
                SELECT
                    ara.storage_provider,
                    ara.object_locator,
                    ara.content_sha256,
                    ara.byte_size,
                    ara.media_type
                FROM archived_reading_artifacts ara
                WHERE ara.reading_id = r.id
                  AND ara.artifact_role = 'rendered_reading'
                  AND ara.storage_provider = 'filesystem'
                  AND ara.availability_state = 'available'
                ORDER BY
                    CASE LOWER(SPLIT_PART(COALESCE(ara.media_type, ''), ';', 1))
                        WHEN 'text/html' THEN 0
                        WHEN 'text/markdown' THEN 1
                        WHEN 'text/plain' THEN 2
                        ELSE 3
                    END,
                    ara.created_at DESC,
                    ara.id DESC
                LIMIT 1
            ) artifact ON true
            WHERE r.id = $1
              AND r.record_state = 'active'
            "#,
        )
        .bind(reading_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(
            row.map(|artifact| LivingReadingPublicationArtifactCandidate {
                storage_provider: artifact.storage_provider,
                object_locator: artifact.object_locator,
                content_sha256: artifact.content_sha256,
                byte_size: artifact.byte_size,
                media_type: artifact.media_type,
            }),
        )
    }

    pub async fn list_invitations(
        &self,
        reading_id: Uuid,
    ) -> Result<Vec<LivingReadingInvitation>, Error> {
        sqlx::query_as::<_, LivingReadingInvitation>(
            r#"
            SELECT id, reading_id, created_by_user_id, expires_at,
                   revoked_at, revoked_by_user_id, created_at
            FROM archived_reading_invitations
            WHERE reading_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
        )
        .bind(reading_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn revoke_invitation(
        &self,
        reading_id: Uuid,
        invitation_id: Uuid,
        actor_user_id: Option<Uuid>,
    ) -> Result<bool, Error> {
        let result = sqlx::query(
            r#"
            UPDATE archived_reading_invitations
            SET revoked_at = NOW(), revoked_by_user_id = $3
            WHERE id = $1 AND reading_id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(invitation_id)
        .bind(reading_id)
        .bind(actor_user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn resolve_invitation(
        &self,
        reading_id: Uuid,
        token_digest: &str,
    ) -> Result<Option<LivingReadingInvitationResolutionRecord>, Error> {
        #[derive(FromRow)]
        struct PublicRow {
            reading_id: Uuid,
            stable_reading_id: String,
            title: String,
            summary: Option<String>,
            reading_type: String,
            language_tag: String,
            editorial_state: Option<String>,
            subjects: Json<Vec<LivingReadingPublicSubject>>,
            relationship_label: Option<String>,
            storage_provider: Option<String>,
            object_locator: Option<String>,
            content_sha256: Option<String>,
            byte_size: Option<i64>,
            media_type: Option<String>,
        }

        let row = sqlx::query_as::<_, PublicRow>(
            r#"
            SELECT
                r.id AS reading_id,
                r.stable_reading_id,
                r.title,
                r.summary,
                r.reading_type,
                r.language_tag,
                editorial.state AS editorial_state,
                COALESCE(subjects.items, '[]'::jsonb) AS subjects,
                rel.label AS relationship_label,
                artifact.storage_provider,
                artifact.object_locator,
                artifact.content_sha256,
                artifact.byte_size,
                artifact.media_type
            FROM archived_reading_invitations invite
            INNER JOIN archived_readings r
                ON r.id = invite.reading_id AND r.record_state = 'active'
            LEFT JOIN corpus_relationships rel ON rel.id = r.relationship_id
            INNER JOIN LATERAL (
                SELECT es.state
                FROM archived_reading_editorial_states es
                WHERE es.reading_id = r.id
                  AND es.is_current
                  AND es.state IN ('approved', 'published')
                LIMIT 1
            ) editorial ON true
            LEFT JOIN LATERAL (
                SELECT jsonb_agg(
                    jsonb_build_object(
                        'canonical_name', cs.canonical_name,
                        'role', ars.subject_role
                    )
                    ORDER BY ars.subject_role, cs.canonical_name
                ) AS items
                FROM archived_reading_subjects ars
                INNER JOIN corpus_subjects cs ON cs.id = ars.subject_id
                WHERE ars.reading_id = r.id
            ) subjects ON true
            LEFT JOIN LATERAL (
                SELECT
                    ara.storage_provider,
                    ara.object_locator,
                    ara.content_sha256,
                    ara.byte_size,
                    ara.media_type
                FROM archived_reading_artifacts ara
                WHERE ara.reading_id = r.id
                  AND ara.artifact_role = 'rendered_reading'
                  AND ara.storage_provider = 'filesystem'
                  AND ara.availability_state = 'available'
                ORDER BY
                    CASE LOWER(SPLIT_PART(COALESCE(ara.media_type, ''), ';', 1))
                        WHEN 'text/html' THEN 0
                        WHEN 'text/markdown' THEN 1
                        WHEN 'text/plain' THEN 2
                        ELSE 3
                    END,
                    ara.created_at DESC,
                    ara.id DESC
                LIMIT 1
            ) artifact ON true
            WHERE invite.reading_id = $1
              AND invite.token_digest = $2
              AND invite.revoked_at IS NULL
              AND invite.expires_at > NOW()
            "#,
        )
        .bind(reading_id)
        .bind(token_digest)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| LivingReadingInvitationResolutionRecord {
            reading_id: row.reading_id,
            stable_reading_id: row.stable_reading_id,
            title: row.title,
            summary: row.summary,
            reading_type: row.reading_type,
            language_tag: row.language_tag,
            editorial_state: row.editorial_state,
            subjects: row.subjects.0,
            relationship_label: row.relationship_label,
            artifact: match (
                row.storage_provider,
                row.object_locator,
                row.content_sha256,
                row.byte_size,
                row.media_type,
            ) {
                (
                    Some(storage_provider),
                    Some(object_locator),
                    Some(content_sha256),
                    Some(byte_size),
                    media_type,
                ) => Some(LivingReadingPublicationArtifactCandidate {
                    storage_provider,
                    object_locator,
                    content_sha256,
                    byte_size,
                    media_type,
                }),
                _ => None,
            },
        }))
    }
}

fn push_filters(query: &mut QueryBuilder<'_, Postgres>, filters: &LivingReadingListFilters) {
    if let Some(search) = filters
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{search}%");
        query
            .push(" AND (r.title ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR r.stable_reading_id ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR u.email ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR rel.label ILIKE ")
            .push_bind(pattern.clone())
            .push(
                " OR EXISTS (
                    SELECT 1
                    FROM archived_reading_subjects search_ars
                    INNER JOIN corpus_subjects search_cs ON search_cs.id = search_ars.subject_id
                    LEFT JOIN corpus_subject_aliases search_alias
                        ON search_alias.subject_id = search_cs.id
                    WHERE search_ars.reading_id = r.id
                      AND (search_cs.canonical_name ILIKE ",
            )
            .push_bind(pattern.clone())
            .push(" OR search_alias.alias ILIKE ")
            .push_bind(pattern)
            .push(")))");
    }
    match filters.reading_type {
        Some(LivingReadingTypeFilter::Solo) => {
            query.push(" AND LOWER(BTRIM(r.reading_type)) = 'solo'");
        }
        Some(LivingReadingTypeFilter::Synastry) => {
            // `relationship` is the legacy archive value used before the 723
            // importer began preserving the canonical `synastry` type.
            query.push(" AND LOWER(BTRIM(r.reading_type)) IN ('synastry', 'relationship')");
        }
        None => {}
    }
    if let Some(owner_user_id) = filters.owner_user_id {
        query
            .push(" AND r.owner_user_id = ")
            .push_bind(owner_user_id);
    }
    if let Some(subject_id) = filters.subject_id {
        query
            .push(
                " AND EXISTS (SELECT 1 FROM archived_reading_subjects filter_subject \
                 WHERE filter_subject.reading_id = r.id AND filter_subject.subject_id = ",
            )
            .push_bind(subject_id)
            .push(")");
    }
    if let Some(relationship_id) = filters.relationship_id {
        query
            .push(" AND r.relationship_id = ")
            .push_bind(relationship_id);
    }
    if let Some(source_id) = filters.source_id {
        query
            .push(" AND r.primary_source_id = ")
            .push_bind(source_id);
    }
    if let Some(import_run_id) = filters.import_run_id {
        query
            .push(" AND r.import_run_id = ")
            .push_bind(import_run_id);
    }
    if let Some(editorial_state) = filters
        .editorial_state
        .as_deref()
        .map(str::trim)
        .filter(|state| !state.is_empty())
    {
        query
            .push(
                " AND EXISTS (SELECT 1 FROM archived_reading_editorial_states filter_editorial \
                 WHERE filter_editorial.reading_id = r.id \
                 AND filter_editorial.is_current \
                 AND filter_editorial.state = ",
            )
            .push_bind(editorial_state.to_string())
            .push(")");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_type_filter_is_bounded_to_normalized_archive_values() {
        let mut solo_query: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT r.id FROM archived_readings r WHERE TRUE");
        push_filters(
            &mut solo_query,
            &LivingReadingListFilters {
                reading_type: Some(LivingReadingTypeFilter::Solo),
                ..Default::default()
            },
        );
        assert!(solo_query
            .sql()
            .contains("LOWER(BTRIM(r.reading_type)) = 'solo'"));

        let mut synastry_query: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT r.id FROM archived_readings r WHERE TRUE");
        push_filters(
            &mut synastry_query,
            &LivingReadingListFilters {
                reading_type: Some(LivingReadingTypeFilter::Synastry),
                ..Default::default()
            },
        );
        assert!(synastry_query
            .sql()
            .contains("LOWER(BTRIM(r.reading_type)) IN ('synastry', 'relationship')"));
    }
}
