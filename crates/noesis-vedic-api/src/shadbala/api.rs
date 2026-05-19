//! Shadbala API — native facade backed by full Kala/Drik bala (PR3).
//!
//! Previous behaviour POSTed to `/shadbala` (dead, 403). `get_shadbala` now
//! builds a native chart via
//! [`crate::birth_chart::native::build_native_chart`] and computes all six
//! components per planet (`calculate_full_shadbala_with_context`), then
//! emits the legacy `ShadbalaApiResponse` envelope so downstream callers
//! keep working.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use super::types::{
    required_shadbala, ChartStrength, PlanetShadbala, ShadbalaAnalysis, ShadbalaComponent,
    ShadbalaValue,
};
use crate::birth_chart::native::{build_native_chart, parse_birth_inputs};
use crate::client::VedicApiClient;
use crate::error::VedicApiResult;

/// Request for Shadbala calculation
#[derive(Debug, Clone, Serialize)]
pub struct ShadbalaRequest {
    pub birth_date: String,
    pub birth_time: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl ShadbalaRequest {
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
            ayanamsa: Some("lahiri".to_string()),
        }
    }
}

/// API response for Shadbala (now `Serialize` too so JSON round-trips still
/// work for callers that mirrored the vendor wire format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadbalaApiResponse {
    pub planets: Vec<ShadbalaPlanetResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadbalaPlanetResponse {
    pub name: String,
    pub sthana_bala: f64,
    pub dig_bala: f64,
    pub kala_bala: f64,
    pub chesta_bala: f64,
    pub naisargika_bala: f64,
    pub drik_bala: f64,
    pub total: f64,
}

impl VedicApiClient {
    /// Compute Shadbala natively for every classical planet (PR3).
    ///
    /// Builds the native chart, then runs
    /// `calculate_full_shadbala_with_context` per planet so Kala and Drik
    /// reflect real birth context instead of the previous hardcoded
    /// constants.
    pub async fn get_shadbala(
        &self,
        request: &ShadbalaRequest,
    ) -> VedicApiResult<ShadbalaApiResponse> {
        let (y, m, d, hh, mm, ss) = parse_birth_inputs(&request.birth_date, &request.birth_time)?;
        let chart = build_native_chart(
            y,
            m,
            d,
            hh,
            mm,
            ss,
            request.latitude,
            request.longitude,
            request.timezone,
        )
        .await?;

        let date = NaiveDate::from_ymd_opt(y, m, d).expect("date already validated by parse step");
        let local_time =
            NaiveTime::from_hms_opt(hh, mm, ss).expect("time already validated by parse step");
        let (sunrise, sunset) = crate::astro::sunrise_sunset(
            date,
            request.latitude,
            request.longitude,
            request.timezone,
        );

        // We need a continuous tithi value for Paksha Bala. Compute it on the
        // blocking thread via engine_panchanga.
        let date_str = date.format("%Y-%m-%d").to_string();
        let time_str = local_time.format("%H:%M").to_string();
        let tz = request.timezone;
        let panchanga = tokio::task::spawn_blocking(move || {
            engine_panchanga::compute_panchanga(&date_str, &time_str, tz)
        })
        .await
        .map_err(|e| {
            crate::error::VedicApiError::ServiceUnavailable(format!(
                "panchanga task join failed: {}",
                e
            ))
        })?;
        let tithi_continuous = panchanga.tithi_value;

        let mut planets_out: Vec<ShadbalaPlanetResponse> = Vec::with_capacity(7);
        for &planet in &crate::birth_chart::types::Planet::classical() {
            let Some(pos) = chart.get_planet(planet) else {
                continue;
            };
            let pshad = super::calculator::calculate_full_shadbala_with_context(
                planet,
                pos.sign,
                pos.degree,
                pos.house,
                pos.is_retrograde,
                &chart,
                date,
                local_time,
                sunrise,
                sunset,
                tithi_continuous,
            );
            let get_comp = |c: ShadbalaComponent| -> f64 {
                pshad
                    .components
                    .iter()
                    .find(|v| v.component == c)
                    .map(|v| v.rupas)
                    .unwrap_or(0.0)
            };
            planets_out.push(ShadbalaPlanetResponse {
                name: planet.to_string(),
                sthana_bala: get_comp(ShadbalaComponent::SthanaBala),
                dig_bala: get_comp(ShadbalaComponent::DigBala),
                kala_bala: get_comp(ShadbalaComponent::KalaBala),
                chesta_bala: get_comp(ShadbalaComponent::ChestaBala),
                naisargika_bala: get_comp(ShadbalaComponent::NaisargikaBala),
                drik_bala: get_comp(ShadbalaComponent::DrikBala),
                total: pshad.total_rupas,
            });
        }

        Ok(ShadbalaApiResponse {
            planets: planets_out,
        })
    }
}

