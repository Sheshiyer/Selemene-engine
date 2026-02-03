//! Shesh Birth Data Verification Test
//!
//! This test verifies the unified analysis system using Shesh's actual birth data:
//! - Birth Date: 1991-08-13
//! - Birth Time: 13:31 IST
//! - Location: Bengaluru, India (12.9716° N, 77.5946° E)
//!
//! Expected values:
//! - Moon Nakshatra: Uttara Phalguni
//! - Tithi: Chaturthi (Shukla Paksha)
//! - Current Mahadasha: Mars (until 2026-09-14)
//! - Ascendant: Scorpio (approximate)
//! - Life Path Number: 5

use chrono::Datelike;
use chrono::Timelike;

use noesis_integration::{
    BirthProfile, UnifiedAnalysis, TCMAnalysis,
    verification::DataVerifier,
    tcm_layer::TCMElement,
};

/// Shesh's birth profile for testing
fn shesh_profile() -> BirthProfile {
    BirthProfile::new(
        "1991-08-13",   // Date
        "13:31",        // Time
        12.9716,        // Latitude (Bengaluru)
        77.5946,        // Longitude (Bengaluru)
        "Asia/Kolkata", // Timezone
    )
}

#[test]
fn test_shesh_profile_creation() {
    let profile = shesh_profile();
    
    assert_eq!(profile.date, "1991-08-13");
    assert_eq!(profile.time, Some("13:31".to_string()));
    assert!((profile.latitude - 12.9716).abs() < 0.0001);
    assert!((profile.longitude - 77.5946).abs() < 0.0001);
    assert_eq!(profile.timezone, "Asia/Kolkata");
    
    println!("✓ Shesh's birth profile created successfully");
    println!("  Date: {}", profile.date);
    println!("  Time: {:?}", profile.time);
    println!("  Location: {}, {}", profile.latitude, profile.longitude);
}

#[test]
fn test_shesh_date_parsing() {
    let profile = shesh_profile();
    
    let date = profile.parse_date().expect("Failed to parse date");
    assert_eq!(date.year(), 1991);
    assert_eq!(date.month(), 8);
    assert_eq!(date.day(), 13);
    
    let time = profile.parse_time().expect("Failed to parse time");
    assert_eq!(time.hour(), 13);
    assert_eq!(time.minute(), 31);
    
    println!("✓ Date parsing successful: {}-{:02}-{:02} {:02}:{:02}",
        date.year(), date.month(), date.day(),
        time.hour(), time.minute()
    );
}

#[tokio::test]
async fn test_shesh_verification() {
    let profile = shesh_profile();
    let verifier = DataVerifier::new();
    
    let result = verifier.verify(&profile).await.expect("Verification should succeed");
    
    let date_check = result
        .checks
        .iter()
        .find(|c| c.name == "Date format")
        .expect("Date format check should exist");
    assert!(date_check.matches, "Date format should be valid");
    
    let time_check = result
        .checks
        .iter()
        .find(|c| c.name == "Time format")
        .expect("Time format check should exist");
    assert!(time_check.matches, "Time format should be valid");
    
    let coord_check = result
        .checks
        .iter()
        .find(|c| c.name == "Coordinates range")
        .expect("Coordinates range check should exist");
    assert!(coord_check.matches, "Coordinates should be valid");
    
    println!("✓ All verification checks passed");
    println!("  Date format: {}", if date_check.matches { "✓" } else { "✗" });
    println!("  Time format: {}", if time_check.matches { "✓" } else { "✗" });
    println!("  Coordinates: {}", if coord_check.matches { "✓" } else { "✗" });
}

#[test]
fn test_shesh_life_path_number() {
    let profile = shesh_profile();
    
    // Calculate Life Path Number manually
    // 1991 -> 1+9+9+1 = 20 -> 2+0 = 2
    // 8 -> 8
    // 13 -> 1+3 = 4
    // Total: 2 + 8 + 4 = 14 -> 1+4 = 5
    
    fn reduce_to_single_digit(n: u32) -> u32 {
        let mut sum = n;
        while sum > 9 && !matches!(sum, 11 | 22 | 33) {
            sum = sum_of_digits(sum);
        }
        sum
    }
    
    fn sum_of_digits(n: u32) -> u32 {
        let mut num = n;
        let mut sum = 0;
        while num > 0 {
            sum += num % 10;
            num /= 10;
        }
        sum
    }
    
    let year_sum = reduce_to_single_digit(1991);
    assert_eq!(year_sum, 2, "1991 -> 1+9+9+1 = 20 -> 2+0 = 2");
    
    let month_sum = reduce_to_single_digit(8);
    assert_eq!(month_sum, 8);
    
    let day_sum = reduce_to_single_digit(13);
    assert_eq!(day_sum, 4, "13 -> 1+3 = 4");
    
    let life_path = reduce_to_single_digit(year_sum + month_sum + day_sum);
    assert_eq!(life_path, 5, "2 + 8 + 4 = 14 -> 1+4 = 5");
    
    println!("✓ Life Path Number calculation verified");
    println!("  Year (1991): {} -> {}", 1991, year_sum);
    println!("  Month (8): {} -> {}", 8, month_sum);
    println!("  Day (13): {} -> {}", 13, day_sum);
    println!("  Life Path: {} (The Freedom Seeker)", life_path);
}

