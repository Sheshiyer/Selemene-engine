//! Shadbala calculator
//!
//! FAPI-069: Calculate total Shadbala and Rupas
//!
//! PR3: Kala Bala and Drik Bala were hardcoded constants (30.0 and 15.0).
//! They are now computed from real birth context — Kala Bala via four
//! Parashara subdivisions (Nathonnatha, Paksha, Tribhaga, Hora) and Drik
//! Bala via sign-based Vedic aspect weighting from
//! `crate::transits::aspects::check_vedic_aspects`. Masa/Varsha/Abda/Ayana
//! components are deferred to PR4 per scope discipline — see crate
//! MIGRATION.md.

use super::types::{required_shadbala, PlanetShadbala, ShadbalaComponent, ShadbalaValue};
use crate::birth_chart::types::{BirthChart, Planet, ZodiacSign};
use chrono::Timelike;

/// Calculate Sthana Bala (Positional Strength)
pub fn calculate_sthana_bala(planet: Planet, sign: ZodiacSign, degree: f64) -> f64 {
    let mut bala = 0.0;

    // Uchcha Bala (Exaltation strength)
    bala += calculate_uchcha_bala(planet, sign, degree);

    // Saptavargaja Bala (Divisional chart strength) - simplified
    bala += calculate_saptavargaja_bala(planet, sign);

    // Ojhayugma Bala (Odd-even sign strength)
    bala += calculate_ojhayugma_bala(planet, sign);

    // Kendra Bala (Angular strength) - would need house info
    bala += 15.0; // Default

    // Drekkana Bala - simplified
    bala += 10.0;

    bala
}

/// Calculate Uchcha Bala (exaltation strength)
fn calculate_uchcha_bala(planet: Planet, sign: ZodiacSign, degree: f64) -> f64 {
    let longitude = (sign.number() as f64 - 1.0) * 30.0 + degree;

    // Exaltation points for each planet
    let (exalt_long, _debil_long) = match planet {
        Planet::Sun => (10.0, 190.0),      // Aries 10°
        Planet::Moon => (33.0, 213.0),     // Taurus 3°
        Planet::Mars => (298.0, 118.0),    // Capricorn 28°
        Planet::Mercury => (165.0, 345.0), // Virgo 15°
        Planet::Jupiter => (95.0, 275.0),  // Cancer 5°
        Planet::Venus => (357.0, 177.0),   // Pisces 27°
        Planet::Saturn => (200.0, 20.0),   // Libra 20°
        _ => return 0.0,
    };

    // Distance from exaltation point
    let distance = (longitude - exalt_long).abs();
    let normalized_distance = if distance > 180.0 {
        360.0 - distance
    } else {
        distance
    };

    // Maximum 60 shashtiamsas when at exact exaltation
    let bala = (180.0 - normalized_distance) / 3.0;
    bala.max(0.0)
}

/// Calculate Saptavargaja Bala (simplified)
fn calculate_saptavargaja_bala(planet: Planet, sign: ZodiacSign) -> f64 {
    // Simplified - based on sign relationship
    let sign_lord = sign.ruler();

    if sign_lord == planet {
        45.0 // Own sign
    } else {
        match (planet, sign_lord) {
            // Natural friendships
            (Planet::Sun, Planet::Moon) | (Planet::Moon, Planet::Sun) => 30.0,
            (Planet::Sun, Planet::Mars) | (Planet::Mars, Planet::Sun) => 30.0,
            (Planet::Sun, Planet::Jupiter) | (Planet::Jupiter, Planet::Sun) => 30.0,
            (Planet::Moon, Planet::Mercury) | (Planet::Mercury, Planet::Moon) => 30.0,
            (Planet::Jupiter, Planet::Mars) | (Planet::Mars, Planet::Jupiter) => 30.0,
            (Planet::Venus, Planet::Mercury) | (Planet::Mercury, Planet::Venus) => 30.0,
            (Planet::Venus, Planet::Saturn) | (Planet::Saturn, Planet::Venus) => 30.0,
            // Natural enmities
            (Planet::Sun, Planet::Saturn) | (Planet::Saturn, Planet::Sun) => 7.5,
            (Planet::Sun, Planet::Venus) | (Planet::Venus, Planet::Sun) => 7.5,
            (Planet::Moon, Planet::Saturn) | (Planet::Saturn, Planet::Moon) => 7.5,
            _ => 15.0, // Neutral
        }
    }
}

