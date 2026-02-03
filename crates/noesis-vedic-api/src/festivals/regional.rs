//! Regional festival variations

use super::{Festival, FestivalCategory, PanchangCriteria};
use chrono::NaiveDate;

/// Get region-specific festivals
pub fn get_regional_festivals(year: i32, region: &str) -> Vec<Festival> {
    match region.to_lowercase().as_str() {
        "south" | "south india" | "tamil nadu" | "karnataka" | "kerala" | "andhra" => {
            get_south_indian_festivals(year)
        }
        "north" | "north india" | "uttar pradesh" | "bihar" | "punjab" => {
            get_north_indian_festivals(year)
        }
        "west" | "west india" | "gujarat" | "maharashtra" | "rajasthan" => {
            get_west_indian_festivals(year)
        }
        "east" | "east india" | "bengal" | "odisha" | "assam" => {
            get_east_indian_festivals(year)
        }
        _ => vec![],
    }
}

fn get_south_indian_festivals(year: i32) -> Vec<Festival> {
    vec![
        Festival {
            name: "Pongal".to_string(),
            date: NaiveDate::from_ymd_opt(year, 1, 14).unwrap(),
            category: FestivalCategory::Regional,
            deity: Some("Sun".to_string()),
            regions: vec!["Tamil Nadu".to_string()],
            description: "Harvest festival of Tamil Nadu. Four-day celebration.".to_string(),
            rituals: vec![
                "Cooking Pongal dish".to_string(),
                "Decorating cattle".to_string(),
                "Kolam drawings".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: None,
                nakshatra: None,
                month: Some("Thai".to_string()),
                paksha: None,
            },
        },
        Festival {
            name: "Ugadi".to_string(),
            date: NaiveDate::from_ymd_opt(year, 4, 9).unwrap(),
            category: FestivalCategory::Regional,
            deity: Some("Brahma".to_string()),
            regions: vec!["Karnataka".to_string(), "Andhra Pradesh".to_string(), "Telangana".to_string()],
            description: "Telugu and Kannada New Year.".to_string(),
            rituals: vec![
                "Bevu-Bella (neem and jaggery)".to_string(),
                "Panchanga Shravana".to_string(),
                "New clothes".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: Some("Pratipada".to_string()),
                nakshatra: None,
                month: Some("Chaitra".to_string()),
                paksha: Some("Shukla".to_string()),
            },
        },
        Festival {
            name: "Onam".to_string(),
            date: NaiveDate::from_ymd_opt(year, 9, 7).unwrap(),
            category: FestivalCategory::Regional,
            deity: Some("Mahabali".to_string()),
            regions: vec!["Kerala".to_string()],
            description: "Harvest festival of Kerala. King Mahabali's annual visit.".to_string(),
            rituals: vec![
                "Onam Sadhya (feast)".to_string(),
                "Pookalam (flower carpet)".to_string(),
                "Boat races".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: None,
                nakshatra: Some("Thiruvonam".to_string()),
                month: Some("Chingam".to_string()),
                paksha: None,
            },
        },
    ]
}

fn get_north_indian_festivals(year: i32) -> Vec<Festival> {
    vec![
        Festival {
            name: "Chhath Puja".to_string(),
            date: NaiveDate::from_ymd_opt(year, 11, 7).unwrap(),
            category: FestivalCategory::Regional,
            deity: Some("Surya".to_string()),
            regions: vec!["Bihar".to_string(), "Jharkhand".to_string(), "Eastern UP".to_string()],
            description: "Sun worship festival. Four-day rigorous fasting.".to_string(),
            rituals: vec![
                "Standing in water".to_string(),
                "Offering arghya".to_string(),
                "36-hour fast".to_string(),
            ],
            fasting: Some(super::FastingInfo {
                fasting_type: "Nirjala (waterless)".to_string(),
                duration: "36 hours".to_string(),
                breaking_time: "After giving arghya to rising sun".to_string(),
                exemptions: vec!["First-time observers can have water".to_string()],
            }),
            panchang_criteria: PanchangCriteria {
                tithi: Some("Shashti".to_string()),
                nakshatra: None,
                month: Some("Kartik".to_string()),
                paksha: Some("Shukla".to_string()),
            },
        },
        Festival {
            name: "Lohri".to_string(),
            date: NaiveDate::from_ymd_opt(year, 1, 13).unwrap(),
            category: FestivalCategory::Regional,
            deity: None,
            regions: vec!["Punjab".to_string(), "Haryana".to_string()],
            description: "Winter bonfire festival marking end of winter.".to_string(),
            rituals: vec![
                "Bonfire".to_string(),
                "Throwing til (sesame) and gur (jaggery)".to_string(),
                "Bhangra dance".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: None,
                nakshatra: None,
                month: Some("Paush".to_string()),
                paksha: None,
            },
        },
    ]
}

