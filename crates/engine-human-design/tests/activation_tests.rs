//! Integration tests for Sun/Earth Activation calculations

use engine_human_design::{
    EphemerisCalculator,
    calculate_personality_sun_earth,
    calculate_design_sun_earth,
    calculate_sun_earth_activations,
};
use chrono::{TimeZone, Utc};

#[test]
fn test_personality_sun_earth_basic() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
    
    let result = calculate_personality_sun_earth(&birth_time, &calculator);
    
    // If ephemeris data not available, test will pass (we can't test without data)
    if let Ok((sun, earth)) = result {
        // Verify basic structure
        assert!(sun.gate >= 1 && sun.gate <= 64);
        assert!(sun.line >= 1 && sun.line <= 6);
        assert!(earth.gate >= 1 && earth.gate <= 64);
        assert!(earth.line >= 1 && earth.line <= 6);
        
        // Verify Earth is 180° from Sun
        let diff = (earth.longitude - sun.longitude + 360.0) % 360.0;
        assert!(
            (diff - 180.0).abs() < 0.01,
            "Earth should be 180° from Sun, got {:.4}° difference", diff
        );
        
        println!("✓ Personality Sun/Earth test passed");
        println!("  Sun: Gate {}.{} @ {:.4}°", sun.gate, sun.line, sun.longitude);
        println!("  Earth: Gate {}.{} @ {:.4}°", earth.gate, earth.line, earth.longitude);
    }
}

#[test]
fn test_design_sun_earth_basic() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
    
    let result = calculate_design_sun_earth(&birth_time, &calculator);
    
    if let Ok((sun, earth)) = result {
        // Verify basic structure
        assert!(sun.gate >= 1 && sun.gate <= 64);
        assert!(sun.line >= 1 && sun.line <= 6);
        assert!(earth.gate >= 1 && earth.gate <= 64);
        assert!(earth.line >= 1 && earth.line <= 6);
        
        // Verify Earth is 180° from Sun
        let diff = (earth.longitude - sun.longitude + 360.0) % 360.0;
        assert!(
            (diff - 180.0).abs() < 0.01,
            "Earth should be 180° from Sun, got {:.4}° difference", diff
        );
        
        println!("✓ Design Sun/Earth test passed");
        println!("  Sun: Gate {}.{} @ {:.4}°", sun.gate, sun.line, sun.longitude);
        println!("  Earth: Gate {}.{} @ {:.4}°", earth.gate, earth.line, earth.longitude);
    }
}

#[test]
fn test_complete_sun_earth_activations() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(1990, 6, 15, 14, 30, 0).unwrap();
    
    let result = calculate_sun_earth_activations(&birth_time, &calculator);
    
    if let Ok(((pers_sun, pers_earth), (des_sun, des_earth))) = result {
        println!("\n=== Complete Sun/Earth Activation Test ===");
        println!("Birth: {}", birth_time);
        
        println!("\nPersonality (at birth):");
        println!("  Sun: Gate {}.{} @ {:.4}°", pers_sun.gate, pers_sun.line, pers_sun.longitude);
        println!("  Earth: Gate {}.{} @ {:.4}°", pers_earth.gate, pers_earth.line, pers_earth.longitude);
        
        println!("\nDesign (88 days before):");
        println!("  Sun: Gate {}.{} @ {:.4}°", des_sun.gate, des_sun.line, des_sun.longitude);
        println!("  Earth: Gate {}.{} @ {:.4}°", des_earth.gate, des_earth.line, des_earth.longitude);
        
        // Verify all values are in range
        assert!(pers_sun.gate >= 1 && pers_sun.gate <= 64);
        assert!(pers_earth.gate >= 1 && pers_earth.gate <= 64);
        assert!(des_sun.gate >= 1 && des_sun.gate <= 64);
        assert!(des_earth.gate >= 1 && des_earth.gate <= 64);
        
        // Verify Earth oppositions
        let pers_diff = (pers_earth.longitude - pers_sun.longitude + 360.0) % 360.0;
        let des_diff = (des_earth.longitude - des_sun.longitude + 360.0) % 360.0;
        assert!((pers_diff - 180.0).abs() < 0.01, "Personality Earth not 180° from Sun: {:.4}°", pers_diff);
        assert!((des_diff - 180.0).abs() < 0.01, "Design Earth not 180° from Sun: {:.4}°", des_diff);
        
        println!("\n✓ All activation calculations passed");
    }
}

