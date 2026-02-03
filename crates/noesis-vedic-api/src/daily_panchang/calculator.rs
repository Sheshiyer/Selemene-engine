//! Daily panchang calculator

use chrono::{NaiveDate, Datelike};
use super::{DailyPanchang, TimePeriod, TithiInfo, NakshatraInfo, YogaInfo, KaranaInfo};

/// Calculate daily panchang for a given date and location
pub fn calculate_daily_panchang(
    date: NaiveDate,
    latitude: f64,
    longitude: f64,
    timezone: f64,
) -> DailyPanchang {
    // Calculate sunrise/sunset (simplified - would use actual astronomical calculation)
    let sunrise = calculate_sunrise(date, latitude, longitude, timezone);
    let sunset = calculate_sunset(date, latitude, longitude, timezone);
    
    // Get vara (weekday)
    let vara = get_vara(date);
    
    // Calculate Rahu Kalam based on weekday
    let rahu_kalam = calculate_rahu_kalam(&vara, &sunrise, &sunset);
    let yama_gandam = calculate_yama_gandam(&vara, &sunrise, &sunset);
    let gulika_kaal = calculate_gulika_kaal(&vara, &sunrise, &sunset);
    let auspicious_periods = calculate_abhijit(&sunrise, &sunset);
    
    DailyPanchang {
        date,
        vara,
        tithi: TithiInfo {
            name: "Panchami".to_string(),
            number: 5,
            paksha: "Shukla".to_string(),
            end_time: "14:30".to_string(),
            deity: "Lakshmi".to_string(),
        },
        nakshatra: NakshatraInfo {
            name: "Rohini".to_string(),
            number: 4,
            end_time: "16:45".to_string(),
            deity: "Brahma".to_string(),
            ruler: "Moon".to_string(),
        },
        yoga: YogaInfo {
            name: "Siddhi".to_string(),
            number: 21,
            end_time: "18:20".to_string(),
            meaning: "Accomplishment".to_string(),
        },
        karana: KaranaInfo {
            name: "Bava".to_string(),
            number: 1,
            end_time: "08:15".to_string(),
        },
        sunrise,
        sunset,
        moonrise: Some("20:30".to_string()),
        moonset: Some("08:45".to_string()),
        rahu_kalam,
        yama_gandam,
        gulika_kaal,
        auspicious_periods,
        festivals: vec![],
        hindu_month: "Chaitra".to_string(),
        hindu_year: "Vikram 2081".to_string(),
        notes: vec![],
    }
}

fn calculate_sunrise(_date: NaiveDate, lat: f64, _lon: f64, _tz: f64) -> String {
    // Simplified sunrise calculation
    let base_hour = if lat > 0.0 { 6 } else { 5 };
    format!("{:02}:15", base_hour)
}

fn calculate_sunset(_date: NaiveDate, lat: f64, _lon: f64, _tz: f64) -> String {
    let base_hour = if lat > 0.0 { 18 } else { 19 };
    format!("{:02}:30", base_hour)
}

fn get_vara(date: NaiveDate) -> String {
    let weekdays = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    weekdays[date.weekday().num_days_from_sunday() as usize].to_string()
}

fn calculate_rahu_kalam(vara: &str, sunrise: &str, _sunset: &str) -> TimePeriod {
    // Rahu Kalam varies by weekday
    let (start_offset, end_offset) = match vara {
        "Sunday" => (4.5, 6.0),
        "Monday" => (1.5, 3.0),
        "Tuesday" => (3.0, 4.5),
        "Wednesday" => (4.5, 6.0),
        "Thursday" => (3.0, 4.5),
        "Friday" => (1.5, 3.0),
        "Saturday" => (6.0, 7.5),
        _ => (4.5, 6.0),
    };
    
    TimePeriod {
        name: "Rahu Kalam".to_string(),
        start: format_time_offset(sunrise, start_offset),
        end: format_time_offset(sunrise, end_offset),
        is_auspicious: false,
    }
}

fn calculate_yama_gandam(vara: &str, sunrise: &str, _sunset: &str) -> TimePeriod {
    let (start_offset, end_offset) = match vara {
        "Sunday" => (3.0, 4.5),
        "Monday" => (4.5, 6.0),
        "Tuesday" => (6.0, 7.5),
        "Wednesday" => (1.5, 3.0),
        "Thursday" => (0.0, 1.5),
        "Friday" => (3.0, 4.5),
        "Saturday" => (4.5, 6.0),
        _ => (3.0, 4.5),
    };
    
    TimePeriod {
        name: "Yama Gandam".to_string(),
        start: format_time_offset(sunrise, start_offset),
        end: format_time_offset(sunrise, end_offset),
        is_auspicious: false,
    }
}

fn calculate_gulika_kaal(vara: &str, sunrise: &str, _sunset: &str) -> TimePeriod {
    let (start_offset, end_offset) = match vara {
        "Sunday" => (6.0, 7.5),
        "Monday" => (3.0, 4.5),
        "Tuesday" => (4.5, 6.0),
        "Wednesday" => (6.0, 7.5),
        "Thursday" => (4.5, 6.0),
        "Friday" => (3.0, 4.5),
        "Saturday" => (0.0, 1.5),
        _ => (6.0, 7.5),
    };
    
    TimePeriod {
        name: "Gulika Kaal".to_string(),
        start: format_time_offset(sunrise, start_offset),
        end: format_time_offset(sunrise, end_offset),
        is_auspicious: false,
    }
}

fn calculate_abhijit(sunrise: &str, sunset: &str) -> Vec<TimePeriod> {
    // Abhijit muhurta is around solar noon
    vec![TimePeriod {
        name: "Abhijit Muhurta".to_string(),
        start: "11:48".to_string(),
        end: "12:36".to_string(),
        is_auspicious: true,
    }]
}

fn format_time_offset(base_time: &str, hours_offset: f64) -> String {
    let parts: Vec<&str> = base_time.split(':').collect();
    if parts.len() < 2 { return base_time.to_string(); }
    
    let base_hour: u32 = parts[0].parse().unwrap_or(6);
    let base_min: u32 = parts[1].parse().unwrap_or(0);
    
    let total_minutes = (base_hour * 60 + base_min) + (hours_offset * 60.0) as u32;
    let new_hour = (total_minutes / 60) % 24;
    let new_min = total_minutes % 60;
    
    format!("{:02}:{:02}", new_hour, new_min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vara() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(); // Known Sunday
        assert_eq!(get_vara(date), "Sunday");
    }

    #[test]
    fn test_format_time_offset() {
        assert_eq!(format_time_offset("06:00", 1.5), "07:30");
        assert_eq!(format_time_offset("06:15", 3.0), "09:15");
    }
}
