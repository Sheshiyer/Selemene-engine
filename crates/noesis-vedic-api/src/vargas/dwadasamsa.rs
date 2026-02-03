//! Dwadasamsa (D12) calculation helpers

use serde::{Deserialize, Serialize};

use crate::chart::{NativeInfo, PlanetPosition, ZodiacSign};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DwadasamsaChart {
    pub source: NativeInfo,
    pub positions: Vec<DwadasamsaPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DwadasamsaPosition {
    pub planet: String,
    pub sign: ZodiacSign,
    pub degree: f64,
}

/// Compute Dwadasamsa positions for planets in a chart.
pub fn calculate_dwadasamsa(source: NativeInfo, planets: &[PlanetPosition]) -> DwadasamsaChart {
    let positions = planets
        .iter()
        .map(|p| DwadasamsaPosition {
            planet: p.name.clone(),
            sign: dwadasamsa_sign(p.sign, p.degree),
            degree: p.degree,
        })
        .collect();

    DwadasamsaChart { source, positions }
}

/// Calculate D12 sign from a Rashi sign and degree.
/// Each sign is divided into 12 parts of 2.5 degrees starting from the same sign.
pub fn dwadasamsa_sign(rashi_sign: ZodiacSign, degree: f64) -> ZodiacSign {
    let division = (degree / 2.5) as usize;
    ZodiacSign::from_index(rashi_sign.index() + division)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dwadasamsa_sign() {
        let sign = dwadasamsa_sign(ZodiacSign::Aries, 5.0);
        assert_eq!(sign, ZodiacSign::Gemini);
    }
}
