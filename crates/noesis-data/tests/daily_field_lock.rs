//! The daily-field day lock serializes runs for one user and date across connections,
//! and only those: another date or user proceeds at once. A run reads and writes
//! through the lock's own connection, and only a finished run is committed.
//! Needs TEST_DATABASE_URL or DATABASE_URL (the second test also needs the migrated
//! schema); skipped otherwise.

use noesis_data::models::reading::NewReading;
use noesis_data::repositories::readings_repository::ReadingsRepository;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn a_day_is_held_by_one_run_at_a_time() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL").or_else(|_| std::env::var("DATABASE_URL")) else {
        eprintln!("Skipping daily-field lock test: TEST_DATABASE_URL/DATABASE_URL not set");
        return;
    };
    let pool = PgPoolOptions::new().max_connections(8).connect(&url).await.expect("connect");
    let repo = std::sync::Arc::new(ReadingsRepository::new(pool));
    let user = Uuid::new_v4();
    let wait = Duration::from_millis(400);

    let first = repo.lock_daily_day(user, "2026-10-30").await.expect("first hold");

    let r = repo.clone();
    let contender = tokio::spawn(async move { r.lock_daily_day(user, "2026-10-30").await.expect("second hold") });
    tokio::time::sleep(wait).await;
    assert!(!contender.is_finished(), "a second run for the same day must wait");

    let other_date = tokio::time::timeout(wait, repo.lock_daily_day(user, "2026-10-31")).await;
    assert!(other_date.is_ok(), "another date is not held");
    let other_user = tokio::time::timeout(wait, repo.lock_daily_day(Uuid::new_v4(), "2026-10-30")).await;
    assert!(other_user.is_ok(), "another user is not held");
    drop((other_date, other_user));

    first.finish().await.expect("finish");
    let second = tokio::time::timeout(Duration::from_secs(5), contender)
        .await
        .expect("the waiting run proceeds once the day is released")
        .expect("join");
    drop(second);

    // A hold dropped without release still ends with its transaction.
    let again = tokio::time::timeout(Duration::from_secs(5), repo.lock_daily_day(user, "2026-10-30")).await;
    assert!(again.is_ok(), "a dropped hold frees the day");
}

fn part(user: Uuid, engine: &str) -> NewReading {
    NewReading {
        user_id: user,
        engine_id: engine.into(),
        workflow_id: Some("daily-field".into()),
        input_hash: format!("{user}:{engine}"),
        input_data: serde_json::json!({"daily_date": "2026-10-30"}),
        result_data: serde_json::json!({"ok": true}),
        witness_prompt: None,
        consciousness_level: 5,
        calculation_time_ms: None,
        client_event_id: None,
        client_device_id: None,
        device_platform: None,
        device_app_version: None,
    }
}

#[tokio::test]
async fn a_run_writes_through_its_own_connection_and_commits_on_finish() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL").or_else(|_| std::env::var("DATABASE_URL")) else {
        eprintln!("Skipping daily-field write test: TEST_DATABASE_URL/DATABASE_URL not set");
        return;
    };
    // One connection in the pool: the run must never need a second one.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(2))
        .connect(&url)
        .await
        .expect("connect");
    let has_readings: bool = sqlx::query_scalar("SELECT to_regclass('public.readings') IS NOT NULL")
        .fetch_one(&pool)
        .await
        .expect("probe schema");
    if !has_readings {
        eprintln!("Skipping daily-field write test: readings table not migrated");
        return;
    }
    let user: Uuid = sqlx::query_scalar("INSERT INTO users (email, password_hash) VALUES ($1, 'x') RETURNING id")
        .bind(format!("daily-{}@test.local", Uuid::new_v4()))
        .fetch_one(&pool)
        .await
        .expect("user");
    let repo = ReadingsRepository::new(pool.clone());

    // A dropped run stores nothing.
    {
        let mut held = repo.lock_daily_day(user, "2026-10-30").await.expect("hold");
        assert!(held.save(&part(user, "transits")).await.expect("save").is_some());
    }
    let after_drop = repo.list_daily_field(user, "2026-10-30").await.expect("list");
    assert!(after_drop.is_empty(), "a dropped run is rolled back");

    // A finished run stores its parts; a repeated part is left as it was.
    let mut held = repo.lock_daily_day(user, "2026-10-30").await.expect("hold");
    assert!(held.stored(user, "2026-10-30").await.expect("stored").is_empty());
    assert!(held.save(&part(user, "transits")).await.expect("save").is_some());
    assert!(held.save(&part(user, "transits")).await.expect("repeat").is_none());
    let mut bad = part(user, "panchanga");
    bad.user_id = Uuid::new_v4(); // no such user: this part fails on its own
    assert!(held.save(&bad).await.is_err());
    assert!(held.save(&part(user, "biorhythm")).await.expect("after a failed part").is_some());
    assert_eq!(held.stored(user, "2026-10-30").await.expect("stored").len(), 2);
    held.finish().await.expect("finish");

    let rows = repo.list_daily_field(user, "2026-10-30").await.expect("list");
    // Parts of one run share its transaction's timestamp, so their order is not fixed.
    let mut engines: Vec<&str> = rows.iter().map(|r| r.engine_id.as_str()).collect();
    engines.sort();
    assert_eq!(engines, vec!["biorhythm", "transits"]);
}
