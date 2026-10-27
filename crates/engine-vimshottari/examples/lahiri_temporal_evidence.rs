//! Emit a local patched-engine packet; no hosted service or fixed chart runs.
use chrono::{DateTime, Utc};
use engine_human_design::ephemeris::{lock_ephemeris, EphemerisCalculator};
use engine_vimshottari::{ConsciousnessEngine, EngineInput, VimshottariEngine};
use libswisseph_sys::{
    tuple_result::{
        create::{swe_calc_ut, swe_version},
        simple::{swe_julday, swe_set_sid_mode},
    },
    SEFLG_SIDEREAL, SEFLG_SPEED, SEFLG_SWIEPH, SE_GREG_CAL, SE_MOON, SE_SIDM_LAHIRI,
};
use noesis_core::{BirthData, Precision};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let as_of = DateTime::parse_from_rfc3339("2026-10-24T04:43:21Z")?.with_timezone(&Utc);
    let output = VimshottariEngine::new()
        .calculate(EngineInput {
            birth_data: Some(BirthData {
                name: Some("Shesh Iyer".into()),
                date: "1991-08-13".into(),
                time: Some("13:31".into()),
                latitude: 12.9716,
                longitude: 77.5946,
                timezone: "Asia/Kolkata".into(),
            }),
            current_time: as_of,
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        })
        .await?;
    let calculator = EphemerisCalculator::new("");
    let (version, flags, moon) = {
        let _guard = lock_ephemeris()
            .map_err(|_| "Swiss Ephemeris mutex poisoned")?;
        // SAFETY: the shared mutex protects global mode, C state and buffers;
        // tuple_result owns buffers and the date is a fixed valid reference.
        unsafe {
            let version = swe_version();
            let jd = swe_julday(1991, 8, 13, 8.0 + 1.0 / 60.0, SE_GREG_CAL as i32);
            swe_set_sid_mode(SE_SIDM_LAHIRI as i32, 0.0, 0.0);
            let (flags, coordinates) = swe_calc_ut(
                jd,
                SE_MOON as i32,
                (SEFLG_SWIEPH | SEFLG_SPEED | SEFLG_SIDEREAL) as i32,
            )
            .map_err(|(code, message)| format!("Swiss error {code}: {message}"))?;
            (version, flags, coordinates[0])
        }
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema_version": "1.0", "scope": "local patched Rust engine; no hosted calculation or deployment", "as_of": as_of,
            "method": {"id": "lahiri-full-v1", "swiss_ephemeris_version": version, "returned_flags": flags, "ephemeris_path": calculator.data_path(), "reference_moon_longitude": moon},
            "engine_output": output,
        }))?
    );
    Ok(())
}
