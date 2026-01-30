// Simple module for basic functionality

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub fn hello() -> &'static str {
    "Hello from Selemene Engine!"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BirthData {
    pub name: String,
    pub date: String,
    pub time: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PanchangaResult {
    pub tithi: f64,
    pub nakshatra: f64,
    pub yoga: f64,
    pub karana: f64,
    pub vara: i32,
    pub solar_longitude: f64,
    pub lunar_longitude: f64,
    pub julian_day: f64,
    pub calculation_time: DateTime<Utc>,
}

pub fn calculate_julian_day(date: &str, time: &str) -> f64 {
    // Parse date and time (assuming format: "1991-08-13" and "13:31")
    let date_parts: Vec<&str> = date.split('-').collect();
    let time_parts: Vec<&str> = time.split(':').collect();
    
    if date_parts.len() != 3 || time_parts.len() != 2 {
        return 0.0;
    }
    
    let year: i32 = date_parts[0].parse().unwrap_or(1991);
    let month: i32 = date_parts[1].parse().unwrap_or(8);
    let day: i32 = date_parts[2].parse().unwrap_or(13);
    let hour: i32 = time_parts[0].parse().unwrap_or(13);
    let minute: i32 = time_parts[1].parse().unwrap_or(31);
    
    // Convert to UTC (Bengaluru is UTC+5:30)
    let utc_hour = hour as f64 - 5.5;
    let utc_minute = minute as f64;
    
    // Simple Julian Day calculation (approximate)
    let jd = 367.0 * year as f64
        - (7.0 * (year + (month + 9) / 12) as f64 / 4.0)
        + (275.0 * month as f64 / 9.0)
        + day as f64
        + 1721013.5
        + utc_hour / 24.0
        + utc_minute / 1440.0;
    
    jd
}

pub fn calculate_solar_position(jd: f64) -> f64 {
    // Simplified solar longitude calculation
    // Using approximate formula for solar position
    let t = (jd - 2451545.0) / 36525.0;
    let l0 = 280.46645 + 36000.76983 * t + 0.0003032 * t * t;
    let l0 = l0 % 360.0;
    if l0 < 0.0 { l0 + 360.0 } else { l0 }
}

pub fn calculate_lunar_position(jd: f64) -> f64 {
    // Simplified lunar longitude calculation
    // Using approximate formula for lunar position
    let t = (jd - 2451545.0) / 36525.0;
    let l = 218.3164477 + 481267.88123421 * t - 0.0015786 * t * t + t * t * t / 538841.0 - t * t * t * t / 65194000.0;
    let l = l % 360.0;
    if l < 0.0 { l + 360.0 } else { l }
}

pub fn calculate_tithi(solar_longitude: f64, lunar_longitude: f64) -> f64 {
    // Tithi = (Lunar Longitude - Solar Longitude) / 12
    let mut tithi = (lunar_longitude - solar_longitude) / 12.0;
    if tithi < 0.0 { tithi += 30.0; }
    tithi
}

pub fn calculate_nakshatra(lunar_longitude: f64) -> f64 {
    // Nakshatra = Lunar Longitude / 13.333... (360/27)
    lunar_longitude / 13.333333333333334
}

pub fn calculate_yoga(solar_longitude: f64, lunar_longitude: f64) -> f64 {
    // Yoga = (Solar Longitude + Lunar Longitude) / 13.333... (360/27)
    let yoga = (solar_longitude + lunar_longitude) / 13.333333333333334;
    yoga % 27.0
}

pub fn calculate_karana(tithi: f64) -> f64 {
    // Karana = Tithi % 11 (11 Karanas in total)
    let karana = (tithi as i32) % 11;
    if karana == 0 { 11.0 } else { karana as f64 }
}

pub fn calculate_vara(jd: f64) -> i32 {
    // Vara = Day of week (0 = Sunday, 1 = Monday, etc.)
    let day_number = (jd + 1.5) as i64;
    (day_number % 7) as i32
}

pub fn calculate_panchanga_for_birth(birth_data: &BirthData) -> PanchangaResult {
    let jd = calculate_julian_day(&birth_data.date, &birth_data.time);
    let solar_longitude = calculate_solar_position(jd);
    let lunar_longitude = calculate_lunar_position(jd);
    let tithi = calculate_tithi(solar_longitude, lunar_longitude);
    let nakshatra = calculate_nakshatra(lunar_longitude);
    let yoga = calculate_yoga(solar_longitude, lunar_longitude);
    let karana = calculate_karana(tithi);
    let vara = calculate_vara(jd);
    
    PanchangaResult {
        tithi,
        nakshatra,
        yoga,
        karana,
        vara,
        solar_longitude,
        lunar_longitude,
        julian_day: jd,
        calculation_time: Utc::now(),
    }
}
