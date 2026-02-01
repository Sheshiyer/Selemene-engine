//! Human Design Gate Sequence Mapping - Rave I-Ching Mandala
//!
//! CRITICAL: Uses the RAVE MANDALA sequence - the actual Human Design wheel.
//! This is NOT sequential (1, 2, 3...) around the zodiac!
//!
//! The Rave Mandala starts with Gate 17 at 0° Aries and follows a specific
//! non-sequential order derived from the I-Ching mapped to the zodiac.
//!
//! Gate calculation:
//! - 360° / 64 gates = 5.625° per gate
//! - Gate 17 starts at 0° Aries (spring equinox)
//! - Gates follow Rave Mandala sequence around the wheel
//!
//! Line calculation:
//! - 5.625° / 6 lines = 0.9375° per line
//! - Lines numbered 1-6 within each gate

const DEGREES_PER_GATE: f64 = 360.0 / 64.0; // 5.625°
const DEGREES_PER_LINE: f64 = DEGREES_PER_GATE / 6.0; // 0.9375°

/// The Rave Mandala gate sequence starting at 0° Aries.
/// This is the actual Human Design gate order around the zodiac wheel.
/// Each entry represents the gate number for that position (0-63 → gates at that position).
///
/// Position 0 = 0°-5.625° Aries = Gate 17
/// Position 1 = 5.625°-11.25° Aries = Gate 21
/// etc.
static RAVE_MANDALA_SEQUENCE: [u8; 64] = [
    17, 21, 51, 42, 3, 27, 24, 2,     // 0°-45° (Aries, Taurus partial)
    23, 8, 20, 16, 35, 45, 12, 15,    // 45°-90° (Taurus, Gemini partial)
    52, 39, 53, 62, 56, 31, 33, 7,    // 90°-135° (Gemini, Cancer partial)
    4, 29, 59, 40, 64, 47, 6, 46,     // 135°-180° (Cancer, Leo partial)
    18, 48, 57, 32, 50, 28, 44, 1,    // 180°-225° (Leo, Virgo partial)
    43, 14, 34, 9, 5, 26, 11, 10,     // 225°-270° (Virgo, Libra partial)
    58, 38, 54, 61, 60, 41, 19, 13,   // 270°-315° (Libra, Scorpio partial)
    49, 30, 55, 37, 63, 22, 36, 25,   // 315°-360° (Scorpio, Sagittarius, Capricorn, Aquarius, Pisces)
];

/// Convert zodiac longitude to Human Design gate number (1-64).
///
/// Uses the Rave Mandala sequence - NOT sequential numbering!
/// Gate 17 is at 0° Aries, Gate 21 at 5.625° Aries, etc.
///
/// # Arguments
/// * `longitude` - Longitude in degrees (0-360)
///
/// # Returns
/// Gate number (1-64)
pub fn longitude_to_gate(longitude: f64) -> u8 {
    // Normalize longitude to 0-360
    let normalized = longitude.rem_euclid(360.0);
    
    // Calculate position index: divide by 5.625° to get 0-63
    let position = (normalized / DEGREES_PER_GATE).floor() as usize;
    
    // Look up the gate number in the Rave Mandala sequence
    // Clamp position to valid range (should not be necessary, but safety)
    let safe_position = position.min(63);
    
    RAVE_MANDALA_SEQUENCE[safe_position]
}

/// Convert zodiac longitude to Human Design line number (1-6) within a gate.
///
/// Each gate is divided into 6 lines of 0.9375° each.
///
/// # Arguments
/// * `longitude` - Longitude in degrees (0-360)
/// * `gate` - Gate number (1-64) for validation
///
/// # Returns
/// Line number (1-6)
pub fn longitude_to_line(longitude: f64, _gate: u8) -> u8 {
    // Normalize longitude to 0-360
    let normalized = longitude.rem_euclid(360.0);
    
    // Find position within the current gate
    let position_in_gate = normalized % DEGREES_PER_GATE;
    
    // Calculate line number: divide by 0.9375° and add 1
    let line = (position_in_gate / DEGREES_PER_LINE).floor() as u8 + 1;
    
    // Clamp to 1-6
    line.clamp(1, 6)
}

