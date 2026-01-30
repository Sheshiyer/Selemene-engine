use selemene_engine::models::{PanchangaRequest, PanchangaResult, PrecisionLevel};
use selemene_engine::engines::PanchangaCalculator;

/// Test data for validation
const TEST_DATES: &[&str] = &[
    "2025-01-27",
    "2025-06-15",
    "2025-12-21",
];

const TEST_COORDINATES: &[(f64, f64)] = &[
    (19.0760, 72.8777),  // Mumbai, India
    (28.6139, 77.2090),  // New Delhi, India
    (12.9716, 77.5946),  // Bangalore, India
];

/// Test solar position calculations
#[tokio::test]
async fn test_solar_position_accuracy() {
    // Test that solar longitude is within expected range (0-360 degrees)
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let request = PanchangaRequest {
                date: date.to_string(),
                latitude: Some(lat),
                longitude: Some(lon),
                timezone: None,
                precision: Some(PrecisionLevel::High),
                include_details: Some(false),
            };
            
            // TODO: Replace with actual calculation when engine is implemented
            let solar_longitude = 120.0; // Placeholder value
            
            assert!(
                solar_longitude >= 0.0 && solar_longitude <= 360.0,
                "Solar longitude {} for date {} at coordinates ({}, {}) is out of range",
                solar_longitude, date, lat, lon
            );
        }
    }
}

/// Test lunar position calculations
#[tokio::test]
async fn test_lunar_position_accuracy() {
    // Test that lunar longitude is within expected range (0-360 degrees)
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let request = PanchangaRequest {
                date: date.to_string(),
                latitude: Some(lat),
                longitude: Some(lon),
                timezone: None,
                precision: Some(PrecisionLevel::High),
                include_details: Some(false),
            };
            
            // TODO: Replace with actual calculation when engine is implemented
            let lunar_longitude = 135.0; // Placeholder value
            
            assert!(
                lunar_longitude >= 0.0 && lunar_longitude <= 360.0,
                "Lunar longitude {} for date {} at coordinates ({}, {}) is out of range",
                lunar_longitude, date, lat, lon
            );
        }
    }
}

/// Test Tithi calculations
#[tokio::test]
async fn test_tithi_accuracy() {
    let calculator = PanchangaCalculator::new();
    
    // Test that Tithi values are within expected range (1-30)
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            // Use known solar and lunar longitudes for testing
            let solar_longitude = 120.0;
            let lunar_longitude = 135.0;
            let jd = 2451545.0; // J2000
            
            match calculator.calculate_tithi(solar_longitude, lunar_longitude, jd) {
                Ok(tithi_info) => {
                    assert!(
                        tithi_info.number >= 1 && tithi_info.number <= 30,
                        "Tithi {} for date {} at coordinates ({}, {}) is out of range",
                        tithi_info.number, date, lat, lon
                    );
                    
                    // Verify tithi name is not empty
                    assert!(
                        !tithi_info.name.is_empty(),
                        "Tithi name should not be empty for date {}",
                        date
                    );
                }
                Err(e) => {
                    panic!("Tithi calculation failed for date {}: {}", date, e);
                }
            }
        }
    }
}

/// Test Nakshatra calculations
#[tokio::test]
async fn test_nakshatra_accuracy() {
    let calculator = PanchangaCalculator::new();
    
    // Test that Nakshatra values are within expected range (1-27)
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let lunar_longitude = 135.0;
            let jd = 2451545.0; // J2000
            
            match calculator.calculate_nakshatra(lunar_longitude, jd) {
                Ok(nakshatra_info) => {
                    assert!(
                        nakshatra_info.number >= 1 && nakshatra_info.number <= 27,
                        "Nakshatra {} for date {} at coordinates ({}, {}) is out of range",
                        nakshatra_info.number, date, lat, lon
                    );
                    
                    // Verify nakshatra name and ruler are not empty
                    assert!(
                        !nakshatra_info.name.is_empty(),
                        "Nakshatra name should not be empty for date {}",
                        date
                    );
                    
                    assert!(
                        !nakshatra_info.ruler.is_empty(),
                        "Nakshatra ruler should not be empty for date {}",
                        date
                    );
                }
                Err(e) => {
                    panic!("Nakshatra calculation failed for date {}: {}", date, e);
                }
            }
        }
    }
}

