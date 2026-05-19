//! Mock data factories for testing noesis-vedic-api
//!
//! Provides realistic mock responses for all VedicApiService methods,
//! covering both success and error scenarios. These mocks are gated
//! behind the `mocks` feature or `#[cfg(test)]`.
//!
//! # Usage
//! ```rust,ignore
//! use noesis_vedic_api::mocks;
//!
//! let panchang = mocks::mock_panchang();
//! let dasha = mocks::mock_vimshottari_dasha();
//! let chart = mocks::mock_birth_chart();
//! ```

use crate::chart::{
    AscendantInfo, BirthChart, HousePosition, HouseType, MoonInfo, NativeInfo, NavamsaChart,
    NavamsaPosition, PlanetPosition as ChartPlanetPosition, SpecialPoints, ZodiacSign,
};
use crate::dasha::{DashaBalance, DashaLevel, DashaPeriod, DashaPlanet, VimshottariDasha};
use crate::error::VedicApiError;
use crate::panchang::data::PlanetPosition;
use crate::panchang::hora::Planet as HoraPlanet;
use crate::panchang::{
    Choghadiya, ChoghadiyaName, ChoghadiyaNature, ChoghadiyaTimings, CompletePanchang, DateInfo,
    DayBoundaries, Hora, HoraTimings, Karana, KaranaName, KaranaType, Location, Muhurta,
    MuhurtaCollection, MuhurtaNature, Nakshatra, NakshatraName, Paksha, Panchang, PanchangMetadata,
    PlanetaryPositions, Tithi, TithiName, Vara, Yoga, YogaName,
};

// ---------------------------------------------------------------------------
// Constants: Bangalore, India test fixture
// ---------------------------------------------------------------------------

/// Bangalore latitude used across all mock fixtures
pub const MOCK_LAT: f64 = 12.9716;
/// Bangalore longitude used across all mock fixtures
pub const MOCK_LNG: f64 = 77.5946;
/// IST timezone offset
pub const MOCK_TZONE: f64 = 5.5;
/// Standard test date: 2024-01-15
pub const MOCK_YEAR: i32 = 2024;
pub const MOCK_MONTH: u32 = 1;
pub const MOCK_DAY: u32 = 15;

// ---------------------------------------------------------------------------
// Panchang mocks
// ---------------------------------------------------------------------------

/// Create a realistic mock Panchang for 2024-01-15 Bangalore
pub fn mock_panchang() -> Panchang {
    Panchang {
        date: DateInfo {
            year: MOCK_YEAR,
            month: MOCK_MONTH,
            day: MOCK_DAY,
            day_of_week: 1, // Monday
            julian_day: 2460325.0,
            hindu_date: None,
        },
        location: Location {
            latitude: MOCK_LAT,
            longitude: MOCK_LNG,
            timezone: MOCK_TZONE,
            name: Some("Bangalore".to_string()),
        },
        tithi: Tithi {
            number: 5,
            name_tithi: TithiName::Panchami,
            start_time: "04:22".to_string(),
            end_time: "26:45".to_string(),
            is_complete: true,
        },
        nakshatra: Nakshatra {
            number: 8,
            name_nakshatra: NakshatraName::Pushya,
            pada: 2,
            start_time: "01:12".to_string(),
            end_time: "23:55".to_string(),
            longitude: 100.5,
        },
        yoga: Yoga {
            number: 4,
            name_yoga: YogaName::Saubhagya,
            start_time: "06:30".to_string(),
            end_time: "29:10".to_string(),
        },
        karana: Karana {
            name_karana: KaranaName::Bava,
            karana_type: KaranaType::Movable,
            start_time: "04:22".to_string(),
            end_time: "15:33".to_string(),
        },
        vara: Vara::Monday,
        paksha: Paksha::Shukla,
        planets: mock_planetary_positions(),
        day_boundaries: DayBoundaries {
            sunrise: "06:48".to_string(),
            sunset: "18:06".to_string(),
            next_sunrise: "06:48".to_string(),
            day_duration: "11:18".to_string(),
            night_duration: "12:42".to_string(),
        },
        ayanamsa: 24.1454,
    }
}

/// Create mock planetary positions for panchang
fn mock_planetary_positions() -> PlanetaryPositions {
    PlanetaryPositions {
        sun: PlanetPosition {
            name: "Sun".to_string(),
            longitude: 270.5,
            latitude: 0.0,
            speed: 1.02,
            sign: "Capricorn".to_string(),
            nakshatra: "Shravana".to_string(),
            pada: 3,
            is_retrograde: false,
        },
        moon: PlanetPosition {
            name: "Moon".to_string(),
            longitude: 100.5,
            latitude: -1.2,
            speed: 13.5,
            sign: "Cancer".to_string(),
            nakshatra: "Pushya".to_string(),
            pada: 2,
            is_retrograde: false,
        },
        mars: Some(PlanetPosition {
            name: "Mars".to_string(),
            longitude: 265.3,
            latitude: 0.5,
            speed: 0.65,
            sign: "Sagittarius".to_string(),
            nakshatra: "Purva Ashadha".to_string(),
            pada: 4,
            is_retrograde: false,
        }),
        mercury: Some(PlanetPosition {
            name: "Mercury".to_string(),
            longitude: 255.8,
            latitude: 1.1,
            speed: 1.5,
            sign: "Sagittarius".to_string(),
            nakshatra: "Mula".to_string(),
            pada: 2,
            is_retrograde: false,
        }),
        jupiter: Some(PlanetPosition {
            name: "Jupiter".to_string(),
            longitude: 8.2,
            latitude: -0.3,
            speed: 0.12,
            sign: "Aries".to_string(),
            nakshatra: "Ashwini".to_string(),
            pada: 3,
            is_retrograde: false,
        }),
        venus: Some(PlanetPosition {
            name: "Venus".to_string(),
            longitude: 240.1,
            latitude: 0.8,
            speed: 1.2,
            sign: "Scorpio".to_string(),
            nakshatra: "Jyeshtha".to_string(),
            pada: 1,
            is_retrograde: false,
        }),
        saturn: Some(PlanetPosition {
            name: "Saturn".to_string(),
            longitude: 326.7,
            latitude: 0.1,
            speed: 0.05,
            sign: "Aquarius".to_string(),
            nakshatra: "Shatabhisha".to_string(),
            pada: 4,
            is_retrograde: false,
        }),
        rahu: Some(PlanetPosition {
            name: "Rahu".to_string(),
            longitude: 15.4,
            latitude: 0.0,
            speed: -0.053,
            sign: "Aries".to_string(),
            nakshatra: "Bharani".to_string(),
            pada: 2,
            is_retrograde: true,
        }),
        ketu: Some(PlanetPosition {
            name: "Ketu".to_string(),
            longitude: 195.4,
            latitude: 0.0,
            speed: -0.053,
            sign: "Libra".to_string(),
            nakshatra: "Swati".to_string(),
            pada: 2,
            is_retrograde: true,
        }),
    }
}

