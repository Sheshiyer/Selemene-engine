//! Eclipse calculator

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use super::{EclipseEvent, EclipseType, VedicEclipseEffects, LocalEclipseVisibility};

/// Get eclipses for a year
pub fn get_eclipses_for_year(year: i32) -> Vec<EclipseEvent> {
    // In production, this would use astronomical calculations
    // These are approximate dates for demonstration
    vec![
        EclipseEvent {
            eclipse_type: EclipseType::SolarTotal,
            date: NaiveDate::from_ymd_opt(year, 4, 8).unwrap(),
            maximum_time: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, 4, 8).unwrap(),
                NaiveTime::from_hms_opt(18, 17, 0).unwrap(),
            ),
            totality_duration: Some("4m 28s".to_string()),
            zodiac_sign: "Pisces".to_string(),
            nakshatra: "Revati".to_string(),
            visibility: vec!["North America".to_string(), "Mexico".to_string()],
            magnitude: 1.0566,
            saros: 139,
            vedic_effects: create_vedic_effects("Pisces", "Revati", true),
        },
        EclipseEvent {
            eclipse_type: EclipseType::LunarPartial,
            date: NaiveDate::from_ymd_opt(year, 3, 25).unwrap(),
            maximum_time: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, 3, 25).unwrap(),
                NaiveTime::from_hms_opt(7, 12, 0).unwrap(),
            ),
            totality_duration: None,
            zodiac_sign: "Virgo".to_string(),
            nakshatra: "Hasta".to_string(),
            visibility: vec!["Americas".to_string(), "Europe".to_string(), "Africa".to_string()],
            magnitude: -0.132,
            saros: 113,
            vedic_effects: create_vedic_effects("Virgo", "Hasta", false),
        },
        EclipseEvent {
            eclipse_type: EclipseType::SolarAnnular,
            date: NaiveDate::from_ymd_opt(year, 10, 2).unwrap(),
            maximum_time: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, 10, 2).unwrap(),
                NaiveTime::from_hms_opt(18, 45, 0).unwrap(),
            ),
            totality_duration: Some("7m 25s".to_string()),
            zodiac_sign: "Virgo".to_string(),
            nakshatra: "Chitra".to_string(),
            visibility: vec!["South America".to_string()],
            magnitude: 0.9326,
            saros: 144,
            vedic_effects: create_vedic_effects("Virgo", "Chitra", true),
        },
        EclipseEvent {
            eclipse_type: EclipseType::LunarTotal,
            date: NaiveDate::from_ymd_opt(year, 9, 18).unwrap(),
            maximum_time: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, 9, 18).unwrap(),
                NaiveTime::from_hms_opt(2, 44, 0).unwrap(),
            ),
            totality_duration: Some("1h 3m".to_string()),
            zodiac_sign: "Pisces".to_string(),
            nakshatra: "Uttara Bhadrapada".to_string(),
            visibility: vec!["Americas".to_string(), "Europe".to_string(), "Africa".to_string()],
            magnitude: 1.0364,
            saros: 118,
            vedic_effects: create_vedic_effects("Pisces", "Uttara Bhadrapada", false),
        },
    ]
}

fn create_vedic_effects(sign: &str, nakshatra: &str, is_solar: bool) -> VedicEclipseEffects {
    let opposite_sign = get_opposite_sign(sign);
    
    let general = if is_solar {
        format!(
            "Solar eclipse in {} affects leadership, government, and ego matters. \
             Those with {} and {} ascendant or Moon should be cautious.",
            sign, sign, opposite_sign
        )
    } else {
        format!(
            "Lunar eclipse in {} affects emotions, mind, and public matters. \
             Mental peace may be disturbed for {} and {} natives.",
            sign, sign, opposite_sign
        )
    };
    
    VedicEclipseEffects {
        affected_signs: vec![sign.to_string(), opposite_sign.to_string()],
        affected_nakshatras: vec![nakshatra.to_string()],
        general_effects: general,
        recommendations: vec![
            "Avoid starting new ventures".to_string(),
            "Chant mantras for protection".to_string(),
            "Fast during eclipse period".to_string(),
            "Take bath after eclipse ends".to_string(),
            "Donate to charity".to_string(),
        ],
        avoid: vec![
            "Eating during eclipse".to_string(),
            "Starting important work".to_string(),
            "Traveling if possible".to_string(),
            "Sleeping during eclipse".to_string(),
        ],
        sutak_starts: None,
        sutak_ends: None,
    }
}

fn get_opposite_sign(sign: &str) -> String {
    match sign {
        "Aries" => "Libra",
        "Taurus" => "Scorpio",
        "Gemini" => "Sagittarius",
        "Cancer" => "Capricorn",
        "Leo" => "Aquarius",
        "Virgo" => "Pisces",
        "Libra" => "Aries",
        "Scorpio" => "Taurus",
        "Sagittarius" => "Gemini",
        "Capricorn" => "Cancer",
        "Aquarius" => "Leo",
        "Pisces" => "Virgo",
        _ => "Unknown",
    }.to_string()
}

/// Calculate local visibility
pub fn calculate_local_visibility(
    eclipse: &EclipseEvent,
    latitude: f64,
    longitude: f64,
) -> LocalEclipseVisibility {
    // Simplified visibility calculation
    // In production, this would use proper astronomical algorithms
    
    let is_visible = eclipse.visibility.iter().any(|v| {
        // Very simplified region check
        match v.as_str() {
            "North America" => latitude > 15.0 && latitude < 70.0 && longitude < -50.0 && longitude > -170.0,
            "South America" => latitude > -60.0 && latitude < 15.0 && longitude < -30.0 && longitude > -90.0,
            "Europe" => latitude > 35.0 && latitude < 70.0 && longitude > -10.0 && longitude < 40.0,
            "Africa" => latitude > -35.0 && latitude < 35.0 && longitude > -20.0 && longitude < 55.0,
            "Asia" => latitude > 5.0 && latitude < 70.0 && longitude > 40.0 && longitude < 150.0,
            "Americas" => longitude < -30.0 && longitude > -170.0,
            _ => false,
        }
    });
    
    LocalEclipseVisibility {
        is_visible,
        visibility_type: if is_visible { "Partial".to_string() } else { "Not visible".to_string() },
        start_time: if is_visible { Some(eclipse.maximum_time) } else { None },
        maximum_time: if is_visible { Some(eclipse.maximum_time) } else { None },
        end_time: if is_visible { Some(eclipse.maximum_time) } else { None },
        obscuration: if is_visible { Some(0.7) } else { None },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_eclipses() {
        let eclipses = get_eclipses_for_year(2024);
        assert!(!eclipses.is_empty());
        assert!(eclipses.iter().any(|e| matches!(e.eclipse_type, EclipseType::SolarTotal)));
    }

    #[test]
    fn test_opposite_sign() {
        assert_eq!(get_opposite_sign("Aries"), "Libra");
        assert_eq!(get_opposite_sign("Virgo"), "Pisces");
    }
}
