//! Sarva Ashtakavarga totals calculation
//!
//! FAPI-072: Calculate Sarva Ashtakavarga totals
//!
//! PR3 adds [`calculate_bhinna_ashtakavarga`] driven by the
//! `bindu_tables` matrices so we can compute per-planet BAV natively from
//! a `BirthChart` (no vendor POST).

use super::bindu_tables::{table_for, CONTRIBUTOR_NAMES};
use super::types::{
    AshtakavargaAnalysis, PlanetAshtakavarga, SarvaAshtakavarga, SignStrength, StrengthCategory,
};
use crate::birth_chart::types::{BirthChart, Planet};

/// Calculate Ashtakavarga analysis from Sarva
pub fn calculate_analysis(sarva: &SarvaAshtakavarga) -> AshtakavargaAnalysis {
    let sign_names = [
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpio",
        "Sagittarius",
        "Capricorn",
        "Aquarius",
        "Pisces",
    ];

    // Create sign strengths
    let mut sign_strengths: Vec<SignStrength> = sarva
        .sarva_points
        .iter()
        .enumerate()
        .map(|(i, &points)| SignStrength {
            sign: (i + 1) as u8,
            sign_name: sign_names[i].to_string(),
            points,
            category: StrengthCategory::from_sarva_points(points),
        })
        .collect();

    // Sort by points descending
    sign_strengths.sort_by_key(|s| std::cmp::Reverse(s.points));

    let strongest_signs: Vec<SignStrength> = sign_strengths.iter().take(3).cloned().collect();

    let weakest_signs: Vec<SignStrength> = sign_strengths.iter().rev().take(3).cloned().collect();

    // Generate transit recommendations
    let transit_recommendations =
        generate_transit_recommendations(&strongest_signs, &weakest_signs);

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

/// Bhinna-Ashtakavarga (BAV) — per-planet point distribution across the 12
/// zodiac signs, using the BPHS contribution tables in
/// [`super::bindu_tables`].
///
/// For each contributor (Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn,
/// Lagna) we look up its position in the chart, then for every position
/// `i` (1..=12 counted **from the contributor's sign**) flagged in the
/// table we award one bindu to the corresponding zodiac sign.
///
/// Returns a `PlanetAshtakavarga` with `sign_points[0]` for Aries through
/// `sign_points[11]` for Pisces. `total_points` is recalculated.
pub fn calculate_bhinna_ashtakavarga(planet: Planet, chart: &BirthChart) -> PlanetAshtakavarga {
    let mut av = PlanetAshtakavarga::empty(&planet.to_string());
    let Some(table) = table_for(planet) else {
        return av;
    };

    // Sign indices (0..=11) for each of the 8 contributors.
    // Order matches `CONTRIBUTOR_NAMES`: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Lagna.
    let contributor_planets = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mars,
        Planet::Mercury,
        Planet::Jupiter,
        Planet::Venus,
        Planet::Saturn,
    ];

    let mut contributor_sign_indices: [Option<u8>; 8] = [None; 8];
    for (i, &p) in contributor_planets.iter().enumerate() {
        if let Some(pos) = chart.get_planet(p) {
            contributor_sign_indices[i] = Some(pos.sign.number() - 1);
        }
    }
    // Lagna is the 8th contributor.
    contributor_sign_indices[7] = Some(chart.ascendant_sign.number() - 1);

    for (c_idx, contributor_sign) in contributor_sign_indices.iter().enumerate() {
        let Some(c_sign) = contributor_sign else {
            continue;
        };
        for (pos_idx, &has_bindu) in table[c_idx].iter().enumerate() {
            if !has_bindu {
                continue;
            }
            // Target sign = (contributor_sign + pos_idx) mod 12.
            let target = ((*c_sign as usize + pos_idx) % 12) as usize;
            // u8 saturating add — each cell can hold up to 8 (one per
            // contributor), so this never overflows the underlying u8.
            av.sign_points[target] = av.sign_points[target].saturating_add(1);
        }
    }

    av.recalculate_total();
    av
}

/// Kakshya breakdown — for a planet's BAV and a target sign, return the 8
/// contributor-wise binary indicators (in canonical order Sun, Moon, Mars,
/// Mercury, Jupiter, Venus, Saturn, Lagna). Drives the "which contributor
/// gave the bindu" question for transit interpretation.
pub fn calculate_kakshya_points_from_chart(
    planet: Planet,
    sign: u8,
    chart: &BirthChart,
) -> Vec<(String, bool)> {
    let Some(table) = table_for(planet) else {
        return Vec::new();
    };
    let contributor_planets = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mars,
        Planet::Mercury,
        Planet::Jupiter,
        Planet::Venus,
        Planet::Saturn,
    ];
    let mut out = Vec::with_capacity(8);
    for (c_idx, name) in CONTRIBUTOR_NAMES.iter().enumerate() {
        let c_sign_idx = if c_idx < 7 {
            chart
                .get_planet(contributor_planets[c_idx])
                .map(|p| p.sign.number() - 1)
        } else {
            Some(chart.ascendant_sign.number() - 1)
        };
        let has = match c_sign_idx {
            Some(idx) => {
                let offset = ((sign as i16 - 1) - idx as i16).rem_euclid(12) as usize;
                table[c_idx][offset]
            }
            None => false,
        };
        out.push((name.to_string(), has));
    }
    out
}

/// Legacy round-robin kakshya helper retained for source-compatibility.
/// Prefer [`calculate_kakshya_points_from_chart`] for native callers.
pub fn calculate_kakshya_points(planet_av: &[u8; 12], sign: u8) -> Vec<(String, bool)> {
    let contributors = [
        "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Lagna",
    ];
    let points = planet_av.get((sign - 1) as usize).copied().unwrap_or(0);

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
    sarva
        .planets
        .iter()
        .find(|p| p.planet.to_lowercase() == planet.to_lowercase())
        .map(|p| p.points_in_sign(sign))
        .unwrap_or(0)
}

/// Calculate Trikona (trine) reduction
pub fn calculate_trikona_reduction(sarva: &SarvaAshtakavarga) -> [u8; 12] {
    let mut reduced = [0u8; 12];

    // For each sign, subtract trikona signs (5th and 9th)
    #[allow(clippy::needless_range_loop)]
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
        assert_eq!(
            StrengthCategory::from_sarva_points(40),
            StrengthCategory::Excellent
        );
        assert_eq!(
            StrengthCategory::from_sarva_points(30),
            StrengthCategory::Average
        );
    }
}
