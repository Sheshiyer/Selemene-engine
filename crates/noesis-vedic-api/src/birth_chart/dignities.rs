//! Planetary dignities for birth charts

use serde::{Deserialize, Serialize};

use crate::chart::{BirthChart, PlanetPosition, ZodiacSign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DignityStatus {
    Exalted,
    Debilitated,
    Moolatrikona,
    OwnSign,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetDignity {
    pub planet: String,
    pub sign: ZodiacSign,
    pub status: DignityStatus,
}

pub fn dignity_for(planet: &PlanetPosition) -> DignityStatus {
    if planet.is_exalted() {
        return DignityStatus::Exalted;
    }

    if planet.is_debilitated() {
        return DignityStatus::Debilitated;
    }

    if is_moolatrikona(planet) {
        return DignityStatus::Moolatrikona;
    }

    if planet.in_own_sign() {
        return DignityStatus::OwnSign;
    }

    DignityStatus::Neutral
}

pub fn chart_dignities(chart: &BirthChart) -> Vec<PlanetDignity> {
    chart
        .planets
        .iter()
        .map(|planet| PlanetDignity {
            planet: planet.name.clone(),
            sign: planet.sign,
            status: dignity_for(planet),
        })
        .collect()
}

fn is_moolatrikona(planet: &PlanetPosition) -> bool {
    let name = planet.name.to_lowercase();
    match name.as_str() {
        "sun" => planet.sign == ZodiacSign::Leo && planet.degree <= 20.0,
        "moon" => planet.sign == ZodiacSign::Taurus && planet.degree >= 4.0,
        "mars" => planet.sign == ZodiacSign::Aries && planet.degree <= 12.0,
        "mercury" => planet.sign == ZodiacSign::Virgo && planet.degree >= 16.0 && planet.degree <= 20.0,
        "jupiter" => planet.sign == ZodiacSign::Sagittarius && planet.degree <= 10.0,
        "venus" => planet.sign == ZodiacSign::Libra && planet.degree <= 15.0,
        "saturn" => planet.sign == ZodiacSign::Aquarius && planet.degree <= 20.0,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_dignity_exalted() {
        let sun = planet("Sun", ZodiacSign::Aries, 10.0);
        assert_eq!(dignity_for(&sun), DignityStatus::Exalted);
    }

    #[test]
    fn test_dignity_moolatrikona() {
        let sun = planet("Sun", ZodiacSign::Leo, 12.0);
        assert_eq!(dignity_for(&sun), DignityStatus::Moolatrikona);
    }

    #[test]
    fn test_dignity_own_sign() {
        let venus = planet("Venus", ZodiacSign::Taurus, 20.0);
        assert_eq!(dignity_for(&venus), DignityStatus::OwnSign);
    }
}
