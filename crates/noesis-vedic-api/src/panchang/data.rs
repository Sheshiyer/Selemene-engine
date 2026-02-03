//! Core Panchang data types and structures
//!
//! This module defines all the data structures needed for comprehensive
//! Panchang calculations including Tithi, Nakshatra, Yoga, Karana, Vara,
//! and related information.

use serde::{Deserialize, Serialize};
use crate::error::VedicApiResult;

/// Complete Panchang data for a specific date/time/location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panchang {
    /// Date information
    pub date: DateInfo,
    /// Location information
    pub location: Location,
    /// Tithi (lunar day)
    pub tithi: Tithi,
    /// Nakshatra (lunar mansion)
    pub nakshatra: Nakshatra,
    /// Yoga (lunar-solar combination)
    pub yoga: Yoga,
    /// Karana (half-tithi)
    pub karana: Karana,
    /// Vara (day of week)
    pub vara: Vara,
    /// Paksha (lunar fortnight)
    pub paksha: Paksha,
    /// Sun/Moon positions and data
    pub planets: PlanetaryPositions,
    /// Sunrise and sunset times
    pub day_boundaries: DayBoundaries,
    /// Ayanamsa value
    pub ayanamsa: f64,
}

impl Panchang {
    /// Check if the current time is auspicious for starting new activities
    pub fn is_auspicious(&self) -> bool {
        let mut score = 0;
        
        // Check Tithi
        if self.tithi.is_auspicious() {
            score += 1;
        }
        
        // Check Nakshatra
        if self.nakshatra.is_auspicious() {
            score += 1;
        }
        
        // Check Yoga
        if self.yoga.is_auspicious() {
            score += 1;
        }
        
        // Check Karana
        if self.karana.is_auspicious() {
            score += 1;
        }
        
        // Check Vara
        if self.vara.is_auspicious() {
            score += 1;
        }
        
        score >= 3
    }
    
    /// Get a summary string for the Panchang
    pub fn summary(&self) -> String {
        format!(
            "{} {}, {} Nakshatra, {} Yoga, {} Karana",
            self.paksha.as_str(),
            self.tithi.name(),
            self.nakshatra.name(),
            self.yoga.name(),
            self.karana.name()
        )
    }
    
    /// Get the ruling planets for the day
    pub fn ruling_planets(&self) -> Vec<&'static str> {
        let mut rulers = vec![];
        
        // Vara lord
        rulers.push(self.vara.ruling_planet());
        
        // Nakshatra lord
        rulers.push(self.nakshatra.ruling_planet());
        
        // Tithi lord (approximate)
        rulers.push(self.tithi.ruling_planet());
        
        rulers
    }
}

/// Date information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateInfo {
    /// Gregorian date components
    pub year: i32,
    pub month: u32,
    pub day: u32,
    /// Day of week (1=Monday, 7=Sunday)
    pub day_of_week: u8,
    /// Julian day number
    pub julian_day: f64,
    /// Hindu lunar date (if available)
    pub hindu_date: Option<HinduDate>,
}

/// Hindu lunar date (Purnimanta system - month ends on full moon)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HinduDate {
    pub year: i32,
    pub month: HinduMonth,
    pub tithi: u8,
    pub paksha: Paksha,
}

/// Hindu lunar months
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HinduMonth {
    Chaitra,
    Vaishakha,
    Jyeshtha,
    Ashadha,
    Shravana,
    Bhadrapada,
    Ashvina,
    Kartika,
    Margashirsha,
    Pausha,
    Magha,
    Phalguna,
}