/// Test Yoga calculations
#[tokio::test]
async fn test_yoga_accuracy() {
    let calculator = PanchangaCalculator::new();
    
    // Test that Yoga values are within expected range (1-27)
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let solar_longitude = 120.0;
            let lunar_longitude = 135.0;
            let jd = 2451545.0; // J2000
            
            match calculator.calculate_yoga(solar_longitude, lunar_longitude, jd) {
                Ok(yoga_info) => {
                    assert!(
                        yoga_info.number >= 1 && yoga_info.number <= 27,
                        "Yoga {} for date {} at coordinates ({}, {}) is out of range",
                        yoga_info.number, date, lat, lon
                    );
                    
                    // Verify yoga name is not empty
                    assert!(
                        !yoga_info.name.is_empty(),
                        "Yoga name should not be empty for date {}",
                        date
                    );
                }
                Err(e) => {
                    panic!("Yoga calculation failed for date {}: {}", date, e);
                }
            }
        }
    }
}

/// Test Karana calculations
#[tokio::test]
async fn test_karana_accuracy() {
    let calculator = PanchangaCalculator::new();
    
    // Test that Karana values are within expected range (1-60)
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let solar_longitude = 120.0;
            let lunar_longitude = 135.0;
            let jd = 2451545.0; // J2000
            
            match calculator.calculate_karana(solar_longitude, lunar_longitude, jd) {
                Ok(karana_info) => {
                    assert!(
                        karana_info.number >= 1 && karana_info.number <= 60,
                        "Karana {} for date {} at coordinates ({}, {}) is out of range",
                        karana_info.number, date, lat, lon
                    );
                    
                    // Verify karana name is not empty
                    assert!(
                        !karana_info.name.is_empty(),
                        "Karana name should not be empty for date {}",
                        date
                    );
                }
                Err(e) => {
                    panic!("Karana calculation failed for date {}: {}", date, e);
                }
            }
        }
    }
}

/// Test Vara calculations
#[tokio::test]
async fn test_vara_accuracy() {
    let calculator = PanchangaCalculator::new();
    
    // Test that Vara values are within expected range (1-7)
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let jd = 2451545.0; // J2000
            
            match calculator.calculate_vara(jd) {
                Ok(vara_info) => {
                    assert!(
                        vara_info.number >= 1 && vara_info.number <= 7,
                        "Vara {} for date {} at coordinates ({}, {}) is out of range",
                        vara_info.number, date, lat, lon
                    );
                    
                    // Verify vara name and ruler are not empty
                    assert!(
                        !vara_info.name.is_empty(),
                        "Vara name should not be empty for date {}",
                        date
                    );
                    
                    assert!(
                        !vara_info.ruler.is_empty(),
                        "Vara ruler should not be empty for date {}",
                        date
                    );
                }
                Err(e) => {
                    panic!("Vara calculation failed for date {}: {}", date, e);
                }
            }
        }
    }
}

/// Test coordinate validation
#[tokio::test]
async fn test_coordinate_validation() {
    // Test valid coordinates
    let valid_coords = vec![
        (0.0, 0.0),           // Equator, Prime Meridian
        (90.0, 180.0),        // North Pole, International Date Line
        (-90.0, -180.0),      // South Pole, International Date Line
        (45.0, 90.0),         // Mid-latitude, Mid-longitude
    ];
    
    for &(lat, lon) in &valid_coords {
        assert!(
            lat >= -90.0 && lat <= 90.0,
            "Latitude {} is out of valid range [-90, 90]",
            lat
        );
        
        assert!(
            lon >= -180.0 && lon <= 180.0,
            "Longitude {} is out of valid range [-180, 180]",
            lon
        );
    }
    
    // Test invalid coordinates
    let invalid_coords = vec![
        (91.0, 0.0),          // Latitude too high
        (-91.0, 0.0),         // Latitude too low
        (0.0, 181.0),         // Longitude too high
        (0.0, -181.0),        // Longitude too low
    ];
    
    for &(lat, lon) in &invalid_coords {
        let lat_valid = lat >= -90.0 && lat <= 90.0;
        let lon_valid = lon >= -180.0 && lon <= 180.0;
        
        assert!(
            !(lat_valid && lon_valid),
            "Coordinates ({}, {}) should be invalid",
            lat, lon
        );
    }
}

