//! Planetary position calculation and dignity for Vedic biofield analysis
//!
//! Reuses `EphemerisCalculator` from engine-human-design for Swiss Ephemeris access.
//! Implements simplified Shadbala-inspired strength scoring.

use chrono::{DateTime, Utc};
use engine_human_design::{EphemerisCalculator, HDPlanet};
use noesis_core::EngineError;

/// Vedic planets used in biofield analysis (Navagraha)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VedicPlanet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

impl VedicPlanet {
    pub fn all() -> &'static [VedicPlanet] {
        &[
            Self::Sun,
            Self::Moon,
            Self::Mars,
            Self::Mercury,
            Self::Jupiter,
            Self::Venus,
            Self::Saturn,
            Self::Rahu,
            Self::Ketu,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sun => "Sun",
            Self::Moon => "Moon",
            Self::Mars => "Mars",
            Self::Mercury => "Mercury",
            Self::Jupiter => "Jupiter",
            Self::Venus => "Venus",
            Self::Saturn => "Saturn",
            Self::Rahu => "Rahu",
            Self::Ketu => "Ketu",
        }
    }

    /// Whether this planet is naturally benefic in Vedic tradition
    pub fn is_benefic(&self) -> bool {
        matches!(self, Self::Jupiter | Self::Venus | Self::Moon | Self::Mercury)
    }

    /// Whether this planet is naturally malefic in Vedic tradition
    pub fn is_malefic(&self) -> bool {
        matches!(
            self,
            Self::Saturn | Self::Mars | Self::Rahu | Self::Ketu | Self::Sun
        )
    }
}

impl std::fmt::Display for VedicPlanet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Zodiac signs for Vedic calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn from_longitude(longitude: f64) -> Self {
        let normalized = ((longitude % 360.0) + 360.0) % 360.0;
        match (normalized / 30.0) as u8 {
            0 => Self::Aries,
            1 => Self::Taurus,
            2 => Self::Gemini,
            3 => Self::Cancer,
            4 => Self::Leo,
            5 => Self::Virgo,
            6 => Self::Libra,
            7 => Self::Scorpio,
            8 => Self::Sagittarius,
            9 => Self::Capricorn,
            10 => Self::Aquarius,
            _ => Self::Pisces,
        }
    }

    pub fn degree_in_sign(longitude: f64) -> f64 {
        let normalized = ((longitude % 360.0) + 360.0) % 360.0;
        normalized % 30.0
    }

    pub fn index(&self) -> u8 {
        match self {
            Self::Aries => 0,
            Self::Taurus => 1,
            Self::Gemini => 2,
            Self::Cancer => 3,
            Self::Leo => 4,
            Self::Virgo => 5,
            Self::Libra => 6,
            Self::Scorpio => 7,
            Self::Sagittarius => 8,
            Self::Capricorn => 9,
            Self::Aquarius => 10,
            Self::Pisces => 11,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Aries => "Aries",
            Self::Taurus => "Taurus",
            Self::Gemini => "Gemini",
            Self::Cancer => "Cancer",
            Self::Leo => "Leo",
            Self::Virgo => "Virgo",
            Self::Libra => "Libra",
            Self::Scorpio => "Scorpio",
            Self::Sagittarius => "Sagittarius",
            Self::Capricorn => "Capricorn",
            Self::Aquarius => "Aquarius",
            Self::Pisces => "Pisces",
        }
    }

    /// Lord (ruler) of this sign
    pub fn lord(&self) -> VedicPlanet {
        match self {
            Self::Aries | Self::Scorpio => VedicPlanet::Mars,
            Self::Taurus | Self::Libra => VedicPlanet::Venus,
            Self::Gemini | Self::Virgo => VedicPlanet::Mercury,
            Self::Cancer => VedicPlanet::Moon,
            Self::Leo => VedicPlanet::Sun,
            Self::Sagittarius | Self::Pisces => VedicPlanet::Jupiter,
            Self::Capricorn | Self::Aquarius => VedicPlanet::Saturn,
        }
    }
}

impl std::fmt::Display for ZodiacSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Position of a planet at a specific time with Vedic metadata
#[derive(Debug, Clone)]
pub struct VedicPosition {
    pub planet: VedicPlanet,
    pub longitude: f64,
    pub latitude: f64,
    pub speed: f64,
    pub sign: ZodiacSign,
    pub degree_in_sign: f64,
    pub is_retrograde: bool,
}