/// Convert zodiac longitude to Human Design gate and line.
///
/// # Arguments
/// * `longitude` - Longitude in degrees (0-360)
///
/// # Returns
/// A tuple of (gate_number, line_number) where gate is 1-64 and line is 1-6
pub fn longitude_to_gate_and_line(longitude: f64) -> (u8, u8) {
    let gate = longitude_to_gate(longitude);
    let line = longitude_to_line(longitude, gate);
    (gate, line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rave_mandala_gate_mapping() {
        // Test that gates follow Rave Mandala sequence starting at 0° Aries
        assert_eq!(longitude_to_gate(0.0), 17, "0° Aries should be Gate 17");
        assert_eq!(longitude_to_gate(5.625), 21, "5.625° should be Gate 21");
        assert_eq!(longitude_to_gate(11.25), 51, "11.25° should be Gate 51");
        assert_eq!(longitude_to_gate(16.875), 42, "16.875° should be Gate 42");
        
        // Test middle of zodiac (180° = position 32)
        assert_eq!(longitude_to_gate(180.0), 18, "180° should be Gate 18");
        
        // Test near end of zodiac (354.375° = position 63)
        assert_eq!(longitude_to_gate(354.375), 25, "354.375° should be Gate 25");
    }

    #[test]
    fn test_line_calculation() {
        // Test line boundaries within Gate 17 (0° - 5.625°)
        assert_eq!(longitude_to_line(0.0, 17), 1, "Start of gate should be line 1");
        assert_eq!(longitude_to_line(0.9375, 17), 2, "0.9375° should be line 2");
        assert_eq!(longitude_to_line(1.875, 17), 3, "1.875° should be line 3");
        assert_eq!(longitude_to_line(2.8125, 17), 4, "2.8125° should be line 4");
        assert_eq!(longitude_to_line(3.75, 17), 5, "3.75° should be line 5");
        assert_eq!(longitude_to_line(4.6875, 17), 6, "4.6875° should be line 6");
    }

    #[test]
    fn test_gate_and_line_combined() {
        // Test 0° Aries
        let (gate, line) = longitude_to_gate_and_line(0.0);
        assert_eq!(gate, 17, "0° Aries should be Gate 17");
        assert_eq!(line, 1, "0° Aries should be Line 1");
        
        // Test 180° (Leo/Virgo cusp area)
        let (gate, line) = longitude_to_gate_and_line(180.0);
        assert_eq!(gate, 18, "180° should be Gate 18");
        assert_eq!(line, 1, "180° exactly should be Line 1");
    }

    #[test]
    fn test_normalization() {
        // Test that values beyond 360 normalize correctly
        let (gate_1, line_1) = longitude_to_gate_and_line(10.0);
        let (gate_2, line_2) = longitude_to_gate_and_line(370.0); // 10 + 360
        let (gate_3, line_3) = longitude_to_gate_and_line(-350.0); // 10 - 360

        assert_eq!(gate_1, gate_2, "360° rotation should give same gate");
        assert_eq!(line_1, line_2, "360° rotation should give same line");
        assert_eq!(gate_1, gate_3, "Negative angles should normalize correctly");
        assert_eq!(line_1, line_3, "Negative angles should normalize correctly");
    }

    #[test]
    fn test_all_64_gates_present() {
        // Verify that the Rave Mandala contains all 64 gates exactly once
        let mut gate_counts = [0u8; 65]; // index 0 unused, 1-64 for gates
        for &gate in &RAVE_MANDALA_SEQUENCE {
            assert!(gate >= 1 && gate <= 64, "Gate {} out of range", gate);
            gate_counts[gate as usize] += 1;
        }
        for gate in 1..=64 {
            assert_eq!(gate_counts[gate], 1, "Gate {} should appear exactly once, found {}", gate, gate_counts[gate]);
        }
    }

    #[test]
    fn test_line_range() {
        // Test various positions to ensure lines stay in 1-6 range
        for i in 0..360 {
            let (gate, line) = longitude_to_gate_and_line(i as f64);
            assert!(gate >= 1 && gate <= 64, "Gate {} out of range at {}°", gate, i);
            assert!(line >= 1 && line <= 6, "Line {} out of range at {}°", line, i);
        }
    }

    #[test]
    fn test_known_positions() {
        // Test some known gate positions from reference data
        // Gate 4 should be around 135°-140° (Cancer/Leo area)
        // Position 24 = Gate 4 at 135°-140.625°
        assert_eq!(longitude_to_gate(136.0), 4, "136° should be Gate 4");
        
        // Gate 23 should be around 45° (Taurus area)  
        // Position 8 = Gate 23 at 45°-50.625°
        assert_eq!(longitude_to_gate(46.0), 23, "46° should be Gate 23");
        
        // Gate 43 should be around 225° (Virgo/Libra area)
        // Position 40 = Gate 43 at 225°-230.625°
        assert_eq!(longitude_to_gate(226.0), 43, "226° should be Gate 43");
        
        // Gate 49 should be around 315° (Scorpio/Sagittarius area)
        // Position 56 = Gate 49 at 315°-320.625°
        assert_eq!(longitude_to_gate(316.0), 49, "316° should be Gate 49");
    }
}
