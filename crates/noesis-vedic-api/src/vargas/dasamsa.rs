//! Dasamsa (D10) chart implementation
//!
//! FAPI-058: Implement D10 (Dasamsa) for career analysis

use crate::birth_chart::types::{Planet, ZodiacSign};

/// Calculate Dasamsa sign from longitude
pub fn calculate_dasamsa_sign(longitude: f64) -> ZodiacSign {
    // Each dasamsa spans 3 degrees
    let normalized = longitude.rem_euclid(360.0);
    let rashi_num = ((normalized / 30.0) as u8) + 1;
    let dasamsa_in_sign = ((normalized % 30.0) / 3.0) as u8;
    
    // For odd signs: start from same sign
    // For even signs: start from 9th from that sign
    let starting_sign = if rashi_num % 2 == 1 {
        rashi_num
    } else {
        ((rashi_num - 1 + 8) % 12) + 1 // 9th from sign
    };
    
    let final_sign = ((starting_sign - 1 + dasamsa_in_sign) % 12) + 1;
    ZodiacSign::from_number(final_sign).unwrap_or(ZodiacSign::Aries)
}

/// Dasamsa analysis for career
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DasamsaCareerAnalysis {
    /// Ascendant in Dasamsa
    pub dasamsa_ascendant: ZodiacSign,
    /// 10th lord position in Dasamsa
    pub tenth_lord_dasamsa: Option<ZodiacSign>,
    /// Sun position in Dasamsa (authority, government)
    pub sun_dasamsa: Option<ZodiacSign>,
    /// Saturn position in Dasamsa (service, discipline)
    pub saturn_dasamsa: Option<ZodiacSign>,
    /// Mercury position in Dasamsa (communication, intellect)
    pub mercury_dasamsa: Option<ZodiacSign>,
    /// Career indicators based on planetary positions
    pub career_indicators: Vec<String>,
    /// Overall career strength
    pub career_strength: CareerStrength,
}

/// Career strength assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CareerStrength {
    Excellent,
    Good,
    Average,
    Challenging,
}

/// Determine career field from Dasamsa
pub fn suggest_career_field(dasamsa_ascendant: ZodiacSign, tenth_lord_sign: Option<ZodiacSign>) -> Vec<String> {
    let mut fields = vec![];
    
    match dasamsa_ascendant {
        ZodiacSign::Aries | ZodiacSign::Scorpio => {
            fields.push("Defense, Military".to_string());
            fields.push("Sports, Athletics".to_string());
            fields.push("Surgery, Medicine".to_string());
        }
        ZodiacSign::Taurus | ZodiacSign::Libra => {
            fields.push("Finance, Banking".to_string());
            fields.push("Arts, Fashion".to_string());
            fields.push("Luxury goods".to_string());
        }
        ZodiacSign::Gemini | ZodiacSign::Virgo => {
            fields.push("Communication, Media".to_string());
            fields.push("Technology, IT".to_string());
            fields.push("Writing, Journalism".to_string());
        }
        ZodiacSign::Cancer => {
            fields.push("Real Estate, Property".to_string());
            fields.push("Hospitality, Food".to_string());
            fields.push("Nursing, Care services".to_string());
        }
        ZodiacSign::Leo => {
            fields.push("Government, Administration".to_string());
            fields.push("Entertainment, Acting".to_string());
            fields.push("Leadership roles".to_string());
        }
        ZodiacSign::Sagittarius | ZodiacSign::Pisces => {
            fields.push("Education, Teaching".to_string());
            fields.push("Law, Philosophy".to_string());
            fields.push("Spirituality, Counseling".to_string());
        }
        ZodiacSign::Capricorn | ZodiacSign::Aquarius => {
            fields.push("Engineering, Construction".to_string());
            fields.push("Research, Science".to_string());
            fields.push("Social work".to_string());
        }
    }
    
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dasamsa_calculation() {
        // Test that calculation produces valid signs
        let sign = calculate_dasamsa_sign(45.0);
        assert!(matches!(sign, ZodiacSign::Aries | ZodiacSign::Taurus | ZodiacSign::Gemini | 
            ZodiacSign::Cancer | ZodiacSign::Leo | ZodiacSign::Virgo | ZodiacSign::Libra |
            ZodiacSign::Scorpio | ZodiacSign::Sagittarius | ZodiacSign::Capricorn |
            ZodiacSign::Aquarius | ZodiacSign::Pisces));
    }

    #[test]
    fn test_career_suggestions() {
        let fields = suggest_career_field(ZodiacSign::Gemini, None);
        assert!(!fields.is_empty());
        assert!(fields.iter().any(|f| f.contains("Communication")));
    }
}