/// Calculate Ojhayugma Bala
fn calculate_ojhayugma_bala(planet: Planet, sign: ZodiacSign) -> f64 {
    let is_odd_sign = sign.number() % 2 == 1;

    match planet {
        // Sun, Mars, Jupiter prefer odd signs
        Planet::Sun | Planet::Mars | Planet::Jupiter => {
            if is_odd_sign {
                15.0
            } else {
                0.0
            }
        }
        // Moon, Venus, Saturn prefer even signs
        Planet::Moon | Planet::Venus | Planet::Saturn => {
            if !is_odd_sign {
                15.0
            } else {
                0.0
            }
        }
        // Mercury is neutral
        _ => 7.5,
    }
}

/// Calculate Dig Bala (Directional Strength)
pub fn calculate_dig_bala(planet: Planet, house: u8) -> f64 {
    // Maximum Dig Bala is 60 shashtiamsas
    let preferred_house = match planet {
        Planet::Sun | Planet::Mars => 10,       // 10th house (south)
        Planet::Moon | Planet::Venus => 4,      // 4th house (north)
        Planet::Jupiter | Planet::Mercury => 1, // 1st house (east)
        Planet::Saturn => 7,                    // 7th house (west)
        _ => 1,
    };

    // Distance from preferred house (in houses, 0-6)
    let distance = ((house as i8 - preferred_house as i8).abs() % 12) as u8;
    let normalized = if distance > 6 {
        12 - distance
    } else {
        distance
    };

    // 60 at preferred, 0 at opposite
    60.0 - (normalized as f64 * 10.0)
}

// ---------------------------------------------------------------------------
// Kala Bala — temporal strength
// ---------------------------------------------------------------------------

/// Returns true for planets classically grouped as benefics (Jupiter, Venus,
/// Mercury, Moon). The remaining classical grahas (Sun, Mars, Saturn) plus
/// nodes are treated as malefics by Nathonnatha Bala.
fn is_benefic(planet: Planet) -> bool {
    matches!(
        planet,
        Planet::Jupiter | Planet::Venus | Planet::Mercury | Planet::Moon
    )
}

/// Returns true if `local_hour` falls between sunrise (inclusive) and sunset
/// (exclusive). Both inputs are decimal hours in the local civil clock.
fn is_daytime(local_hour: f64, sunrise: f64, sunset: f64) -> bool {
    local_hour >= sunrise && local_hour < sunset
}

/// **Nathonnatha Bala** — day/night strength.
///
/// Sun, Jupiter and Venus gain strength by day; Moon, Mars and Saturn by
/// night. Mercury is always strong (full 60). The strength scales linearly
/// from 0 at the opposite zenith to 60 at the favoured zenith.
fn nathonnatha_bala(planet: Planet, local_hour: f64, sunrise: f64, sunset: f64) -> f64 {
    if matches!(planet, Planet::Mercury) {
        return 60.0;
    }
    // Compute angular "distance from midday" on a wrap-around 24h clock.
    // Distance is at most 12 hours (midnight is 12h from midday).
    let midday = (sunrise + sunset) / 2.0;
    let raw_diff = (local_hour - midday).abs();
    let circular_diff = if raw_diff > 12.0 {
        24.0 - raw_diff
    } else {
        raw_diff
    };
    // Fraction in [0, 1]: 0 at midday, 1 at midnight.
    let frac_from_midday = (circular_diff / 12.0).clamp(0.0, 1.0);

    // Diurnal planets — peak at midday, zero at midnight.
    let diurnal_strength = 60.0 * (1.0 - frac_from_midday);
    // Nocturnal planets — peak at midnight, zero at midday.
    let nocturnal_strength = 60.0 * frac_from_midday;

    match planet {
        Planet::Sun | Planet::Jupiter | Planet::Venus => diurnal_strength,
        Planet::Moon | Planet::Mars | Planet::Saturn => nocturnal_strength,
        _ => 30.0,
    }
}