impl HinduMonth {
    pub fn as_str(&self) -> &'static str {
        match self {
            HinduMonth::Chaitra => "Chaitra",
            HinduMonth::Vaishakha => "Vaishakha",
            HinduMonth::Jyeshtha => "Jyeshtha",
            HinduMonth::Ashadha => "Ashadha",
            HinduMonth::Shravana => "Shravana",
            HinduMonth::Bhadrapada => "Bhadrapada",
            HinduMonth::Ashvina => "Ashvina",
            HinduMonth::Kartika => "Kartika",
            HinduMonth::Margashirsha => "Margashirsha",
            HinduMonth::Pausha => "Pausha",
            HinduMonth::Magha => "Magha",
            HinduMonth::Phalguna => "Phalguna",
        }
    }
    
    pub fn number(&self) -> u8 {
        match self {
            HinduMonth::Chaitra => 1,
            HinduMonth::Vaishakha => 2,
            HinduMonth::Jyeshtha => 3,
            HinduMonth::Ashadha => 4,
            HinduMonth::Shravana => 5,
            HinduMonth::Bhadrapada => 6,
            HinduMonth::Ashvina => 7,
            HinduMonth::Kartika => 8,
            HinduMonth::Margashirsha => 9,
            HinduMonth::Pausha => 10,
            HinduMonth::Magha => 11,
            HinduMonth::Phalguna => 12,
        }
    }
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    pub name: Option<String>,
}

/// Tithi (lunar day) - 30 tithis in a lunar month
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tithi {
    /// Tithi number (1-30)
    pub number: u8,
    /// Tithi name
    pub name_tithi: TithiName,
    /// Start time (when this tithi began)
    pub start_time: String,
    /// End time (when this tithi ends)
    pub end_time: String,
    /// Whether this is a complete tithi or intercepted
    pub is_complete: bool,
}

impl Tithi {
    /// Get the tithi name as a string
    pub fn name(&self) -> &'static str {
        self.name_tithi.as_str()
    }
    
    /// Check if this tithi is auspicious
    pub fn is_auspicious(&self) -> bool {
        use TithiName::*;
        matches!(
            self.name_tithi,
            Pratipada | Dwitiya | Tritiya | Panchami |
            Saptami | Dashami | Ekadashi | Trayodashi |
            Purnima | Amavasya
        )
    }
    
    /// Get ruling planet of the tithi
    pub fn ruling_planet(&self) -> &'static str {
        match (self.number - 1) % 7 {
            0 => "Sun",     // 1, 8, 15, 22
            1 => "Moon",    // 2, 9, 16, 23
            2 => "Mars",    // 3, 10, 17, 24
            3 => "Mercury", // 4, 11, 18, 25
            4 => "Jupiter", // 5, 12, 19, 26
            5 => "Venus",   // 6, 13, 20, 27
            _ => "Saturn",  // 7, 14, 21, 28
        }
    }
}

/// Names of the 15 Tithis (each appears twice in a month - Shukla and Krishna)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TithiName {
    Pratipada,   // 1st
    Dwitiya,     // 2nd
    Tritiya,     // 3rd
    Chaturthi,   // 4th
    Panchami,    // 5th
    Shashthi,    // 6th
    Saptami,     // 7th
    Ashtami,     // 8th
    Navami,      // 9th
    Dashami,     // 10th
    Ekadashi,    // 11th
    Dwadashi,    // 12th
    Trayodashi,  // 13th
    Chaturdashi, // 14th
    Purnima,     // Full Moon (15th of Shukla)
    Amavasya,    // New Moon (15th of Krishna)
}

impl TithiName {
    pub fn as_str(&self) -> &'static str {
        match self {
            TithiName::Pratipada => "Pratipada",
            TithiName::Dwitiya => "Dwitiya",
            TithiName::Tritiya => "Tritiya",
            TithiName::Chaturthi => "Chaturthi",
            TithiName::Panchami => "Panchami",
            TithiName::Shashthi => "Shashthi",
            TithiName::Saptami => "Saptami",
            TithiName::Ashtami => "Ashtami",
            TithiName::Navami => "Navami",
            TithiName::Dashami => "Dashami",
            TithiName::Ekadashi => "Ekadashi",
            TithiName::Dwadashi => "Dwadashi",
            TithiName::Trayodashi => "Trayodashi",
            TithiName::Chaturdashi => "Chaturdashi",
            TithiName::Purnima => "Purnima",
            TithiName::Amavasya => "Amavasya",
        }
    }
    
    /// Get the number (1-15)
    pub fn number(&self) -> u8 {
        match self {
            TithiName::Pratipada => 1,
            TithiName::Dwitiya => 2,
            TithiName::Tritiya => 3,
            TithiName::Chaturthi => 4,
            TithiName::Panchami => 5,
            TithiName::Shashthi => 6,
            TithiName::Saptami => 7,
            TithiName::Ashtami => 8,
            TithiName::Navami => 9,
            TithiName::Dashami => 10,
            TithiName::Ekadashi => 11,
            TithiName::Dwadashi => 12,
            TithiName::Trayodashi => 13,
            TithiName::Chaturdashi => 14,
            TithiName::Purnima => 15,
            TithiName::Amavasya => 15,
        }
    }

    /// Get TithiName from number (1-30). Numbers 1-15 map to Shukla, 16-30 to Krishna equivalents.
    pub fn from_number(n: u32) -> Self {
        match ((n - 1) % 15) + 1 {
            1 => TithiName::Pratipada,
            2 => TithiName::Dwitiya,
            3 => TithiName::Tritiya,
            4 => TithiName::Chaturthi,
            5 => TithiName::Panchami,
            6 => TithiName::Shashthi,
            7 => TithiName::Saptami,
            8 => TithiName::Ashtami,
            9 => TithiName::Navami,
            10 => TithiName::Dashami,
            11 => TithiName::Ekadashi,
            12 => TithiName::Dwadashi,
            13 => TithiName::Trayodashi,
            14 => TithiName::Chaturdashi,
            15 if n <= 15 => TithiName::Purnima,
            _ => TithiName::Amavasya,
        }
    }
}

