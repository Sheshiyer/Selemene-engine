use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

use noesis_vedic_api::{Config, CachedVedicClient};
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
async fn test_cached_panchang_uses_cache_for_same_date_location() {
    let server = MockServer::start().await;
    let sample = sample_panchang();

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let client = CachedVedicClient::new(config);

    let _first = client
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect("panchang first call");

    let _second = client
        .get_panchang(2024, 1, 1, 13, 30, 0, 12.97, 77.59, 5.5)
        .await
        .expect("panchang second call");

    let received = server.received_requests().await.expect("requests");
    assert_eq!(received.len(), 1, "expected cached response on second call");
}

#[tokio::test]
async fn test_cached_panchang_diff_date_triggers_new_request() {
    let server = MockServer::start().await;
    let sample = sample_panchang();

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let client = CachedVedicClient::new(config);

    let _first = client
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect("panchang first call");

    let _second = client
        .get_panchang(2024, 1, 2, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect("panchang second call");

    let received = server.received_requests().await.expect("requests");
    assert_eq!(received.len(), 2, "expected new request for different date");
}
