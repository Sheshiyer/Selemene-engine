//! Regression: concurrent first-time upserts of the same provider subscription
//! must all succeed and converge on one row.
//!
//! `upsert_subscription` arbitrates `ON CONFLICT` on
//! `uq_billing_subscriptions_provider_subscription`, but the table also has the
//! non-arbiter `uq_billing_subscriptions_active_user`. Without serialisation, a
//! racing duplicate (e.g. a replayed webhook) could fail with 23505 on that
//! index instead of taking the `DO UPDATE` path, which surfaced as an HTTP 500
//! in `billing_replay_tests::webhook_replay_concurrent_yields_exactly_one_mutation`.
//!
//! Synthetic rows only; each round uses a fresh user, deleted afterwards.
use noesis_data::repositories::billing_repository::BillingRepository;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

const ROUNDS: usize = 40;
const CONCURRENCY: usize = 16;

#[tokio::test]
async fn concurrent_duplicate_subscription_upserts_converge_on_one_row() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL").or_else(|_| std::env::var("DATABASE_URL"))
    else {
        eprintln!("not run: TEST_DATABASE_URL/DATABASE_URL not set");
        return;
    };
    let pool = PgPoolOptions::new()
        .max_connections(CONCURRENCY as u32)
        .connect(&url)
        .await
        .expect("connect to local PostgreSQL test database");
    let plan_id: Uuid = sqlx::query_scalar("SELECT id FROM plan_catalog WHERE code = 'basic'")
        .fetch_one(&pool)
        .await
        .expect("basic plan seeded by migrations");
    let repo = Arc::new(BillingRepository::new(pool.clone()));

    for _ in 0..ROUNDS {
        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'synthetic-test-only')",
        )
        .bind(user_id)
        .bind(format!("upsert-race-{user_id}@example.test"))
        .execute(&pool)
        .await
        .expect("seed synthetic user");
        let subscription_id = format!("sub_race_{}", Uuid::new_v4());
        let customer_id = format!("cus_race_{}", Uuid::new_v4());

        let tasks: Vec<_> = (0..CONCURRENCY)
            .map(|_| {
                let repo = repo.clone();
                let (sub, cus) = (subscription_id.clone(), customer_id.clone());
                tokio::spawn(async move {
                    repo.upsert_subscription(
                        user_id,
                        plan_id,
                        &cus,
                        &sub,
                        "active",
                        false,
                        None,
                        None,
                        serde_json::json!({ "synthetic": true }),
                    )
                    .await
                })
            })
            .collect();
        let mut ids = Vec::with_capacity(CONCURRENCY);
        for task in tasks {
            let row = task
                .await
                .expect("task")
                .expect("duplicate upsert must not fail");
            ids.push(row.id);
        }
        assert!(
            ids.windows(2).all(|w| w[0] == w[1]),
            "all upserts must return the same row"
        );
        let rows: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM billing_subscriptions WHERE provider_subscription_id = $1",
        )
        .bind(&subscription_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(rows, 1, "exactly one billing_subscriptions row");

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("cleanup synthetic user");
    }
}
