//! Ashtakavarga types
//!
//! FAPI-070: Define Ashtakavarga types

use serde::{Deserialize, Serialize};

/// Ashtakavarga for a single planet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetAshtakavarga {
    /// Planet name
    pub planet: String,
    /// Points in each sign (1-12)
    pub sign_points: [u8; 12],
    /// Total points (out of 8 per sign, max 56)
    pub total_points: u8,
}

impl PlanetAshtakavarga {
    /// Create empty Ashtakavarga
    pub fn empty(planet: &str) -> Self {
        Self {
            planet: planet.to_string(),
            sign_points: [0; 12],
            total_points: 0,
        }
    }

    /// Get points for a specific sign (1-indexed)
    pub fn points_in_sign(&self, sign: u8) -> u8 {
        if sign >= 1 && sign <= 12 {
            self.sign_points[(sign - 1) as usize]
        } else {
            0
        }
    }

    /// Recalculate total
    pub fn recalculate_total(&mut self) {
        self.total_points = self.sign_points.iter().sum();
    }
}

/// Sarva Ashtakavarga (combined for all planets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarvaAshtakavarga {
    /// Individual planet Ashtakavargas
    pub planets: Vec<PlanetAshtakavarga>,
    /// Combined points per sign
    pub sarva_points: [u8; 12],
    /// Grand total (max 337)
    pub grand_total: u16,
}

impl SarvaAshtakavarga {
    /// Create empty Sarva Ashtakavarga
    pub fn empty() -> Self {
        Self {
            planets: vec![],
            sarva_points: [0; 12],
            grand_total: 0,
        }
    }

    /// Get combined points for a sign
    pub fn points_in_sign(&self, sign: u8) -> u8 {
        if sign >= 1 && sign <= 12 {
            self.sarva_points[(sign - 1) as usize]
        } else {
            0
        }
    }

    /// Calculate Sarva from planet Ashtakavargas
    pub fn calculate_from_planets(&mut self) {
        self.sarva_points = [0; 12];
        self.grand_total = 0;
        
        for planet in &self.planets {
            for (i, points) in planet.sign_points.iter().enumerate() {
                self.sarva_points[i] += points;
            }
            self.grand_total += planet.total_points as u16;
        }
    }

    /// Add planet Ashtakavarga
    pub fn add_planet(&mut self, planet: PlanetAshtakavarga) {
        self.planets.push(planet);
        self.calculate_from_planets();
    }
}

/// Ashtakavarga analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshtakavargaAnalysis {
    /// Sarva Ashtakavarga
    pub sarva_ashtakavarga: SarvaAshtakavarga,
    /// Strongest signs (highest SAV points)
    pub strongest_signs: Vec<SignStrength>,
    /// Weakest signs
    pub weakest_signs: Vec<SignStrength>,
    /// Transit recommendations based on SAV
    pub transit_recommendations: Vec<String>,
}

/// Sign strength based on Ashtakavarga
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignStrength {
    /// Sign number (1-12)
    pub sign: u8,
    /// Sign name
    pub sign_name: String,
    /// Points
    pub points: u8,
    /// Strength category
    pub category: StrengthCategory,
}

/// Strength category for Ashtakavarga points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrengthCategory {
    /// 0-24 points - Very weak
    VeryWeak,
    /// 25-28 points - Weak
    Weak,
    /// 29-30 points - Average
    Average,
    /// 31-35 points - Good
    Good,
    /// 36+ points - Excellent
    Excellent,
}

impl StrengthCategory {
    pub fn from_sarva_points(points: u8) -> Self {
        if points >= 36 {
            StrengthCategory::Excellent
        } else if points >= 31 {
            StrengthCategory::Good
        } else if points >= 29 {
            StrengthCategory::Average
        } else if points >= 25 {
            StrengthCategory::Weak
        } else {
            StrengthCategory::VeryWeak
        }
    }
}

impl std::fmt::Display for StrengthCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrengthCategory::VeryWeak => write!(f, "Very Weak"),
            StrengthCategory::Weak => write!(f, "Weak"),
            StrengthCategory::Average => write!(f, "Average"),
            StrengthCategory::Good => write!(f, "Good"),
            StrengthCategory::Excellent => write!(f, "Excellent"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planet_ashtakavarga() {
        let mut av = PlanetAshtakavarga::empty("Sun");
        av.sign_points = [5, 6, 4, 3, 5, 4, 6, 5, 4, 5, 6, 3];
        av.recalculate_total();
        
        assert_eq!(av.total_points, 56);
        assert_eq!(av.points_in_sign(1), 5);
        assert_eq!(av.points_in_sign(2), 6);
    }

    #[test]
    fn test_sarva_ashtakavarga() {
        let mut sarva = SarvaAshtakavarga::empty();
        
        let mut sun_av = PlanetAshtakavarga::empty("Sun");
        sun_av.sign_points = [4; 12];
        sun_av.recalculate_total();
        
        sarva.add_planet(sun_av);
        
        assert_eq!(sarva.points_in_sign(1), 4);
        assert_eq!(sarva.grand_total, 48);
    }

    #[test]
    fn test_strength_category() {
        assert_eq!(StrengthCategory::from_sarva_points(40), StrengthCategory::Excellent);
        assert_eq!(StrengthCategory::from_sarva_points(32), StrengthCategory::Good);
        assert_eq!(StrengthCategory::from_sarva_points(20), StrengthCategory::VeryWeak);
    }
}
