//! Human Design Gate Sequence Mapping
//!
//! CRITICAL: Uses SEQUENTIAL gate numbering (1→64) around the zodiac wheel.
//! This is NOT the I-Ching King Wen sequence — that's a common mistake.
//!
//! Gate calculation:
//! - 360° / 64 gates = 5.625° per gate
//! - Gate 1 starts at 0° Aries (spring equinox)
//! - Gates increment sequentially around the zodiac (1, 2, 3, ..., 64)
//!
//! Line calculation:
//! - 5.625° / 6 lines = 0.9375° per line
//! - Lines numbered 1-6 within each gate

const DEGREES_PER_GATE: f64 = 360.0 / 64.0; // 5.625°
const DEGREES_PER_LINE: f64 = DEGREES_PER_GATE / 6.0; // 0.9375°

/// Convert zodiac longitude to Human Design gate number (1-64).
///
/// Gates are numbered SEQUENTIALLY around the zodiac wheel starting at 0° Aries.
/// This is NOT the King Wen I-Ching sequence.
///
/// # Arguments
/// * `longitude` - Longitude in degrees (0-360)
///
/// # Returns
/// Gate number (1-64)
pub fn longitude_to_gate(longitude: f64) -> u8 {
    // Normalize longitude to 0-360
    let normalized = longitude.rem_euclid(360.0);
    
    // Calculate gate number: divide by 5.625° and add 1
    // Gates 1-64 correspond to 0°-5.625°, 5.625°-11.25°, etc.
    let gate = (normalized / DEGREES_PER_GATE).floor() as u8 + 1;
    
    // Clamp to 1-64 (should not be necessary, but safety check)
    gate.clamp(1, 64)
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
    fn test_sequential_gate_mapping() {
        // Test that gates increment sequentially from 0° Aries
        assert_eq!(longitude_to_gate(0.0), 1, "0° Aries should be Gate 1");
        assert_eq!(longitude_to_gate(5.625), 2, "5.625° should be Gate 2");
        assert_eq!(longitude_to_gate(11.25), 3, "11.25° should be Gate 3");
        assert_eq!(longitude_to_gate(16.875), 4, "16.875° should be Gate 4");
        
        // Test middle of zodiac
        assert_eq!(longitude_to_gate(180.0), 33, "180° should be Gate 33");
        
        // Test near end of zodiac
        assert_eq!(longitude_to_gate(354.375), 64, "354.375° should be Gate 64");
        assert_eq!(longitude_to_gate(359.9), 64, "359.9° should be Gate 64");
    }

    #[test]
    fn test_line_calculation() {
        // Test line boundaries within Gate 1 (0° - 5.625°)
        assert_eq!(longitude_to_line(0.0, 1), 1, "Start of gate should be line 1");
        assert_eq!(longitude_to_line(0.9375, 1), 2, "0.9375° should be line 2");
        assert_eq!(longitude_to_line(1.875, 1), 3, "1.875° should be line 3");
        assert_eq!(longitude_to_line(2.8125, 1), 4, "2.8125° should be line 4");
        assert_eq!(longitude_to_line(3.75, 1), 5, "3.75° should be line 5");
        assert_eq!(longitude_to_line(4.6875, 1), 6, "4.6875° should be line 6");
        
        // Test line in middle gate
        assert_eq!(longitude_to_line(180.0, 33), 1, "Start of Gate 33 should be line 1");
        assert_eq!(longitude_to_line(184.6875, 33), 6, "End of Gate 33 should be line 6");
    }

    #[test]
    fn test_gate_and_line_combined() {
        // Test 10° Aries (should be Gate 2 or 3)
        let (gate, line) = longitude_to_gate_and_line(10.0);
        assert!(gate >= 1 && gate <= 64, "Gate must be 1-64");
        assert!(line >= 1 && line <= 6, "Line must be 1-6");
        assert_eq!(gate, 2, "10° should be Gate 2");
        
        // Test 0° Aries
        let (gate, line) = longitude_to_gate_and_line(0.0);
        assert_eq!(gate, 1, "0° Aries should be Gate 1");
        assert_eq!(line, 1, "0° Aries should be Line 1");
        
        // Test 180° (middle of zodiac)
        let (gate, line) = longitude_to_gate_and_line(180.0);
        assert_eq!(gate, 33, "180° should be Gate 33");
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
    fn test_all_gates_sequential() {
        // Verify that as we move through 360°, gates increment from 1 to 64
        let mut prev_gate = 0u8;
        for i in 0..64 {
            let longitude = i as f64 * DEGREES_PER_GATE + 0.1; // Slightly into each gate
            let gate = longitude_to_gate(longitude);
            
            if i == 0 {
                assert_eq!(gate, 1, "First gate should be 1");
            } else {
                assert_eq!(gate, prev_gate + 1, "Gates should increment sequentially");
            }
            prev_gate = gate;
        }
        assert_eq!(prev_gate, 64, "Last gate should be 64");
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
    fn test_gate_boundaries() {
        // Test exact gate boundaries
        for gate_num in 1..=64 {
            let gate_start = (gate_num - 1) as f64 * DEGREES_PER_GATE;
            let gate = longitude_to_gate(gate_start);
            assert_eq!(gate, gate_num, "Gate boundary at {:.4}° should be Gate {}", gate_start, gate_num);
        }
    }
}