/// Create a second, distinct Panchang for cache differentiation tests
pub fn mock_panchang_alternate() -> Panchang {
    let mut p = mock_panchang();
    p.date.year = 2024;
    p.date.month = 6;
    p.date.day = 21;
    p.date.day_of_week = 5; // Friday
    p.vara = Vara::Friday;
    p.tithi = Tithi {
        number: 14,
        name_tithi: TithiName::Chaturdashi,
        start_time: "03:10".to_string(),
        end_time: "25:40".to_string(),
        is_complete: true,
    };
    p.paksha = Paksha::Krishna;
    p
}

// ---------------------------------------------------------------------------
// Muhurta mocks
// ---------------------------------------------------------------------------

/// Create a realistic MuhurtaCollection for a Monday
pub fn mock_muhurta_collection() -> MuhurtaCollection {
    MuhurtaCollection {
        abhijit: Some(Muhurta {
            name: "Abhijit Muhurta".to_string(),
            start: "11:40".to_string(),
            end: "12:20".to_string(),
            duration_minutes: 40,
            nature: MuhurtaNature::Auspicious,
            ruler: "Mercury".to_string(),
            suitable_activities: vec!["All activities".to_string(), "New beginnings".to_string()],
            avoid_activities: vec![],
        }),
        amrit_kaal: Some(Muhurta {
            name: "Amrit Kaal".to_string(),
            start: "06:00".to_string(),
            end: "07:30".to_string(),
            duration_minutes: 90,
            nature: MuhurtaNature::Auspicious,
            ruler: "Moon".to_string(),
            suitable_activities: vec!["New ventures".to_string(), "Purchases".to_string()],
            avoid_activities: vec![],
        }),
        rahu_kalam: Some(Muhurta {
            name: "Rahu Kalam".to_string(),
            start: "07:30".to_string(),
            end: "09:00".to_string(),
            duration_minutes: 90,
            nature: MuhurtaNature::VeryInauspicious,
            ruler: "Rahu".to_string(),
            suitable_activities: vec!["Worship of Durga".to_string()],
            avoid_activities: vec![
                "New beginnings".to_string(),
                "Business ventures".to_string(),
            ],
        }),
        yama_gandam: Some(Muhurta {
            name: "Yama Gandam".to_string(),
            start: "10:30".to_string(),
            end: "12:00".to_string(),
            duration_minutes: 90,
            nature: MuhurtaNature::VeryInauspicious,
            ruler: "Yama".to_string(),
            suitable_activities: vec!["Charity".to_string()],
            avoid_activities: vec!["Important activities".to_string()],
        }),
        gulika_kaal: Some(Muhurta {
            name: "Gulika Kaal".to_string(),
            start: "13:30".to_string(),
            end: "15:00".to_string(),
            duration_minutes: 90,
            nature: MuhurtaNature::Inauspicious,
            ruler: "Gulika".to_string(),
            suitable_activities: vec![],
            avoid_activities: vec!["New ventures".to_string()],
        }),
        dur_muhurta: None,
        varjyam: None,
        brahma_muhurta: Some(Muhurta {
            name: "Brahma Muhurta".to_string(),
            start: "04:24".to_string(),
            end: "06:48".to_string(),
            duration_minutes: 96,
            nature: MuhurtaNature::Auspicious,
            ruler: "Brahma".to_string(),
            suitable_activities: vec!["Meditation".to_string(), "Spiritual practices".to_string()],
            avoid_activities: vec!["Sleep".to_string()],
        }),
    }
}

// ---------------------------------------------------------------------------
// Hora mocks
// ---------------------------------------------------------------------------

