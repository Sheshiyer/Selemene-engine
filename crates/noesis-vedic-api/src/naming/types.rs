//! Naming types

use serde::{Deserialize, Serialize};

/// Nakshatra syllable mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NakshatraSyllables {
    pub nakshatra: String,
    pub pada1: Vec<String>,
    pub pada2: Vec<String>,
    pub pada3: Vec<String>,
    pub pada4: Vec<String>,
}

impl NakshatraSyllables {
    pub fn get_syllables_for_pada(&self, pada: u8) -> &[String] {
        match pada {
            1 => &self.pada1,
            2 => &self.pada2,
            3 => &self.pada3,
            4 => &self.pada4,
            _ => &self.pada1,
        }
    }
}

/// Name database entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameEntry {
    pub name: String,
    pub syllable: String,
    pub meaning: String,
    pub gender: String,
    pub origin: String,
}

/// Numerology calculation
pub fn calculate_numerology(name: &str) -> u8 {
    let sum: u32 = name
        .to_uppercase()
        .chars()
        .filter(|c| c.is_alphabetic())
        .map(|c| {
            let val = (c as u32) - ('A' as u32) + 1;
            val % 9
        })
        .sum();
    
    // Reduce to single digit
    let mut result = sum;
    while result > 9 {
        result = result.to_string()
            .chars()
            .map(|c| c.to_digit(10).unwrap_or(0))
            .sum();
    }
    
    result as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numerology() {
        let num = calculate_numerology("Rama");
        assert!(num >= 1 && num <= 9);
    }
}