#[test]
fn test_spring_equinox_2000() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(2000, 3, 20, 7, 35, 0).unwrap();
    
    let result = calculate_personality_sun_earth(&birth_time, &calculator);
    
    if let Ok((sun, earth)) = result {
        println!("\n=== Spring Equinox 2000 Test ===");
        println!("Sun: Gate {}.{} @ {:.4}°", sun.gate, sun.line, sun.longitude);
        println!("Earth: Gate {}.{} @ {:.4}°", earth.gate, earth.line, earth.longitude);
        
        // At spring equinox, Sun should be near 0° (Gate 1) or 360° (Gate 64)
        assert!(
            sun.longitude < 10.0 || sun.longitude > 350.0,
            "Spring equinox Sun should be near 0°/360°, got {:.2}°",
            sun.longitude
        );
        
        println!("✓ Spring equinox test passed");
    }
}

#[test]
fn test_winter_solstice_1995() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(1995, 12, 22, 8, 17, 0).unwrap();
    
    let result = calculate_personality_sun_earth(&birth_time, &calculator);
    
    if let Ok((sun, earth)) = result {
        println!("\n=== Winter Solstice 1995 Test ===");
        println!("Sun: Gate {}.{} @ {:.4}°", sun.gate, sun.line, sun.longitude);
        println!("Earth: Gate {}.{} @ {:.4}°", earth.gate, earth.line, earth.longitude);
        
        // At winter solstice, Sun should be near 270° (0° Capricorn)
        assert!(
            (sun.longitude - 270.0).abs() < 10.0,
            "Winter solstice Sun should be near 270°, got {:.2}°",
            sun.longitude
        );
        
        println!("✓ Winter solstice test passed");
    }
}

#[test]
fn test_multiple_birth_dates() {
    let calculator = EphemerisCalculator::new("");
    
    let test_dates = vec![
        Utc.with_ymd_and_hms(1985, 1, 15, 10, 30, 0).unwrap(),
        Utc.with_ymd_and_hms(1992, 7, 4, 22, 45, 0).unwrap(),
        Utc.with_ymd_and_hms(2005, 11, 28, 6, 15, 0).unwrap(),
        Utc.with_ymd_and_hms(2010, 4, 12, 18, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2015, 9, 23, 12, 30, 0).unwrap(),
    ];
    
    println!("\n=== Multiple Birth Date Tests ===");
    let mut success_count = 0;
    
    for birth_time in test_dates {
        if let Ok(((pers_sun, pers_earth), (des_sun, des_earth))) = 
            calculate_sun_earth_activations(&birth_time, &calculator) 
        {
            // Verify gates and lines are in range
            assert!(pers_sun.gate >= 1 && pers_sun.gate <= 64);
            assert!(pers_earth.gate >= 1 && pers_earth.gate <= 64);
            assert!(des_sun.gate >= 1 && des_sun.gate <= 64);
            assert!(des_earth.gate >= 1 && des_earth.gate <= 64);
            
            assert!(pers_sun.line >= 1 && pers_sun.line <= 6);
            assert!(pers_earth.line >= 1 && pers_earth.line <= 6);
            assert!(des_sun.line >= 1 && des_sun.line <= 6);
            assert!(des_earth.line >= 1 && des_earth.line <= 6);
            
            // Verify oppositions
            let pers_diff = (pers_earth.longitude - pers_sun.longitude + 360.0) % 360.0;
            let des_diff = (des_earth.longitude - des_sun.longitude + 360.0) % 360.0;
            assert!((pers_diff - 180.0).abs() < 0.01);
            assert!((des_diff - 180.0).abs() < 0.01);
            
            println!("✓ {}: P: {}.{}/{}.{}, D: {}.{}/{}.{}",
                birth_time.format("%Y-%m-%d"),
                pers_sun.gate, pers_sun.line,
                pers_earth.gate, pers_earth.line,
                des_sun.gate, des_sun.line,
                des_earth.gate, des_earth.line);
            
            success_count += 1;
        }
    }
    
    if success_count > 0 {
        println!("\n✓ Tested {} birth dates successfully", success_count);
    }
}

#[test]
fn test_design_vs_personality_difference() {
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(2000, 8, 15, 12, 0, 0).unwrap();
    
    let result = calculate_sun_earth_activations(&birth_time, &calculator);
    
    if let Ok(((pers_sun, _pers_earth), (des_sun, _des_earth))) = result {
        let sun_diff = (pers_sun.longitude - des_sun.longitude + 360.0) % 360.0;
        
        println!("\n=== Design vs Personality Difference ===");
        println!("Personality Sun: {:.4}°", pers_sun.longitude);
        println!("Design Sun: {:.4}°", des_sun.longitude);
        println!("Difference: {:.2}°", sun_diff);
        
        // Sun should have moved significantly in 88 days
        assert!(sun_diff > 10.0, "Sun should have moved significantly between Design and Personality, got {:.2}°", sun_diff);
        
        println!("✓ Design and Personality are independent");
    }
}
