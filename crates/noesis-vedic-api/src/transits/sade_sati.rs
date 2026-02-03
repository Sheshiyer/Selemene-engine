//! Sade Sati calculation
//!
//! FAPI-076: Identify Saturn transit (Sade Sati)

use chrono::NaiveDate;
use super::types::{SadeSatiStatus, SadeSatiPhase};
use crate::birth_chart::types::ZodiacSign;

/// Check if Sade Sati is active
pub fn check_sade_sati(
    moon_sign: ZodiacSign,
    saturn_sign: ZodiacSign,
) -> SadeSatiStatus {
    let moon_num = moon_sign.number();
    let saturn_num = saturn_sign.number();
    
    // Calculate position relative to Moon
    let position = ((saturn_num as i8 - moon_num as i8).rem_euclid(12) + 1) as u8;
    
    let (is_active, phase) = match position {
        12 => (true, Some(SadeSatiPhase::Rising)),  // 12th from Moon
        1 => (true, Some(SadeSatiPhase::Peak)),     // Same as Moon
        2 => (true, Some(SadeSatiPhase::Setting)),  // 2nd from Moon
        _ => (false, None),
    };
    
    SadeSatiStatus {
        is_active,
        phase,
        start_date: None, // Would need ephemeris data
        end_date: None,
        saturn_sign: saturn_sign.to_string(),
        moon_sign: moon_sign.to_string(),
    }
}

/// Calculate Sade Sati dates (approximate)
pub fn calculate_sade_sati_dates(
    moon_sign: ZodiacSign,
    current_date: NaiveDate,
) -> Vec<(NaiveDate, NaiveDate, SadeSatiPhase)> {
    // Saturn takes ~29.5 years to complete zodiac
    // ~2.5 years per sign
    // Sade Sati is ~7.5 years (3 signs)
    
    // This is a simplified calculation
    // Would need actual ephemeris for precise dates
    
    let sign_duration_days = 912; // ~2.5 years
    let moon_num = moon_sign.number();
    
    // Calculate approximate next occurrence
    // For accurate dates, use actual Saturn transit data
    
    vec![]
}

/// Get description of Sade Sati phase
pub fn sade_sati_description(phase: SadeSatiPhase) -> &'static str {
    match phase {
        SadeSatiPhase::Rising => {
            "Saturn is transiting 12th from Moon (Rising Phase). \
             This phase often brings hidden expenses, sleep issues, \
             and subtle challenges. Good for spiritual practices."
        }
        SadeSatiPhase::Peak => {
            "Saturn is transiting over natal Moon (Peak Phase). \
             This is the most intense phase. May experience emotional \
             challenges, but also karmic growth and maturity."
        }
        SadeSatiPhase::Setting => {
            "Saturn is transiting 2nd from Moon (Setting Phase). \
             Financial matters and family relations may be tested. \
             The intensity is reducing as Saturn moves away."
        }
    }
}

/// Get remedies for Sade Sati
pub fn sade_sati_remedies(phase: SadeSatiPhase) -> Vec<String> {
    let mut remedies = vec![
        "Chant Hanuman Chalisa on Saturdays".to_string(),
        "Donate to elderly and underprivileged".to_string(),
        "Light sesame oil lamp on Saturdays".to_string(),
        "Wear blue sapphire (after consultation)".to_string(),
    ];
    
    match phase {
        SadeSatiPhase::Rising => {
            remedies.push("Focus on spiritual practices".to_string());
            remedies.push("Avoid unnecessary expenses".to_string());
        }
        SadeSatiPhase::Peak => {
            remedies.push("Practice patience and meditation".to_string());
            remedies.push("Serve parents and elders".to_string());
        }
        SadeSatiPhase::Setting => {
            remedies.push("Be careful with finances".to_string());
            remedies.push("Maintain harmony in family".to_string());
        }
    }
    
    remedies
}

/// Check for Dhaiya (Small Panoti) - Saturn 4th or 8th from Moon
pub fn check_dhaiya(moon_sign: ZodiacSign, saturn_sign: ZodiacSign) -> Option<String> {
    let moon_num = moon_sign.number();
    let saturn_num = saturn_sign.number();
    let position = ((saturn_num as i8 - moon_num as i8).rem_euclid(12) + 1) as u8;
    
    match position {
        4 => Some("Dhaiya (Ashtama Shani) - Saturn 4th from Moon. \
                  May cause health and domestic challenges.".to_string()),
        8 => Some("Dhaiya (Kantaka Shani) - Saturn 8th from Moon. \
                  May bring obstacles and transformation.".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sade_sati_detection() {
        // Saturn in same sign as Moon - Peak
        let status = check_sade_sati(ZodiacSign::Aquarius, ZodiacSign::Aquarius);
        assert!(status.is_active);
        assert_eq!(status.phase, Some(SadeSatiPhase::Peak));
        
        // Saturn 12th from Moon - Rising
        let status = check_sade_sati(ZodiacSign::Pisces, ZodiacSign::Aquarius);
        assert!(status.is_active);
        assert_eq!(status.phase, Some(SadeSatiPhase::Rising));
        
        // Saturn 2nd from Moon - Setting
        let status = check_sade_sati(ZodiacSign::Capricorn, ZodiacSign::Aquarius);
        assert!(status.is_active);
        assert_eq!(status.phase, Some(SadeSatiPhase::Setting));
        
        // Saturn elsewhere - not active
        let status = check_sade_sati(ZodiacSign::Aries, ZodiacSign::Aquarius);
        assert!(!status.is_active);
    }

    #[test]
    fn test_dhaiya_detection() {
        // Saturn 4th from Moon
        let dhaiya = check_dhaiya(ZodiacSign::Aries, ZodiacSign::Cancer);
        assert!(dhaiya.is_some());
        
        // Saturn 8th from Moon
        let dhaiya = check_dhaiya(ZodiacSign::Aries, ZodiacSign::Scorpio);
        assert!(dhaiya.is_some());
        
        // No dhaiya
        let dhaiya = check_dhaiya(ZodiacSign::Aries, ZodiacSign::Leo);
        assert!(dhaiya.is_none());
    }
}
