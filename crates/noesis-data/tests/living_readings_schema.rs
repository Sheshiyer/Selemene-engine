use noesis_data::models::living_reading::{LivingReadingListFilters, LivingReadingTypeFilter};
use noesis_data::repositories::living_readings_repository::LivingReadingsRepository;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, Executor, PgConnection, Row};
use uuid::Uuid;

const MIGRATION: &str = include_str!("../../../migrations/036_living_readings_archive.sql");

const EXPECTED_TABLES: &[&str] = &[
    "reading_import_runs",
    "reading_sources",
    "corpus_subjects",
    "corpus_subject_aliases",
    "corpus_relationships",
    "corpus_relationship_members",
    "archived_readings",
    "archived_reading_subjects",
    "archived_reading_artifacts",
    "archived_reading_evidence",
    "archived_reading_editorial_states",
];

fn down_sql() -> String {
    MIGRATION
        .split_once("-- <down>")
        .expect("migration must declare a DOWN block")
        .1
        .split_once("-- </down>")
        .expect("migration must close its DOWN block")
        .0
        .lines()
        .filter_map(|line| line.trim().strip_prefix("-- "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn living_readings_schema_declares_archive_contracts() {
    for table in EXPECTED_TABLES {
        assert!(
            MIGRATION.contains(&format!("CREATE TABLE {table}")),
            "missing archive table {table}"
        );
        assert!(
            down_sql().contains(&format!("DROP TABLE IF EXISTS {table};")),
            "DOWN block must remove {table}"
        );
    }

    assert!(MIGRATION.contains("UNIQUE (owner_user_id, idempotency_key)"));
    assert!(MIGRATION.contains("UNIQUE (owner_user_id, manifest_sha256)"));
    assert!(MIGRATION.contains("UNIQUE (owner_user_id, object_locator)"));
    assert!(MIGRATION.contains("CREATE INDEX idx_reading_sources_content_sha256"));
    assert!(MIGRATION.contains("CREATE INDEX idx_archived_readings_content_sha256"));
    assert!(MIGRATION.contains("CREATE INDEX idx_archived_reading_artifacts_sha256"));
    assert!(!MIGRATION.contains("uq_reading_sources_run_content_sha256"));
    assert!(!MIGRATION.contains("owner_content_sha256_key"));
    assert!(MIGRATION.contains("CHECK (member_role IN"));
    assert!(MIGRATION.contains("CHECK (subject_role IN"));
    assert!(MIGRATION.contains("CHECK (state IN ('imported'"));
    assert!(MIGRATION.contains("corpus_subject_aliases_subject_owner_fk"));
    assert!(MIGRATION.contains("corpus_subject_aliases_source_owner_fk"));
    assert!(MIGRATION.contains("ON DELETE CASCADE"));
    assert!(MIGRATION.contains("ON DELETE RESTRICT"));

    let subjects_definition = MIGRATION
        .split_once("CREATE TABLE corpus_subjects")
        .expect("subjects table")
        .1
        .split_once(");")
        .expect("subjects table terminator")
        .0;
    assert!(
        !subjects_definition.contains("subject_user_id")
            && !subjects_definition.contains("linked_user_id"),
        "a corpus subject must never be represented by a users row"
    );

    let artifacts_definition = MIGRATION
        .split_once("CREATE TABLE archived_reading_artifacts")
        .expect("artifacts table")
        .1
        .split_once(");")
        .expect("artifacts table terminator")
        .0;
    for forbidden_bytes_column in [" BYTEA", " body ", " payload ", " content_bytes"] {
        assert!(
            !artifacts_definition.contains(forbidden_bytes_column),
            "artifact table must store locators and checksums, not bytes"
        );
    }
}

#[test]
fn living_readings_down_block_follows_dependency_order() {
    let down = down_sql();
    let positions = EXPECTED_TABLES
        .iter()
        .map(|table| {
            down.find(&format!("DROP TABLE IF EXISTS {table};"))
                .unwrap_or_else(|| panic!("missing DROP for {table}"))
        })
        .collect::<Vec<_>>();

    // EXPECTED_TABLES is parent-first for the UP graph; DOWN must be child-first.
    for pair in positions.windows(2) {
        assert!(
            pair[0] > pair[1],
            "DOWN block is not the reverse of the dependency order"
        );
    }
}

#[tokio::test]
async fn living_readings_schema_apply_round_trip_and_rollback() {
    let database_url =
        match std::env::var("TEST_DATABASE_URL").or_else(|_| std::env::var("DATABASE_URL")) {
            Ok(url) => url,
            Err(_) => {
                eprintln!(
                "Skipping live living-readings schema test: TEST_DATABASE_URL/DATABASE_URL not set"
            );
                return;
            }
        };

    let mut connection = PgConnection::connect(&database_url)
        .await
        .expect("connect to PostgreSQL test database");
    let schema = format!("living_readings_test_{}", Uuid::new_v4().simple());

    connection
        .execute(format!("CREATE SCHEMA {schema}").as_str())
        .await
        .expect("create isolated schema");
    connection
        .execute(format!("SET search_path TO {schema}, public").as_str())
        .await
        .expect("select isolated schema");
    connection
        .execute(
            r#"
            CREATE TABLE users (
                id UUID PRIMARY KEY,
                email TEXT NOT NULL UNIQUE
            );
            CREATE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = CURRENT_TIMESTAMP;
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await
        .expect("create migration prerequisites");
    connection
        .execute(MIGRATION)
        .await
        .expect("apply living-readings migration");

    let owner_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email) VALUES ($1, 'owner@example.com')")
        .bind(owner_id)
        .execute(&mut connection)
        .await
        .expect("insert owner account");

    let import_run_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO reading_import_runs (
            owner_user_id,
            manifest_id,
            idempotency_key,
            manifest_schema_version,
            manifest_sha256,
            source_root_locator,
            state
        )
        VALUES ($1, 'manifest-fixture', 'fixture-run', '1', $2, 'file:///fixture', 'running')
        RETURNING id
        "#,
    )
    .bind(owner_id)
    .bind("a".repeat(64))
    .fetch_one(&mut connection)
    .await
    .expect("insert import run");

    let duplicate_run = sqlx::query(
        r#"
        INSERT INTO reading_import_runs (
            owner_user_id,
            manifest_id,
            idempotency_key,
            manifest_schema_version,
            manifest_sha256,
            source_root_locator
        )
        VALUES ($1, 'different-manifest', 'fixture-run', '1', $2, 'file:///fixture')
        "#,
    )
    .bind(owner_id)
    .bind("b".repeat(64))
    .execute(&mut connection)
    .await;
    assert!(
        duplicate_run.is_err(),
        "idempotency key must reject a duplicate import run"
    );

    let source_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO reading_sources (
            import_run_id,
            owner_user_id,
            stable_source_id,
            source_kind,
            locator,
            content_sha256,
            byte_size,
            media_type
        )
        VALUES ($1, $2, 'source-fixture', 'file', 'file:///fixture/reading.md', $3, 42, 'text/markdown')
        RETURNING id
        "#,
    )
    .bind(import_run_id)
    .bind(owner_id)
    .bind("c".repeat(64))
    .fetch_one(&mut connection)
    .await
    .expect("insert source");

    let primary_subject_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO corpus_subjects (owner_user_id, subject_key, canonical_name)
        VALUES ($1, 'subject-primary', 'Primary Subject')
        RETURNING id
        "#,
    )
    .bind(owner_id)
    .fetch_one(&mut connection)
    .await
    .expect("insert primary corpus subject");
    let secondary_subject_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO corpus_subjects (owner_user_id, subject_key, canonical_name)
        VALUES ($1, 'subject-secondary', 'Secondary Subject')
        RETURNING id
        "#,
    )
    .bind(owner_id)
    .fetch_one(&mut connection)
    .await
    .expect("insert secondary corpus subject");

    sqlx::query(
        r#"
        INSERT INTO corpus_subject_aliases (
            subject_id, source_id, owner_user_id, alias, normalized_alias, review_state
        )
        VALUES ($1, $2, $3, 'Primary', 'primary', 'verified')
        "#,
    )
    .bind(primary_subject_id)
    .bind(source_id)
    .bind(owner_id)
    .execute(&mut connection)
    .await
    .expect("insert subject alias");

    sqlx::query(
        r#"
        INSERT INTO reading_sources (
            import_run_id,
            owner_user_id,
            stable_source_id,
            source_kind,
            locator,
            content_sha256,
            byte_size,
            media_type
        )
        VALUES ($1, $2, 'source-fixture-copy', 'file', 'file:///fixture/reading-copy.md', $3, 42, 'text/markdown')
        "#,
    )
    .bind(import_run_id)
    .bind(owner_id)
    .bind("c".repeat(64))
    .execute(&mut connection)
    .await
    .expect("identical source bytes at another locator must be representable");

    let other_owner_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, email) VALUES ($1, 'other@example.com')")
        .bind(other_owner_id)
        .execute(&mut connection)
        .await
        .expect("insert second owner account");
    let other_run_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO reading_import_runs (
            owner_user_id,
            manifest_id,
            idempotency_key,
            manifest_schema_version,
            manifest_sha256,
            source_root_locator
        )
        VALUES ($1, 'manifest-other', 'other-run', '1', $2, 'file:///other')
        RETURNING id
        "#,
    )
    .bind(other_owner_id)
    .bind("f".repeat(64))
    .fetch_one(&mut connection)
    .await
    .expect("insert second owner's import run");
    let other_source_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO reading_sources (
            import_run_id,
            owner_user_id,
            stable_source_id,
            source_kind,
            locator,
            content_sha256
        )
        VALUES ($1, $2, 'source-other', 'file', 'file:///other/reading.md', $3)
        RETURNING id
        "#,
    )
    .bind(other_run_id)
    .bind(other_owner_id)
    .bind("1".repeat(64))
    .fetch_one(&mut connection)
    .await
    .expect("insert second owner's source");
    let cross_owner_alias = sqlx::query(
        r#"
        INSERT INTO corpus_subject_aliases (
            subject_id, source_id, owner_user_id, alias, normalized_alias
        )
        VALUES ($1, $2, $3, 'Cross Owner', 'cross owner')
        "#,
    )
    .bind(primary_subject_id)
    .bind(other_source_id)
    .bind(owner_id)
    .execute(&mut connection)
    .await;
    assert!(
        cross_owner_alias.is_err(),
        "alias provenance must not cross archive owners"
    );

    let relationship_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO corpus_relationships (
            owner_user_id, relationship_key, relationship_kind, label
        )
        VALUES ($1, 'relationship-fixture', 'synastry', 'Fixture Dyad')
        RETURNING id
        "#,
    )
    .bind(owner_id)
    .fetch_one(&mut connection)
    .await
    .expect("insert relationship");

    sqlx::query(
        r#"
        INSERT INTO corpus_relationship_members (
            relationship_id, subject_id, owner_user_id, member_role, position
        )
        VALUES
            ($1, $2, $4, 'primary', 1),
            ($1, $3, $4, 'secondary', 2)
        "#,
    )
    .bind(relationship_id)
    .bind(primary_subject_id)
    .bind(secondary_subject_id)
    .bind(owner_id)
    .execute(&mut connection)
    .await
    .expect("insert relationship members");

    let reading_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO archived_readings (
            owner_user_id,
            import_run_id,
            primary_source_id,
            relationship_id,
            stable_reading_id,
            title,
            reading_type,
            producer_kind,
            producer_ref,
            content_locator,
            content_sha256
        )
        VALUES (
            $1, $2, $3, $4, 'reading-fixture', 'Fixture Reading', 'relationship',
            'imported', 'historical-corpus', 'r2://archive/reading-fixture.json', $5
        )
        RETURNING id
        "#,
    )
    .bind(owner_id)
    .bind(import_run_id)
    .bind(source_id)
    .bind(relationship_id)
    .bind("d".repeat(64))
    .fetch_one(&mut connection)
    .await
    .expect("insert archived reading");

    sqlx::query(
        r#"
        INSERT INTO archived_readings (
            owner_user_id,
            import_run_id,
            primary_source_id,
            stable_reading_id,
            title,
            reading_type,
            producer_kind,
            content_locator,
            content_sha256
        )
        VALUES (
            $1, $2, $3, 'reading-fixture-copy', 'Fixture Reading Copy', 'relationship',
            'imported', 'r2://archive/reading-fixture-copy.json', $4
        )
        "#,
    )
    .bind(owner_id)
    .bind(import_run_id)
    .bind(source_id)
    .bind("d".repeat(64))
    .execute(&mut connection)
    .await
    .expect("identical reading bytes at another locator must be representable");

    sqlx::query(
        r#"
        INSERT INTO archived_reading_subjects (
            reading_id, subject_id, owner_user_id, subject_role, confidence
        )
        VALUES
            ($1, $2, $4, 'primary', 1),
            ($1, $3, $4, 'secondary', 1)
        "#,
    )
    .bind(reading_id)
    .bind(primary_subject_id)
    .bind(secondary_subject_id)
    .bind(owner_id)
    .execute(&mut connection)
    .await
    .expect("link reading subjects");

    let artifact_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO archived_reading_artifacts (
            reading_id,
            owner_user_id,
            artifact_key,
            artifact_role,
            storage_provider,
            object_locator,
            content_sha256,
            byte_size,
            media_type
        )
        VALUES (
            $1, $2, 'rendered-fixture', 'rendered_reading', 'r2',
            'r2://archive/rendered-fixture.pdf', $3, 512, 'application/pdf'
        )
        RETURNING id
        "#,
    )
    .bind(reading_id)
    .bind(owner_id)
    .bind("e".repeat(64))
    .fetch_one(&mut connection)
    .await
    .expect("insert artifact metadata");

    sqlx::query(
        r#"
        INSERT INTO archived_reading_artifacts (
            reading_id,
            owner_user_id,
            artifact_key,
            artifact_role,
            storage_provider,
            object_locator,
            content_sha256,
            byte_size,
            media_type
        )
        VALUES (
            $1, $2, 'rendered-fixture-copy', 'attachment', 'r2',
            'r2://archive/rendered-fixture-copy.pdf', $3, 512, 'application/pdf'
        )
        "#,
    )
    .bind(reading_id)
    .bind(owner_id)
    .bind("e".repeat(64))
    .execute(&mut connection)
    .await
    .expect("identical artifact bytes at another locator must be representable");

    sqlx::query(
        r#"
        INSERT INTO archived_reading_evidence (
            reading_id,
            owner_user_id,
            source_id,
            artifact_id,
            evidence_key,
            evidence_type,
            claim,
            review_state
        )
        VALUES (
            $1, $2, $3, $4, 'evidence-fixture', 'citation',
            'Fixture provenance was preserved.', 'supported'
        )
        "#,
    )
    .bind(reading_id)
    .bind(owner_id)
    .bind(source_id)
    .bind(artifact_id)
    .execute(&mut connection)
    .await
    .expect("insert evidence");

    sqlx::query(
        r#"
        INSERT INTO archived_reading_editorial_states (
            reading_id,
            owner_user_id,
            state,
            visibility,
            changed_by_user_id,
            change_role,
            revision
        )
        VALUES ($1, $2, 'approved', 'owner_only', $2, 'owner', 1)
        "#,
    )
    .bind(reading_id)
    .bind(owner_id)
    .execute(&mut connection)
    .await
    .expect("insert editorial state");

    let round_trip = sqlx::query(
        r#"
        SELECT
            r.stable_reading_id,
            COUNT(DISTINCT rs.subject_id) AS subject_count,
            COUNT(DISTINCT a.id) AS artifact_count,
            COUNT(DISTINCT e.id) AS evidence_count,
            MAX(es.state) AS editorial_state
        FROM archived_readings r
        JOIN archived_reading_subjects rs ON rs.reading_id = r.id
        JOIN archived_reading_artifacts a ON a.reading_id = r.id
        JOIN archived_reading_evidence e ON e.reading_id = r.id
        JOIN archived_reading_editorial_states es
          ON es.reading_id = r.id AND es.is_current
        WHERE r.id = $1
        GROUP BY r.id, r.stable_reading_id
        "#,
    )
    .bind(reading_id)
    .fetch_one(&mut connection)
    .await
    .expect("round-trip archived reading");
    assert_eq!(
        round_trip.get::<String, _>("stable_reading_id"),
        "reading-fixture"
    );
    assert_eq!(round_trip.get::<i64, _>("subject_count"), 2);
    assert_eq!(round_trip.get::<i64, _>("artifact_count"), 2);
    assert_eq!(round_trip.get::<i64, _>("evidence_count"), 1);
    assert_eq!(round_trip.get::<String, _>("editorial_state"), "approved");

    let repository_schema = schema.clone();
    let repository_pool = PgPoolOptions::new()
        .max_connections(1)
        .after_connect(move |connection, _| {
            let search_path = format!("SET search_path TO {repository_schema}, public");
            Box::pin(async move {
                connection.execute(search_path.as_str()).await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await
        .expect("connect repository to isolated schema");
    let repository = LivingReadingsRepository::new(repository_pool.clone());
    let page = repository
        .list(
            &LivingReadingListFilters {
                query: None,
                reading_type: Some(LivingReadingTypeFilter::Synastry),
                owner_user_id: Some(owner_id),
                subject_id: Some(primary_subject_id),
                relationship_id: Some(relationship_id),
                source_id: Some(source_id),
                import_run_id: Some(import_run_id),
                editorial_state: Some("approved".to_string()),
            },
            500,
            -10,
        )
        .await
        .expect("list living readings through typed repository");
    assert_eq!(page.total, 1);
    assert_eq!(page.completed_total, 1);
    assert_eq!(page.limit, 100, "repository must bound page size");
    assert_eq!(page.offset, 0, "repository must normalize negative offset");
    assert_eq!(page.items[0].id, reading_id);
    assert_eq!(page.items[0].subjects.len(), 2);

    let detail = repository
        .get(reading_id)
        .await
        .expect("fetch living reading detail")
        .expect("seeded living reading exists");
    assert!(detail.access_reason.contains("admin:analytics:read"));
    assert_eq!(detail.source.id, source_id);
    assert_eq!(detail.import_run.id, import_run_id);
    assert_eq!(detail.relationships.len(), 1);
    assert_eq!(detail.artifacts.len(), 2);
    assert_eq!(detail.evidence.len(), 1);
    assert_eq!(detail.editorial_history.len(), 1);
    repository_pool.close().await;

    connection
        .execute(down_sql().as_str())
        .await
        .expect("apply dependency-ordered DOWN block");
    let remaining: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM information_schema.tables
        WHERE table_schema = current_schema()
          AND table_name = ANY($1)
        "#,
    )
    .bind(EXPECTED_TABLES)
    .fetch_one(&mut connection)
    .await
    .expect("verify archive tables removed");
    assert_eq!(
        remaining, 0,
        "DOWN block must remove the complete archive schema"
    );

    connection
        .execute(format!("DROP SCHEMA {schema} CASCADE").as_str())
        .await
        .expect("drop isolated schema");
}
