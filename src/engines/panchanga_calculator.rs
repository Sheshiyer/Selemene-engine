use crate::models::{EngineError, Tithi, Nakshatra, Yoga, Karana, Vara};
use chrono::{DateTime, Utc};

/// Panchanga calculation errors
#[derive(Debug, thiserror::Error)]
pub enum PanchangaError {
    #[error("Invalid longitude: {0}")]
    InvalidLongitude(f64),
    #[error("Calculation precision error: {0}")]
    PrecisionError(String),
    #[error("Date parsing error: {0}")]
    DateError(String),
}

impl From<PanchangaError> for EngineError {
    fn from(err: PanchangaError) -> Self {
        EngineError::CalculationError(err.to_string())
    }
}

/// Panchanga calculator for Vedic astronomical calculations
pub struct PanchangaCalculator {
    nakshatra_data: Vec<NakshatraData>,
    yoga_data: Vec<YogaData>,
    karana_data: Vec<KaranaData>,
    tithi_names: Vec<String>,
    vara_names: Vec<String>,
}

#[derive(Debug, Clone)]
struct NakshatraData {
    number: u8,
    name: String,
    #[allow(dead_code)]
    start_longitude: f64,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    number: u8,
    name: String,
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

    /// Calculate Tithi (lunar day) from solar and lunar longitudes
    pub fn calculate_tithi(
        &self,
        solar_longitude: f64,
        lunar_longitude: f64,
        jd: f64,
    ) -> Result<Tithi, PanchangaError> {
        // Tithi = (Moon longitude - Sun longitude) / 12 degrees
        let longitude_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        let tithi_value = (longitude_diff / 12.0).floor() + 1.0;
        
        // Ensure tithi is between 1 and 30
        let tithi_number = ((tithi_value as u8 - 1) % 30) + 1;
        
        // Calculate tithi end time (when next tithi begins)
        let next_tithi_longitude = (tithi_value * 12.0) % 360.0;
        let end_time = self.calculate_tithi_end_time(
            jd,
            solar_longitude,
            lunar_longitude,
            next_tithi_longitude,
        )?;
        
        Ok(Tithi {
            number: tithi_number,
            name: self.tithi_names.get((tithi_number - 1) as usize)
                .unwrap_or(&"Unknown".to_string())
                .clone(),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0)
                .unwrap_or(Utc::now())),
            end_time: Some(end_time),
            duration: Some((end_time.timestamp() - 
                DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0)
                    .unwrap_or(Utc::now()).timestamp()) as f64 / 3600.0), // hours
        })
    }

    /// Calculate Nakshatra (lunar mansion) from lunar longitude
    pub fn calculate_nakshatra(
        &self,
        lunar_longitude: f64,
        jd: f64,
    ) -> Result<Nakshatra, PanchangaError> {
        // Each nakshatra spans 13.333... degrees (360/27)
        let nakshatra_span = 360.0 / 27.0;
        let nakshatra_index = (lunar_longitude / nakshatra_span).floor() as usize;
        
        if nakshatra_index >= self.nakshatra_data.len() {
            return Err(PanchangaError::InvalidLongitude(lunar_longitude));
        }
        
        let nakshatra_info = &self.nakshatra_data[nakshatra_index];
        let start_longitude = nakshatra_index as f64 * nakshatra_span;
        let end_longitude = start_longitude + nakshatra_span;
        
        // Calculate nakshatra end time
        let end_time = self.calculate_nakshatra_end_time(
            jd,
            lunar_longitude,
            end_longitude,
        )?;
        
        Ok(Nakshatra {
            number: nakshatra_info.number,
            name: nakshatra_info.name.clone(),
            start_longitude,
            end_longitude,
            ruler: nakshatra_info.ruler.clone(),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0)
                .unwrap_or(Utc::now())),
            end_time: Some(end_time),
        })
    }

    /// Calculate Yoga (astrological combination) from solar and lunar longitudes
    pub fn calculate_yoga(
        &self,
        solar_longitude: f64,
        lunar_longitude: f64,
        jd: f64,
    ) -> Result<Yoga, PanchangaError> {
        // Yoga = (Sun longitude + Moon longitude) / 13.333... degrees
        let yoga_sum = (solar_longitude + lunar_longitude).rem_euclid(360.0);
        let yoga_span = 360.0 / 27.0;
        let yoga_index = (yoga_sum / yoga_span).floor() as usize;
        
        if yoga_index >= self.yoga_data.len() {
            return Err(PanchangaError::InvalidLongitude(yoga_sum));
        }
        
        let yoga_info = &self.yoga_data[yoga_index];
        let next_yoga_longitude = ((yoga_index + 1) as f64 * yoga_span) % 360.0;
        
        // Calculate yoga end time
        let end_time = self.calculate_yoga_end_time(
            jd,
            solar_longitude,
            lunar_longitude,
            next_yoga_longitude,
        )?;
        
        Ok(Yoga {
            number: yoga_info.number,
            name: yoga_info.name.clone(),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0)
                .unwrap_or(Utc::now())),
            end_time: Some(end_time),
        })
    }

    /// Calculate Karana (half-Tithi) from solar and lunar longitudes
    pub fn calculate_karana(
        &self,
        solar_longitude: f64,
        lunar_longitude: f64,
        jd: f64,
    ) -> Result<Karana, PanchangaError> {
        // Karana = (Moon longitude - Sun longitude) / 6 degrees
        let longitude_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        let karana_value = (longitude_diff / 6.0).floor() + 1.0;
        
        // There are 60 karanas in a lunar month (2 per tithi)
        let karana_number = ((karana_value as u8 - 1) % 60) + 1;
        
        // Calculate karana end time
        let next_karana_longitude = (karana_value * 6.0) % 360.0;
        let end_time = self.calculate_karana_end_time(
            jd,
            solar_longitude,
            lunar_longitude,
            next_karana_longitude,
        )?;
        
        Ok(Karana {
            number: karana_number,
            name: self.karana_data.get((karana_number - 1) as usize)
                .map(|k| k.name.clone())
                .unwrap_or_else(|| {
                    // For karanas 1-7 and 57-60, use fixed names
                    if karana_number <= 7 || karana_number >= 57 {
                        self.karana_data.get(((karana_number - 1) % 11) as usize).unwrap().name.clone()
                    } else {
                        "Unknown".to_string()
                    }
                }),
            start_time: Some(DateTime::from_timestamp(((jd - 2440588.0) * 86400.0) as i64, 0)
                .unwrap_or(Utc::now())),
            end_time: Some(end_time),
        })
    }

    /// Calculate Vara (weekday) from Julian Day
    pub fn calculate_vara(&self, jd: f64) -> Result<Vara, PanchangaError> {
        // Julian Day 0 was a Monday (weekday 1)
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

    /// Calculate when a tithi ends using iterative method
    fn calculate_tithi_end_time(
        &self,
        jd: f64,
        solar_longitude: f64,
        lunar_longitude: f64,
        target_longitude: f64,
    ) -> Result<DateTime<Utc>, PanchangaError> {
        // Simplified calculation - in practice, this would use iterative refinement
        // with proper solar and lunar position calculations
        let current_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        let target_diff = target_longitude.rem_euclid(360.0);
        
        let diff_degrees = (target_diff - current_diff).rem_euclid(360.0);
        
        // Approximate time based on lunar motion (about 13.2 degrees per day)
        let days_to_end = diff_degrees / 13.2;
        let end_jd = jd + days_to_end;
        
        DateTime::from_timestamp(((end_jd - 2440588.0) * 86400.0) as i64, 0)
            .ok_or_else(|| PanchangaError::DateError("Invalid timestamp".to_string()))
    }

    /// Calculate when a nakshatra ends
    fn calculate_nakshatra_end_time(
        &self,
        jd: f64,
        lunar_longitude: f64,
        target_longitude: f64,
    ) -> Result<DateTime<Utc>, PanchangaError> {
        let diff_degrees = (target_longitude - lunar_longitude).rem_euclid(360.0);
        let days_to_end = diff_degrees / 13.2; // Lunar motion
        let end_jd = jd + days_to_end;
        
        DateTime::from_timestamp(((end_jd - 2440588.0) * 86400.0) as i64, 0)
            .ok_or_else(|| PanchangaError::DateError("Invalid timestamp".to_string()))
    }

    /// Calculate when a yoga ends
    fn calculate_yoga_end_time(
        &self,
        jd: f64,
        solar_longitude: f64,
        lunar_longitude: f64,
        target_longitude: f64,
    ) -> Result<DateTime<Utc>, PanchangaError> {
        let current_sum = (solar_longitude + lunar_longitude).rem_euclid(360.0);
        let diff_degrees = (target_longitude - current_sum).rem_euclid(360.0);
        
        // Combined motion of Sun and Moon (about 0.9856 + 13.2 = 14.1856 degrees per day)
        let days_to_end = diff_degrees / 14.1856;
        let end_jd = jd + days_to_end;
        
        DateTime::from_timestamp(((end_jd - 2440588.0) * 86400.0) as i64, 0)
            .ok_or_else(|| PanchangaError::DateError("Invalid timestamp".to_string()))
    }

    /// Calculate when a karana ends
    fn calculate_karana_end_time(
        &self,
        jd: f64,
        solar_longitude: f64,
        lunar_longitude: f64,
        target_longitude: f64,
    ) -> Result<DateTime<Utc>, PanchangaError> {
        let current_diff = (lunar_longitude - solar_longitude).rem_euclid(360.0);
        let diff_degrees = (target_longitude - current_diff).rem_euclid(360.0);
        
        // Same as tithi but with 6-degree intervals
        let days_to_end = diff_degrees / 13.2;
        let end_jd = jd + days_to_end;
        
        DateTime::from_timestamp(((end_jd - 2440588.0) * 86400.0) as i64, 0)
            .ok_or_else(|| PanchangaError::DateError("Invalid timestamp".to_string()))
    }

    /// Initialize Nakshatra data
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

    /// Initialize Yoga data
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

    /// Initialize Karana data
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

    /// Initialize Tithi names
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

    /// Initialize Vara (weekday) names
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
