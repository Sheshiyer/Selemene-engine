//! Donation recommendations

use super::types::DonationRecommendation;

/// Get donation items for a planet
pub fn get_donation_for_planet(planet: &str) -> DonationRecommendation {
    match planet.to_lowercase().as_str() {
        "sun" => DonationRecommendation {
            planet: "Sun".to_string(),
            items: vec![
                "Wheat".to_string(),
                "Jaggery (gur)".to_string(),
                "Red cloth".to_string(),
                "Copper items".to_string(),
                "Ruby (if affordable)".to_string(),
            ],
            day: "Sunday".to_string(),
            to_whom: "Father, father figure, or temple priest".to_string(),
            benefits: "Reduces Sun's malefic effects, improves health and authority".to_string(),
        },
        "moon" => DonationRecommendation {
            planet: "Moon".to_string(),
            items: vec![
                "Rice".to_string(),
                "Milk".to_string(),
                "White cloth".to_string(),
                "Silver items".to_string(),
                "Curd".to_string(),
                "Pearl (if affordable)".to_string(),
            ],
            day: "Monday".to_string(),
            to_whom: "Mother, elderly women, or those in need".to_string(),
            benefits: "Mental peace, emotional stability, good relationship with mother".to_string(),
        },
        "mars" => DonationRecommendation {
            planet: "Mars".to_string(),
            items: vec![
                "Red lentils (masoor dal)".to_string(),
                "Red cloth".to_string(),
                "Copper items".to_string(),
                "Weapons (toy)".to_string(),
                "Red coral".to_string(),
            ],
            day: "Tuesday".to_string(),
            to_whom: "Soldiers, police, or young men".to_string(),
            benefits: "Reduces aggression, protects from accidents, success in competition".to_string(),
        },
        "mercury" => DonationRecommendation {
            planet: "Mercury".to_string(),
            items: vec![
                "Green mung beans".to_string(),
                "Green cloth".to_string(),
                "Books".to_string(),
                "Writing instruments".to_string(),
                "Emerald".to_string(),
            ],
            day: "Wednesday".to_string(),
            to_whom: "Students, scholars, or aunts/uncles".to_string(),
            benefits: "Improves intelligence, communication, and business".to_string(),
        },
        "jupiter" => DonationRecommendation {
            planet: "Jupiter".to_string(),
            items: vec![
                "Yellow gram (chana dal)".to_string(),
                "Yellow cloth".to_string(),
                "Turmeric".to_string(),
                "Gold (if affordable)".to_string(),
                "Bananas".to_string(),
                "Yellow sapphire".to_string(),
            ],
            day: "Thursday".to_string(),
            to_whom: "Teachers, priests, or elderly learned persons".to_string(),
            benefits: "Wisdom, good fortune, children, and spiritual growth".to_string(),
        },
        "venus" => DonationRecommendation {
            planet: "Venus".to_string(),
            items: vec![
                "Rice".to_string(),
                "White cloth".to_string(),
                "Perfume".to_string(),
                "Sweets".to_string(),
                "Sugar".to_string(),
                "Diamond or white sapphire".to_string(),
            ],
            day: "Friday".to_string(),
            to_whom: "Young women, artists, or wife".to_string(),
            benefits: "Love, beauty, comforts, and artistic success".to_string(),
        },
        "saturn" => DonationRecommendation {
            planet: "Saturn".to_string(),
            items: vec![
                "Black urad dal".to_string(),
                "Black cloth".to_string(),
                "Iron items".to_string(),
                "Mustard oil".to_string(),
                "Blue sapphire".to_string(),
                "Black sesame (til)".to_string(),
            ],
            day: "Saturday".to_string(),
            to_whom: "Elderly, servants, disabled, or poor people".to_string(),
            benefits: "Reduces Saturn's afflictions, career stability, longevity".to_string(),
        },
        "rahu" => DonationRecommendation {
            planet: "Rahu".to_string(),
            items: vec![
                "Blue cloth".to_string(),
                "Coconut".to_string(),
                "Blankets".to_string(),
                "Lead items".to_string(),
                "Hessonite".to_string(),
            ],
            day: "Saturday".to_string(),
            to_whom: "Sweepers, lower-caste people, or those in distress".to_string(),
            benefits: "Protection from hidden enemies, success abroad".to_string(),
        },
        "ketu" => DonationRecommendation {
            planet: "Ketu".to_string(),
            items: vec![
                "Multi-colored cloth".to_string(),
                "Blankets for animals".to_string(),
                "Dog food".to_string(),
                "Sesame seeds".to_string(),
                "Cat's eye stone".to_string(),
            ],
            day: "Tuesday or Saturday".to_string(),
            to_whom: "Ascetics, sadhus, or animal shelters".to_string(),
            benefits: "Spiritual progress, protection from accidents, moksha".to_string(),
        },
        _ => DonationRecommendation {
            planet: planet.to_string(),
            items: vec!["Food".to_string(), "Clothing".to_string()],
            day: "Any auspicious day".to_string(),
            to_whom: "Those in need".to_string(),
            benefits: "General well-being and karma improvement".to_string(),
        },
    }
}

/// Get charity recommendations for doshas
pub fn get_dosha_remedies(dosha: &str) -> Vec<String> {
    match dosha.to_lowercase().as_str() {
        "manglik" | "mangal dosha" => vec![
            "Donate red items on Tuesday".to_string(),
            "Chant Hanuman Chalisa".to_string(),
            "Fast on Tuesdays".to_string(),
            "Donate to Hanuman temple".to_string(),
            "Kumbh vivah if severe".to_string(),
        ],
        "kaal sarp" | "kalsarpa" => vec![
            "Donate to Rahu-Ketu temple".to_string(),
            "Perform Kaal Sarp Dosh puja".to_string(),
            "Feed birds and animals".to_string(),
            "Donate blue and brown items".to_string(),
            "Worship Shiva with milk".to_string(),
        ],
        "pitru dosha" => vec![
            "Perform Pitru tarpan on Amavasya".to_string(),
            "Donate food on death anniversaries".to_string(),
            "Feed crows and cows".to_string(),
            "Perform Narayan Bali if severe".to_string(),
            "Donate to Brahmins on Mahalaya".to_string(),
        ],
        "sade sati" => vec![
            "Worship Shani on Saturday".to_string(),
            "Donate oil and black items".to_string(),
            "Feed black dog or crow".to_string(),
            "Offer mustard oil to Hanuman".to_string(),
            "Recite Shani Chalisa".to_string(),
        ],
        _ => vec![
            "Perform relevant puja".to_string(),
            "Consult astrologer for specific remedies".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_donation_for_planet() {
        let donation = get_donation_for_planet("Saturn");
        assert!(donation.items.iter().any(|i| i.contains("urad")));
        assert_eq!(donation.day, "Saturday");
    }

    #[test]
    fn test_dosha_remedies() {
        let remedies = get_dosha_remedies("Manglik");
        assert!(remedies.iter().any(|r| r.contains("Tuesday")));
    }
}
