//! Marriage Muhurta calculation
//!
//! FAPI-083: Calculate marriage Muhurta

use super::types::{SelectedMuhurta, MuhurtaQuality, MuhurtaDosha};

/// Tithis favorable for marriage
const MARRIAGE_GOOD_TITHIS: &[&str] = &[
    "Dwitiya", "Tritiya", "Panchami", "Saptami", "Ekadashi", "Trayodashi",
];

/// Tithis to avoid for marriage
const MARRIAGE_BAD_TITHIS: &[&str] = &[
    "Amavasya", "Purnima", "Chaturthi", "Navami", "Chaturdashi",
];

/// Nakshatras favorable for marriage
const MARRIAGE_GOOD_NAKSHATRAS: &[&str] = &[
    "Rohini", "Mrigashira", "Magha", "Uttara Phalguni", "Hasta",
    "Swati", "Anuradha", "Mula", "Uttara Ashadha", "Uttara Bhadrapada", "Revati",
];

/// Nakshatras to avoid for marriage
const MARRIAGE_BAD_NAKSHATRAS: &[&str] = &[
    "Bharani", "Krittika", "Ardra", "Ashlesha", "Vishakha", "Jyeshtha",
];

/// Months to avoid for marriage (adhik mas, pitru paksha, etc.)
const MARRIAGE_BAD_MONTHS: &[&str] = &[
    "Adhik Mas", "Pitru Paksha",
];

/// Evaluate marriage muhurta score
pub fn evaluate_marriage_muhurta(
    tithi: &str,
    nakshatra: &str,
    yoga: &str,
    vara: &str,
    has_rahu_kalam: bool,
    has_yama_gandam: bool,
) -> (MuhurtaQuality, u8, Vec<String>, Vec<String>) {
    let mut score: i32 = 50; // Start at 50
    let mut favorable = vec![];
    let mut unfavorable = vec![];
    
    // Check Tithi
    if MARRIAGE_GOOD_TITHIS.iter().any(|t| tithi.contains(t)) {
        score += 15;
        favorable.push(format!("{} is auspicious for marriage", tithi));
    } else if MARRIAGE_BAD_TITHIS.iter().any(|t| tithi.contains(t)) {
        score -= 20;
        unfavorable.push(format!("{} is not ideal for marriage", tithi));
    }
    
    // Check Nakshatra
    if MARRIAGE_GOOD_NAKSHATRAS.iter().any(|n| nakshatra.contains(n)) {
        score += 15;
        favorable.push(format!("{} nakshatra is very auspicious", nakshatra));
    } else if MARRIAGE_BAD_NAKSHATRAS.iter().any(|n| nakshatra.contains(n)) {
        score -= 20;
        unfavorable.push(format!("{} nakshatra should be avoided", nakshatra));
    }
    
    // Check Vara (weekday)
    match vara.to_lowercase().as_str() {
        "monday" | "wednesday" | "thursday" | "friday" => {
            score += 10;
            favorable.push(format!("{} is good for marriage", vara));
        }
        "tuesday" | "saturday" => {
            score -= 15;
            unfavorable.push(format!("{} is not preferred for marriage", vara));
        }
        "sunday" => {
            score += 5;
        }
        _ => {}
    }
    
    // Check for doshas
    if has_rahu_kalam {
        score -= 25;
        unfavorable.push("Rahu Kalam active - avoid this period".to_string());
    }
    
    if has_yama_gandam {
        score -= 20;
        unfavorable.push("Yama Gandam active - not recommended".to_string());
    }
    
    // Clamp score
    let final_score = score.clamp(0, 100) as u8;
    
    // Determine quality
    let quality = if final_score >= 80 {
        MuhurtaQuality::Excellent
    } else if final_score >= 60 {
        MuhurtaQuality::Good
    } else if final_score >= 40 {
        MuhurtaQuality::Average
    } else if final_score >= 20 {
        MuhurtaQuality::NotRecommended
    } else {
        MuhurtaQuality::Avoid
    };
    
    (quality, final_score, favorable, unfavorable)
}

/// Generate marriage muhurta recommendation
pub fn marriage_recommendation(quality: MuhurtaQuality, score: u8) -> String {
    match quality {
        MuhurtaQuality::Excellent => {
            "Highly auspicious time for marriage ceremonies. \
             All major factors are favorable.".to_string()
        }
        MuhurtaQuality::Good => {
            "Good muhurta for marriage. Minor unfavorable factors can be \
             mitigated with appropriate remedies.".to_string()
        }
        MuhurtaQuality::Average => {
            "Average muhurta. Consider finding a better time if possible, \
             or perform remedial measures.".to_string()
        }
        MuhurtaQuality::NotRecommended => {
            "This muhurta has significant unfavorable factors. \
             It's advisable to look for an alternative time.".to_string()
        }
        MuhurtaQuality::Avoid => {
            "This time should be avoided for marriage. \
             Several major doshas are present.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_marriage_muhurta() {
        let (quality, score, favorable, _) = evaluate_marriage_muhurta(
            "Panchami",
            "Rohini",
            "Siddha",
            "Thursday",
            false,
            false,
        );
        
        assert!(score >= 60);
        assert!(matches!(quality, MuhurtaQuality::Excellent | MuhurtaQuality::Good));
        assert!(!favorable.is_empty());
    }

    #[test]
    fn test_bad_marriage_muhurta() {
        let (quality, score, _, unfavorable) = evaluate_marriage_muhurta(
            "Amavasya",
            "Bharani",
            "Vyatipata",
            "Saturday",
            true,
            true,
        );
        
        assert!(score < 40);
        assert!(matches!(quality, MuhurtaQuality::NotRecommended | MuhurtaQuality::Avoid));
        assert!(!unfavorable.is_empty());
    }
}
