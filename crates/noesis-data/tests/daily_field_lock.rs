//! The daily-field day lock serializes runs for one user and date across connections,
//! and only those: another date or user proceeds at once.
//! Needs TEST_DATABASE_URL or DATABASE_URL; skipped otherwise.

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

    first.release().await.expect("release");
    let second = tokio::time::timeout(Duration::from_secs(5), contender)
        .await
        .expect("the waiting run proceeds once the day is released")
        .expect("join");
    drop(second);

    // A hold dropped without release still ends with its transaction.
    let again = tokio::time::timeout(Duration::from_secs(5), repo.lock_daily_day(user, "2026-10-30")).await;
    assert!(again.is_ok(), "a dropped hold frees the day");
}