/// Nakshatra (lunar mansion) - 27 nakshatras in the zodiac
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nakshatra {
    /// Nakshatra number (1-27)
    pub number: u8,
    /// Nakshatra name
    pub name_nakshatra: NakshatraName,
    /// Pada (quarter) - 1-4
    pub pada: u8,
    /// Start time
    pub start_time: String,
    /// End time
    pub end_time: String,
    /// Longitude
    pub longitude: f64,
}

impl Nakshatra {
    pub fn name(&self) -> &'static str {
        self.name_nakshatra.as_str()
    }
    
    /// Check if this nakshatra is auspicious
    pub fn is_auspicious(&self) -> bool {
        use NakshatraName::*;
        matches!(
            self.name_nakshatra,
            Ashwini | Rohini | Mrigashira | Pushya |
            UttaraPhalguni | Hasta | Chitra | Swati |
            Anuradha | Mula | UttaraAshadha | Shravana |
            Dhanishta | Shatabhisha | UttaraBhadrapada | Revati
        )
    }
    
    /// Get ruling planet
    pub fn ruling_planet(&self) -> &'static str {
        self.name_nakshatra.ruler()
    }
    
    /// Get deity
    pub fn deity(&self) -> &'static str {
        self.name_nakshatra.deity()
    }
}

/// Names of the 27 Nakshatras
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NakshatraName {
    Ashwini,
    Bharani,
    Krittika,
    Rohini,
    Mrigashira,
    Ardra,
    Punarvasu,
    Pushya,
    Ashlesha,
    Magha,
    PurvaPhalguni,
    UttaraPhalguni,
    Hasta,
    Chitra,
    Swati,
    Vishakha,
    Anuradha,
    Jyeshtha,
    Mula,
    PurvaAshadha,
    UttaraAshadha,
    Shravana,
    Dhanishta,
    Shatabhisha,
    PurvaBhadrapada,
    UttaraBhadrapada,
    Revati,
}

