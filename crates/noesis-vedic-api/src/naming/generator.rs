//! Name generator

use super::{NameSuggestion, NamingRequest, NamingResponse, NamkaranInfo, Gender};
use super::syllables::get_syllables_for_nakshatra;
use super::types::calculate_numerology;

/// Generate name suggestions
pub fn generate_names(request: &NamingRequest) -> NamingResponse {
    let syllables = get_syllables_for_nakshatra(&request.moon_nakshatra, request.nakshatra_pada);
    
    let mut suggestions = Vec::new();
    
    for syllable in &syllables {
        let names = get_names_for_syllable(syllable, request.gender, request.preferred_origin.as_deref());
        for name in names.into_iter().take(request.count / syllables.len().max(1) + 1) {
            suggestions.push(name);
        }
    }
    
    // Limit to requested count
    suggestions.truncate(request.count);
    
    NamingResponse {
        nakshatra: request.moon_nakshatra.clone(),
        pada: request.nakshatra_pada,
        syllables: syllables.clone(),
        suggestions,
        namkaran_info: get_namkaran_info(),
    }
}

fn get_names_for_syllable(syllable: &str, gender: Gender, origin: Option<&str>) -> Vec<NameSuggestion> {
    // In production, this would query a database
    // Here we provide some example names
    
    let all_names = get_sample_names();
    
    all_names.into_iter()
        .filter(|n| n.syllable.eq_ignore_ascii_case(syllable))
        .filter(|n| match gender {
            Gender::Male => n.gender == Gender::Male || n.gender == Gender::Neutral,
            Gender::Female => n.gender == Gender::Female || n.gender == Gender::Neutral,
            Gender::Neutral => true,
        })
        .filter(|n| {
            origin.map(|o| n.origin.eq_ignore_ascii_case(o)).unwrap_or(true)
        })
        .collect()
}

fn get_sample_names() -> Vec<NameSuggestion> {
    vec![
        // Ashwini - Chu, Che, Cho, La
        NameSuggestion { name: "Chudamani".to_string(), syllable: "Chu".to_string(), meaning: "Crest jewel".to_string(), gender: Gender::Female, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Chudamani"), popularity: 60 },
        NameSuggestion { name: "Chetan".to_string(), syllable: "Che".to_string(), meaning: "Consciousness".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Chetan"), popularity: 75 },
        NameSuggestion { name: "Lakshmi".to_string(), syllable: "La".to_string(), meaning: "Goddess of wealth".to_string(), gender: Gender::Female, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Lakshmi"), popularity: 90 },
        NameSuggestion { name: "Lakshmana".to_string(), syllable: "La".to_string(), meaning: "Marked with auspicious signs".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Lakshmana"), popularity: 70 },
        
        // Rohini - O, Va/Wa, Vi/Wi, Vu/Wu
        NameSuggestion { name: "Om".to_string(), syllable: "O".to_string(), meaning: "Sacred sound".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Om"), popularity: 85 },
        NameSuggestion { name: "Varun".to_string(), syllable: "Va".to_string(), meaning: "God of water".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Varun"), popularity: 80 },
        NameSuggestion { name: "Vidya".to_string(), syllable: "Vi".to_string(), meaning: "Knowledge".to_string(), gender: Gender::Female, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Vidya"), popularity: 85 },
        NameSuggestion { name: "Vivek".to_string(), syllable: "Vi".to_string(), meaning: "Wisdom".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Vivek"), popularity: 80 },
        
        // Pushya - Hu, He, Ho, Da
        NameSuggestion { name: "Harsh".to_string(), syllable: "Ha".to_string(), meaning: "Joy".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Harsh"), popularity: 75 },
        NameSuggestion { name: "Daksha".to_string(), syllable: "Da".to_string(), meaning: "Able, skilled".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Daksha"), popularity: 65 },
        
        // Magha - Ma, Mi, Mu, Me
        NameSuggestion { name: "Madhav".to_string(), syllable: "Ma".to_string(), meaning: "Krishna".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Madhav"), popularity: 80 },
        NameSuggestion { name: "Meera".to_string(), syllable: "Me".to_string(), meaning: "Ocean, limit".to_string(), gender: Gender::Female, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Meera"), popularity: 85 },
        
        // Revati - De, Do, Cha, Chi
        NameSuggestion { name: "Dev".to_string(), syllable: "De".to_string(), meaning: "God".to_string(), gender: Gender::Male, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Dev"), popularity: 85 },
        NameSuggestion { name: "Chitra".to_string(), syllable: "Chi".to_string(), meaning: "Picture".to_string(), gender: Gender::Female, origin: "Sanskrit".to_string(), numerology: calculate_numerology("Chitra"), popularity: 70 },
    ]
}

fn get_namkaran_info() -> NamkaranInfo {
    NamkaranInfo {
        recommended_day: "12th day after birth (or 11th, 13th if 12th is inauspicious)".to_string(),
        auspicious_tithis: vec![
            "Dwitiya".to_string(),
            "Tritiya".to_string(),
            "Panchami".to_string(),
            "Saptami".to_string(),
            "Dashami".to_string(),
            "Ekadashi".to_string(),
            "Trayodashi".to_string(),
        ],
        procedure: vec![
            "Perform Ganapati puja".to_string(),
            "Invoke blessings of family deity".to_string(),
            "Father whispers name in child's right ear".to_string(),
            "Mother whispers name in child's left ear".to_string(),
            "Name is announced to family and guests".to_string(),
            "Blessings and gifts given to child".to_string(),
        ],
        avoid: vec![
            "Amavasya (new moon)".to_string(),
            "Chaturthi".to_string(),
            "Navami".to_string(),
            "Chaturdashi".to_string(),
            "Tuesday and Saturday".to_string(),
            "Rahu Kalam".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_names() {
        let request = NamingRequest {
            moon_nakshatra: "Rohini".to_string(),
            nakshatra_pada: 1,
            gender: Gender::Male,
            preferred_origin: None,
            count: 5,
        };
        
        let response = generate_names(&request);
        assert!(!response.syllables.is_empty());
    }
}
