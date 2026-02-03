//! Dasha transition calculations
//!
//! FAPI-040: Implement upcoming transitions

use chrono::NaiveDate;

use super::types::{DashaLord, DashaPeriod, DashaLevel, VimshottariTimeline};

/// A dasha transition event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashaTransition {
    /// Date of the transition
    pub date: NaiveDate,
    /// Level of the transition
    pub level: DashaLevel,
    /// Previous lord
    pub from_lord: DashaLord,
    /// New lord
    pub to_lord: DashaLord,
    /// Human-readable description
    pub description: String,
}

/// Get upcoming dasha transitions within a date range
pub fn upcoming_transitions(
    timeline: &VimshottariTimeline,
    from_date: NaiveDate,
    to_date: NaiveDate,
    include_levels: &[DashaLevel],
) -> Vec<DashaTransition> {
    let mut transitions = vec![];

    for (i, md) in timeline.mahadashas.iter().enumerate() {
        // Mahadasha transitions
        if include_levels.contains(&DashaLevel::Mahadasha) {
            if md.start_date > from_date && md.start_date <= to_date {
                let prev_lord = if i > 0 {
                    timeline.mahadashas[i - 1].lord
                } else {
                    md.lord.next() // Wrap around
                };
                
                transitions.push(DashaTransition {
                    date: md.start_date,
                    level: DashaLevel::Mahadasha,
                    from_lord: prev_lord,
                    to_lord: md.lord,
                    description: format!(
                        "Mahadasha changes from {} to {}",
                        prev_lord, md.lord
                    ),
                });
            }
        }

        // Antardasha transitions
        if include_levels.contains(&DashaLevel::Antardasha) {
            for (j, ad) in md.sub_periods.iter().enumerate() {
                if ad.start_date > from_date && ad.start_date <= to_date {
                    let prev_lord = if j > 0 {
                        md.sub_periods[j - 1].lord
                    } else {
                        ad.lord
                    };
                    
                    transitions.push(DashaTransition {
                        date: ad.start_date,
                        level: DashaLevel::Antardasha,
                        from_lord: prev_lord,
                        to_lord: ad.lord,
                        description: format!(
                            "Antardasha changes from {} to {} (in {} Mahadasha)",
                            prev_lord, ad.lord, md.lord
                        ),
                    });
                }

                // Pratyantardasha transitions
                if include_levels.contains(&DashaLevel::Pratyantardasha) {
                    for (k, pd) in ad.sub_periods.iter().enumerate() {
                        if pd.start_date > from_date && pd.start_date <= to_date {
                            let prev_lord = if k > 0 {
                                ad.sub_periods[k - 1].lord
                            } else {
                                pd.lord
                            };
                            
                            transitions.push(DashaTransition {
                                date: pd.start_date,
                                level: DashaLevel::Pratyantardasha,
                                from_lord: prev_lord,
                                to_lord: pd.lord,
                                description: format!(
                                    "Pratyantardasha changes from {} to {} (in {}-{} period)",
                                    prev_lord, pd.lord, md.lord, ad.lord
                                ),
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by date
    transitions.sort_by_key(|t| t.date);
    transitions
}

/// Get the next mahadasha transition
pub fn next_mahadasha_transition(
    timeline: &VimshottariTimeline,
    after_date: NaiveDate,
) -> Option<DashaTransition> {
    upcoming_transitions(
        timeline,
        after_date,
        after_date + chrono::Duration::days(365 * 20), // 20 years max
        &[DashaLevel::Mahadasha],
    ).into_iter().next()
}

/// Get the next antardasha transition
pub fn next_antardasha_transition(
    timeline: &VimshottariTimeline,
    after_date: NaiveDate,
) -> Option<DashaTransition> {
    upcoming_transitions(
        timeline,
        after_date,
        after_date + chrono::Duration::days(365 * 3), // 3 years max
        &[DashaLevel::Antardasha],
    ).into_iter().next()
}

/// Get all transitions in the next N days
pub fn transitions_in_next_days(
    timeline: &VimshottariTimeline,
    from_date: NaiveDate,
    days: i64,
) -> Vec<DashaTransition> {
    let to_date = from_date + chrono::Duration::days(days);
    upcoming_transitions(
        timeline,
        from_date,
        to_date,
        &[DashaLevel::Mahadasha, DashaLevel::Antardasha, DashaLevel::Pratyantardasha],
    )
}

/// Summary of upcoming transitions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransitionSummary {
    /// Total transitions found
    pub total_transitions: usize,
    /// Next mahadasha change
    pub next_mahadasha: Option<DashaTransition>,
    /// Next antardasha change
    pub next_antardasha: Option<DashaTransition>,
    /// All transitions in the period
    pub all_transitions: Vec<DashaTransition>,
}

/// Build a transition summary for a date range
pub fn build_transition_summary(
    timeline: &VimshottariTimeline,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> TransitionSummary {
    let all_transitions = upcoming_transitions(
        timeline,
        from_date,
        to_date,
        &[DashaLevel::Mahadasha, DashaLevel::Antardasha],
    );

    let next_mahadasha = all_transitions.iter()
        .find(|t| t.level == DashaLevel::Mahadasha)
        .cloned();

    let next_antardasha = all_transitions.iter()
        .find(|t| t.level == DashaLevel::Antardasha)
        .cloned();

    TransitionSummary {
        total_transitions: all_transitions.len(),
        next_mahadasha,
        next_antardasha,
        all_transitions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::DashaBalance;
    use chrono::{NaiveDateTime, NaiveTime};

    fn sample_timeline() -> VimshottariTimeline {
        VimshottariTimeline {
            birth_datetime: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            ),
            birth_nakshatra: "Ashwini".to_string(),
            birth_dasha_balance: DashaBalance {
                lord: DashaLord::Ketu,
                years: 5,
                months: 3,
                days: 10,
                total_days: 1930,
            },
            mahadashas: vec![
                DashaPeriod {
                    lord: DashaLord::Ketu,
                    level: DashaLevel::Mahadasha,
                    start_date: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(1995, 4, 11).unwrap(),
                    duration_days: 1926,
                    sub_periods: vec![
                        DashaPeriod {
                            lord: DashaLord::Ketu,
                            level: DashaLevel::Antardasha,
                            start_date: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
                            end_date: NaiveDate::from_ymd_opt(1990, 5, 28).unwrap(),
                            duration_days: 147,
                            sub_periods: vec![],
                        },
                        DashaPeriod {
                            lord: DashaLord::Venus,
                            level: DashaLevel::Antardasha,
                            start_date: NaiveDate::from_ymd_opt(1990, 5, 29).unwrap(),
                            end_date: NaiveDate::from_ymd_opt(1991, 7, 28).unwrap(),
                            duration_days: 425,
                            sub_periods: vec![],
                        },
                    ],
                },
                DashaPeriod {
                    lord: DashaLord::Venus,
                    level: DashaLevel::Mahadasha,
                    start_date: NaiveDate::from_ymd_opt(1995, 4, 12).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(2015, 4, 11).unwrap(),
                    duration_days: 7305,
                    sub_periods: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_upcoming_mahadasha_transitions() {
        let timeline = sample_timeline();
        let from = NaiveDate::from_ymd_opt(1994, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(1996, 1, 1).unwrap();
        
        let transitions = upcoming_transitions(&timeline, from, to, &[DashaLevel::Mahadasha]);
        
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].to_lord, DashaLord::Venus);
    }

    #[test]
    fn test_upcoming_antardasha_transitions() {
        let timeline = sample_timeline();
        let from = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(1991, 1, 1).unwrap();
        
        let transitions = upcoming_transitions(&timeline, from, to, &[DashaLevel::Antardasha]);
        
        assert!(!transitions.is_empty());
        // First antardasha transition should be Ketu to Venus
        let venus_trans = transitions.iter().find(|t| t.to_lord == DashaLord::Venus);
        assert!(venus_trans.is_some());
    }

    #[test]
    fn test_next_mahadasha_transition() {
        let timeline = sample_timeline();
        let from = NaiveDate::from_ymd_opt(1992, 1, 1).unwrap();
        
        let next = next_mahadasha_transition(&timeline, from);
        assert!(next.is_some());
        let trans = next.unwrap();
        assert_eq!(trans.to_lord, DashaLord::Venus);
    }
}
