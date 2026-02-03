//! Navamsa (D9) chart implementation
//!
//! FAPI-054: Implement GET /navamsa-chart endpoint

use crate::birth_chart::types::{Planet, ZodiacSign};

/// Calculate Navamsa sign from longitude
pub fn calculate_navamsa_sign(longitude: f64) -> ZodiacSign {
    // Each navamsa spans 3°20' (3.333... degrees)
    let normalized = longitude.rem_euclid(360.0);
    let navamsa_num = ((normalized / 3.333333) as u8) % 12;
    
    // Navamsa starts from the sign's first navamsa
    // For fire signs (1,5,9): starts from Aries
    // For earth signs (2,6,10): starts from Capricorn
    // For air signs (3,7,11): starts from Libra
    // For water signs (4,8,12): starts from Cancer
    
    let rashi_num = ((normalized / 30.0) as u8) + 1;
    let navamsa_in_sign = ((normalized % 30.0) / 3.333333) as u8;
    
    let starting_sign = match rashi_num % 4 {
        1 => 1,  // Fire - Aries
        2 => 10, // Earth - Capricorn
        3 => 7,  // Air - Libra
        0 => 4,  // Water - Cancer
        _ => 1,
    };
    
    let final_sign = ((starting_sign - 1 + navamsa_in_sign) % 12) + 1;
    ZodiacSign::from_number(final_sign).unwrap_or(ZodiacSign::Aries)
}

/// Navamsa position for a planet
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NavamsaPlanetPosition {
    pub planet: Planet,
    pub rashi_sign: ZodiacSign,
    pub navamsa_sign: ZodiacSign,
    pub rashi_degree: f64,
    pub is_vargottama: bool,
}

impl NavamsaPlanetPosition {
    /// Create from rashi position
    pub fn from_rashi(planet: Planet, sign: ZodiacSign, degree: f64, longitude: f64) -> Self {
        let navamsa_sign = calculate_navamsa_sign(longitude);
        let is_vargottama = sign == navamsa_sign;
        
        Self {
            planet,
            rashi_sign: sign,
            navamsa_sign,
            rashi_degree: degree,
            is_vargottama,
        }
    }
}

/// Check if a planet is Vargottama (same sign in D1 and D9)
pub fn is_vargottama(rashi_sign: ZodiacSign, navamsa_sign: ZodiacSign) -> bool {
    rashi_sign == navamsa_sign
}

/// Analyze Navamsa for relationship insights
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NavamsaRelationshipAnalysis {
    /// Venus position in Navamsa
    pub venus_navamsa: Option<ZodiacSign>,
    /// 7th lord in Navamsa
    pub seventh_lord_navamsa: Option<ZodiacSign>,
    /// Planets in 7th house of Navamsa
    pub planets_in_7th_navamsa: Vec<Planet>,
    /// Is Venus vargottama
    pub venus_vargottama: bool,
    /// Overall relationship indicator
    pub relationship_strength: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navamsa_calculation() {
        // 0° Aries should give 1st navamsa of Aries = Aries
        assert_eq!(calculate_navamsa_sign(0.0), ZodiacSign::Aries);
        
        // 3.33° Aries should give 2nd navamsa = Taurus
        assert_eq!(calculate_navamsa_sign(3.5), ZodiacSign::Taurus);
    }

    #[test]
    fn test_vargottama() {
        assert!(is_vargottama(ZodiacSign::Aries, ZodiacSign::Aries));
        assert!(!is_vargottama(ZodiacSign::Aries, ZodiacSign::Taurus));
    }

    #[test]
    fn test_navamsa_planet_position() {
        let pos = NavamsaPlanetPosition::from_rashi(
            Planet::Sun,
            ZodiacSign::Aries,
            5.0,
            5.0,
        );
        
        assert_eq!(pos.planet, Planet::Sun);
        assert_eq!(pos.rashi_sign, ZodiacSign::Aries);
    }
}
