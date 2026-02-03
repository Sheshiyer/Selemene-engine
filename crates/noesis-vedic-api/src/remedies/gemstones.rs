//! Gemstone recommendations

use super::types::GemstoneRecommendation;

/// Get gemstone for a planet
pub fn get_gemstone_for_planet(planet: &str) -> GemstoneRecommendation {
    match planet.to_lowercase().as_str() {
        "sun" => GemstoneRecommendation {
            gemstone: "Ruby (Manik)".to_string(),
            planet: "Sun".to_string(),
            weight_carats: "3-6 carats".to_string(),
            metal: "Gold".to_string(),
            finger: "Ring finger".to_string(),
            day_to_wear: "Sunday".to_string(),
            nakshatra: Some("Pushya, Hasta, Uttara Phalguni".to_string()),
            mantra_before_wearing: "Om Suryaya Namah (7000 times or 108 times)".to_string(),
            benefits: vec![
                "Enhances leadership qualities".to_string(),
                "Improves health and vitality".to_string(),
                "Success in government matters".to_string(),
                "Increases confidence".to_string(),
            ],
            alternatives: vec!["Red Garnet".to_string(), "Red Spinel".to_string()],
        },
        "moon" => GemstoneRecommendation {
            gemstone: "Pearl (Moti)".to_string(),
            planet: "Moon".to_string(),
            weight_carats: "4-6 carats".to_string(),
            metal: "Silver".to_string(),
            finger: "Little finger".to_string(),
            day_to_wear: "Monday".to_string(),
            nakshatra: Some("Rohini, Hasta, Shravana".to_string()),
            mantra_before_wearing: "Om Chandraya Namah (11000 times or 108 times)".to_string(),
            benefits: vec![
                "Mental peace and stability".to_string(),
                "Good relationships with mother".to_string(),
                "Emotional balance".to_string(),
                "Enhanced intuition".to_string(),
            ],
            alternatives: vec!["Moonstone".to_string()],
        },
        "mars" => GemstoneRecommendation {
            gemstone: "Red Coral (Moonga)".to_string(),
            planet: "Mars".to_string(),
            weight_carats: "6-9 carats".to_string(),
            metal: "Gold or Copper".to_string(),
            finger: "Ring finger".to_string(),
            day_to_wear: "Tuesday".to_string(),
            nakshatra: Some("Mrigashira, Chitra, Dhanishta".to_string()),
            mantra_before_wearing: "Om Mangalaya Namah (10000 times or 108 times)".to_string(),
            benefits: vec![
                "Increases courage and energy".to_string(),
                "Success in competition".to_string(),
                "Property matters".to_string(),
                "Blood-related health".to_string(),
            ],
            alternatives: vec!["Carnelian".to_string()],
        },
        "mercury" => GemstoneRecommendation {
            gemstone: "Emerald (Panna)".to_string(),
            planet: "Mercury".to_string(),
            weight_carats: "3-5 carats".to_string(),
            metal: "Gold".to_string(),
            finger: "Little finger".to_string(),
            day_to_wear: "Wednesday".to_string(),
            nakshatra: Some("Ashlesha, Jyeshtha, Revati".to_string()),
            mantra_before_wearing: "Om Budhaya Namah (9000 times or 108 times)".to_string(),
            benefits: vec![
                "Enhanced intelligence".to_string(),
                "Better communication".to_string(),
                "Business success".to_string(),
                "Educational achievements".to_string(),
            ],
            alternatives: vec!["Green Tourmaline".to_string(), "Peridot".to_string()],
        },
        "jupiter" => GemstoneRecommendation {
            gemstone: "Yellow Sapphire (Pukhraj)".to_string(),
            planet: "Jupiter".to_string(),
            weight_carats: "3-5 carats".to_string(),
            metal: "Gold".to_string(),
            finger: "Index finger".to_string(),
            day_to_wear: "Thursday".to_string(),
            nakshatra: Some("Punarvasu, Vishakha, Purva Bhadrapada".to_string()),
            mantra_before_wearing: "Om Gurave Namah (19000 times or 108 times)".to_string(),
            benefits: vec![
                "Wisdom and knowledge".to_string(),
                "Good fortune".to_string(),
                "Marriage and children".to_string(),
                "Spiritual growth".to_string(),
            ],
            alternatives: vec!["Yellow Topaz".to_string(), "Citrine".to_string()],
        },
        "venus" => GemstoneRecommendation {
            gemstone: "Diamond (Heera)".to_string(),
            planet: "Venus".to_string(),
            weight_carats: "0.5-1 carat".to_string(),
            metal: "Platinum or White Gold".to_string(),
            finger: "Ring finger or Middle finger".to_string(),
            day_to_wear: "Friday".to_string(),
            nakshatra: Some("Bharani, Purva Phalguni, Purva Ashadha".to_string()),
            mantra_before_wearing: "Om Shukraya Namah (16000 times or 108 times)".to_string(),
            benefits: vec![
                "Love and relationships".to_string(),
                "Luxury and comfort".to_string(),
                "Artistic abilities".to_string(),
                "Beauty and charm".to_string(),
            ],
            alternatives: vec!["White Sapphire".to_string(), "Zircon".to_string()],
        },
        "saturn" => GemstoneRecommendation {
            gemstone: "Blue Sapphire (Neelam)".to_string(),
            planet: "Saturn".to_string(),
            weight_carats: "4-7 carats".to_string(),
            metal: "Gold or Silver".to_string(),
            finger: "Middle finger".to_string(),
            day_to_wear: "Saturday".to_string(),
            nakshatra: Some("Pushya, Anuradha, Uttara Bhadrapada".to_string()),
            mantra_before_wearing: "Om Shanishcharaya Namah (23000 times or 108 times)".to_string(),
            benefits: vec![
                "Career stability".to_string(),
                "Protection from enemies".to_string(),
                "Longevity".to_string(),
                "Discipline and focus".to_string(),
            ],
            alternatives: vec!["Amethyst".to_string(), "Iolite".to_string()],
        },
        "rahu" => GemstoneRecommendation {
            gemstone: "Hessonite (Gomed)".to_string(),
            planet: "Rahu".to_string(),
            weight_carats: "5-7 carats".to_string(),
            metal: "Silver".to_string(),
            finger: "Middle finger".to_string(),
            day_to_wear: "Saturday".to_string(),
            nakshatra: Some("Ardra, Swati, Shatabhisha".to_string()),
            mantra_before_wearing: "Om Rahave Namah (18000 times or 108 times)".to_string(),
            benefits: vec![
                "Protection from enemies".to_string(),
                "Success in foreign lands".to_string(),
                "Research abilities".to_string(),
                "Overcoming obstacles".to_string(),
            ],
            alternatives: vec!["Orange Zircon".to_string()],
        },
        "ketu" => GemstoneRecommendation {
            gemstone: "Cat's Eye (Lehsunia)".to_string(),
            planet: "Ketu".to_string(),
            weight_carats: "3-5 carats".to_string(),
            metal: "Gold or Silver".to_string(),
            finger: "Ring finger or Little finger".to_string(),
            day_to_wear: "Tuesday or Saturday".to_string(),
            nakshatra: Some("Ashwini, Magha, Mula".to_string()),
            mantra_before_wearing: "Om Ketave Namah (17000 times or 108 times)".to_string(),
            benefits: vec![
                "Spiritual enlightenment".to_string(),
                "Protection from accidents".to_string(),
                "Moksha path".to_string(),
                "Hidden knowledge".to_string(),
            ],
            alternatives: vec!["Tiger's Eye".to_string()],
        },
        _ => GemstoneRecommendation {
            gemstone: "Unknown".to_string(),
            planet: planet.to_string(),
            weight_carats: "Consult astrologer".to_string(),
            metal: "Varies".to_string(),
            finger: "Varies".to_string(),
            day_to_wear: "Varies".to_string(),
            nakshatra: None,
            mantra_before_wearing: "Consult astrologer".to_string(),
            benefits: vec![],
            alternatives: vec![],
        },
    }
}

/// Check if gemstone is suitable (basic rules)
pub fn is_gemstone_suitable(planet: &str, is_benefic: bool, is_well_placed: bool) -> (bool, String) {
    if is_benefic && is_well_placed {
        (true, "Gemstone is recommended to strengthen this benefic planet.".to_string())
    } else if is_benefic && !is_well_placed {
        (true, "Gemstone may help strengthen the weakened benefic planet.".to_string())
    } else if !is_benefic && is_well_placed {
        (false, "Malefic planet is already strong. Gemstone not recommended.".to_string())
    } else {
        (false, "Weak malefic planet. Consult astrologer before wearing gemstone.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemstone_for_planet() {
        let ruby = get_gemstone_for_planet("Sun");
        assert_eq!(ruby.gemstone, "Ruby (Manik)");
        assert_eq!(ruby.day_to_wear, "Sunday");
    }

    #[test]
    fn test_gemstone_suitability() {
        let (suitable, _) = is_gemstone_suitable("Jupiter", true, true);
        assert!(suitable);
    }
}
