use selemene_engine::engines::PanchangaCalculator;

/// Test Panchanga calculator functionality
#[test]
fn test_panchanga_calculator_basic() {
    let calculator = PanchangaCalculator::new();
    
    // Test Tithi calculation
    let solar_longitude = 120.0;
    let lunar_longitude = 135.0;
    let jd = 2451545.0; // J2000
    
    // Test Tithi
    match calculator.calculate_tithi(solar_longitude, lunar_longitude, jd) {
        Ok(tithi_info) => {
            assert!(tithi_info.number >= 1 && tithi_info.number <= 30);
            assert!(!tithi_info.name.is_empty());
            println!("Tithi: {} - {}", tithi_info.number, tithi_info.name);
        }
        Err(e) => panic!("Tithi calculation failed: {}", e),
    }
    
    // Test Nakshatra
    match calculator.calculate_nakshatra(lunar_longitude, jd) {
        Ok(nakshatra_info) => {
            assert!(nakshatra_info.number >= 1 && nakshatra_info.number <= 27);
            assert!(!nakshatra_info.name.is_empty());
            assert!(!nakshatra_info.ruler.is_empty());
            println!("Nakshatra: {} - {} (Ruler: {})", nakshatra_info.number, nakshatra_info.name, nakshatra_info.ruler);
        }
        Err(e) => panic!("Nakshatra calculation failed: {}", e),
    }
    
    // Test Yoga
    match calculator.calculate_yoga(solar_longitude, lunar_longitude, jd) {
        Ok(yoga_info) => {
            assert!(yoga_info.number >= 1 && yoga_info.number <= 27);
            assert!(!yoga_info.name.is_empty());
            println!("Yoga: {} - {}", yoga_info.number, yoga_info.name);
        }
        Err(e) => panic!("Yoga calculation failed: {}", e),
    }
    
    // Test Karana
    match calculator.calculate_karana(solar_longitude, lunar_longitude, jd) {
        Ok(karana_info) => {
            assert!(karana_info.number >= 1 && karana_info.number <= 60);
            assert!(!karana_info.name.is_empty());
            println!("Karana: {} - {}", karana_info.number, karana_info.name);
        }
        Err(e) => panic!("Karana calculation failed: {}", e),
    }
    
    // Test Vara
    match calculator.calculate_vara(jd) {
        Ok(vara_info) => {
            assert!(vara_info.number >= 1 && vara_info.number <= 7);
            assert!(!vara_info.name.is_empty());
            assert!(!vara_info.ruler.is_empty());
            println!("Vara: {} - {} (Ruler: {})", vara_info.number, vara_info.name, vara_info.ruler);
        }
        Err(e) => panic!("Vara calculation failed: {}", e),
    }
}

/// Test mathematical consistency
#[test]
fn test_mathematical_consistency() {
    let calculator = PanchangaCalculator::new();
    
    // Test Tithi calculation consistency
    let solar_longitude = 120.0;
    let lunar_longitude = 135.0;
    let jd = 2451545.0;
    
    match calculator.calculate_tithi(solar_longitude, lunar_longitude, jd) {
        Ok(tithi_info) => {
            let tithi_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0_f64);
            let expected_tithi = (tithi_diff / 12.0_f64).floor() + 1.0_f64;
            let actual_tithi = tithi_info.number as f64;
            
            let tolerance = 0.001;
            assert!(
                (expected_tithi - actual_tithi).abs() < tolerance,
                "Tithi calculation inconsistency: expected {}, actual {}",
                expected_tithi, actual_tithi
            );
        }
        Err(e) => panic!("Tithi calculation failed: {}", e),
    }
    
    // Test Nakshatra calculation consistency
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
        Err(e) => panic!("Nakshatra calculation failed: {}", e),
    }
}

/// Test edge cases
#[test]
fn test_edge_cases() {
    let calculator = PanchangaCalculator::new();
    
    // Test with zero longitudes
    let jd = 2451545.0;
    
    match calculator.calculate_tithi(0.0, 0.0, jd) {
        Ok(tithi_info) => {
            assert_eq!(tithi_info.number, 1);
            println!("Zero longitudes Tithi: {}", tithi_info.number);
        }
        Err(e) => panic!("Zero longitude Tithi calculation failed: {}", e),
    }
    
    // Test with maximum longitudes
    match calculator.calculate_tithi(359.0, 1.0, jd) {
        Ok(tithi_info) => {
            assert!(tithi_info.number >= 1 && tithi_info.number <= 30);
            println!("Max longitudes Tithi: {}", tithi_info.number);
        }
        Err(e) => panic!("Max longitude Tithi calculation failed: {}", e),
    }
}
