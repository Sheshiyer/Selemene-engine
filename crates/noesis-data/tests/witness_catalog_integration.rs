//! Additive synthetic PostgreSQL checks. Never truncate shared tables or use a production URL.
use noesis_data::{
    models::witness_catalog::*,
    repositories::witness_catalog_repository::{self as repo, WitnessCatalogError},
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres, Transaction};
use uuid::Uuid;

async fn tx(pool: &PgPool, actor: Uuid) -> Transaction<'_, Postgres> {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.account_id',$1,true)")
        .bind(actor.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    tx
}
async fn actor(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO users(id,email,password_hash) VALUES($1,$2,'synthetic-test-only')")
        .bind(id)
        .bind(format!("witness-{id}@example.test"))
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO user_account_state(user_id,state) VALUES($1,'active') ON CONFLICT(user_id) DO UPDATE SET state='active',locked_until=NULL").bind(id).execute(pool).await.unwrap();
    id
}
async fn grant(pool: &PgPool, owner: Uuid, subject: Uuid, actor: Uuid, edit: bool) {
    sqlx::query("INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,can_edit,can_calculate,grantor_user_id) VALUES($1,$2,$3,$4,$4,$2)")
        .bind(subject).bind(owner).bind(actor).bind(edit).execute(pool).await.unwrap();
}
fn publication(
    subject: Uuid,
    profile: &str,
    sha: char,
    expected: serde_json::Value,
) -> WitnessPublish {
    let hash = sha.to_string().repeat(64);
    WitnessPublish {
        expected_head: expected,
        approval: "owner".into(),
        artifact: WitnessArtifact {
            profile: profile.into(),
            kind: "daily".into(),
            date: "2026-11-07".into(),
            sha256: hash.clone(),
            object_key: format!("witness-v2/{subject}/{profile}/daily/2026-11-07/{hash}.json"),
            report_schema: "noesis.witness-report.v2".into(),
            packet_sha256: "d".repeat(64),
            contract_sha256: "e".repeat(64),
            effective_at: "2026-11-07T00:30:00Z".into(),
        },
    }
}
async fn publish(
    pool: &PgPool,
    actor: Uuid,
    subject: Uuid,
    p: &WitnessPublish,
) -> Result<WitnessPublication, WitnessCatalogError> {
    let mut t = tx(pool, actor).await;
    let result = repo::publish(&mut t, actor, subject, p).await?;
    t.commit().await.unwrap();
    Ok(result)
}
#[test]
fn rejects_malformed_descriptors_and_missing_cas() {
    let subject = Uuid::new_v4();
    let good = publication(subject, "synthetic-subject", 'a', json!(null));
    assert!(good.validate(subject, "synthetic-subject").is_ok());
    let mut p = good.clone();
    p.artifact.profile = "another".into();
    assert!(p.validate(subject, "synthetic-subject").is_err());
    let mut p = good.clone();
    p.artifact.object_key = "witness-v2/../secret.json".into();
    assert!(p.validate(subject, "synthetic-subject").is_err());
    let mut p = good.clone();
    p.artifact.effective_at = "2026-11-07T06:00:00Z".into();
    assert!(p.validate(subject, "synthetic-subject").is_err());
    let mut p = good.clone();
    p.artifact.date = "2026-02-30".into();
    assert!(p.validate(subject, "synthetic-subject").is_err());
    let mut p = good.clone();
    p.artifact.sha256 = "A".repeat(64);
    assert!(p.validate(subject, "synthetic-subject").is_err());
    let mut p = good.clone();
    p.expected_head = json!({"sha256":"a".repeat(64),"revision":0});
    assert!(p.expected().is_err());
    let mut payload = serde_json::to_value(good).unwrap();
    payload.as_object_mut().unwrap().remove("expected_head");
    assert!(serde_json::from_value::<WitnessPublish>(payload).is_err());
}
#[tokio::test]
async fn real_postgres_catalog_authorization_cas_rollback_and_rls() {
    let url = std::env::var("WITNESS_TEST_DATABASE_URL").expect(
        "explicit isolated WITNESS_TEST_DATABASE_URL required \
         (bash scripts/cloud/test-db.sh provisions it and prints the URL)",
    );
    assert!(
        url.contains("@127.0.0.1:") && url.ends_with("/noesis_identity_test"),
        "isolated local test database only"
    );
    let pool = PgPoolOptions::new()
        .max_connections(6)
        .connect(&url)
        .await
        .unwrap();
    let owner = actor(&pool).await;
    let reader = actor(&pool).await;
    let editor = actor(&pool).await;
    let outsider = actor(&pool).await;
    let subject = Uuid::new_v4();
    let profile = format!("synthetic-{}", subject.simple());
    sqlx::query("INSERT INTO corpus_subjects(id,owner_user_id,subject_key,canonical_name) VALUES($1,$2,$3,'Synthetic catalog subject')").bind(subject).bind(owner).bind(&profile).execute(&pool).await.unwrap();
    grant(&pool, owner, subject, owner, true).await;
    grant(&pool, owner, subject, reader, false).await;
    grant(&pool, owner, subject, editor, true).await;
    let first = publication(subject, &profile, 'a', json!(null));
    for actor in [reader, editor, outsider] {
        assert!(matches!(
            publish(&pool, actor, subject, &first).await,
            Err(WitnessCatalogError::Denied)
        ));
    }
    let accepted = publish(&pool, owner, subject, &first).await.unwrap();
    assert_eq!(accepted.entry.head_revision, 1);
    assert!(!accepted.idempotent);
    let retry = publish(&pool, owner, subject, &first).await.unwrap();
    assert!(retry.idempotent);
    assert_eq!(retry.entry.head_revision, 1);
    let mut t = tx(&pool, reader).await;
    let catalog = repo::catalog(&mut t, reader, subject).await.unwrap();
    assert_eq!(catalog.daily.len(), 1);
    assert!(catalog.folio.is_none());
    t.commit().await.unwrap();
    let mut t = tx(&pool, outsider).await;
    assert!(matches!(
        repo::catalog(&mut t, outsider, subject).await,
        Err(WitnessCatalogError::Denied)
    ));
    t.rollback().await.unwrap();
    // A different subject cannot claim an uploaded-but-unaccepted locator either.
    let unaccepted = publication(Uuid::new_v4(), &profile, '9', json!(null));
    assert!(matches!(
        publish(&pool, owner, subject, &unaccepted).await,
        Err(WitnessCatalogError::Invalid("INVALID_WITNESS_OBJECT_SCOPE"))
    ));
    // Two publishers with the same expected head cannot both replace it.
    let expected = json!({"sha256":"a".repeat(64),"revision":1});
    let second = publication(subject, &profile, 'b', expected.clone());
    let competing = publication(subject, &profile, 'c', expected);
    let (a, b) = tokio::join!(
        publish(&pool, owner, subject, &second),
        publish(&pool, owner, subject, &competing)
    );
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    let winner = if a.is_ok() { second } else { competing };
    assert!(matches!(
        if a.is_err() {
            a.err().unwrap()
        } else {
            b.err().unwrap()
        },
        WitnessCatalogError::Conflict("WITNESS_HEAD_CONFLICT")
    ));
    // Restore a known immutable prior version only against the exact current revision.
    let mut rollback = first.clone();
    rollback.expected_head = json!({"sha256":winner.artifact.sha256,"revision":2});
    let restored = publish(&pool, owner, subject, &rollback).await.unwrap();
    assert_eq!(restored.entry.sha256, first.artifact.sha256);
    assert_eq!(restored.entry.head_revision, 3);
    assert!(matches!(
        publish(&pool, owner, subject, &winner).await,
        Err(WitnessCatalogError::Conflict("WITNESS_HEAD_CONFLICT"))
    ));
    // Same bytes cannot acquire a different packet, schema or approval descriptor.
    let mut forged = first.clone();
    forged.expected_head = json!({"sha256":first.artifact.sha256,"revision":3});
    forged.approval = "automatic".into();
    assert!(matches!(
        publish(&pool, owner, subject, &forged).await,
        Err(WitnessCatalogError::Conflict(
            "IMMUTABLE_DESCRIPTOR_CONFLICT"
        ))
    ));
    // Granting publisher rights is independent of ordinary edit/calculate access.
    sqlx::query("INSERT INTO witness_publish_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id) VALUES($1,$2,$3,$2)").bind(subject).bind(owner).bind(editor).execute(&pool).await.unwrap();
    let mut folio = publication(subject, &profile, 'f', json!(null));
    folio.artifact.kind = "folio".into();
    folio.artifact.object_key = format!(
        "witness-v2/{subject}/{profile}/folio/2026-11-07/{}.json",
        folio.artifact.sha256
    );
    publish(&pool, editor, subject, &folio).await.unwrap();
    sqlx::query("UPDATE witness_publish_grants SET revoked_at=now() WHERE subject_id=$1 AND grantee_user_id=$2").bind(subject).bind(editor).execute(&pool).await.unwrap();
    assert!(matches!(
        publish(&pool, editor, subject, &folio).await,
        Err(WitnessCatalogError::Denied)
    ));
    // RLS under an actual non-superuser role, including table-owner FORCE RLS.
    // Role creation/grants/ownership change are all rolled back together.
    let role = format!("witness_test_{}", Uuid::new_v4().simple());
    let mut rls = pool.begin().await.unwrap();
    sqlx::query(&format!(
        "CREATE ROLE {role} NOLOGIN NOSUPERUSER NOBYPASSRLS"
    ))
    .execute(&mut *rls)
    .await
    .unwrap();
    sqlx::query(&format!("GRANT SELECT ON corpus_subjects,subject_grants,user_account_state,witness_publish_grants,witness_heads,witness_head_events TO {role}")).execute(&mut *rls).await.unwrap();
    sqlx::query(&format!("ALTER TABLE witness_versions OWNER TO {role}"))
        .execute(&mut *rls)
        .await
        .unwrap();
    sqlx::query(&format!("SET LOCAL ROLE {role}"))
        .execute(&mut *rls)
        .await
        .unwrap();
    sqlx::query("SELECT set_config('app.account_id',$1,true)")
        .bind(outsider.to_string())
        .execute(&mut *rls)
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM witness_versions WHERE subject_id=$1")
            .bind(subject)
            .fetch_one(&mut *rls)
            .await
            .unwrap();
    assert_eq!(count, 0);
    sqlx::query("SELECT set_config('app.account_id',$1,true)")
        .bind(reader.to_string())
        .execute(&mut *rls)
        .await
        .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM witness_versions WHERE subject_id=$1")
            .bind(subject)
            .fetch_one(&mut *rls)
            .await
            .unwrap();
    assert_eq!(count, 3);
    sqlx::query("SAVEPOINT forbidden_write")
        .execute(&mut *rls)
        .await
        .unwrap();
    let result=sqlx::query("INSERT INTO witness_versions(subject_id,owner_user_id,profile_key,kind,report_date,sha256,object_key,report_schema,packet_sha256,contract_sha256,effective_at,approval,published_by) SELECT subject_id,owner_user_id,profile_key,kind,report_date,repeat('9',64),replace(object_key,sha256,repeat('9',64)),report_schema,packet_sha256,contract_sha256,effective_at,approval,$2 FROM witness_versions WHERE subject_id=$1 LIMIT 1").bind(subject).bind(reader).execute(&mut *rls).await;
    assert_eq!(
        result
            .unwrap_err()
            .as_database_error()
            .unwrap()
            .code()
            .as_deref(),
        Some("42501")
    );
    sqlx::query("ROLLBACK TO SAVEPOINT forbidden_write")
        .execute(&mut *rls)
        .await
        .unwrap();
    rls.rollback().await.unwrap();
    // Even valid subjects belonging to different owners cannot claim an already
    // accepted private object locator with the same owner-scoped profile alias.
    let other_subject = Uuid::new_v4();
    sqlx::query("INSERT INTO corpus_subjects(id,owner_user_id,subject_key,canonical_name) VALUES($1,$2,$3,'Synthetic other subject')").bind(other_subject).bind(outsider).bind(&profile).execute(&pool).await.unwrap();
    grant(&pool, outsider, other_subject, outsider, true).await;
    assert!(matches!(
        publish(&pool, outsider, other_subject, &first).await,
        Err(WitnessCatalogError::Invalid("INVALID_WITNESS_OBJECT_SCOPE"))
    ));
    // Cross-owner links are rejected by composite FK even on a privileged connection.
    let mut t = tx(&pool, owner).await;
    let cross=sqlx::query("INSERT INTO witness_publish_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id) VALUES($1,$2,$3,$2)").bind(subject).bind(outsider).bind(outsider).execute(&mut *t).await;
    assert_eq!(
        cross
            .unwrap_err()
            .as_database_error()
            .unwrap()
            .code()
            .as_deref(),
        Some("23503")
    );
    t.rollback().await.unwrap();
    // Accepted history is immutable even if a connection can bypass RLS.
    let immutable =
        sqlx::query("UPDATE witness_versions SET approval='automatic' WHERE subject_id=$1")
            .bind(subject)
            .execute(&pool)
            .await;
    assert_eq!(
        immutable
            .unwrap_err()
            .as_database_error()
            .unwrap()
            .code()
            .as_deref(),
        Some("55000")
    );
    let events: i64 =
        sqlx::query_scalar("SELECT count(*) FROM witness_head_events WHERE subject_id=$1")
            .bind(subject)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(events, 4);
    // Revocation and account locking affect subsequent reads, with no stale pool context.
    sqlx::query(
        "UPDATE subject_grants SET revoked_at=now() WHERE subject_id=$1 AND grantee_user_id=$2",
    )
    .bind(subject)
    .bind(reader)
    .execute(&pool)
    .await
    .unwrap();
    let mut t = tx(&pool, reader).await;
    assert!(matches!(
        repo::catalog(&mut t, reader, subject).await,
        Err(WitnessCatalogError::Denied)
    ));
    t.rollback().await.unwrap();
    sqlx::query("UPDATE user_account_state SET state='locked',locked_until=now()+interval '1 hour' WHERE user_id=$1").bind(editor).execute(&pool).await.unwrap();
    let mut t = tx(&pool, editor).await;
    assert!(matches!(
        repo::catalog(&mut t, editor, subject).await,
        Err(WitnessCatalogError::Denied)
    ));
    t.commit().await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let context: Option<String> =
        sqlx::query_scalar("SELECT nullif(current_setting('app.account_id',true),'')")
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert_eq!(context, None);
}
