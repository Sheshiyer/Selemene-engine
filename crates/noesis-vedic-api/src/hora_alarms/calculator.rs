//! Hora calculator

use chrono::{NaiveDate, NaiveTime, Timelike, Datelike};
use super::{HoraPlanet, HoraQuality, PlanetaryHora};

/// Calculate all horas for a day
pub fn calculate_day_horas(
    date: NaiveDate,
    sunrise: NaiveTime,
    sunset: NaiveTime,
) -> Vec<PlanetaryHora> {
    let mut horas = Vec::with_capacity(24);
    
    // Calculate day and night duration
    let day_minutes = time_diff_minutes(sunrise, sunset);
    let night_minutes = 24 * 60 - day_minutes;
    
    let day_hora_minutes = day_minutes / 12;
    let night_hora_minutes = night_minutes / 12;
    
    // Get day ruler
    let weekday = date.weekday().num_days_from_sunday() as u8;
    let day_ruler = HoraPlanet::day_ruler(weekday);
    
    // Find starting position in Chaldean sequence
    let sequence = HoraPlanet::chaldean_sequence();
    let start_idx = sequence.iter()
        .position(|p| *p == day_ruler)
        .unwrap_or(0);
    
    // Calculate day horas (sunrise to sunset)
    let mut current_time = sunrise;
    for i in 0..12 {
        let planet_idx = (start_idx + i) % 7;
        let planet = sequence[planet_idx];
        
        let end_time = add_minutes(current_time, day_hora_minutes);
        
        horas.push(PlanetaryHora {
            number: (i + 1) as u8,
            ruler: planet.name().to_string(),
            start_time: current_time,
            end_time,
            is_day_hora: true,
            quality: assess_hora_quality(planet, true),
        });
        
        current_time = end_time;
    }
    
    // Calculate night horas (sunset to next sunrise)
    for i in 0..12 {
        let planet_idx = (start_idx + 12 + i) % 7;
        let planet = sequence[planet_idx];
        
        let end_time = add_minutes(current_time, night_hora_minutes);
        
        horas.push(PlanetaryHora {
            number: (i + 13) as u8,
            ruler: planet.name().to_string(),
            start_time: current_time,
            end_time,
            is_day_hora: false,
            quality: assess_hora_quality(planet, false),
        });
        
        current_time = end_time;
    }
    
    horas
}

/// Get current hora
pub fn get_current_hora(
    horas: &[PlanetaryHora],
    current_time: NaiveTime,
) -> Option<&PlanetaryHora> {
    horas.iter().find(|h| {
        is_time_between(current_time, h.start_time, h.end_time)
    })
}

/// Get next hora of a specific planet
pub fn get_next_hora_of_planet<'a>(
    horas: &'a [PlanetaryHora],
    planet: &str,
    after_time: NaiveTime,
) -> Option<&'a PlanetaryHora> {
    horas.iter().find(|h| {
        h.ruler.eq_ignore_ascii_case(planet) && h.start_time >= after_time
    })
}

fn time_diff_minutes(start: NaiveTime, end: NaiveTime) -> u32 {
    let start_mins = start.hour() * 60 + start.minute();
    let end_mins = end.hour() * 60 + end.minute();
    
    if end_mins >= start_mins {
        end_mins - start_mins
    } else {
        (24 * 60 - start_mins) + end_mins
    }
}

fn add_minutes(time: NaiveTime, minutes: u32) -> NaiveTime {
    let total_mins = time.hour() * 60 + time.minute() + minutes;
    NaiveTime::from_hms_opt(
        (total_mins / 60) % 24,
        total_mins % 60,
        0,
    ).unwrap_or(time)
}

fn is_time_between(time: NaiveTime, start: NaiveTime, end: NaiveTime) -> bool {
    if start <= end {
        time >= start && time < end
    } else {
        // Crosses midnight
        time >= start || time < end
    }
}

fn assess_hora_quality(planet: HoraPlanet, is_day: bool) -> HoraQuality {
    match planet {
        HoraPlanet::Jupiter | HoraPlanet::Venus => HoraQuality::Excellent,
        HoraPlanet::Mercury | HoraPlanet::Moon => HoraQuality::Good,
        HoraPlanet::Sun => {
            if is_day { HoraQuality::Good } else { HoraQuality::Neutral }
        }
        HoraPlanet::Mars | HoraPlanet::Saturn => HoraQuality::Challenging,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_horas() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(); // Sunday
        let sunrise = NaiveTime::from_hms_opt(6, 30, 0).unwrap();
        let sunset = NaiveTime::from_hms_opt(18, 30, 0).unwrap();
        
        let horas = calculate_day_horas(date, sunrise, sunset);
        
        assert_eq!(horas.len(), 24);
        // First hora of Sunday should be Sun
        assert_eq!(horas[0].ruler, "Sun");
    }

    #[test]
    fn test_hora_planet_activities() {
        let activities = HoraPlanet::Jupiter.favorable_activities();
        assert!(activities.iter().any(|a| a.contains("Spiritual")));
    }
}
