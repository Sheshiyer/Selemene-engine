//! Ashtakavarga API — native facade backed by BPHS BAV contribution tables.
//!
//! PR3: previous behaviour POSTed to `/ashtakavarga` (dead, 403).
//! `get_ashtakavarga` now builds a native chart and computes each planet's
//! Bhinna-Ashtakavarga from the published BPHS bindu tables in
//! `bindu_tables.rs`, then aggregates Sarva-Ashtakavarga via the existing
//! `SarvaAshtakavarga::add_planet` reduction.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::totals::calculate_bhinna_ashtakavarga;
use super::types::{PlanetAshtakavarga, SarvaAshtakavarga};
use crate::birth_chart::native::{build_native_chart, parse_birth_inputs};
use crate::birth_chart::types::Planet;
use crate::client::VedicApiClient;
use crate::error::VedicApiResult;

/// Request for Ashtakavarga calculation
#[derive(Debug, Clone, Serialize)]
pub struct AshtakavargaRequest {
    pub birth_date: String,
    pub birth_time: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl AshtakavargaRequest {
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

/// API response for Ashtakavarga (now `Serialize` too so callers
/// roundtrip JSON unchanged).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshtakavargaApiResponse {
    pub planets: Vec<AshtakavargaPlanetResponse>,
    pub sarva: SarvaResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AshtakavargaPlanetResponse {
    pub name: String,
    pub points: Vec<u8>,
    pub total: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarvaResponse {
    pub points: Vec<u8>,
    pub total: u16,
}

impl VedicApiClient {
    /// Compute Ashtakavarga (BAV per planet + SAV combined) natively.
    ///
    /// PR3: no HTTP call. Builds the native chart, runs
    /// `calculate_bhinna_ashtakavarga` per classical planet, aggregates
    /// SAV via the existing `SarvaAshtakavarga::add_planet` reduction,
    /// then emits the legacy response envelope.
    pub async fn get_ashtakavarga(
        &self,
        request: &AshtakavargaRequest,
    ) -> VedicApiResult<AshtakavargaApiResponse> {
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

        let mut sarva = SarvaAshtakavarga::empty();
        let mut planets_out: Vec<AshtakavargaPlanetResponse> = Vec::with_capacity(7);
        for &planet in &Planet::classical() {
            let bav = calculate_bhinna_ashtakavarga(planet, &chart);
            planets_out.push(AshtakavargaPlanetResponse {
                name: planet.to_string(),
                points: bav.sign_points.to_vec(),
                total: bav.total_points,
            });
            sarva.add_planet(bav);
        }

        Ok(AshtakavargaApiResponse {
            planets: planets_out,
            sarva: SarvaResponse {
                points: sarva.sarva_points.to_vec(),
                total: sarva.grand_total,
            },
        })
    }
}

/// Map API response to internal Sarva Ashtakavarga
pub fn map_ashtakavarga_response(response: AshtakavargaApiResponse) -> SarvaAshtakavarga {
    let mut sarva = SarvaAshtakavarga::empty();

    for planet in response.planets {
        let mut av = PlanetAshtakavarga::empty(&planet.name);

        for (i, points) in planet.points.iter().enumerate() {
            if i < 12 {
                av.sign_points[i] = *points;
            }
        }
        av.total_points = planet.total;

        sarva.planets.push(av);
    }

    for (i, points) in response.sarva.points.iter().enumerate() {
        if i < 12 {
            sarva.sarva_points[i] = *points;
        }
    }
    sarva.grand_total = response.sarva.total;

    sarva
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_ashtakavarga_request() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = AshtakavargaRequest::new(dt, 12.97, 77.59, 5.5);

        assert_eq!(request.birth_date, "1990-06-15");
    }

    fn test_client() -> VedicApiClient {
        VedicApiClient::new(crate::mocks::mock_config("http://localhost:0"))
    }

    /// Sanity-check: SAV grand total over the Bangalore fixture must not
    /// exceed the classical maximum of 337 (sum of all BAV row totals).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn native_ashtakavarga_bangalore_1991_sav_under_337() {
        let client = test_client();
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1991, 8, 13).unwrap(),
            NaiveTime::from_hms_opt(13, 31, 0).unwrap(),
        );
        let req = AshtakavargaRequest::new(dt, 12.9716, 77.5946, 5.5);
        let resp = client
            .get_ashtakavarga(&req)
            .await
            .expect("native ashtakavarga should succeed");

        // All 7 classical planets present, each with 12 sign points.
        assert_eq!(resp.planets.len(), 7);
        for p in &resp.planets {
            assert_eq!(p.points.len(), 12, "{} missing sign points", p.name);
            // Each sign point should be 0..=8.
            for (i, pts) in p.points.iter().enumerate() {
                assert!(*pts <= 8, "{} sign {}: {} > 8 max", p.name, i + 1, pts);
            }
        }

        // SAV grand total = sum of all per-planet totals; classical maximum is 337.
        assert!(
            resp.sarva.total <= 337,
            "SAV grand total {} exceeds classical max 337",
            resp.sarva.total
        );
        // And must be positive (some planet should have bindus for any
        // real chart).
        assert!(resp.sarva.total > 0);
    }

    /// The per-planet BAV total should match the published reference total
    /// regardless of input chart — that's a property of the table itself.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn per_planet_bav_totals_match_published_references() {
        let client = test_client();
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1991, 8, 13).unwrap(),
            NaiveTime::from_hms_opt(13, 31, 0).unwrap(),
        );
        let req = AshtakavargaRequest::new(dt, 12.9716, 77.5946, 5.5);
        let resp = client
            .get_ashtakavarga(&req)
            .await
            .expect("native ashtakavarga should succeed");
        let expected: std::collections::HashMap<&'static str, u8> = [
            ("Sun", 48),
            ("Moon", 49),
            ("Mars", 39),
            ("Mercury", 54),
            ("Jupiter", 56),
            ("Venus", 52),
            ("Saturn", 39),
        ]
        .into_iter()
        .collect();
        for p in &resp.planets {
            let want = expected.get(p.name.as_str()).copied().expect("planet name");
            assert_eq!(p.total, want, "{} BAV total mismatch", p.name);
        }
    }
}
