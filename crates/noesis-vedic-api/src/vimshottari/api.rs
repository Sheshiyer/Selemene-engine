//! Vimshottari Dasha — native facade backed by `engine-vimshottari`.
//!
//! PR2 of 3 (#871 stack): the previous implementation POSTed to
//! `/vimshottari-dasha` which returns 403. This module now computes the full
//! 120-year timeline natively via `engine-vimshottari` (Moon longitude →
//! nakshatra → dasha balance → mahadashas → complete timeline) and maps the
//! result into the canonical `crate::dasha::VimshottariDasha` shape so
//! `CachedVedicClient`, `VedicApiService`, and downstream callers keep
//! working unchanged.
//!
//! Public types (`VimshottariRequest`, `VimshottariApiResponse`, …) are
//! preserved for source-compat.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};

use super::types::{DashaLevel, DashaLord};
use crate::client::VedicApiClient;
use crate::dasha::{
    DashaBalance as CanonicalBalance, DashaLevel as CanonicalLevel, DashaPeriod as CanonicalPeriod,
    DashaPlanet as CanonicalPlanet, VimshottariDasha,
};
use crate::error::{VedicApiError, VedicApiResult};

/// Request for Vimshottari Dasha calculation
#[derive(Debug, Clone, Serialize)]
pub struct VimshottariRequest {
    /// Birth date (YYYY-MM-DD)
    pub birth_date: String,
    /// Birth time (HH:MM:SS)
    pub birth_time: String,
    /// Latitude of birth location
    pub latitude: f64,
    /// Longitude of birth location
    pub longitude: f64,
    /// Timezone offset in hours
    pub timezone: f64,
    /// Level of detail to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    /// Ayanamsa to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl VimshottariRequest {
    /// Create a new Vimshottari request from birth details
    pub fn new(
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> Self {
        Self {
            birth_date: birth_datetime.date().format("%Y-%m-%d").to_string(),
            birth_time: birth_datetime.time().format("%H:%M:%S").to_string(),
            latitude,
            longitude,
            timezone,
            level: None,
            ayanamsa: Some("lahiri".to_string()),
        }
    }

    /// Set the dasha level detail
    pub fn with_level(mut self, level: DashaLevel) -> Self {
        self.level = Some(match level {
            DashaLevel::Mahadasha => "mahadasha".to_string(),
            DashaLevel::Antardasha => "antardasha".to_string(),
            DashaLevel::Pratyantardasha => "pratyantardasha".to_string(),
            DashaLevel::Sookshma => "sookshma".to_string(),
            DashaLevel::Prana => "prana".to_string(),
        });
        self
    }

    /// Set the ayanamsa
    pub fn with_ayanamsa(mut self, ayanamsa: &str) -> Self {
        self.ayanamsa = Some(ayanamsa.to_string());
        self
    }
}

/// Raw API response shape (preserved for source-compat — populated natively).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimshottariApiResponse {
    /// Moon nakshatra at birth
    pub moon_nakshatra: MoonNakshatraInfo,
    /// Balance of dasha at birth
    pub dasha_balance: DashaBalanceResponse,
    /// Mahadasha periods
    pub mahadashas: Vec<MahadashaResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonNakshatraInfo {
    pub name: String,
    pub number: u8,
    pub lord: String,
    #[serde(default)]
    pub pada: Option<u8>,
    #[serde(default)]
    pub degree: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaBalanceResponse {
    pub lord: String,
    pub years: u32,
    pub months: u32,
    pub days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MahadashaResponse {
    pub lord: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub antardashas: Option<Vec<AntardashaResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntardashaResponse {
    pub lord: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub pratyantardashas: Option<Vec<PratyantardashaResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PratyantardashaResponse {
    pub lord: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub sookshmas: Option<Vec<SookshmaResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SookshmaResponse {
    pub lord: String,
    pub start_date: String,
    pub end_date: String,
}

// ---------------------------------------------------------------------------
// Native facade — engine-vimshottari pipeline
// ---------------------------------------------------------------------------

/// Convert local civil time into UTC `DateTime`.
fn local_to_utc(
    date: NaiveDate,
    time: NaiveTime,
    tz_offset_hours: f64,
) -> VedicApiResult<chrono::DateTime<Utc>> {
    let naive = NaiveDateTime::new(date, time);
    let offset_seconds = (tz_offset_hours * 3600.0) as i64;
    Utc.from_utc_datetime(&naive)
        .checked_sub_signed(chrono::Duration::seconds(offset_seconds))
        .ok_or_else(|| VedicApiError::ParseError("datetime out of range".to_string()))
}

/// Map an engine-native Vedic planet to the canonical `DashaPlanet`.
fn vedic_to_dasha_planet(p: engine_vimshottari::VedicPlanet) -> CanonicalPlanet {
    use engine_vimshottari::VedicPlanet as V;
    match p {
        V::Sun => CanonicalPlanet::Sun,
        V::Moon => CanonicalPlanet::Moon,
        V::Mars => CanonicalPlanet::Mars,
        V::Rahu => CanonicalPlanet::Rahu,
        V::Jupiter => CanonicalPlanet::Jupiter,
        V::Saturn => CanonicalPlanet::Saturn,
        V::Mercury => CanonicalPlanet::Mercury,
        V::Ketu => CanonicalPlanet::Ketu,
        V::Venus => CanonicalPlanet::Venus,
    }
}

fn ymd(date: chrono::DateTime<Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn duration_days(start: chrono::DateTime<Utc>, end: chrono::DateTime<Utc>) -> u32 {
    let secs = (end - start).num_seconds().max(0);
    (secs / 86_400) as u32
}

/// Build the canonical `crate::dasha::DashaPeriod` tree from an engine-native
/// `Mahadasha`, honoring the requested `level` (deeper levels populate
/// `sub_periods`).
fn build_period(
    md: &engine_vimshottari::models::Mahadasha,
    level: DashaLevel,
) -> CanonicalPeriod {
    let antardashas: Option<Vec<CanonicalPeriod>> = if matches!(
        level,
        DashaLevel::Antardasha
            | DashaLevel::Pratyantardasha
            | DashaLevel::Sookshma
            | DashaLevel::Prana
    ) {
        let inner = md
            .antardashas
            .iter()
            .map(|ad| {
                let pratyantar: Option<Vec<CanonicalPeriod>> = if matches!(
                    level,
                    DashaLevel::Pratyantardasha | DashaLevel::Sookshma | DashaLevel::Prana
                ) {
                    Some(
                        ad.pratyantardashas
                            .iter()
                            .map(|pd| CanonicalPeriod {
                                planet: vedic_to_dasha_planet(pd.planet),
                                level: CanonicalLevel::Pratyantardasha,
                                start_date: ymd(pd.start_date),
                                end_date: ymd(pd.end_date),
                                duration_years: pd.duration_days / 365.25,
                                duration_days: pd.duration_days as u32,
                                sub_periods: None,
                            })
                            .collect(),
                    )
                } else {
                    None
                };
                CanonicalPeriod {
                    planet: vedic_to_dasha_planet(ad.planet),
                    level: CanonicalLevel::Antardasha,
                    start_date: ymd(ad.start_date),
                    end_date: ymd(ad.end_date),
                    duration_years: ad.duration_years,
                    duration_days: duration_days(ad.start_date, ad.end_date),
                    sub_periods: pratyantar,
                }
            })
            .collect::<Vec<_>>();
        Some(inner)
    } else {
        None
    };

    CanonicalPeriod {
        planet: vedic_to_dasha_planet(md.planet),
        level: CanonicalLevel::Mahadasha,
        start_date: ymd(md.start_date),
        end_date: ymd(md.end_date),
        duration_years: md.duration_years,
        duration_days: duration_days(md.start_date, md.end_date),
        sub_periods: antardashas,
    }
}

/// Internal native helper — compute a full `VimshottariDasha` from birth
/// inputs without touching the network. All Swiss-Ephemeris work is wrapped
/// in `spawn_blocking` because libswisseph-sys uses a global mutex.
pub(crate) async fn compute_vimshottari_native(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    tz_offset_hours: f64,
    level: DashaLevel,
) -> VedicApiResult<VimshottariDasha> {
    let date = NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| {
        VedicApiError::InvalidInput {
            field: "birth_date".to_string(),
            message: format!("invalid date {}-{:02}-{:02}", year, month, day),
        }
    })?;
    let time = NaiveTime::from_hms_opt(hour, minute, second).ok_or_else(|| {
        VedicApiError::InvalidInput {
            field: "birth_time".to_string(),
            message: format!("invalid time {:02}:{:02}:{:02}", hour, minute, second),
        }
    })?;
    let birth_utc = local_to_utc(date, time, tz_offset_hours)?;

    // Swiss Ephemeris call — must be on a blocking thread.
    let moon_longitude: f64 = tokio::task::spawn_blocking(move || {
        let calc = engine_human_design::EphemerisCalculator::new("");
        engine_transits::ephemeris::calculate_position(
            &calc,
            engine_transits::models::TransitPlanet::Moon,
            &birth_utc,
        )
        .map(|p| p.longitude)
    })
    .await
    .map_err(|e| VedicApiError::ServiceUnavailable(format!("ephemeris task join failed: {}", e)))?
    .map_err(|e| {
        VedicApiError::ServiceUnavailable(format!("Moon ephemeris failed: {}", e))
    })?;

    let nakshatra = engine_vimshottari::get_nakshatra_from_longitude(moon_longitude);
    let balance_years = engine_vimshottari::calculate_dasha_balance(moon_longitude, nakshatra);
    let mahadashas_raw = engine_vimshottari::calculate_mahadashas(
        birth_utc,
        nakshatra.ruling_planet,
        balance_years,
    );
    let mahadashas_full = engine_vimshottari::calculate_complete_timeline(mahadashas_raw);

    // Build canonical periods at requested depth.
    let mahadashas: Vec<CanonicalPeriod> = mahadashas_full
        .iter()
        .map(|md| build_period(md, level))
        .collect();

    // Locate "now"-period at each level via linear scan against today's date.
    let now = Utc::now();
    let today = ymd(now);
    let current_mahadasha = mahadashas
        .iter()
        .find(|md| md.start_date.as_str() <= today.as_str() && today.as_str() <= md.end_date.as_str())
        .cloned()
        .unwrap_or_else(|| mahadashas[0].clone());
    let current_antardasha = current_mahadasha
        .sub_periods
        .as_ref()
        .and_then(|subs| {
            subs.iter()
                .find(|ad| ad.start_date.as_str() <= today.as_str() && today.as_str() <= ad.end_date.as_str())
                .cloned()
        });
    let current_pratyantardasha = current_antardasha
        .as_ref()
        .and_then(|ad| ad.sub_periods.as_ref())
        .and_then(|subs| {
            subs.iter()
                .find(|pd| pd.start_date.as_str() <= today.as_str() && today.as_str() <= pd.end_date.as_str())
                .cloned()
        });

    let years = balance_years.floor() as u32;
    let months_frac = (balance_years - years as f64) * 12.0;
    let months = months_frac.floor() as u32;
    let days = ((months_frac - months as f64) * 30.4375).floor() as u32;

    Ok(VimshottariDasha {
        birth_date: format!("{:04}-{:02}-{:02}", year, month, day),
        moon_nakshatra: nakshatra.name.clone(),
        moon_longitude,
        balance: CanonicalBalance {
            planet: vedic_to_dasha_planet(nakshatra.ruling_planet),
            years_remaining: years as f64,
            months_remaining: months as f64,
            days_remaining: days as f64,
            total_period_years: nakshatra.ruling_planet.period_years() as f64,
        },
        mahadashas,
        current_mahadasha,
        current_antardasha,
        current_pratyantardasha,
        current_sookshma: None,
    })
}

/// Convert a `crate::dasha::VimshottariDasha` into the legacy
/// `VimshottariApiResponse` envelope for any caller still wired against
/// that shape. (Internal use only — the canonical public surface remains
/// `VimshottariDasha`.)
fn to_api_response(d: &VimshottariDasha) -> VimshottariApiResponse {
    let mahadashas = d
        .mahadashas
        .iter()
        .map(|md| MahadashaResponse {
            lord: md.planet.as_str().to_string(),
            start_date: md.start_date.clone(),
            end_date: md.end_date.clone(),
            antardashas: md.sub_periods.as_ref().map(|subs| {
                subs.iter()
                    .map(|ad| AntardashaResponse {
                        lord: ad.planet.as_str().to_string(),
                        start_date: ad.start_date.clone(),
                        end_date: ad.end_date.clone(),
                        pratyantardashas: ad.sub_periods.as_ref().map(|inner| {
                            inner
                                .iter()
                                .map(|pd| PratyantardashaResponse {
                                    lord: pd.planet.as_str().to_string(),
                                    start_date: pd.start_date.clone(),
                                    end_date: pd.end_date.clone(),
                                    sookshmas: None,
                                })
                                .collect()
                        }),
                    })
                    .collect()
            }),
        })
        .collect();

    VimshottariApiResponse {
        moon_nakshatra: MoonNakshatraInfo {
            name: d.moon_nakshatra.clone(),
            number: 0,
            lord: d.balance.planet.as_str().to_string(),
            pada: Some(((d.moon_longitude.rem_euclid(13.333_333) / 13.333_333) * 4.0).floor() as u8 + 1),
            degree: Some(d.moon_longitude),
        },
        dasha_balance: DashaBalanceResponse {
            lord: d.balance.planet.as_str().to_string(),
            years: d.balance.years_remaining as u32,
            months: d.balance.months_remaining as u32,
            days: d.balance.days_remaining as u32,
        },
        mahadashas,
    }
}

impl VedicApiClient {
    /// Get Vimshottari Dasha timeline via native `engine-vimshottari`.
    ///
    /// PR2: no HTTP call. Computes Moon longitude via Swiss Ephemeris (on a
    /// blocking thread), derives the birth nakshatra, runs the full 120-year
    /// pipeline, then mirrors the result back as `VimshottariApiResponse`
    /// for legacy callers.
    pub async fn get_vimshottari_dasha_request(
        &self,
        request: &VimshottariRequest,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let date = NaiveDate::parse_from_str(&request.birth_date, "%Y-%m-%d").map_err(|e| {
            VedicApiError::ParseError(format!("invalid birth_date '{}': {}", request.birth_date, e))
        })?;
        let time = NaiveTime::parse_from_str(&request.birth_time, "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(&request.birth_time, "%H:%M"))
            .map_err(|e| {
                VedicApiError::ParseError(format!(
                    "invalid birth_time '{}': {}",
                    request.birth_time, e
                ))
            })?;
        let level = match request.level.as_deref() {
            Some("mahadasha") => DashaLevel::Mahadasha,
            Some("antardasha") => DashaLevel::Antardasha,
            Some("pratyantardasha") => DashaLevel::Pratyantardasha,
            Some("sookshma") => DashaLevel::Sookshma,
            Some("prana") => DashaLevel::Prana,
            _ => DashaLevel::Antardasha,
        };

        let canonical = compute_vimshottari_native(
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            time.second(),
            request.timezone,
            level,
        )
        .await?;

        Ok(to_api_response(&canonical))
    }

    /// Get Mahadasha level only
    pub async fn get_mahadasha_only(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let request = VimshottariRequest::new(birth_datetime, latitude, longitude, timezone)
            .with_level(DashaLevel::Mahadasha);
        self.get_vimshottari_dasha_request(&request).await
    }

    /// Get Antardasha level
    pub async fn get_antardasha_level(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let request = VimshottariRequest::new(birth_datetime, latitude, longitude, timezone)
            .with_level(DashaLevel::Antardasha);
        self.get_vimshottari_dasha_request(&request).await
    }

    /// Get Pratyantardasha level
    pub async fn get_pratyantardasha_level(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let request = VimshottariRequest::new(birth_datetime, latitude, longitude, timezone)
            .with_level(DashaLevel::Pratyantardasha);
        self.get_vimshottari_dasha_request(&request).await
    }

    /// Get Sookshma Dasha level
    pub async fn get_sookshma_level(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let request = VimshottariRequest::new(birth_datetime, latitude, longitude, timezone)
            .with_level(DashaLevel::Sookshma);
        self.get_vimshottari_dasha_request(&request).await
    }
}

/// Parse lord string to DashaLord enum
pub fn parse_dasha_lord(lord: &str) -> VedicApiResult<DashaLord> {
    match lord.to_lowercase().as_str() {
        "sun" | "surya" => Ok(DashaLord::Sun),
        "moon" | "chandra" => Ok(DashaLord::Moon),
        "mars" | "mangal" => Ok(DashaLord::Mars),
        "rahu" => Ok(DashaLord::Rahu),
        "jupiter" | "guru" => Ok(DashaLord::Jupiter),
        "saturn" | "shani" => Ok(DashaLord::Saturn),
        "mercury" | "budha" => Ok(DashaLord::Mercury),
        "ketu" => Ok(DashaLord::Ketu),
        "venus" | "shukra" => Ok(DashaLord::Venus),
        _ => Err(VedicApiError::ParseError(format!("Unknown dasha lord: {}", lord))),
    }
}

/// Parse date string from API
pub fn parse_date(date_str: &str) -> VedicApiResult<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%d-%m-%Y"))
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%d/%m/%Y"))
        .map_err(|e| VedicApiError::ParseError(format!("Invalid date format '{}': {}", date_str, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> VedicApiClient {
        VedicApiClient::new(crate::mocks::mock_config("http://localhost:0"))
    }

    #[test]
    fn test_vimshottari_request_creation() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = VimshottariRequest::new(dt, 12.97, 77.59, 5.5);

        assert_eq!(request.birth_date, "1990-06-15");
        assert_eq!(request.birth_time, "10:30:00");
    }

    #[test]
    fn test_parse_dasha_lord() {
        assert_eq!(parse_dasha_lord("Sun").unwrap(), DashaLord::Sun);
        assert_eq!(parse_dasha_lord("surya").unwrap(), DashaLord::Sun);
        assert_eq!(parse_dasha_lord("RAHU").unwrap(), DashaLord::Rahu);
        assert!(parse_dasha_lord("Unknown").is_err());
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2024-01-15").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_native_vimshottari_bangalore_1991() {
        // 1991-08-13 13:31:00 IST, Bangalore.
        let dasha = compute_vimshottari_native(1991, 8, 13, 13, 31, 0, 5.5, DashaLevel::Antardasha)
            .await
            .expect("native vimshottari computation must succeed");

        // 120-year mahadasha sequence (typically 9..=10 periods).
        assert!(dasha.mahadashas.len() >= 9, "got {} mahadashas", dasha.mahadashas.len());
        // Birth nakshatra name must be non-empty.
        assert!(!dasha.moon_nakshatra.is_empty());
        // Each mahadasha must have antardashas at this level.
        let first = &dasha.mahadashas[0];
        assert!(
            first.sub_periods.as_ref().map_or(false, |s| !s.is_empty()),
            "first mahadasha missing antardashas"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_get_vimshottari_dasha_via_client() {
        let client = test_client();
        let req = VimshottariRequest::new(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(1991, 8, 13).unwrap(),
                NaiveTime::from_hms_opt(13, 31, 0).unwrap(),
            ),
            12.9716,
            77.5946,
            5.5,
        );
        let resp = client
            .get_vimshottari_dasha_request(&req)
            .await
            .expect("native vimshottari via client must succeed");
        assert!(!resp.mahadashas.is_empty());
        assert!(!resp.moon_nakshatra.name.is_empty());
    }
}