impl NakshatraName {
    pub fn as_str(&self) -> &'static str {
        match self {
            NakshatraName::Ashwini => "Ashwini",
            NakshatraName::Bharani => "Bharani",
            NakshatraName::Krittika => "Krittika",
            NakshatraName::Rohini => "Rohini",
            NakshatraName::Mrigashira => "Mrigashira",
            NakshatraName::Ardra => "Ardra",
            NakshatraName::Punarvasu => "Punarvasu",
            NakshatraName::Pushya => "Pushya",
            NakshatraName::Ashlesha => "Ashlesha",
            NakshatraName::Magha => "Magha",
            NakshatraName::PurvaPhalguni => "Purva Phalguni",
            NakshatraName::UttaraPhalguni => "Uttara Phalguni",
            NakshatraName::Hasta => "Hasta",
            NakshatraName::Chitra => "Chitra",
            NakshatraName::Swati => "Swati",
            NakshatraName::Vishakha => "Vishakha",
            NakshatraName::Anuradha => "Anuradha",
            NakshatraName::Jyeshtha => "Jyeshtha",
            NakshatraName::Mula => "Mula",
            NakshatraName::PurvaAshadha => "Purva Ashadha",
            NakshatraName::UttaraAshadha => "Uttara Ashadha",
            NakshatraName::Shravana => "Shravana",
            NakshatraName::Dhanishta => "Dhanishta",
            NakshatraName::Shatabhisha => "Shatabhisha",
            NakshatraName::PurvaBhadrapada => "Purva Bhadrapada",
            NakshatraName::UttaraBhadrapada => "Uttara Bhadrapada",
            NakshatraName::Revati => "Revati",
        }
    }
    
    /// Get ruling planet (Vimshottari dasha lord)
    pub fn ruler(&self) -> &'static str {
        match self {
            NakshatraName::Krittika | NakshatraName::UttaraPhalguni | NakshatraName::UttaraAshadha => "Sun",
            NakshatraName::Rohini | NakshatraName::Hasta | NakshatraName::Shravana => "Moon",
            NakshatraName::Mrigashira | NakshatraName::Chitra | NakshatraName::Dhanishta => "Mars",
            NakshatraName::Ashlesha | NakshatraName::Jyeshtha | NakshatraName::Revati => "Mercury",
            NakshatraName::Punarvasu | NakshatraName::Vishakha | NakshatraName::PurvaBhadrapada => "Jupiter",
            NakshatraName::Bharani | NakshatraName::PurvaPhalguni | NakshatraName::PurvaAshadha => "Venus",
            NakshatraName::Pushya | NakshatraName::Anuradha | NakshatraName::UttaraBhadrapada => "Saturn",
            NakshatraName::Ashwini | NakshatraName::Magha | NakshatraName::Mula => "Ketu",
            NakshatraName::Ardra | NakshatraName::Swati | NakshatraName::Shatabhisha => "Rahu",
        }
    }
    
    /// Get NakshatraName from number (1-27)
    pub fn from_number(n: u32) -> Self {
        match n.min(27) {
            1 => NakshatraName::Ashwini,
            2 => NakshatraName::Bharani,
            3 => NakshatraName::Krittika,
            4 => NakshatraName::Rohini,
            5 => NakshatraName::Mrigashira,
            6 => NakshatraName::Ardra,
            7 => NakshatraName::Punarvasu,
            8 => NakshatraName::Pushya,
            9 => NakshatraName::Ashlesha,
            10 => NakshatraName::Magha,
            11 => NakshatraName::PurvaPhalguni,
            12 => NakshatraName::UttaraPhalguni,
            13 => NakshatraName::Hasta,
            14 => NakshatraName::Chitra,
            15 => NakshatraName::Swati,
            16 => NakshatraName::Vishakha,
            17 => NakshatraName::Anuradha,
            18 => NakshatraName::Jyeshtha,
            19 => NakshatraName::Mula,
            20 => NakshatraName::PurvaAshadha,
            21 => NakshatraName::UttaraAshadha,
            22 => NakshatraName::Shravana,
            23 => NakshatraName::Dhanishta,
            24 => NakshatraName::Shatabhisha,
            25 => NakshatraName::PurvaBhadrapada,
            26 => NakshatraName::UttaraBhadrapada,
            _ => NakshatraName::Revati,
        }
    }

    /// Get ruling deity
    pub fn deity(&self) -> &'static str {
        match self {
            NakshatraName::Ashwini => "Ashwini Kumaras",
            NakshatraName::Bharani => "Yama",
            NakshatraName::Krittika => "Agni",
            NakshatraName::Rohini => "Prajapati",
            NakshatraName::Mrigashira => "Soma",
            NakshatraName::Ardra => "Rudra",
            NakshatraName::Punarvasu => "Aditi",
            NakshatraName::Pushya => "Brihaspati",
            NakshatraName::Ashlesha => "Nagas",
            NakshatraName::Magha => "Pitris",
            NakshatraName::PurvaPhalguni => "Bhaga",
            NakshatraName::UttaraPhalguni => "Aryaman",
            NakshatraName::Hasta => "Savitar",
            NakshatraName::Chitra => "Tvashtar",
            NakshatraName::Swati => "Vayu",
            NakshatraName::Vishakha => "Indra-Agni",
            NakshatraName::Anuradha => "Mitra",
            NakshatraName::Jyeshtha => "Indra",
            NakshatraName::Mula => "Nirriti",
            NakshatraName::PurvaAshadha => "Apah",
            NakshatraName::UttaraAshadha => "Vishvadevas",
            NakshatraName::Shravana => "Vishnu",
            NakshatraName::Dhanishta => "Vasus",
            NakshatraName::Shatabhisha => "Varuna",
            NakshatraName::PurvaBhadrapada => "Ajaikapada",
            NakshatraName::UttaraBhadrapada => "Ahirbudhnya",
            NakshatraName::Revati => "Pushan",
        }
    }
}

