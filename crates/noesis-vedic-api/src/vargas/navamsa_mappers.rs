//! Mapping helpers for Navamsa DTOs

use crate::chart::{NavamsaChart, NavamsaPosition, NativeInfo, ZodiacSign};
use crate::vargas::navamsa_types::{NavamsaChartDto, NavamsaPositionDto};

pub fn map_navamsa_chart(dto: &NavamsaChartDto, source: NativeInfo) -> NavamsaChart {
    let positions = dto
        .positions
        .iter()
        .map(map_position)
        .collect::<Vec<_>>();

    let vargottama = dto.vargottama.clone().unwrap_or_default();

    let d9_lagna = dto
        .lagna
        .as_deref()
        .map(parse_sign)
        .unwrap_or(ZodiacSign::Aries);

    NavamsaChart {
        source,
        navamsa_positions: positions,
        vargottama,
        d9_lagna,
    }
}

fn map_position(pos: &NavamsaPositionDto) -> NavamsaPosition {
    NavamsaPosition {
        planet: pos.planet.clone(),
        sign: parse_sign(&pos.sign),
        degree: pos.degree,
        is_vargottama: false,
    }
}

fn parse_sign(sign: &str) -> ZodiacSign {
    match sign.to_lowercase().as_str() {
        "aries" => ZodiacSign::Aries,
        "taurus" => ZodiacSign::Taurus,
        "gemini" => ZodiacSign::Gemini,
        "cancer" => ZodiacSign::Cancer,
        "leo" => ZodiacSign::Leo,
        "virgo" => ZodiacSign::Virgo,
        "libra" => ZodiacSign::Libra,
        "scorpio" => ZodiacSign::Scorpio,
        "sagittarius" => ZodiacSign::Sagittarius,
        "capricorn" => ZodiacSign::Capricorn,
        "aquarius" => ZodiacSign::Aquarius,
        _ => ZodiacSign::Pisces,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_navamsa_chart() {
        let dto = NavamsaChartDto {
            positions: vec![NavamsaPositionDto {
                planet: "Sun".to_string(),
                sign: "Leo".to_string(),
                degree: 12.0,
            }],
            lagna: Some("Aries".to_string()),
            vargottama: Some(vec!["Sun".to_string()]),
        };

        let source = NativeInfo {
            birth_date: "1991-08-13".to_string(),
            birth_time: "13:31".to_string(),
            latitude: 12.97,
            longitude: 77.59,
            timezone: 5.5,
        };

        let chart = map_navamsa_chart(&dto, source);
        assert_eq!(chart.navamsa_positions[0].sign, ZodiacSign::Leo);
        assert_eq!(chart.d9_lagna, ZodiacSign::Aries);
        assert_eq!(chart.vargottama.len(), 1);
    }
}
