//! Birth chart (Rashi) and Varga (divisional chart) types

use serde::{Deserialize, Serialize};

/// Main birth chart (D1 - Rashi)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthChart {
    /// Native details
    pub native: NativeInfo,
    /// Ayanamsa used
    pub ayanamsa: f64,
    /// House system
    pub house_system: String,
    /// Planetary positions
    pub planets: Vec<PlanetPosition>,
    /// House cusps
    pub houses: Vec<HousePosition>,
    /// Ascendant details
    pub ascendant: AscendantInfo,
    /// Moon details
    pub moon: MoonInfo,
    /// Special points
    pub special_points: SpecialPoints,
    /// Chart notes
    pub notes: Vec<String>,
}

impl BirthChart {
    /// Get planet by name
    pub fn get_planet(&self, name: &str) -> Option<&PlanetPosition> {
        self.planets.iter().find(|p| p.name.eq_ignore_ascii_case(name))
    }
    
    /// Get planets in a house
    pub fn planets_in_house(&self, house: u8) -> Vec<&PlanetPosition> {
        self.planets.iter()
            .filter(|p| p.house == house)
            .collect()
    }
    
    /// Get planets in a sign
    pub fn planets_in_sign(&self, sign: ZodiacSign) -> Vec<&PlanetPosition> {
        self.planets.iter()
            .filter(|p| p.sign == sign)
            .collect()
    }
    
    /// Get ascendant lord
    pub fn ascendant_lord(&self) -> Option<&PlanetPosition> {
        let asc_sign = self.ascendant.sign;
        let lord = asc_sign.ruler();
        self.get_planet(lord)
    }
    
    /// Get 9th lord (Bhagya/Dharma)
    pub fn ninth_lord(&self) -> Option<&PlanetPosition> {
        let ninth_sign = ZodiacSign::from_index((self.ascendant.sign.index() + 8) % 12);
        let lord = ninth_sign.ruler();
        self.get_planet(lord)
    }
    
    /// Get 10th lord (Karma/Career)
    pub fn tenth_lord(&self) -> Option<&PlanetPosition> {
        let tenth_sign = ZodiacSign::from_index((self.ascendant.sign.index() + 9) % 12);
        let lord = tenth_sign.ruler();
        self.get_planet(lord)
    }
}

/// Native (person) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeInfo {
    pub birth_date: String,
    pub birth_time: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
}

/// Planet position in the chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetPosition {
    /// Planet name
    pub name: String,
    /// Longitude (0-360°)
    pub longitude: f64,
    /// Zodiac sign
    pub sign: ZodiacSign,
    /// Degree within sign (0-30°)
    pub degree: f64,
    /// Minutes within degree
    pub minutes: f64,
    /// House placement (1-12)
    pub house: u8,
    /// Retrograde status
    pub is_retrograde: bool,
    /// Combust status (too close to Sun)
    pub is_combust: bool,
    /// Nakshatra
    pub nakshatra: String,
    /// Nakshatra pada (1-4)
    pub pada: u8,
    /// Speed (degrees per day)
    pub speed: f64,
    /// Latitude (north/south)
    pub latitude: f64,
}

impl PlanetPosition {
    /// Get full longitude (sign start + degree)
    pub fn full_longitude(&self) -> f64 {
        (self.sign.index() as f64 * 30.0) + self.degree
    }
    
    /// Check if planet is in own sign
    pub fn in_own_sign(&self) -> bool {
        let ruler = self.sign.ruler();
        self.name.eq_ignore_ascii_case(ruler)
    }
    
    /// Check if planet is exalted
    pub fn is_exalted(&self) -> bool {
        let exaltation_degree = match self.name.to_lowercase().as_str() {
            "sun" => (ZodiacSign::Aries, 10.0),
            "moon" => (ZodiacSign::Taurus, 3.0),
            "mars" => (ZodiacSign::Capricorn, 28.0),
            "mercury" => (ZodiacSign::Virgo, 15.0),
            "jupiter" => (ZodiacSign::Cancer, 5.0),
            "venus" => (ZodiacSign::Pisces, 27.0),
            "saturn" => (ZodiacSign::Libra, 20.0),
            _ => return false,
        };
        
        self.sign == exaltation_degree.0 && 
        (self.degree - exaltation_degree.1).abs() < 5.0 // Within 5° of exaltation point
    }
    
