//! Daily panchang formatter

use super::DailyPanchang;

/// Format panchang for display
pub fn format_panchang_text(panchang: &DailyPanchang) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("═══════════════════════════════════════\n"));
    output.push_str(&format!("          PANCHANG - {}\n", panchang.date));
    output.push_str(&format!("═══════════════════════════════════════\n\n"));
    
    output.push_str(&format!("📅 Day: {}\n", panchang.vara));
    output.push_str(&format!("🌅 Sunrise: {} | 🌇 Sunset: {}\n", panchang.sunrise, panchang.sunset));
    
    if let Some(ref moonrise) = panchang.moonrise {
        output.push_str(&format!("🌙 Moonrise: {} | Moonset: {}\n", 
            moonrise, 
            panchang.moonset.as_deref().unwrap_or("-")
        ));
    }
    
    output.push_str("\n--- Five Elements (Panchang) ---\n");
    output.push_str(&format!("Tithi: {} ({} Paksha) until {}\n", 
        panchang.tithi.name, panchang.tithi.paksha, panchang.tithi.end_time));
    output.push_str(&format!("Nakshatra: {} until {}\n", 
        panchang.nakshatra.name, panchang.nakshatra.end_time));
    output.push_str(&format!("Yoga: {} until {}\n", 
        panchang.yoga.name, panchang.yoga.end_time));
    output.push_str(&format!("Karana: {} until {}\n", 
        panchang.karana.name, panchang.karana.end_time));
    
    output.push_str("\n--- Inauspicious Periods ---\n");
    output.push_str(&format!("⚠️ Rahu Kalam: {} - {}\n", 
        panchang.rahu_kalam.start, panchang.rahu_kalam.end));
    output.push_str(&format!("⚠️ Yama Gandam: {} - {}\n", 
        panchang.yama_gandam.start, panchang.yama_gandam.end));
    output.push_str(&format!("⚠️ Gulika Kaal: {} - {}\n", 
        panchang.gulika_kaal.start, panchang.gulika_kaal.end));
    
    if !panchang.auspicious_periods.is_empty() {
        output.push_str("\n--- Auspicious Periods ---\n");
        for period in &panchang.auspicious_periods {
            output.push_str(&format!("✨ {}: {} - {}\n", 
                period.name, period.start, period.end));
        }
    }
    
    if !panchang.festivals.is_empty() {
        output.push_str("\n--- Festivals ---\n");
        for festival in &panchang.festivals {
            output.push_str(&format!("🎉 {}\n", festival));
        }
    }
    
    output.push_str(&format!("\n📆 Hindu Calendar: {} {}\n", 
        panchang.hindu_month, panchang.hindu_year));
    
    output
}

/// Format panchang as JSON
pub fn format_panchang_json(panchang: &DailyPanchang) -> String {
    serde_json::to_string_pretty(panchang).unwrap_or_default()
}

/// Get short summary for a day
pub fn get_day_summary(panchang: &DailyPanchang) -> String {
    format!(
        "{} | {} {} | {} | {}",
        panchang.date,
        panchang.tithi.name,
        panchang.tithi.paksha,
        panchang.nakshatra.name,
        panchang.vara
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daily_panchang::calculate_daily_panchang;
    use chrono::NaiveDate;

    #[test]
    fn test_format_panchang() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let panchang = calculate_daily_panchang(date, 12.97, 77.59, 5.5);
        
        let text = format_panchang_text(&panchang);
        assert!(text.contains("PANCHANG"));
        assert!(text.contains("Tithi"));
        assert!(text.contains("Nakshatra"));
    }
}
