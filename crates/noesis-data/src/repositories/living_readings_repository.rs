use crate::models::living_reading::{
    LivingReadingArtifact, LivingReadingDetail, LivingReadingEditorialState, LivingReadingEvidence,
    LivingReadingImportRun, LivingReadingListFilters, LivingReadingListItem, LivingReadingListPage,
    LivingReadingRelationship, LivingReadingRelationshipMember, LivingReadingSource,
    LivingReadingSubject,
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
            "SELECT COUNT(*)::BIGINT FROM archived_readings r WHERE r.record_state = 'active'",
        );
        push_filters(&mut count_query, filters);
        let total = count_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await?;

        Ok(LivingReadingListPage {
            items,
            total,
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
}

fn push_filters(query: &mut QueryBuilder<'_, Postgres>, filters: &LivingReadingListFilters) {
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
