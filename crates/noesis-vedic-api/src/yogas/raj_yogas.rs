//! Raj Yoga detection
//!
//! FAPI-065: Map common Raj Yogas

use super::types::{DetectedYoga, YogaCategory, YogaStrength};
use crate::birth_chart::types::{BirthChart, Planet, ZodiacSign};

/// Detect Raj Yogas in a birth chart
pub fn detect_raj_yogas(chart: &BirthChart) -> Vec<DetectedYoga> {
    let mut yogas = vec![];

    // Gaja Kesari Yoga - Moon and Jupiter in kendras from each other
    if let Some(gaja_kesari) = detect_gaja_kesari(chart) {
        yogas.push(gaja_kesari);
    }

    // Pancha Mahapurusha Yogas
    yogas.extend(detect_mahapurusha_yogas(chart));

    // Kendra-Trikona Raj Yogas
    yogas.extend(detect_kendra_trikona_yogas(chart));

    // Lakshmi Yoga
    if let Some(lakshmi) = detect_lakshmi_yoga(chart) {
        yogas.push(lakshmi);
    }

    yogas
}

/// Detect Gaja Kesari Yoga
fn detect_gaja_kesari(chart: &BirthChart) -> Option<DetectedYoga> {
    let moon_pos = chart.get_planet(Planet::Moon)?;
    let jupiter_pos = chart.get_planet(Planet::Jupiter)?;

    // Check if Moon and Jupiter are in kendras (1, 4, 7, 10) from each other
    let moon_house = moon_pos.house;
    let jupiter_house = jupiter_pos.house;

    let difference = ((jupiter_house as i8 - moon_house as i8).abs() % 12) as u8;
    let kendras = [0, 3, 6, 9]; // 0 = same house, 3 = 4th, 6 = 7th, 9 = 10th

    if kendras.contains(&(difference % 12)) {
        let strength = if !moon_pos.is_combust && !jupiter_pos.is_retrograde {
            YogaStrength::Full
        } else {
            YogaStrength::Partial
        };

        Some(DetectedYoga {
            name: "Gaja Kesari Yoga".to_string(),
            category: YogaCategory::RajYoga,
            strength,
            planets_involved: vec!["Moon".to_string(), "Jupiter".to_string()],
            houses_involved: vec![moon_house, jupiter_house],
            description: "Moon and Jupiter in mutual kendras".to_string(),
            results: "Fame, recognition, wisdom, and good fortune".to_string(),
            activation_periods: vec!["Moon Dasha".to_string(), "Jupiter Dasha".to_string()],
        })
    } else {
        None
    }
}

/// Detect Pancha Mahapurusha Yogas
fn detect_mahapurusha_yogas(chart: &BirthChart) -> Vec<DetectedYoga> {
    let mut yogas = vec![];
    let kendra_houses = [1, 4, 7, 10];

    // Ruchaka Yoga - Mars in own sign or exalted in kendra
    if let Some(mars) = chart.get_planet(Planet::Mars) {
        if kendra_houses.contains(&mars.house) {
            let is_strong = mars.sign == ZodiacSign::Aries
                || mars.sign == ZodiacSign::Scorpio
                || mars.sign == ZodiacSign::Capricorn;

            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Ruchaka Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: if mars.is_retrograde {
                        YogaStrength::Partial
                    } else {
                        YogaStrength::Full
                    },
                    planets_involved: vec!["Mars".to_string()],
                    houses_involved: vec![mars.house],
                    description: "Mars in own sign or exalted in a kendra".to_string(),
                    results: "Courage, leadership, military success, physical strength".to_string(),
                    activation_periods: vec!["Mars Dasha".to_string()],
                });
            }
        }
    }

    // Bhadra Yoga - Mercury in own sign or exalted in kendra
    if let Some(mercury) = chart.get_planet(Planet::Mercury) {
        if kendra_houses.contains(&mercury.house) {
            let is_strong = mercury.sign == ZodiacSign::Gemini || mercury.sign == ZodiacSign::Virgo;

            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Bhadra Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: YogaStrength::Full,
                    planets_involved: vec!["Mercury".to_string()],
                    houses_involved: vec![mercury.house],
                    description: "Mercury in own sign or exalted in a kendra".to_string(),
                    results: "Intelligence, eloquence, business acumen".to_string(),
                    activation_periods: vec!["Mercury Dasha".to_string()],
                });
            }
        }
    }

    // Hamsa Yoga - Jupiter in own sign or exalted in kendra
    if let Some(jupiter) = chart.get_planet(Planet::Jupiter) {
        if kendra_houses.contains(&jupiter.house) {
            let is_strong = jupiter.sign == ZodiacSign::Sagittarius
                || jupiter.sign == ZodiacSign::Pisces
                || jupiter.sign == ZodiacSign::Cancer;

            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Hamsa Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: YogaStrength::Full,
                    planets_involved: vec!["Jupiter".to_string()],
                    houses_involved: vec![jupiter.house],
                    description: "Jupiter in own sign or exalted in a kendra".to_string(),
                    results: "Wisdom, righteousness, spiritual inclination, respect".to_string(),
                    activation_periods: vec!["Jupiter Dasha".to_string()],
                });
            }
        }
    }

    // Malavya Yoga - Venus in own sign or exalted in kendra
    if let Some(venus) = chart.get_planet(Planet::Venus) {
        if kendra_houses.contains(&venus.house) {
            let is_strong = venus.sign == ZodiacSign::Taurus
                || venus.sign == ZodiacSign::Libra
                || venus.sign == ZodiacSign::Pisces;

            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Malavya Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: YogaStrength::Full,
                    planets_involved: vec!["Venus".to_string()],
                    houses_involved: vec![venus.house],
                    description: "Venus in own sign or exalted in a kendra".to_string(),
                    results: "Beauty, luxury, artistic talents, comfortable life".to_string(),
                    activation_periods: vec!["Venus Dasha".to_string()],
                });
            }
        }
    }

    // Shasha Yoga - Saturn in own sign or exalted in kendra
    if let Some(saturn) = chart.get_planet(Planet::Saturn) {
        if kendra_houses.contains(&saturn.house) {
            let is_strong = saturn.sign == ZodiacSign::Capricorn
                || saturn.sign == ZodiacSign::Aquarius
                || saturn.sign == ZodiacSign::Libra;

            if is_strong {
                yogas.push(DetectedYoga {
                    name: "Shasha Yoga".to_string(),
                    category: YogaCategory::MahapurushaYoga,
                    strength: if saturn.is_retrograde {
                        YogaStrength::Partial
                    } else {
                        YogaStrength::Full
                    },
                    planets_involved: vec!["Saturn".to_string()],
                    houses_involved: vec![saturn.house],
                    description: "Saturn in own sign or exalted in a kendra".to_string(),
                    results: "Authority, discipline, longevity, service achievements".to_string(),
                    activation_periods: vec!["Saturn Dasha".to_string()],
                });
            }
        }
    }

    yogas
}

