//! Birth Chart types
//!
//! FAPI-045: Define Birth Chart request/response types

use serde::{Deserialize, Serialize};

/// Zodiac signs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl ZodiacSign {
    /// Get sign from degree (0-359)
    pub fn from_degree(degree: f64) -> Self {
        let normalized = degree.rem_euclid(360.0);
        let sign_num = (normalized / 30.0) as u8;
        Self::from_number(sign_num + 1).unwrap_or(ZodiacSign::Aries)
    }

    /// Get sign from number (1-12)
    pub fn from_number(num: u8) -> Option<Self> {
        match num {
            1 => Some(ZodiacSign::Aries),
            2 => Some(ZodiacSign::Taurus),
            3 => Some(ZodiacSign::Gemini),
            4 => Some(ZodiacSign::Cancer),
            5 => Some(ZodiacSign::Leo),
            6 => Some(ZodiacSign::Virgo),
            7 => Some(ZodiacSign::Libra),
            8 => Some(ZodiacSign::Scorpio),
            9 => Some(ZodiacSign::Sagittarius),
            10 => Some(ZodiacSign::Capricorn),
            11 => Some(ZodiacSign::Aquarius),
            12 => Some(ZodiacSign::Pisces),
            _ => None,
        }
    }

    /// Get sign number (1-12)
    pub fn number(&self) -> u8 {
        match self {
            ZodiacSign::Aries => 1,
            ZodiacSign::Taurus => 2,
            ZodiacSign::Gemini => 3,
            ZodiacSign::Cancer => 4,
            ZodiacSign::Leo => 5,
            ZodiacSign::Virgo => 6,
            ZodiacSign::Libra => 7,
            ZodiacSign::Scorpio => 8,
            ZodiacSign::Sagittarius => 9,
            ZodiacSign::Capricorn => 10,
            ZodiacSign::Aquarius => 11,
            ZodiacSign::Pisces => 12,
        }
    }

    /// Get the ruling planet
    pub fn ruler(&self) -> Planet {
        match self {
            ZodiacSign::Aries => Planet::Mars,
            ZodiacSign::Taurus => Planet::Venus,
            ZodiacSign::Gemini => Planet::Mercury,
            ZodiacSign::Cancer => Planet::Moon,
            ZodiacSign::Leo => Planet::Sun,
            ZodiacSign::Virgo => Planet::Mercury,
            ZodiacSign::Libra => Planet::Venus,
            ZodiacSign::Scorpio => Planet::Mars,
            ZodiacSign::Sagittarius => Planet::Jupiter,
            ZodiacSign::Capricorn => Planet::Saturn,
            ZodiacSign::Aquarius => Planet::Saturn,
            ZodiacSign::Pisces => Planet::Jupiter,
        }
    }
}

impl std::fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZodiacSign::Aries => write!(f, "Aries"),
            ZodiacSign::Taurus => write!(f, "Taurus"),
            ZodiacSign::Gemini => write!(f, "Gemini"),
            ZodiacSign::Cancer => write!(f, "Cancer"),
            ZodiacSign::Leo => write!(f, "Leo"),
            ZodiacSign::Virgo => write!(f, "Virgo"),
            ZodiacSign::Libra => write!(f, "Libra"),
            ZodiacSign::Scorpio => write!(f, "Scorpio"),
            ZodiacSign::Sagittarius => write!(f, "Sagittarius"),
            ZodiacSign::Capricorn => write!(f, "Capricorn"),
            ZodiacSign::Aquarius => write!(f, "Aquarius"),
            ZodiacSign::Pisces => write!(f, "Pisces"),
        }
    }
}

/// Planets in Vedic astrology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Planet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
    Ascendant,
}

impl Planet {
    /// Get all classical planets (no nodes or ascendant)
    pub fn classical() -> [Planet; 7] {
        [
            Planet::Sun, Planet::Moon, Planet::Mars,
            Planet::Mercury, Planet::Jupiter, Planet::Venus, Planet::Saturn,
        ]
    }

