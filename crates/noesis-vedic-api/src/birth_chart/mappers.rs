//! Birth Chart mappers
//!
//! FAPI-047: Map planet positions to internal model
//! FAPI-048: Map house cusps to internal model

use crate::error::VedicApiResult;
use super::api::{
    BirthChartApiResponse, PlanetApiResponse, HouseApiResponse,
    parse_planet, parse_sign,
};
use super::types::{BirthChart, PlanetPosition, HouseCusp, Planet, ZodiacSign, calculate_dignity};

/// Map complete birth chart API response to internal model
pub fn map_birth_chart_response(response: BirthChartApiResponse) -> VedicApiResult<BirthChart> {
    let planets = response.planets.iter()
        .filter_map(|p| map_planet_position(p).ok())
        .collect::<Vec<_>>();

    let houses = response.houses.iter()
        .map(map_house_cusp)
        .collect::<VedicApiResult<Vec<_>>>()?;

    let ascendant_sign = parse_sign(&response.ascendant.sign)?;
    
    // Extract signs before moving planets
    let moon_sign = planets.iter()
        .find(|p| p.planet == Planet::Moon)
        .map(|p| p.sign)
        .unwrap_or(ZodiacSign::Aries);
    let sun_sign = planets.iter()
        .find(|p| p.planet == Planet::Sun)
        .map(|p| p.sign)
        .unwrap_or(ZodiacSign::Aries);

    let (ayanamsa_name, ayanamsa_value) = if let Some(ref ayanamsa) = response.ayanamsa {
        (ayanamsa.name.clone(), ayanamsa.value)
    } else {
        ("Lahiri".to_string(), 23.85)
    };

    Ok(BirthChart {
        planets,
        houses,
        ascendant_sign,
        ascendant_degree: response.ascendant.degree,
        moon_sign,
        sun_sign,
        ayanamsa: ayanamsa_name,
        ayanamsa_value,
    })
}

/// Map planet position from API response
///
/// FAPI-047: Map planet positions to internal model
pub fn map_planet_position(planet: &PlanetApiResponse) -> VedicApiResult<PlanetPosition> {
    let planet_enum = parse_planet(&planet.name)?;
    let sign = parse_sign(&planet.sign)?;
    let is_retrograde = planet.is_retrograde.unwrap_or(false);
    
    // Calculate dignity
    let dignity = calculate_dignity(planet_enum, sign);

    Ok(PlanetPosition {
        planet: planet_enum,
        sign,
        degree: planet.degree,
        longitude: planet.longitude,
        house: planet.house,
        nakshatra: planet.nakshatra.clone().unwrap_or_else(|| "Unknown".to_string()),
        pada: planet.nakshatra_pada.unwrap_or(1),
        is_retrograde,
        is_combust: false, // Will be calculated separately
        dignity: Some(dignity),
    })
}

/// Map house cusp from API response
///
/// FAPI-048: Map house cusps to internal model
pub fn map_house_cusp(house: &HouseApiResponse) -> VedicApiResult<HouseCusp> {
    let sign = parse_sign(&house.sign)?;
    let lord = sign.ruler();

    Ok(HouseCusp {
        house: house.house,
        sign,
        degree: house.degree,
        lord,
    })
}

/// Map multiple planet positions
pub fn map_planet_positions(planets: &[PlanetApiResponse]) -> Vec<PlanetPosition> {
    planets.iter()
        .filter_map(|p| map_planet_position(p).ok())
        .collect()
}

/// Map multiple house cusps
pub fn map_house_cusps(houses: &[HouseApiResponse]) -> VedicApiResult<Vec<HouseCusp>> {
    houses.iter().map(map_house_cusp).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_planet_position() {
        let api_planet = PlanetApiResponse {
            name: "Sun".to_string(),
            sign: "Aries".to_string(),
            sign_num: 1,
            degree: 15.5,
            longitude: 15.5,
            house: 1,
            nakshatra: Some("Ashwini".to_string()),
            nakshatra_pada: Some(2),
            is_retrograde: Some(false),
            speed: Some(1.0),
        };

        let result = map_planet_position(&api_planet).unwrap();
        assert_eq!(result.planet, Planet::Sun);
        assert_eq!(result.sign, ZodiacSign::Aries);
        assert_eq!(result.degree, 15.5);
        assert!(!result.is_retrograde);
    }

    #[test]
    fn test_map_house_cusp() {
        let api_house = HouseApiResponse {
            house: 1,
            sign: "Taurus".to_string(),
            sign_num: 2,
            degree: 5.0,
        };

        let result = map_house_cusp(&api_house).unwrap();
        assert_eq!(result.house, 1);
        assert_eq!(result.sign, ZodiacSign::Taurus);
        assert_eq!(result.lord, Planet::Venus);
    }

    #[test]
    fn test_map_retrograde_planet() {
        let api_planet = PlanetApiResponse {
            name: "Saturn".to_string(),
            sign: "Aquarius".to_string(),
            sign_num: 11,
            degree: 20.0,
            longitude: 320.0,
            house: 10,
            nakshatra: None,
            nakshatra_pada: None,
            is_retrograde: Some(true),
            speed: Some(-0.5),
        };

        let result = map_planet_position(&api_planet).unwrap();
        assert!(result.is_retrograde);
        assert_eq!(result.planet, Planet::Saturn);
    }
}
