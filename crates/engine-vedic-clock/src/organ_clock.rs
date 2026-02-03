//! Organ clock integration with FreeAstrologyAPI Panchang

use chrono::{DateTime, Utc, Timelike, Datelike};

use noesis_vedic_api::{CachedVedicClient, VedicApiError};

use crate::integration::get_temporal_recommendation;
use crate::models::TemporalRecommendation;

/// Fetch Panchang from API and generate temporal recommendation.
pub async fn get_temporal_recommendation_with_api(
    client: &CachedVedicClient,
    datetime: DateTime<Utc>,
    timezone_offset_minutes: i32,
    latitude: f64,
    longitude: f64,
) -> Result<TemporalRecommendation, VedicApiError> {
    let local = datetime + chrono::Duration::minutes(timezone_offset_minutes as i64);
    let tzone = timezone_offset_minutes as f64 / 60.0;

    let panchang = client
        .get_panchang(
            local.year(),
            local.month(),
            local.day(),
            local.hour(),
            local.minute(),
            local.second(),
            latitude,
            longitude,
            tzone,
        )
        .await?;

    let tithi_index = panchang.tithi.number.saturating_sub(1) % 15;
    let nakshatra_index = panchang.nakshatra.number.saturating_sub(1);

    Ok(get_temporal_recommendation(
        datetime,
        timezone_offset_minutes,
        Some(tithi_index),
        Some(nakshatra_index),
    ))
}
