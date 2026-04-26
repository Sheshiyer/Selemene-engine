//! Issue a new API key by cloning tier/permissions/limits/user_id
//! from an existing active key.
//!
//! Usage:
//!   DATABASE_URL=postgres://... cargo run --package noesis-auth --features postgres --example issue_api_key_from_existing -- <EXISTING_KEY>

use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
struct ExistingKeyRecord {
    user_id: uuid::Uuid,
    tier: String,
    permissions: serde_json::Value,
    consciousness_level: i32,
    rate_limit: i32,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    is_active: bool,
}

fn generate_random_key(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let existing_key = std::env::args()
        .nth(1)
        .ok_or("Usage: issue_api_key_from_existing <EXISTING_KEY>")?;

    if !existing_key.starts_with("nk_") {
        return Err("EXISTING_KEY must start with nk_".into());
    }

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
    let pool = PgPool::connect(&database_url).await?;

    let existing_hash = sha256_hex(&existing_key);

    let existing: ExistingKeyRecord = sqlx::query_as(
        "SELECT user_id, tier, permissions, consciousness_level, rate_limit, expires_at, is_active \
         FROM api_keys WHERE key_hash = $1",
    )
    .bind(&existing_hash)
    .fetch_optional(&pool)
    .await?
    .ok_or("Existing key hash not found in api_keys")?;

    if !existing.is_active {
        return Err("Existing key is inactive; cannot clone".into());
    }

    let new_key = format!("nk_{}", generate_random_key(32));
    let new_hash = sha256_hex(&new_key);

    sqlx::query(
        "INSERT INTO api_keys (key_hash, user_id, tier, permissions, consciousness_level, rate_limit, expires_at, is_active) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, true)",
    )
    .bind(&new_hash)
    .bind(existing.user_id)
    .bind(&existing.tier)
    .bind(&existing.permissions)
    .bind(existing.consciousness_level)
    .bind(existing.rate_limit)
    .bind(existing.expires_at)
    .execute(&pool)
    .await?;

    println!("Key issued.");
    println!("New key: {}", new_key);
    println!("New key hash: {}", new_hash);
    println!("Tier: {}", existing.tier);

    Ok(())
}