fn get_west_indian_festivals(year: i32) -> Vec<Festival> {
    vec![
        Festival {
            name: "Ganesh Chaturthi".to_string(),
            date: NaiveDate::from_ymd_opt(year, 9, 7).unwrap(),
            category: FestivalCategory::Regional,
            deity: Some("Ganesha".to_string()),
            regions: vec!["Maharashtra".to_string(), "Gujarat".to_string()],
            description: "Birth of Lord Ganesha. 10-day celebration.".to_string(),
            rituals: vec![
                "Ganesh idol installation".to_string(),
                "Daily aarti".to_string(),
                "Modak offerings".to_string(),
                "Visarjan (immersion)".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: Some("Chaturthi".to_string()),
                nakshatra: None,
                month: Some("Bhadrapada".to_string()),
                paksha: Some("Shukla".to_string()),
            },
        },
        Festival {
            name: "Navratri (Gujarat)".to_string(),
            date: NaiveDate::from_ymd_opt(year, 10, 3).unwrap(),
            category: FestivalCategory::Regional,
            deity: Some("Durga".to_string()),
            regions: vec!["Gujarat".to_string()],
            description: "Nine nights of Garba and Dandiya Raas.".to_string(),
            rituals: vec![
                "Garba dance".to_string(),
                "Dandiya Raas".to_string(),
                "Fasting".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: Some("Pratipada to Navami".to_string()),
                nakshatra: None,
                month: Some("Ashwin".to_string()),
                paksha: Some("Shukla".to_string()),
            },
        },
    ]
}

fn get_east_indian_festivals(year: i32) -> Vec<Festival> {
    vec![
        Festival {
            name: "Durga Puja".to_string(),
            date: NaiveDate::from_ymd_opt(year, 10, 10).unwrap(),
            category: FestivalCategory::Regional,
            deity: Some("Durga".to_string()),
            regions: vec!["Bengal".to_string(), "Odisha".to_string(), "Assam".to_string()],
            description: "Five-day worship of Goddess Durga. Biggest festival of Bengal.".to_string(),
            rituals: vec![
                "Pandal visits".to_string(),
                "Kumari puja".to_string(),
                "Sindoor khela".to_string(),
                "Bijaya Dashami".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: Some("Shashti to Dashami".to_string()),
                nakshatra: None,
                month: Some("Ashwin".to_string()),
                paksha: Some("Shukla".to_string()),
            },
        },
        Festival {
            name: "Rath Yatra".to_string(),
            date: NaiveDate::from_ymd_opt(year, 7, 7).unwrap(),
            category: FestivalCategory::Regional,
            deity: Some("Jagannath".to_string()),
            regions: vec!["Odisha".to_string()],
            description: "Chariot festival of Lord Jagannath.".to_string(),
            rituals: vec![
                "Chariot procession".to_string(),
                "Rope pulling".to_string(),
                "Prasad distribution".to_string(),
            ],
            fasting: None,
            panchang_criteria: PanchangCriteria {
                tithi: Some("Dwitiya".to_string()),
                nakshatra: None,
                month: Some("Ashadha".to_string()),
                paksha: Some("Shukla".to_string()),
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regional_festivals() {
        let south = get_regional_festivals(2024, "South India");
        assert!(!south.is_empty());
        assert!(south.iter().any(|f| f.name == "Pongal"));
        
        let north = get_regional_festivals(2024, "North");
        assert!(north.iter().any(|f| f.name == "Chhath Puja"));
    }
}