/// Map VedicPlanet to engine-human-design's HDPlanet
fn to_hd_planet(planet: VedicPlanet) -> HDPlanet {
    match planet {
        VedicPlanet::Sun => HDPlanet::Sun,
        VedicPlanet::Moon => HDPlanet::Moon,
        VedicPlanet::Mars => HDPlanet::Mars,
        VedicPlanet::Mercury => HDPlanet::Mercury,
        VedicPlanet::Jupiter => HDPlanet::Jupiter,
        VedicPlanet::Venus => HDPlanet::Venus,
        VedicPlanet::Saturn => HDPlanet::Saturn,
        VedicPlanet::Rahu => HDPlanet::NorthNode,
        VedicPlanet::Ketu => HDPlanet::SouthNode,
    }
}

/// Calculate position for a single Vedic planet
pub fn calculate_position(
    calculator: &EphemerisCalculator,
    planet: VedicPlanet,
    datetime: &DateTime<Utc>,
) -> Result<VedicPosition, EngineError> {
    let hd_planet = to_hd_planet(planet);
    let pos = calculator.get_planet_position(hd_planet, datetime)?;

    let sign = ZodiacSign::from_longitude(pos.longitude);
    let degree_in_sign = ZodiacSign::degree_in_sign(pos.longitude);
    let is_retrograde = pos.speed < 0.0;

    Ok(VedicPosition {
        planet,
        longitude: pos.longitude,
        latitude: pos.latitude,
        speed: pos.speed,
        sign,
        degree_in_sign,
        is_retrograde,
    })
}

/// Calculate positions for all 9 Vedic planets
pub fn calculate_all_positions(
    calculator: &EphemerisCalculator,
    datetime: &DateTime<Utc>,
) -> Result<Vec<VedicPosition>, EngineError> {
    VedicPlanet::all()
        .iter()
        .map(|&planet| calculate_position(calculator, planet, datetime))
        .collect()
}

// ── Aspects ──────────────────────────────────────────────────────────

/// Aspect types used in biofield analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Self::Conjunction => 0.0,
            Self::Opposition => 180.0,
            Self::Trine => 120.0,
            Self::Square => 90.0,
            Self::Sextile => 60.0,
        }
    }

    pub fn default_orb(&self) -> f64 {
        match self {
            Self::Conjunction => 8.0,
            Self::Opposition => 8.0,
            Self::Trine => 6.0,
            Self::Square => 6.0,
            Self::Sextile => 4.0,
        }
    }

    pub fn all() -> &'static [AspectType] {
        &[
            Self::Conjunction,
            Self::Opposition,
            Self::Trine,
            Self::Square,
            Self::Sextile,
        ]
    }

    pub fn is_harmonious(&self) -> bool {
        matches!(self, Self::Trine | Self::Sextile)
    }

    pub fn is_challenging(&self) -> bool {
        matches!(self, Self::Square | Self::Opposition)
    }
}

/// An aspect between two natal planets
#[derive(Debug, Clone)]
pub struct PlanetAspect {
    pub planet_a: VedicPlanet,
    pub planet_b: VedicPlanet,
    pub aspect_type: AspectType,
    pub orb: f64,
}

/// Shortest angular difference between two ecliptic angles
pub fn angular_diff(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs();
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Detect all aspects between natal planet positions
pub fn calculate_natal_aspects(positions: &[VedicPosition]) -> Vec<PlanetAspect> {
    let mut aspects = Vec::new();

    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let separation = angular_diff(positions[i].longitude, positions[j].longitude);
            for &aspect_type in AspectType::all() {
                let orb = (separation - aspect_type.angle()).abs();
                if orb <= aspect_type.default_orb() {
                    aspects.push(PlanetAspect {
                        planet_a: positions[i].planet,
                        planet_b: positions[j].planet,
                        aspect_type,
                        orb,
                    });
                }
            }
        }
    }

    aspects
}

// ── Dignity ──────────────────────────────────────────────────────────

/// Planet dignity in a sign: +3 exalted, +2 own sign, +1 friendly, 0 neutral, -1 enemy, -2 debilitated
pub fn planet_dignity(planet: VedicPlanet, sign: ZodiacSign) -> i8 {
    if is_exalted(planet, sign) {
        return 3;
    }
    if is_debilitated(planet, sign) {
        return -2;
    }
    if is_own_sign(planet, sign) {
        return 2;
    }
    natural_friendship(planet, sign.lord())
}