    /// Check if planet is debilitated
    pub fn is_debilitated(&self) -> bool {
        let debilitation_sign = match self.name.to_lowercase().as_str() {
            "sun" => ZodiacSign::Libra,
            "moon" => ZodiacSign::Scorpio,
            "mars" => ZodiacSign::Cancer,
            "mercury" => ZodiacSign::Pisces,
            "jupiter" => ZodiacSign::Capricorn,
            "venus" => ZodiacSign::Virgo,
            "saturn" => ZodiacSign::Aries,
            _ => return false,
        };
        
        self.sign == debilitation_sign
    }
    
    /// Check if in friend's sign
    pub fn in_friend_sign(&self, friends: &[&str]) -> bool {
        let ruler = self.sign.ruler();
        friends.iter().any(|f| f.eq_ignore_ascii_case(ruler))
    }
}

/// House position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HousePosition {
    /// House number (1-12)
    pub number: u8,
    /// Sign on cusp
    pub sign: ZodiacSign,
    /// Cusp longitude
    pub cusp: f64,
    /// House degree (0-30°)
    pub degree: f64,
    /// House nature: Dharma, Artha, Kama, or Moksha
    pub house_type: HouseType,
    /// Is angular (Kendra)
    pub is_kendra: bool,
    /// Is succedent (Panapara)
    pub is_panapara: bool,
    /// Is cadent (Apoklima)
    pub is_apoklima: bool,
}

/// House types (Purusharthas)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HouseType {
    Dharma, // 1, 5, 9 - Purpose/duty
    Artha,  // 2, 6, 10 - Wealth/work
    Kama,   // 3, 7, 11 - Desire/relationships
    Moksha, // 4, 8, 12 - Liberation/spirituality
}

impl HousePosition {
    /// Get houses of same type
    pub fn same_type(&self) -> Vec<u8> {
        match self.house_type {
            HouseType::Dharma => vec![1, 5, 9],
            HouseType::Artha => vec![2, 6, 10],
            HouseType::Kama => vec![3, 7, 11],
            HouseType::Moksha => vec![4, 8, 12],
        }
    }
}

