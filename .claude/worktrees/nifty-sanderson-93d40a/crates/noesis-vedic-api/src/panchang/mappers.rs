//! Panchang response mappers
//!
//! FAPI-013: Map API Tithi response to internal model
//! FAPI-014: Map API Nakshatra response to internal model
//! FAPI-015: Map API Yoga response to internal model
//! FAPI-016: Map API Karana response to internal model

use crate::error::{VedicApiError, VedicApiResult};
use super::api::{
    PanchangApiResponse, TithiApiResponse, NakshatraApiResponse,
    YogaApiResponse, KaranaApiResponse, VaraApiResponse,
};
use super::data::{Panchang, Tithi, Nakshatra, Yoga, Karana, Vara, Paksha};

/// Map complete Panchang API response to internal model
pub fn map_panchang_response(response: PanchangApiResponse) -> VedicApiResult<Panchang> {
    Ok(Panchang {
        tithi: map_tithi_response(response.tithi)?,
        nakshatra: map_nakshatra_response(response.nakshatra)?,
        yoga: map_yoga_response(response.yoga)?,
        karana: map_karana_response(response.karana)?,
        vara: map_vara_response(response.vara)?,
        sunrise: response.sunrise,
        sunset: response.sunset,
        moonrise: response.moonrise,
        moonset: response.moonset,
    })
}

/// Map Tithi from API response to internal model
///
/// FAPI-013: Map API Tithi response to internal model
pub fn map_tithi_response(tithi: TithiApiResponse) -> VedicApiResult<Tithi> {
    let paksha = match tithi.paksha.to_lowercase().as_str() {
        "shukla" | "bright" | "waxing" => Paksha::Shukla,
        "krishna" | "dark" | "waning" => Paksha::Krishna,
        _ => return Err(VedicApiError::ParseError(
            format!("Unknown paksha: {}", tithi.paksha)
        )),
    };

    Ok(Tithi {
        number: tithi.number,
        name: tithi.name,
        paksha,
        end_time: tithi.end_time,
        deity: tithi.deity,
        percentage_elapsed: None,
    })
}

/// Map Nakshatra from API response to internal model
///
/// FAPI-014: Map API Nakshatra response to internal model
pub fn map_nakshatra_response(nakshatra: NakshatraApiResponse) -> VedicApiResult<Nakshatra> {
    Ok(Nakshatra {
        number: nakshatra.number,
        name: nakshatra.name,
        pada: nakshatra.pada.unwrap_or(1),
        end_time: nakshatra.end_time,
        lord: nakshatra.lord,
        deity: nakshatra.deity,
        percentage_elapsed: None,
    })
}

/// Map Yoga from API response to internal model
///
/// FAPI-015: Map API Yoga response to internal model
pub fn map_yoga_response(yoga: YogaApiResponse) -> VedicApiResult<Yoga> {
    let nature = classify_yoga_nature(yoga.number, &yoga.name);
    
    Ok(Yoga {
        number: yoga.number,
        name: yoga.name,
        end_time: yoga.end_time,
        meaning: yoga.meaning,
        nature,
    })
}

/// Map Karana from API response to internal model
///
/// FAPI-016: Map API Karana response to internal model
pub fn map_karana_response(karana: KaranaApiResponse) -> VedicApiResult<Karana> {
    let nature = classify_karana_nature(&karana.name);
    
    Ok(Karana {
        number: karana.number,
        name: karana.name,
        end_time: karana.end_time,
        nature,
    })
}

/// Map Vara (weekday) from API response
pub fn map_vara_response(vara: VaraApiResponse) -> VedicApiResult<Vara> {
    Ok(Vara {
        number: vara.number,
        name: vara.name,
        lord: vara.lord,
    })
}

/// Classify yoga nature based on number and name
fn classify_yoga_nature(number: u8, name: &str) -> YogaNature {
    // Auspicious yogas
    let auspicious = ["Siddhi", "Shubha", "Sadhya", "Shiva", "Siddha", "Dhruva", "Harshana", "Vajra"];
    // Inauspicious yogas
    let inauspicious = ["Vishkumbha", "Atiganda", "Shoola", "Ganda", "Vyatipata", "Vaidhrti", "Parigha", "Vyaghata"];
    
    if auspicious.iter().any(|y| name.contains(y)) {
        YogaNature::Auspicious
    } else if inauspicious.iter().any(|y| name.contains(y)) {
        YogaNature::Inauspicious
    } else {
        YogaNature::Mixed
    }
}

/// Classify karana nature based on name
fn classify_karana_nature(name: &str) -> KaranaNature {
    let fixed = ["Shakuni", "Chatushpada", "Nagava", "Kimstughna"];
    let movable = ["Bava", "Balava", "Kaulava", "Taitila", "Gara", "Vanija", "Vishti"];
    
    if fixed.iter().any(|k| name.contains(k)) {
        KaranaNature::Fixed
    } else if movable.iter().any(|k| name.contains(k)) {
        KaranaNature::Movable
    } else if name.contains("Vishti") || name.contains("Bhadra") {
        KaranaNature::Inauspicious
    } else {
        KaranaNature::Neutral
    }
}

/// Nature of a Yoga
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum YogaNature {
    Auspicious,
    Inauspicious,
    Mixed,
}

/// Nature of a Karana
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KaranaNature {
    Fixed,
    Movable,
    Inauspicious,
    Neutral,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_tithi_shukla() {
        let api_tithi = TithiApiResponse {
            number: 5,
            name: "Panchami".to_string(),
            paksha: "Shukla".to_string(),
            end_time: Some("14:30".to_string()),
            deity: Some("Naga".to_string()),
        };

        let tithi = map_tithi_response(api_tithi).unwrap();
        assert_eq!(tithi.number, 5);
        assert_eq!(tithi.paksha, Paksha::Shukla);
        assert_eq!(tithi.name, "Panchami");
    }

    #[test]
    fn test_map_tithi_krishna() {
        let api_tithi = TithiApiResponse {
            number: 10,
            name: "Dashami".to_string(),
            paksha: "Krishna".to_string(),
            end_time: None,
            deity: None,
        };

        let tithi = map_tithi_response(api_tithi).unwrap();
        assert_eq!(tithi.paksha, Paksha::Krishna);
    }

    #[test]
    fn test_map_nakshatra() {
        let api_nakshatra = NakshatraApiResponse {
            number: 1,
            name: "Ashwini".to_string(),
            pada: Some(3),
            end_time: Some("06:45".to_string()),
            lord: Some("Ketu".to_string()),
            deity: Some("Ashwini Kumaras".to_string()),
        };

        let nakshatra = map_nakshatra_response(api_nakshatra).unwrap();
        assert_eq!(nakshatra.number, 1);
        assert_eq!(nakshatra.name, "Ashwini");
        assert_eq!(nakshatra.pada, 3);
    }

    #[test]
    fn test_classify_yoga_nature() {
        assert_eq!(classify_yoga_nature(1, "Siddhi Yoga"), YogaNature::Auspicious);
        assert_eq!(classify_yoga_nature(2, "Vishkumbha"), YogaNature::Inauspicious);
        assert_eq!(classify_yoga_nature(15, "Vriksha"), YogaNature::Mixed);
    }

    #[test]
    fn test_classify_karana_nature() {
        assert_eq!(classify_karana_nature("Shakuni"), KaranaNature::Fixed);
        assert_eq!(classify_karana_nature("Bava"), KaranaNature::Movable);
        assert_eq!(classify_karana_nature("Vishti"), KaranaNature::Inauspicious);
    }
}