/// Yoga (lunar-solar combination) - 27 yogas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Yoga {
    /// Yoga number (1-27)
    pub number: u8,
    /// Yoga name
    pub name_yoga: YogaName,
    /// Start time
    pub start_time: String,
    /// End time
    pub end_time: String,
}

impl Yoga {
    pub fn name(&self) -> &'static str {
        self.name_yoga.as_str()
    }
    
    pub fn is_auspicious(&self) -> bool {
        use YogaName::*;
        matches!(
            self.name_yoga,
            Preeti | Ayushman | Saubhagya | Shobhana | Sukarma |
            Dhriti | Vriddhi | Harshana | Siddhi | Vyatipata |
            Variyan | Shiva | Siddha | Sadhya | Shubha | Shukla |
            Brahma | Indra
        )
    }
    
    /// Get nature of the yoga
    pub fn nature(&self) -> &'static str {
        self.name_yoga.nature()
    }
}

/// Names of the 27 Yogas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YogaName {
    Vishkumbha,
    Preeti,
    Ayushman,
    Saubhagya,
    Shobhana,
    Atiganda,
    Sukarma,
    Dhriti,
    Shoola,
    Ganda,
    Vriddhi,
    Dhruva,
    Vyaghaata,
    Harshana,
    Vajra,
    Siddhi,
    Vyatipata,
    Variyan,
    Parigha,
    Shiva,
    Siddha,
    Sadhya,
    Shubha,
    Shukla,
    Brahma,
    Indra,
    Vaidhriti,
}

impl YogaName {
    pub fn as_str(&self) -> &'static str {
        match self {
            YogaName::Vishkumbha => "Vishkumbha",
            YogaName::Preeti => "Preeti",
            YogaName::Ayushman => "Ayushman",
            YogaName::Saubhagya => "Saubhagya",
            YogaName::Shobhana => "Shobhana",
            YogaName::Atiganda => "Atiganda",
            YogaName::Sukarma => "Sukarma",
            YogaName::Dhriti => "Dhriti",
            YogaName::Shoola => "Shoola",
            YogaName::Ganda => "Ganda",
            YogaName::Vriddhi => "Vriddhi",
            YogaName::Dhruva => "Dhruva",
            YogaName::Vyaghaata => "Vyaghaata",
            YogaName::Harshana => "Harshana",
            YogaName::Vajra => "Vajra",
            YogaName::Siddhi => "Siddhi",
            YogaName::Vyatipata => "Vyatipata",
            YogaName::Variyan => "Variyan",
            YogaName::Parigha => "Parigha",
            YogaName::Shiva => "Shiva",
            YogaName::Siddha => "Siddha",
            YogaName::Sadhya => "Sadhya",
            YogaName::Shubha => "Shubha",
            YogaName::Shukla => "Shukla",
            YogaName::Brahma => "Brahma",
            YogaName::Indra => "Indra",
            YogaName::Vaidhriti => "Vaidhriti",
        }
    }
    
    /// Get YogaName from number (1-27)
    pub fn from_number(n: u32) -> Self {
        match n.min(27) {
            1 => YogaName::Vishkumbha,
            2 => YogaName::Preeti,
            3 => YogaName::Ayushman,
            4 => YogaName::Saubhagya,
            5 => YogaName::Shobhana,
            6 => YogaName::Atiganda,
            7 => YogaName::Sukarma,
            8 => YogaName::Dhriti,
            9 => YogaName::Shoola,
            10 => YogaName::Ganda,
            11 => YogaName::Vriddhi,
            12 => YogaName::Dhruva,
            13 => YogaName::Vyaghaata,
            14 => YogaName::Harshana,
            15 => YogaName::Vajra,
            16 => YogaName::Siddhi,
            17 => YogaName::Vyatipata,
            18 => YogaName::Variyan,
            19 => YogaName::Parigha,
            20 => YogaName::Shiva,
            21 => YogaName::Siddha,
            22 => YogaName::Sadhya,
            23 => YogaName::Shubha,
            24 => YogaName::Shukla,
            25 => YogaName::Brahma,
            26 => YogaName::Indra,
            _ => YogaName::Vaidhriti,
        }
    }

    /// Get nature: good, mixed, or bad
    pub fn nature(&self) -> &'static str {
        match self {
            // Good yogas
            YogaName::Preeti | YogaName::Ayushman | YogaName::Saubhagya |
            YogaName::Shobhana | YogaName::Sukarma | YogaName::Dhriti |
            YogaName::Vriddhi | YogaName::Harshana | YogaName::Siddhi |
            YogaName::Variyan | YogaName::Shiva | YogaName::Siddha |
            YogaName::Sadhya | YogaName::Shubha | YogaName::Shukla |
            YogaName::Brahma | YogaName::Indra => "auspicious",
            
            // Mixed
            YogaName::Dhruva => "mixed",
            
            // Bad yogas
            YogaName::Vishkumbha | YogaName::Atiganda | YogaName::Shoola |
            YogaName::Ganda | YogaName::Vyaghaata | YogaName::Vajra |
            YogaName::Vyatipata | YogaName::Parigha | YogaName::Vaidhriti => "inauspicious",
        }
    }
}