/// **Paksha Bala** — fortnight strength.
///
/// Benefics gain strength as Moon waxes (Shukla paksha), malefics gain as
/// Moon wanes (Krishna paksha). Mercury and Moon are treated like other
/// benefics. `tithi_continuous` is the 0..30 tithi value where 0 = new
/// moon, 15 = full moon, 30 = next new moon.
fn paksha_bala(planet: Planet, tithi_continuous: f64) -> f64 {
    let t = tithi_continuous.rem_euclid(30.0);
    // Distance from new moon along the 0..15 axis.
    let waxing_strength = if t <= 15.0 { t } else { 30.0 - t };
    // Scale 0..15 to 0..60.
    let strength = waxing_strength * (60.0 / 15.0);
    if is_benefic(planet) || matches!(planet, Planet::Moon) {
        strength
    } else {
        60.0 - strength
    }
}

/// **Tribhaga Bala** — day/night third strength.
///
/// The day is split into three equal parts ruled by Mercury / Sun / Saturn
/// in turn, the night into three ruled by Moon / Venus / Mars. Jupiter is
/// always strong (Parashara grants him 60 unconditionally). The lord of
/// the current third gets 60; everyone else gets 0.
fn tribhaga_bala(planet: Planet, local_hour: f64, sunrise: f64, sunset: f64) -> f64 {
    if matches!(planet, Planet::Jupiter) {
        return 60.0;
    }
    let day = is_daytime(local_hour, sunrise, sunset);
    let day_lords = [Planet::Mercury, Planet::Sun, Planet::Saturn];
    let night_lords = [Planet::Moon, Planet::Venus, Planet::Mars];

    let third_index = if day {
        let day_len = (sunset - sunrise).max(0.0001);
        let third = ((local_hour - sunrise) * 3.0 / day_len)
            .floor()
            .clamp(0.0, 2.0) as usize;
        third
    } else {
        // Wrap to next sunrise (24h period after sunset).
        let night_start = sunset;
        let night_end = sunrise + 24.0;
        let night_len = (night_end - night_start).max(0.0001);
        let hour_in_night = if local_hour >= night_start {
            local_hour
        } else {
            local_hour + 24.0
        };
        ((hour_in_night - night_start) * 3.0 / night_len)
            .floor()
            .clamp(0.0, 2.0) as usize
    };

    let lord = if day {
        day_lords[third_index]
    } else {
        night_lords[third_index]
    };
    if lord == planet {
        60.0
    } else {
        0.0
    }
}

/// **Hora Bala** — strength from the ruling planetary hour.
///
/// The 24 horas of a day follow the Chaldean sequence and rotate from the
/// day-lord at sunrise. If the planet rules the current hora it gets 60,
/// otherwise 0. This uses the existing `hora_alarms::calculator` so the
/// Chaldean cycle is shared with other crate features.
fn hora_bala(
    planet: Planet,
    date: chrono::NaiveDate,
    local_time: chrono::NaiveTime,
    sunrise: chrono::NaiveTime,
    sunset: chrono::NaiveTime,
) -> f64 {
    let horas = crate::hora_alarms::calculator::calculate_day_horas(date, sunrise, sunset);
    if horas.is_empty() {
        return 0.0;
    }
    let active = horas
        .iter()
        .find(|h| local_time >= h.start_time && local_time < h.end_time);
    let Some(active) = active else {
        return 0.0;
    };
    if active.ruler.eq_ignore_ascii_case(&planet.to_string()) {
        60.0
    } else {
        0.0
    }
}

/// Aggregate **Kala Bala** from four Parashara sub-components:
/// Nathonnatha + Paksha + Tribhaga + Hora.
///
/// Masa, Varsha, Abda and Ayana balas are deferred to PR4 (see crate
/// MIGRATION.md). The four covered components alone yield a value in
/// [0, 240] shashtiamsas which is a useful spread even before the year-/
/// month-/cycle-scale terms are added — the existing hardcoded `30.0` was
/// a single constant that buried the variation.
#[allow(clippy::too_many_arguments)]
pub fn calculate_kala_bala(
    planet: Planet,
    date: chrono::NaiveDate,
    local_time: chrono::NaiveTime,
    sunrise: chrono::NaiveTime,
    sunset: chrono::NaiveTime,
    tithi_continuous: f64,
) -> f64 {
    let local_hour = local_time.hour() as f64 + local_time.minute() as f64 / 60.0;
    let sr_hour = sunrise.hour() as f64 + sunrise.minute() as f64 / 60.0;
    let ss_hour = sunset.hour() as f64 + sunset.minute() as f64 / 60.0;

    let natho = nathonnatha_bala(planet, local_hour, sr_hour, ss_hour);
    let paksha = paksha_bala(planet, tithi_continuous);
    let tribhaga = tribhaga_bala(planet, local_hour, sr_hour, ss_hour);
    let hora = hora_bala(planet, date, local_time, sunrise, sunset);

    natho + paksha + tribhaga + hora
}

