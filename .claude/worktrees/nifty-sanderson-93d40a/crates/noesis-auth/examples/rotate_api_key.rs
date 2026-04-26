//! Rotate a single API key in PostgreSQL by revoking the old key hash
//! and issuing one new key with identical tier/permissions/limits.
//!
//! Usage:
//!   DATABASE_URL=postgres://... cargo run --package noesis-auth --features postgres --example rotate_api_key -- <OLD_KEY>

use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::fs::OpenOptions;
use std::io::Write;

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
    let old_key = std::env::args()
        .nth(1)
        .ok_or("Usage: rotate_api_key <OLD_KEY> [OUT_FILE]")?;
    let out_file = std::env::args().nth(2);

    if !old_key.starts_with("nk_") {
        return Err("OLD_KEY must start with nk_".into());
    }

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
    let pool = PgPool::connect(&database_url).await?;

    let old_hash = sha256_hex(&old_key);

    let existing: ExistingKeyRecord = sqlx::query_as(
        "SELECT user_id, tier, permissions, consciousness_level, rate_limit, expires_at, is_active \
         FROM api_keys WHERE key_hash = $1",
    )
    .bind(&old_hash)
    .fetch_optional(&pool)
    .await?
    .ok_or("Old key hash not found in api_keys")?;

    if !existing.is_active {
        return Err("Old key is already inactive; refusing to rotate twice".into());
    }

    let new_key = format!("nk_{}", generate_random_key(32));
    let new_hash = sha256_hex(&new_key);

    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE api_keys SET is_active = false WHERE key_hash = $1")
        .bind(&old_hash)
        .execute(&mut *tx)
        .await?;

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
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    if let Some(path) = out_file {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(new_key.as_bytes())?;
        file.write_all(b"\n")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }

        println!("Rotation complete.");
        println!("Old key hash revoked: {}", old_hash);
        println!("New key hash: {}", new_hash);
        println!("New key written to: {}", path);
    } else {
        println!("Rotation complete.");
        println!("Old key hash revoked: {}", old_hash);
        println!("New key: {}", new_key);
        println!("New key hash: {}", new_hash);
    }

    Ok(())
}
