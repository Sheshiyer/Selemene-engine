//! Combine organ clock recommendations with Panchang/Hora/Choghadiya data

use chrono::{DateTime, Utc, Timelike};

use noesis_vedic_api::panchang::CompletePanchang;

use crate::integration::get_temporal_recommendation;
use crate::models::TemporalRecommendation;
use crate::hora_integration::recommendations_from_hora;
use crate::choghadiya_integration::recommendations_from_choghadiya;

pub fn recommendation_from_complete_panchang(
    datetime: DateTime<Utc>,
    timezone_offset_minutes: i32,
    panchang: &CompletePanchang,
) -> TemporalRecommendation {
    let tithi_index = panchang.panchang.tithi.number.saturating_sub(1) % 15;
    let nakshatra_index = panchang.panchang.nakshatra.number.saturating_sub(1);

    let mut recommendation = get_temporal_recommendation(
        datetime,
        timezone_offset_minutes,
        Some(tithi_index),
        Some(nakshatra_index),
    );

    let current_time = chrono::NaiveTime::from_hms_opt(
        datetime.hour(),
        datetime.minute(),
        datetime.second(),
    )
    .map(|t| t.format("%H:%M").to_string())
    .unwrap_or_else(|| "00:00".to_string());

    recommendation
        .activities
        .extend(recommendations_from_hora(&panchang.hora_timings, &current_time));
    recommendation
        .activities
        .extend(recommendations_from_choghadiya(&panchang.choghadiya, &current_time));

    recommendation
}
