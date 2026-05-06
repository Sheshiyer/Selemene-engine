use chrono::Utc;
use noesis_data::repositories::billing_repository::BillingRepository;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::Duration;
use uuid::Uuid;

/// Inbound event types we subscribe to from Dodo. Must stay in lockstep with
/// the TypeScript `DodoInboundEventType` in `@selemene/sdk` and §API in
/// `.context/billing/contracts.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DodoInboundEventType {
    #[serde(rename = "subscription.active")]
    SubscriptionActive,
    #[serde(rename = "subscription.updated")]
    SubscriptionUpdated,
    #[serde(rename = "subscription.on_hold")]
    SubscriptionOnHold,
    #[serde(rename = "subscription.cancelled")]
    SubscriptionCancelled,
    #[serde(rename = "subscription.failed")]
    SubscriptionFailed,
    #[serde(rename = "payment.succeeded")]
    PaymentSucceeded,
    #[serde(rename = "payment.failed")]
    PaymentFailed,
    #[serde(rename = "credit.added")]
    CreditAdded,
    #[serde(rename = "credit.deducted")]
    CreditDeducted,
    #[serde(rename = "credit.balance_low")]
    CreditBalanceLow,
    #[serde(rename = "credit.overage_charged")]
    CreditOverageCharged,
}

/// Envelope posted by the Next.js webhook adaptor to
/// `POST /internal/billing/events`. Payload is the raw verified Dodo body —
/// kept as `serde_json::Value` so we don't have to model every Dodo schema.
///
/// Wave 1.1 contract-freeze type — the handler that consumes it lands in T15.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct BillingForwardRequest {
    pub webhook_id: String,
    pub webhook_timestamp: String,
    pub event_type: DodoInboundEventType,
    pub payload: Value,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum BillingForwardResponse {
    Ok,
    Dedup,
}

pub trait BillingEventEmitter: Send + Sync {
    fn emit_usage_event(&self, user_id: &str, engine_id: &str, tier: &str);
    fn emit_quota_exceeded(&self, user_id: &str, tier: &str);
}

pub struct NoopBillingEmitter;

impl BillingEventEmitter for NoopBillingEmitter {
    fn emit_usage_event(&self, user_id: &str, engine_id: &str, tier: &str) {
        tracing::debug!(
            user_id = user_id,
            engine_id = engine_id,
            tier = tier,
            "noop billing usage event"
        );
    }

    fn emit_quota_exceeded(&self, user_id: &str, tier: &str) {
        tracing::debug!(
            user_id = user_id,
            tier = tier,
            "noop billing quota exceeded event"
        );
    }
}

pub struct StripeWebhookEmitter {
    webhook_url: String,
}

impl StripeWebhookEmitter {
    pub fn new(webhook_url: impl Into<String>) -> Self {
        Self {
            webhook_url: webhook_url.into(),
        }
    }

    pub fn format_usage_payload(&self, user_id: &str, engine_id: &str, tier: &str) -> Value {
        serde_json::json!({
            "event_type": "usage_event",
            "provider": "stripe",
            "webhook_url": self.webhook_url,
            "user_id": user_id,
            "engine_id": engine_id,
            "tier": tier,
            "timestamp": Utc::now().to_rfc3339(),
        })
    }

    pub fn format_quota_exceeded_payload(&self, user_id: &str, tier: &str) -> Value {
        serde_json::json!({
            "event_type": "quota_exceeded",
            "provider": "stripe",
            "webhook_url": self.webhook_url,
            "user_id": user_id,
            "tier": tier,
            "timestamp": Utc::now().to_rfc3339(),
        })
    }
}

impl BillingEventEmitter for StripeWebhookEmitter {
    fn emit_usage_event(&self, user_id: &str, engine_id: &str, tier: &str) {
        let payload = self.format_usage_payload(user_id, engine_id, tier);
        tracing::debug!(payload = %payload, "stripe webhook usage payload formatted");
    }

    fn emit_quota_exceeded(&self, user_id: &str, tier: &str) {
        let payload = self.format_quota_exceeded_payload(user_id, tier);
        tracing::debug!(payload = %payload, "stripe webhook quota payload formatted");
    }
}

