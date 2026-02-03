//! Eclipse effects and interpretations

use super::EclipseType;

/// Get astrological effects based on eclipse sign
pub fn get_sign_effects(sign: &str, eclipse_type: &EclipseType) -> Vec<String> {
    let base_effects = match sign {
        "Aries" => vec![
            "Self-identity and physical health focus",
            "Leadership challenges or changes",
            "New beginnings in personal matters",
        ],
        "Taurus" => vec![
            "Financial matters highlighted",
            "Values and possessions reviewed",
            "Changes in material security",
        ],
        "Gemini" => vec![
            "Communication matters affected",
            "Sibling relationships in focus",
            "Short travel plans may change",
        ],
        "Cancer" => vec![
            "Home and family matters emphasized",
            "Emotional revelations",
            "Mother figure in focus",
        ],
        "Leo" => vec![
            "Creative expression affected",
            "Romance and children matters",
            "Ego transformation",
        ],
        "Virgo" => vec![
            "Health and daily routines reviewed",
            "Service and work environment changes",
            "Analytical abilities enhanced",
        ],
        "Libra" => vec![
            "Partnerships under spotlight",
            "Balance in relationships tested",
            "Legal matters may arise",
        ],
        "Scorpio" => vec![
            "Transformation and regeneration",
            "Hidden matters revealed",
            "Joint resources affected",
        ],
        "Sagittarius" => vec![
            "Belief systems questioned",
            "Higher education or travel",
            "Philosophical insights",
        ],
        "Capricorn" => vec![
            "Career and public image",
            "Authority figures in focus",
            "Ambition and goals reviewed",
        ],
        "Aquarius" => vec![
            "Friendships and groups affected",
            "Future hopes examined",
            "Technology and innovation",
        ],
        "Pisces" => vec![
            "Spiritual matters heightened",
            "Hidden enemies or self-undoing",
            "Psychic abilities enhanced",
        ],
        _ => vec!["General transformation period"],
    };
    
    base_effects.into_iter().map(String::from).collect()
}

/// Get nakshatra-specific effects
pub fn get_nakshatra_effects(nakshatra: &str) -> String {
    match nakshatra {
        "Ashwini" => "Quick healing abilities tested. Medical matters in focus.",
        "Bharani" => "Transformation and endings. New beginnings from old patterns.",
        "Krittika" => "Purification through challenges. Truth revealed.",
        "Rohini" => "Material comforts affected. Creative expression changes.",
        "Mrigashira" => "Seeking and searching intensified. Curiosity peaks.",
        "Ardra" => "Destruction leading to renewal. Storms before calm.",
        "Punarvasu" => "Return and restoration. Recovery from setbacks.",
        "Pushya" => "Nourishment matters. Spiritual protection active.",
        "Ashlesha" => "Hidden matters surface. Intuition heightened.",
        "Magha" => "Ancestral connections. Authority and legacy.",
        "Purva Phalguni" => "Pleasure and recreation. Romantic matters.",
        "Uttara Phalguni" => "Partnerships and contracts. Commitments tested.",
        "Hasta" => "Skills and craftsmanship. Manual dexterity important.",
        "Chitra" => "Beauty and creativity. Artistic endeavors affected.",
        "Swati" => "Independence and self-reliance. Wind of change.",
        "Vishakha" => "Goals and determination. Single-minded focus.",
        "Anuradha" => "Friendship and devotion. Loyalty tested.",
        "Jyeshtha" => "Seniority and authority. Elder matters.",
        "Mula" => "Root causes examined. Destruction of illusions.",
        "Purva Ashadha" => "Invincibility tested. Water-related matters.",
        "Uttara Ashadha" => "Final victory. Unstoppable progress after effort.",
        "Shravana" => "Learning and listening. Knowledge acquisition.",
        "Dhanishta" => "Wealth and prosperity. Musical talents.",
        "Shatabhisha" => "Healing and secrets. Hundred physicians.",
        "Purva Bhadrapada" => "Intense transformations. Fire element active.",
        "Uttara Bhadrapada" => "Stability after storms. Wisdom gained.",
        "Revati" => "Journeys and completions. Safe passage.",
        _ => "General transformative influences.",
    }.to_string()
}

/// Get remedies for eclipse
pub fn get_eclipse_remedies(eclipse_type: &EclipseType) -> Vec<String> {
    let mut remedies = vec![
        "Chant protective mantras (Gayatri, Mahamrityunjaya)".to_string(),
        "Fast during eclipse period".to_string(),
        "Avoid eating and drinking during eclipse".to_string(),
        "Take bath after eclipse ends".to_string(),
        "Donate food and essentials to needy".to_string(),
    ];
    
    match eclipse_type {
        EclipseType::SolarTotal | EclipseType::SolarPartial | EclipseType::SolarAnnular | EclipseType::SolarHybrid => {
            remedies.push("Chant Aditya Hridayam".to_string());
            remedies.push("Offer water (arghya) to Sun after eclipse".to_string());
        }
        EclipseType::LunarTotal | EclipseType::LunarPartial | EclipseType::LunarPenumbral => {
            remedies.push("Chant Chandra mantras".to_string());
            remedies.push("Offer milk to Shiva lingam".to_string());
        }
    }
    
    remedies
}

/// Calculate sutak period
pub fn calculate_sutak_hours(eclipse_type: &EclipseType) -> u32 {
    // Sutak period before eclipse
    match eclipse_type {
        EclipseType::SolarTotal | EclipseType::SolarAnnular | EclipseType::SolarHybrid => 12,
        EclipseType::SolarPartial => 12,
        EclipseType::LunarTotal => 9,
        EclipseType::LunarPartial | EclipseType::LunarPenumbral => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_effects() {
        let effects = get_sign_effects("Aries", &EclipseType::SolarTotal);
        assert!(!effects.is_empty());
        assert!(effects.iter().any(|e| e.contains("Self-identity")));
    }

    #[test]
    fn test_sutak() {
        assert_eq!(calculate_sutak_hours(&EclipseType::SolarTotal), 12);
        assert_eq!(calculate_sutak_hours(&EclipseType::LunarTotal), 9);
    }
}
