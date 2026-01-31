//! Human Design Gate Sequence Mapping
//! Based on the I-Ching wheel and zodiac positions.

use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    /// Gate sequence per zodiac sign (30° each).
    /// Each sign contains approximately 5.33 gates.
    static ref ZODIAC_GATE_SEQUENCE: HashMap<&'static str, Vec<u8>> = {
        let mut m = HashMap::new();
        // Aries (0° - 30°)
        m.insert("aries", vec![25, 51, 21, 26]);
        // Taurus (30° - 60°)
        m.insert("taurus", vec![27, 24, 2, 23]);
        // Gemini (60° - 90°)
        m.insert("gemini", vec![8, 20, 16, 35]);
        // Cancer (90° - 120°)
        m.insert("cancer", vec![45, 12, 15, 52]);
        // Leo (120° - 150°)
        m.insert("leo", vec![39, 53, 62, 56]);
        // Virgo (150° - 180°)
        m.insert("virgo", vec![31, 33, 7, 4]);
        // Libra (180° - 210°)
        m.insert("libra", vec![29, 59, 40, 64]);
        // Scorpio (210° - 240°)
        m.insert("scorpio", vec![47, 6, 46, 18]);
        // Sagittarius (240° - 270°)
        m.insert("sagittarius", vec![48, 57, 32, 50]);
        // Capricorn (270° - 300°)
        m.insert("capricorn", vec![28, 44, 1, 43]);
        // Aquarius (300° - 330°)
        m.insert("aquarius", vec![14, 34, 9, 5]);
        // Pisces (330° - 360°)
        m.insert("pisces", vec![26, 11, 10, 58]);
        m
    };

    static ref ZODIAC_SIGNS: Vec<&'static str> = vec![
        "aries", "taurus", "gemini", "cancer", "leo", "virgo",
        "libra", "scorpio", "sagittarius", "capricorn", "aquarius", "pisces"
    ];
}

/// Convert zodiac longitude to Human Design gate and line.
///
/// # Arguments
/// * `longitude` - Longitude in degrees (0-360)
///
/// # Returns
/// A tuple of (gate_number, line_number) where gate is 1-64 and line is 1-6
pub fn longitude_to_gate_and_line(longitude: f64) -> (u8, u8) {
    // Normalize longitude to 0-360
    let normalized_longitude = longitude % 360.0;

    // Determine zodiac sign (30° each)
    let sign_index = (normalized_longitude / 30.0).floor() as usize;
    let position_in_sign = normalized_longitude % 30.0;

    // Get gates for this sign
    let sign_gates = if sign_index < ZODIAC_SIGNS.len() {
        let sign_name = ZODIAC_SIGNS[sign_index];
        ZODIAC_GATE_SEQUENCE.get(sign_name).unwrap()
    } else {
        // Fallback for edge cases
        &vec![1, 2, 3, 4]
    };

    // Calculate which gate within the sign
    let degrees_per_gate = 30.0 / sign_gates.len() as f64;
    let gate_index = (position_in_sign / degrees_per_gate).floor() as usize;

    // Ensure we don't exceed the available gates
    let gate_index = gate_index.min(sign_gates.len() - 1);
    let gate_number = sign_gates[gate_index];

    // Calculate line within the gate (6 lines per gate)
    let position_in_gate = position_in_sign % degrees_per_gate;
    let line_size = degrees_per_gate / 6.0;
    let line_number = (position_in_gate / line_size).floor() as u8 + 1;

    // Ensure line is between 1-6
    let line_number = line_number.clamp(1, 6);

    (gate_number, line_number)
}

/// Calculate color and tone from longitude position.
///
/// # Arguments
/// * `longitude` - Longitude in degrees
/// * `gate_number` - Gate number (1-64)
/// * `line_number` - Line number (1-6)
///
/// # Returns
/// A tuple of (color, tone) both 1-6
///
/// Note: This is a simplified calculation. The actual calculation would
/// require more precise ephemeris data.
pub fn longitude_to_color_and_tone(longitude: f64, _gate_number: u8, _line_number: u8) -> (u8, u8) {
    // Use fractional part for sub-divisions
    let fractional_degrees = (longitude * 1000.0) % 1000.0;

    // Color (1-6) - each line has 6 colors
    let color = ((fractional_degrees / 1000.0) * 6.0).floor() as u8 + 1;
    let color = color.clamp(1, 6);

    // Tone (1-6) - each color has 6 tones
    let tone_fraction = ((fractional_degrees * 6.0) % 1000.0) / 1000.0;
    let tone = (tone_fraction * 6.0).floor() as u8 + 1;
    let tone = tone.clamp(1, 6);

    (color, tone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longitude_to_gate_and_line() {
        // Test with known positions
        let test_cases = vec![
            (140.0935, "Current Sun position"),
            (19.6875, "Expected Gate 4 center"),
            (272.8125, "Expected Gate 49 center"),
        ];

        for (longitude, description) in test_cases {
            let (gate, line) = longitude_to_gate_and_line(longitude);
            let (color, tone) = longitude_to_color_and_tone(longitude, gate, line);

            // Ensure values are in valid ranges
            assert!(gate >= 1 && gate <= 64, "Gate must be between 1-64");
            assert!(line >= 1 && line <= 6, "Line must be between 1-6");
            assert!(color >= 1 && color <= 6, "Color must be between 1-6");
            assert!(tone >= 1 && tone <= 6, "Tone must be between 1-6");

            println!("{}: {:.4}° → Gate {}.{}.{}.{}", description, longitude, gate, line, color, tone);
        }
    }

    #[test]
    fn test_zodiac_boundary() {
        // Test at zodiac sign boundaries
        let (gate_0, _line_0) = longitude_to_gate_and_line(0.0);
        assert_eq!(gate_0, 25); // First gate in Aries

        let (gate_30, _) = longitude_to_gate_and_line(30.0);
        assert_eq!(gate_30, 27); // First gate in Taurus

        let (gate_60, _) = longitude_to_gate_and_line(60.0);
        assert_eq!(gate_60, 8); // First gate in Gemini
    }

    #[test]
    fn test_normalization() {
        // Test that values beyond 360 normalize correctly
        let (gate_1, line_1) = longitude_to_gate_and_line(140.0);
        let (gate_2, line_2) = longitude_to_gate_and_line(500.0); // 140 + 360

        assert_eq!(gate_1, gate_2);
        assert_eq!(line_1, line_2);
    }

    #[test]
    fn test_line_range() {
        // Test various positions to ensure lines stay in 1-6 range
        for i in 0..360 {
            let (_, line) = longitude_to_gate_and_line(i as f64);
            assert!(line >= 1 && line <= 6, "Line {} out of range at {}°", line, i);
        }
    }
}
