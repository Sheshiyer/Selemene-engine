//! Saptamsa (D7) calculation helpers

use serde::{Deserialize, Serialize};

use crate::chart::{NativeInfo, PlanetPosition, ZodiacSign};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaptamsaChart {
    pub source: NativeInfo,
    pub positions: Vec<SaptamsaPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaptamsaPosition {
    pub planet: String,
    pub sign: ZodiacSign,
    pub degree: f64,
}

/// Compute Saptamsa positions for planets in a chart.
pub fn calculate_saptamsa(source: NativeInfo, planets: &[PlanetPosition]) -> SaptamsaChart {
    let positions = planets
        .iter()
        .map(|p| SaptamsaPosition {
            planet: p.name.clone(),
            sign: saptamsa_sign(p.sign, p.degree),
            degree: p.degree,
        })
        .collect();

    SaptamsaChart { source, positions }
}

/// Calculate D7 sign from a Rashi sign and degree.
/// Odd signs start from Aries, even signs start from Libra.
pub fn saptamsa_sign(rashi_sign: ZodiacSign, degree: f64) -> ZodiacSign {
    let division = (degree / (30.0 / 7.0)) as usize;
    let sign_number = rashi_sign.index() + 1;
    let start_index = if sign_number % 2 == 1 { 0 } else { 6 };
    ZodiacSign::from_index(start_index + division)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saptamsa_sign_odd() {
        // At 10 degrees, division = floor(10 / 4.2857) = 2
        // Odd sign (Aries) starts from Aries, so index 0 + 2 = 2 (Gemini)
        let sign = saptamsa_sign(ZodiacSign::Aries, 10.0);
        assert_eq!(sign, ZodiacSign::Gemini);
    }

    #[test]
    fn test_saptamsa_sign_even() {
        // At 10 degrees, division = floor(10 / 4.2857) = 2
        // Even sign (Taurus) starts from Libra (index 6), so 6 + 2 = 8 (Sagittarius)
        let sign = saptamsa_sign(ZodiacSign::Taurus, 10.0);
        assert_eq!(sign, ZodiacSign::Sagittarius);
    }
}
