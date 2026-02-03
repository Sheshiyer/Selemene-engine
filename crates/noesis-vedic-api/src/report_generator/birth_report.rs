//! Birth report generation

use super::types::{BirthDetails, GeneratedReport, ReportSectionContent};
use super::{DetailLevel, ReportConfig, ReportSection};
use chrono::Utc;

/// Generate a birth chart report
pub fn generate_birth_report(
    details: &BirthDetails,
    config: &ReportConfig,
) -> GeneratedReport {
    let mut sections = vec![];
    
    for section in &config.sections {
        let content = generate_section(section, details, config.detail_level);
        sections.push(content);
    }
    
    GeneratedReport {
        title: format!("Birth Chart Report for {}", details.name),
        subject_name: details.name.clone(),
        birth_datetime: details.datetime,
        generated_at: Utc::now().naive_utc(),
        sections,
        summary: generate_summary(details),
    }
}

fn generate_section(
    section: &ReportSection,
    details: &BirthDetails,
    detail_level: DetailLevel,
) -> ReportSectionContent {
    match section {
        ReportSection::PersonalInfo => ReportSectionContent {
            title: "Personal Information".to_string(),
            content: format!(
                "Name: {}\nBirth Date: {}\nBirth Time: {}\nLocation: {:.4}°N, {:.4}°E",
                details.name,
                details.datetime.date(),
                details.datetime.time(),
                details.latitude,
                details.longitude
            ),
            key_points: vec![],
            chart_data: None,
        },
        ReportSection::BirthChart => ReportSectionContent {
            title: "Birth Chart (Rashi)".to_string(),
            content: "The birth chart shows the position of planets at the time of birth.".to_string(),
            key_points: vec![
                "Ascendant determines physical appearance and personality".to_string(),
                "Moon sign indicates emotional nature".to_string(),
                "Sun sign shows soul purpose".to_string(),
            ],
            chart_data: None, // Would be populated with actual chart
        },
        ReportSection::PlanetPositions => ReportSectionContent {
            title: "Planet Positions".to_string(),
            content: "Analysis of each planet's placement in signs and houses.".to_string(),
            key_points: vec![
                "Benefic planets in angles strengthen the chart".to_string(),
                "Malefic planets need careful analysis".to_string(),
            ],
            chart_data: None,
        },
        ReportSection::HouseCusps => ReportSectionContent {
            title: "House Analysis".to_string(),
            content: "The twelve houses govern different areas of life.".to_string(),
            key_points: vec![
                "1st house: Self, personality, physical body".to_string(),
                "7th house: Partnerships, marriage".to_string(),
                "10th house: Career, public life".to_string(),
            ],
            chart_data: None,
        },
        ReportSection::DivisionalCharts => ReportSectionContent {
            title: "Divisional Charts (Vargas)".to_string(),
            content: "Divisional charts provide deeper insights into specific life areas.".to_string(),
            key_points: vec![
                "Navamsa (D9): Marriage and dharma".to_string(),
                "Dasamsa (D10): Career and profession".to_string(),
            ],
            chart_data: None,
        },
        ReportSection::Yogas => ReportSectionContent {
            title: "Yoga Analysis".to_string(),
            content: "Special planetary combinations that influence destiny.".to_string(),
            key_points: vec![
                "Raj Yogas indicate success and fame".to_string(),
                "Dhana Yogas indicate wealth".to_string(),
            ],
            chart_data: None,
        },
        ReportSection::DashaPeriods => ReportSectionContent {
            title: "Dasha Periods".to_string(),
            content: "Planetary periods that determine timing of events.".to_string(),
            key_points: vec![
                "Current Mahadasha influences overall life direction".to_string(),
                "Antardasha shows sub-periods within main period".to_string(),
            ],
            chart_data: None,
        },
        ReportSection::Remedies => ReportSectionContent {
            title: "Remedial Measures".to_string(),
            content: "Recommended remedies to enhance positive influences.".to_string(),
            key_points: vec![
                "Gemstone recommendations".to_string(),
                "Mantra practices".to_string(),
                "Charitable activities".to_string(),
            ],
            chart_data: None,
        },
        ReportSection::Summary => ReportSectionContent {
            title: "Summary".to_string(),
            content: generate_summary(details),
            key_points: vec![],
            chart_data: None,
        },
    }
}

fn generate_summary(details: &BirthDetails) -> String {
    format!(
        "This report provides a comprehensive analysis of the birth chart for {}. \
         The planetary positions at the time of birth indicate unique strengths and \
         areas for growth. Please consult with a qualified astrologer for personalized guidance.",
        details.name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_generate_birth_report() {
        let details = BirthDetails {
            name: "Test Person".to_string(),
            datetime: NaiveDate::from_ymd_opt(1990, 1, 15)
                .unwrap()
                .and_hms_opt(10, 30, 0)
                .unwrap(),
            latitude: 12.97,
            longitude: 77.59,
            timezone: 5.5,
            place_name: Some("Bangalore".to_string()),
        };
        
        let config = ReportConfig::default();
        let report = generate_birth_report(&details, &config);
        
        assert_eq!(report.subject_name, "Test Person");
        assert!(!report.sections.is_empty());
    }
}