fn is_exalted(planet: VedicPlanet, sign: ZodiacSign) -> bool {
    matches!(
        (planet, sign),
        (VedicPlanet::Sun, ZodiacSign::Aries)
            | (VedicPlanet::Moon, ZodiacSign::Taurus)
            | (VedicPlanet::Mars, ZodiacSign::Capricorn)
            | (VedicPlanet::Mercury, ZodiacSign::Virgo)
            | (VedicPlanet::Jupiter, ZodiacSign::Cancer)
            | (VedicPlanet::Venus, ZodiacSign::Pisces)
            | (VedicPlanet::Saturn, ZodiacSign::Libra)
            | (VedicPlanet::Rahu, ZodiacSign::Taurus)
            | (VedicPlanet::Ketu, ZodiacSign::Scorpio)
    )
}

fn is_debilitated(planet: VedicPlanet, sign: ZodiacSign) -> bool {
    matches!(
        (planet, sign),
        (VedicPlanet::Sun, ZodiacSign::Libra)
            | (VedicPlanet::Moon, ZodiacSign::Scorpio)
            | (VedicPlanet::Mars, ZodiacSign::Cancer)
            | (VedicPlanet::Mercury, ZodiacSign::Pisces)
            | (VedicPlanet::Jupiter, ZodiacSign::Capricorn)
            | (VedicPlanet::Venus, ZodiacSign::Virgo)
            | (VedicPlanet::Saturn, ZodiacSign::Aries)
            | (VedicPlanet::Rahu, ZodiacSign::Scorpio)
            | (VedicPlanet::Ketu, ZodiacSign::Taurus)
    )
}

fn is_own_sign(planet: VedicPlanet, sign: ZodiacSign) -> bool {
    matches!(
        (planet, sign),
        (VedicPlanet::Sun, ZodiacSign::Leo)
            | (VedicPlanet::Moon, ZodiacSign::Cancer)
            | (VedicPlanet::Mars, ZodiacSign::Aries)
            | (VedicPlanet::Mars, ZodiacSign::Scorpio)
            | (VedicPlanet::Mercury, ZodiacSign::Gemini)
            | (VedicPlanet::Mercury, ZodiacSign::Virgo) // also exaltation; exaltation checked first
            | (VedicPlanet::Jupiter, ZodiacSign::Sagittarius)
            | (VedicPlanet::Jupiter, ZodiacSign::Pisces)
            | (VedicPlanet::Venus, ZodiacSign::Taurus)
            | (VedicPlanet::Venus, ZodiacSign::Libra)
            | (VedicPlanet::Saturn, ZodiacSign::Capricorn)
            | (VedicPlanet::Saturn, ZodiacSign::Aquarius)
            | (VedicPlanet::Rahu, ZodiacSign::Aquarius)
            | (VedicPlanet::Ketu, ZodiacSign::Scorpio) // also exaltation; exaltation checked first
    )
}

/// Naisargika (natural) friendship: +1 friend, 0 neutral, -1 enemy
fn natural_friendship(planet: VedicPlanet, lord: VedicPlanet) -> i8 {
    use VedicPlanet::*;

    if planet == lord {
        return 2; // own sign — shouldn't reach here since is_own_sign checked first
    }

    match (planet, lord) {
        // Sun: friends with Moon, Mars, Jupiter
        (Sun, Moon) | (Sun, Mars) | (Sun, Jupiter) => 1,
        (Sun, Venus) | (Sun, Saturn) => -1,

        // Moon: friends with Sun, Mercury
        (Moon, Sun) | (Moon, Mercury) => 1,
        (Moon, _) => 0, // Moon has no natural enemies

        // Mars: friends with Sun, Moon, Jupiter
        (Mars, Sun) | (Mars, Moon) | (Mars, Jupiter) => 1,
        (Mars, Mercury) => -1,

        // Mercury: friends with Sun, Venus
        (Mercury, Sun) | (Mercury, Venus) => 1,
        (Mercury, Moon) => -1,

        // Jupiter: friends with Sun, Moon, Mars
        (Jupiter, Sun) | (Jupiter, Moon) | (Jupiter, Mars) => 1,
        (Jupiter, Mercury) | (Jupiter, Venus) => -1,

        // Venus: friends with Mercury, Saturn
        (Venus, Mercury) | (Venus, Saturn) => 1,
        (Venus, Sun) | (Venus, Moon) => -1,

        // Saturn: friends with Mercury, Venus
        (Saturn, Mercury) | (Saturn, Venus) => 1,
        (Saturn, Sun) | (Saturn, Moon) | (Saturn, Mars) => -1,

        // Rahu acts like Saturn
        (Rahu, Mercury) | (Rahu, Venus) | (Rahu, Saturn) => 1,
        (Rahu, Sun) | (Rahu, Moon) | (Rahu, Mars) => -1,

        // Ketu acts like Mars
        (Ketu, Sun) | (Ketu, Moon) | (Ketu, Jupiter) => 1,
        (Ketu, Mercury) | (Ketu, Venus) => -1,

        _ => 0,
    }
}

