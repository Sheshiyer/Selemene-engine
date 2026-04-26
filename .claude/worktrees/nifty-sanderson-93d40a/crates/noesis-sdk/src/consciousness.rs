//! Consciousness Level Definitions — The 6-level awareness progression
//!
//! Not gamification. Meeting the witness where they are.

/// Consciousness level metadata.
pub struct ConsciousnessLevel {
    /// Numeric level (0-5)
    pub level: u8,
    /// Single-word state name
    pub state: &'static str,
    /// Brief description of this level
    pub description: &'static str,
    /// How witness prompts are calibrated at this level
    pub prompt_style: &'static str,
    /// Visual indicator (filled/empty dots)
    pub dots: &'static str,
}

/// All 6 consciousness levels.
pub const LEVELS: &[ConsciousnessLevel] = &[
    ConsciousnessLevel {
        level: 0,
        state: "Dormant",
        description: "Beginning to notice patterns exist",
        prompt_style: "Observational — just notice, no interpretation",
        dots: "○○○○○",
    },
    ConsciousnessLevel {
        level: 1,
        state: "Glimpsing",
        description: "Starting to see recurring themes in your life",
        prompt_style: "Reflective — what feels familiar?",
        dots: "●○○○○",
    },
    ConsciousnessLevel {
        level: 2,
        state: "Practicing",
        description: "Actively working with patterns through inquiry",
        prompt_style: "Inquiry — separating observer from observed",
        dots: "●●○○○",
    },
    ConsciousnessLevel {
        level: 3,
        state: "Integrated",
        description: "Choosing conscious response over reaction",
        prompt_style: "Authorship — choosing how to respond",
        dots: "●●●○○",
    },
    ConsciousnessLevel {
        level: 4,
        state: "Embodied",
        description: "Awareness is your natural state",
        prompt_style: "Open — what wants to emerge?",
        dots: "●●●●○",
    },
    ConsciousnessLevel {
        level: 5,
        state: "Embodied",
        description: "The witness and the witnessed are one",
        prompt_style: "Open — what wants to emerge?",
        dots: "●●●●●",
    },
];

/// Reading count thresholds for auto-promotion.
/// After N readings, the user is promoted to the corresponding level.
pub const READING_THRESHOLDS: &[(u32, u8)] = &[
    (5, 1),   // 5+ readings → Phase 1 (Glimpsing)
    (15, 2),  // 15+ readings → Phase 2 (Practicing)
    (40, 3),  // 40+ readings → Phase 3 (Integrated)
    (80, 4),  // 80+ readings → Phase 4 (Embodied)
    (150, 5), // 150+ readings → Phase 5 (Embodied)
];

/// Compute the consciousness level from a reading count.
/// Returns the highest phase the reading count qualifies for.
pub fn level_from_readings(reading_count: u32) -> u8 {
    let mut level = 0u8;
    for &(threshold, phase) in READING_THRESHOLDS {
        if reading_count >= threshold {
            level = phase;
        }
    }
    level
}

/// Get the consciousness level info for a given level (clamped to 0-5).
pub fn get_level(level: u8) -> &'static ConsciousnessLevel {
    let idx = (level as usize).min(LEVELS.len() - 1);
    &LEVELS[idx]
}

/// Get the state name for a level (e.g., "Practicing").
pub fn level_name(level: u8) -> &'static str {
    get_level(level).state
}

/// Get the dot indicator for a level (e.g., "●●○○○").
pub fn level_dots(level: u8) -> &'static str {
    get_level(level).dots
}

/// Format a display string like "Phase 2/5 — Practicing ●●○○○"
pub fn level_display(level: u8) -> String {
    let info = get_level(level);
    format!("Phase {}/5 — {} {}", level, info.state, info.dots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_levels_defined() {
        assert_eq!(LEVELS.len(), 6);
    }

    #[test]
    fn test_level_display() {
        assert_eq!(level_display(0), "Phase 0/5 — Dormant ○○○○○");
        assert_eq!(level_display(2), "Phase 2/5 — Practicing ●●○○○");
        assert_eq!(level_display(5), "Phase 5/5 — Embodied ●●●●●");
    }

    #[test]
    fn test_clamp_out_of_range() {
        let info = get_level(99);
        assert_eq!(info.level, 5);
    }

    #[test]
    fn test_level_from_readings() {
        assert_eq!(level_from_readings(0), 0);
        assert_eq!(level_from_readings(4), 0);
        assert_eq!(level_from_readings(5), 1);
        assert_eq!(level_from_readings(14), 1);
        assert_eq!(level_from_readings(15), 2);
        assert_eq!(level_from_readings(39), 2);
        assert_eq!(level_from_readings(40), 3);
        assert_eq!(level_from_readings(79), 3);
        assert_eq!(level_from_readings(80), 4);
        assert_eq!(level_from_readings(149), 4);
        assert_eq!(level_from_readings(150), 5);
        assert_eq!(level_from_readings(9999), 5);
    }
}
