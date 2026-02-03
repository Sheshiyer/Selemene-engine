//! Transit report generation

use super::types::{BirthDetails, GeneratedReport, ReportSectionContent};
use super::ReportConfig;
use chrono::{NaiveDate, Utc};

/// Generate transit report for a period
pub fn generate_transit_report(
    details: &BirthDetails,
    from_date: NaiveDate,
    to_date: NaiveDate,
    config: &ReportConfig,
) -> GeneratedReport {
    let sections = vec![
        ReportSectionContent {
            title: "Transit Overview".to_string(),
            content: format!(
                "Transit analysis for {} from {} to {}",
                details.name, from_date, to_date
            ),
            key_points: vec![
                "Major planetary transits are analyzed".to_string(),
                "Effects on natal chart are interpreted".to_string(),
            ],
            chart_data: None,
        },
        ReportSectionContent {
            title: "Jupiter Transit".to_string(),
            content: "Jupiter transits are significant for growth and expansion.".to_string(),
            key_points: vec![
                "Jupiter in favorable houses brings opportunities".to_string(),
                "Transit over natal planets activates their significations".to_string(),
            ],
            chart_data: None,
        },
        ReportSectionContent {
            title: "Saturn Transit".to_string(),
            content: "Saturn transits bring discipline, challenges, and karmic lessons.".to_string(),
            key_points: vec![
                "Sade Sati status if applicable".to_string(),
                "Saturn aspects to natal planets".to_string(),
            ],
            chart_data: None,
        },
        ReportSectionContent {
            title: "Rahu-Ketu Transit".to_string(),
            content: "The nodal axis transit influences karmic direction.".to_string(),
            key_points: vec![
                "Rahu transit brings material desires".to_string(),
                "Ketu transit encourages spiritual growth".to_string(),
            ],
            chart_data: None,
        },
        ReportSectionContent {
            title: "Monthly Highlights".to_string(),
            content: "Key dates and events to watch for.".to_string(),
            key_points: vec![
                "New Moon and Full Moon effects".to_string(),
                "Eclipses during the period".to_string(),
                "Retrograde periods of inner planets".to_string(),
            ],
            chart_data: None,
        },
    ];
    
    GeneratedReport {
        title: format!("Transit Report for {}", details.name),
        subject_name: details.name.clone(),
        birth_datetime: details.datetime,
        generated_at: Utc::now().naive_utc(),
        sections,
        summary: format!(
            "This transit report covers the period from {} to {}. \
             Major planetary movements and their effects on your natal chart are analyzed.",
            from_date, to_date
        ),
    }
}

/// Get transit priority (higher = more important)
pub fn get_transit_priority(planet: &str) -> u8 {
    match planet.to_lowercase().as_str() {
        "saturn" => 10,
        "jupiter" => 9,
        "rahu" | "ketu" => 8,
        "mars" => 6,
        "venus" => 5,
        "mercury" => 4,
        "sun" => 3,
        "moon" => 2,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transit_priority() {
        assert!(get_transit_priority("Saturn") > get_transit_priority("Moon"));
        assert!(get_transit_priority("Jupiter") > get_transit_priority("Mars"));
    }
}
