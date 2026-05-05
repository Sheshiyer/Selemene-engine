use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Mirrors the `billing_subscriptions` table from migration 014, with the
/// `metadata` column added in 020. One row per (user, provider, subscription
/// instance). The active row per user is enforced by the partial unique index
/// `uq_billing_subscriptions_active_user`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BillingSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub provider: String,
    pub provider_customer_id: Option<String>,
    pub provider_subscription_id: Option<String>,
    pub status: String,
    pub cancel_at_period_end: bool,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Incomplete,
    Trialing,
    Active,
    PastDue,
    Canceled,
    Expired,
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Incomplete => "incomplete",
            Self::Trialing => "trialing",
            Self::Active => "active",
            Self::PastDue => "past_due",
            Self::Canceled => "canceled",
            Self::Expired => "expired",
        }
    }
}

impl BillingSubscription {
    /// True iff this row participates in the partial unique index that defines
    /// "the user's currently active subscription".
    pub fn is_active_slot(&self) -> bool {
        matches!(self.status.as_str(), "trialing" | "active" | "past_due") && self.canceled_at.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlanCatalogEntry {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub dodo_product_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
