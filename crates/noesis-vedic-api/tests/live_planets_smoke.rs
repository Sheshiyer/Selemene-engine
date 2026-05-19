//! Live smoke test for the typed `/planets` path.
//!
//! Gated by both the `real-api` Cargo feature and an `#[ignore]` attribute so
//! it never runs in default CI. To exercise:
//!
//! ```sh
//! FREE_ASTROLOGY_API_KEY=… cargo test --package noesis-vedic-api \
//!   --features real-api --test live_planets_smoke -- --ignored
//! ```
//!
//! The test reads the API key from the environment and skips gracefully if
//! it is absent — never panics on a missing key, so accidentally enabling
//! the feature in CI does not turn the suite red.

#![cfg(feature = "real-api")]

use noesis_vedic_api::chart::ZodiacSign;
use noesis_vedic_api::{Config, VedicApiClient};

#[tokio::test]
#[ignore]
async fn live_planets_bangalore_1991_08_13() {
    let Ok(api_key) = std::env::var("FREE_ASTROLOGY_API_KEY") else {
        eprintln!("FREE_ASTROLOGY_API_KEY not set — skipping live smoke test");
        return;
    };

    let config = Config::new(&api_key);
    let client = VedicApiClient::new(config);

    let chart = client
        .get_birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
        .await
        .expect("live /planets call must succeed (HTTP 200) for Bangalore 1991-08-13");

    // Ascendant current_sign == 8 → Scorpio in the 1-indexed scheme.
    assert_eq!(
        chart.ascendant.sign,
        ZodiacSign::Scorpio,
        "Bangalore 1991-08-13 13:31:00 IST Lahiri Ascendant must be Scorpio"
    );

    // Sanity: we got 12 planet entries (Sun..Pluto + Rahu/Ketu/Uranus/Neptune).
    assert_eq!(chart.planets.len(), 12, "expected 12 planet entries");
}
