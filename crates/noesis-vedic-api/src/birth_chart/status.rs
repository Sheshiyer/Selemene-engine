//! Retrograde and combustion status helpers

use serde::{Deserialize, Serialize};

use crate::chart::{BirthChart, PlanetPosition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetStatus {
    pub planet: String,
    pub is_retrograde: bool,
    pub is_combust: bool,
    pub combust_orb: Option<f64>,
}

pub fn compute_statuses(chart: &BirthChart) -> Vec<PlanetStatus> {
    let sun = chart.get_planet("Sun");

    chart
        .planets
        .iter()
        .map(|planet| PlanetStatus {
            planet: planet.name.clone(),
            is_retrograde: planet.is_retrograde,
            is_combust: sun.map(|s| is_combust(planet, s)).unwrap_or(false),
            combust_orb: sun.map(|s| combustion_orb(planet, s)),
        })
        .collect()
}

pub fn is_combust(planet: &PlanetPosition, sun: &PlanetPosition) -> bool {
    if planet.name.eq_ignore_ascii_case("Sun") {
        return false;
    }

    let orb = combustion_orb(planet, sun);
    let threshold = combust_threshold(&planet.name);
    orb <= threshold
}

pub fn combustion_orb(planet: &PlanetPosition, sun: &PlanetPosition) -> f64 {
    angular_distance(planet.full_longitude(), sun.full_longitude())
}

fn combust_threshold(planet_name: &str) -> f64 {
    match planet_name.to_lowercase().as_str() {
        "mercury" => 14.0,
        "venus" => 10.0,
        "mars" => 17.0,
        "jupiter" => 11.0,
        "saturn" => 15.0,
        "moon" => 12.0,
        _ => 8.0,
    }
}

fn angular_distance(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs() % 360.0;
    if diff > 180.0 { 360.0 - diff } else { diff }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::{PlanetPosition, ZodiacSign};

    fn planet(name: &str, sign: ZodiacSign, degree: f64) -> PlanetPosition {
        PlanetPosition {
            name: name.to_string(),
            longitude: 0.0,
            sign,
            degree,
            minutes: 0.0,
            house: 1,
            is_retrograde: false,
            is_combust: false,
            nakshatra: "".to_string(),
            pada: 1,
            speed: 0.0,
            latitude: 0.0,
        }
    }

    #[test]
    fn test_combustion_orb() {
        let sun = planet("Sun", ZodiacSign::Aries, 5.0);
        let mercury = planet("Mercury", ZodiacSign::Aries, 15.0);
        let orb = combustion_orb(&mercury, &sun);
        assert!(orb <= 10.0);
    }

    #[test]
    fn test_is_combust() {
        let sun = planet("Sun", ZodiacSign::Aries, 5.0);
        let venus = planet("Venus", ZodiacSign::Aries, 12.0);
        assert!(is_combust(&venus, &sun));
    }
}