/// Create mock HoraTimings for a Monday
pub fn mock_hora_timings() -> HoraTimings {
    let day_planets = [
        HoraPlanet::Moon,
        HoraPlanet::Saturn,
        HoraPlanet::Jupiter,
        HoraPlanet::Mars,
        HoraPlanet::Sun,
        HoraPlanet::Venus,
        HoraPlanet::Mercury,
        HoraPlanet::Moon,
        HoraPlanet::Saturn,
        HoraPlanet::Jupiter,
        HoraPlanet::Mars,
        HoraPlanet::Sun,
    ];

    let night_planets = [
        HoraPlanet::Venus,
        HoraPlanet::Mercury,
        HoraPlanet::Moon,
        HoraPlanet::Saturn,
        HoraPlanet::Jupiter,
        HoraPlanet::Mars,
        HoraPlanet::Sun,
        HoraPlanet::Venus,
        HoraPlanet::Mercury,
        HoraPlanet::Moon,
        HoraPlanet::Saturn,
        HoraPlanet::Jupiter,
    ];

    let day_horas: Vec<Hora> = day_planets
        .iter()
        .enumerate()
        .map(|(i, planet)| Hora {
            number: (i + 1) as u8,
            ruler: *planet,
            start: format!("{:02}:00", 6 + i),
            end: format!("{:02}:00", 7 + i),
            is_favorable: !matches!(planet, HoraPlanet::Saturn),
            quality: format!("{} Hora", planet.as_str()),
        })
        .collect();

    let night_horas: Vec<Hora> = night_planets
        .iter()
        .enumerate()
        .map(|(i, planet)| Hora {
            number: (i + 13) as u8,
            ruler: *planet,
            start: format!("{:02}:00", 18 + i),
            end: format!("{:02}:00", 19 + i),
            is_favorable: !matches!(planet, HoraPlanet::Saturn),
            quality: format!("{} Hora", planet.as_str()),
        })
        .collect();

    HoraTimings {
        day_horas,
        night_horas,
        day_of_week: "Monday".to_string(),
        sunrise: "06:48".to_string(),
        sunset: "18:06".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Choghadiya mocks
// ---------------------------------------------------------------------------

/// Create mock ChoghadiyaTimings for a Monday
pub fn mock_choghadiya_timings() -> ChoghadiyaTimings {
    use ChoghadiyaName::*;
    use ChoghadiyaNature::*;

    let day_sequence = [
        (Amrit, Good, "Moon"),
        (Kaal, Bad, "Saturn"),
        (Shubh, Good, "Sun"),
        (Rog, Bad, "Mars"),
        (Udveg, Bad, "Mercury"),
        (Char, Medium, "Venus"),
        (Labh, Good, "Jupiter"),
        (Amrit, Good, "Moon"),
    ];

    let night_sequence = [
        (Amrit, Good, "Moon"),
        (Char, Medium, "Venus"),
        (Rog, Bad, "Mars"),
        (Kaal, Bad, "Saturn"),
        (Labh, Good, "Jupiter"),
        (Udveg, Bad, "Mercury"),
        (Shubh, Good, "Sun"),
        (Amrit, Good, "Moon"),
    ];

    let day: Vec<Choghadiya> = day_sequence
        .iter()
        .enumerate()
        .map(|(i, (name, nature, ruler))| Choghadiya {
            name: *name,
            start: format!("{:02}:00", 6 + i),
            end: format!("{:02}:30", 7 + i),
            duration_minutes: 90,
            nature: *nature,
            ruler: ruler.to_string(),
            suitable_for: crate::panchang::choghadiya::get_suitable_activities(*name),
            avoid: crate::panchang::choghadiya::get_activities_to_avoid(*name),
        })
        .collect();

    let night: Vec<Choghadiya> = night_sequence
        .iter()
        .enumerate()
        .map(|(i, (name, nature, ruler))| Choghadiya {
            name: *name,
            start: format!("{:02}:00", 18 + i),
            end: format!("{:02}:30", 19 + i),
            duration_minutes: 90,
            nature: *nature,
            ruler: ruler.to_string(),
            suitable_for: crate::panchang::choghadiya::get_suitable_activities(*name),
            avoid: crate::panchang::choghadiya::get_activities_to_avoid(*name),
        })
        .collect();

    ChoghadiyaTimings {
        day,
        night,
        day_of_week: "Monday".to_string(),
        sunrise: "06:48".to_string(),
        sunset: "18:06".to_string(),
    }
}

// ---------------------------------------------------------------------------
// CompletePanchang mock
// ---------------------------------------------------------------------------

/// Create a full CompletePanchang combining all sub-systems
pub fn mock_complete_panchang() -> CompletePanchang {
    CompletePanchang {
        panchang: mock_panchang(),
        muhurtas: mock_muhurta_collection(),
        hora_timings: mock_hora_timings(),
        choghadiya: mock_choghadiya_timings(),
        metadata: PanchangMetadata {
            source: "MockTestFactory".to_string(),
            calculated_at: "2024-01-15T12:00:00Z".to_string(),
            ayanamsa: "Lahiri".to_string(),
            timezone: MOCK_TZONE,
            dst_active: false,
        },
    }
}

// ---------------------------------------------------------------------------
// Vimshottari Dasha mocks
// ---------------------------------------------------------------------------

/// Create a realistic VimshottariDasha for a Moon-in-Pushya native
pub fn mock_vimshottari_dasha() -> VimshottariDasha {
    let current_maha = DashaPeriod {
        planet: DashaPlanet::Saturn,
        level: DashaLevel::Mahadasha,
        start_date: "2020-03-15".to_string(),
        end_date: "2039-03-15".to_string(),
        duration_years: 19.0,
        duration_days: 6940,
        sub_periods: None,
    };

    let current_antar = DashaPeriod {
        planet: DashaPlanet::Mercury,
        level: DashaLevel::Antardasha,
        start_date: "2023-06-01".to_string(),
        end_date: "2026-02-28".to_string(),
        duration_years: 2.72,
        duration_days: 993,
        sub_periods: None,
    };

    VimshottariDasha {
        birth_date: "1991-08-13".to_string(),
        moon_nakshatra: "Pushya".to_string(),
        moon_longitude: 100.5,
        balance: DashaBalance {
            planet: DashaPlanet::Saturn,
            years_remaining: 14.2,
            months_remaining: 2.4,
            days_remaining: 12.0,
            total_period_years: 19.0,
        },
        mahadashas: vec![
            DashaPeriod {
                planet: DashaPlanet::Saturn,
                level: DashaLevel::Mahadasha,
                start_date: "2020-03-15".to_string(),
                end_date: "2039-03-15".to_string(),
                duration_years: 19.0,
                duration_days: 6940,
                sub_periods: Some(vec![
                    DashaPeriod {
                        planet: DashaPlanet::Saturn,
                        level: DashaLevel::Antardasha,
                        start_date: "2020-03-15".to_string(),
                        end_date: "2023-03-18".to_string(),
                        duration_years: 3.0,
                        duration_days: 1098,
                        sub_periods: None,
                    },
                    current_antar.clone(),
                ]),
            },
            DashaPeriod {
                planet: DashaPlanet::Mercury,
                level: DashaLevel::Mahadasha,
                start_date: "2039-03-15".to_string(),
                end_date: "2056-03-15".to_string(),
                duration_years: 17.0,
                duration_days: 6210,
                sub_periods: None,
            },
            DashaPeriod {
                planet: DashaPlanet::Ketu,
                level: DashaLevel::Mahadasha,
                start_date: "2056-03-15".to_string(),
                end_date: "2063-03-15".to_string(),
                duration_years: 7.0,
                duration_days: 2557,
                sub_periods: None,
            },
        ],
        current_mahadasha: current_maha,
        current_antardasha: Some(current_antar),
        current_pratyantardasha: None,
        current_sookshma: None,
    }
}

/// Create a minimal dasha for edge-case testing (no sub-periods)
pub fn mock_vimshottari_dasha_minimal() -> VimshottariDasha {
    let maha = DashaPeriod {
        planet: DashaPlanet::Venus,
        level: DashaLevel::Mahadasha,
        start_date: "2010-01-01".to_string(),
        end_date: "2030-01-01".to_string(),
        duration_years: 20.0,
        duration_days: 7305,
        sub_periods: None,
    };

    VimshottariDasha {
        birth_date: "1990-06-15".to_string(),
        moon_nakshatra: "Bharani".to_string(),
        moon_longitude: 20.0,
        balance: DashaBalance {
            planet: DashaPlanet::Venus,
            years_remaining: 18.5,
            months_remaining: 6.0,
            days_remaining: 0.0,
            total_period_years: 20.0,
        },
        mahadashas: vec![maha.clone()],
        current_mahadasha: maha,
        current_antardasha: None,
        current_pratyantardasha: None,
        current_sookshma: None,
    }
}

// ---------------------------------------------------------------------------
// Birth Chart mocks
// ---------------------------------------------------------------------------

/// Create a realistic BirthChart with Aries ascendant
pub fn mock_birth_chart() -> BirthChart {
    BirthChart {
        native: NativeInfo {
            birth_date: "1991-08-13".to_string(),
            birth_time: "13:31:00".to_string(),
            latitude: MOCK_LAT,
            longitude: MOCK_LNG,
            timezone: MOCK_TZONE,
        },
        ayanamsa: 23.7512,
        house_system: "Placidus".to_string(),
        planets: vec![
            ChartPlanetPosition {
                name: "Sun".to_string(),
                longitude: 120.5,
                sign: ZodiacSign::Leo,
                degree: 0.5,
                minutes: 30.0,
                house: 5,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Magha".to_string(),
                pada: 1,
                speed: 0.98,
                latitude: 0.0,
            },
            ChartPlanetPosition {
                name: "Moon".to_string(),
                longitude: 100.5,
                sign: ZodiacSign::Cancer,
                degree: 10.5,
                minutes: 30.0,
                house: 4,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Pushya".to_string(),
                pada: 2,
                speed: 13.2,
                latitude: -1.1,
            },
            ChartPlanetPosition {
                name: "Mars".to_string(),
                longitude: 160.3,
                sign: ZodiacSign::Virgo,
                degree: 10.3,
                minutes: 18.0,
                house: 6,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Hasta".to_string(),
                pada: 1,
                speed: 0.7,
                latitude: 1.5,
            },
            ChartPlanetPosition {
                name: "Mercury".to_string(),
                longitude: 135.8,
                sign: ZodiacSign::Leo,
                degree: 15.8,
                minutes: 48.0,
                house: 5,
                is_retrograde: true,
                is_combust: false,
                nakshatra: "Purva Phalguni".to_string(),
                pada: 3,
                speed: -0.5,
                latitude: 2.1,
            },
            ChartPlanetPosition {
                name: "Jupiter".to_string(),
                longitude: 118.2,
                sign: ZodiacSign::Cancer,
                degree: 28.2,
                minutes: 12.0,
                house: 4,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Ashlesha".to_string(),
                pada: 4,
                speed: 0.13,
                latitude: -0.3,
            },
            ChartPlanetPosition {
                name: "Venus".to_string(),
                longitude: 110.1,
                sign: ZodiacSign::Cancer,
                degree: 20.1,
                minutes: 6.0,
                house: 4,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Ashlesha".to_string(),
                pada: 1,
                speed: 1.2,
                latitude: 0.8,
            },
            ChartPlanetPosition {
                name: "Saturn".to_string(),
                longitude: 306.7,
                sign: ZodiacSign::Aquarius,
                degree: 6.7,
                minutes: 42.0,
                house: 11,
                is_retrograde: true,
                is_combust: false,
                nakshatra: "Shatabhisha".to_string(),
                pada: 1,
                speed: -0.04,
                latitude: 0.1,
            },
        ],
        houses: (1..=12)
            .map(|n| HousePosition {
                number: n,
                sign: ZodiacSign::from_index((n as usize - 1) % 12),
                cusp: (n as f64 - 1.0) * 30.0 + 5.0,
                degree: 5.0,
                house_type: match n {
                    1 | 5 | 9 => HouseType::Dharma,
                    2 | 6 | 10 => HouseType::Artha,
                    3 | 7 | 11 => HouseType::Kama,
                    _ => HouseType::Moksha,
                },
                is_kendra: matches!(n, 1 | 4 | 7 | 10),
                is_panapara: matches!(n, 2 | 5 | 8 | 11),
                is_apoklima: matches!(n, 3 | 6 | 9 | 12),
            })
            .collect(),
        ascendant: AscendantInfo {
            sign: ZodiacSign::Aries,
            degree: 5.0,
            nakshatra: "Ashwini".to_string(),
            pada: 2,
        },
        moon: MoonInfo {
            sign: ZodiacSign::Cancer,
            degree: 10.5,
            nakshatra: "Pushya".to_string(),
            pada: 2,
            rashi_lord: "Moon".to_string(),
        },
        special_points: SpecialPoints {
            lagna: 5.0,
            midheaven: Some(275.0),
            part_of_fortune: Some(150.0),
        },
        notes: vec![
            "Jupiter exalted in Cancer".to_string(),
            "Mercury retrograde".to_string(),
        ],
    }
}

// ---------------------------------------------------------------------------
// Navamsa Chart mocks
// ---------------------------------------------------------------------------

/// Create a realistic NavamsaChart (D9)
pub fn mock_navamsa_chart() -> NavamsaChart {
    NavamsaChart {
        source: NativeInfo {
            birth_date: "1991-08-13".to_string(),
            birth_time: "13:31:00".to_string(),
            latitude: MOCK_LAT,
            longitude: MOCK_LNG,
            timezone: MOCK_TZONE,
        },
        navamsa_positions: vec![
            NavamsaPosition {
                planet: "Sun".to_string(),
                sign: ZodiacSign::Aries,
                degree: 1.5,
                is_vargottama: false,
            },
            NavamsaPosition {
                planet: "Moon".to_string(),
                sign: ZodiacSign::Virgo,
                degree: 15.0,
                is_vargottama: false,
            },
            NavamsaPosition {
                planet: "Mars".to_string(),
                sign: ZodiacSign::Aries,
                degree: 10.0,
                is_vargottama: false,
            },
            NavamsaPosition {
                planet: "Jupiter".to_string(),
                sign: ZodiacSign::Pisces,
                degree: 25.0,
                is_vargottama: false,
            },
        ],
        vargottama: vec!["Mars".to_string()],
        d9_lagna: ZodiacSign::Sagittarius,
    }
}

// ---------------------------------------------------------------------------
// Error scenario mocks
// ---------------------------------------------------------------------------

/// Create a rate limit error
pub fn mock_rate_limit_error() -> VedicApiError {
    VedicApiError::RateLimit {
        retry_after: Some(3600),
    }
}

/// Create a network error
pub fn mock_network_error() -> VedicApiError {
    VedicApiError::Network {
        message: "Connection refused: mock API server not reachable".to_string(),
    }
}

/// Create an API error (server-side)
pub fn mock_api_server_error() -> VedicApiError {
    VedicApiError::Api {
        status_code: 500,
        message: "Internal server error: mock failure".to_string(),
    }
}

/// Create a configuration error (missing API key)
pub fn mock_config_error() -> VedicApiError {
    VedicApiError::Configuration {
        field: "FREE_ASTROLOGY_API_KEY".to_string(),
        message: "API key not found in environment".to_string(),
    }
}

/// Create an invalid input error
pub fn mock_invalid_input_error() -> VedicApiError {
    VedicApiError::InvalidInput {
        field: "latitude".to_string(),
        message: "Latitude must be between -90 and 90".to_string(),
    }
}

/// Create a parse error
pub fn mock_parse_error() -> VedicApiError {
    VedicApiError::Parse {
        message: "Expected JSON object, got array".to_string(),
    }
}

/// Create a fallback failed error
pub fn mock_fallback_error() -> VedicApiError {
    VedicApiError::FallbackFailed {
        api_error: Box::new(mock_rate_limit_error()),
        native_error: "Native fallback not yet implemented".to_string(),
    }
}

/// Create a circuit breaker open error
pub fn mock_circuit_breaker_error() -> VedicApiError {
    VedicApiError::CircuitBreakerOpen
}

// ---------------------------------------------------------------------------
// Transit mocks (PR2: native facade — used for `MockApiClient::get_transits`
// and for downstream callers that need a stable Transit fixture without
// hitting Swiss Ephemeris).
// ---------------------------------------------------------------------------

use crate::transits::api::{
    JupiterTransitResponse, NatalAspectResponse, SadeSatiResponse, TransitApiResponse,
    TransitPlanetResponse,
};

/// Create a realistic mock `TransitApiResponse` matching the shape returned
/// by the native facade. Twelve planets, Sade-Sati populated, Jupiter
/// transit summary populated. Used by `MockApiClient::get_transits` and by
/// downstream consumers that don't want to spin up Swiss Ephemeris.
pub fn mock_transit_analysis() -> TransitApiResponse {
    let transits = vec![
        TransitPlanetResponse {
            planet: "Sun".to_string(),
            sign: "Capricorn".to_string(),
            degree: 0.5,
            is_retrograde: Some(false),
            natal_aspects: Some(vec![NatalAspectResponse {
                natal_planet: "Moon".to_string(),
                aspect_type: "Trine".to_string(),
                orb: 2.1,
            }]),
        },
        TransitPlanetResponse {
            planet: "Moon".to_string(),
            sign: "Cancer".to_string(),
            degree: 10.0,
            is_retrograde: Some(false),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Mercury".to_string(),
            sign: "Sagittarius".to_string(),
            degree: 25.0,
            is_retrograde: Some(false),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Venus".to_string(),
            sign: "Scorpio".to_string(),
            degree: 12.0,
            is_retrograde: Some(false),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Mars".to_string(),
            sign: "Sagittarius".to_string(),
            degree: 18.0,
            is_retrograde: Some(false),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Jupiter".to_string(),
            sign: "Aries".to_string(),
            degree: 5.0,
            is_retrograde: Some(false),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Saturn".to_string(),
            sign: "Aquarius".to_string(),
            degree: 22.0,
            is_retrograde: Some(false),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Uranus".to_string(),
            sign: "Taurus".to_string(),
            degree: 15.0,
            is_retrograde: Some(true),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Neptune".to_string(),
            sign: "Pisces".to_string(),
            degree: 24.0,
            is_retrograde: Some(false),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Pluto".to_string(),
            sign: "Capricorn".to_string(),
            degree: 28.0,
            is_retrograde: Some(false),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Rahu".to_string(),
            sign: "Aries".to_string(),
            degree: 14.0,
            is_retrograde: Some(true),
            natal_aspects: None,
        },
        TransitPlanetResponse {
            planet: "Ketu".to_string(),
            sign: "Libra".to_string(),
            degree: 14.0,
            is_retrograde: Some(true),
            natal_aspects: None,
        },
    ];

    TransitApiResponse {
        transits,
        sade_sati: Some(SadeSatiResponse {
            is_active: false,
            phase: None,
            saturn_sign: "Aquarius".to_string(),
            moon_sign: "Cancer".to_string(),
        }),
        jupiter_transit: Some(JupiterTransitResponse {
            sign: "Aries".to_string(),
            from_ascendant: 10,
            from_moon: 10,
            quality: Some("Challenging".to_string()),
        }),
    }
}

// ---------------------------------------------------------------------------
// JSON response mocks (for wiremock / HTTP-level testing)
// ---------------------------------------------------------------------------

/// Serialize a mock Panchang to JSON string (for HTTP mock servers)
pub fn mock_panchang_json() -> String {
    serde_json::to_string_pretty(&mock_panchang()).expect("Panchang serialization must succeed")
}

/// Serialize a mock VimshottariDasha to JSON string
pub fn mock_vimshottari_dasha_json() -> String {
    serde_json::to_string_pretty(&mock_vimshottari_dasha())
        .expect("Dasha serialization must succeed")
}

/// Serialize a mock BirthChart to JSON string
pub fn mock_birth_chart_json() -> String {
    serde_json::to_string_pretty(&mock_birth_chart())
        .expect("BirthChart serialization must succeed")
}

/// Serialize a mock NavamsaChart to JSON string
pub fn mock_navamsa_chart_json() -> String {
    serde_json::to_string_pretty(&mock_navamsa_chart())
        .expect("NavamsaChart serialization must succeed")
}

/// Create a mock API error JSON response
pub fn mock_error_json(status: u16, message: &str) -> String {
    serde_json::json!({
        "error": {
            "code": status,
            "message": message
        }
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Test helper: Config factory
// ---------------------------------------------------------------------------

/// Create a Config suitable for testing (points to localhost, short timeouts)
pub fn mock_config(base_url: &str) -> crate::Config {
    crate::Config {
        api_key: "test-mock-key-00000000000000000000".to_string(),
        base_url: base_url.to_string(),
        timeout_seconds: 5,
        retry_count: 1,
        cache_ttl_birth_data: 0,
        cache_ttl_daily: 86400,
        provider: crate::config::ProviderType::Api,
        fallback_enabled: false,
        rate_limit: crate::rate_limit::RateLimitConfig::default(),
    }
}

/// Create a Config with fallback enabled
pub fn mock_config_with_fallback(base_url: &str) -> crate::Config {
    let mut config = mock_config(base_url);
    config.fallback_enabled = true;
    config
}

// ---------------------------------------------------------------------------
// Module-level tests to verify mock data integrity
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_panchang_is_valid() {
        let p = mock_panchang();
        assert_eq!(p.date.year, 2024);
        assert_eq!(p.date.month, 1);
        assert_eq!(p.date.day, 15);
        assert_eq!(p.vara, Vara::Monday);
        assert_eq!(p.paksha, Paksha::Shukla);
        assert_eq!(p.tithi.name(), "Panchami");
        assert_eq!(p.nakshatra.name(), "Pushya");
        assert!(
            p.is_auspicious(),
            "Monday Shukla Panchami Pushya should be auspicious"
        );
    }

    #[test]
    fn test_mock_panchang_alternate_differs() {
        let p1 = mock_panchang();
        let p2 = mock_panchang_alternate();
        assert_ne!(p1.date.month, p2.date.month);
        assert_ne!(p1.vara, p2.vara);
    }

    #[test]
    fn test_mock_panchang_serialization_roundtrip() {
        let original = mock_panchang();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Panchang = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.date.year, original.date.year);
        assert_eq!(deserialized.tithi.number, original.tithi.number);
    }

    #[test]
    fn test_mock_dasha_is_valid() {
        let d = mock_vimshottari_dasha();
        assert_eq!(d.moon_nakshatra, "Pushya");
        assert_eq!(d.current_mahadasha.planet, DashaPlanet::Saturn);
        assert!(d.current_antardasha.is_some());
        assert!(!d.mahadashas.is_empty());
    }

    #[test]
    fn test_mock_dasha_minimal_has_no_subperiods() {
        let d = mock_vimshottari_dasha_minimal();
        assert!(d.current_antardasha.is_none());
        assert_eq!(d.mahadashas.len(), 1);
    }

    #[test]
    fn test_mock_birth_chart_is_valid() {
        let c = mock_birth_chart();
        assert_eq!(c.ascendant.sign, ZodiacSign::Aries);
        assert!(!c.planets.is_empty());
        assert_eq!(c.houses.len(), 12);
        assert!(c.get_planet("Sun").is_some());
        assert!(c.get_planet("Moon").is_some());
    }

    #[test]
    fn test_mock_navamsa_chart_is_valid() {
        let n = mock_navamsa_chart();
        assert_eq!(n.d9_lagna, ZodiacSign::Sagittarius);
        assert!(!n.navamsa_positions.is_empty());
        assert!(n.is_vargottama("Mars"));
        assert!(!n.is_vargottama("Sun"));
    }

    #[test]
    fn test_mock_complete_panchang_is_valid() {
        let cp = mock_complete_panchang();
        assert!(cp.muhurtas.abhijit.is_some());
        assert!(cp.muhurtas.rahu_kalam.is_some());
        assert!(!cp.hora_timings.day_horas.is_empty());
        assert!(!cp.choghadiya.day.is_empty());
        assert_eq!(cp.metadata.source, "MockTestFactory");
    }

    #[test]
    fn test_mock_hora_timings_valid() {
        let h = mock_hora_timings();
        assert_eq!(h.day_horas.len(), 12);
        assert_eq!(h.night_horas.len(), 12);
        assert_eq!(h.total_horas(), 24);
        assert_eq!(h.day_of_week, "Monday");
    }

    #[test]
    fn test_mock_choghadiya_timings_valid() {
        let c = mock_choghadiya_timings();
        assert_eq!(c.day.len(), 8);
        assert_eq!(c.night.len(), 8);
        assert_eq!(c.day_of_week, "Monday");
        let favorable = c.get_favorable();
        assert!(
            !favorable.is_empty(),
            "Should have at least one favorable choghadiya"
        );
    }

    #[test]
    fn test_mock_errors_are_distinct() {
        let errors: Vec<VedicApiError> = vec![
            mock_rate_limit_error(),
            mock_network_error(),
            mock_api_server_error(),
            mock_config_error(),
            mock_invalid_input_error(),
            mock_parse_error(),
            mock_fallback_error(),
            mock_circuit_breaker_error(),
        ];
        // Each error should have a unique Display string
        let displays: Vec<String> = errors.iter().map(|e| format!("{}", e)).collect();
        for (i, d1) in displays.iter().enumerate() {
            for (j, d2) in displays.iter().enumerate() {
                if i != j {
                    assert_ne!(d1, d2, "Error {} and {} should differ", i, j);
                }
            }
        }
    }

    #[test]
    fn test_mock_error_properties() {
        assert!(mock_rate_limit_error().is_retryable());
        assert!(mock_network_error().is_retryable());
        assert!(mock_api_server_error().is_retryable());
        assert!(!mock_config_error().is_retryable());
        assert!(!mock_invalid_input_error().is_retryable());
        assert!(!mock_parse_error().is_retryable());

        assert!(mock_rate_limit_error().should_fallback());
        assert!(mock_network_error().should_fallback());
        assert!(mock_circuit_breaker_error().should_fallback());
        assert!(!mock_config_error().should_fallback());
    }

    #[test]
    fn test_mock_json_serialization() {
        let panchang_json = mock_panchang_json();
        assert!(panchang_json.contains("Pushya"));
        assert!(panchang_json.contains("2024"));

        let dasha_json = mock_vimshottari_dasha_json();
        assert!(dasha_json.contains("saturn"));

        let chart_json = mock_birth_chart_json();
        assert!(chart_json.contains("aries"));

        let navamsa_json = mock_navamsa_chart_json();
        assert!(navamsa_json.contains("sagittarius"));
    }

    #[test]
    fn test_mock_error_json() {
        let json = mock_error_json(429, "Rate limit exceeded");
        assert!(json.contains("429"));
        assert!(json.contains("Rate limit"));
    }

    #[test]
    fn test_mock_config() {
        let config = mock_config("http://localhost:9999");
        assert_eq!(config.base_url, "http://localhost:9999");
        assert_eq!(config.timeout_seconds, 5);
        assert!(!config.fallback_enabled);
    }

    #[test]
    fn test_mock_config_with_fallback() {
        let config = mock_config_with_fallback("http://localhost:9999");
        assert!(config.fallback_enabled);
    }

    // PR3 — smoke tests for the four new analytical mock factories.

    #[test]
    fn pr3_mock_yoga_analysis_has_content() {
        let y = super::mock_yoga_analysis();
        assert!(
            !y.yogas.is_empty(),
            "mock yoga analysis must yield at least one yoga"
        );
        assert!(y.total_yoga_score > 0.0);
        assert!(!y.summary.is_empty());
    }

    #[test]
    fn pr3_mock_shadbala_analysis_all_seven_planets() {
        let s = super::mock_shadbala_analysis();
        assert_eq!(s.planets.len(), 7);
        for p in &s.planets {
            assert!(p.total_rupas > 0.0);
            assert_eq!(p.components.len(), 6);
        }
    }

    #[test]
    fn pr3_mock_ashtakavarga_analysis_sav_bounded() {
        let a = super::mock_ashtakavarga_analysis();
        assert_eq!(a.sarva_ashtakavarga.planets.len(), 7);
        assert!(a.sarva_ashtakavarga.grand_total <= 337);
        assert_eq!(a.strongest_signs.len(), 3);
    }

    #[test]
    fn pr3_mock_muhurta_results_quality_tiers() {
        let r = super::mock_muhurta_results();
        assert_eq!(r.muhurtas.len(), 3);
        assert_eq!(r.excellent_count, 1);
        assert_eq!(r.good_count, 1);
        assert!(r.from_date < r.to_date);
    }
}

// ---------------------------------------------------------------------------
// PR3 — mock factories for the four newly native analytical modules.
// These mirror realistic Bangalore 2024-01-15 fixtures so downstream tests
// can render full UIs without spinning up the native Swiss-Ephemeris path.
// ---------------------------------------------------------------------------

use crate::ashtakavarga::types::{
    AshtakavargaAnalysis, PlanetAshtakavarga, SarvaAshtakavarga, SignStrength, StrengthCategory,
};
use crate::muhurta::types::{MuhurtaActivity, MuhurtaQuality, MuhurtaResults, SelectedMuhurta};
use crate::shadbala::types::{
    ChartStrength, PlanetShadbala, ShadbalaAnalysis, ShadbalaComponent, ShadbalaValue,
};
use crate::yogas::types::{DetectedYoga, YogaAnalysis, YogaCategory, YogaStrength};

/// Mock yoga analysis (PR3) — three representative yogas across raj/dhana/
/// mahapurusha categories, plus the aggregated YogaAnalysis envelope.
pub fn mock_yoga_analysis() -> YogaAnalysis {
    let mut analysis = YogaAnalysis::empty();
    analysis.add_yoga(DetectedYoga {
        name: "Gaja Kesari Yoga".to_string(),
        category: YogaCategory::RajYoga,
        strength: YogaStrength::Full,
        planets_involved: vec!["Moon".to_string(), "Jupiter".to_string()],
        houses_involved: vec![1, 4],
        description: "Moon and Jupiter in mutual kendras".to_string(),
        results: "Fame, recognition, wisdom".to_string(),
        activation_periods: vec!["Moon Dasha".to_string(), "Jupiter Dasha".to_string()],
    });
    analysis.add_yoga(DetectedYoga {
        name: "Ruchaka Yoga".to_string(),
        category: YogaCategory::MahapurushaYoga,
        strength: YogaStrength::Full,
        planets_involved: vec!["Mars".to_string()],
        houses_involved: vec![10],
        description: "Mars in own sign in a kendra".to_string(),
        results: "Courage, leadership, success".to_string(),
        activation_periods: vec!["Mars Dasha".to_string()],
    });
    analysis.add_yoga(DetectedYoga {
        name: "Eleventh House Dhana Yoga".to_string(),
        category: YogaCategory::DhanaYoga,
        strength: YogaStrength::Partial,
        planets_involved: vec!["Venus".to_string()],
        houses_involved: vec![11],
        description: "Strong 11th house indicating gains".to_string(),
        results: "Income from multiple sources".to_string(),
        activation_periods: vec!["Venus Dasha".to_string()],
    });
    analysis.calculate_score();
    analysis.generate_summary();
    analysis
}

/// Mock Shadbala analysis (PR3) — populated with plausible rupas/ratios for
/// every classical planet so dashboards can render bars without running
/// Swiss Ephemeris.
pub fn mock_shadbala_analysis() -> ShadbalaAnalysis {
    fn p(name: &str, total: f64, required: f64) -> PlanetShadbala {
        // Distribute total roughly evenly across the six components.
        let share = total / 6.0;
        PlanetShadbala {
            planet: name.to_string(),
            components: vec![
                ShadbalaValue {
                    component: ShadbalaComponent::SthanaBala,
                    rupas: share,
                    shashtiamsas: share,
                },
                ShadbalaValue {
                    component: ShadbalaComponent::DigBala,
                    rupas: share,
                    shashtiamsas: share,
                },
                ShadbalaValue {
                    component: ShadbalaComponent::KalaBala,
                    rupas: share,
                    shashtiamsas: share,
                },
                ShadbalaValue {
                    component: ShadbalaComponent::ChestaBala,
                    rupas: share,
                    shashtiamsas: share,
                },
                ShadbalaValue {
                    component: ShadbalaComponent::NaisargikaBala,
                    rupas: share,
                    shashtiamsas: share,
                },
                ShadbalaValue {
                    component: ShadbalaComponent::DrikBala,
                    rupas: share,
                    shashtiamsas: share,
                },
            ],
            total_rupas: total,
            total_shashtiamsas: total,
            required_minimum: required,
            strength_ratio: total / required,
            is_strong: total >= required,
        }
    }
    let planets = vec![
        p("Sun", 420.0, 390.0),
        p("Moon", 380.0, 360.0),
        p("Mars", 310.0, 300.0),
        p("Mercury", 450.0, 420.0),
        p("Jupiter", 400.0, 390.0),
        p("Venus", 350.0, 330.0),
        p("Saturn", 290.0, 300.0),
    ];
    ShadbalaAnalysis {
        strongest_planet: "Mercury".to_string(),
        weakest_planet: "Saturn".to_string(),
        chart_strength: ChartStrength::Strong,
        planets,
    }
}

/// Mock Ashtakavarga analysis (PR3) — uniformly seeded BAV per planet,
/// SAV grand total well under the 337 classical max, and pre-computed
/// sign-strength rankings so dashboards have rendering data.
pub fn mock_ashtakavarga_analysis() -> AshtakavargaAnalysis {
    let mut sarva = SarvaAshtakavarga::empty();
    let template = [4u8, 5, 3, 4, 5, 3, 4, 5, 4, 4, 3, 4];
    for name in [
        "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
    ] {
        let mut av = PlanetAshtakavarga::empty(name);
        av.sign_points = template;
        av.recalculate_total();
        sarva.add_planet(av);
    }
    let sign_names = [
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpio",
        "Sagittarius",
        "Capricorn",
        "Aquarius",
        "Pisces",
    ];
    let strongest = sign_names
        .iter()
        .enumerate()
        .map(|(i, n)| SignStrength {
            sign: (i + 1) as u8,
            sign_name: n.to_string(),
            points: sarva.sarva_points[i],
            category: StrengthCategory::from_sarva_points(sarva.sarva_points[i]),
        })
        .take(3)
        .collect::<Vec<_>>();
    let weakest = sign_names
        .iter()
        .enumerate()
        .rev()
        .map(|(i, n)| SignStrength {
            sign: (i + 1) as u8,
            sign_name: n.to_string(),
            points: sarva.sarva_points[i],
            category: StrengthCategory::from_sarva_points(sarva.sarva_points[i]),
        })
        .take(3)
        .collect::<Vec<_>>();
    AshtakavargaAnalysis {
        sarva_ashtakavarga: sarva,
        strongest_signs: strongest,
        weakest_signs: weakest,
        transit_recommendations: vec![
            "Favourable transits expected through Taurus and Leo.".to_string(),
            "Use caution during Pisces transits.".to_string(),
        ],
    }
}

/// Mock muhurta results (PR3) — three slots spread over a 7-day Bangalore
/// search, one Excellent + one Good + one Average so dashboards can render
/// every quality tier.
pub fn mock_muhurta_results() -> MuhurtaResults {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    let make_slot =
        |y: i32, m: u32, d: u32, h: u32, q: MuhurtaQuality, score: u8| SelectedMuhurta {
            start_time: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(y, m, d).unwrap(),
                NaiveTime::from_hms_opt(h, 0, 0).unwrap(),
            ),
            end_time: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(y, m, d).unwrap(),
                NaiveTime::from_hms_opt(h + 1, 0, 0).unwrap(),
            ),
            quality: q,
            tithi: "Panchami (Shukla)".to_string(),
            nakshatra: "Rohini".to_string(),
            yoga: "Siddhi".to_string(),
            karana: "Bava".to_string(),
            vara: "Thursday".to_string(),
            score,
            favorable_factors: vec!["Rohini nakshatra is very auspicious".to_string()],
            unfavorable_factors: vec![],
            recommendation: format!("{} muhurta at {:02}:00", q, h),
        };
    let muhurtas = vec![
        make_slot(2024, 5, 2, 9, MuhurtaQuality::Excellent, 85),
        make_slot(2024, 5, 4, 11, MuhurtaQuality::Good, 70),
        make_slot(2024, 5, 6, 15, MuhurtaQuality::Average, 55),
    ];
    let excellent = muhurtas
        .iter()
        .filter(|m| m.quality == MuhurtaQuality::Excellent)
        .count();
    let good = muhurtas
        .iter()
        .filter(|m| m.quality == MuhurtaQuality::Good)
        .count();
    MuhurtaResults {
        activity: MuhurtaActivity::General,
        from_date: chrono::NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(),
        to_date: chrono::NaiveDate::from_ymd_opt(2024, 5, 7).unwrap(),
        muhurtas,
        excellent_count: excellent,
        good_count: good,
        advice: "Three favourable slots identified across the window.".to_string(),
    }
}