/// Zodiac signs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl ZodiacSign {
    /// Get sign index (0-11)
    pub fn index(&self) -> usize {
        match self {
            ZodiacSign::Aries => 0,
            ZodiacSign::Taurus => 1,
            ZodiacSign::Gemini => 2,
            ZodiacSign::Cancer => 3,
            ZodiacSign::Leo => 4,
            ZodiacSign::Virgo => 5,
            ZodiacSign::Libra => 6,
            ZodiacSign::Scorpio => 7,
            ZodiacSign::Sagittarius => 8,
            ZodiacSign::Capricorn => 9,
            ZodiacSign::Aquarius => 10,
            ZodiacSign::Pisces => 11,
        }
    }
    
    /// Create from index
    pub fn from_index(idx: usize) -> Self {
        match idx % 12 {
            0 => ZodiacSign::Aries,
            1 => ZodiacSign::Taurus,
            2 => ZodiacSign::Gemini,
            3 => ZodiacSign::Cancer,
            4 => ZodiacSign::Leo,
            5 => ZodiacSign::Virgo,
            6 => ZodiacSign::Libra,
            7 => ZodiacSign::Scorpio,
            8 => ZodiacSign::Sagittarius,
            9 => ZodiacSign::Capricorn,
            10 => ZodiacSign::Aquarius,
            _ => ZodiacSign::Pisces,
        }
    }
    
    /// Get sign name
    pub fn as_str(&self) -> &'static str {
        match self {
            ZodiacSign::Aries => "Aries",
            ZodiacSign::Taurus => "Taurus",
            ZodiacSign::Gemini => "Gemini",
            ZodiacSign::Cancer => "Cancer",
            ZodiacSign::Leo => "Leo",
            ZodiacSign::Virgo => "Virgo",
            ZodiacSign::Libra => "Libra",
            ZodiacSign::Scorpio => "Scorpio",
            ZodiacSign::Sagittarius => "Sagittarius",
            ZodiacSign::Capricorn => "Capricorn",
            ZodiacSign::Aquarius => "Aquarius",
            ZodiacSign::Pisces => "Pisces",
        }
    }
    
    /// Get symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            ZodiacSign::Aries => "♈",
            ZodiacSign::Taurus => "♉",
            ZodiacSign::Gemini => "♊",
            ZodiacSign::Cancer => "♋",
            ZodiacSign::Leo => "♌",
            ZodiacSign::Virgo => "♍",
            ZodiacSign::Libra => "♎",
            ZodiacSign::Scorpio => "♏",
            ZodiacSign::Sagittarius => "♐",
            ZodiacSign::Capricorn => "♑",
            ZodiacSign::Aquarius => "♒",
            ZodiacSign::Pisces => "♓",
        }
    }
    
    /// Get ruling planet
    pub fn ruler(&self) -> &'static str {
        match self {
            ZodiacSign::Aries => "Mars",
            ZodiacSign::Taurus => "Venus",
            ZodiacSign::Gemini => "Mercury",
            ZodiacSign::Cancer => "Moon",
            ZodiacSign::Leo => "Sun",
            ZodiacSign::Virgo => "Mercury",
            ZodiacSign::Libra => "Venus",
            ZodiacSign::Scorpio => "Mars",
            ZodiacSign::Sagittarius => "Jupiter",
            ZodiacSign::Capricorn => "Saturn",
            ZodiacSign::Aquarius => "Saturn",
            ZodiacSign::Pisces => "Jupiter",
        }
    }
    
    /// Get element
    pub fn element(&self) -> &'static str {
        match self {
            ZodiacSign::Aries | ZodiacSign::Leo | ZodiacSign::Sagittarius => "Fire",
            ZodiacSign::Taurus | ZodiacSign::Virgo | ZodiacSign::Capricorn => "Earth",
            ZodiacSign::Gemini | ZodiacSign::Libra | ZodiacSign::Aquarius => "Air",
            ZodiacSign::Cancer | ZodiacSign::Scorpio | ZodiacSign::Pisces => "Water",
        }
    }
    
    /// Get modality
    pub fn modality(&self) -> &'static str {
        match self {
            ZodiacSign::Aries | ZodiacSign::Cancer | ZodiacSign::Libra | ZodiacSign::Capricorn => "Cardinal",
            ZodiacSign::Taurus | ZodiacSign::Leo | ZodiacSign::Scorpio | ZodiacSign::Aquarius => "Fixed",
            ZodiacSign::Gemini | ZodiacSign::Virgo | ZodiacSign::Sagittarius | ZodiacSign::Pisces => "Mutable",
        }
    }
}

/// Ascendant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AscendantInfo {
    pub sign: ZodiacSign,
    pub degree: f64,
    pub nakshatra: String,
    pub pada: u8,
}

/// Moon information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonInfo {
    pub sign: ZodiacSign,
    pub degree: f64,
    pub nakshatra: String,
    pub pada: u8,
    pub rashi_lord: String,
}

/// Special points in the chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialPoints {
    /// Ascendant degree
    pub lagna: f64,
    /// Midheaven (10th cusp)
    pub midheaven: Option<f64>,
    /// Part of Fortune
    pub part_of_fortune: Option<f64>,
}

/// Navamsa chart (D9) - for marriage and dharma
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavamsaChart {
    /// Source birth data reference
    pub source: NativeInfo,
    /// Navamsa positions
    pub navamsa_positions: Vec<NavamsaPosition>,
    /// Vargottama planets (same sign in Rashi and Navamsa)
    pub vargottama: Vec<String>,
    /// D9 Lagna
    pub d9_lagna: ZodiacSign,
}