// ── Planet Strength ──────────────────────────────────────────────────

/// House strength: angular (+2), succedent (+1), cadent (0)
pub fn house_strength(sign: ZodiacSign) -> i8 {
    match sign.index() + 1 {
        1 | 4 | 7 | 10 => 2,
        2 | 5 | 8 | 11 => 1,
        _ => 0,
    }
}

/// Aggregate planet strength score
#[derive(Debug, Clone)]
pub struct PlanetStrength {
    pub planet: VedicPlanet,
    pub dignity_score: i8,
    pub aspect_score: f64,
    pub retrograde_bonus: f64,
    pub house_strength_score: i8,
    /// Normalized to 0.0-1.0
    pub total: f64,
}

/// Calculate strength for all planets from positions and aspects
pub fn calculate_planet_strengths(
    positions: &[VedicPosition],
    aspects: &[PlanetAspect],
) -> Vec<PlanetStrength> {
    positions
        .iter()
        .map(|pos| {
            let dignity = planet_dignity(pos.planet, pos.sign);

            // Aspect score: benefic contacts positive, malefic contacts negative
            let aspect_score: f64 = aspects
                .iter()
                .filter(|a| a.planet_a == pos.planet || a.planet_b == pos.planet)
                .map(|a| {
                    let other = if a.planet_a == pos.planet {
                        a.planet_b
                    } else {
                        a.planet_a
                    };
                    let is_benefic_contact = other.is_benefic();

                    if a.aspect_type.is_harmonious() {
                        if is_benefic_contact {
                            2.0
                        } else {
                            1.0
                        }
                    } else if a.aspect_type.is_challenging() {
                        if other.is_malefic() {
                            -2.0
                        } else {
                            -1.0
                        }
                    } else {
                        // Conjunction: depends on the other planet
                        if is_benefic_contact {
                            1.0
                        } else {
                            -0.5
                        }
                    }
                })
                .sum();

            let retrograde_bonus = if pos.is_retrograde { 1.0 } else { 0.0 };
            let h_strength = house_strength(pos.sign);

            // Raw score: dignity (-2..+3) + aspect_score + retrograde (0..1) + house (0..2)
            // Normalize from roughly [-6, +10] to [0.0, 1.0]
            let raw = (dignity as f64) + aspect_score + retrograde_bonus + (h_strength as f64);
            let total = ((raw + 6.0) / 16.0).clamp(0.0, 1.0);

            PlanetStrength {
                planet: pos.planet,
                dignity_score: dignity,
                aspect_score,
                retrograde_bonus,
                house_strength_score: h_strength,
                total,
            }
        })
        .collect()
}

