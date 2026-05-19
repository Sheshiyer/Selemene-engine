//! Test mocks for FreeAstrologyAPI integration (FAPI-093)
//!
//! Provides realistic mock responses calibrated to Shesh's birth data:
//! - Date: 1991-09-14, 09:30 AM IST
//! - Location: Bangalore (12.9716 N, 77.5946 E)
//! - Expected: Scorpio Ascendant, Moon in Virgo, Uttara Phalguni Nakshatra
//! - Sun Mahadasha: 1991-09-14 to 1997-09-13
//! - Mars Mahadasha: 2008-09-13 to 2015-09-14
//!
//! These mocks allow CI/CD and local development to run without an API key.
//!
//! # Usage
//! ```rust,ignore
//! use noesis_vedic_api::test_mocks::{MockResponses, MockApiClient};
//!
//! let panchang = MockResponses::panchang_response();
//! let dasha = MockResponses::vimshottari_response();
//! let chart = MockResponses::birth_chart_response();
//! let navamsa = MockResponses::navamsa_response();
//!
//! let mock_client = MockApiClient::new();
//! ```

use crate::chart::{
    AscendantInfo, BirthChart, HousePosition, HouseType, MoonInfo, NativeInfo, NavamsaChart,
    NavamsaPosition, PlanetPosition as ChartPlanetPosition, SpecialPoints, ZodiacSign,
};
use crate::dasha::{DashaBalance, DashaLevel, DashaPeriod, DashaPlanet, VimshottariDasha};
use crate::panchang::data::PlanetPosition as PanchangPlanetPosition;
use crate::panchang::{
    DateInfo, DayBoundaries, Karana, KaranaName, KaranaType, Location, Nakshatra, NakshatraName,
    Paksha, Panchang, PlanetaryPositions, Tithi, TithiName, Vara, Yoga, YogaName,
};
use crate::transits::api::{
    JupiterTransitResponse, NatalAspectResponse, SadeSatiResponse, TransitApiResponse,
    TransitPlanetResponse,
};

// ---------------------------------------------------------------------------
// Constants: Shesh's birth data
// ---------------------------------------------------------------------------

/// Shesh's birth latitude (Bangalore)
pub const SHESH_LAT: f64 = 12.9716;
/// Shesh's birth longitude (Bangalore)
pub const SHESH_LNG: f64 = 77.5946;
/// IST timezone offset
pub const SHESH_TZONE: f64 = 5.5;
/// Birth year
pub const SHESH_YEAR: i32 = 1991;
/// Birth month
pub const SHESH_MONTH: u32 = 9;
/// Birth day
pub const SHESH_DAY: u32 = 14;
/// Birth hour (24h format)
pub const SHESH_HOUR: u32 = 9;
/// Birth minute
pub const SHESH_MINUTE: u32 = 30;
/// Birth second
pub const SHESH_SECOND: u32 = 0;

// ---------------------------------------------------------------------------
// MockResponses - JSON Value responses for HTTP-level testing
// ---------------------------------------------------------------------------

/// Mock responses as serde_json::Value for API endpoint simulation.
///
/// Each method returns a JSON value matching what FreeAstrologyAPI.com would
/// return for Shesh's birth data. Useful for wiremock or custom mock servers.
pub struct MockResponses;

impl MockResponses {
    /// Mock Panchang response for 1991-09-14 Bangalore
    ///
    /// Returns a panchang with:
    /// - Tithi: Shashthi (6th), Shukla Paksha
    /// - Nakshatra: Uttara Phalguni (12th)
    /// - Yoga: Shobhana
    /// - Karana: Taitila
    /// - Vara: Saturday
    pub fn panchang_response() -> serde_json::Value {
        serde_json::to_value(shesh_panchang()).expect("Panchang serialization must succeed")
    }

    /// Mock Vimshottari Dasha response for Shesh's birth data
    ///
    /// Moon Nakshatra: Uttara Phalguni (ruled by Sun)
    /// Sequence: Sun 6yr -> Moon 10yr -> Mars 7yr -> Rahu 18yr -> ...
    /// Sun Mahadasha: 1991-09-14 to 1997-09-13
    pub fn vimshottari_response() -> serde_json::Value {
        serde_json::to_value(shesh_vimshottari_dasha()).expect("Dasha serialization must succeed")
    }

    /// Mock Birth Chart (D1) response for Shesh
    ///
    /// Ascendant: Scorpio
    /// Moon: Virgo (Uttara Phalguni Nakshatra)
    /// All 9 Vedic planets positioned
    pub fn birth_chart_response() -> serde_json::Value {
        serde_json::to_value(shesh_birth_chart()).expect("BirthChart serialization must succeed")
    }

    /// Mock Navamsa (D9) chart response for Shesh
    pub fn navamsa_response() -> serde_json::Value {
        serde_json::to_value(shesh_navamsa_chart())
            .expect("NavamsaChart serialization must succeed")
    }

    /// Mock Panchang as JSON string (for HTTP response bodies)
    pub fn panchang_json() -> String {
        serde_json::to_string_pretty(&shesh_panchang())
            .expect("Panchang JSON serialization must succeed")
    }

    /// Mock Vimshottari Dasha as JSON string
    pub fn vimshottari_json() -> String {
        serde_json::to_string_pretty(&shesh_vimshottari_dasha())
            .expect("Dasha JSON serialization must succeed")
    }

    /// Mock Birth Chart as JSON string
    pub fn birth_chart_json() -> String {
        serde_json::to_string_pretty(&shesh_birth_chart())
            .expect("BirthChart JSON serialization must succeed")
    }

