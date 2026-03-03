use chrono::Utc;
use serde_json::Value;
use std::sync::{Arc, LazyLock, RwLock};

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
}