    /// Get all planets including nodes
    pub fn all_grahas() -> [Planet; 9] {
        [
            Planet::Sun, Planet::Moon, Planet::Mars, Planet::Mercury,
            Planet::Jupiter, Planet::Venus, Planet::Saturn,
            Planet::Rahu, Planet::Ketu,
        ]
    }
}

impl std::fmt::Display for Planet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Planet::Sun => write!(f, "Sun"),
            Planet::Moon => write!(f, "Moon"),
            Planet::Mars => write!(f, "Mars"),
            Planet::Mercury => write!(f, "Mercury"),
            Planet::Jupiter => write!(f, "Jupiter"),
            Planet::Venus => write!(f, "Venus"),
            Planet::Saturn => write!(f, "Saturn"),
            Planet::Rahu => write!(f, "Rahu"),
            Planet::Ketu => write!(f, "Ketu"),
            Planet::Ascendant => write!(f, "Ascendant"),
        }
    }
}

/// Planetary dignity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dignity {
    Exalted,
    OwnSign,
    MoolaTrikona,
    Friendly,
    Neutral,
    Enemy,
    Debilitated,
}

impl std::fmt::Display for Dignity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dignity::Exalted => write!(f, "Exalted"),
            Dignity::OwnSign => write!(f, "Own Sign"),
            Dignity::MoolaTrikona => write!(f, "Moola Trikona"),
            Dignity::Friendly => write!(f, "Friendly"),
            Dignity::Neutral => write!(f, "Neutral"),
            Dignity::Enemy => write!(f, "Enemy"),
            Dignity::Debilitated => write!(f, "Debilitated"),
        }
    }
}

/// Calculate dignity for a planet in a sign
pub fn calculate_dignity(planet: Planet, sign: ZodiacSign) -> Dignity {
    // Exaltation
    let is_exalted = match planet {
        Planet::Sun => sign == ZodiacSign::Aries,
        Planet::Moon => sign == ZodiacSign::Taurus,
        Planet::Mars => sign == ZodiacSign::Capricorn,
        Planet::Mercury => sign == ZodiacSign::Virgo,
        Planet::Jupiter => sign == ZodiacSign::Cancer,
        Planet::Venus => sign == ZodiacSign::Pisces,
        Planet::Saturn => sign == ZodiacSign::Libra,
        _ => false,
    };
    if is_exalted {
        return Dignity::Exalted;
    }
    
    // Debilitation
    let is_debilitated = match planet {
        Planet::Sun => sign == ZodiacSign::Libra,
        Planet::Moon => sign == ZodiacSign::Scorpio,
        Planet::Mars => sign == ZodiacSign::Cancer,
        Planet::Mercury => sign == ZodiacSign::Pisces,
        Planet::Jupiter => sign == ZodiacSign::Capricorn,
        Planet::Venus => sign == ZodiacSign::Virgo,
        Planet::Saturn => sign == ZodiacSign::Aries,
        _ => false,
    };
    if is_debilitated {
        return Dignity::Debilitated;
    }
    
    // Own sign
    let is_own_sign = match planet {
        Planet::Sun => sign == ZodiacSign::Leo,
        Planet::Moon => sign == ZodiacSign::Cancer,
        Planet::Mars => sign == ZodiacSign::Aries || sign == ZodiacSign::Scorpio,
        Planet::Mercury => sign == ZodiacSign::Gemini || sign == ZodiacSign::Virgo,
        Planet::Jupiter => sign == ZodiacSign::Sagittarius || sign == ZodiacSign::Pisces,
        Planet::Venus => sign == ZodiacSign::Taurus || sign == ZodiacSign::Libra,
        Planet::Saturn => sign == ZodiacSign::Capricorn || sign == ZodiacSign::Aquarius,
        _ => false,
    };
    if is_own_sign {
        return Dignity::OwnSign;
    }
    
    Dignity::Neutral
}

