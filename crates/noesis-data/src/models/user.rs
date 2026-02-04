use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    #[sqlx(rename = "full_name")]
    pub full_name: String,
    pub tier: String, // e.g., "Free", "Basic", "Premium"
    pub consciousness_level: i32,
    pub experience_points: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub reset_token: Option<String>,
    pub reset_token_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub birth_date: Option<chrono::NaiveDate>,
    pub birth_time: Option<chrono::NaiveTime>,
    pub birth_location_lat: Option<f64>, // using f64 for DECIMAL
    pub birth_location_lng: Option<f64>,
    pub birth_location_name: Option<String>,
    pub timezone: Option<String>,
    pub preferences: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
