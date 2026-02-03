//! Mantra recommendations

use super::types::MantraRecommendation;

/// Get mantra for a planet
pub fn get_mantra_for_planet(planet: &str) -> MantraRecommendation {
    match planet.to_lowercase().as_str() {
        "sun" => MantraRecommendation {
            mantra: "Om Suryaya Namah".to_string(),
            planet: "Sun".to_string(),
            count: 7000,
            timing: "Sunday morning at sunrise".to_string(),
            direction: Some("East".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Increases vitality".to_string(),
                "Enhances leadership".to_string(),
                "Government favor".to_string(),
            ],
        },
        "moon" => MantraRecommendation {
            mantra: "Om Chandraya Namah".to_string(),
            planet: "Moon".to_string(),
            count: 11000,
            timing: "Monday evening".to_string(),
            direction: Some("North-West".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Mental peace".to_string(),
                "Emotional stability".to_string(),
                "Good sleep".to_string(),
            ],
        },
        "mars" => MantraRecommendation {
            mantra: "Om Mangalaya Namah".to_string(),
            planet: "Mars".to_string(),
            count: 10000,
            timing: "Tuesday morning".to_string(),
            direction: Some("South".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Courage and strength".to_string(),
                "Victory over enemies".to_string(),
                "Property gains".to_string(),
            ],
        },
        "mercury" => MantraRecommendation {
            mantra: "Om Budhaya Namah".to_string(),
            planet: "Mercury".to_string(),
            count: 9000,
            timing: "Wednesday morning".to_string(),
            direction: Some("North".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Intelligence".to_string(),
                "Communication skills".to_string(),
                "Business success".to_string(),
            ],
        },
        "jupiter" => MantraRecommendation {
            mantra: "Om Gurave Namah".to_string(),
            planet: "Jupiter".to_string(),
            count: 19000,
            timing: "Thursday morning".to_string(),
            direction: Some("North-East".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Wisdom".to_string(),
                "Good fortune".to_string(),
                "Spiritual progress".to_string(),
            ],
        },
        "venus" => MantraRecommendation {
            mantra: "Om Shukraya Namah".to_string(),
            planet: "Venus".to_string(),
            count: 16000,
            timing: "Friday morning".to_string(),
            direction: Some("South-East".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Love and relationships".to_string(),
                "Artistic abilities".to_string(),
                "Material comforts".to_string(),
            ],
        },
        "saturn" => MantraRecommendation {
            mantra: "Om Shanishcharaya Namah".to_string(),
            planet: "Saturn".to_string(),
            count: 23000,
            timing: "Saturday evening".to_string(),
            direction: Some("West".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Removes obstacles".to_string(),
                "Career stability".to_string(),
                "Longevity".to_string(),
            ],
        },
        "rahu" => MantraRecommendation {
            mantra: "Om Rahave Namah".to_string(),
            planet: "Rahu".to_string(),
            count: 18000,
            timing: "Saturday night".to_string(),
            direction: Some("South-West".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Protection from enemies".to_string(),
                "Success abroad".to_string(),
                "Removes confusion".to_string(),
            ],
        },
        "ketu" => MantraRecommendation {
            mantra: "Om Ketave Namah".to_string(),
            planet: "Ketu".to_string(),
            count: 17000,
            timing: "Tuesday or Saturday".to_string(),
            direction: Some("South-West".to_string()),
            duration: "Minimum 40 days".to_string(),
            benefits: vec![
                "Spiritual progress".to_string(),
                "Moksha path".to_string(),
                "Hidden knowledge".to_string(),
            ],
        },
        _ => MantraRecommendation {
            mantra: "Om".to_string(),
            planet: planet.to_string(),
            count: 108,
            timing: "Any time".to_string(),
            direction: None,
            duration: "As needed".to_string(),
            benefits: vec!["General well-being".to_string()],
        },
    }
}

/// Get Gayatri mantra variant for planet
pub fn get_gayatri_for_planet(planet: &str) -> String {
    match planet.to_lowercase().as_str() {
        "sun" => "Om Bhaskaraya Vidmahe, Divakaraya Dhimahi, Tanno Suryah Prachodayat".to_string(),
        "moon" => "Om Padmadwajaya Vidmahe, Hema Roopaya Dhimahi, Tanno Soma Prachodayat".to_string(),
        "mars" => "Om Angarkaya Vidmahe, Shakti Hastaya Dhimahi, Tanno Bhaumah Prachodayat".to_string(),
        "mercury" => "Om Gajadhwajaya Vidmahe, Shukla Hastaya Dhimahi, Tanno Budhah Prachodayat".to_string(),
        "jupiter" => "Om Vrishabadhwajaya Vidmahe, Gruni Hastaya Dhimahi, Tanno Guruh Prachodayat".to_string(),
        "venus" => "Om Ashwadhwajaya Vidmahe, Dhanur Hastaya Dhimahi, Tanno Shukrah Prachodayat".to_string(),
        "saturn" => "Om Kakadwajaya Vidmahe, Khadga Hastaya Dhimahi, Tanno Mandah Prachodayat".to_string(),
        "rahu" => "Om Sookdantaya Vidmahe, Ugraroopaya Dhimahi, Tanno Rahuh Prachodayat".to_string(),
        "ketu" => "Om Ashwadhwajaya Vidmahe, Shool Hastaya Dhimahi, Tanno Ketuh Prachodayat".to_string(),
        _ => "Om Bhur Bhuva Swaha, Tat Savitur Varenyam, Bhargo Devasya Dhimahi, Dhiyo Yo Nah Prachodayat".to_string(),
    }
}

/// Get Beej (seed) mantra for planet
pub fn get_beej_mantra(planet: &str) -> String {
    match planet.to_lowercase().as_str() {
        "sun" => "Om Hraam Hreem Hraum Sah Suryaya Namah".to_string(),
        "moon" => "Om Shraam Shreem Shraum Sah Chandraya Namah".to_string(),
        "mars" => "Om Kraam Kreem Kraum Sah Bhaumaya Namah".to_string(),
        "mercury" => "Om Braam Breem Braum Sah Budhaya Namah".to_string(),
        "jupiter" => "Om Graam Greem Graum Sah Gurave Namah".to_string(),
        "venus" => "Om Draam Dreem Draum Sah Shukraya Namah".to_string(),
        "saturn" => "Om Praam Preem Praum Sah Shanaye Namah".to_string(),
        "rahu" => "Om Bhraam Bhreem Bhraum Sah Rahave Namah".to_string(),
        "ketu" => "Om Sraam Sreem Sraum Sah Ketave Namah".to_string(),
        _ => "Om".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mantra_for_planet() {
        let mantra = get_mantra_for_planet("Jupiter");
        assert!(mantra.mantra.contains("Gurave"));
        assert_eq!(mantra.count, 19000);
    }

    #[test]
    fn test_beej_mantra() {
        let beej = get_beej_mantra("Saturn");
        assert!(beej.contains("Praam"));
    }
}
