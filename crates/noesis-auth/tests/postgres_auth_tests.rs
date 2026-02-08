//! Integration tests for Postgres-backed API key validation in AuthService.
//!
//! Tests that require a live database are gated behind the `postgres` feature
//! and the `TEST_DATABASE_URL` environment variable. They are skipped at runtime
//! when the variable is not set.
//!
//! Run all tests:
//!   TEST_DATABASE_URL=postgres://... cargo test --package noesis-auth --features postgres
//!
//! Run without database (in-memory only):
//!   cargo test --package noesis-auth --features postgres

#[cfg(feature = "postgres")]
mod postgres_tests {
    use noesis_auth::{sha256_hex, AuthService, ApiKey};
    use chrono::Utc;
    use std::time::Duration;

    // ---------------------------------------------------------------------------
    // Helper: get a PgPool or return None when TEST_DATABASE_URL is unset
    // ---------------------------------------------------------------------------
    async fn maybe_pool() -> Option<sqlx::PgPool> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        sqlx::PgPool::connect(&url).await.ok()
    }

    /// Insert a test API key and return the raw key string.
    async fn insert_test_key(
        pool: &sqlx::PgPool,
        tier: &str,
        is_active: bool,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> String {
        let raw_key = format!("test_{}", uuid::Uuid::new_v4());
        let key_hash = sha256_hex(&raw_key);

        sqlx::query(
            "INSERT INTO api_keys (key_hash, user_id, tier, permissions, consciousness_level, rate_limit, is_active, expires_at) \
             VALUES ($1, gen_random_uuid(), $2, $3, $4, $5, $6, $7)"
        )
        .bind(&key_hash)
        .bind(tier)
        .bind(serde_json::json!(["basic:access"]))
        .bind(0_i32)
        .bind(100_i32)
        .bind(is_active)
        .bind(expires_at)
        .execute(pool)
        .await
        .expect("failed to insert test API key");

        raw_key
    }

    /// Clean up a test key by hash.
    async fn cleanup_key(pool: &sqlx::PgPool, raw_key: &str) {
        let key_hash = sha256_hex(raw_key);
        let _ = sqlx::query("DELETE FROM api_keys WHERE key_hash = $1")
            .bind(&key_hash)
            .execute(pool)
            .await;
    }

    // ---------------------------------------------------------------------------
    // SHA-256 hashing (no database required)
    // ---------------------------------------------------------------------------

    #[test]
    fn test_sha256_hex_deterministic() {
        let hash1 = sha256_hex("test-api-key-12345");
        let hash2 = sha256_hex("test-api-key-12345");
        assert_eq!(hash1, hash2, "same input must produce same hash");
    }

    #[test]
    fn test_sha256_hex_length() {
        let hash = sha256_hex("any-input");
        assert_eq!(hash.len(), 64, "SHA-256 hex digest must be 64 chars");
    }

    #[test]
    fn test_sha256_hex_different_inputs() {
        let h1 = sha256_hex("key-a");
        let h2 = sha256_hex("key-b");
        assert_ne!(h1, h2, "different inputs must produce different hashes");
    }

    #[test]
    fn test_sha256_hex_known_vector() {
        // SHA-256 of "" = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let hash = sha256_hex("");
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    // ---------------------------------------------------------------------------
    // In-memory fallback (no database required)
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn test_memory_fallback_valid_key() {
        let auth = AuthService::with_pool("test-secret".to_string(), None);
        let api_key = ApiKey {
            key: "mem-key-123".to_string(),
            user_id: "user-1".to_string(),
            tier: "free".to_string(),
            permissions: vec!["basic:access".to_string()],
            created_at: Utc::now(),
            expires_at: None,
            last_used: None,
            rate_limit: 60,
            consciousness_level: 0,
        };
        auth.add_api_key(api_key).await.unwrap();

        let user = auth.validate_api_key("mem-key-123").await.unwrap();
        assert_eq!(user.user_id, "user-1");
        assert_eq!(user.tier, "free");
    }

    #[tokio::test]
    async fn test_memory_fallback_invalid_key() {
        let auth = AuthService::with_pool("test-secret".to_string(), None);
        let result = auth.validate_api_key("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_memory_fallback_expired_key() {
        let auth = AuthService::with_pool("test-secret".to_string(), None);
        let api_key = ApiKey {
            key: "expired-key".to_string(),
            user_id: "user-2".to_string(),
            tier: "free".to_string(),
            permissions: vec![],
            created_at: Utc::now(),
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
            last_used: None,
            rate_limit: 60,
            consciousness_level: 0,
        };
        auth.add_api_key(api_key).await.unwrap();

        let result = auth.validate_api_key("expired-key").await;
        assert!(result.is_err(), "expired key should be rejected");
    }

    #[tokio::test]
    async fn test_jwt_unchanged_with_pool() {
        // JWT validation must work identically regardless of pool presence
        let secret = "test-jwt-secret-at-least-32-chars-long".to_string();
        let auth = AuthService::with_pool(secret.clone(), None);

        let token = auth.generate_jwt_token("user-1", "free", &["basic:access".to_string()], 0).unwrap();
        let user = auth.validate_jwt_token(&token).await.unwrap();
        assert_eq!(user.user_id, "user-1");
        assert_eq!(user.tier, "free");
    }

    // ---------------------------------------------------------------------------
    // Postgres integration tests (require TEST_DATABASE_URL)
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn test_valid_key_validation() {
        let Some(pool) = maybe_pool().await else {
            eprintln!("SKIP: TEST_DATABASE_URL not set");
            return;
        };

        let raw_key = insert_test_key(&pool, "free", true, None).await;
        let auth = AuthService::with_pool("secret".to_string(), Some(pool.clone()));

        let user = auth.validate_api_key(&raw_key).await.unwrap();
        assert_eq!(user.tier, "free");
        assert_eq!(user.rate_limit, 100);
        assert_eq!(user.consciousness_level, 0);

        cleanup_key(&pool, &raw_key).await;
    }

    #[tokio::test]
    async fn test_expired_key_rejection() {
        let Some(pool) = maybe_pool().await else {
            eprintln!("SKIP: TEST_DATABASE_URL not set");
            return;
        };

        let expired = Utc::now() - chrono::Duration::hours(1);
        let raw_key = insert_test_key(&pool, "free", true, Some(expired)).await;
        let auth = AuthService::with_pool("secret".to_string(), Some(pool.clone()));

        let result = auth.validate_api_key(&raw_key).await;
        assert!(result.is_err(), "expired key must be rejected");

        cleanup_key(&pool, &raw_key).await;
    }

    #[tokio::test]
    async fn test_inactive_key_rejection() {
        let Some(pool) = maybe_pool().await else {
            eprintln!("SKIP: TEST_DATABASE_URL not set");
            return;
        };

        let raw_key = insert_test_key(&pool, "free", false, None).await;
        let auth = AuthService::with_pool("secret".to_string(), Some(pool.clone()));

        let result = auth.validate_api_key(&raw_key).await;
        assert!(result.is_err(), "inactive key must be rejected");

        cleanup_key(&pool, &raw_key).await;
    }

    #[tokio::test]
    async fn test_last_used_update() {
        let Some(pool) = maybe_pool().await else {
            eprintln!("SKIP: TEST_DATABASE_URL not set");
            return;
        };

        let raw_key = insert_test_key(&pool, "premium", true, None).await;
        let key_hash = sha256_hex(&raw_key);
        let auth = AuthService::with_pool("secret".to_string(), Some(pool.clone()));

        // Validate key (triggers async last_used update)
        auth.validate_api_key(&raw_key).await.unwrap();

        // Give the spawned task time to execute
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Check last_used was set
        let row: (Option<chrono::DateTime<Utc>>,) =
            sqlx::query_as("SELECT last_used FROM api_keys WHERE key_hash = $1")
                .bind(&key_hash)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(row.0.is_some(), "last_used should be set after validation");

        cleanup_key(&pool, &raw_key).await;
    }

    #[tokio::test]
    async fn test_concurrent_validation() {
        let Some(pool) = maybe_pool().await else {
            eprintln!("SKIP: TEST_DATABASE_URL not set");
            return;
        };

        let raw_key = insert_test_key(&pool, "enterprise", true, None).await;
        let auth = std::sync::Arc::new(
            AuthService::with_pool("secret".to_string(), Some(pool.clone()))
        );

        let mut handles = Vec::new();
        for _ in 0..10 {
            let auth_clone = auth.clone();
            let key_clone = raw_key.clone();
            handles.push(tokio::spawn(async move {
                auth_clone.validate_api_key(&key_clone).await
            }));
        }

        let results: Vec<_> = futures::future::join_all(handles).await;
        for result in &results {
            let user = result.as_ref().unwrap().as_ref().unwrap();
            assert_eq!(user.tier, "enterprise");
        }

        cleanup_key(&pool, &raw_key).await;
    }

    #[tokio::test]
    async fn test_nonexistent_key_postgres() {
        let Some(pool) = maybe_pool().await else {
            eprintln!("SKIP: TEST_DATABASE_URL not set");
            return;
        };

        let auth = AuthService::with_pool("secret".to_string(), Some(pool));
        let result = auth.validate_api_key("totally-fake-key-that-does-not-exist").await;
        assert!(result.is_err());
    }
}