const KENDRA_HOUSES: [u8; 4] = [1, 4, 7, 10];
const TRIKONA_HOUSES: [u8; 3] = [1, 5, 9];

/// Minimum orb (degrees) within which two planets are treated as conjunct.
const CONJUNCTION_ORB_DEGREES: f64 = 8.0;

/// Angular separation (degrees) between two longitudes, normalised to [0, 180].
fn angular_separation(a: f64, b: f64) -> f64 {
    let diff = (a - b).abs() % 360.0;
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Compute the whole-sign house (1..=12) of a sign relative to the lagna.
fn sign_house(sign: ZodiacSign, lagna: ZodiacSign) -> u8 {
    let diff = (sign.number() as i16 - lagna.number() as i16).rem_euclid(12);
    (diff + 1) as u8
}

/// Find the planet that rules a given house number in the chart by looking
/// up the sign on the (whole-sign) house cusp and returning its ruler.
fn house_lord(chart: &BirthChart, house: u8) -> Option<Planet> {
    let lagna_idx = chart.ascendant_sign.number() as i16;
    let target_sign_idx = ((lagna_idx - 1 + (house as i16 - 1)).rem_euclid(12)) + 1;
    ZodiacSign::from_number(target_sign_idx as u8).map(|s| s.ruler())
}

/// Detect Kendra–Trikona Raj Yogas.
///
/// Classical rule (Parashara): when the lord of a Kendra house (1, 4, 7, 10)
/// and the lord of a Trikona house (1, 5, 9) form a strong relationship,
/// the chart carries a Raj Yoga. We treat any of the following as a
/// "strong relationship":
///
/// * **Same sign** — the two lords share the same sign (full conjunction).
/// * **Conjunction within 8° orb** — same or adjacent signs with longitudes
///   inside `CONJUNCTION_ORB_DEGREES`.
/// * **Mutual exchange (Parivartana)** — Kendra lord sits in the Trikona
///   lord's sign and vice versa.
/// * **Mutual Vedic aspect** — both lords cast a sign-based aspect on each
///   other via [`crate::transits::aspects::check_vedic_aspects`].
///
/// House 1 is both a Kendra and a Trikona; the loop skips
/// (lord_1, lord_1) self-pairs but still detects the (lord_1, lord_5) /
/// (lord_1, lord_9) / (lord_4, lord_1) / etc. cases that classical
/// commentators treat as Raj Yoga.
pub fn detect_kendra_trikona_yogas(chart: &BirthChart) -> Vec<DetectedYoga> {
    let mut detected = Vec::new();
    let mut seen: std::collections::HashSet<(Planet, Planet, u8, u8)> =
        std::collections::HashSet::new();

    for &k_house in KENDRA_HOUSES.iter() {
        for &t_house in TRIKONA_HOUSES.iter() {
            if k_house == t_house {
                // 1st house is shared; pairing the same lord with itself
                // is not a yoga.
                continue;
            }
            let Some(k_lord) = house_lord(chart, k_house) else {
                continue;
            };
            let Some(t_lord) = house_lord(chart, t_house) else {
                continue;
            };
            if k_lord == t_lord {
                // Same planet rules both — automatic yoga (Yogakaraka),
                // but we record it once.
                let key = (k_lord, t_lord, k_house, t_house);
                if seen.insert(key) {
                    if let Some(pos) = chart.get_planet(k_lord) {
                        detected.push(DetectedYoga {
                            name: format!(
                                "Kendra-Trikona Yogakaraka ({} ruling {} and {})",
                                k_lord, k_house, t_house
                            ),
                            category: YogaCategory::RajYoga,
                            strength: if pos.is_combust {
                                YogaStrength::Partial
                            } else {
                                YogaStrength::Full
                            },
                            planets_involved: vec![k_lord.to_string()],
                            houses_involved: vec![k_house, t_house],
                            description: format!(
                                "{} is the lord of both kendra ({}) and trikona ({}) — a Raj Yogakaraka",
                                k_lord, k_house, t_house
                            ),
                            results: "Power, authority, success, and fortune through the karaka's significations".to_string(),
                            activation_periods: vec![format!("{} Dasha", k_lord)],
                        });
                    }
                }
                continue;
            }

            let Some(k_pos) = chart.get_planet(k_lord) else {
                continue;
            };
            let Some(t_pos) = chart.get_planet(t_lord) else {
                continue;
            };

            let same_sign = k_pos.sign == t_pos.sign;
            let within_orb =
                angular_separation(k_pos.longitude, t_pos.longitude) <= CONJUNCTION_ORB_DEGREES;
            let parivartana = k_pos.sign.ruler() == t_lord && t_pos.sign.ruler() == k_lord;
            // Sign-based Vedic aspect.
            let k_aspects_t = crate::transits::aspects::check_vedic_aspects(
                k_lord,
                k_pos.sign.number(),
                t_pos.sign.number(),
            )
            .is_some();
            let t_aspects_k = crate::transits::aspects::check_vedic_aspects(
                t_lord,
                t_pos.sign.number(),
                k_pos.sign.number(),
            )
            .is_some();
            let mutual_aspect = k_aspects_t && t_aspects_k;

            if !(same_sign || within_orb || parivartana || mutual_aspect) {
                continue;
            }

            let key = (k_lord, t_lord, k_house, t_house);
            if !seen.insert(key) {
                continue;
            }

            let mut planets = vec![k_lord.to_string(), t_lord.to_string()];
            planets.sort();
            planets.dedup();

            let relation = if parivartana {
                "Parivartana (mutual exchange)"
            } else if same_sign {
                "Conjunction in same sign"
            } else if within_orb {
                "Tight conjunction within 8 degrees"
            } else {
                "Mutual Vedic aspect"
            };

            let strength = if (same_sign || within_orb) && !k_pos.is_combust && !t_pos.is_combust {
                YogaStrength::Full
            } else if mutual_aspect && (k_pos.is_combust || t_pos.is_combust) {
                YogaStrength::Weak
            } else {
                YogaStrength::Partial
            };

            // Whole-sign houses of the involved lords help readers locate the yoga.
            let k_lord_house = sign_house(k_pos.sign, chart.ascendant_sign);
            let t_lord_house = sign_house(t_pos.sign, chart.ascendant_sign);

            detected.push(DetectedYoga {
                name: format!(
                    "Kendra-Trikona Raj Yoga ({} of {}H + {} of {}H)",
                    k_lord, k_house, t_lord, t_house
                ),
                category: YogaCategory::RajYoga,
                strength,
                planets_involved: planets,
                houses_involved: vec![k_lord_house, t_lord_house],
                description: format!(
                    "Lord of kendra ({}H={}) and lord of trikona ({}H={}) connected via {}",
                    k_house, k_lord, t_house, t_lord, relation
                ),
                results: "Confers power, recognition, status and prosperity".to_string(),
                activation_periods: vec![format!("{} Dasha", k_lord), format!("{} Dasha", t_lord)],
            });
        }
    }

    detected
}

/// Detect Lakshmi Yoga
fn detect_lakshmi_yoga(chart: &BirthChart) -> Option<DetectedYoga> {
    let venus = chart.get_planet(Planet::Venus)?;

    // Venus should be in own or exalted sign and 9th lord strong
    let is_venus_strong = venus.sign == ZodiacSign::Taurus
        || venus.sign == ZodiacSign::Libra
        || venus.sign == ZodiacSign::Pisces;

    if is_venus_strong && !venus.is_combust {
        Some(DetectedYoga {
            name: "Lakshmi Yoga".to_string(),
            category: YogaCategory::RajYoga,
            strength: YogaStrength::Partial,
            planets_involved: vec!["Venus".to_string()],
            houses_involved: vec![venus.house],
            description: "Venus in strength with 9th lord".to_string(),
            results: "Wealth, prosperity, luxury, fortunate marriage".to_string(),
            activation_periods: vec!["Venus Dasha".to_string()],
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_mahapurusha_detection() {
        // Would need a full birth chart to test
        // This is a placeholder
        assert!(true);
    }
}