/// Karana (half-tithi) - 11 karana types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Karana {
    /// Karana name
    pub name_karana: KaranaName,
    /// Karana type
    pub karana_type: KaranaType,
    /// Start time
    pub start_time: String,
    /// End time
    pub end_time: String,
}

impl Karana {
    pub fn name(&self) -> &'static str {
        self.name_karana.as_str()
    }
    
    pub fn is_auspicious(&self) -> bool {
        use KaranaType::*;
        matches!(self.karana_type, Fixed | Movable)
    }
}

/// Names of the 11 Karanas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KaranaName {
    Bava,
    Balava,
    Kaulava,
    Taitila,
    Gara,
    Vanija,
    Vishti,
    Shakuni,
    Chatushpada,
    Naga,
    Kimstughna,
}

impl KaranaName {
    /// Get KaranaName from number (1-11, cyclically)
    pub fn from_number(n: u32) -> Self {
        match ((n - 1) % 11) + 1 {
            1 => KaranaName::Bava,
            2 => KaranaName::Balava,
            3 => KaranaName::Kaulava,
            4 => KaranaName::Taitila,
            5 => KaranaName::Gara,
            6 => KaranaName::Vanija,
            7 => KaranaName::Vishti,
            8 => KaranaName::Shakuni,
            9 => KaranaName::Chatushpada,
            10 => KaranaName::Naga,
            _ => KaranaName::Kimstughna,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            KaranaName::Bava => "Bava",
            KaranaName::Balava => "Balava",
            KaranaName::Kaulava => "Kaulava",
            KaranaName::Taitila => "Taitila",
            KaranaName::Gara => "Gara",
            KaranaName::Vanija => "Vanija",
            KaranaName::Vishti => "Vishti",
            KaranaName::Shakuni => "Shakuni",
            KaranaName::Chatushpada => "Chatushpada",
            KaranaName::Naga => "Naga",
            KaranaName::Kimstughna => "Kimstughna",
        }
    }
}

/// Type of Karana
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KaranaType {
    /// Bava, Balava, Kaulava, Taitila, Gara, Vanija - good for most activities
    Movable,
    /// Shakuni, Chatushpada, Naga, Kimstughna - only good during night
    Fixed,
    /// Vishti (Bhadra) - generally inauspicious
    Vishti,
}

impl KaranaType {
    pub fn is_auspicious(&self) -> bool {
        !matches!(self, KaranaType::Vishti)
    }
}

