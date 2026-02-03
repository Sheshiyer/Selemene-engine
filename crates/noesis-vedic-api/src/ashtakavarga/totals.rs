//! Sarva Ashtakavarga totals calculation
//!
//! FAPI-072: Calculate Sarva Ashtakavarga totals

use super::types::{SarvaAshtakavarga, AshtakavargaAnalysis, SignStrength, StrengthCategory};

/// Calculate Ashtakavarga analysis from Sarva
pub fn calculate_analysis(sarva: &SarvaAshtakavarga) -> AshtakavargaAnalysis {
    let sign_names = [
        "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo",
        "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces"
    ];
    
    // Create sign strengths
    let mut sign_strengths: Vec<SignStrength> = sarva.sarva_points.iter()
        .enumerate()
        .map(|(i, &points)| SignStrength {
            sign: (i + 1) as u8,
            sign_name: sign_names[i].to_string(),
            points,
            category: StrengthCategory::from_sarva_points(points),
        })
        .collect();
    
    // Sort by points descending
    sign_strengths.sort_by(|a, b| b.points.cmp(&a.points));
    
    let strongest_signs: Vec<SignStrength> = sign_strengths.iter()
        .take(3)
        .cloned()
        .collect();
    
    let weakest_signs: Vec<SignStrength> = sign_strengths.iter()
        .rev()
        .take(3)
        .cloned()
        .collect();
    
    // Generate transit recommendations
    let transit_recommendations = generate_transit_recommendations(&strongest_signs, &weakest_signs);
    
    AshtakavargaAnalysis {
        sarva_ashtakavarga: sarva.clone(),
        strongest_signs,
        weakest_signs,
        transit_recommendations,
    }
}

/// Generate transit recommendations based on SAV
fn generate_transit_recommendations(
    strongest: &[SignStrength],
    weakest: &[SignStrength],
) -> Vec<String> {
    let mut recommendations = vec![];
    
    // Recommendations for strongest signs
    for sign in strongest {
        recommendations.push(format!(
            "Transits through {} ({} points) are generally favorable for new initiatives",
            sign.sign_name, sign.points
        ));
    }
    
    // Warnings for weakest signs
    for sign in weakest {
        if sign.points < 25 {
            recommendations.push(format!(
                "Exercise caution during transits through {} ({} points) - may face obstacles",
                sign.sign_name, sign.points
            ));
        }
    }
    
    // General recommendations based on grand total
    recommendations
}

/// Calculate Kakshya-wise points for detailed analysis
pub fn calculate_kakshya_points(planet_av: &[u8; 12], sign: u8) -> Vec<(String, bool)> {
    let contributors = ["Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Lagna"];
    let points = planet_av.get((sign - 1) as usize).copied().unwrap_or(0);
    
    // This is simplified - actual calculation needs the full Ashtakavarga matrix
    let mut kakshyas = vec![];
    let mut remaining = points;
    
    for contributor in contributors {
        let has_point = remaining > 0;
        if has_point {
            remaining -= 1;
        }
        kakshyas.push((contributor.to_string(), has_point));
    }
    
    kakshyas
}

/// Get benefic bindus (points) for a specific planet in a sign
pub fn get_benefic_bindus(sarva: &SarvaAshtakavarga, planet: &str, sign: u8) -> u8 {
    sarva.planets.iter()
        .find(|p| p.planet.to_lowercase() == planet.to_lowercase())
        .map(|p| p.points_in_sign(sign))
        .unwrap_or(0)
}

/// Calculate Trikona (trine) reduction
pub fn calculate_trikona_reduction(sarva: &SarvaAshtakavarga) -> [u8; 12] {
    let mut reduced = [0u8; 12];
    
    // For each sign, subtract trikona signs (5th and 9th)
    for i in 0..12 {
        let fifth = (i + 4) % 12;
        let ninth = (i + 8) % 12;
        
        let min = sarva.sarva_points[i]
            .min(sarva.sarva_points[fifth])
            .min(sarva.sarva_points[ninth]);
        
        reduced[i] = sarva.sarva_points[i].saturating_sub(min);
    }
    
    reduced
}

/// Calculate Ekadhipatya (single lordship) reduction
pub fn calculate_ekadhipatya_reduction(sarva: &SarvaAshtakavarga) -> [u8; 12] {
    let mut reduced = sarva.sarva_points;
    
    // Mars rules Aries (0) and Scorpio (7)
    let min_mars = reduced[0].min(reduced[7]);
    reduced[0] = min_mars;
    reduced[7] = 0;
    
    // Venus rules Taurus (1) and Libra (6)
    let min_venus = reduced[1].min(reduced[6]);
    reduced[1] = min_venus;
    reduced[6] = 0;
    
    // Mercury rules Gemini (2) and Virgo (5)
    let min_mercury = reduced[2].min(reduced[5]);
    reduced[2] = min_mercury;
    reduced[5] = 0;
    
    // Jupiter rules Sagittarius (8) and Pisces (11)
    let min_jupiter = reduced[8].min(reduced[11]);
    reduced[8] = min_jupiter;
    reduced[11] = 0;
    
    // Saturn rules Capricorn (9) and Aquarius (10)
    let min_saturn = reduced[9].min(reduced[10]);
    reduced[9] = min_saturn;
    reduced[10] = 0;
    
    reduced
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ashtakavarga::types::PlanetAshtakavarga;

    #[test]
    fn test_calculate_analysis() {
        let mut sarva = SarvaAshtakavarga::empty();
        sarva.sarva_points = [28, 32, 25, 30, 35, 27, 29, 31, 33, 26, 28, 24];
        sarva.grand_total = 348;
        
        let analysis = calculate_analysis(&sarva);
        
        assert!(!analysis.strongest_signs.is_empty());
        assert!(!analysis.weakest_signs.is_empty());
        assert!(analysis.strongest_signs[0].points >= analysis.weakest_signs[0].points);
    }

    #[test]
    fn test_strength_categorization() {
        assert_eq!(StrengthCategory::from_sarva_points(40), StrengthCategory::Excellent);
        assert_eq!(StrengthCategory::from_sarva_points(30), StrengthCategory::Average);
    }
}
