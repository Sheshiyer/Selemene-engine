use engine_human_design::{
    init_wisdom, GATES, CENTERS, CHANNELS, TYPES, AUTHORITIES, PROFILES, 
    LINES, DEFINITIONS, CIRCUITRY, INCARNATION_CROSSES, VARIABLES, PLANETARY_ACTIVATIONS
};

#[test]
fn test_wisdom_data_loads() {
    // Force initialization
    init_wisdom();
    
    // Verify gates loaded (should have 64 gates)
    assert!(!GATES.is_empty(), "Gates should not be empty");
    println!("✓ Loaded {} gates", GATES.len());
    
    // Verify centers loaded (should have 9 centers)
    assert!(!CENTERS.is_empty(), "Centers should not be empty");
    println!("✓ Loaded {} centers", CENTERS.len());
    
    // Verify channels loaded (should have 36 channels)
    assert!(!CHANNELS.is_empty(), "Channels should not be empty");
    println!("✓ Loaded {} channels", CHANNELS.len());
    
    // Verify types loaded (should have 5 types)
    assert!(!TYPES.is_empty(), "Types should not be empty");
    println!("✓ Loaded {} types", TYPES.len());
    
    // Verify authorities loaded (should have 7 authorities)
    assert!(!AUTHORITIES.is_empty(), "Authorities should not be empty");
    println!("✓ Loaded {} authorities", AUTHORITIES.len());
    
    // Verify profiles loaded (should have 12 profiles)
    assert!(!PROFILES.is_empty(), "Profiles should not be empty");
    println!("✓ Loaded {} profiles", PROFILES.len());
    
    // Verify lines loaded (should have 6 lines)
    assert!(!LINES.is_empty(), "Lines should not be empty");
    println!("✓ Loaded {} lines", LINES.len());
    
    // Verify definitions loaded (should have 5 definition types)
    assert!(!DEFINITIONS.is_empty(), "Definitions should not be empty");
    println!("✓ Loaded {} definitions", DEFINITIONS.len());
    
    // Verify circuitry loaded (should have 3 main circuits)
    assert!(!CIRCUITRY.is_empty(), "Circuitry should not be empty");
    println!("✓ Loaded {} circuitry types", CIRCUITRY.len());
    
    // Verify incarnation crosses loaded
    assert!(!INCARNATION_CROSSES.is_empty(), "Incarnation crosses should not be empty");
    println!("✓ Loaded {} incarnation crosses", INCARNATION_CROSSES.len());
    
    // Verify variables loaded
    assert!(!VARIABLES.is_empty(), "Variables should not be empty");
    println!("✓ Loaded {} variables", VARIABLES.len());
    
    // Verify planetary activations loaded
    assert!(!PLANETARY_ACTIVATIONS.is_empty(), "Planetary activations should not be empty");
    println!("✓ Loaded {} planetary activations", PLANETARY_ACTIVATIONS.len());
}

#[test]
fn test_gate_wisdom_sample() {
    init_wisdom();
    
    // Test that gate 1 exists and has expected data
    let gate1 = GATES.get("1").expect("Gate 1 should exist");
    assert_eq!(gate1.number, 1);
    assert_eq!(gate1.name, "The Creative");
    assert_eq!(gate1.keynote, "Self-Expression");
    assert_eq!(gate1.center, "G");
    
    println!("✓ Gate 1: {} - {}", gate1.name, gate1.keynote);
    println!("  Description: {}", gate1.description);
}

#[test]
fn test_center_wisdom_sample() {
    init_wisdom();
    
    // Test that Sacral center exists
    let sacral = CENTERS.get("Sacral").expect("Sacral center should exist");
    assert_eq!(sacral.name, "Sacral Center");
    assert!(!sacral.gates.is_empty(), "Sacral should have gates");
    
    println!("✓ {} - {}", sacral.name, sacral.function);
    println!("  Gates: {:?}", sacral.gates);
}

#[test]
fn test_type_wisdom_sample() {
    init_wisdom();
    
    // Test Generator type
    let generator = TYPES.get("Generator").expect("Generator type should exist");
    assert_eq!(generator.name, "Generator");
    assert_eq!(generator.strategy, "To Respond");
    assert_eq!(generator.signature, "Satisfaction");
    
    println!("✓ Type: {}", generator.name);
    println!("  Strategy: {}", generator.strategy);
    println!("  Signature: {}", generator.signature);
}

#[test]
fn test_authority_wisdom_sample() {
    init_wisdom();
    
    // Test Sacral Authority
    let sacral_auth = AUTHORITIES.get("Sacral_Authority")
        .expect("Sacral Authority should exist");
    assert_eq!(sacral_auth.name, "Sacral Authority");
    assert_eq!(sacral_auth.center, "Sacral");
    
    println!("✓ Authority: {}", sacral_auth.name);
    println!("  Center: {}", sacral_auth.center);
    println!("  Description: {}", sacral_auth.description);
}

#[test]
fn test_profile_wisdom_sample() {
    init_wisdom();
    
    // Test 1/3 profile
    let profile_1_3 = PROFILES.get("1_3").expect("Profile 1/3 should exist");
    assert_eq!(profile_1_3.name, "1/3 - Investigator/Martyr");
    
    println!("✓ Profile: {}", profile_1_3.name);
    println!("  Theme: {}", profile_1_3.theme);
    println!("  Life Purpose: {}", profile_1_3.life_purpose);
}

#[test]
fn test_channel_wisdom_sample() {
    init_wisdom();
    
    // Find a channel (they're keyed by gate combinations)
    if let Some((key, channel)) = CHANNELS.iter().next() {
        println!("✓ Channel ({}): {}", key, channel.name);
        println!("  Gates: {:?}", channel.gates);
        println!("  Centers: {:?}", channel.centers);
        println!("  Circuitry: {}", channel.circuitry);
    }
}
