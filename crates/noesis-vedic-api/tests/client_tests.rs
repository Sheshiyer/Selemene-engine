use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, header};

use noesis_vedic_api::{Config, VedicApiClient, VedicApiError};
use noesis_vedic_api::panchang::{
    Panchang, DateInfo, Location, Tithi, TithiName, Nakshatra, NakshatraName,
    Yoga, YogaName, Karana, KaranaName, KaranaType, Vara, Paksha,
    PlanetaryPositions, PlanetPosition, DayBoundaries,
};

fn sample_planet(name: &str, sign: &str, nakshatra: &str) -> PlanetPosition {
    PlanetPosition {
        name: name.to_string(),
        longitude: 120.0,
        latitude: 0.0,
        speed: 1.0,
        sign: sign.to_string(),
        nakshatra: nakshatra.to_string(),
        pada: 1,
        is_retrograde: false,
    }
}

fn sample_panchang() -> Panchang {
    Panchang {
        date: DateInfo {
            year: 2024,
            month: 1,
            day: 1,
            day_of_week: 1,
            julian_day: 2459945.5,
            hindu_date: None,
        },
        location: Location {
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: 5.5,
            name: Some("Bengaluru".to_string()),
        },
        tithi: Tithi {
            number: 1,
            name_tithi: TithiName::Pratipada,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
            is_complete: true,
        },
        nakshatra: Nakshatra {
            number: 1,
            name_nakshatra: NakshatraName::Ashwini,
            pada: 1,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
            longitude: 13.3,
        },
        yoga: Yoga {
            number: 1,
            name_yoga: YogaName::Preeti,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
        },
        karana: Karana {
            name_karana: KaranaName::Bava,
            karana_type: KaranaType::Movable,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
        },
        vara: Vara::Monday,
        paksha: Paksha::Shukla,
        planets: PlanetaryPositions {
            sun: sample_planet("Sun", "Capricorn", "Shravana"),
            moon: sample_planet("Moon", "Aries", "Ashwini"),
            mars: None,
            mercury: None,
            jupiter: None,
            venus: None,
            saturn: None,
            rahu: None,
            ketu: None,
        },
        day_boundaries: DayBoundaries {
            sunrise: "06:30".to_string(),
            sunset: "18:15".to_string(),
            next_sunrise: "06:31".to_string(),
            day_duration: "11:45".to_string(),
            night_duration: "12:15".to_string(),
        },
        ayanamsa: 24.0,
    }
}

#[tokio::test]
async fn test_get_panchang_adds_auth_header_and_parses() {
    let server = MockServer::start().await;
    let sample = sample_panchang();

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .and(header("authorization", "Bearer test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let client = VedicApiClient::new(config);

    let result = client
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect("Expected panchang response");

    assert_eq!(result.tithi.name(), sample.tithi.name());
    assert_eq!(result.nakshatra.name(), sample.nakshatra.name());
}

#[tokio::test]
async fn test_get_panchang_unauthorized_maps_to_configuration_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid api key"))
        .mount(&server)
        .await;

    let config = Config::new("bad_key").with_base_url(server.uri());
    let client = VedicApiClient::new(config);

    let err = client
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect_err("Expected configuration error");

    match err {
        VedicApiError::Configuration { field, .. } => assert_eq!(field, "api_key"),
        _ => panic!("Unexpected error: {err:?}"),
    }
}

#[tokio::test]
async fn test_get_panchang_rate_limit_maps_to_rate_limit_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(429).set_body_string("rate limit"))
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let client = VedicApiClient::new(config);

    let err = client
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect_err("Expected rate limit error");

    match err {
        VedicApiError::RateLimit { .. } => {}
        _ => panic!("Unexpected error: {err:?}"),
    }
}