/// Outbound usage emitter for Dodo Payments.
///
/// Sends `noesis.engine_query` events to `<api_base>/usage-events/ingest`
/// against the meter created during provisioning. Each event deducts 1
/// Witness Credit from the customer's balance per the meter's Sum
/// aggregation on `metadata.amount`.
///
/// Retry policy: 2 attempts after the first (200 ms then 1 s). Final failure
/// is logged + Sentry-breadcrumbed but **never blocks the engine response**.
/// Free-tier users have no `dodo_customer_id` and are silently skipped —
/// they are gated by quota at the request entry, not by usage emission.
#[derive(Clone)]
pub struct DodoWebhookEmitter {
    api_key: String,
    api_base: String,
    client: reqwest::Client,
    repo: Option<Arc<BillingRepository>>,
}

impl DodoWebhookEmitter {
    /// Construct an emitter pointed at a Dodo API base URL (`https://test…`
    /// or `https://live…`). The Bearer key authenticates outbound calls.
    pub fn new(api_key: impl Into<String>, api_base: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client should build with default config");
        Self {
            api_key: api_key.into(),
            api_base: api_base.into(),
            client,
            repo: None,
        }
    }

    /// Attach the BillingRepository so the emitter can resolve a Dodo
    /// customer ID from the internal user UUID. Without a repo, the trait
    /// methods become no-ops (with a warn log) — useful for partial-config
    /// dev environments where the DB isn't connected.
    pub fn with_repository(mut self, repo: Arc<BillingRepository>) -> Self {
        self.repo = Some(repo);
        self
    }

    pub fn format_usage_payload(
        &self,
        dodo_customer_id: &str,
        internal_user_id: &str,
        engine_id: &str,
        tier: &str,
    ) -> Value {
        let event_id = format!(
            "noesis_engine_{}_{}_{}",
            internal_user_id,
            engine_id,
            Utc::now().timestamp_nanos_opt().unwrap_or(0),
        );
        serde_json::json!({
            "events": [{
                "event_id": event_id,
                "customer_id": dodo_customer_id,
                "event_name": "noesis.engine_query",
                "timestamp": Utc::now().to_rfc3339(),
                "metadata": {
                    "engine_id": engine_id,
                    "tier": tier,
                    "internal_user_id": internal_user_id,
                    "amount": 1
                }
            }]
        })
    }

    pub fn format_quota_exceeded_payload(&self, internal_user_id: &str, tier: &str) -> Value {
        // Quota-exceeded is a metric-only signal; we don't currently send it
        // to Dodo (no API for it). This payload is captured for logs/Sentry.
        serde_json::json!({
            "event_type": "quota_exceeded",
            "internal_user_id": internal_user_id,
            "tier": tier,
            "timestamp": Utc::now().to_rfc3339(),
        })
    }

    /// Retry-on-transient POST to /usage-events/ingest. Returns `Ok(())` on
    /// 2xx; `Err(detail)` after exhausted retries. Caller is expected to log
    /// + Sentry-breadcrumb the error and continue (never block).
    async fn ingest_usage_event(
        &self,
        dodo_customer_id: &str,
        internal_user_id: &str,
        engine_id: &str,
        tier: &str,
    ) -> Result<(), String> {
        let body = self.format_usage_payload(dodo_customer_id, internal_user_id, engine_id, tier);
        let url = format!(
            "{}/usage-events/ingest",
            self.api_base.trim_end_matches('/')
        );
        let backoffs_ms = [0u64, 200, 1_000];
        let mut last_err = String::new();

        for (attempt, delay_ms) in backoffs_ms.iter().enumerate() {
            if *delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
            }
            match self
                .client
                .post(&url)
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                Ok(resp) => {
                    let status = resp.status();
                    last_err = format!("HTTP {} on attempt {}", status, attempt + 1);
                    // 4xx (except 429) means the request itself is malformed —
                    // retrying won't help.
                    if status.is_client_error() && status.as_u16() != 429 {
                        break;
                    }
                }
                Err(e) => {
                    last_err = format!("network error on attempt {}: {}", attempt + 1, e);
                }
            }
        }
        Err(last_err)
    }
}

