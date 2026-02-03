//! Integration tests for Panchang endpoints
//! 
//! These tests verify that:
//! 1. The Panchang data structures are correct
//! 2. Muhurta calculations work
//! 3. Hora and Choghadiya calculations work

use noesis_vedic_api::panchang::*;

#[test]
fn test_tithi_creation() {
    // Use an auspicious tithi - Pratipada (1st tithi)
    let tithi = Tithi {
        number: 1,
        name_tithi: TithiName::Pratipada,
        start_time: "06:00".to_string(),
        end_time: "18:00".to_string(),
        is_complete: true,
    };
    
    assert_eq!(tithi.number, 1);
    assert_eq!(tithi.name(), "Pratipada");
    assert!(tithi.is_auspicious());
    
    // Test inauspicious tithi - Chaturthi
    let chaturthi = Tithi {
        number: 4,
        name_tithi: TithiName::Chaturthi,
        start_time: "06:00".to_string(),
        end_time: "18:00".to_string(),
        is_complete: true,
    };
    assert!(!chaturthi.is_auspicious());
}

#[test]
fn test_nakshatra_ruler() {
    let nakshatra = Nakshatra {
        number: 12,
        name_nakshatra: NakshatraName::UttaraPhalguni,
        pada: 1,
        start_time: "06:00".to_string(),
        end_time: "18:00".to_string(),
        longitude: 156.0,
    };
    
    assert_eq!(nakshatra.name(), "Uttara Phalguni");
    assert_eq!(nakshatra.ruling_planet(), "Sun");
    assert!(nakshatra.is_auspicious());
}

#[test]
fn test_vara_creation() {
    let vara = Vara::Tuesday;
    assert_eq!(vara.ruling_planet(), "Mars");
    assert!(!vara.is_auspicious()); // Tuesday is not generally auspicious
}

#[test]
fn test_paksha() {
    let shukla = Paksha::Shukla;
    assert_eq!(shukla.as_str(), "Shukla");
    assert_eq!(shukla.english(), "Waxing");
}

#[test]
fn test_yoga_nature() {
    let yoga = Yoga {
        number: 20,
        name_yoga: YogaName::Shiva,
        start_time: "06:00".to_string(),
        end_time: "06:00".to_string(),
    };
    
    assert_eq!(yoga.name(), "Shiva");
    assert_eq!(yoga.nature(), "auspicious");
    assert!(yoga.is_auspicious());
}

#[test]
fn test_karana_type() {
    let karana = Karana {
        name_karana: KaranaName::Bava,
        karana_type: KaranaType::Movable,
        start_time: "06:00".to_string(),
        end_time: "18:00".to_string(),
    };
    
    assert_eq!(karana.name(), "Bava");
    assert!(karana.is_auspicious());
}

#[test]
fn test_muhurta_calculations() {
    use muhurta::*;
    
    // Test Rahu Kalam for different days
    let rahu_sunday = RahuKalam::for_day("Sunday", "06:00", "18:00");
    assert_eq!(rahu_sunday.start, "16:30");
    assert_eq!(rahu_sunday.end, "18:00");
    
    let rahu_monday = RahuKalam::for_day("Monday", "06:00", "18:00");
    assert_eq!(rahu_monday.start, "07:30");
    
    // Test Yama Gandam
    let yama_tuesday = YamaGandam::for_day("Tuesday");
    assert_eq!(yama_tuesday.start, "09:00");
    
    // Test Gulika Kaal
    let gulika_saturday = GulikaKaal::for_day("Saturday");
    assert_eq!(gulika_saturday.start, "06:00");
}

#[test]
fn test_hora_sequence() {
    use hora::*;
    
    // Sunday should start with Sun
    let sunday_sequence = HoraSequence::generate_sequence("Sunday");
    assert_eq!(sunday_sequence[0], Planet::Sun);
    assert_eq!(sunday_sequence.len(), 24);
    
    // Monday should start with Moon
    let monday_sequence = HoraSequence::generate_sequence("Monday");
    assert_eq!(monday_sequence[0], Planet::Moon);
    
    // Saturday should start with Saturn
    let saturday_sequence = HoraSequence::generate_sequence("Saturday");
    assert_eq!(saturday_sequence[0], Planet::Saturn);
}

#[test]
fn test_choghadiya_sequence() {
    use choghadiya::*;
    
    // Sunday day starts with Shubh
    let sunday_day = ChoghadiyaSequence::get_day_sequence("Sunday");
    assert_eq!(sunday_day[0].0, ChoghadiyaName::Shubh);
    
    // Saturday day starts with Kaal
    let saturday_day = ChoghadiyaSequence::get_day_sequence("Saturday");
    assert_eq!(saturday_day[0].0, ChoghadiyaName::Kaal);
}

#[test]
fn test_dasha_planet() {
    use noesis_vedic_api::dasha::*;
    
    // Test periods
    assert_eq!(DashaPlanet::Sun.full_period_years(), 6.0);
    assert_eq!(DashaPlanet::Moon.full_period_years(), 10.0);
    assert_eq!(DashaPlanet::Saturn.full_period_years(), 19.0);
    
    // Test nature
    assert!(DashaPlanet::Jupiter.is_benefic());
    assert!(DashaPlanet::Saturn.is_malefic());
    
    // Test rulers
    assert!(DashaPlanet::Mars.ruling_nakshatras().contains(&"Mrigashira"));
}

#[test]
fn test_panchang_query_builder() {
    let query = PanchangQuery::new(2024, 1, 15, 12.97, 77.59)
        .at(14, 30, 0)
        .with_timezone(5.5)
        .without_hora();
    
    assert_eq!(query.year, 2024);
    assert_eq!(query.month, 1);
    assert_eq!(query.day, 15);
    assert_eq!(query.hour, 14);
    assert_eq!(query.minute, 30);
    assert!(!query.include_hora);
    assert!(query.include_muhurtas);
}

#[test]
fn test_zodiac_sign() {
    use noesis_vedic_api::chart::ZodiacSign;
    
    assert_eq!(ZodiacSign::Aries.index(), 0);
    assert_eq!(ZodiacSign::Aries.ruler(), "Mars");
    assert_eq!(ZodiacSign::Leo.element(), "Fire");
    assert_eq!(ZodiacSign::Taurus.modality(), "Fixed");
    
    // Test wraparound
    assert_eq!(ZodiacSign::from_index(12), ZodiacSign::Aries);
}
