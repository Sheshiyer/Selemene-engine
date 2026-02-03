//! Business Muhurta calculation
//!
//! FAPI-084: Calculate business Muhurta

use super::types::MuhurtaQuality;

/// Tithis favorable for business
const BUSINESS_GOOD_TITHIS: &[&str] = &[
    "Dwitiya", "Tritiya", "Panchami", "Saptami", "Dashami",
    "Ekadashi", "Trayodashi", "Purnima",
];

/// Nakshatras favorable for business
const BUSINESS_GOOD_NAKSHATRAS: &[&str] = &[
    "Ashwini", "Rohini", "Mrigashira", "Punarvasu", "Pushya",
    "Hasta", "Chitra", "Swati", "Anuradha", "Shravana", "Dhanishta", "Revati",
];

/// Days favorable for business
const BUSINESS_GOOD_DAYS: &[&str] = &[
    "Wednesday", "Thursday", "Friday",
];

/// Evaluate business muhurta score
pub fn evaluate_business_muhurta(
    tithi: &str,
    nakshatra: &str,
    yoga: &str,
    vara: &str,
    has_rahu_kalam: bool,
    hora_lord: &str,
) -> (MuhurtaQuality, u8, Vec<String>, Vec<String>) {
    let mut score: i32 = 50;
    let mut favorable = vec![];
    let mut unfavorable = vec![];
    
    // Check Tithi
    if BUSINESS_GOOD_TITHIS.iter().any(|t| tithi.contains(t)) {
        score += 10;
        favorable.push(format!("{} is favorable for business", tithi));
    }
    
    // Check Nakshatra
    if BUSINESS_GOOD_NAKSHATRAS.iter().any(|n| nakshatra.contains(n)) {
        score += 15;
        favorable.push(format!("{} supports business activities", nakshatra));
    }
    
    // Check Vara (weekday)
    if BUSINESS_GOOD_DAYS.iter().any(|d| vara.eq_ignore_ascii_case(d)) {
        score += 10;
        favorable.push(format!("{} is excellent for business", vara));
    } else if vara.eq_ignore_ascii_case("Saturday") {
        score -= 10;
        unfavorable.push("Saturday requires caution for new ventures".to_string());
    }
    
    // Check Hora Lord
    match hora_lord.to_lowercase().as_str() {
        "mercury" => {
            score += 15;
            favorable.push("Mercury hora - excellent for commerce and communication".to_string());
        }
        "jupiter" => {
            score += 10;
            favorable.push("Jupiter hora - good for expansion and growth".to_string());
        }
        "venus" => {
            score += 10;
            favorable.push("Venus hora - favorable for partnerships".to_string());
        }
        "sun" => {
            score += 5;
            favorable.push("Sun hora - good for authority matters".to_string());
        }
        "saturn" => {
            score -= 5;
            unfavorable.push("Saturn hora - better for consolidation than new starts".to_string());
        }
        "mars" => {
            score -= 5;
            unfavorable.push("Mars hora - may cause conflicts".to_string());
        }
        _ => {}
    }
    
    // Check Rahu Kalam
    if has_rahu_kalam {
        score -= 20;
        unfavorable.push("Rahu Kalam - avoid starting new business".to_string());
    }
    
    let final_score = score.clamp(0, 100) as u8;
    
    let quality = if final_score >= 75 {
        MuhurtaQuality::Excellent
    } else if final_score >= 55 {
        MuhurtaQuality::Good
    } else if final_score >= 40 {
        MuhurtaQuality::Average
    } else if final_score >= 25 {
        MuhurtaQuality::NotRecommended
    } else {
        MuhurtaQuality::Avoid
    };
    
    (quality, final_score, favorable, unfavorable)
}

/// Generate business muhurta recommendation
pub fn business_recommendation(activity_type: &str, quality: MuhurtaQuality) -> String {
    let base = match quality {
        MuhurtaQuality::Excellent => "Highly favorable time for",
        MuhurtaQuality::Good => "Good time for",
        MuhurtaQuality::Average => "Acceptable time for",
        MuhurtaQuality::NotRecommended => "Not ideal for",
        MuhurtaQuality::Avoid => "Avoid this time for",
    };
    
    format!("{} {}.", base, activity_type)
}

/// Specific recommendations for different business activities
pub fn get_business_activity_tips(activity: &str) -> Vec<String> {
    match activity.to_lowercase().as_str() {
        "signing contracts" => vec![
            "Prefer Mercury hora for contracts".to_string(),
            "Wednesday is most auspicious".to_string(),
            "Avoid retrograde Mercury periods".to_string(),
        ],
        "opening business" | "new venture" => vec![
            "Choose a day when Moon is waxing".to_string(),
            "Prefer Pushya or Shravana nakshatra".to_string(),
            "Thursday ruled by Jupiter brings growth".to_string(),
        ],
        "financial transactions" => vec![
            "Venus hora favors financial dealings".to_string(),
            "Avoid Amavasya (new moon) for major transactions".to_string(),
            "Friday is excellent for financial matters".to_string(),
        ],
        "interviews" | "meetings" => vec![
            "Mercury or Jupiter hora is best".to_string(),
            "Wednesday or Thursday preferred".to_string(),
            "Avoid Mars hora to prevent conflicts".to_string(),
        ],
        _ => vec![
            "Avoid Rahu Kalam for any new start".to_string(),
            "Prefer waxing Moon phase".to_string(),
            "Check for auspicious nakshatra".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_business_muhurta() {
        let (quality, score, favorable, _) = evaluate_business_muhurta(
            "Panchami",
            "Pushya",
            "Siddha",
            "Wednesday",
            false,
            "Mercury",
        );
        
        assert!(score >= 60);
        assert!(matches!(quality, MuhurtaQuality::Excellent | MuhurtaQuality::Good));
        assert!(favorable.len() >= 2);
    }

    #[test]
    fn test_business_tips() {
        let tips = get_business_activity_tips("signing contracts");
        assert!(!tips.is_empty());
        assert!(tips.iter().any(|t| t.contains("Mercury")));
    }
}