/// Look up a planet's strength by name
pub fn get_planet_strength(strengths: &[PlanetStrength], planet: VedicPlanet) -> f64 {
    strengths
        .iter()
        .find(|s| s.planet == planet)
        .map(|s| s.total)
        .unwrap_or(0.375) // neutral default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vedic_planet_all() {
        assert_eq!(VedicPlanet::all().len(), 9);
    }

    #[test]
    fn test_zodiac_from_longitude() {
        assert_eq!(ZodiacSign::from_longitude(0.0), ZodiacSign::Aries);
        assert_eq!(ZodiacSign::from_longitude(45.0), ZodiacSign::Taurus);
        assert_eq!(ZodiacSign::from_longitude(90.0), ZodiacSign::Cancer);
        assert_eq!(ZodiacSign::from_longitude(270.0), ZodiacSign::Capricorn);
        assert_eq!(ZodiacSign::from_longitude(359.9), ZodiacSign::Pisces);
    }

    #[test]
    fn test_sign_lord() {
        assert_eq!(ZodiacSign::Aries.lord(), VedicPlanet::Mars);
        assert_eq!(ZodiacSign::Cancer.lord(), VedicPlanet::Moon);
        assert_eq!(ZodiacSign::Leo.lord(), VedicPlanet::Sun);
        assert_eq!(ZodiacSign::Pisces.lord(), VedicPlanet::Jupiter);
    }

    #[test]
    fn test_dignity_exaltation() {
        assert_eq!(planet_dignity(VedicPlanet::Sun, ZodiacSign::Aries), 3);
        assert_eq!(planet_dignity(VedicPlanet::Moon, ZodiacSign::Taurus), 3);
        assert_eq!(planet_dignity(VedicPlanet::Jupiter, ZodiacSign::Cancer), 3);
    }

    #[test]
    fn test_dignity_debilitation() {
        assert_eq!(planet_dignity(VedicPlanet::Sun, ZodiacSign::Libra), -2);
        assert_eq!(planet_dignity(VedicPlanet::Moon, ZodiacSign::Scorpio), -2);
        assert_eq!(planet_dignity(VedicPlanet::Saturn, ZodiacSign::Aries), -2);
    }

    #[test]
    fn test_dignity_own_sign() {
        assert_eq!(planet_dignity(VedicPlanet::Sun, ZodiacSign::Leo), 2);
        assert_eq!(planet_dignity(VedicPlanet::Mars, ZodiacSign::Aries), 2);
        assert_eq!(planet_dignity(VedicPlanet::Venus, ZodiacSign::Taurus), 2);
    }

    #[test]
    fn test_dignity_friendly() {
        // Sun in Sagittarius (Jupiter's sign, Jupiter is Sun's friend)
        assert_eq!(planet_dignity(VedicPlanet::Sun, ZodiacSign::Sagittarius), 1);
    }

    #[test]
    fn test_dignity_enemy() {
        // Sun in Capricorn (Saturn's sign, Saturn is Sun's enemy)
        assert_eq!(planet_dignity(VedicPlanet::Sun, ZodiacSign::Capricorn), -1);
    }

    #[test]
    fn test_angular_diff() {
        assert!((angular_diff(10.0, 350.0) - 20.0).abs() < 0.001);
        assert!((angular_diff(0.0, 180.0) - 180.0).abs() < 0.001);
        assert!(angular_diff(90.0, 90.0).abs() < 0.001);
    }

    #[test]
    fn test_natal_aspects_conjunction() {
        let positions = vec![
            VedicPosition {
                planet: VedicPlanet::Sun,
                longitude: 100.0,
                latitude: 0.0,
                speed: 1.0,
                sign: ZodiacSign::Cancer,
                degree_in_sign: 10.0,
                is_retrograde: false,
            },
            VedicPosition {
                planet: VedicPlanet::Mars,
                longitude: 103.0,
                latitude: 0.0,
                speed: 0.5,
                sign: ZodiacSign::Cancer,
                degree_in_sign: 13.0,
                is_retrograde: false,
            },
        ];

        let aspects = calculate_natal_aspects(&positions);
        assert!(
            aspects
                .iter()
                .any(|a| a.aspect_type == AspectType::Conjunction),
            "Should detect conjunction"
        );
    }

    #[test]
    fn test_planet_strengths_normalized() {
        let positions = vec![
            VedicPosition {
                planet: VedicPlanet::Sun,
                longitude: 0.0, // Aries (exalted)
                latitude: 0.0,
                speed: 1.0,
                sign: ZodiacSign::Aries,
                degree_in_sign: 0.0,
                is_retrograde: false,
            },
            VedicPosition {
                planet: VedicPlanet::Saturn,
                longitude: 0.0, // Aries (debilitated)
                latitude: 0.0,
                speed: -0.05,
                sign: ZodiacSign::Aries,
                degree_in_sign: 0.0,
                is_retrograde: true,
            },
        ];

        let aspects = calculate_natal_aspects(&positions);
        let strengths = calculate_planet_strengths(&positions, &aspects);

        for s in &strengths {
            assert!(
                s.total >= 0.0 && s.total <= 1.0,
                "{} strength {} out of range",
                s.planet,
                s.total
            );
        }

        // Sun in Aries (exalted) should be stronger than Saturn in Aries (debilitated)
        let sun_strength = get_planet_strength(&strengths, VedicPlanet::Sun);
        let saturn_strength = get_planet_strength(&strengths, VedicPlanet::Saturn);
        assert!(
            sun_strength > saturn_strength,
            "Exalted Sun ({}) should be stronger than debilitated Saturn ({})",
            sun_strength,
            saturn_strength
        );
    }

    #[test]
    fn test_ephemeris_position() {
        let calc = EphemerisCalculator::new("");
        let now = Utc::now();
        let pos = calculate_position(&calc, VedicPlanet::Sun, &now);
        assert!(pos.is_ok(), "Sun position should succeed: {:?}", pos.err());
        let pos = pos.unwrap();
        assert!((0.0..360.0).contains(&pos.longitude));
        assert!(!pos.is_retrograde, "Sun is never retrograde");
    }

    #[test]
    fn test_ephemeris_all_positions() {
        let calc = EphemerisCalculator::new("");
        let now = Utc::now();
        let positions = calculate_all_positions(&calc, &now);
        assert!(positions.is_ok());
        assert_eq!(positions.unwrap().len(), 9, "Should have all 9 planets");
    }
}
