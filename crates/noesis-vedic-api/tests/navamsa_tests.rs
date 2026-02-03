use noesis_vedic_api::chart::{NavamsaChart, NativeInfo, ZodiacSign};
use noesis_vedic_api::vargas::navamsa_types::{NavamsaChartDto, NavamsaPositionDto};
use noesis_vedic_api::vargas::navamsa_mappers::map_navamsa_chart;

#[test]
fn test_navamsa_calculation() {
    let sign = NavamsaChart::calculate_navamsa(ZodiacSign::Aries, 10.0);
    assert_eq!(sign, ZodiacSign::Cancer);
}

#[test]
fn test_navamsa_mapper() {
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
    assert_eq!(chart.navamsa_positions.len(), 1);
    assert_eq!(chart.navamsa_positions[0].sign, ZodiacSign::Leo);
    assert_eq!(chart.d9_lagna, ZodiacSign::Aries);
    assert_eq!(chart.vargottama, vec!["Sun".to_string()]);
}
