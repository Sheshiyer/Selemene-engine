//! General activity Muhurta calculation
//!
//! FAPI-086: Calculate general activity Muhurta

use super::types::MuhurtaQuality;

/// Evaluate general activity muhurta
pub fn evaluate_general_muhurta(
    tithi: &str,
    nakshatra: &str,
    yoga: &str,
    vara: &str,
    has_rahu_kalam: bool,
    has_gulika_kaal: bool,
) -> (MuhurtaQuality, u8, Vec<String>, Vec<String>) {
    let mut score: i32 = 50;
    let mut favorable = vec![];
    let mut unfavorable = vec![];
    
    // Tithi evaluation
    let good_tithis = ["Dwitiya", "Tritiya", "Panchami", "Saptami", 
                       "Dashami", "Ekadashi", "Trayodashi"];
    let bad_tithis = ["Chaturthi", "Navami", "Chaturdashi", "Amavasya"];
    
    if good_tithis.iter().any(|t| tithi.contains(t)) {
        score += 10;
        favorable.push(format!("{} is auspicious", tithi));
    } else if bad_tithis.iter().any(|t| tithi.contains(t)) {
        score -= 10;
        unfavorable.push(format!("{} is not ideal", tithi));
    }
    
    // Nakshatra evaluation
    let good_nakshatras = ["Ashwini", "Rohini", "Mrigashira", "Punarvasu", 
                          "Pushya", "Hasta", "Chitra", "Swati", 
                          "Anuradha", "Shravana", "Dhanishta", "Revati"];
    let bad_nakshatras = ["Bharani", "Krittika", "Ardra", "Ashlesha", 
                         "Vishakha", "Jyeshtha", "Mula"];
    
    if good_nakshatras.iter().any(|n| nakshatra.contains(n)) {
        score += 15;
        favorable.push(format!("{} nakshatra is favorable", nakshatra));
    } else if bad_nakshatras.iter().any(|n| nakshatra.contains(n)) {
        score -= 15;
        unfavorable.push(format!("{} nakshatra requires caution", nakshatra));
    }
    
    // Yoga evaluation
    let good_yogas = ["Siddhi", "Amrita", "Shubha", "Sadhya", "Shiva"];
    let bad_yogas = ["Vishkumbha", "Atiganda", "Shoola", "Ganda", "Vyatipata"];
    
    if good_yogas.iter().any(|y| yoga.contains(y)) {
        score += 10;
        favorable.push(format!("{} yoga is beneficial", yoga));
    } else if bad_yogas.iter().any(|y| yoga.contains(y)) {
        score -= 10;
        unfavorable.push(format!("{} yoga is challenging", yoga));
    }
    
    // Vara evaluation
    match vara.to_lowercase().as_str() {
        "monday" | "wednesday" | "thursday" | "friday" => {
            score += 5;
            favorable.push(format!("{} is generally favorable", vara));
        }
        "tuesday" | "saturday" => {
            score -= 5;
            unfavorable.push(format!("{} requires extra care", vara));
        }
        _ => {}
    }
    
    // Dosha evaluation
    if has_rahu_kalam {
        score -= 20;
        unfavorable.push("Rahu Kalam active".to_string());
    }
    
    if has_gulika_kaal {
        score -= 15;
        unfavorable.push("Gulika Kaal active".to_string());
    }
    
    let final_score = score.clamp(0, 100) as u8;
    
    let quality = if final_score >= 70 {
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

/// Get general recommendations based on panchanga
pub fn general_recommendations(
    quality: MuhurtaQuality,
    favorable: &[String],
    unfavorable: &[String],
) -> String {
    let mut rec = String::new();
    
    match quality {
        MuhurtaQuality::Excellent => {
            rec.push_str("This is an excellent time for most activities. ");
            if !favorable.is_empty() {
                rec.push_str(&format!("Favorable factors: {}. ", favorable.join(", ")));
            }
        }
        MuhurtaQuality::Good => {
            rec.push_str("This is a good time for most activities. ");
        }
        MuhurtaQuality::Average => {
            rec.push_str("This is an average muhurta. Proceed with care. ");
            if !unfavorable.is_empty() {
                rec.push_str(&format!("Note: {}. ", unfavorable.join("; ")));
            }
        }
        MuhurtaQuality::NotRecommended => {
            rec.push_str("This muhurta is not recommended for important activities. ");
            rec.push_str("Consider postponing if possible. ");
        }
        MuhurtaQuality::Avoid => {
            rec.push_str("Avoid important activities during this time. ");
            rec.push_str("Wait for a more auspicious muhurta. ");
        }
    }
    
    rec
}

/// Quick check if current time is generally auspicious
pub fn quick_auspicious_check(
    has_rahu_kalam: bool,
    has_yama_gandam: bool,
    has_gulika_kaal: bool,
) -> (bool, String) {
    let is_auspicious = !has_rahu_kalam && !has_yama_gandam && !has_gulika_kaal;
    
    let message = if is_auspicious {
        "Current time is free from major doshas".to_string()
    } else {
        let mut doshas = vec![];
        if has_rahu_kalam { doshas.push("Rahu Kalam"); }
        if has_yama_gandam { doshas.push("Yama Gandam"); }
        if has_gulika_kaal { doshas.push("Gulika Kaal"); }
        format!("Active doshas: {}", doshas.join(", "))
    };
    
    (is_auspicious, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_muhurta_good() {
        let (quality, score, favorable, _) = evaluate_general_muhurta(
            "Panchami",
            "Pushya",
            "Siddhi",
            "Thursday",
            false,
            false,
        );
        
        assert!(score >= 60);
        assert!(favorable.len() >= 3);
    }

    #[test]
    fn test_general_muhurta_bad() {
        let (quality, score, _, unfavorable) = evaluate_general_muhurta(
            "Navami",
            "Ashlesha",
            "Ganda",
            "Saturday",
            true,
            true,
        );
        
        assert!(score < 40);
        assert!(unfavorable.len() >= 3);
    }

    #[test]
    fn test_quick_check() {
        let (is_good, _) = quick_auspicious_check(false, false, false);
        assert!(is_good);
        
        let (is_good, msg) = quick_auspicious_check(true, false, false);
        assert!(!is_good);
        assert!(msg.contains("Rahu"));
    }
}