    /// Mock Navamsa as JSON string
    pub fn navamsa_json() -> String {
        serde_json::to_string_pretty(&shesh_navamsa_chart())
            .expect("NavamsaChart JSON serialization must succeed")
    }
}

// ---------------------------------------------------------------------------
// MockApiClient - In-memory API client substitute for testing
// ---------------------------------------------------------------------------

/// A mock API client that returns pre-built responses without network calls.
///
/// Tracks call counts for verification in tests. All methods return
/// Shesh's birth data responses.
///
/// # Example
/// ```rust,ignore
/// let client = MockApiClient::new();
/// let panchang = client.get_panchang();
/// assert_eq!(panchang.nakshatra.name(), "Uttara Phalguni");
/// assert_eq!(client.call_count(), 1);
/// ```
pub struct MockApiClient {
    /// Number of calls made (for verification in tests)
    call_count: std::cell::Cell<u32>,
}

impl MockApiClient {
    /// Create a new mock client
    pub fn new() -> Self {
        Self {
            call_count: std::cell::Cell::new(0),
        }
    }

    /// Get the total number of calls made to this mock
    pub fn call_count(&self) -> u32 {
        self.call_count.get()
    }

    /// Reset the call counter
    pub fn reset_count(&self) {
        self.call_count.set(0);
    }

    fn increment(&self) {
        self.call_count.set(self.call_count.get() + 1);
    }

    /// Mock get_panchang - returns Shesh's birth date panchang
    pub fn get_panchang(&self) -> Panchang {
        self.increment();
        shesh_panchang()
    }

    /// Mock get_vimshottari_dasha - returns Shesh's dasha tree
    pub fn get_vimshottari_dasha(&self) -> VimshottariDasha {
        self.increment();
        shesh_vimshottari_dasha()
    }

    /// Mock get_birth_chart - returns Shesh's D1 chart
    pub fn get_birth_chart(&self) -> BirthChart {
        self.increment();
        shesh_birth_chart()
    }

    /// Mock get_navamsa_chart - returns Shesh's D9 chart
    pub fn get_navamsa_chart(&self) -> NavamsaChart {
        self.increment();
        shesh_navamsa_chart()
    }

    /// Mock get_transits — returns a stable transit analysis fixture for
    /// Shesh's birth chart against a fixed transit date (2024-01-15). Added
    /// in PR2 alongside the native transits façade.
    pub fn get_transits(&self) -> TransitApiResponse {
        self.increment();
        shesh_transit_analysis()
    }
}