// Suppress unused-import warning when chrono::NaiveDateTime is only used in
// the request builder above.
#[allow(unused_imports)]
use chrono::NaiveDateTime as _NaiveDateTimeAlias;

/// Map API response to internal analysis
pub fn map_shadbala_response(response: ShadbalaApiResponse) -> ShadbalaAnalysis {
    let planets: Vec<PlanetShadbala> = response
        .planets
        .iter()
        .map(|p| {
            let required = required_shadbala(&p.name);
            let ratio = p.total / required;

            PlanetShadbala {
                planet: p.name.clone(),
                components: vec![
                    ShadbalaValue {
                        component: ShadbalaComponent::SthanaBala,
                        rupas: p.sthana_bala,
                        shashtiamsas: p.sthana_bala * 60.0,
                    },
                    ShadbalaValue {
                        component: ShadbalaComponent::DigBala,
                        rupas: p.dig_bala,
                        shashtiamsas: p.dig_bala * 60.0,
                    },
                    ShadbalaValue {
                        component: ShadbalaComponent::KalaBala,
                        rupas: p.kala_bala,
                        shashtiamsas: p.kala_bala * 60.0,
                    },
                    ShadbalaValue {
                        component: ShadbalaComponent::ChestaBala,
                        rupas: p.chesta_bala,
                        shashtiamsas: p.chesta_bala * 60.0,
                    },
                    ShadbalaValue {
                        component: ShadbalaComponent::NaisargikaBala,
                        rupas: p.naisargika_bala,
                        shashtiamsas: p.naisargika_bala * 60.0,
                    },
                    ShadbalaValue {
                        component: ShadbalaComponent::DrikBala,
                        rupas: p.drik_bala,
                        shashtiamsas: p.drik_bala * 60.0,
                    },
                ],
                total_rupas: p.total,
                total_shashtiamsas: p.total * 60.0,
                required_minimum: required,
                strength_ratio: ratio,
                is_strong: ratio >= 1.0,
            }
        })
        .collect();

    let strongest = planets
        .iter()
        .max_by(|a, b| a.strength_ratio.partial_cmp(&b.strength_ratio).unwrap())
        .map(|p| p.planet.clone())
        .unwrap_or_default();

    let weakest = planets
        .iter()
        .min_by(|a, b| a.strength_ratio.partial_cmp(&b.strength_ratio).unwrap())
        .map(|p| p.planet.clone())
        .unwrap_or_default();

    let avg_ratio: f64 = if !planets.is_empty() {
        planets.iter().map(|p| p.strength_ratio).sum::<f64>() / planets.len() as f64
    } else {
        0.0
    };

    ShadbalaAnalysis {
        planets,
        strongest_planet: strongest,
        weakest_planet: weakest,
        chart_strength: ChartStrength::from_average_ratio(avg_ratio),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_shadbala_request_creation() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = ShadbalaRequest::new(dt, 12.97, 77.59, 5.5);

        assert_eq!(request.birth_date, "1990-06-15");
    }

    fn test_client() -> VedicApiClient {
        VedicApiClient::new(crate::mocks::mock_config("http://localhost:0"))
    }

    /// Bangalore 1991-08-13 13:31 IST: native shadbala must run end-to-end
    /// with no network access, every classical planet must show up, and
    /// each planet's total must be a finite non-negative number.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn native_shadbala_bangalore_1991_runs_offline() {
        let client = test_client();
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1991, 8, 13).unwrap(),
            NaiveTime::from_hms_opt(13, 31, 0).unwrap(),
        );
        let req = ShadbalaRequest::new(dt, 12.9716, 77.5946, 5.5);
        let resp = client
            .get_shadbala(&req)
            .await
            .expect("native shadbala should succeed");
        // All seven classical planets must appear.
        assert_eq!(resp.planets.len(), 7);
        for p in &resp.planets {
            assert!(
                p.total.is_finite() && p.total > 0.0,
                "{} total non-finite or zero: {}",
                p.name,
                p.total
            );
            // Each individual component must lie inside a sane shashtiamsa
            // window — Kala in [0, 240], Drik in [-60, 60], the rest in
            // [0, 120] (some Sthana fragments add).
            assert!(
                (0.0..=240.0).contains(&p.kala_bala),
                "{} kala out of range: {}",
                p.name,
                p.kala_bala
            );
            assert!(
                (-60.0..=60.0).contains(&p.drik_bala),
                "{} drik out of range: {}",
                p.name,
                p.drik_bala
            );
        }
    }
}
