//! Vimshottari Dasha mappers
//!
//! FAPI-037: Map API Mahadasha to internal model
//! FAPI-038: Map API Antar Dasha to internal model

use chrono::NaiveDate;

use crate::error::VedicApiResult;
use super::api::{
    VimshottariApiResponse, MahadashaResponse, AntardashaResponse,
    PratyantardashaResponse, SookshmaResponse, parse_dasha_lord, parse_date,
};
use super::types::{
    DashaLord, DashaLevel, DashaPeriod, DashaBalance, VimshottariTimeline,
};

/// Map complete Vimshottari API response to internal timeline
pub fn map_vimshottari_response(
    response: VimshottariApiResponse,
    birth_datetime: chrono::NaiveDateTime,
) -> VedicApiResult<VimshottariTimeline> {
    let birth_nakshatra = response.moon_nakshatra.name.clone();
    let birth_dasha_balance = map_dasha_balance(&response)?;
    let mahadashas = response.mahadashas.iter()
        .map(map_mahadasha)
        .collect::<VedicApiResult<Vec<_>>>()?;

    Ok(VimshottariTimeline {
        birth_datetime,
        birth_nakshatra,
        birth_dasha_balance,
        mahadashas,
    })
}

/// Map dasha balance at birth
fn map_dasha_balance(response: &VimshottariApiResponse) -> VedicApiResult<DashaBalance> {
    let lord = parse_dasha_lord(&response.dasha_balance.lord)?;
    let years = response.dasha_balance.years;
    let months = response.dasha_balance.months;
    let days = response.dasha_balance.days;
    
    // Calculate total days
    let total_days = (years as i64 * 365) + (months as i64 * 30) + (days as i64);

    Ok(DashaBalance {
        lord,
        years,
        months,
        days,
        total_days,
    })
}

/// Map Mahadasha from API response
///
/// FAPI-037: Map API Mahadasha to internal model
pub fn map_mahadasha(md: &MahadashaResponse) -> VedicApiResult<DashaPeriod> {
    let lord = parse_dasha_lord(&md.lord)?;
    let start_date = parse_date(&md.start_date)?;
    let end_date = parse_date(&md.end_date)?;
    let duration_days = (end_date - start_date).num_days();

    let sub_periods = if let Some(ref ads) = md.antardashas {
        ads.iter().map(map_antardasha).collect::<VedicApiResult<Vec<_>>>()?
    } else {
        vec![]
    };

    Ok(DashaPeriod {
        lord,
        level: DashaLevel::Mahadasha,
        start_date,
        end_date,
        duration_days,
        sub_periods,
    })
}

/// Map Antardasha from API response
///
/// FAPI-038: Map API Antar Dasha to internal model
pub fn map_antardasha(ad: &AntardashaResponse) -> VedicApiResult<DashaPeriod> {
    let lord = parse_dasha_lord(&ad.lord)?;
    let start_date = parse_date(&ad.start_date)?;
    let end_date = parse_date(&ad.end_date)?;
    let duration_days = (end_date - start_date).num_days();

    let sub_periods = if let Some(ref pads) = ad.pratyantardashas {
        pads.iter().map(map_pratyantardasha).collect::<VedicApiResult<Vec<_>>>()?
    } else {
        vec![]
    };

    Ok(DashaPeriod {
        lord,
        level: DashaLevel::Antardasha,
        start_date,
        end_date,
        duration_days,
        sub_periods,
    })
}

/// Map Pratyantardasha from API response
pub fn map_pratyantardasha(pad: &PratyantardashaResponse) -> VedicApiResult<DashaPeriod> {
    let lord = parse_dasha_lord(&pad.lord)?;
    let start_date = parse_date(&pad.start_date)?;
    let end_date = parse_date(&pad.end_date)?;
    let duration_days = (end_date - start_date).num_days();

    let sub_periods = if let Some(ref sookshmas) = pad.sookshmas {
        sookshmas.iter().map(map_sookshma).collect::<VedicApiResult<Vec<_>>>()?
    } else {
        vec![]
    };

    Ok(DashaPeriod {
        lord,
        level: DashaLevel::Pratyantardasha,
        start_date,
        end_date,
        duration_days,
        sub_periods,
    })
}

/// Map Sookshma dasha from API response
pub fn map_sookshma(sookshma: &SookshmaResponse) -> VedicApiResult<DashaPeriod> {
    let lord = parse_dasha_lord(&sookshma.lord)?;
    let start_date = parse_date(&sookshma.start_date)?;
    let end_date = parse_date(&sookshma.end_date)?;
    let duration_days = (end_date - start_date).num_days();

    Ok(DashaPeriod {
        lord,
        level: DashaLevel::Sookshma,
        start_date,
        end_date,
        duration_days,
        sub_periods: vec![],
    })
}

/// Calculate days between two dates
pub fn days_between(start: NaiveDate, end: NaiveDate) -> i64 {
    (end - start).num_days()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::api::DashaBalanceResponse;

    #[test]
    fn test_map_mahadasha() {
        let md = MahadashaResponse {
            lord: "Sun".to_string(),
            start_date: "2020-01-01".to_string(),
            end_date: "2026-01-01".to_string(),
            antardashas: None,
        };

        let result = map_mahadasha(&md).unwrap();
        assert_eq!(result.lord, DashaLord::Sun);
        assert_eq!(result.level, DashaLevel::Mahadasha);
        assert!(result.sub_periods.is_empty());
    }

    #[test]
    fn test_map_antardasha() {
        let ad = AntardashaResponse {
            lord: "Moon".to_string(),
            start_date: "2020-01-01".to_string(),
            end_date: "2020-07-01".to_string(),
            pratyantardashas: None,
        };

        let result = map_antardasha(&ad).unwrap();
        assert_eq!(result.lord, DashaLord::Moon);
        assert_eq!(result.level, DashaLevel::Antardasha);
    }

    #[test]
    fn test_map_nested_dashas() {
        let ad = AntardashaResponse {
            lord: "Mars".to_string(),
            start_date: "2020-01-01".to_string(),
            end_date: "2020-06-01".to_string(),
            pratyantardashas: Some(vec![
                PratyantardashaResponse {
                    lord: "Rahu".to_string(),
                    start_date: "2020-01-01".to_string(),
                    end_date: "2020-02-01".to_string(),
                    sookshmas: None,
                },
            ]),
        };

        let result = map_antardasha(&ad).unwrap();
        assert_eq!(result.sub_periods.len(), 1);
        assert_eq!(result.sub_periods[0].lord, DashaLord::Rahu);
    }
}