/// Position of a planet in the chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetPosition {
    /// The planet
    pub planet: Planet,
    /// Zodiac sign
    pub sign: ZodiacSign,
    /// Degree within the sign (0-29.999)
    pub degree: f64,
    /// Total degree in zodiac (0-359.999)
    pub longitude: f64,
    /// House number (1-12)
    pub house: u8,
    /// Nakshatra
    pub nakshatra: String,
    /// Nakshatra pada (1-4)
    pub pada: u8,
    /// Whether retrograde
    pub is_retrograde: bool,
    /// Whether combust (too close to Sun)
    pub is_combust: bool,
    /// Dignity status
    pub dignity: Option<Dignity>,
}

/// House cusp position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseCusp {
    /// House number (1-12)
    pub house: u8,
    /// Sign on cusp
    pub sign: ZodiacSign,
    /// Degree of cusp
    pub degree: f64,
    /// Lord of this house
    pub lord: Planet,
}

/// Complete birth chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthChart {
    /// Planet positions
    pub planets: Vec<PlanetPosition>,
    /// House cusps
    pub houses: Vec<HouseCusp>,
    /// Ascendant sign
    pub ascendant_sign: ZodiacSign,
    /// Ascendant degree
    pub ascendant_degree: f64,
    /// Moon sign (Rashi)
    pub moon_sign: ZodiacSign,
    /// Sun sign
    pub sun_sign: ZodiacSign,
    /// Ayanamsa used
    pub ayanamsa: String,
    /// Ayanamsa value in degrees
    pub ayanamsa_value: f64,
}

impl BirthChart {
    /// Get planet position by planet
    pub fn get_planet(&self, planet: Planet) -> Option<&PlanetPosition> {
        self.planets.iter().find(|p| p.planet == planet)
    }

    /// Get house cusp by number
    pub fn get_house(&self, house: u8) -> Option<&HouseCusp> {
        self.houses.iter().find(|h| h.house == house)
    }

    /// Get all planets in a specific house
    pub fn planets_in_house(&self, house: u8) -> Vec<&PlanetPosition> {
        self.planets.iter().filter(|p| p.house == house).collect()
    }

    /// Get all planets in a specific sign
    pub fn planets_in_sign(&self, sign: ZodiacSign) -> Vec<&PlanetPosition> {
        self.planets.iter().filter(|p| p.sign == sign).collect()
    }

    /// Get retrograde planets
    pub fn retrograde_planets(&self) -> Vec<&PlanetPosition> {
        self.planets.iter().filter(|p| p.is_retrograde).collect()
    }

    /// Get combust planets
    pub fn combust_planets(&self) -> Vec<&PlanetPosition> {
        self.planets.iter().filter(|p| p.is_combust).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zodiac_from_degree() {
        assert_eq!(ZodiacSign::from_degree(0.0), ZodiacSign::Aries);
        assert_eq!(ZodiacSign::from_degree(45.0), ZodiacSign::Taurus);
        assert_eq!(ZodiacSign::from_degree(359.0), ZodiacSign::Pisces);
    }

    #[test]
    fn test_zodiac_ruler() {
        assert_eq!(ZodiacSign::Aries.ruler(), Planet::Mars);
        assert_eq!(ZodiacSign::Taurus.ruler(), Planet::Venus);
        assert_eq!(ZodiacSign::Leo.ruler(), Planet::Sun);
    }

    #[test]
    fn test_planet_arrays() {
        assert_eq!(Planet::classical().len(), 7);
        assert_eq!(Planet::all_grahas().len(), 9);
    }
    
    #[test]
    fn test_dignity_calculation() {
        assert_eq!(calculate_dignity(Planet::Sun, ZodiacSign::Aries), Dignity::Exalted);
        assert_eq!(calculate_dignity(Planet::Sun, ZodiacSign::Libra), Dignity::Debilitated);
        assert_eq!(calculate_dignity(Planet::Sun, ZodiacSign::Leo), Dignity::OwnSign);
    }
}
