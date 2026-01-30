use chrono::{DateTime, Utc, NaiveDate, Datelike};
use std::f64::consts::PI;

// Completely standalone Panchanga calculation demo
// This file can be run independently without any library dependencies

#[derive(Debug, Clone)]
pub struct Tithi {
    pub number: u8,
    pub name: String,
    pub ruler: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Nakshatra {
    pub number: u8,
    pub name: String,
    pub ruler: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Yoga {
    pub number: u8,
    pub name: String,
    pub ruler: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Karana {
    pub number: u8,
    pub name: String,
    pub ruler: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Vara {
    pub number: u8,
    pub name: String,
    pub ruler: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

pub struct StandalonePanchangaCalculator {
    tithi_data: Vec<TithiData>,
    nakshatra_data: Vec<NakshatraData>,
    yoga_data: Vec<YogaData>,
    karana_data: Vec<KaranaData>,
    vara_data: Vec<VaraData>,
}

#[derive(Debug, Clone)]
struct TithiData {
    name: String,
    ruler: String,
}

#[derive(Debug, Clone)]
struct NakshatraData {
    name: String,
    ruler: String,
}

#[derive(Debug, Clone)]
struct YogaData {
    name: String,
    ruler: String,
}

#[derive(Debug, Clone)]
struct KaranaData {
    name: String,
    ruler: String,
}

#[derive(Debug, Clone)]
struct VaraData {
    name: String,
    ruler: String,
}

impl StandalonePanchangaCalculator {
    pub fn new() -> Self {
        let tithi_data = vec![
            TithiData { name: "Pratipada".to_string(), ruler: "Agni".to_string() },
            TithiData { name: "Dvitiya".to_string(), ruler: "Brahma".to_string() },
            TithiData { name: "Tritiya".to_string(), ruler: "Gauri".to_string() },
            TithiData { name: "Chaturthi".to_string(), ruler: "Ganapati".to_string() },
            TithiData { name: "Panchami".to_string(), ruler: "Serpent".to_string() },
            TithiData { name: "Shashthi".to_string(), ruler: "Kartikeya".to_string() },
            TithiData { name: "Saptami".to_string(), ruler: "Surya".to_string() },
            TithiData { name: "Ashtami".to_string(), ruler: "Shiva".to_string() },
            TithiData { name: "Navami".to_string(), ruler: "Durga".to_string() },
            TithiData { name: "Dashami".to_string(), ruler: "Yama".to_string() },
            TithiData { name: "Ekadashi".to_string(), ruler: "Vishnu".to_string() },
            TithiData { name: "Dvadashi".to_string(), ruler: "Vishnu".to_string() },
            TithiData { name: "Trayodashi".to_string(), ruler: "Kamadeva".to_string() },
            TithiData { name: "Chaturdashi".to_string(), ruler: "Shiva".to_string() },
            TithiData { name: "Purnima".to_string(), ruler: "Chandra".to_string() },
            TithiData { name: "Amavasya".to_string(), ruler: "Pitrs".to_string() },
        ];

        let nakshatra_data = vec![
            NakshatraData { name: "Ashwini".to_string(), ruler: "Ketu".to_string() },
            NakshatraData { name: "Bharani".to_string(), ruler: "Venus".to_string() },
            NakshatraData { name: "Krittika".to_string(), ruler: "Sun".to_string() },
            NakshatraData { name: "Rohini".to_string(), ruler: "Moon".to_string() },
            NakshatraData { name: "Mrigashira".to_string(), ruler: "Mars".to_string() },
            NakshatraData { name: "Ardra".to_string(), ruler: "Rahu".to_string() },
            NakshatraData { name: "Punarvasu".to_string(), ruler: "Jupiter".to_string() },
            NakshatraData { name: "Pushya".to_string(), ruler: "Saturn".to_string() },
            NakshatraData { name: "Ashlesha".to_string(), ruler: "Mercury".to_string() },
            NakshatraData { name: "Magha".to_string(), ruler: "Ketu".to_string() },
            NakshatraData { name: "Purva Phalguni".to_string(), ruler: "Venus".to_string() },
            NakshatraData { name: "Uttara Phalguni".to_string(), ruler: "Sun".to_string() },
            NakshatraData { name: "Hasta".to_string(), ruler: "Moon".to_string() },
            NakshatraData { name: "Chitra".to_string(), ruler: "Mars".to_string() },
            NakshatraData { name: "Swati".to_string(), ruler: "Rahu".to_string() },
            NakshatraData { name: "Vishakha".to_string(), ruler: "Jupiter".to_string() },
            NakshatraData { name: "Anuradha".to_string(), ruler: "Saturn".to_string() },
            NakshatraData { name: "Jyeshtha".to_string(), ruler: "Mercury".to_string() },
            NakshatraData { name: "Mula".to_string(), ruler: "Ketu".to_string() },
            NakshatraData { name: "Purva Ashadha".to_string(), ruler: "Venus".to_string() },
            NakshatraData { name: "Uttara Ashadha".to_string(), ruler: "Sun".to_string() },
            NakshatraData { name: "Shravana".to_string(), ruler: "Moon".to_string() },
            NakshatraData { name: "Dhanishtha".to_string(), ruler: "Mars".to_string() },
            NakshatraData { name: "Shatabhisha".to_string(), ruler: "Rahu".to_string() },
            NakshatraData { name: "Purva Bhadrapada".to_string(), ruler: "Jupiter".to_string() },
            NakshatraData { name: "Uttara Bhadrapada".to_string(), ruler: "Saturn".to_string() },
            NakshatraData { name: "Revati".to_string(), ruler: "Mercury".to_string() },
        ];

        let yoga_data = vec![
            YogaData { name: "Vishkambha".to_string(), ruler: "Agni".to_string() },
            YogaData { name: "Priti".to_string(), ruler: "Brahma".to_string() },
            YogaData { name: "Ayushman".to_string(), ruler: "Gauri".to_string() },
            YogaData { name: "Saubhagya".to_string(), ruler: "Ganapati".to_string() },
            YogaData { name: "Shobhana".to_string(), ruler: "Serpent".to_string() },
            YogaData { name: "Atiganda".to_string(), ruler: "Kartikeya".to_string() },
            YogaData { name: "Sukarma".to_string(), ruler: "Surya".to_string() },
            YogaData { name: "Dhriti".to_string(), ruler: "Shiva".to_string() },
            YogaData { name: "Shula".to_string(), ruler: "Durga".to_string() },
            YogaData { name: "Ganda".to_string(), ruler: "Yama".to_string() },
            YogaData { name: "Vriddhi".to_string(), ruler: "Vishnu".to_string() },
            YogaData { name: "Dhruva".to_string(), ruler: "Vishnu".to_string() },
            YogaData { name: "Vyaghata".to_string(), ruler: "Kamadeva".to_string() },
            YogaData { name: "Harshana".to_string(), ruler: "Shiva".to_string() },
            YogaData { name: "Vajra".to_string(), ruler: "Chandra".to_string() },
            YogaData { name: "Siddhi".to_string(), ruler: "Pitrs".to_string() },
            YogaData { name: "Vyatipata".to_string(), ruler: "Agni".to_string() },
            YogaData { name: "Variyan".to_string(), ruler: "Brahma".to_string() },
            YogaData { name: "Parigha".to_string(), ruler: "Gauri".to_string() },
            YogaData { name: "Shiva".to_string(), ruler: "Ganapati".to_string() },
            YogaData { name: "Siddha".to_string(), ruler: "Serpent".to_string() },
            YogaData { name: "Sadhya".to_string(), ruler: "Kartikeya".to_string() },
            YogaData { name: "Shubha".to_string(), ruler: "Surya".to_string() },
            YogaData { name: "Shukla".to_string(), ruler: "Shiva".to_string() },
            YogaData { name: "Brahma".to_string(), ruler: "Durga".to_string() },
            YogaData { name: "Indra".to_string(), ruler: "Yama".to_string() },
            YogaData { name: "Vaidhriti".to_string(), ruler: "Vishnu".to_string() },
        ];

        let karana_data = vec![
            KaranaData { name: "Bava".to_string(), ruler: "Agni".to_string() },
            KaranaData { name: "Balava".to_string(), ruler: "Brahma".to_string() },
            KaranaData { name: "Kaulava".to_string(), ruler: "Gauri".to_string() },
            KaranaData { name: "Taitila".to_string(), ruler: "Ganapati".to_string() },
            KaranaData { name: "Gara".to_string(), ruler: "Serpent".to_string() },
            KaranaData { name: "Vanija".to_string(), ruler: "Kartikeya".to_string() },
            KaranaData { name: "Vishti".to_string(), ruler: "Surya".to_string() },
            KaranaData { name: "Shakuni".to_string(), ruler: "Shiva".to_string() },
            KaranaData { name: "Chatushpada".to_string(), ruler: "Durga".to_string() },
            KaranaData { name: "Naga".to_string(), ruler: "Yama".to_string() },
            KaranaData { name: "Kimstughna".to_string(), ruler: "Vishnu".to_string() },
        ];

        let vara_data = vec![
            VaraData { name: "Ravivara".to_string(), ruler: "Sun".to_string() },
            VaraData { name: "Somavara".to_string(), ruler: "Moon".to_string() },
            VaraData { name: "Mangalavara".to_string(), ruler: "Mars".to_string() },
            VaraData { name: "Budhavara".to_string(), ruler: "Mercury".to_string() },
            VaraData { name: "Guruvara".to_string(), ruler: "Jupiter".to_string() },
            VaraData { name: "Shukravara".to_string(), ruler: "Venus".to_string() },
            VaraData { name: "Shanivara".to_string(), ruler: "Saturn".to_string() },
        ];

        Self {
            tithi_data,
            nakshatra_data,
            yoga_data,
            karana_data,
            vara_data,
        }
    }

    pub fn calculate_tithi(&self, solar_longitude: f64, lunar_longitude: f64, jd: f64) -> Tithi {
        let tithi_longitude = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        let tithi_number = ((tithi_longitude / 12.0).floor() as u8) + 1;
        
        let tithi_data = &self.tithi_data[(tithi_number - 1) as usize % 16];
        
        Tithi {
            number: tithi_number,
            name: tithi_data.name.clone(),
            ruler: tithi_data.ruler.clone(),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
            end_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
        }
    }

    pub fn calculate_nakshatra(&self, lunar_longitude: f64, jd: f64) -> Nakshatra {
        let nakshatra_longitude = lunar_longitude.rem_euclid(360.0);
        let nakshatra_number = ((nakshatra_longitude / 13.333333333333334).floor() as u8) + 1;
        
        let nakshatra_data = &self.nakshatra_data[(nakshatra_number - 1) as usize % 27];
        
        Nakshatra {
            number: nakshatra_number,
            name: nakshatra_data.name.clone(),
            ruler: nakshatra_data.ruler.clone(),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
            end_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
        }
    }

    pub fn calculate_yoga(&self, solar_longitude: f64, lunar_longitude: f64, jd: f64) -> Yoga {
        let yoga_longitude = (solar_longitude + lunar_longitude).rem_euclid(360.0);
        let yoga_number = ((yoga_longitude / 13.333333333333334).floor() as u8) + 1;
        
        let yoga_data = &self.yoga_data[(yoga_number - 1) as usize % 27];
        
        Yoga {
            number: yoga_number,
            name: yoga_data.name.clone(),
            ruler: yoga_data.ruler.clone(),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
            end_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
        }
    }

    pub fn calculate_karana(&self, solar_longitude: f64, lunar_longitude: f64, jd: f64) -> Karana {
        let tithi_longitude = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        let karana_number = ((tithi_longitude / 6.0).floor() as u8) + 1;
        
        let karana_data = &self.karana_data[(karana_number - 1) as usize % 11];
        
        Karana {
            number: karana_number,
            name: karana_data.name.clone(),
            ruler: karana_data.ruler.clone(),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
            end_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
        }
    }

    pub fn calculate_vara(&self, jd: f64) -> Vara {
        let vara_number = ((jd + 1.0).rem_euclid(7.0) as u8) + 1;
        
        let vara_data = &self.vara_data[(vara_number - 1) as usize % 7];
        
        Vara {
            number: vara_number,
            name: vara_data.name.clone(),
            ruler: vara_data.ruler.clone(),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
            end_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0).unwrap_or_default()),
        }
    }
}

// Helper functions for astronomical calculations
fn calculate_julian_day(date: NaiveDate) -> f64 {
    let year = date.year() as f64;
    let month = date.month() as f64;
    let day = date.day() as f64;
    
    // Julian Day calculation
    let a = ((14.0 - month) / 12.0).floor() as f64;
    let y = year + 4800.0 - a;
    let m = month + 12.0 * a - 3.0;
    
    let jd = day + ((153.0 * m + 2.0) / 5.0).floor() as f64 + 365.0 * y + (y / 4.0).floor() as f64 - (y / 100.0).floor() as f64 + (y / 400.0).floor() as f64 - 32045.0;
    
    jd
}

fn calculate_solar_longitude(jd: f64) -> f64 {
    // Simplified solar longitude calculation
    // This is an approximation - for production use, use proper ephemeris data
    let n = jd - 2451545.0; // Days since J2000.0
    let l = 280.460 + 0.9856474 * n; // Mean longitude
    let g = (357.528 + 0.9856003 * n) * PI / 180.0; // Mean anomaly in radians
    let lambda = l + 1.915 * g.sin() + 0.020 * (2.0 * g).sin(); // Ecliptic longitude
    
    lambda.rem_euclid(360.0)
}

fn calculate_lunar_longitude(jd: f64) -> f64 {
    // Simplified lunar longitude calculation
    // This is an approximation - for production use, use proper ephemeris data
    let n = jd - 2451545.0; // Days since J2000.0
    let l = 218.316 + 13.176396 * n; // Mean longitude
    let m = 134.963 + 13.064993 * n; // Mean anomaly
    let f = 93.272 + 13.229350 * n; // Argument of latitude
    
    let m_rad = m * PI / 180.0;
    let f_rad = f * PI / 180.0;
    
    // Major perturbations
    let lambda = l + 6.289 * m_rad.sin() + 1.274 * (2.0 * f_rad - 2.0 * m_rad).sin() + 0.658 * (2.0 * f_rad).sin();
    
    lambda.rem_euclid(360.0)
}

fn main() {
    println!("=== Current Day Panchanga Calculator ===\n");
    
    let calculator = StandalonePanchangaCalculator::new();
    
    // Current date and time
    let now = Utc::now();
    let current_date = now.date_naive();
    
    // Your coordinates: 22.626539502207024, 114.09190270441447
    let latitude = 22.626539502207024;
    let longitude = 114.09190270441447;
    
    // Calculate Julian Day for current date
    let jd = calculate_julian_day(current_date);
    
    // Simplified solar longitude calculation (approximate)
    let solar_longitude = calculate_solar_longitude(jd);
    
    // Simplified lunar longitude calculation (approximate)
    let lunar_longitude = calculate_lunar_longitude(jd);
    
    println!("Location: {:.6}°N, {:.6}°E", latitude, longitude);
    println!("Date: {}", current_date.format("%Y-%m-%d"));
    println!("Time: {}", now.format("%H:%M:%S UTC"));
    println!();
    println!("Calculated parameters:");
    println!("  Solar Longitude: {:.2}°", solar_longitude);
    println!("  Lunar Longitude: {:.2}°", lunar_longitude);
    println!("  Julian Day: {:.2}", jd);
    println!();
    
    // Calculate Tithi
    let tithi = calculator.calculate_tithi(solar_longitude, lunar_longitude, jd);
    println!("Tithi: {} ({}) - Ruler: {}", tithi.number, tithi.name, tithi.ruler);
    
    // Calculate Nakshatra
    let nakshatra = calculator.calculate_nakshatra(lunar_longitude, jd);
    println!("Nakshatra: {} ({}) - Ruler: {}", nakshatra.number, nakshatra.name, nakshatra.ruler);
    
    // Calculate Yoga
    let yoga = calculator.calculate_yoga(solar_longitude, lunar_longitude, jd);
    println!("Yoga: {} ({}) - Ruler: {}", yoga.number, yoga.name, yoga.ruler);
    
    // Calculate Karana
    let karana = calculator.calculate_karana(solar_longitude, lunar_longitude, jd);
    println!("Karana: {} ({}) - Ruler: {}", karana.number, karana.name, karana.ruler);
    
    // Calculate Vara
    let vara = calculator.calculate_vara(jd);
    println!("Vara: {} ({}) - Ruler: {}", vara.number, vara.name, vara.ruler);
    
    println!("\n=== Demo Complete ===");
}
