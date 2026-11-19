use noesis_data::repositories::admin_repository::{AdminRepository, NewApiKeyRecord};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

// Explicit opt-in disposable local database; never silently fall back to DATABASE_URL.
#[tokio::test]
async fn creation_and_audit_are_atomic() {
    let Ok(url) = std::env::var("ADMIN_KEY_TEST_DATABASE_URL") else {
        eprintln!("not run: ADMIN_KEY_TEST_DATABASE_URL is required");
        return;
    };
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    let schema = format!("key_test_{}", Uuid::new_v4().simple());
    sqlx::raw_sql(&format!("CREATE SCHEMA {schema}; SET search_path TO {schema};
        CREATE TABLE users (id uuid PRIMARY KEY, email text NOT NULL);
        CREATE TABLE api_keys (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), key_hash text UNIQUE NOT NULL,
        name text, key_prefix text, user_id uuid REFERENCES users(id), created_by_user_id uuid,
        tier text, permissions jsonb, consciousness_level int, rate_limit int, expires_at timestamptz,
        rotated_from_key_id uuid, is_active bool, created_at timestamptz DEFAULT now(), last_used timestamptz, revoked_at timestamptz, revoked_by_user_id uuid);
        CREATE TABLE user_roles (user_id uuid REFERENCES users(id), role text);
        CREATE TABLE user_profiles (user_id uuid PRIMARY KEY REFERENCES users(id), preferences jsonb, created_at timestamptz, updated_at timestamptz);
        CREATE TABLE api_key_events (id uuid DEFAULT gen_random_uuid(), key_id uuid REFERENCES api_keys(id),
        actor_user_id uuid, event_type text, metadata jsonb, created_at timestamptz);"))
        .execute(&pool).await.unwrap();
    let user = Uuid::new_v4();
    sqlx::query("INSERT INTO users VALUES ($1,'synthetic@example.test')")
        .bind(user)
        .execute(&pool)
        .await
        .unwrap();
    let repo = AdminRepository::new(pool.clone());
    let new_key = |hash: &str| NewApiKeyRecord {
        key_hash: hash.into(),
        name: Some("synthetic key".into()),
        key_prefix: "nk_test".into(),
        user_id: user,
        created_by_user_id: Some(user),
        tier: "enterprise".into(),
        permissions: serde_json::json!(["basic:access", "numerology:read"]),
        consciousness_level: 2,
        rate_limit: 10000,
        expires_at: None,
        rotated_from_key_id: None,
        tier_managed: false,
    };
    let created = repo
        .create_api_key(new_key("first"))
        .await
        .expect("creation insert must succeed");
    assert_eq!(created.name.as_deref(), Some("synthetic key"));
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM api_key_events WHERE key_id=$1")
        .bind(created.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    repo.set_user_roles_and_permissions(
        user,
        &["platform-admin".into()],
        &["basic:access".into(), "admin:*".into()],
    )
    .await
    .unwrap();
    let after = repo.get_api_key(created.id).await.unwrap().unwrap();
    assert!(
        after
            .permissions
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("numerology:read")),
        "role assignment must preserve engine scopes"
    );
    // Hold the owner lock as a tier change does; rotation must wait before reading privileges.
    let worker_schema = schema.clone();
    let workers = PgPoolOptions::new()
        .max_connections(2)
        .after_connect(move |conn, _| {
            let sql = format!("SET search_path TO {}", worker_schema);
            Box::pin(async move {
                sqlx::query(&sql).execute(conn).await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .unwrap();
    let mut downgrade = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE")
        .bind(user)
        .fetch_one(&mut *downgrade)
        .await
        .unwrap();
    let worker_repo = AdminRepository::new(workers.clone());
    let rotation = tokio::spawn(async move {
        worker_repo
            .rotate_api_key(created.id, "rotated", "nk_rotated", Some(user))
            .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(
        !rotation.is_finished(),
        "rotation must serialize with an owner tier change"
    );
    sqlx::query("UPDATE api_keys SET tier='free', permissions='[\"basic:access\",\"numerology:read\"]', consciousness_level=0, rate_limit=60 WHERE user_id=$1")
        .bind(user).execute(&mut *downgrade).await.unwrap();
    downgrade.commit().await.unwrap();
    let rotated = rotation.await.unwrap().unwrap().unwrap();
    assert_eq!(rotated.tier, "free");
    assert_eq!(rotated.consciousness_level, 0);
    assert_eq!(rotated.rate_limit, 60);
    workers.close().await;
    sqlx::query("ALTER TABLE api_key_events ADD CONSTRAINT reject_test_event CHECK (metadata->>'name' <> 'synthetic key') NOT VALID")
        .execute(&pool).await.unwrap();
    assert!(repo.create_api_key(new_key("must_rollback")).await.is_err());
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM api_keys WHERE key_hash='must_rollback'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0, "event failure must roll back the key insert");
    sqlx::raw_sql(&format!(
        "SET search_path TO public; DROP SCHEMA {schema} CASCADE"
    ))
    .execute(&pool)
    .await
    .unwrap();
}
