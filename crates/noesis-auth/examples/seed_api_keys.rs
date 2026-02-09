//! Seed a default admin user and API keys into the PostgreSQL database.
//!
//! Creates one admin user (admin@tryambakam.com) with an Argon2-hashed password,
//! then generates 5 API keys linked to that user via FK. The raw password and
//! API keys are printed once — they cannot be recovered from the database.
//!
//! Usage:
//!   DATABASE_URL=postgres://... cargo run --package noesis-auth --features postgres --example seed_api_keys

use noesis_auth::password::hash_password;
use rand::Rng;
use sha2::{Digest, Sha256};
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

/// Tiers and permissions for the 5 seeded API keys.
struct KeySpec {
    tier: &'static str,
    permissions: Vec<&'static str>,
    consciousness_level: i32,
    rate_limit: i32,
}

fn key_specs() -> Vec<KeySpec> {
    vec![
        KeySpec {
            tier: "enterprise",
            permissions: vec![
                "basic:access",
                "panchanga:read",
                "panchanga:batch",
                "numerology:read",
                "biorhythm:read",
                "human-design:read",
                "gene-keys:read",
                "vimshottari:read",
                "admin:users",
                "admin:analytics",
            ],
            consciousness_level: 5,
            rate_limit: 10000,
        },
        KeySpec {
            tier: "premium",
            permissions: vec![
                "basic:access",
                "panchanga:read",
                "panchanga:batch",
                "numerology:read",
                "biorhythm:read",
                "human-design:read",
                "gene-keys:read",
                "vimshottari:read",
            ],
            consciousness_level: 3,
            rate_limit: 1000,
        },
        KeySpec {
            tier: "premium",
            permissions: vec![
                "basic:access",
                "panchanga:read",
                "numerology:read",
                "biorhythm:read",
            ],
            consciousness_level: 2,
            rate_limit: 1000,
        },
        KeySpec {
            tier: "free",
            permissions: vec!["basic:access", "panchanga:read"],
            consciousness_level: 0,
            rate_limit: 100,
        },
        KeySpec {
            tier: "free",
            permissions: vec!["basic:access", "panchanga:read"],
            consciousness_level: 0,
            rate_limit: 100,
        },
    ]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let pool = PgPool::connect(&database_url).await?;

    // ── 1. Create admin user ────────────────────────────────────────
    let admin_email = "admin@tryambakam.com";
    let admin_password = generate_random_key(24); // secure random password
    let password_hash = hash_password(&admin_password)
        .map_err(|e| format!("Failed to hash password: {}", e))?;

    let user_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, password_hash, full_name, tier, consciousness_level) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (email) DO UPDATE SET password_hash = $2, tier = $4, consciousness_level = $5 \
         RETURNING id",
    )
    .bind(admin_email)
    .bind(&password_hash)
    .bind("Tryambakam Admin")
    .bind("enterprise")
    .bind(5_i32)
    .fetch_one(&pool)
    .await?;

    println!("════════════════════════════════════════════════");
    println!("  ADMIN USER CREATED");
    println!("════════════════════════════════════════════════");
    println!("  Email:    {}", admin_email);
    println!("  Password: {}", admin_password);
    println!("  User ID:  {}", user_id);
    println!("  Tier:     enterprise");
    println!("════════════════════════════════════════════════\n");

    // ── 2. Create API keys linked to admin user ─────────────────────
    let specs = key_specs();

    println!("Generating {} API keys...\n", specs.len());

    for (i, spec) in specs.iter().enumerate() {
        let raw_key = format!("nk_{}", generate_random_key(32));
        let key_hash = sha256_hex(&raw_key);
        let permissions = serde_json::json!(spec.permissions);

        sqlx::query(
            "INSERT INTO api_keys (key_hash, user_id, tier, permissions, consciousness_level, rate_limit, is_active) \
             VALUES ($1, $2, $3, $4, $5, $6, true) \
             ON CONFLICT (key_hash) DO NOTHING",
        )
        .bind(&key_hash)
        .bind(user_id)
        .bind(spec.tier)
        .bind(&permissions)
        .bind(spec.consciousness_level)
        .bind(spec.rate_limit)
        .execute(&pool)
        .await?;

        println!(
            "Key {} [{}]: {}",
            i + 1,
            spec.tier,
            raw_key
        );
        println!("   Hash: {}", key_hash);
        println!(
            "   Permissions: {:?}",
            spec.permissions
        );
        println!();
    }

    println!("════════════════════════════════════════════════");
    println!("  SAVE THESE CREDENTIALS — THEY CANNOT BE");
    println!("  RECOVERED FROM THE DATABASE.");
    println!("════════════════════════════════════════════════");

    Ok(())
}