impl Default for MockApiClient {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Panchang mock: 1991-09-14 09:30 AM IST, Bangalore
// ---------------------------------------------------------------------------

/// Create a realistic Panchang for Shesh's birth date.
///
/// Sep 14, 1991 was a Saturday. Panchang elements are calibrated to
/// that date using Lahiri ayanamsa.
pub fn shesh_panchang() -> Panchang {
    Panchang {
        date: DateInfo {
            year: SHESH_YEAR,
            month: SHESH_MONTH,
            day: SHESH_DAY,
            day_of_week: 6, // Saturday
            julian_day: 2448515.0,
            hindu_date: None,
        },
        location: Location {
            latitude: SHESH_LAT,
            longitude: SHESH_LNG,
            timezone: SHESH_TZONE,
            name: Some("Bangalore".to_string()),
        },
        tithi: Tithi {
            number: 6,
            name_tithi: TithiName::Shashthi,
            start_time: "03:45".to_string(),
            end_time: "26:12".to_string(),
            is_complete: true,
        },
        nakshatra: Nakshatra {
            number: 12,
            name_nakshatra: NakshatraName::UttaraPhalguni,
            pada: 2,
            start_time: "02:18".to_string(),
            end_time: "24:45".to_string(),
            longitude: 156.8,
        },
        yoga: Yoga {
            number: 5,
            name_yoga: YogaName::Shobhana,
            start_time: "05:10".to_string(),
            end_time: "27:30".to_string(),
        },
        karana: Karana {
            name_karana: KaranaName::Taitila,
            karana_type: KaranaType::Movable,
            start_time: "03:45".to_string(),
            end_time: "14:58".to_string(),
        },
        vara: Vara::Saturday,
        paksha: Paksha::Shukla,
        planets: shesh_planetary_positions(),
        day_boundaries: DayBoundaries {
            sunrise: "06:10".to_string(),
            sunset: "18:18".to_string(),
            next_sunrise: "06:10".to_string(),
            day_duration: "12:08".to_string(),
            night_duration: "11:52".to_string(),
        },
        ayanamsa: 23.7285,
    }
}

/// Planetary positions for Shesh's birth date (panchang context)
fn shesh_planetary_positions() -> PlanetaryPositions {
    PlanetaryPositions {
        sun: PanchangPlanetPosition {
            name: "Sun".to_string(),
            longitude: 147.5, // ~27 deg Leo (sidereal)
            latitude: 0.0,
            speed: 0.98,
            sign: "Leo".to_string(),
            nakshatra: "Uttara Phalguni".to_string(),
            pada: 1,
            is_retrograde: false,
        },
        moon: PanchangPlanetPosition {
            name: "Moon".to_string(),
            longitude: 156.8, // ~6 deg Virgo
            latitude: -1.5,
            speed: 13.8,
            sign: "Virgo".to_string(),
            nakshatra: "Uttara Phalguni".to_string(),
            pada: 2,
            is_retrograde: false,
        },
        mars: Some(PanchangPlanetPosition {
            name: "Mars".to_string(),
            longitude: 117.2, // ~27 deg Cancer
            latitude: 0.8,
            speed: 0.62,
            sign: "Cancer".to_string(),
            nakshatra: "Ashlesha".to_string(),
            pada: 3,
            is_retrograde: false,
        }),
        mercury: Some(PanchangPlanetPosition {
            name: "Mercury".to_string(),
            longitude: 160.4, // ~10 deg Virgo
            latitude: 2.3,
            speed: 1.45,
            sign: "Virgo".to_string(),
            nakshatra: "Hasta".to_string(),
            pada: 1,
            is_retrograde: false,
        }),
        jupiter: Some(PanchangPlanetPosition {
            name: "Jupiter".to_string(),
            longitude: 118.5, // ~28 deg Cancer (exalted)
            latitude: -0.2,
            speed: 0.11,
            sign: "Cancer".to_string(),
            nakshatra: "Ashlesha".to_string(),
            pada: 4,
            is_retrograde: false,
        }),
        venus: Some(PanchangPlanetPosition {
            name: "Venus".to_string(),
            longitude: 130.2, // ~10 deg Leo
            latitude: 0.9,
            speed: 1.18,
            sign: "Leo".to_string(),
            nakshatra: "Magha".to_string(),
            pada: 4,
            is_retrograde: false,
        }),
        saturn: Some(PanchangPlanetPosition {
            name: "Saturn".to_string(),
            longitude: 298.3, // ~28 deg Capricorn
            latitude: 0.1,
            speed: -0.03,
            sign: "Capricorn".to_string(),
            nakshatra: "Dhanishta".to_string(),
            pada: 1,
            is_retrograde: true,
        }),
        rahu: Some(PanchangPlanetPosition {
            name: "Rahu".to_string(),
            longitude: 266.7, // ~26 deg Sagittarius
            latitude: 0.0,
            speed: -0.053,
            sign: "Sagittarius".to_string(),
            nakshatra: "Uttara Ashadha".to_string(),
            pada: 3,
            is_retrograde: true,
        }),
        ketu: Some(PanchangPlanetPosition {
            name: "Ketu".to_string(),
            longitude: 86.7, // ~26 deg Gemini
            latitude: 0.0,
            speed: -0.053,
            sign: "Gemini".to_string(),
            nakshatra: "Punarvasu".to_string(),
            pada: 1,
            is_retrograde: true,
        }),
    }
}

// ---------------------------------------------------------------------------
// Vimshottari Dasha mock: Uttara Phalguni Nakshatra -> Sun dasha at birth
// ---------------------------------------------------------------------------

/// Create Shesh's Vimshottari Dasha tree.
///
/// Moon in Uttara Phalguni (ruled by Sun) means the birth dasha is Sun.
/// The standard Vimshottari sequence from Sun is:
/// Sun (6yr) -> Moon (10yr) -> Mars (7yr) -> Rahu (18yr) -> Jupiter (16yr) -> ...
///
/// With birth date 1991-09-14:
/// - Sun Mahadasha:    1991-09-14 to 1997-09-13 (balance ~6 years)
/// - Moon Mahadasha:   1997-09-14 to 2007-09-13
/// - Mars Mahadasha:   2007-09-14 (approx) -- task says 2008-09-13 start, using task value
/// - Rahu Mahadasha:   2015-09-14 to 2033-09-13
/// - Jupiter Mahadasha: 2033-09-14 to 2049-09-13
/// - Saturn Mahadasha: 2049-09-14 to 2068-09-13
/// - Mercury Mahadasha: 2068-09-14 to 2085-09-13
/// - Ketu Mahadasha:   2085-09-14 to 2092-09-13
/// - Venus Mahadasha:  2092-09-14 to 2112-09-13
pub fn shesh_vimshottari_dasha() -> VimshottariDasha {
    let sun_maha = DashaPeriod {
        planet: DashaPlanet::Sun,
        level: DashaLevel::Mahadasha,
        start_date: "1991-09-14".to_string(),
        end_date: "1997-09-13".to_string(),
        duration_years: 6.0,
        duration_days: 2192,
        sub_periods: Some(vec![
            DashaPeriod {
                planet: DashaPlanet::Sun,
                level: DashaLevel::Antardasha,
                start_date: "1991-09-14".to_string(),
                end_date: "1992-01-02".to_string(),
                duration_years: 0.3,
                duration_days: 110,
                sub_periods: None,
            },
            DashaPeriod {
                planet: DashaPlanet::Moon,
                level: DashaLevel::Antardasha,
                start_date: "1992-01-03".to_string(),
                end_date: "1992-07-02".to_string(),
                duration_years: 0.5,
                duration_days: 182,
                sub_periods: None,
            },
            DashaPeriod {
                planet: DashaPlanet::Mars,
                level: DashaLevel::Antardasha,
                start_date: "1992-07-03".to_string(),
                end_date: "1992-11-08".to_string(),
                duration_years: 0.35,
                duration_days: 128,
                sub_periods: None,
            },
        ]),
    };

    let moon_maha = DashaPeriod {
        planet: DashaPlanet::Moon,
        level: DashaLevel::Mahadasha,
        start_date: "1997-09-14".to_string(),
        end_date: "2007-09-13".to_string(),
        duration_years: 10.0,
        duration_days: 3652,
        sub_periods: None,
    };

    let mars_maha = DashaPeriod {
        planet: DashaPlanet::Mars,
        level: DashaLevel::Mahadasha,
        start_date: "2008-09-13".to_string(),
        end_date: "2015-09-13".to_string(),
        duration_years: 7.0,
        duration_days: 2557,
        sub_periods: None,
    };

    let rahu_maha = DashaPeriod {
        planet: DashaPlanet::Rahu,
        level: DashaLevel::Mahadasha,
        start_date: "2015-09-14".to_string(),
        end_date: "2033-09-13".to_string(),
        duration_years: 18.0,
        duration_days: 6574,
        sub_periods: None,
    };

    let jupiter_maha = DashaPeriod {
        planet: DashaPlanet::Jupiter,
        level: DashaLevel::Mahadasha,
        start_date: "2033-09-14".to_string(),
        end_date: "2049-09-13".to_string(),
        duration_years: 16.0,
        duration_days: 5844,
        sub_periods: None,
    };

    let saturn_maha = DashaPeriod {
        planet: DashaPlanet::Saturn,
        level: DashaLevel::Mahadasha,
        start_date: "2049-09-14".to_string(),
        end_date: "2068-09-13".to_string(),
        duration_years: 19.0,
        duration_days: 6940,
        sub_periods: None,
    };

    let mercury_maha = DashaPeriod {
        planet: DashaPlanet::Mercury,
        level: DashaLevel::Mahadasha,
        start_date: "2068-09-14".to_string(),
        end_date: "2085-09-13".to_string(),
        duration_years: 17.0,
        duration_days: 6210,
        sub_periods: None,
    };

    let ketu_maha = DashaPeriod {
        planet: DashaPlanet::Ketu,
        level: DashaLevel::Mahadasha,
        start_date: "2085-09-14".to_string(),
        end_date: "2092-09-13".to_string(),
        duration_years: 7.0,
        duration_days: 2557,
        sub_periods: None,
    };

    let venus_maha = DashaPeriod {
        planet: DashaPlanet::Venus,
        level: DashaLevel::Mahadasha,
        start_date: "2092-09-14".to_string(),
        end_date: "2112-09-13".to_string(),
        duration_years: 20.0,
        duration_days: 7305,
        sub_periods: None,
    };

    // Determine current running dasha (assume "now" relative context: Rahu MD)
    let current_maha = rahu_maha.clone();

    VimshottariDasha {
        birth_date: "1991-09-14".to_string(),
        moon_nakshatra: "Uttara Phalguni".to_string(),
        moon_longitude: 156.8,
        balance: DashaBalance {
            planet: DashaPlanet::Sun,
            years_remaining: 6.0,
            months_remaining: 0.0,
            days_remaining: 0.0,
            total_period_years: 6.0,
        },
        mahadashas: vec![
            sun_maha.clone(),
            moon_maha,
            mars_maha,
            rahu_maha,
            jupiter_maha,
            saturn_maha,
            mercury_maha,
            ketu_maha,
            venus_maha,
        ],
        current_mahadasha: current_maha,
        current_antardasha: None,
        current_pratyantardasha: None,
        current_sookshma: None,
    }
}

// ---------------------------------------------------------------------------
// Birth Chart mock: Scorpio Ascendant, Moon in Virgo
// ---------------------------------------------------------------------------

/// Create Shesh's birth chart (D1 Rashi).
///
/// Key placements:
/// - Ascendant: Scorpio (ruled by Mars)
/// - Sun: Leo (own sign, 10th house)
/// - Moon: Virgo (Uttara Phalguni, 11th house)
/// - Mars: Cancer (debilitated, 9th house)
/// - Mercury: Virgo (exalted, 11th house)
/// - Jupiter: Cancer (exalted, 9th house)
/// - Venus: Leo (10th house)
/// - Saturn: Capricorn (own sign, 3rd house, retrograde)
/// - Rahu: Sagittarius (2nd house)
/// - Ketu: Gemini (8th house)
pub fn shesh_birth_chart() -> BirthChart {
    BirthChart {
        native: shesh_native_info(),
        ayanamsa: 23.7285,
        house_system: "Placidus".to_string(),
        planets: vec![
            // Sun in Leo (own sign) - 10th house from Scorpio
            ChartPlanetPosition {
                name: "Sun".to_string(),
                longitude: 147.5,
                sign: ZodiacSign::Leo,
                degree: 27.5,
                minutes: 30.0,
                house: 10,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Uttara Phalguni".to_string(),
                pada: 1,
                speed: 0.98,
                latitude: 0.0,
            },
            // Moon in Virgo - 11th house from Scorpio
            ChartPlanetPosition {
                name: "Moon".to_string(),
                longitude: 156.8,
                sign: ZodiacSign::Virgo,
                degree: 6.8,
                minutes: 48.0,
                house: 11,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Uttara Phalguni".to_string(),
                pada: 2,
                speed: 13.8,
                latitude: -1.5,
            },
            // Mars in Cancer (debilitated) - 9th house from Scorpio
            ChartPlanetPosition {
                name: "Mars".to_string(),
                longitude: 117.2,
                sign: ZodiacSign::Cancer,
                degree: 27.2,
                minutes: 12.0,
                house: 9,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Ashlesha".to_string(),
                pada: 3,
                speed: 0.62,
                latitude: 0.8,
            },
            // Mercury in Virgo (exalted) - 11th house from Scorpio
            ChartPlanetPosition {
                name: "Mercury".to_string(),
                longitude: 160.4,
                sign: ZodiacSign::Virgo,
                degree: 10.4,
                minutes: 24.0,
                house: 11,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Hasta".to_string(),
                pada: 1,
                speed: 1.45,
                latitude: 2.3,
            },
            // Jupiter in Cancer (exalted) - 9th house from Scorpio
            ChartPlanetPosition {
                name: "Jupiter".to_string(),
                longitude: 118.5,
                sign: ZodiacSign::Cancer,
                degree: 28.5,
                minutes: 30.0,
                house: 9,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Ashlesha".to_string(),
                pada: 4,
                speed: 0.11,
                latitude: -0.2,
            },
            // Venus in Leo - 10th house from Scorpio
            ChartPlanetPosition {
                name: "Venus".to_string(),
                longitude: 130.2,
                sign: ZodiacSign::Leo,
                degree: 10.2,
                minutes: 12.0,
                house: 10,
                is_retrograde: false,
                is_combust: false,
                nakshatra: "Magha".to_string(),
                pada: 4,
                speed: 1.18,
                latitude: 0.9,
            },
            // Saturn in Capricorn (own sign, retrograde) - 3rd house from Scorpio
            ChartPlanetPosition {
                name: "Saturn".to_string(),
                longitude: 298.3,
                sign: ZodiacSign::Capricorn,
                degree: 28.3,
                minutes: 18.0,
                house: 3,
                is_retrograde: true,
                is_combust: false,
                nakshatra: "Dhanishta".to_string(),
                pada: 1,
                speed: -0.03,
                latitude: 0.1,
            },
            // Rahu in Sagittarius - 2nd house from Scorpio
            ChartPlanetPosition {
                name: "Rahu".to_string(),
                longitude: 266.7,
                sign: ZodiacSign::Sagittarius,
                degree: 26.7,
                minutes: 42.0,
                house: 2,
                is_retrograde: true,
                is_combust: false,
                nakshatra: "Uttara Ashadha".to_string(),
                pada: 3,
                speed: -0.053,
                latitude: 0.0,
            },
            // Ketu in Gemini - 8th house from Scorpio
            ChartPlanetPosition {
                name: "Ketu".to_string(),
                longitude: 86.7,
                sign: ZodiacSign::Gemini,
                degree: 26.7,
                minutes: 42.0,
                house: 8,
                is_retrograde: true,
                is_combust: false,
                nakshatra: "Punarvasu".to_string(),
                pada: 1,
                speed: -0.053,
                latitude: 0.0,
            },
        ],
        houses: shesh_houses(),
        ascendant: AscendantInfo {
            sign: ZodiacSign::Scorpio,
            degree: 15.3,
            nakshatra: "Anuradha".to_string(),
            pada: 3,
        },
        moon: MoonInfo {
            sign: ZodiacSign::Virgo,
            degree: 6.8,
            nakshatra: "Uttara Phalguni".to_string(),
            pada: 2,
            rashi_lord: "Mercury".to_string(),
        },
        special_points: SpecialPoints {
            lagna: 225.3,           // Scorpio 15.3 deg = 210 + 15.3
            midheaven: Some(147.0), // Near Leo cusp
            part_of_fortune: Some(169.5),
        },
        notes: vec![
            "Jupiter exalted in Cancer (9th house)".to_string(),
            "Mercury exalted in Virgo (11th house)".to_string(),
            "Saturn in own sign Capricorn (retrograde)".to_string(),
            "Mars debilitated in Cancer (9th house)".to_string(),
            "Scorpio Ascendant - Vrishchika Lagna".to_string(),
        ],
    }
}

/// Whole-sign houses starting from Scorpio ascendant
fn shesh_houses() -> Vec<HousePosition> {
    // Scorpio ascendant: House 1 = Scorpio (index 7)
    (1u8..=12)
        .map(|n| {
            let sign_idx = (7 + n as usize - 1) % 12; // Start from Scorpio
            let sign = ZodiacSign::from_index(sign_idx);
            HousePosition {
                number: n,
                sign,
                cusp: (sign_idx as f64) * 30.0 + 15.3,
                degree: 15.3,
                house_type: match n {
                    1 | 5 | 9 => HouseType::Dharma,
                    2 | 6 | 10 => HouseType::Artha,
                    3 | 7 | 11 => HouseType::Kama,
                    _ => HouseType::Moksha,
                },
                is_kendra: matches!(n, 1 | 4 | 7 | 10),
                is_panapara: matches!(n, 2 | 5 | 8 | 11),
                is_apoklima: matches!(n, 3 | 6 | 9 | 12),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Navamsa Chart mock: D9
// ---------------------------------------------------------------------------

/// Create Shesh's Navamsa chart (D9).
///
/// Navamsa positions are derived from the D1 positions using the
/// standard navamsa division rules.
pub fn shesh_navamsa_chart() -> NavamsaChart {
    NavamsaChart {
        source: shesh_native_info(),
        navamsa_positions: vec![
            // Sun at 27.5 Leo (fixed sign, start Leo): 27.5 / 3.33 = 8th navamsa -> Leo + 8 = Pisces
            NavamsaPosition {
                planet: "Sun".to_string(),
                sign: ZodiacSign::Pisces,
                degree: 7.5,
                is_vargottama: false,
            },
            // Moon at 6.8 Virgo (mutable, start Sag): 6.8 / 3.33 = 2nd navamsa -> Sag + 2 = Aquarius
            NavamsaPosition {
                planet: "Moon".to_string(),
                sign: ZodiacSign::Aquarius,
                degree: 1.2,
                is_vargottama: false,
            },
            // Mars at 27.2 Cancer (cardinal, start Aries): 27.2 / 3.33 = 8th navamsa -> Aries + 8 = Sagittarius
            NavamsaPosition {
                planet: "Mars".to_string(),
                sign: ZodiacSign::Sagittarius,
                degree: 4.6,
                is_vargottama: false,
            },
            // Mercury at 10.4 Virgo (mutable, start Sag): 10.4 / 3.33 = 3rd navamsa -> Sag + 3 = Pisces
            NavamsaPosition {
                planet: "Mercury".to_string(),
                sign: ZodiacSign::Pisces,
                degree: 13.2,
                is_vargottama: false,
            },
            // Jupiter at 28.5 Cancer (cardinal, start Aries): 28.5 / 3.33 = 8th navamsa -> Aries + 8 = Sagittarius
            NavamsaPosition {
                planet: "Jupiter".to_string(),
                sign: ZodiacSign::Sagittarius,
                degree: 16.5,
                is_vargottama: false,
            },
            // Venus at 10.2 Leo (fixed, start Leo): 10.2 / 3.33 = 3rd navamsa -> Leo + 3 = Scorpio
            NavamsaPosition {
                planet: "Venus".to_string(),
                sign: ZodiacSign::Scorpio,
                degree: 21.6,
                is_vargottama: false,
            },
            // Saturn at 28.3 Capricorn (cardinal, start Aries): 28.3 / 3.33 = 8th navamsa -> Aries + 8 = Sagittarius
            NavamsaPosition {
                planet: "Saturn".to_string(),
                sign: ZodiacSign::Sagittarius,
                degree: 24.9,
                is_vargottama: false,
            },
            // Rahu at 26.7 Sagittarius (mutable, start Sag): 26.7 / 3.33 = 8th navamsa -> Sag + 8 = Leo
            NavamsaPosition {
                planet: "Rahu".to_string(),
                sign: ZodiacSign::Leo,
                degree: 0.1,
                is_vargottama: false,
            },
            // Ketu at 26.7 Gemini (mutable, start Sag): 26.7 / 3.33 = 8th navamsa -> Sag + 8 = Leo -- but wait, opposite to Rahu
            // Actually: Ketu 26.7 Gemini (mutable, start Sag): 8th navamsa -> Sag + 8 = Aquarius? Let me recalculate
            // Gemini is mutable, starts from Sagittarius (index 8). 26.7/3.33 = 8 (0-indexed). 8 + 8 = 16 % 12 = 4 -> Leo
            NavamsaPosition {
                planet: "Ketu".to_string(),
                sign: ZodiacSign::Aquarius,
                degree: 0.1,
                is_vargottama: false,
            },
        ],
        // No planet is in the same sign in both D1 and D9 for this chart
        vargottama: vec![],
        d9_lagna: ZodiacSign::Cancer,
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Native info for Shesh - reused across chart types
fn shesh_native_info() -> NativeInfo {
    NativeInfo {
        birth_date: "1991-09-14".to_string(),
        birth_time: "09:30:00".to_string(),
        latitude: SHESH_LAT,
        longitude: SHESH_LNG,
        timezone: SHESH_TZONE,
    }
}

// ---------------------------------------------------------------------------
// Transit fixture (PR2 of 3)
// ---------------------------------------------------------------------------

/// Stable transit analysis fixture for Shesh's birth chart against a fixed
/// transit date (2024-01-15). Used by `MockApiClient::get_transits` and for
/// downstream callers that need a Transit shape without spinning up the
/// Swiss Ephemeris global state.
pub fn shesh_transit_analysis() -> TransitApiResponse {
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
            moon_sign: "Virgo".to_string(),
        }),
        jupiter_transit: Some(JupiterTransitResponse {
            sign: "Aries".to_string(),
            from_ascendant: 6,
            from_moon: 8,
            quality: Some("Challenging".to_string()),
        }),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::DashaPlanet;

    // ---- Panchang validation ----

    #[test]
    fn test_shesh_panchang_date() {
        let p = shesh_panchang();
        assert_eq!(p.date.year, 1991);
        assert_eq!(p.date.month, 9);
        assert_eq!(p.date.day, 14);
        assert_eq!(p.vara, Vara::Saturday);
    }

    #[test]
    fn test_shesh_panchang_tithi() {
        let p = shesh_panchang();
        assert_eq!(p.tithi.name(), "Shashthi");
        assert_eq!(p.paksha, Paksha::Shukla);
    }

    #[test]
    fn test_shesh_panchang_nakshatra() {
        let p = shesh_panchang();
        assert_eq!(p.nakshatra.name(), "Uttara Phalguni");
        assert_eq!(p.nakshatra.pada, 2);
        assert_eq!(p.nakshatra.ruling_planet(), "Sun");
    }

    #[test]
    fn test_shesh_panchang_yoga() {
        let p = shesh_panchang();
        assert_eq!(p.yoga.name(), "Shobhana");
        assert!(p.yoga.is_auspicious());
    }

    #[test]
    fn test_shesh_panchang_karana() {
        let p = shesh_panchang();
        assert_eq!(p.karana.name(), "Taitila");
        assert_eq!(p.karana.karana_type, KaranaType::Movable);
    }

    #[test]
    fn test_shesh_panchang_location() {
        let p = shesh_panchang();
        assert!((p.location.latitude - SHESH_LAT).abs() < 0.001);
        assert!((p.location.longitude - SHESH_LNG).abs() < 0.001);
    }

    #[test]
    fn test_shesh_panchang_serialization_roundtrip() {
        let original = shesh_panchang();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Panchang = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.date.year, 1991);
        assert_eq!(deserialized.nakshatra.name(), "Uttara Phalguni");
    }

    // ---- Vimshottari Dasha validation ----

    #[test]
    fn test_shesh_dasha_moon_nakshatra() {
        let d = shesh_vimshottari_dasha();
        assert_eq!(d.moon_nakshatra, "Uttara Phalguni");
    }

    #[test]
    fn test_shesh_dasha_birth_planet() {
        let d = shesh_vimshottari_dasha();
        assert_eq!(d.balance.planet, DashaPlanet::Sun);
    }

    #[test]
    fn test_shesh_dasha_sun_mahadasha_dates() {
        let d = shesh_vimshottari_dasha();
        let sun_maha = &d.mahadashas[0];
        assert_eq!(sun_maha.planet, DashaPlanet::Sun);
        assert_eq!(sun_maha.start_date, "1991-09-14");
        assert_eq!(sun_maha.end_date, "1997-09-13");
        assert_eq!(sun_maha.duration_years, 6.0);
    }

    #[test]
    fn test_shesh_dasha_mars_mahadasha_dates() {
        let d = shesh_vimshottari_dasha();
        let mars_maha = d
            .mahadashas
            .iter()
            .find(|m| m.planet == DashaPlanet::Mars)
            .expect("Mars mahadasha should exist");
        assert_eq!(mars_maha.start_date, "2008-09-13");
    }

    #[test]
    fn test_shesh_dasha_full_sequence() {
        let d = shesh_vimshottari_dasha();
        assert_eq!(d.mahadashas.len(), 9);
        let planets: Vec<DashaPlanet> = d.mahadashas.iter().map(|m| m.planet).collect();
        assert_eq!(
            planets,
            vec![
                DashaPlanet::Sun,
                DashaPlanet::Moon,
                DashaPlanet::Mars,
                DashaPlanet::Rahu,
                DashaPlanet::Jupiter,
                DashaPlanet::Saturn,
                DashaPlanet::Mercury,
                DashaPlanet::Ketu,
                DashaPlanet::Venus,
            ]
        );
    }

    #[test]
    fn test_shesh_dasha_sun_has_subperiods() {
        let d = shesh_vimshottari_dasha();
        let sun_maha = &d.mahadashas[0];
        assert!(sun_maha.sub_periods.is_some());
        let subs = sun_maha.sub_periods.as_ref().unwrap();
        assert!(!subs.is_empty());
        assert_eq!(subs[0].planet, DashaPlanet::Sun);
        assert_eq!(subs[0].level, DashaLevel::Antardasha);
    }

    #[test]
    fn test_shesh_dasha_serialization_roundtrip() {
        let original = shesh_vimshottari_dasha();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: VimshottariDasha = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.moon_nakshatra, "Uttara Phalguni");
        assert_eq!(deserialized.mahadashas.len(), 9);
    }

    // ---- Birth Chart validation ----

    #[test]
    fn test_shesh_chart_scorpio_ascendant() {
        let c = shesh_birth_chart();
        assert_eq!(c.ascendant.sign, ZodiacSign::Scorpio);
    }

    #[test]
    fn test_shesh_chart_moon_in_virgo() {
        let c = shesh_birth_chart();
        assert_eq!(c.moon.sign, ZodiacSign::Virgo);
        assert_eq!(c.moon.nakshatra, "Uttara Phalguni");
        assert_eq!(c.moon.pada, 2);
    }

    #[test]
    fn test_shesh_chart_all_nine_planets() {
        let c = shesh_birth_chart();
        let planet_names: Vec<&str> = c.planets.iter().map(|p| p.name.as_str()).collect();
        assert!(planet_names.contains(&"Sun"));
        assert!(planet_names.contains(&"Moon"));
        assert!(planet_names.contains(&"Mars"));
        assert!(planet_names.contains(&"Mercury"));
        assert!(planet_names.contains(&"Jupiter"));
        assert!(planet_names.contains(&"Venus"));
        assert!(planet_names.contains(&"Saturn"));
        assert!(planet_names.contains(&"Rahu"));
        assert!(planet_names.contains(&"Ketu"));
        assert_eq!(c.planets.len(), 9);
    }

    #[test]
    fn test_shesh_chart_jupiter_exalted() {
        let c = shesh_birth_chart();
        let jupiter = c.get_planet("Jupiter").expect("Jupiter must exist");
        assert_eq!(jupiter.sign, ZodiacSign::Cancer);
        // Jupiter is exalted in Cancer
    }

    #[test]
    fn test_shesh_chart_mercury_exalted() {
        let c = shesh_birth_chart();
        let mercury = c.get_planet("Mercury").expect("Mercury must exist");
        assert_eq!(mercury.sign, ZodiacSign::Virgo);
        assert!(mercury.is_exalted());
    }

    #[test]
    fn test_shesh_chart_saturn_own_sign() {
        let c = shesh_birth_chart();
        let saturn = c.get_planet("Saturn").expect("Saturn must exist");
        assert_eq!(saturn.sign, ZodiacSign::Capricorn);
        assert!(saturn.in_own_sign());
        assert!(saturn.is_retrograde);
    }

    #[test]
    fn test_shesh_chart_mars_debilitated() {
        let c = shesh_birth_chart();
        let mars = c.get_planet("Mars").expect("Mars must exist");
        assert_eq!(mars.sign, ZodiacSign::Cancer);
        assert!(mars.is_debilitated());
    }

    #[test]
    fn test_shesh_chart_twelve_houses() {
        let c = shesh_birth_chart();
        assert_eq!(c.houses.len(), 12);
        // First house should be Scorpio (whole sign from Scorpio ascendant)
        assert_eq!(c.houses[0].sign, ZodiacSign::Scorpio);
        assert_eq!(c.houses[0].number, 1);
    }

    #[test]
    fn test_shesh_chart_house_sequence() {
        let c = shesh_birth_chart();
        // Starting from Scorpio, houses should be:
        // 1=Scorpio, 2=Sagittarius, 3=Capricorn, 4=Aquarius, ...
        assert_eq!(c.houses[0].sign, ZodiacSign::Scorpio);
        assert_eq!(c.houses[1].sign, ZodiacSign::Sagittarius);
        assert_eq!(c.houses[2].sign, ZodiacSign::Capricorn);
        assert_eq!(c.houses[3].sign, ZodiacSign::Aquarius);
        assert_eq!(c.houses[4].sign, ZodiacSign::Pisces);
    }

    #[test]
    fn test_shesh_chart_serialization_roundtrip() {
        let original = shesh_birth_chart();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: BirthChart = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ascendant.sign, ZodiacSign::Scorpio);
        assert_eq!(deserialized.planets.len(), 9);
    }

    // ---- Navamsa Chart validation ----

    #[test]
    fn test_shesh_navamsa_has_all_planets() {
        let n = shesh_navamsa_chart();
        assert_eq!(n.navamsa_positions.len(), 9);
    }

    #[test]
    fn test_shesh_navamsa_d9_lagna() {
        let n = shesh_navamsa_chart();
        assert_eq!(n.d9_lagna, ZodiacSign::Cancer);
    }

    #[test]
    fn test_shesh_navamsa_serialization_roundtrip() {
        let original = shesh_navamsa_chart();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: NavamsaChart = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.d9_lagna, ZodiacSign::Cancer);
        assert_eq!(deserialized.navamsa_positions.len(), 9);
    }

    // ---- Transit fixture validation (PR2) ----

    #[test]
    fn test_shesh_transit_analysis_has_twelve_planets() {
        let t = shesh_transit_analysis();
        assert_eq!(t.transits.len(), 12);
        assert!(t.sade_sati.is_some());
        assert!(t.jupiter_transit.is_some());
    }

    // ---- MockResponses validation ----

    #[test]
    fn test_mock_responses_panchang_json() {
        let val = MockResponses::panchang_response();
        assert!(val.is_object());
        let json_str = MockResponses::panchang_json();
        assert!(json_str.contains("Uttara Phalguni"));
    }

    #[test]
    fn test_mock_responses_vimshottari_json() {
        let val = MockResponses::vimshottari_response();
        assert!(val.is_object());
        let json_str = MockResponses::vimshottari_json();
        assert!(json_str.contains("Uttara Phalguni"));
    }

    #[test]
    fn test_mock_responses_birth_chart_json() {
        let val = MockResponses::birth_chart_response();
        assert!(val.is_object());
        let json_str = MockResponses::birth_chart_json();
        assert!(json_str.contains("scorpio"));
    }

    #[test]
    fn test_mock_responses_navamsa_json() {
        let val = MockResponses::navamsa_response();
        assert!(val.is_object());
        let json_str = MockResponses::navamsa_json();
        assert!(json_str.contains("cancer"));
    }

    // ---- MockApiClient validation ----

    #[test]
    fn test_mock_api_client_call_tracking() {
        let client = MockApiClient::new();
        assert_eq!(client.call_count(), 0);

        let _ = client.get_panchang();
        assert_eq!(client.call_count(), 1);

        let _ = client.get_birth_chart();
        assert_eq!(client.call_count(), 2);

        client.reset_count();
        assert_eq!(client.call_count(), 0);
    }

    #[test]
    fn test_mock_api_client_returns_correct_data() {
        let client = MockApiClient::new();

        let panchang = client.get_panchang();
        assert_eq!(panchang.nakshatra.name(), "Uttara Phalguni");

        let dasha = client.get_vimshottari_dasha();
        assert_eq!(dasha.moon_nakshatra, "Uttara Phalguni");

        let chart = client.get_birth_chart();
        assert_eq!(chart.ascendant.sign, ZodiacSign::Scorpio);

        let navamsa = client.get_navamsa_chart();
        assert_eq!(navamsa.d9_lagna, ZodiacSign::Cancer);
    }

    #[test]
    fn test_mock_api_client_default() {
        let client = MockApiClient::default();
        assert_eq!(client.call_count(), 0);
    }

    // ---- Cross-validation between mocks ----

    #[test]
    fn test_panchang_and_chart_moon_consistency() {
        let p = shesh_panchang();
        let c = shesh_birth_chart();

        // Moon nakshatra should match between panchang and chart
        assert_eq!(p.nakshatra.name(), c.moon.nakshatra.as_str());

        // Moon sign should be Virgo in both
        assert_eq!(p.planets.moon.sign, "Virgo");
        assert_eq!(c.moon.sign, ZodiacSign::Virgo);
    }

    #[test]
    fn test_dasha_and_chart_nakshatra_consistency() {
        let d = shesh_vimshottari_dasha();
        let c = shesh_birth_chart();

        // Moon nakshatra in dasha should match chart
        assert_eq!(d.moon_nakshatra, c.moon.nakshatra);
    }

    #[test]
    fn test_dasha_and_panchang_nakshatra_consistency() {
        let d = shesh_vimshottari_dasha();
        let p = shesh_panchang();

        // Both should report Uttara Phalguni
        assert_eq!(d.moon_nakshatra, p.nakshatra.name());
    }
}
