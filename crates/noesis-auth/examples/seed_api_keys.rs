//! Seed API keys into the PostgreSQL api_keys table.
//!
//! Generates 5 random API keys, hashes them with SHA-256, and inserts them
//! into the database. The raw keys are printed once -- they cannot be recovered.
//!
//! Usage:
//!   DATABASE_URL=postgres://... cargo run --package noesis-auth --features postgres --example seed_api_keys

use sha2::{Sha256, Digest};
use rand::Rng;
use sqlx::PgPool;

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
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    let pool = PgPool::connect(&database_url).await?;

    println!("Generating 5 API keys...\n");

    for i in 1..=5 {
        // Generate random 32-char key prefixed with "nk_" for easy identification
        let raw_key = format!("nk_{}", generate_random_key(32));
        let key_hash = sha256_hex(&raw_key);

        // Insert into database with default tier and permissions
        sqlx::query(
            "INSERT INTO api_keys (key_hash, user_id, tier, permissions, consciousness_level, rate_limit, is_active) \
             VALUES ($1, gen_random_uuid(), $2, $3, $4, $5, true)"
        )
        .bind(&key_hash)
        .bind("free")
        .bind(serde_json::json!(["basic:access", "panchanga:read"]))
        .bind(0_i32)
        .bind(100_i32)
        .execute(&pool)
        .await?;

        println!("Key {}: {}", i, raw_key);
        println!("   Hash: {}\n", key_hash);
    }

    println!("SAVE THESE KEYS -- they cannot be recovered from the database.");

    Ok(())
}