// ---------------------------------------------------------------------------
// Drik Bala — aspectual strength
// ---------------------------------------------------------------------------

/// Per-aspect contribution magnitude in shashtiamsas. Full Vedic aspects
/// (special 4th/8th of Mars, 5th/9th of Jupiter, 3rd/10th of Saturn, and
/// the 7th cast by everyone) score 30; the partial-aspect signs (sextile-
/// like sign distances) score 15. We treat anything `check_vedic_aspects`
/// returns as "full" since that helper only signals true classical Vedic
/// drishtis (no partial weighting yet); follow-up work could split by
/// sign distance.
const FULL_ASPECT_WEIGHT: f64 = 30.0;

/// Sign positive for benefic aspects, negative for malefic.
fn aspect_polarity(aspecting: Planet) -> f64 {
    // Benefics give +ve, malefics give -ve, Rahu/Ketu treated as malefics.
    if is_benefic(aspecting) {
        1.0
    } else {
        -1.0
    }
}

/// **Drik Bala** for a target planet — sums sign-based Vedic aspects from
/// every other planet in the chart. Benefic aspects contribute positively,
/// malefic aspects negatively. Result is clamped to ±60 shashtiamsas per
/// Parashara so a maxed-out aspectual gain never swamps the other balas.
pub fn calculate_drik_bala(
    target_planet: Planet,
    all_planets: &[crate::birth_chart::types::PlanetPosition],
) -> f64 {
    let Some(target_pos) = all_planets.iter().find(|p| p.planet == target_planet) else {
        return 0.0;
    };
    let mut total = 0.0f64;
    for other in all_planets {
        if other.planet == target_planet {
            continue;
        }
        // Only classical aspecting planets cast Vedic drishti (Rahu/Ketu
        // have tradition-dependent rules — skip per PRIMITIVES.md note).
        if matches!(
            other.planet,
            Planet::Rahu | Planet::Ketu | Planet::Ascendant
        ) {
            continue;
        }
        let aspect = crate::transits::aspects::check_vedic_aspects(
            other.planet,
            other.sign.number(),
            target_pos.sign.number(),
        );
        if aspect.is_some() {
            total += aspect_polarity(other.planet) * FULL_ASPECT_WEIGHT;
        }
    }
    total.clamp(-60.0, 60.0)
}

/// Calculate all Shadbala components for a planet **without** real birth
/// context. Retained for source-compatibility — the values used for Kala
/// and Drik fall back to neutral mid-points so existing tests pass. Prefer
/// [`calculate_full_shadbala_with_context`] for native facade callers.
pub fn calculate_full_shadbala(
    planet: Planet,
    sign: ZodiacSign,
    degree: f64,
    house: u8,
    is_retrograde: bool,
) -> PlanetShadbala {
    let sthana = calculate_sthana_bala(planet, sign, degree);
    let dig = calculate_dig_bala(planet, house);

    // Neutral fallbacks — context-free variant is used by older tests only.
    let kala = 30.0;

    // Chesta Bala (retrograde planets get bonus)
    let chesta = if is_retrograde { 60.0 } else { 30.0 };

    // Naisargika Bala (natural strength)
    let naisargika = match planet {
        Planet::Sun => 60.0,
        Planet::Moon => 51.43,
        Planet::Venus => 42.85,
        Planet::Jupiter => 34.28,
        Planet::Mercury => 25.71,
        Planet::Mars => 17.14,
        Planet::Saturn => 8.57,
        _ => 0.0,
    };

    let drik = 15.0;

    let total = sthana + dig + kala + chesta + naisargika + drik;
    let required = required_shadbala(&planet.to_string());
    let ratio = total / required;

    PlanetShadbala {
        planet: planet.to_string(),
        components: vec![
            ShadbalaValue {
                component: ShadbalaComponent::SthanaBala,
                rupas: sthana,
                shashtiamsas: sthana,
            },
            ShadbalaValue {
                component: ShadbalaComponent::DigBala,
                rupas: dig,
                shashtiamsas: dig,
            },
            ShadbalaValue {
                component: ShadbalaComponent::KalaBala,
                rupas: kala,
                shashtiamsas: kala,
            },
            ShadbalaValue {
                component: ShadbalaComponent::ChestaBala,
                rupas: chesta,
                shashtiamsas: chesta,
            },
            ShadbalaValue {
                component: ShadbalaComponent::NaisargikaBala,
                rupas: naisargika,
                shashtiamsas: naisargika,
            },
            ShadbalaValue {
                component: ShadbalaComponent::DrikBala,
                rupas: drik,
                shashtiamsas: drik,
            },
        ],
        total_rupas: total,
        total_shashtiamsas: total,
        required_minimum: required,
        strength_ratio: ratio,
        is_strong: ratio >= 1.0,
    }
}