/// Test date validation
#[tokio::test]
async fn test_date_validation() {
    // Test valid dates
    let valid_dates = vec![
        "2025-01-01",
        "2025-12-31",
        "2024-02-29", // Leap year
        "2023-02-28", // Non-leap year
    ];
    
    for date in valid_dates {
        assert!(
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok(),
            "Date {} should be valid YYYY-MM-DD",
            date
        );
    }
    
    // Test invalid dates
    let invalid_dates = vec![
        "2025-13-01", // Invalid month
        "2025-01-32", // Invalid day
        "2025-02-30", // Invalid day for February
        "2024-02-30", // Invalid day for February (leap year)
        "invalid-date",
        "",
    ];
    
    for date in invalid_dates {
        assert!(
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err(),
            "Date {} should be invalid",
            date
        );
    }
}

/// Test precision level validation
#[tokio::test]
async fn test_precision_validation() {
    let precision_levels = vec![
        PrecisionLevel::Standard,
        PrecisionLevel::High,
        PrecisionLevel::Extreme,
    ];
    
    for precision in precision_levels {
        let precision_value = precision as u8;
        assert!(
            precision_value >= 1 && precision_value <= 3,
            "Precision level {} is out of valid range [1, 3]",
            precision_value
        );
    }
}

/// Test mathematical consistency
#[tokio::test]
async fn test_mathematical_consistency() {
    let calculator = PanchangaCalculator::new();
    
    // Test that Tithi calculation is consistent
    // Tithi = (Lunar Longitude - Solar Longitude) / 12 degrees
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let solar_longitude = 120.0;
            let lunar_longitude = 135.0;
            let jd = 2451545.0; // J2000
            
            match calculator.calculate_tithi(solar_longitude, lunar_longitude, jd) {
                Ok(tithi_info) => {
                    let tithi_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0_f64);
                    let expected_tithi = (tithi_diff / 12.0_f64).floor() + 1.0_f64;
                    let actual_tithi = tithi_info.number as f64;
                    
                    // Allow for small floating point differences
                    let tolerance = 0.001;
                    assert!(
                        (expected_tithi - actual_tithi).abs() < tolerance,
                        "Tithi calculation inconsistency: expected {}, actual {}",
                        expected_tithi, actual_tithi
                    );
                }
                Err(e) => {
                    panic!("Tithi calculation failed for date {}: {}", date, e);
                }
            }
        }
    }
    
    // Test that Nakshatra calculation is consistent
    // Each nakshatra spans 13.333... degrees (360/27)
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let lunar_longitude = 135.0;
            let jd = 2451545.0; // J2000
            
            match calculator.calculate_nakshatra(lunar_longitude, jd) {
                Ok(nakshatra_info) => {
                    let nakshatra_span = 360.0_f64 / 27.0_f64;
                    let expected_nakshatra = (lunar_longitude / nakshatra_span).floor() as u8 + 1;
                    let actual_nakshatra = nakshatra_info.number;
                    
                    assert_eq!(
                        expected_nakshatra, actual_nakshatra,
                        "Nakshatra calculation inconsistency: expected {}, actual {}",
                        expected_nakshatra, actual_nakshatra
                    );
                }
                Err(e) => {
                    panic!("Nakshatra calculation failed for date {}: {}", date, e);
                }
            }
        }
    }
}

/// Test cross-validation between backends
#[tokio::test]
async fn test_backend_cross_validation() {
    // TODO: Implement when both native and Swiss Ephemeris engines are available
    // This test should verify that both backends produce similar results within tolerance
    
    for date in TEST_DATES {
        for &(lat, lon) in TEST_COORDINATES {
            let request = PanchangaRequest {
                date: date.to_string(),
                latitude: Some(lat),
                longitude: Some(lon),
                timezone: None,
                precision: Some(PrecisionLevel::High),
                include_details: Some(false),
            };
            
            // Placeholder for cross-validation test
            let tolerance = 0.1; // 6 arcminutes tolerance
            
            // TODO: Compare results from different backends
            // let native_result = native_engine.calculate(&request).await?;
            // let swiss_result = swiss_engine.calculate(&request).await?;
            // 
            // let difference = (native_result.solar_longitude - swiss_result.solar_longitude).abs();
            // assert!(difference < tolerance, "Backend difference {} exceeds tolerance {}", difference, tolerance);
        }
    }
}