/// Position in Navamsa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavamsaPosition {
    pub planet: String,
    pub sign: ZodiacSign,
    pub degree: f64,
    pub is_vargottama: bool,
}

impl NavamsaChart {
    /// Calculate Navamsa sign from Rashi position
    /// 
    /// Navamsa division:
    /// - Movable signs (1,4,7,10): start from Aries
    /// - Fixed signs (2,5,8,11): start from Leo  
    /// - Dual signs (3,6,9,12): start from Sagittarius
    pub fn calculate_navamsa(rashi_sign: ZodiacSign, degree: f64) -> ZodiacSign {
        // Each navamsa is 3°20' (1/9th of 30°)
        let navamsa_number = (degree / (30.0 / 9.0)) as usize;
        
        let start_index = match rashi_sign.modality() {
            "Cardinal" => 0,  // Start from Aries
            "Fixed" => 4,     // Start from Leo
            _ => 8,           // Start from Sagittarius (Mutable/Dual)
        };
        
        ZodiacSign::from_index(start_index + navamsa_number)
    }
    
    /// Check if a planet is Vargottama
    pub fn is_vargottama(&self, planet_name: &str) -> bool {
        self.vargottama.iter().any(|p| p.eq_ignore_ascii_case(planet_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zodiac_sign() {
        assert_eq!(ZodiacSign::Aries.index(), 0);
        assert_eq!(ZodiacSign::Aries.ruler(), "Mars");
        assert_eq!(ZodiacSign::Leo.element(), "Fire");
        assert_eq!(ZodiacSign::Taurus.modality(), "Fixed");
    }

    #[test]
    fn test_zodiac_from_index() {
        assert_eq!(ZodiacSign::from_index(0), ZodiacSign::Aries);
        assert_eq!(ZodiacSign::from_index(11), ZodiacSign::Pisces);
        assert_eq!(ZodiacSign::from_index(12), ZodiacSign::Aries); // Wraps around
    }

    #[test]
    fn test_planet_position() {
        let pos = PlanetPosition {
            name: "Mars".to_string(),
            longitude: 0.0,
            sign: ZodiacSign::Aries,
            degree: 0.0,
            minutes: 0.0,
            house: 1,
            is_retrograde: false,
            is_combust: false,
            nakshatra: "Ashwini".to_string(),
            pada: 1,
            speed: 0.5,
            latitude: 0.0,
        };
        
        assert!(pos.in_own_sign()); // Mars rules Aries
    }

    #[test]
    fn test_house_type() {
        let house = HousePosition {
            number: 1,
            sign: ZodiacSign::Aries,
            cusp: 0.0,
            degree: 10.0,
            house_type: HouseType::Dharma,
            is_kendra: true,
            is_panapara: false,
            is_apoklima: false,
        };
        
        assert_eq!(house.same_type(), vec![1, 5, 9]);
    }

    #[test]
    fn test_navamsa_calculation() {
        // Test: 0° Aries should be in first navamsa = Aries
        let navamsa = NavamsaChart::calculate_navamsa(ZodiacSign::Aries, 0.0);
        assert_eq!(navamsa, ZodiacSign::Aries);
        
        // Test: 10° Aries (cardinal sign starts from Aries)
        // 10° / 3.33° = 3rd navamsa (index 3) → Aries + 3 = Cancer
        let navamsa = NavamsaChart::calculate_navamsa(ZodiacSign::Aries, 10.0);
        assert_eq!(navamsa, ZodiacSign::Cancer);
        
        // Test: 10° Taurus (fixed sign starts from Leo)
        // 10° = 3rd navamsa (index 3) → Leo + 3 = Scorpio
        let navamsa = NavamsaChart::calculate_navamsa(ZodiacSign::Taurus, 10.0);
        assert_eq!(navamsa, ZodiacSign::Scorpio);
    }
}