impl BillingEventEmitter for DodoWebhookEmitter {
    fn emit_usage_event(&self, user_id: &str, engine_id: &str, tier: &str) {
        let Some(repo) = self.repo.clone() else {
            tracing::warn!(
                user_id = user_id,
                "dodo emitter has no repository attached — usage event dropped"
            );
            return;
        };
        let user_id = user_id.to_string();
        let engine_id = engine_id.to_string();
        let tier = tier.to_string();
        let emitter = self.clone();

        tokio::spawn(async move {
            let user_uuid = match Uuid::parse_str(&user_id) {
                Ok(u) => u,
                Err(_) => {
                    tracing::warn!(user_id = %user_id, "usage emit: invalid user_id");
                    return;
                }
            };
            let dodo_customer_id = match repo.find_user_dodo_customer_id(user_uuid).await {
                Ok(Some(c)) => c,
                Ok(None) => {
                    // Free tier — no customer ID yet. Quota gate handles
                    // billing-side enforcement; usage just isn't metered.
                    tracing::trace!(user_id = %user_id, "usage emit skipped: no dodo_customer_id");
                    noesis_metrics::record_dodo_usage_emit("skipped");
                    return;
                }
                Err(e) => {
                    tracing::error!(error = %e, "usage emit: dodo_customer_id lookup failed");
                    noesis_metrics::record_dodo_usage_emit("failed");
                    return;
                }
            };

            match emitter
                .ingest_usage_event(&dodo_customer_id, &user_id, &engine_id, &tier)
                .await
            {
                Ok(()) => {
                    noesis_metrics::record_dodo_usage_emit("success");
                }
                Err(detail) => {
                    tracing::error!(
                        user_id = %user_id,
                        engine_id = %engine_id,
                        error = %detail,
                        "dodo usage ingest failed after retries"
                    );
                    noesis_metrics::record_dodo_usage_emit("failed");
                    // Sentry breadcrumb so this surfaces in error tracking.
                    sentry::add_breadcrumb(sentry::Breadcrumb {
                        category: Some("dodo.usage_emit".into()),
                        level: sentry::Level::Error,
                        message: Some(format!("dodo usage ingest failed: {}", detail)),
                        data: {
                            let mut m = std::collections::BTreeMap::new();
                            m.insert("user_id".into(), user_id.clone().into());
                            m.insert("engine_id".into(), engine_id.clone().into());
                            m.insert("tier".into(), tier.clone().into());
                            m
                        },
                        ..Default::default()
                    });
                }
            }
        });
    }

    fn emit_quota_exceeded(&self, user_id: &str, tier: &str) {
        let payload = self.format_quota_exceeded_payload(user_id, tier);
        tracing::info!(payload = %payload, "dodo quota exceeded");
    }
}

static BILLING_EMITTER: LazyLock<RwLock<Arc<dyn BillingEventEmitter>>> =
    LazyLock::new(|| RwLock::new(Arc::new(NoopBillingEmitter)));

pub fn set_billing_emitter(emitter: Arc<dyn BillingEventEmitter>) {
    if let Ok(mut guard) = BILLING_EMITTER.write() {
        *guard = emitter;
    }
}

pub fn reset_billing_emitter() {
    set_billing_emitter(Arc::new(NoopBillingEmitter));
}

pub fn emit_usage_event(user_id: &str, engine_id: &str, tier: &str) {
    if let Ok(guard) = BILLING_EMITTER.read() {
        guard.emit_usage_event(user_id, engine_id, tier);
    }
}

