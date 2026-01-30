// Standalone test for Panchanga calculator without full engine dependencies

use chrono::{DateTime, Utc};
use std::f64::consts::PI;

// Copy the essential types and calculator for standalone testing
#[derive(Debug, Clone)]
pub struct Tithi {
    pub number: u8,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Nakshatra {
    pub number: u8,
    pub name: String,
    pub start_longitude: f64,
    pub end_longitude: f64,
    pub ruler: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Yoga {
    pub number: u8,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Karana {
    pub number: u8,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Vara {
    pub number: u8,
    pub name: String,
    pub ruler: String,
}

#[derive(Debug, Clone)]
struct NakshatraData {
    number: u8,
    name: String,
    start_longitude: f64,
    end_longitude: f64,
    ruler: String,
}

#[derive(Debug, Clone)]
struct YogaData {
    number: u8,
    name: String,
}

#[derive(Debug, Clone)]
struct KaranaData {
    number: u8,
    name: String,
}

pub struct PanchangaCalculator {
    nakshatra_data: Vec<NakshatraData>,
    yoga_data: Vec<YogaData>,
    karana_data: Vec<KaranaData>,
    tithi_names: Vec<String>,
    vara_names: Vec<String>,
}

impl PanchangaCalculator {
    pub fn new() -> Self {
        Self {
            nakshatra_data: Self::initialize_nakshatra_data(),
            yoga_data: Self::initialize_yoga_data(),
            karana_data: Self::initialize_karana_data(),
            tithi_names: Self::initialize_tithi_names(),
            vara_names: Self::initialize_vara_names(),
        }
    }

    pub fn calculate_tithi(&self, solar_longitude: f64, lunar_longitude: f64, jd: f64) -> Result<Tithi, String> {
        let longitude_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        let tithi_value = (longitude_diff / 12.0).floor() + 1.0;
        let tithi_number = ((tithi_value as u8 - 1) % 30) + 1;
        
        Ok(Tithi {
            number: tithi_number,
            name: self.tithi_names.get((tithi_number - 1) as usize)
                .unwrap_or(&"Unknown".to_string())
                .clone(),
            start_time: None,
            end_time: None,
            duration: None,
        })
    }

    pub fn calculate_nakshatra(&self, lunar_longitude: f64, jd: f64) -> Result<Nakshatra, String> {
        let nakshatra_span = 360.0 / 27.0;
        let nakshatra_index = (lunar_longitude / nakshatra_span).floor() as usize;
        
        if nakshatra_index >= self.nakshatra_data.len() {
            return Err("Invalid longitude".to_string());
        }
        
        let nakshatra_info = &self.nakshatra_data[nakshatra_index];
        let start_longitude = nakshatra_index as f64 * nakshatra_span;
        let end_longitude = start_longitude + nakshatra_span;
        
        Ok(Nakshatra {
            number: nakshatra_info.number,
            name: nakshatra_info.name.clone(),
            start_longitude,
            end_longitude,
            ruler: nakshatra_info.ruler.clone(),
            start_time: None,
            end_time: None,
        })
    }

    pub fn calculate_yoga(&self, solar_longitude: f64, lunar_longitude: f64, jd: f64) -> Result<Yoga, String> {
        let yoga_sum = (solar_longitude + lunar_longitude).rem_euclid(360.0);
        let yoga_span = 360.0 / 27.0;
        let yoga_index = (yoga_sum / yoga_span).floor() as usize;
        
        if yoga_index >= self.yoga_data.len() {
            return Err("Invalid longitude".to_string());
        }
        
        let yoga_info = &self.yoga_data[yoga_index];
        
        Ok(Yoga {
            number: yoga_info.number,
            name: yoga_info.name.clone(),
            start_time: None,
            end_time: None,
        })
    }

    pub fn calculate_karana(&self, solar_longitude: f64, lunar_longitude: f64, jd: f64) -> Result<Karana, String> {
        let longitude_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        let karana_value = (longitude_diff / 6.0).floor() + 1.0;
        let karana_number = ((karana_value as u8 - 1) % 60) + 1;
        
        Ok(Karana {
            number: karana_number,
            name: self.karana_data.get((karana_number - 1) as usize)
                .map(|k| k.name.clone())
                .unwrap_or_else(|| {
                    if karana_number <= 7 || karana_number >= 57 {
                        self.karana_data.get(((karana_number - 1) % 11) as usize).unwrap().name.clone()
                    } else {
                        "Unknown".to_string()
                    }
                }),
            start_time: None,
            end_time: None,
        })
    }

    pub fn calculate_vara(&self, jd: f64) -> Result<Vara, String> {
        let weekday_number = ((jd + 1.0) as u8 % 7) + 1;
        
        let vara_name = self.vara_names.get((weekday_number - 1) as usize)
            .unwrap_or(&"Unknown".to_string())
            .clone();
        
        let ruler = match weekday_number {
            1 => "Moon",
            2 => "Mars", 
            3 => "Mercury",
            4 => "Jupiter",
            5 => "Venus",
            6 => "Saturn",
            7 => "Sun",
            _ => "Unknown",
        }.to_string();
        
        Ok(Vara {
            number: weekday_number,
            name: vara_name,
            ruler,
        })
    }

    fn initialize_nakshatra_data() -> Vec<NakshatraData> {
        vec![
            NakshatraData { number: 1, name: "Ashwini".to_string(), start_longitude: 0.0, end_longitude: 13.333, ruler: "Ketu".to_string() },
            NakshatraData { number: 2, name: "Bharani".to_string(), start_longitude: 13.333, end_longitude: 26.667, ruler: "Venus".to_string() },
            NakshatraData { number: 3, name: "Krittika".to_string(), start_longitude: 26.667, end_longitude: 40.0, ruler: "Sun".to_string() },
            NakshatraData { number: 4, name: "Rohini".to_string(), start_longitude: 40.0, end_longitude: 53.333, ruler: "Moon".to_string() },
            NakshatraData { number: 5, name: "Mrigashira".to_string(), start_longitude: 53.333, end_longitude: 66.667, ruler: "Mars".to_string() },
            NakshatraData { number: 6, name: "Ardra".to_string(), start_longitude: 66.667, end_longitude: 80.0, ruler: "Rahu".to_string() },
            NakshatraData { number: 7, name: "Punarvasu".to_string(), start_longitude: 80.0, end_longitude: 93.333, ruler: "Jupiter".to_string() },
            NakshatraData { number: 8, name: "Pushya".to_string(), start_longitude: 93.333, end_longitude: 106.667, ruler: "Saturn".to_string() },
            NakshatraData { number: 9, name: "Ashlesha".to_string(), start_longitude: 106.667, end_longitude: 120.0, ruler: "Mercury".to_string() },
            NakshatraData { number: 10, name: "Magha".to_string(), start_longitude: 120.0, end_longitude: 133.333, ruler: "Ketu".to_string() },
            NakshatraData { number: 11, name: "Purva Phalguni".to_string(), start_longitude: 133.333, end_longitude: 146.667, ruler: "Venus".to_string() },
            NakshatraData { number: 12, name: "Uttara Phalguni".to_string(), start_longitude: 146.667, end_longitude: 160.0, ruler: "Sun".to_string() },
            NakshatraData { number: 13, name: "Hasta".to_string(), start_longitude: 160.0, end_longitude: 173.333, ruler: "Moon".to_string() },
            NakshatraData { number: 14, name: "Chitra".to_string(), start_longitude: 173.333, end_longitude: 186.667, ruler: "Mars".to_string() },
            NakshatraData { number: 15, name: "Swati".to_string(), start_longitude: 186.667, end_longitude: 200.0, ruler: "Rahu".to_string() },
            NakshatraData { number: 16, name: "Vishakha".to_string(), start_longitude: 200.0, end_longitude: 213.333, ruler: "Jupiter".to_string() },
            NakshatraData { number: 17, name: "Anuradha".to_string(), start_longitude: 213.333, end_longitude: 226.667, ruler: "Saturn".to_string() },
            NakshatraData { number: 18, name: "Jyeshtha".to_string(), start_longitude: 226.667, end_longitude: 240.0, ruler: "Mercury".to_string() },
            NakshatraData { number: 19, name: "Mula".to_string(), start_longitude: 240.0, end_longitude: 253.333, ruler: "Ketu".to_string() },
            NakshatraData { number: 20, name: "Purva Ashadha".to_string(), start_longitude: 253.333, end_longitude: 266.667, ruler: "Venus".to_string() },
            NakshatraData { number: 21, name: "Uttara Ashadha".to_string(), start_longitude: 266.667, end_longitude: 280.0, ruler: "Sun".to_string() },
            NakshatraData { number: 22, name: "Shravana".to_string(), start_longitude: 280.0, end_longitude: 293.333, ruler: "Moon".to_string() },
            NakshatraData { number: 23, name: "Dhanishtha".to_string(), start_longitude: 293.333, end_longitude: 306.667, ruler: "Mars".to_string() },
            NakshatraData { number: 24, name: "Shatabhisha".to_string(), start_longitude: 306.667, end_longitude: 320.0, ruler: "Rahu".to_string() },
            NakshatraData { number: 25, name: "Purva Bhadrapada".to_string(), start_longitude: 320.0, end_longitude: 333.333, ruler: "Jupiter".to_string() },
            NakshatraData { number: 26, name: "Uttara Bhadrapada".to_string(), start_longitude: 333.333, end_longitude: 346.667, ruler: "Saturn".to_string() },
            NakshatraData { number: 27, name: "Revati".to_string(), start_longitude: 346.667, end_longitude: 360.0, ruler: "Mercury".to_string() },
        ]
    }

    fn initialize_yoga_data() -> Vec<YogaData> {
        vec![
            YogaData { number: 1, name: "Vishkambha".to_string() },
            YogaData { number: 2, name: "Priti".to_string() },
            YogaData { number: 3, name: "Ayushman".to_string() },
            YogaData { number: 4, name: "Saubhagya".to_string() },
            YogaData { number: 5, name: "Shobhana".to_string() },
            YogaData { number: 6, name: "Atiganda".to_string() },
            YogaData { number: 7, name: "Sukarma".to_string() },
            YogaData { number: 8, name: "Dhriti".to_string() },
            YogaData { number: 9, name: "Shula".to_string() },
            YogaData { number: 10, name: "Ganda".to_string() },
            YogaData { number: 11, name: "Vriddhi".to_string() },
            YogaData { number: 12, name: "Dhruva".to_string() },
            YogaData { number: 13, name: "Vyaghata".to_string() },
            YogaData { number: 14, name: "Harshana".to_string() },
            YogaData { number: 15, name: "Vajra".to_string() },
            YogaData { number: 16, name: "Siddhi".to_string() },
            YogaData { number: 17, name: "Vyatipata".to_string() },
            YogaData { number: 18, name: "Variyan".to_string() },
            YogaData { number: 19, name: "Parigha".to_string() },
            YogaData { number: 20, name: "Shiva".to_string() },
            YogaData { number: 21, name: "Siddha".to_string() },
            YogaData { number: 22, name: "Sadhya".to_string() },
            YogaData { number: 23, name: "Shubha".to_string() },
            YogaData { number: 24, name: "Shukla".to_string() },
            YogaData { number: 25, name: "Brahma".to_string() },
            YogaData { number: 26, name: "Indra".to_string() },
            YogaData { number: 27, name: "Vaidhriti".to_string() },
        ]
    }

    fn initialize_karana_data() -> Vec<KaranaData> {
        vec![
            KaranaData { number: 1, name: "Bava".to_string() },
            KaranaData { number: 2, name: "Balava".to_string() },
            KaranaData { number: 3, name: "Kaulava".to_string() },
            KaranaData { number: 4, name: "Taitila".to_string() },
            KaranaData { number: 5, name: "Gara".to_string() },
            KaranaData { number: 6, name: "Vanija".to_string() },
            KaranaData { number: 7, name: "Vishti".to_string() },
            KaranaData { number: 8, name: "Shakuni".to_string() },
            KaranaData { number: 9, name: "Chatushpada".to_string() },
            KaranaData { number: 10, name: "Naga".to_string() },
            KaranaData { number: 11, name: "Kimstughna".to_string() },
        ]
    }

    fn initialize_tithi_names() -> Vec<String> {
        vec![
            "Pratipada".to_string(),
            "Dvitiya".to_string(),
            "Tritiya".to_string(),
            "Chaturthi".to_string(),
            "Panchami".to_string(),
            "Shashthi".to_string(),
            "Saptami".to_string(),
            "Ashtami".to_string(),
            "Navami".to_string(),
            "Dashami".to_string(),
            "Ekadashi".to_string(),
            "Dvadashi".to_string(),
            "Trayodashi".to_string(),
            "Chaturdashi".to_string(),
            "Purnima".to_string(),
            "Pratipada".to_string(),
            "Dvitiya".to_string(),
            "Tritiya".to_string(),
            "Chaturthi".to_string(),
            "Panchami".to_string(),
            "Shashthi".to_string(),
            "Saptami".to_string(),
            "Ashtami".to_string(),
            "Navami".to_string(),
            "Dashami".to_string(),
            "Ekadashi".to_string(),
            "Dvadashi".to_string(),
            "Trayodashi".to_string(),
            "Chaturdashi".to_string(),
            "Amavasya".to_string(),
        ]
    }

    fn initialize_vara_names() -> Vec<String> {
        vec![
            "Ravivara".to_string(),    // Sunday
            "Somavara".to_string(),    // Monday
            "Mangalavara".to_string(), // Tuesday
            "Budhavara".to_string(),   // Wednesday
            "Guruvara".to_string(),    // Thursday
            "Shukravara".to_string(),  // Friday
            "Shanivara".to_string(),   // Saturday
        ]
    }
}

/// Test Panchanga calculator functionality
#[test]
fn test_panchanga_calculator_basic() {
    let calculator = PanchangaCalculator::new();
    
    // Test Tithi calculation
    let solar_longitude = 120.0;
    let lunar_longitude = 135.0;
    let jd = 2451545.0; // J2000
    
    // Test Tithi
    match calculator.calculate_tithi(solar_longitude, lunar_longitude, jd) {
        Ok(tithi_info) => {
            assert!(tithi_info.number >= 1 && tithi_info.number <= 30);
            assert!(!tithi_info.name.is_empty());
            println!("Tithi: {} - {}", tithi_info.number, tithi_info.name);
        }
        Err(e) => panic!("Tithi calculation failed: {}", e),
    }
    
    // Test Nakshatra
    match calculator.calculate_nakshatra(lunar_longitude, jd) {
        Ok(nakshatra_info) => {
            assert!(nakshatra_info.number >= 1 && nakshatra_info.number <= 27);
            assert!(!nakshatra_info.name.is_empty());
            assert!(!nakshatra_info.ruler.is_empty());
            println!("Nakshatra: {} - {} (Ruler: {})", nakshatra_info.number, nakshatra_info.name, nakshatra_info.ruler);
        }
        Err(e) => panic!("Nakshatra calculation failed: {}", e),
    }
    
    // Test Yoga
    match calculator.calculate_yoga(solar_longitude, lunar_longitude, jd) {
        Ok(yoga_info) => {
            assert!(yoga_info.number >= 1 && yoga_info.number <= 27);
            assert!(!yoga_info.name.is_empty());
            println!("Yoga: {} - {}", yoga_info.number, yoga_info.name);
        }
        Err(e) => panic!("Yoga calculation failed: {}", e),
    }
    
    // Test Karana
    match calculator.calculate_karana(solar_longitude, lunar_longitude, jd) {
        Ok(karana_info) => {
            assert!(karana_info.number >= 1 && karana_info.number <= 60);
            assert!(!karana_info.name.is_empty());
            println!("Karana: {} - {}", karana_info.number, karana_info.name);
        }
        Err(e) => panic!("Karana calculation failed: {}", e),
    }
    
    // Test Vara
    match calculator.calculate_vara(jd) {
        Ok(vara_info) => {
            assert!(vara_info.number >= 1 && vara_info.number <= 7);
            assert!(!vara_info.name.is_empty());
            assert!(!vara_info.ruler.is_empty());
            println!("Vara: {} - {} (Ruler: {})", vara_info.number, vara_info.name, vara_info.ruler);
        }
        Err(e) => panic!("Vara calculation failed: {}", e),
    }
}

/// Test mathematical consistency
#[test]
fn test_mathematical_consistency() {
    let calculator = PanchangaCalculator::new();
    
    // Test Tithi calculation consistency
    let solar_longitude = 120.0;
    let lunar_longitude = 135.0;
    let jd = 2451545.0;
    
    match calculator.calculate_tithi(solar_longitude, lunar_longitude, jd) {
        Ok(tithi_info) => {
            let tithi_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0_f64);
            let expected_tithi = (tithi_diff / 12.0_f64).floor() + 1.0_f64;
            let actual_tithi = tithi_info.number as f64;
            
            let tolerance = 0.001;
            assert!(
                (expected_tithi - actual_tithi).abs() < tolerance,
                "Tithi calculation inconsistency: expected {}, actual {}",
                expected_tithi, actual_tithi
            );
        }
        Err(e) => panic!("Tithi calculation failed: {}", e),
    }
    
    // Test Nakshatra calculation consistency
    match calculator.calculate_nakshatra(lunar_longitude, jd) {
        Ok(nakshatra_info) => {
            let nakshatra_span = 360.0_f64 / 27.0_f64;
            let expected_nakshatra = (lunar_longitude / nakshatra_span).floor() as u8 + 1;
            let actual_nakshatra = nakshatra_info.number;
            
            assert_eq!(
                expected_nakshatra, actual_nakshatra,
                "Nakshatra calculation inconsistency: expected {}, actual {}",
                expected_nakshatra, actual_nakshatra
            );
        }
        Err(e) => panic!("Nakshatra calculation failed: {}", e),
    }
}

/// Test edge cases
#[test]
fn test_edge_cases() {
    let calculator = PanchangaCalculator::new();
    
    // Test with zero longitudes
    let jd = 2451545.0;
    
    match calculator.calculate_tithi(0.0, 0.0, jd) {
        Ok(tithi_info) => {
            assert_eq!(tithi_info.number, 1);
            println!("Zero longitudes Tithi: {}", tithi_info.number);
        }
        Err(e) => panic!("Zero longitude Tithi calculation failed: {}", e),
    }
    
    // Test with maximum longitudes
    match calculator.calculate_tithi(359.0, 1.0, jd) {
        Ok(tithi_info) => {
            assert!(tithi_info.number >= 1 && tithi_info.number <= 30);
            println!("Max longitudes Tithi: {}", tithi_info.number);
        }
        Err(e) => panic!("Max longitude Tithi calculation failed: {}", e),
    }
}
