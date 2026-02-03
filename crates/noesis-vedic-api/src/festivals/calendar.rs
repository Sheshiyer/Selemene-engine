//! Festival calendar calculations

use chrono::{NaiveDate, Datelike};
use super::{Festival, FestivalCategory, FestivalList, PanchangCriteria};

/// Get festivals for a date range
pub fn get_festivals_for_range(
    from_date: NaiveDate,
    to_date: NaiveDate,
    region: Option<&str>,
) -> FestivalList {
    let mut festivals = get_all_major_festivals(from_date.year());
    
    // Filter by date range
    festivals.retain(|f| f.date >= from_date && f.date <= to_date);
    
    // Filter by region if specified
    if let Some(reg) = region {
        festivals.retain(|f| {
            f.regions.is_empty() || f.regions.iter().any(|r| r.eq_ignore_ascii_case(reg))
        });
    }
    
    let total_count = festivals.len();
    
    FestivalList {
        from_date,
        to_date,
        festivals,
        total_count,
    }
}

/// Get major Hindu festivals for a year
pub fn get_all_major_festivals(year: i32) -> Vec<Festival> {
    vec![
        Festival {
            name: "Makar Sankranti".to_string(),
            date: NaiveDate::from_ymd_opt(year, 1, 14).unwrap(),
            category: FestivalCategory::Major,
            deity: Some("Sun".to_string()),
            regions: vec!["All India".to_string()],
            description: "Sun's transition into Capricorn (Makar). Marks end of winter solstice.".to_string(),
            rituals: vec![
                "Til-gul offerings".to_string(),
                "Kite flying".to_string(),
                "Holy bath in rivers".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: None,
                nakshatra: None,
                month: Some("Magha".to_string()),
                paksha: None,
            },
        },
        Festival {
            name: "Maha Shivaratri".to_string(),
            date: NaiveDate::from_ymd_opt(year, 3, 8).unwrap(), // Approximate
            category: FestivalCategory::Major,
            deity: Some("Shiva".to_string()),
            regions: vec!["All India".to_string()],
            description: "Night of Lord Shiva. One of the most auspicious nights.".to_string(),
            rituals: vec![
                "Night-long vigil".to_string(),
                "Shiva abhishek".to_string(),
                "Fasting".to_string(),
                "Rudra chanting".to_string(),
            ],
            fasting: Some(super::FastingInfo {
                fasting_type: "Full day".to_string(),
                duration: "24 hours".to_string(),
                breaking_time: "Next day morning".to_string(),
                exemptions: vec!["Elderly".to_string(), "Pregnant women".to_string()],
            }),
            panchang_criteria: PanchangCriteria {
                tithi: Some("Chaturdashi".to_string()),
                nakshatra: None,
                month: Some("Phalguna".to_string()),
                paksha: Some("Krishna".to_string()),
            },
        },
        Festival {
            name: "Holi".to_string(),
            date: NaiveDate::from_ymd_opt(year, 3, 25).unwrap(), // Approximate
            category: FestivalCategory::Major,
            deity: Some("Krishna".to_string()),
            regions: vec!["North India".to_string(), "West India".to_string()],
            description: "Festival of colors celebrating victory of good over evil.".to_string(),
            rituals: vec![
                "Holika dahan".to_string(),
                "Playing with colors".to_string(),
                "Thandai preparation".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: Some("Purnima".to_string()),
                nakshatra: None,
                month: Some("Phalguna".to_string()),
                paksha: Some("Shukla".to_string()),
            },
        },
        Festival {
            name: "Ram Navami".to_string(),
            date: NaiveDate::from_ymd_opt(year, 4, 17).unwrap(), // Approximate
            category: FestivalCategory::Major,
            deity: Some("Rama".to_string()),
            regions: vec!["All India".to_string()],
            description: "Birth anniversary of Lord Rama.".to_string(),
            rituals: vec![
                "Rama Katha".to_string(),
                "Temple visits".to_string(),
                "Fasting".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: Some("Navami".to_string()),
                nakshatra: None,
                month: Some("Chaitra".to_string()),
                paksha: Some("Shukla".to_string()),
            },
        },
        Festival {
            name: "Diwali".to_string(),
            date: NaiveDate::from_ymd_opt(year, 11, 1).unwrap(), // Approximate
            category: FestivalCategory::Major,
            deity: Some("Lakshmi".to_string()),
            regions: vec!["All India".to_string()],
            description: "Festival of lights. Victory of light over darkness.".to_string(),
            rituals: vec![
                "Lakshmi puja".to_string(),
                "Lighting diyas".to_string(),
                "Rangoli".to_string(),
                "Firecrackers".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: Some("Amavasya".to_string()),
                nakshatra: None,
                month: Some("Kartik".to_string()),
                paksha: Some("Krishna".to_string()),
            },
        },
    ]
}

/// Check if a date is a festival
pub fn is_festival_day(date: NaiveDate) -> bool {
    let festivals = get_all_major_festivals(date.year());
    festivals.iter().any(|f| f.date == date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_festivals() {
        let festivals = get_all_major_festivals(2024);
        assert!(!festivals.is_empty());
        assert!(festivals.iter().any(|f| f.name == "Diwali"));
    }

    #[test]
    fn test_festival_range() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        
        let list = get_festivals_for_range(from, to, None);
        assert!(list.total_count >= 5);
    }
}
