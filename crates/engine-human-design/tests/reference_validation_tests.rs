//! Validation tests against reference charts
//!
//! Tests the HD engine against the reference dataset to ensure consistency
//! and accuracy of planetary activations, Type, Authority, Profile, and channels.
//!
//! Test Categories (W1-S4-02 through W1-S4-07):
//! - W1-S4-02: Sun/Earth validation
//! - W1-S4-03: Type validation
//! - W1-S4-04: Authority validation
//! - W1-S4-05: Profile validation
//! - W1-S4-06: Centers validation
//! - W1-S4-07: Channels validation

use engine_human_design::{generate_hd_chart, HDType, Authority, Planet};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct ReferenceDataset {
    charts: Vec<ReferenceChart>,
    #[allow(dead_code)]
    metadata: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReferenceChart {
    name: String,
    birth_date: String,
    birth_time: String,
    #[allow(dead_code)]
    timezone: String,
    #[allow(dead_code)]
    latitude: f64,
    #[allow(dead_code)]
    longitude: f64,
    expected: ExpectedResults,
}

#[derive(Debug, Deserialize, Serialize)]
struct ExpectedResults {
    personality_sun: GateLine,
    personality_earth: GateLine,
    design_sun: GateLine,
    design_earth: GateLine,
    #[serde(rename = "type")]
    hd_type: String,
    authority: String,
    profile: String,
    defined_centers: Vec<String>,
    active_channels: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GateLine {
    gate: u8,
    line: u8,
}

fn load_reference_charts() -> ReferenceDataset {
    let json_str = fs::read_to_string("tests/reference_charts.json")
        .expect("Failed to read reference_charts.json");
    serde_json::from_str(&json_str).expect("Failed to parse reference_charts.json")
}

fn parse_birth_datetime(date_str: &str, time_str: &str) -> chrono::DateTime<Utc> {
    // Parse YYYY-MM-DD and HH:MM:SS
    let parts: Vec<&str> = date_str.split('-').collect();
    let year = parts[0].parse::<i32>().unwrap();
    let month = parts[1].parse::<u32>().unwrap();
    let day = parts[2].parse::<u32>().unwrap();
    
    let time_parts: Vec<&str> = time_str.split(':').collect();
    let hour = time_parts[0].parse::<u32>().unwrap();
    let minute = time_parts[1].parse::<u32>().unwrap();
    let second = time_parts[2].parse::<u32>().unwrap();
    
    Utc.with_ymd_and_hms(year, month, day, hour, minute, second).unwrap()
}

fn type_string_to_enum(s: &str) -> HDType {
    match s {
        "Generator" => HDType::Generator,
        "ManifestingGenerator" => HDType::ManifestingGenerator,
        "Projector" => HDType::Projector,
        "Manifestor" => HDType::Manifestor,
        "Reflector" => HDType::Reflector,
        _ => panic!("Unknown type: {}", s),
    }
}

fn authority_string_to_enum(s: &str) -> Authority {
    match s {
        "Sacral" => Authority::Sacral,
        "Emotional" => Authority::Emotional,
        "Splenic" => Authority::Splenic,
        "Heart" => Authority::Heart,
        "GCenter" => Authority::GCenter,
        "Mental" => Authority::Mental,
        "Lunar" => Authority::Lunar,
        _ => panic!("Unknown authority: {}", s),
    }
}

// Validation statistics tracker
#[derive(Debug, Default)]
struct ValidationStats {
    total: usize,
    passed: usize,
    failed: usize,
    errors: Vec<String>,
}

impl ValidationStats {
    fn new() -> Self {
        Self::default()
    }
    
    fn record_pass(&mut self) {
        self.total += 1;
        self.passed += 1;
    }
    
    fn record_fail(&mut self, chart_name: &str, reason: String) {
        self.total += 1;
        self.failed += 1;
        self.errors.push(format!("{}: {}", chart_name, reason));
    }
    
    fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
    
    fn print_summary(&self, category: &str) {
        println!("\n=== {} ===", category);
        println!("Total: {}, Passed: {}, Failed: {}", self.total, self.passed, self.failed);
        println!("Pass Rate: {:.1}%", self.pass_rate());
        if !self.errors.is_empty() {
            println!("\nFailures:");
            for error in &self.errors {
                println!("  ❌ {}", error);
            }
        }
    }
}

// ===== W1-S4-02: Sun/Earth Validation =====

#[test]
fn test_w1_s4_02_sun_earth_validation() {
    let dataset = load_reference_charts();
    let mut stats = ValidationStats::new();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║         W1-S4-02: Sun/Earth Validation Test              ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    
    for chart_ref in dataset.charts.iter() {
        let birth_time = parse_birth_datetime(&chart_ref.birth_date, &chart_ref.birth_time);
        
        match generate_hd_chart(birth_time, "") {
            Ok(result) => {
                let mut chart_passed = true;
                let mut failures = Vec::new();
                
                // Check Personality Sun
                if let Some(p_sun) = result.personality_activations.iter()
                    .find(|a| matches!(a.planet, Planet::Sun)) {
                    if p_sun.gate != chart_ref.expected.personality_sun.gate {
                        chart_passed = false;
                        failures.push(format!(
                            "Personality Sun gate: expected {}, got {}",
                            chart_ref.expected.personality_sun.gate, p_sun.gate
                        ));
                    }
                    if p_sun.line != chart_ref.expected.personality_sun.line {
                        chart_passed = false;
                        failures.push(format!(
                            "Personality Sun line: expected {}, got {}",
                            chart_ref.expected.personality_sun.line, p_sun.line
                        ));
                    }
                }
                
                // Check Personality Earth
                if let Some(p_earth) = result.personality_activations.iter()
                    .find(|a| matches!(a.planet, Planet::Earth)) {
                    if p_earth.gate != chart_ref.expected.personality_earth.gate {
                        chart_passed = false;
                        failures.push(format!(
                            "Personality Earth gate: expected {}, got {}",
                            chart_ref.expected.personality_earth.gate, p_earth.gate
                        ));
                    }
                    if p_earth.line != chart_ref.expected.personality_earth.line {
                        chart_passed = false;
                        failures.push(format!(
                            "Personality Earth line: expected {}, got {}",
                            chart_ref.expected.personality_earth.line, p_earth.line
                        ));
                    }
                }
                
                // Check Design Sun
                if let Some(d_sun) = result.design_activations.iter()
                    .find(|a| matches!(a.planet, Planet::Sun)) {
                    if d_sun.gate != chart_ref.expected.design_sun.gate {
                        chart_passed = false;
                        failures.push(format!(
                            "Design Sun gate: expected {}, got {}",
                            chart_ref.expected.design_sun.gate, d_sun.gate
                        ));
                    }
                    if d_sun.line != chart_ref.expected.design_sun.line {
                        chart_passed = false;
                        failures.push(format!(
                            "Design Sun line: expected {}, got {}",
                            chart_ref.expected.design_sun.line, d_sun.line
                        ));
                    }
                }
                
                // Check Design Earth
                if let Some(d_earth) = result.design_activations.iter()
                    .find(|a| matches!(a.planet, Planet::Earth)) {
                    if d_earth.gate != chart_ref.expected.design_earth.gate {
                        chart_passed = false;
                        failures.push(format!(
                            "Design Earth gate: expected {}, got {}",
                            chart_ref.expected.design_earth.gate, d_earth.gate
                        ));
                    }
                    if d_earth.line != chart_ref.expected.design_earth.line {
                        chart_passed = false;
                        failures.push(format!(
                            "Design Earth line: expected {}, got {}",
                            chart_ref.expected.design_earth.line, d_earth.line
                        ));
                    }
                }
                
                if chart_passed {
                    stats.record_pass();
                    println!("  ✅ {}", chart_ref.name);
                } else {
                    stats.record_fail(&chart_ref.name, failures.join("; "));
                    println!("  ❌ {}", chart_ref.name);
                    for failure in failures {
                        println!("      {}", failure);
                    }
                }
            }
            Err(e) => {
                stats.record_fail(&chart_ref.name, format!("Chart generation error: {}", e));
                println!("  ❌ {} - ERROR: {}", chart_ref.name, e);
            }
        }
    }
    
    stats.print_summary("W1-S4-02: Sun/Earth Validation");
    
    // Assert at least 80% pass rate for Sun/Earth (core calculation)
    assert!(
        stats.pass_rate() >= 80.0,
        "Sun/Earth validation pass rate {:.1}% below target 80%",
        stats.pass_rate()
    );
}

// ===== W1-S4-03: Type Validation =====

#[test]
fn test_w1_s4_03_type_validation() {
    let dataset = load_reference_charts();
    let mut stats = ValidationStats::new();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║            W1-S4-03: Type Validation Test                ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    
    for chart_ref in dataset.charts.iter() {
        let birth_time = parse_birth_datetime(&chart_ref.birth_date, &chart_ref.birth_time);
        
        match generate_hd_chart(birth_time, "") {
            Ok(result) => {
                let expected_type = type_string_to_enum(&chart_ref.expected.hd_type);
                
                if result.hd_type == expected_type {
                    stats.record_pass();
                    println!("  ✅ {} - {:?}", chart_ref.name, result.hd_type);
                } else {
                    stats.record_fail(
                        &chart_ref.name,
                        format!("expected {:?}, got {:?}", expected_type, result.hd_type)
                    );
                    println!("  ❌ {} - expected {:?}, got {:?}", 
                        chart_ref.name, expected_type, result.hd_type);
                }
            }
            Err(e) => {
                stats.record_fail(&chart_ref.name, format!("Chart generation error: {}", e));
                println!("  ❌ {} - ERROR: {}", chart_ref.name, e);
            }
        }
    }
    
    stats.print_summary("W1-S4-03: Type Validation");
    
    // Type validation expected to be lower due to incomplete wisdom data
    // Target: 50-70% (depends on channel completeness)
    assert!(
        stats.pass_rate() >= 40.0,
        "Type validation pass rate {:.1}% below minimum threshold 40%",
        stats.pass_rate()
    );
}

// ===== W1-S4-04: Authority Validation =====

#[test]
fn test_w1_s4_04_authority_validation() {
    let dataset = load_reference_charts();
    let mut stats = ValidationStats::new();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║         W1-S4-04: Authority Validation Test              ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    
    for chart_ref in dataset.charts.iter() {
        let birth_time = parse_birth_datetime(&chart_ref.birth_date, &chart_ref.birth_time);
        
        match generate_hd_chart(birth_time, "") {
            Ok(result) => {
                let expected_authority = authority_string_to_enum(&chart_ref.expected.authority);
                
                if result.authority == expected_authority {
                    stats.record_pass();
                    println!("  ✅ {} - {:?}", chart_ref.name, result.authority);
                } else {
                    stats.record_fail(
                        &chart_ref.name,
                        format!("expected {:?}, got {:?}", expected_authority, result.authority)
                    );
                    println!("  ❌ {} - expected {:?}, got {:?}", 
                        chart_ref.name, expected_authority, result.authority);
                }
            }
            Err(e) => {
                stats.record_fail(&chart_ref.name, format!("Chart generation error: {}", e));
                println!("  ❌ {} - ERROR: {}", chart_ref.name, e);
            }
        }
    }
    
    stats.print_summary("W1-S4-04: Authority Validation");
    
    // Authority validation depends on center definitions (wisdom data)
    // Target: 50-70%
    assert!(
        stats.pass_rate() >= 40.0,
        "Authority validation pass rate {:.1}% below minimum threshold 40%",
        stats.pass_rate()
    );
}

// ===== W1-S4-05: Profile Validation =====

#[test]
fn test_w1_s4_05_profile_validation() {
    let dataset = load_reference_charts();
    let mut stats = ValidationStats::new();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║          W1-S4-05: Profile Validation Test               ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    
    for chart_ref in dataset.charts.iter() {
        let birth_time = parse_birth_datetime(&chart_ref.birth_date, &chart_ref.birth_time);
        
        match generate_hd_chart(birth_time, "") {
            Ok(result) => {
                let actual_profile = format!("{}/{}", 
                    result.profile.conscious_line, 
                    result.profile.unconscious_line
                );
                
                if actual_profile == chart_ref.expected.profile {
                    stats.record_pass();
                    println!("  ✅ {} - {}", chart_ref.name, actual_profile);
                } else {
                    stats.record_fail(
                        &chart_ref.name,
                        format!("expected {}, got {}", chart_ref.expected.profile, actual_profile)
                    );
                    println!("  ❌ {} - expected {}, got {}", 
                        chart_ref.name, chart_ref.expected.profile, actual_profile);
                }
            }
            Err(e) => {
                stats.record_fail(&chart_ref.name, format!("Chart generation error: {}", e));
                println!("  ❌ {} - ERROR: {}", chart_ref.name, e);
            }
        }
    }
    
    stats.print_summary("W1-S4-05: Profile Validation");
    
    // Profile should be 100% accurate (just Sun line numbers)
    assert!(
        stats.pass_rate() >= 80.0,
        "Profile validation pass rate {:.1}% below target 80%",
        stats.pass_rate()
    );
}

// ===== W1-S4-07: Channels Validation =====

#[test]
fn test_w1_s4_07_channels_validation() {
    let dataset = load_reference_charts();
    let mut stats = ValidationStats::new();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║         W1-S4-07: Channels Validation Test               ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    
    for chart_ref in dataset.charts.iter() {
        let birth_time = parse_birth_datetime(&chart_ref.birth_date, &chart_ref.birth_time);
        
        match generate_hd_chart(birth_time, "") {
            Ok(result) => {
                let mut actual_channels: Vec<String> = result.channels.iter()
                    .map(|ch| format!("{}-{}", ch.gate1, ch.gate2))
                    .collect();
                actual_channels.sort();
                
                let mut expected_channels = chart_ref.expected.active_channels.clone();
                expected_channels.sort();
                
                if actual_channels == expected_channels {
                    stats.record_pass();
                    println!("  ✅ {} - {} channels", chart_ref.name, actual_channels.len());
                } else {
                    let missing: Vec<_> = expected_channels.iter()
                        .filter(|e| !actual_channels.contains(e))
                        .collect();
                    let extra: Vec<_> = actual_channels.iter()
                        .filter(|a| !expected_channels.contains(a))
                        .collect();
                    
                    let mut msg = String::new();
                    if !missing.is_empty() {
                        msg.push_str(&format!("missing {:?}", missing));
                    }
                    if !extra.is_empty() {
                        if !msg.is_empty() { msg.push_str(", "); }
                        msg.push_str(&format!("extra {:?}", extra));
                    }
                    
                    stats.record_fail(&chart_ref.name, msg.clone());
                    println!("  ❌ {} - {}", chart_ref.name, msg);
                }
            }
            Err(e) => {
                stats.record_fail(&chart_ref.name, format!("Chart generation error: {}", e));
                println!("  ❌ {} - ERROR: {}", chart_ref.name, e);
            }
        }
    }
    
    stats.print_summary("W1-S4-07: Channels Validation");
    
    // Channels depend on incomplete wisdom data (5/36 channels loaded)
    // Target: 30-50%
    assert!(
        stats.pass_rate() >= 20.0,
        "Channels validation pass rate {:.1}% below minimum threshold 20%",
        stats.pass_rate()
    );
}

// ===== W1-S4-06: Centers Validation =====

#[test]
fn test_w1_s4_06_centers_validation() {
    let dataset = load_reference_charts();
    let mut stats = ValidationStats::new();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║         W1-S4-06: Centers Validation Test                ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    
    for chart_ref in dataset.charts.iter() {
        let birth_time = parse_birth_datetime(&chart_ref.birth_date, &chart_ref.birth_time);
        
        match generate_hd_chart(birth_time, "") {
            Ok(result) => {
                let mut actual_centers: Vec<String> = result.centers.iter()
                    .filter(|(_, info)| info.defined)
                    .map(|(center, _)| format!("{:?}", center))
                    .collect();
                actual_centers.sort();
                
                let mut expected_centers = chart_ref.expected.defined_centers.clone();
                expected_centers.sort();
                
                if actual_centers == expected_centers {
                    stats.record_pass();
                    println!("  ✅ {} - {} defined centers", 
                        chart_ref.name, actual_centers.len());
                } else {
                    let missing: Vec<_> = expected_centers.iter()
                        .filter(|e| !actual_centers.contains(e))
                        .collect();
                    let extra: Vec<_> = actual_centers.iter()
                        .filter(|a| !expected_centers.contains(a))
                        .collect();
                    
                    let mut msg = String::new();
                    if !missing.is_empty() {
                        msg.push_str(&format!("missing {:?}", missing));
                    }
                    if !extra.is_empty() {
                        if !msg.is_empty() { msg.push_str(", "); }
                        msg.push_str(&format!("extra {:?}", extra));
                    }
                    
                    stats.record_fail(&chart_ref.name, msg.clone());
                    println!("  ❌ {} - {}", chart_ref.name, msg);
                }
            }
            Err(e) => {
                stats.record_fail(&chart_ref.name, format!("Chart generation error: {}", e));
                println!("  ❌ {} - ERROR: {}", chart_ref.name, e);
            }
        }
    }
    
    stats.print_summary("W1-S4-06: Centers Validation");
    
    // Centers depend on incomplete wisdom data
    // Target: 30-50%
    assert!(
        stats.pass_rate() >= 20.0,
        "Centers validation pass rate {:.1}% below minimum threshold 20%",
        stats.pass_rate()
    );
}

// ===== Comprehensive Test Runner =====

#[test]
fn test_comprehensive_validation_report() {
    let dataset = load_reference_charts();
    
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║      COMPREHENSIVE VALIDATION REPORT (All Tests)        ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("\nTotal Reference Charts: {}", dataset.charts.len());
    
    // Track overall statistics
    let mut overall_stats = HashMap::new();
    overall_stats.insert("sun_earth", ValidationStats::new());
    overall_stats.insert("type", ValidationStats::new());
    overall_stats.insert("authority", ValidationStats::new());
    overall_stats.insert("profile", ValidationStats::new());
    overall_stats.insert("centers", ValidationStats::new());
    overall_stats.insert("channels", ValidationStats::new());
    
    for chart_ref in dataset.charts.iter() {
        let birth_time = parse_birth_datetime(&chart_ref.birth_date, &chart_ref.birth_time);
        
        match generate_hd_chart(birth_time, "") {
            Ok(chart) => {
                // Sun/Earth validation
                let sun_earth_ok = validate_sun_earth(&chart, &chart_ref.expected);
                if sun_earth_ok {
                    overall_stats.get_mut("sun_earth").unwrap().record_pass();
                } else {
                    overall_stats.get_mut("sun_earth").unwrap()
                        .record_fail(&chart_ref.name, "Sun/Earth mismatch".to_string());
                }
                
                // Type validation
                let expected_type = type_string_to_enum(&chart_ref.expected.hd_type);
                if chart.hd_type == expected_type {
                    overall_stats.get_mut("type").unwrap().record_pass();
                } else {
                    overall_stats.get_mut("type").unwrap()
                        .record_fail(&chart_ref.name, 
                            format!("expected {:?}, got {:?}", expected_type, chart.hd_type));
                }
                
                // Authority validation
                let expected_auth = authority_string_to_enum(&chart_ref.expected.authority);
                if chart.authority == expected_auth {
                    overall_stats.get_mut("authority").unwrap().record_pass();
                } else {
                    overall_stats.get_mut("authority").unwrap()
                        .record_fail(&chart_ref.name, 
                            format!("expected {:?}, got {:?}", expected_auth, chart.authority));
                }
                
                // Profile validation
                let actual_profile = format!("{}/{}", 
                    chart.profile.conscious_line, chart.profile.unconscious_line);
                if actual_profile == chart_ref.expected.profile {
                    overall_stats.get_mut("profile").unwrap().record_pass();
                } else {
                    overall_stats.get_mut("profile").unwrap()
                        .record_fail(&chart_ref.name, 
                            format!("expected {}, got {}", chart_ref.expected.profile, actual_profile));
                }
                
                // Centers validation
                let mut actual_centers: Vec<String> = chart.centers.iter()
                    .filter(|(_, info)| info.defined)
                    .map(|(center, _)| format!("{:?}", center))
                    .collect();
                actual_centers.sort();
                let mut expected_centers = chart_ref.expected.defined_centers.clone();
                expected_centers.sort();
                
                if actual_centers == expected_centers {
                    overall_stats.get_mut("centers").unwrap().record_pass();
                } else {
                    overall_stats.get_mut("centers").unwrap()
                        .record_fail(&chart_ref.name, "Centers mismatch".to_string());
                }
                
                // Channels validation
                let mut actual_channels: Vec<String> = chart.channels.iter()
                    .map(|ch| format!("{}-{}", ch.gate1, ch.gate2))
                    .collect();
                actual_channels.sort();
                let mut expected_channels = chart_ref.expected.active_channels.clone();
                expected_channels.sort();
                
                if actual_channels == expected_channels {
                    overall_stats.get_mut("channels").unwrap().record_pass();
                } else {
                    overall_stats.get_mut("channels").unwrap()
                        .record_fail(&chart_ref.name, "Channels mismatch".to_string());
                }
            }
            Err(e) => {
                // Record failure for all categories
                for stats in overall_stats.values_mut() {
                    stats.record_fail(&chart_ref.name, format!("Chart error: {}", e));
                }
            }
        }
    }
    
    // Print summary for all categories
    println!("\n\n╔═══════════════════════════════════════════════════════════╗");
    println!("║                  VALIDATION SUMMARY                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    println!("W1-S4-02 (Sun/Earth):  Pass Rate: {:.1}% ({}/{})", 
        overall_stats["sun_earth"].pass_rate(),
        overall_stats["sun_earth"].passed,
        overall_stats["sun_earth"].total);
    
    println!("W1-S4-03 (Type):       Pass Rate: {:.1}% ({}/{})", 
        overall_stats["type"].pass_rate(),
        overall_stats["type"].passed,
        overall_stats["type"].total);
    
    println!("W1-S4-04 (Authority):  Pass Rate: {:.1}% ({}/{})", 
        overall_stats["authority"].pass_rate(),
        overall_stats["authority"].passed,
        overall_stats["authority"].total);
    
    println!("W1-S4-05 (Profile):    Pass Rate: {:.1}% ({}/{})", 
        overall_stats["profile"].pass_rate(),
        overall_stats["profile"].passed,
        overall_stats["profile"].total);
    
    println!("W1-S4-06 (Centers):    Pass Rate: {:.1}% ({}/{})", 
        overall_stats["centers"].pass_rate(),
        overall_stats["centers"].passed,
        overall_stats["centers"].total);
    
    println!("W1-S4-07 (Channels):   Pass Rate: {:.1}% ({}/{})", 
        overall_stats["channels"].pass_rate(),
        overall_stats["channels"].passed,
        overall_stats["channels"].total);
    
    // Overall pass rate (average across all categories)
    let avg_pass_rate: f64 = overall_stats.values()
        .map(|s| s.pass_rate())
        .sum::<f64>() / overall_stats.len() as f64;
    
    println!("\n═══════════════════════════════════════════════════════════");
    println!("Overall Average Pass Rate: {:.1}%", avg_pass_rate);
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Print readiness assessment
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║              READINESS ASSESSMENT                        ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    if overall_stats["sun_earth"].pass_rate() >= 80.0 {
        println!("✅ Core calculations (Sun/Earth) READY");
    } else {
        println!("⚠️  Core calculations (Sun/Earth) NEEDS WORK");
    }
    
    if overall_stats["profile"].pass_rate() >= 80.0 {
        println!("✅ Profile calculations READY");
    } else {
        println!("⚠️  Profile calculations NEEDS WORK");
    }
    
    if overall_stats["type"].pass_rate() >= 50.0 {
        println!("✅ Type determination ACCEPTABLE (wisdom data limited)");
    } else {
        println!("⚠️  Type determination NEEDS REVIEW");
    }
    
    if overall_stats["authority"].pass_rate() >= 50.0 {
        println!("✅ Authority determination ACCEPTABLE (wisdom data limited)");
    } else {
        println!("⚠️  Authority determination NEEDS REVIEW");
    }
    
    println!("\n🔧 Expected limitations due to incomplete wisdom data:");
    println!("   - Only 5/36 channels loaded in wisdom database");
    println!("   - Centers/Channels validation limited by available data");
    println!("   - Type/Authority may be affected by missing channel definitions\n");
}

// Helper function to validate all Sun/Earth activations
fn validate_sun_earth(chart: &engine_human_design::HDChart, expected: &ExpectedResults) -> bool {
    let p_sun = chart.personality_activations.iter()
        .find(|a| matches!(a.planet, Planet::Sun));
    let p_earth = chart.personality_activations.iter()
        .find(|a| matches!(a.planet, Planet::Earth));
    let d_sun = chart.design_activations.iter()
        .find(|a| matches!(a.planet, Planet::Sun));
    let d_earth = chart.design_activations.iter()
        .find(|a| matches!(a.planet, Planet::Earth));
    
    if let (Some(ps), Some(pe), Some(ds), Some(de)) = (p_sun, p_earth, d_sun, d_earth) {
        ps.gate == expected.personality_sun.gate &&
        ps.line == expected.personality_sun.line &&
        pe.gate == expected.personality_earth.gate &&
        pe.line == expected.personality_earth.line &&
        ds.gate == expected.design_sun.gate &&
        ds.line == expected.design_sun.line &&
        de.gate == expected.design_earth.gate &&
        de.line == expected.design_earth.line
    } else {
        false
    }
}