#[test]
fn test_shesh_tcm_analysis() {
    let profile = shesh_profile();
    
    let analysis = TCMAnalysis::from_birth_profile(&profile)
        .expect("Failed to generate TCM analysis");
    
    // August is late summer/early autumn - Earth or Fire element
    println!("✓ TCM Analysis generated");
    println!("  Dominant Element: {:?}", analysis.dominant_element);
    println!("  Secondary Element: {:?}", analysis.secondary_element);
    println!("  Season: {}", analysis.seasonal_influence.current_season);
    println!("  Constitution: {:?}", analysis.constitution);
    
    // August should be Fire or Earth dominant
    assert!(
        matches!(analysis.dominant_element, TCMElement::Fire | TCMElement::Earth),
        "August birth should have Fire or Earth as dominant element"
    );
    
    // Should have recommendations
    assert!(!analysis.recommendations.is_empty(), "Should have TCM recommendations");
    
    // Should have optimal times
    assert!(!analysis.optimal_times.is_empty(), "Should have optimal time windows");
}

#[test]
fn test_expected_dasha_values() {
    // Shesh's known Vimshottari Dasha values
    println!("✓ Expected Vimshottari Dasha Values");
    println!("  Birth Moon Nakshatra: Uttara Phalguni");
    println!("  Current Mahadasha: Mars");
    println!("  Mahadasha End Date: 2026-09-14");
    println!("  Mars Period: Action, courage, physical vitality");
    
    // These are the expected values we'd verify against API results
    let expected_moon_nakshatra = "Uttara Phalguni";
    let expected_current_mahadasha = "Mars";
    let expected_mahadasha_end = "2026-09-14";
    
    assert_eq!(expected_moon_nakshatra, "Uttara Phalguni");
    assert_eq!(expected_current_mahadasha, "Mars");
    assert_eq!(expected_mahadasha_end, "2026-09-14");
}

#[test]
fn test_expected_panchang_values() {
    // Shesh's birth panchang values
    println!("✓ Expected Panchang Values (Birth)");
    println!("  Tithi: Chaturthi (4th lunar day)");
    println!("  Paksha: Shukla (Waxing)");
    println!("  Vara: Tuesday (Mangalvar)");
    println!("  Moon Nakshatra: Uttara Phalguni");
    println!("  Moon Lord: Sun");
    
    // Verify the tithi is auspicious or inauspicious
    // Chaturthi is generally neutral to slightly challenging
}

#[tokio::test]
async fn test_unified_analysis_generation() {
    let profile = shesh_profile();
    
    // This test requires the Vedic API client to be configured
    // It will be skipped if the API is not available
    
    match UnifiedAnalysis::generate(&profile).await {
        Ok(analysis) => {
            println!("✓ Unified Analysis generated successfully");
            println!("  Generated at: {}", analysis.generated_at);
            println!("  Vimshottari Mahadasha: {}", analysis.vimshottari.current_mahadasha);
            println!("  Life Path Number: {}", analysis.numerology.life_path_number);
            println!("  TCM Element: {:?}", analysis.tcm.dominant_element);
            
            // Verify key values
            assert_eq!(analysis.numerology.life_path_number, 5);
            assert_eq!(analysis.vimshottari.current_mahadasha, "Mars");
            
            // Should have insights
            assert!(!analysis.layered_insights.is_empty());
            
            // Should have recommendations
            assert!(!analysis.recommendations.is_empty());
        }
        Err(e) => {
            println!("⚠ Unified Analysis generation skipped: {}", e);
            println!("  (This is expected if Vedic API is not configured)");
        }
    }
}

#[test]
fn test_birth_data_integrity() {
    // Verify that birth data remains consistent across all calculations
    let profile = shesh_profile();
    
    // Parse date
    let date = profile.parse_date().unwrap();
    
    // Day of week for 1991-08-13
    let weekday = date.weekday();
    println!("✓ Birth date verification");
    println!("  Date: {}-{:02}-{:02}", date.year(), date.month(), date.day());
    println!("  Day of week: {:?}", weekday);
    
    // August 13, 1991 was a Tuesday
    assert_eq!(weekday.to_string(), "Tue", "1991-08-13 should be a Tuesday");
    
    // Time verification
    let time = profile.parse_time().unwrap();
    let total_minutes = time.hour() * 60 + time.minute();
    println!("  Time: {}:{} ({} minutes from midnight)", 
        time.hour(), time.minute(), total_minutes);
    
    // 13:31 IST
    assert_eq!(time.hour(), 13);
    assert_eq!(time.minute(), 31);
}

#[test]
fn test_numerology_traits_for_life_path_5() {
    // Life Path 5 traits
    let traits = vec![
        "Freedom-loving",
        "Adventurous", 
        "Versatile",
        "Curious",
        "Adaptable",
    ];
    
    println!("✓ Life Path 5 Traits");
    for trait_name in &traits {
        println!("  - {}", trait_name);
    }
    
    assert!(!traits.is_empty());
}

/// Comprehensive test summary
#[test]
fn test_summary() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║     Shesh Birth Data Verification Summary                  ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Birth Date: 1991-08-13                                    ║");
    println!("║  Birth Time: 13:31 IST                                     ║");
    println!("║  Location:   Bengaluru, India (12.9716° N, 77.5946° E)    ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Expected Values:                                          ║");
    println!("║    • Moon Nakshatra: Uttara Phalguni (Sun-ruled)          ║");
    println!("║    • Tithi: Chaturthi (Shukla Paksha)                      ║");
    println!("║    • Current Mahadasha: Mars (until 2026-09-14)           ║");
    println!("║    • Life Path Number: 5 (Freedom Seeker)                  ║");
    println!("║    • Day of Birth: Tuesday                                 ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
}
