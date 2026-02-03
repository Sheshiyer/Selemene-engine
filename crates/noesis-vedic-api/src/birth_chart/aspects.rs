//! Aspect calculations between planets

use serde::{Deserialize, Serialize};

use crate::chart::{BirthChart, PlanetPosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AspectType {
    Conjunction,
    Opposition,
    Trine,
    Square,
    Sextile,
}

impl AspectType {
    pub fn angle(&self) -> f64 {
        match self {
            AspectType::Conjunction => 0.0,
            AspectType::Opposition => 180.0,
            AspectType::Trine => 120.0,
            AspectType::Square => 90.0,
            AspectType::Sextile => 60.0,
        }
    }

    pub fn orb(&self) -> f64 {
        match self {
            AspectType::Conjunction => 8.0,
            AspectType::Opposition => 8.0,
            AspectType::Trine => 7.0,
            AspectType::Square => 6.0,
            AspectType::Sextile => 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aspect {
    pub planet_a: String,
    pub planet_b: String,
    pub aspect_type: AspectType,
    pub orb: f64,
    pub exact_angle: f64,
}

pub fn calculate_aspects(chart: &BirthChart) -> Vec<Aspect> {
    let mut aspects = Vec::new();

    for i in 0..chart.planets.len() {
        for j in (i + 1)..chart.planets.len() {
            let a = &chart.planets[i];
            let b = &chart.planets[j];
            if let Some(aspect) = aspect_between(a, b) {
                aspects.push(aspect);
            }
        }
    }

    aspects
}

fn aspect_between(a: &PlanetPosition, b: &PlanetPosition) -> Option<Aspect> {
    let angle = angular_distance(a.full_longitude(), b.full_longitude());

    for aspect_type in [
        AspectType::Conjunction,
        AspectType::Opposition,
        AspectType::Trine,
        AspectType::Square,
        AspectType::Sextile,
    ] {
        let exact = aspect_type.angle();
        let orb = (angle - exact).abs();
        if orb <= aspect_type.orb() {
            return Some(Aspect {
                planet_a: a.name.clone(),
                planet_b: b.name.clone(),
                aspect_type,
                orb,
                exact_angle: exact,
            });
        }
    }

    None
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
    fn test_aspect_between_trine() {
        let a = planet("Sun", ZodiacSign::Aries, 0.0);
        let b = planet("Moon", ZodiacSign::Leo, 0.0); // 120 degrees
        let aspect = aspect_between(&a, &b).expect("aspect");
        assert_eq!(aspect.aspect_type, AspectType::Trine);
    }
}