/// Vara (day of the week)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vara {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Vara {
    pub fn as_str(&self) -> &'static str {
        match self {
            Vara::Sunday => "Sunday",
            Vara::Monday => "Monday",
            Vara::Tuesday => "Tuesday",
            Vara::Wednesday => "Wednesday",
            Vara::Thursday => "Thursday",
            Vara::Friday => "Friday",
            Vara::Saturday => "Saturday",
        }
    }
    
    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Vara::Monday),
            2 => Some(Vara::Tuesday),
            3 => Some(Vara::Wednesday),
            4 => Some(Vara::Thursday),
            5 => Some(Vara::Friday),
            6 => Some(Vara::Saturday),
            7 => Some(Vara::Sunday),
            _ => None,
        }
    }
    
    /// Get ruling planet
    pub fn ruling_planet(&self) -> &'static str {
        match self {
            Vara::Sunday => "Sun",
            Vara::Monday => "Moon",
            Vara::Tuesday => "Mars",
            Vara::Wednesday => "Mercury",
            Vara::Thursday => "Jupiter",
            Vara::Friday => "Venus",
            Vara::Saturday => "Saturn",
        }
    }
    
    /// Check if this day is generally auspicious
    pub fn is_auspicious(&self) -> bool {
        !matches!(self, Vara::Saturday | Vara::Tuesday)
    }
    
    /// Get day number (1=Monday, 7=Sunday)
    pub fn number(&self) -> u8 {
        match self {
            Vara::Monday => 1,
            Vara::Tuesday => 2,
            Vara::Wednesday => 3,
            Vara::Thursday => 4,
            Vara::Friday => 5,
            Vara::Saturday => 6,
            Vara::Sunday => 7,
        }
    }
}

/// Paksha (lunar fortnight)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Paksha {
    /// Waxing moon - first fortnight (Pratipada to Purnima)
    Shukla,
    /// Waning moon - second fortnight (Pratipada to Amavasya)
    Krishna,
}

impl Paksha {
    pub fn as_str(&self) -> &'static str {
        match self {
            Paksha::Shukla => "Shukla",
            Paksha::Krishna => "Krishna",
        }
    }
    
    /// Get English name
    pub fn english(&self) -> &'static str {
        match self {
            Paksha::Shukla => "Waxing",
            Paksha::Krishna => "Waning",
        }
    }
}

/// Planetary positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryPositions {
    pub sun: PlanetPosition,
    pub moon: PlanetPosition,
    pub mars: Option<PlanetPosition>,
    pub mercury: Option<PlanetPosition>,
    pub jupiter: Option<PlanetPosition>,
    pub venus: Option<PlanetPosition>,
    pub saturn: Option<PlanetPosition>,
    pub rahu: Option<PlanetPosition>,
    pub ketu: Option<PlanetPosition>,
}

/// Single planet position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetPosition {
    pub name: String,
    pub longitude: f64,
    pub latitude: f64,
    pub speed: f64,
    pub sign: String,
    pub nakshatra: String,
    pub pada: u8,
    pub is_retrograde: bool,
}

/// Sunrise and sunset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayBoundaries {
    pub sunrise: String,
    pub sunset: String,
    pub next_sunrise: String,
    pub day_duration: String,  // HH:MM format
    pub night_duration: String, // HH:MM format
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tithi_name() {
        assert_eq!(TithiName::Purnima.as_str(), "Purnima");
        assert_eq!(TithiName::Ekadashi.number(), 11);
    }

    #[test]
    fn test_nakshatra() {
        assert_eq!(NakshatraName::Rohini.ruler(), "Moon");
        assert_eq!(NakshatraName::Ashwini.deity(), "Ashwini Kumaras");
    }

    #[test]
    fn test_vara() {
        assert_eq!(Vara::Tuesday.ruling_planet(), "Mars");
        assert_eq!(Vara::Sunday.number(), 7);
        assert_eq!(Vara::from_number(1), Some(Vara::Monday));
    }

    #[test]
    fn test_paksha() {
        assert_eq!(Paksha::Shukla.english(), "Waxing");
        assert_eq!(Paksha::Krishna.as_str(), "Krishna");
    }

    #[test]
    fn test_yoga_nature() {
        assert_eq!(YogaName::Shiva.nature(), "auspicious");
        assert_eq!(YogaName::Vishkumbha.nature(), "inauspicious");
    }
}
