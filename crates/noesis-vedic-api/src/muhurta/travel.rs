//! Travel Muhurta calculation
//!
//! FAPI-085: Calculate travel Muhurta

use super::types::MuhurtaQuality;

/// Nakshatras favorable for travel
const TRAVEL_GOOD_NAKSHATRAS: &[&str] = &[
    "Ashwini", "Mrigashira", "Punarvasu", "Pushya", "Hasta",
    "Anuradha", "Shravana", "Revati",
];

/// Nakshatras to avoid for travel
const TRAVEL_BAD_NAKSHATRAS: &[&str] = &[
    "Bharani", "Krittika", "Ardra", "Ashlesha", "Magha",
    "Vishakha", "Jyeshtha", "Mula",
];

/// Directions favorable on each weekday (Panchak)
pub fn get_favorable_direction(weekday: &str) -> Option<&'static str> {
    match weekday.to_lowercase().as_str() {
        "sunday" => Some("East"),
        "monday" => Some("North"),
        "tuesday" => Some("South"),
        "wednesday" => Some("North"),
        "thursday" => Some("East"),
        "friday" => Some("West"),
        "saturday" => Some("West"),
        _ => None,
    }
}

/// Evaluate travel muhurta
pub fn evaluate_travel_muhurta(
    nakshatra: &str,
    vara: &str,
    has_rahu_kalam: bool,
    travel_direction: Option<&str>,
) -> (MuhurtaQuality, u8, Vec<String>, Vec<String>) {
    let mut score: i32 = 50;
    let mut favorable = vec![];
    let mut unfavorable = vec![];
    
    // Check Nakshatra
    if TRAVEL_GOOD_NAKSHATRAS.iter().any(|n| nakshatra.contains(n)) {
        score += 20;
        favorable.push(format!("{} is excellent for travel", nakshatra));
    } else if TRAVEL_BAD_NAKSHATRAS.iter().any(|n| nakshatra.contains(n)) {
        score -= 20;
        unfavorable.push(format!("{} is not ideal for travel", nakshatra));
    }
    
    // Check Vara
    match vara.to_lowercase().as_str() {
        "sunday" | "wednesday" | "thursday" | "friday" => {
            score += 10;
            favorable.push(format!("{} is good for travel", vara));
        }
        "tuesday" => {
            score -= 10;
            unfavorable.push("Tuesday requires caution for travel".to_string());
        }
        "saturday" => {
            score -= 5;
            unfavorable.push("Saturday travel may face delays".to_string());
        }
        _ => {}
    }
    
    // Check Rahu Kalam
    if has_rahu_kalam {
        score -= 25;
        unfavorable.push("Never start travel during Rahu Kalam".to_string());
    }
    
    // Check direction compatibility
    if let Some(direction) = travel_direction {
        if let Some(good_dir) = get_favorable_direction(vara) {
            if direction.eq_ignore_ascii_case(good_dir) {
                score += 10;
                favorable.push(format!("Traveling {} on {} is auspicious", direction, vara));
            }
        }
    }
    
    let final_score = score.clamp(0, 100) as u8;
    
    let quality = if final_score >= 70 {
        MuhurtaQuality::Excellent
    } else if final_score >= 50 {
        MuhurtaQuality::Good
    } else if final_score >= 35 {
        MuhurtaQuality::Average
    } else if final_score >= 20 {
        MuhurtaQuality::NotRecommended
    } else {
        MuhurtaQuality::Avoid
    };
    
    (quality, final_score, favorable, unfavorable)
}

/// Get travel safety tips based on muhurta
pub fn travel_safety_tips(quality: MuhurtaQuality) -> Vec<String> {
    let mut tips = vec![
        "Offer prayers before starting journey".to_string(),
        "Carry some dry fruits and water".to_string(),
    ];
    
    match quality {
        MuhurtaQuality::Excellent | MuhurtaQuality::Good => {
            tips.push("This is an auspicious time to begin travel".to_string());
        }
        MuhurtaQuality::Average => {
            tips.push("Take extra precautions during travel".to_string());
            tips.push("Avoid overnight stops if possible".to_string());
        }
        MuhurtaQuality::NotRecommended | MuhurtaQuality::Avoid => {
            tips.push("Postpone travel if possible".to_string());
            tips.push("If travel is essential, chant Hanuman Chalisa".to_string());
            tips.push("Avoid traveling alone".to_string());
        }
    }
    
    tips
}

/// Check for Panchak dosha (certain nakshatras to avoid)
pub fn check_panchak(nakshatra: &str) -> bool {
    let panchak_nakshatras = ["Dhanishta", "Shatabhisha", "Purva Bhadrapada", 
                             "Uttara Bhadrapada", "Revati"];
    panchak_nakshatras.iter().any(|n| nakshatra.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_favorable_direction() {
        assert_eq!(get_favorable_direction("Sunday"), Some("East"));
        assert_eq!(get_favorable_direction("Thursday"), Some("East"));
    }

    #[test]
    fn test_travel_muhurta() {
        let (quality, score, favorable, _) = evaluate_travel_muhurta(
            "Pushya",
            "Thursday",
            false,
            Some("East"),
        );
        
        assert!(score >= 60);
        assert!(!favorable.is_empty());
    }

    #[test]
    fn test_panchak() {
        assert!(check_panchak("Dhanishta"));
        assert!(!check_panchak("Rohini"));
    }
}