pub fn emit_quota_exceeded(user_id: &str, tier: &str) {
    if let Ok(guard) = BILLING_EMITTER.read() {
        guard.emit_quota_exceeded(user_id, tier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stripe_usage_payload_shape() {
        let emitter = StripeWebhookEmitter::new("https://stripe.example/webhook");
        let payload = emitter.format_usage_payload("user-1", "panchanga", "pro");

        assert_eq!(payload["event_type"], "usage_event");
        assert_eq!(payload["provider"], "stripe");
        assert_eq!(payload["webhook_url"], "https://stripe.example/webhook");
        assert_eq!(payload["user_id"], "user-1");
        assert_eq!(payload["engine_id"], "panchanga");
        assert_eq!(payload["tier"], "pro");
        assert!(payload["timestamp"].as_str().is_some());
    }

    #[test]
    fn stripe_quota_payload_shape() {
        let emitter = StripeWebhookEmitter::new("https://stripe.example/webhook");
        let payload = emitter.format_quota_exceeded_payload("user-1", "free");

        assert_eq!(payload["event_type"], "quota_exceeded");
        assert_eq!(payload["provider"], "stripe");
        assert_eq!(payload["webhook_url"], "https://stripe.example/webhook");
        assert_eq!(payload["user_id"], "user-1");
        assert_eq!(payload["tier"], "free");
        assert!(payload["timestamp"].as_str().is_some());
    }

    #[test]
    fn dodo_usage_payload_shape() {
        let emitter = DodoWebhookEmitter::new("sk_test_key", "https://test.dodopayments.com");
        let payload = emitter.format_usage_payload("cus_abc", "user-1", "panchanga", "premium");

        let events = payload["events"].as_array().expect("events array");
        assert_eq!(events.len(), 1);
        let ev = &events[0];
        assert_eq!(ev["customer_id"], "cus_abc");
        assert_eq!(ev["event_name"], "noesis.engine_query");
        assert!(ev["timestamp"].as_str().is_some());
        assert!(
            ev["event_id"]
                .as_str()
                .unwrap()
                .starts_with("noesis_engine_user-1_panchanga_"),
            "event_id should be prefixed deterministically"
        );
        assert_eq!(ev["metadata"]["engine_id"], "panchanga");
        assert_eq!(ev["metadata"]["tier"], "premium");
        assert_eq!(ev["metadata"]["internal_user_id"], "user-1");
        assert_eq!(ev["metadata"]["amount"], 1);
    }

    #[test]
    fn dodo_quota_payload_includes_user_and_tier() {
        let emitter = DodoWebhookEmitter::new("sk_test_key", "https://test.dodopayments.com");
        let payload = emitter.format_quota_exceeded_payload("user-1", "free");
        assert_eq!(payload["event_type"], "quota_exceeded");
        assert_eq!(payload["internal_user_id"], "user-1");
        assert_eq!(payload["tier"], "free");
        assert!(payload["timestamp"].as_str().is_some());
    }

    #[test]
    fn inbound_event_type_serde_roundtrip() {
        let cases = [
            (
                "subscription.active",
                DodoInboundEventType::SubscriptionActive,
            ),
            (
                "subscription.updated",
                DodoInboundEventType::SubscriptionUpdated,
            ),
            (
                "subscription.on_hold",
                DodoInboundEventType::SubscriptionOnHold,
            ),
            (
                "subscription.cancelled",
                DodoInboundEventType::SubscriptionCancelled,
            ),
            (
                "subscription.failed",
                DodoInboundEventType::SubscriptionFailed,
            ),
            ("payment.succeeded", DodoInboundEventType::PaymentSucceeded),
            ("payment.failed", DodoInboundEventType::PaymentFailed),
            ("credit.added", DodoInboundEventType::CreditAdded),
            ("credit.deducted", DodoInboundEventType::CreditDeducted),
            ("credit.balance_low", DodoInboundEventType::CreditBalanceLow),
            (
                "credit.overage_charged",
                DodoInboundEventType::CreditOverageCharged,
            ),
        ];
        for (wire, variant) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(
                json,
                format!("\"{}\"", wire),
                "encode mismatch for {}",
                wire
            );
            let decoded: DodoInboundEventType = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, variant, "decode mismatch for {}", wire);
        }
    }

    #[test]
    fn forward_request_parses_full_envelope() {
        let body = serde_json::json!({
            "webhook_id": "msg_29abc",
            "webhook_timestamp": "1714867200",
            "event_type": "subscription.active",
            "payload": { "data": { "subscription_id": "sub_test" } },
        });
        let parsed: BillingForwardRequest = serde_json::from_value(body).unwrap();
        assert_eq!(parsed.webhook_id, "msg_29abc");
        assert_eq!(parsed.event_type, DodoInboundEventType::SubscriptionActive);
        assert_eq!(parsed.payload["data"]["subscription_id"], "sub_test");
    }

    #[test]
    fn forward_response_serialises_with_status_tag() {
        let ok = serde_json::to_value(BillingForwardResponse::Ok).unwrap();
        assert_eq!(ok, serde_json::json!({"status": "ok"}));
        let dedup = serde_json::to_value(BillingForwardResponse::Dedup).unwrap();
        assert_eq!(dedup, serde_json::json!({"status": "dedup"}));
    }
}
