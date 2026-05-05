//! Dodo Payments ↔ local subscription reconciliation cron (T24).
//!
//! Runs once per invocation (intended cadence: hourly via k8s CronJob /
//! Railway cron / OS cron). Pulls `GET /subscriptions?status=active` from
//! Dodo, compares against `billing_subscriptions` rows in the
//! `dodo_payments` provider, and reports drift in three classes:
//!
//!   • `local_only_active`  — local says active, Dodo doesn't list it
//!     → force-cancel locally (Dodo is source of truth).
//!   • `dodo_only_active`   — Dodo says active, we have no row
//!     → log + Sentry. Cannot synthesise a subscription.active without
//!       knowing the user_id mapping; flag for manual triage.
//!   • `both_active`        — both sides agree (no action).
//!
//! Output is structured JSON on stdout for easy ingestion by log scrapers.
//! Exits 0 on success (drift count printed), non-zero on operational
//! failure (DB unreachable, Dodo API unreachable, etc).
//!
//! Usage:
//!     DATABASE_URL=...  DODO_PAYMENTS_API_KEY=...  cargo run --bin dodo_reconcile
//!
//! Optional: set `DODO_RECONCILE_FORCE_CANCEL=true` to actually mutate the DB
//! for `local_only_active` drift. Without it the bin is read-only and just
//! reports counts (recommended for first deploy + Wave 3 verification).

use noesis_data::repositories::billing_repository::{BillingRepository, PROVIDER_DODO};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashSet;
use std::process::ExitCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> ExitCode {
    let _ = tracing_subscriber::fmt::try_init();

    let api_key = match std::env::var("DODO_PAYMENTS_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => {
            eprintln!("error: DODO_PAYMENTS_API_KEY not set");
            return ExitCode::from(2);
        }
    };
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => {
            eprintln!("error: DATABASE_URL not set");
            return ExitCode::from(2);
        }
    };
    let dodo_env = std::env::var("DODO_PAYMENTS_ENV").unwrap_or_else(|_| "test".into());
    let api_base = if dodo_env == "live" {
        "https://live.dodopayments.com"
    } else {
        "https://test.dodopayments.com"
    };
    let force_cancel = std::env::var("DODO_RECONCILE_FORCE_CANCEL")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    // -- DB --
    let pool = match PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: connect Postgres: {}", e);
            return ExitCode::from(2);
        }
    };
    let repo = BillingRepository::new(pool);

    let local_ids = match repo.list_active_provider_subscription_ids(PROVIDER_DODO).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: list local active subs: {}", e);
            return ExitCode::from(2);
        }
    };
    let local_set: HashSet<String> = local_ids.iter().cloned().collect();

    // -- Dodo --
    let dodo_set = match fetch_active_dodo_subscription_ids(api_base, &api_key).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: fetch active dodo subs: {}", e);
            return ExitCode::from(2);
        }
    };

    // -- Diff --
    let local_only: Vec<&String> = local_set.difference(&dodo_set).collect();
    let dodo_only: Vec<&String> = dodo_set.difference(&local_set).collect();
    let both = local_set.intersection(&dodo_set).count();

    if !local_only.is_empty() {
        noesis_metrics::record_dodo_reconcile_drift("local_only", local_only.len() as u64);
    }
    if !dodo_only.is_empty() {
        noesis_metrics::record_dodo_reconcile_drift("dodo_only", dodo_only.len() as u64);
    }

    // -- Optional remediation: force-cancel local rows Dodo no longer lists --
    let mut forced_cancellations = 0usize;
    if force_cancel && !local_only.is_empty() {
        for sid in &local_only {
            match repo.force_cancel_subscription(sid).await {
                Ok(Some(_)) => {
                    forced_cancellations += 1;
                    tracing::warn!(
                        provider_subscription_id = %sid,
                        "reconciler force-cancelled local subscription (Dodo no longer reports it active)"
                    );
                }
                Ok(None) => {
                    tracing::warn!(
                        provider_subscription_id = %sid,
                        "force_cancel target row not found (concurrent webhook?)"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        provider_subscription_id = %sid,
                        "force_cancel failed"
                    );
                }
            }
        }
    }

    let report = serde_json::json!({
        "provider": PROVIDER_DODO,
        "env": dodo_env,
        "local_active": local_set.len(),
        "dodo_active": dodo_set.len(),
        "drift": {
            "local_only_active":  local_only.len(),
            "dodo_only_active":   dodo_only.len(),
            "both_active":        both,
        },
        "force_cancel_enabled": force_cancel,
        "forced_cancellations": forced_cancellations,
        "samples": {
            "local_only": local_only.iter().take(5).collect::<Vec<_>>(),
            "dodo_only":  dodo_only.iter().take(5).collect::<Vec<_>>(),
        },
    });
    println!("{}", report);

    // Non-zero drift but no operational failure → exit 0 (cron should not
    // fail-loud on every drift; alerting handles that). Reserve non-zero
    // exits for actual operational errors (DB/Dodo unreachable, etc).
    ExitCode::SUCCESS
}

/// Pulls every page of `GET /subscriptions?status=active` and returns the
/// set of subscription_ids. Pages of 100, max 100 pages (10k subs).
async fn fetch_active_dodo_subscription_ids(
    api_base: &str,
    api_key: &str,
) -> Result<HashSet<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("build client: {}", e))?;

    let mut ids = HashSet::new();
    for page in 0..100u32 {
        let url = format!(
            "{}/subscriptions?status=active&page_number={}&page_size=100",
            api_base, page
        );
        let resp = client
            .get(&url)
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| format!("GET subscriptions page {}: {}", page, e))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("Dodo {} on page {}: {}", status, page, text));
        }
        let parsed: Value = serde_json::from_str(&text)
            .map_err(|e| format!("parse page {}: {}", page, e))?;
        let items = parsed
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if items.is_empty() {
            break;
        }
        for item in &items {
            if let Some(s) = item.get("subscription_id").and_then(|v| v.as_str()) {
                ids.insert(s.to_string());
            }
        }
        if items.len() < 100 {
            break;
        }
    }
    Ok(ids)
}
