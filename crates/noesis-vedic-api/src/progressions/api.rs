//! Progressed chart calculations

use chrono::NaiveDate;

use crate::chart::{BirthChart, ZodiacSign};
use crate::error::VedicApiError;
use crate::progressions::types::{ProgressionChart, ProgressionMethod, ProgressedPlanet};

pub fn calculate_progression(
    chart: &BirthChart,
    target_date: &str,
    method: ProgressionMethod,
) -> Result<ProgressionChart, VedicApiError> {
    let birth_date = NaiveDate::parse_from_str(&chart.native.birth_date, "%Y-%m-%d")
        .map_err(|e| VedicApiError::InvalidInput {
            field: "birth_date".to_string(),
            message: e.to_string(),
        })?;

    let target_date = NaiveDate::parse_from_str(target_date, "%Y-%m-%d")
        .map_err(|e| VedicApiError::InvalidInput {
            field: "target_date".to_string(),
            message: e.to_string(),
        })?;

    let years = (target_date - birth_date).num_days() as f64 / 365.25;
    let arc = match method {
        ProgressionMethod::Secondary => years, // simplified
        ProgressionMethod::SolarArc => years,
    };

    let planets = chart
        .planets
        .iter()
        .map(|planet| {
            let progressed_longitude = (planet.full_longitude() + arc) % 360.0;
            let sign_index = (progressed_longitude / 30.0).floor() as usize;
            let sign = ZodiacSign::from_index(sign_index);
            let degree = progressed_longitude % 30.0;

            ProgressedPlanet {
                name: planet.name.clone(),
                longitude: progressed_longitude,
                sign,
                degree,
                is_retrograde: planet.is_retrograde,
            }
        })
        .collect();

    Ok(ProgressionChart {
        method,
        target_date: target_date.to_string(),
        planets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::{BirthChart, NativeInfo, PlanetPosition, HousePosition, HouseType, AscendantInfo, MoonInfo, SpecialPoints};

    fn sample_chart() -> BirthChart {
        BirthChart {
            native: NativeInfo {
                birth_date: "1991-08-13".to_string(),
                birth_time: "13:31".to_string(),
                latitude: 12.97,
                longitude: 77.59,
                timezone: 5.5,
            },
            ayanamsa: 24.0,
            house_system: "placidus".to_string(),
            planets: vec![PlanetPosition {
                name: "Sun".to_string(),
                longitude: 0.0,
                sign: ZodiacSign::Aries,
                degree: 0.0,
                minutes: 0.0,
                house: 1,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "".to_string(),
                pada: 1,
                speed: 0.0,
                latitude: 0.0,
            }],
            houses: vec![HousePosition {
                number: 1,
                sign: ZodiacSign::Aries,
                cusp: 0.0,
                degree: 0.0,
                house_type: HouseType::Dharma,
                is_kendra: true,
                is_panapara: false,
                is_apoklima: false,
            }],
            ascendant: AscendantInfo {
                sign: ZodiacSign::Aries,
                degree: 0.0,
                nakshatra: "".to_string(),
                pada: 1,
            },
            moon: MoonInfo {
                sign: ZodiacSign::Aries,
                degree: 0.0,
                nakshatra: "".to_string(),
                pada: 1,
                rashi_lord: "Mars".to_string(),
            },
            special_points: SpecialPoints {
                lagna: 0.0,
                midheaven: None,
                part_of_fortune: None,
            },
            notes: vec![],
        }
    }

    #[test]
    fn test_calculate_progression() {
        let chart = sample_chart();
        let result = calculate_progression(&chart, "2001-08-13", ProgressionMethod::SolarArc).unwrap();
        assert_eq!(result.planets.len(), 1);
    }
}
