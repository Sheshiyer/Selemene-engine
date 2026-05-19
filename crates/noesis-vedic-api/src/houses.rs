//! Thin typed wrapper around FreeAstrologyAPI's `POST /western/houses`
//! endpoint.
//!
//! Despite the "western" route name, the endpoint accepts the same
//! `config.ayanamsha: "lahiri"` knob as `/planets` and emits sidereal cusps
//! when Lahiri is requested. The response shape is:
//!
//! ```json
//! { "statusCode": 200,
//!   "output": {
//!     "Houses": [
//!       { "House": 1, "degree": 222.118, "normDegree": 12.118,
//!         "zodiac_sign": { "number": 8, "name": { "en": "Scorpio" } } },
//!       ...
//!     ]
//!   } }
//! ```
//!
//! `zodiac_sign.number` is 1-indexed (1 = Aries, 12 = Pisces).

use serde_json::Value;

use crate::chart::ZodiacSign;
use crate::chart_mapping::{require_f64, require_u64};
use crate::client::VedicApiClient;
use crate::error::{VedicApiError, VedicApiResult};
use crate::types::BirthData;

/// Typed projection of `/western/houses` for the 12 cusps.
#[derive(Debug, Clone, PartialEq)]
pub struct HousesApiResponse {
    pub houses: Vec<HouseCusp>,
}

/// One house cusp.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseCusp {
    /// 1..=12.
    pub house: u8,
    /// Absolute ecliptic longitude of the cusp (sidereal when ayanamsha=lahiri).
    pub degree: f64,
    /// Degree within the cusp's sign (0..30).
    pub norm_degree: f64,
    /// Sign on the cusp.
    pub sign: ZodiacSign,
}

/// Fetch and parse the 12 house cusps for the given birth data. Flows
/// through the same retry / circuit-breaker path as every other live call.
pub async fn fetch_houses_with(
    client: &VedicApiClient,
    b: &BirthData,
) -> VedicApiResult<HousesApiResponse> {
    let raw = client.get_western_houses_raw_with(b).await?;
    map_houses_envelope(&raw)
}

/// 9-positional-arg shim for [`fetch_houses_with`].
pub async fn fetch_houses(
    client: &VedicApiClient,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    lat: f64,
    lng: f64,
    tzone: f64,
) -> VedicApiResult<HousesApiResponse> {
    fetch_houses_with(
        client,
        &BirthData::new(year, month, day, hour, minute, second, lat, lng, tzone),
    )
    .await
}

/// Map a captured `/western/houses` JSON envelope into the typed response.
///
/// Exposed for tests and offline use.
pub fn map_houses_envelope(raw: &Value) -> VedicApiResult<HousesApiResponse> {
    let arr = raw
        .get("output")
        .and_then(|o| o.get("Houses"))
        .and_then(Value::as_array)
        .ok_or_else(|| VedicApiError::Parse {
            message: "/western/houses envelope missing output.Houses array".to_string(),
        })?;

    let mut houses = Vec::with_capacity(arr.len());
    for entry in arr {
        let house = require_u64(entry, "House", "house entry")? as u8;
        let ctx = format!("house {}", house);
        let degree = require_f64(entry, "degree", &ctx)?;
        let norm_degree = require_f64(entry, "normDegree", &ctx)?;
        let sign_num = entry
            .get("zodiac_sign")
            .and_then(|z| z.get("number"))
            .and_then(Value::as_u64)
            .ok_or_else(|| VedicApiError::Parse {
                message: format!("{}: missing zodiac_sign.number", ctx),
            })? as u8;
        let sign = ZodiacSign::from_number(sign_num).ok_or_else(|| VedicApiError::Parse {
            message: format!("{}: out-of-range zodiac_sign.number={}", ctx, sign_num),
        })?;

        houses.push(HouseCusp {
            house,
            degree,
            norm_degree,
            sign,
        });
    }

    Ok(HousesApiResponse { houses })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_fixture(name: &str) -> Value {
        let path = format!(
            "{}/tests/fixtures/captures/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path, e));
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|e| panic!("failed to parse fixture {}: {}", path, e))
    }

    #[test]
    fn test_maps_western_houses_fixture() {
        let raw = load_fixture("western_houses_bangalore_1991-08-13.json");
        let parsed = map_houses_envelope(&raw).expect("must parse");
        assert_eq!(parsed.houses.len(), 12);
        assert_eq!(parsed.houses[0].house, 1);
        assert_eq!(parsed.houses[0].sign, ZodiacSign::Scorpio); // zodiac_sign.number=8

        // 7th house must be 2 (Taurus) per the fixture
        let seventh = parsed.houses.iter().find(|h| h.house == 7).unwrap();
        assert_eq!(seventh.sign, ZodiacSign::Taurus);
    }

    #[test]
    fn test_rejects_missing_houses_array() {
        let raw: Value = serde_json::json!({"output": {}});
        let err = map_houses_envelope(&raw).unwrap_err();
        assert!(matches!(err, VedicApiError::Parse { .. }));
    }
}