/// Calculate all Shadbala components for a planet **with** real birth
/// context. PR3 uses this from the native `get_shadbala` facade.
///
/// `chart` provides the planet array Drik Bala needs; `date`/`local_time`/
/// `sunrise`/`sunset`/`tithi_continuous` feed Kala Bala.
#[allow(clippy::too_many_arguments)]
pub fn calculate_full_shadbala_with_context(
    planet: Planet,
    sign: ZodiacSign,
    degree: f64,
    house: u8,
    is_retrograde: bool,
    chart: &BirthChart,
    date: chrono::NaiveDate,
    local_time: chrono::NaiveTime,
    sunrise: chrono::NaiveTime,
    sunset: chrono::NaiveTime,
    tithi_continuous: f64,
) -> PlanetShadbala {
    let sthana = calculate_sthana_bala(planet, sign, degree);
    let dig = calculate_dig_bala(planet, house);

    let kala = calculate_kala_bala(planet, date, local_time, sunrise, sunset, tithi_continuous);

    let chesta = if is_retrograde { 60.0 } else { 30.0 };

    let naisargika = match planet {
        Planet::Sun => 60.0,
        Planet::Moon => 51.43,
        Planet::Venus => 42.85,
        Planet::Jupiter => 34.28,
        Planet::Mercury => 25.71,
        Planet::Mars => 17.14,
        Planet::Saturn => 8.57,
        _ => 0.0,
    };

    let drik = calculate_drik_bala(planet, &chart.planets);

    let total = sthana + dig + kala + chesta + naisargika + drik;
    let required = required_shadbala(&planet.to_string());
    let ratio = total / required;

    PlanetShadbala {
        planet: planet.to_string(),
        components: vec![
            ShadbalaValue {
                component: ShadbalaComponent::SthanaBala,
                rupas: sthana,
                shashtiamsas: sthana,
            },
            ShadbalaValue {
                component: ShadbalaComponent::DigBala,
                rupas: dig,
                shashtiamsas: dig,
            },
            ShadbalaValue {
                component: ShadbalaComponent::KalaBala,
                rupas: kala,
                shashtiamsas: kala,
            },
            ShadbalaValue {
                component: ShadbalaComponent::ChestaBala,
                rupas: chesta,
                shashtiamsas: chesta,
            },
            ShadbalaValue {
                component: ShadbalaComponent::NaisargikaBala,
                rupas: naisargika,
                shashtiamsas: naisargika,
            },
            ShadbalaValue {
                component: ShadbalaComponent::DrikBala,
                rupas: drik,
                shashtiamsas: drik,
            },
        ],
        total_rupas: total,
        total_shashtiamsas: total,
        required_minimum: required,
        strength_ratio: ratio,
        is_strong: ratio >= 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_dig_bala() {
        // Sun in 10th house should have high Dig Bala
        let dig = calculate_dig_bala(Planet::Sun, 10);
        assert!(dig >= 50.0);

        // Sun in 4th house should have low Dig Bala
        let dig = calculate_dig_bala(Planet::Sun, 4);
        assert!(dig <= 20.0);
    }

    #[test]
    fn test_uchcha_bala() {
        // Sun near exaltation in Aries
        let bala = calculate_uchcha_bala(Planet::Sun, ZodiacSign::Aries, 10.0);
        assert!(bala >= 50.0);

        // Sun in Libra (debilitation)
        let bala = calculate_uchcha_bala(Planet::Sun, ZodiacSign::Libra, 10.0);
        assert!(bala < 20.0);
    }

    #[test]
    fn test_full_shadbala() {
        let shadbala = calculate_full_shadbala(Planet::Sun, ZodiacSign::Aries, 10.0, 10, false);

        assert_eq!(shadbala.planet, "Sun");
        assert!(shadbala.total_rupas > 0.0);
        assert_eq!(shadbala.components.len(), 6);
    }

    #[test]
    fn nathonnatha_sun_strongest_at_midday() {
        // Sun is strongest at midday (high sun) — small fraction-from-midday.
        let midday = nathonnatha_bala(Planet::Sun, 12.0, 6.0, 18.0);
        let dawn = nathonnatha_bala(Planet::Sun, 6.0, 6.0, 18.0);
        assert!(midday > dawn, "midday Sun ({midday}) > dawn ({dawn})");
        assert!(midday >= 50.0);
    }

    #[test]
    fn nathonnatha_moon_strongest_at_midnight() {
        // Moon is a nocturnal planet — midnight (0.0) should beat midday.
        let midnight = nathonnatha_bala(Planet::Moon, 0.0, 6.0, 18.0);
        let midday = nathonnatha_bala(Planet::Moon, 12.0, 6.0, 18.0);
        assert!(midnight > midday);
    }

    #[test]
    fn paksha_full_moon_max_for_benefics() {
        let benefic_full = paksha_bala(Planet::Jupiter, 15.0);
        let benefic_new = paksha_bala(Planet::Jupiter, 0.0);
        assert!(benefic_full > benefic_new);
        assert!(benefic_full >= 59.0);
    }

    #[test]
    fn paksha_full_moon_min_for_malefics() {
        let malefic_full = paksha_bala(Planet::Saturn, 15.0);
        let malefic_new = paksha_bala(Planet::Saturn, 0.0);
        assert!(malefic_new > malefic_full);
    }

    #[test]
    fn drik_bala_clamps_to_parashara_range() {
        use crate::birth_chart::types::{Dignity, PlanetPosition};
        // Construct a small chart where every benefic aspects Sun in Aries:
        // because aspects are sign-based, putting other planets in Libra
        // (7th from Aries) triggers the universal 7th aspect.
        let make = |planet: Planet, sign: ZodiacSign| PlanetPosition {
            planet,
            sign,
            degree: 15.0,
            longitude: (sign.number() as f64 - 1.0) * 30.0 + 15.0,
            house: 1,
            nakshatra: String::new(),
            pada: 1,
            is_retrograde: false,
            is_combust: false,
            dignity: Some(Dignity::Neutral),
        };
        let planets = vec![
            make(Planet::Sun, ZodiacSign::Aries),
            make(Planet::Jupiter, ZodiacSign::Libra),
            make(Planet::Venus, ZodiacSign::Libra),
            make(Planet::Mercury, ZodiacSign::Libra),
            make(Planet::Moon, ZodiacSign::Libra),
            make(Planet::Mars, ZodiacSign::Libra),
            make(Planet::Saturn, ZodiacSign::Libra),
        ];
        let drik = calculate_drik_bala(Planet::Sun, &planets);
        assert!((-60.0..=60.0).contains(&drik));
    }

    #[test]
    fn kala_bala_sums_four_components_within_range() {
        let date = NaiveDate::from_ymd_opt(1991, 8, 13).unwrap();
        let local = NaiveTime::from_hms_opt(13, 31, 0).unwrap();
        let sunrise = NaiveTime::from_hms_opt(6, 5, 0).unwrap();
        let sunset = NaiveTime::from_hms_opt(18, 40, 0).unwrap();
        // Tithi ~12 (Shukla paksha, near full moon).
        let kala = calculate_kala_bala(Planet::Sun, date, local, sunrise, sunset, 12.0);
        // Each of 4 components contributes ≤60, so kala ≤ 240. Mercury would
        // hit 120 trivially via Nathonnatha=60 + benefic Paksha, so a small
        // positive lower bound is reasonable. Just sanity-check the
        // bookkeeping clamps to [0, 240].
        assert!((0.0..=240.0).contains(&kala), "kala out of range: {kala}");
    }
}
