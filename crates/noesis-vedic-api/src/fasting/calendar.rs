//! Fasting calendar

use chrono::{NaiveDate, Datelike};
use super::{FastingDay, FastType, MonthlyFastingSchedule};

/// Get fasting days for a month
pub fn get_monthly_fasting_schedule(year: i32, month: u32) -> MonthlyFastingSchedule {
    let ekadashis = get_ekadashis_for_month(year, month);
    let pradosh_vrats = get_pradosh_for_month(year, month);
    let special_vrats = get_special_vrats_for_month(year, month);
    
    let mut all_days = vec![];
    all_days.extend(ekadashis.clone());
    all_days.extend(pradosh_vrats.clone());
    all_days.extend(special_vrats.clone());
    
    all_days.sort_by_key(|d| d.date);
    
    MonthlyFastingSchedule {
        year,
        month,
        fasting_days: all_days,
        ekadashis,
        pradosh_vrats,
        special_vrats,
    }
}

/// Get Ekadashi dates for a month
pub fn get_ekadashis_for_month(year: i32, month: u32) -> Vec<FastingDay> {
    // Approximate - would use actual tithi calculation
    let mut ekadashis = vec![];
    
    // Two Ekadashis per month (Shukla and Krishna paksha)
    let date1 = NaiveDate::from_ymd_opt(year, month, 11).unwrap_or(
        NaiveDate::from_ymd_opt(year, month, 1).unwrap()
    );
    let date2 = NaiveDate::from_ymd_opt(year, month, 26).unwrap_or(
        NaiveDate::from_ymd_opt(year, month, 20).unwrap()
    );
    
    ekadashis.push(FastingDay {
        date: date1,
        name: "Shukla Ekadashi".to_string(),
        fast_type: FastType::Ekadashi,
        deity: "Vishnu".to_string(),
        tithi: "Ekadashi".to_string(),
        parana_time: Some("Next day after sunrise".to_string()),
        allowed_foods: vec![
            "Fruits".to_string(),
            "Milk products".to_string(),
            "Nuts".to_string(),
            "Sabudana".to_string(),
        ],
        restricted_foods: vec![
            "Rice".to_string(),
            "Grains".to_string(),
            "Beans".to_string(),
            "Onion".to_string(),
            "Garlic".to_string(),
        ],
        benefits: "Liberation from sins, spiritual progress, Vishnu's blessings".to_string(),
        mantras: vec![
            "Om Namo Narayanaya".to_string(),
            "Om Namo Bhagavate Vasudevaya".to_string(),
        ],
    });
    
    ekadashis.push(FastingDay {
        date: date2,
        name: "Krishna Ekadashi".to_string(),
        fast_type: FastType::Ekadashi,
        deity: "Vishnu".to_string(),
        tithi: "Ekadashi".to_string(),
        parana_time: Some("Next day after sunrise".to_string()),
        allowed_foods: vec![
            "Fruits".to_string(),
            "Milk products".to_string(),
            "Nuts".to_string(),
        ],
        restricted_foods: vec![
            "Rice".to_string(),
            "Grains".to_string(),
            "Beans".to_string(),
        ],
        benefits: "Destroys sins, grants moksha, pleases Lord Vishnu".to_string(),
        mantras: vec!["Om Namo Narayanaya".to_string()],
    });
    
    ekadashis
}

/// Get Pradosh vrat dates
pub fn get_pradosh_for_month(year: i32, month: u32) -> Vec<FastingDay> {
    // Pradosh is on Trayodashi (13th tithi)
    let date1 = NaiveDate::from_ymd_opt(year, month, 13).unwrap_or(
        NaiveDate::from_ymd_opt(year, month, 1).unwrap()
    );
    let date2 = NaiveDate::from_ymd_opt(year, month, 28).unwrap_or(
        NaiveDate::from_ymd_opt(year, month, 20).unwrap()
    );
    
    vec![
        FastingDay {
            date: date1,
            name: "Shukla Pradosh".to_string(),
            fast_type: FastType::Partial,
            deity: "Shiva".to_string(),
            tithi: "Trayodashi".to_string(),
            parana_time: Some("After Shiva puja in evening".to_string()),
            allowed_foods: vec!["Light sattvic food".to_string(), "Fruits".to_string()],
            restricted_foods: vec!["Non-veg".to_string(), "Alcohol".to_string()],
            benefits: "Removal of sins, Lord Shiva's blessings".to_string(),
            mantras: vec!["Om Namah Shivaya".to_string()],
        },
        FastingDay {
            date: date2,
            name: "Krishna Pradosh".to_string(),
            fast_type: FastType::Partial,
            deity: "Shiva".to_string(),
            tithi: "Trayodashi".to_string(),
            parana_time: Some("After Shiva puja in evening".to_string()),
            allowed_foods: vec!["Light sattvic food".to_string()],
            restricted_foods: vec!["Non-veg".to_string()],
            benefits: "Shiva's grace, removal of obstacles".to_string(),
            mantras: vec!["Om Namah Shivaya".to_string()],
        },
    ]
}

/// Get special vrats for a month
pub fn get_special_vrats_for_month(year: i32, month: u32) -> Vec<FastingDay> {
    let mut special = vec![];
    
    // Add Purnima (full moon) vrat
    if let Some(date) = NaiveDate::from_ymd_opt(year, month, 15) {
        special.push(FastingDay {
            date,
            name: "Purnima Vrat".to_string(),
            fast_type: FastType::Partial,
            deity: "Chandra (Moon)".to_string(),
            tithi: "Purnima".to_string(),
            parana_time: Some("After moonrise".to_string()),
            allowed_foods: vec!["Fruits".to_string(), "Milk".to_string()],
            restricted_foods: vec!["Heavy meals".to_string()],
            benefits: "Mental peace, lunar blessings".to_string(),
            mantras: vec!["Om Som Somaya Namah".to_string()],
        });
    }
    
    // Add Amavasya (new moon) vrat
    if let Some(date) = NaiveDate::from_ymd_opt(year, month, 1) {
        special.push(FastingDay {
            date,
            name: "Amavasya Tarpan".to_string(),
            fast_type: FastType::Partial,
            deity: "Pitru (Ancestors)".to_string(),
            tithi: "Amavasya".to_string(),
            parana_time: Some("After tarpan rituals".to_string()),
            allowed_foods: vec!["Simple food".to_string()],
            restricted_foods: vec!["Heavy non-veg meals".to_string()],
            benefits: "Ancestral blessings, pitru dosha remedy".to_string(),
            mantras: vec!["Om Pitru Devaya Namah".to_string()],
        });
    }
    
    special
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monthly_schedule() {
        let schedule = get_monthly_fasting_schedule(2024, 1);
        assert!(!schedule.ekadashis.is_empty());
        assert_eq!(schedule.ekadashis.len(), 2);
    }

    #[test]
    fn test_ekadashi() {
        let ekadashis = get_ekadashis_for_month(2024, 3);
        assert!(ekadashis.iter().all(|e| e.deity == "Vishnu"));
    }
}
