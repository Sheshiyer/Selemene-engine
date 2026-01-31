//! Nakshatra and Mahadasha calculation engine
//!
//! Core logic for:
//! - W1-S6-03: Birth nakshatra from Moon longitude
//! - W1-S6-04: Generate 9 Mahadasha periods
//! - W1-S6-05: Calculate balance of first dasha

use crate::models::{Mahadasha, Nakshatra, Pratyantardasha, VedicPlanet};
use chrono::{DateTime, Duration, Utc};
use engine_human_design::ephemeris::{EphemerisCalculator, HDPlanet};
use lazy_static::lazy_static;
use noesis_core::EngineError;

// Nakshatra data: 27 lunar mansions
lazy_static! {
    pub static ref NAKSHATRAS: Vec<Nakshatra> = vec![
        Nakshatra { number: 1, name: "Ashwini".to_string(), start_degree: 0.0, end_degree: 13.333333, ruling_planet: VedicPlanet::Ketu, deity: "Ashwini Kumaras".to_string(), symbol: "Horse's Head".to_string(), qualities: vec!["Swift".to_string(), "Healing".to_string()], description: "Beginning, healing energy".to_string() },
        Nakshatra { number: 2, name: "Bharani".to_string(), start_degree: 13.333333, end_degree: 26.666667, ruling_planet: VedicPlanet::Venus, deity: "Yama".to_string(), symbol: "Yoni".to_string(), qualities: vec!["Restraint".to_string(), "Transformation".to_string()], description: "Bearer of life and death".to_string() },
        Nakshatra { number: 3, name: "Krittika".to_string(), start_degree: 26.666667, end_degree: 40.0, ruling_planet: VedicPlanet::Sun, deity: "Agni".to_string(), symbol: "Razor".to_string(), qualities: vec!["Cutting".to_string(), "Purification".to_string()], description: "Cutter, purifying fire".to_string() },
        Nakshatra { number: 4, name: "Rohini".to_string(), start_degree: 40.0, end_degree: 53.333333, ruling_planet: VedicPlanet::Moon, deity: "Brahma".to_string(), symbol: "Cart".to_string(), qualities: vec!["Growth".to_string(), "Beauty".to_string()], description: "The growing one".to_string() },
        Nakshatra { number: 5, name: "Mrigashira".to_string(), start_degree: 53.333333, end_degree: 66.666667, ruling_planet: VedicPlanet::Mars, deity: "Soma".to_string(), symbol: "Deer's Head".to_string(), qualities: vec!["Seeking".to_string(), "Gentle".to_string()], description: "The searching star".to_string() },
        Nakshatra { number: 6, name: "Ardra".to_string(), start_degree: 66.666667, end_degree: 80.0, ruling_planet: VedicPlanet::Rahu, deity: "Rudra".to_string(), symbol: "Teardrop".to_string(), qualities: vec!["Storm".to_string(), "Destruction".to_string()], description: "The moist one, stormy".to_string() },
        Nakshatra { number: 7, name: "Punarvasu".to_string(), start_degree: 80.0, end_degree: 93.333333, ruling_planet: VedicPlanet::Jupiter, deity: "Aditi".to_string(), symbol: "Bow and Quiver".to_string(), qualities: vec!["Return".to_string(), "Renewal".to_string()], description: "Return of the light".to_string() },
        Nakshatra { number: 8, name: "Pushya".to_string(), start_degree: 93.333333, end_degree: 106.666667, ruling_planet: VedicPlanet::Saturn, deity: "Brihaspati".to_string(), symbol: "Cow's Udder".to_string(), qualities: vec!["Nourishing".to_string(), "Spiritual".to_string()], description: "The nourisher".to_string() },
        Nakshatra { number: 9, name: "Ashlesha".to_string(), start_degree: 106.666667, end_degree: 120.0, ruling_planet: VedicPlanet::Mercury, deity: "Nagas".to_string(), symbol: "Serpent".to_string(), qualities: vec!["Entwining".to_string(), "Clinging".to_string()], description: "The entwiner".to_string() },
        Nakshatra { number: 10, name: "Magha".to_string(), start_degree: 120.0, end_degree: 133.333333, ruling_planet: VedicPlanet::Ketu, deity: "Pitris".to_string(), symbol: "Throne".to_string(), qualities: vec!["Regal".to_string(), "Ancestral".to_string()], description: "The mighty one".to_string() },
        Nakshatra { number: 11, name: "Purva Phalguni".to_string(), start_degree: 133.333333, end_degree: 146.666667, ruling_planet: VedicPlanet::Venus, deity: "Bhaga".to_string(), symbol: "Hammock".to_string(), qualities: vec!["Pleasure".to_string(), "Rest".to_string()], description: "Former reddish one".to_string() },
        Nakshatra { number: 12, name: "Uttara Phalguni".to_string(), start_degree: 146.666667, end_degree: 160.0, ruling_planet: VedicPlanet::Sun, deity: "Aryaman".to_string(), symbol: "Bed".to_string(), qualities: vec!["Patronage".to_string(), "Generosity".to_string()], description: "Latter reddish one".to_string() },
        Nakshatra { number: 13, name: "Hasta".to_string(), start_degree: 160.0, end_degree: 173.333333, ruling_planet: VedicPlanet::Moon, deity: "Savitar".to_string(), symbol: "Hand".to_string(), qualities: vec!["Skill".to_string(), "Dexterity".to_string()], description: "The hand".to_string() },
        Nakshatra { number: 14, name: "Chitra".to_string(), start_degree: 173.333333, end_degree: 186.666667, ruling_planet: VedicPlanet::Mars, deity: "Tvashtar".to_string(), symbol: "Pearl".to_string(), qualities: vec!["Brilliant".to_string(), "Creation".to_string()], description: "The bright one".to_string() },
        Nakshatra { number: 15, name: "Swati".to_string(), start_degree: 186.666667, end_degree: 200.0, ruling_planet: VedicPlanet::Rahu, deity: "Vayu".to_string(), symbol: "Coral".to_string(), qualities: vec!["Independent".to_string(), "Movement".to_string()], description: "The sword".to_string() },
        Nakshatra { number: 16, name: "Vishakha".to_string(), start_degree: 200.0, end_degree: 213.333333, ruling_planet: VedicPlanet::Jupiter, deity: "Indra-Agni".to_string(), symbol: "Triumphal Arch".to_string(), qualities: vec!["Determination".to_string(), "Goal".to_string()], description: "Forked, two-branched".to_string() },
        Nakshatra { number: 17, name: "Anuradha".to_string(), start_degree: 213.333333, end_degree: 226.666667, ruling_planet: VedicPlanet::Saturn, deity: "Mitra".to_string(), symbol: "Lotus".to_string(), qualities: vec!["Devotion".to_string(), "Friendship".to_string()], description: "Following Radha".to_string() },
        Nakshatra { number: 18, name: "Jyeshtha".to_string(), start_degree: 226.666667, end_degree: 240.0, ruling_planet: VedicPlanet::Mercury, deity: "Indra".to_string(), symbol: "Earring".to_string(), qualities: vec!["Seniority".to_string(), "Protection".to_string()], description: "The eldest".to_string() },
        Nakshatra { number: 19, name: "Mula".to_string(), start_degree: 240.0, end_degree: 253.333333, ruling_planet: VedicPlanet::Ketu, deity: "Nirriti".to_string(), symbol: "Root".to_string(), qualities: vec!["Foundation".to_string(), "Destruction".to_string()], description: "The root".to_string() },
        Nakshatra { number: 20, name: "Purva Ashadha".to_string(), start_degree: 253.333333, end_degree: 266.666667, ruling_planet: VedicPlanet::Venus, deity: "Apas".to_string(), symbol: "Elephant Tusk".to_string(), qualities: vec!["Invincible".to_string(), "Purification".to_string()], description: "Former invincible one".to_string() },
        Nakshatra { number: 21, name: "Uttara Ashadha".to_string(), start_degree: 266.666667, end_degree: 280.0, ruling_planet: VedicPlanet::Sun, deity: "Vishvadevas".to_string(), symbol: "Planks".to_string(), qualities: vec!["Victory".to_string(), "Leadership".to_string()], description: "Latter invincible one".to_string() },
        Nakshatra { number: 22, name: "Shravana".to_string(), start_degree: 280.0, end_degree: 293.333333, ruling_planet: VedicPlanet::Moon, deity: "Vishnu".to_string(), symbol: "Ear".to_string(), qualities: vec!["Listening".to_string(), "Learning".to_string()], description: "The hearing".to_string() },
        Nakshatra { number: 23, name: "Dhanishta".to_string(), start_degree: 293.333333, end_degree: 306.666667, ruling_planet: VedicPlanet::Mars, deity: "Eight Vasus".to_string(), symbol: "Drum".to_string(), qualities: vec!["Wealthy".to_string(), "Musical".to_string()], description: "The most famous".to_string() },
        Nakshatra { number: 24, name: "Shatabhisha".to_string(), start_degree: 306.666667, end_degree: 320.0, ruling_planet: VedicPlanet::Rahu, deity: "Varuna".to_string(), symbol: "Empty Circle".to_string(), qualities: vec!["Healing".to_string(), "Mysterious".to_string()], description: "Hundred physicians".to_string() },
        Nakshatra { number: 25, name: "Purva Bhadrapada".to_string(), start_degree: 320.0, end_degree: 333.333333, ruling_planet: VedicPlanet::Jupiter, deity: "Aja Ekapada".to_string(), symbol: "Sword".to_string(), qualities: vec!["Intensity".to_string(), "Transformation".to_string()], description: "Former blessed feet".to_string() },
        Nakshatra { number: 26, name: "Uttara Bhadrapada".to_string(), start_degree: 333.333333, end_degree: 346.666667, ruling_planet: VedicPlanet::Saturn, deity: "Ahir Budhnya".to_string(), symbol: "Twin".to_string(), qualities: vec!["Depth".to_string(), "Wisdom".to_string()], description: "Latter blessed feet".to_string() },
        Nakshatra { number: 27, name: "Revati".to_string(), start_degree: 346.666667, end_degree: 360.0, ruling_planet: VedicPlanet::Mercury, deity: "Pushan".to_string(), symbol: "Fish".to_string(), qualities: vec!["Nourishing".to_string(), "Prosperous".to_string()], description: "The wealthy".to_string() },
    ];
}

/// W1-S6-03: Calculate birth nakshatra from Moon longitude
///
/// # Arguments
/// * `birth_time` - Birth date and time (UTC)
/// * `ephe_path` - Path to Swiss Ephemeris data files (use "" for built-in)
///
/// # Returns
/// Birth nakshatra with ruling planet and other details
pub fn calculate_birth_nakshatra(
    birth_time: DateTime<Utc>,
    ephe_path: &str,
) -> Result<Nakshatra, EngineError> {
    // Get Moon longitude using Swiss Ephemeris from HD engine
    let ephe = EphemerisCalculator::new(ephe_path);
    let moon_position = ephe.get_planet_position(HDPlanet::Moon, &birth_time)?;
    
    // Determine nakshatra: floor(longitude / 13.333) gives index 0-26
    let moon_longitude = moon_position.longitude;
    let nakshatra = get_nakshatra_from_longitude(moon_longitude);
    
    Ok(nakshatra.clone())
}

/// Get nakshatra from Moon longitude (0-360°)
pub fn get_nakshatra_from_longitude(longitude: f64) -> &'static Nakshatra {
    let normalized = longitude % 360.0;
    let index = (normalized / 13.333333).floor() as usize;
    &NAKSHATRAS[index.min(26)]
}

/// W1-S6-05: Calculate balance of dasha at birth (remaining portion of first Mahadasha)
///
/// # Arguments
/// * `moon_longitude` - Moon's longitude in degrees (0-360°)
/// * `nakshatra` - Birth nakshatra
///
/// # Returns
/// Balance in years (decimal) - how much of the first Mahadasha remains
///
/// # Formula
/// 1. Find position within nakshatra (0-13.333°)
/// 2. Calculate fraction remaining = (end - current) / 13.333
/// 3. Balance = fraction × planet_period_years
pub fn calculate_dasha_balance(
    moon_longitude: f64,
    nakshatra: &Nakshatra,
) -> f64 {
    // Normalize longitude
    let normalized_lng = moon_longitude % 360.0;
    
    // Calculate position within nakshatra
    let position_in_nakshatra = normalized_lng - nakshatra.start_degree;
    
    // Calculate remaining degrees in nakshatra
    let remaining_degrees = nakshatra.end_degree - normalized_lng;
    
    // Calculate fraction remaining (0.0 to 1.0)
    let fraction_remaining = remaining_degrees / 13.333333;
    
    // Apply to Mahadasha period
    let planet_period = nakshatra.ruling_planet.period_years() as f64;
    let balance_years = fraction_remaining * planet_period;
    
    balance_years
}

/// W1-S6-04: Generate 9 Mahadasha periods (120-year cycle)
///
/// # Arguments
/// * `birth_time` - Birth date and time (UTC)
/// * `starting_planet` - Birth nakshatra's ruling planet
/// * `balance_years` - Remaining years of first Mahadasha
///
/// # Returns
/// Vec of 9 Mahadasha periods with calculated dates
///
/// # Logic
/// 1. First Mahadasha is partial (balance_years duration)
/// 2. Cycle through all 9 planets in fixed order
/// 3. Each subsequent Mahadasha uses full period
/// 4. Calculate exact start/end dates for each
pub fn calculate_mahadashas(
    birth_time: DateTime<Utc>,
    starting_planet: VedicPlanet,
    balance_years: f64,
) -> Vec<Mahadasha> {
    let mut mahadashas = Vec::new();
    let mut current_date = birth_time;
    let mut current_planet = starting_planet;
    
    // Generate 9 Mahadashas
    for i in 0..9 {
        let duration_years = if i == 0 {
            // First Mahadasha uses balance
            balance_years
        } else {
            // Subsequent use full periods
            current_planet.period_years() as f64
        };
        
        // Calculate end date
        let days = (duration_years * 365.25) as i64;
        let end_date = current_date + Duration::days(days);
        
        mahadashas.push(Mahadasha {
            planet: current_planet,
            start_date: current_date,
            end_date,
            duration_years,
            antardashas: Vec::new(), // Will be filled by W1-S6-06
            qualities: crate::models::PlanetaryQualities {
                themes: vec![],
                qualities: vec![],
                element: String::new(),
                description: String::new(),
                consciousness_lessons: vec![],
                optimal_practices: vec![],
                challenges: vec![],
            },
        });
        
        // Move to next period
        current_date = end_date;
        current_planet = current_planet.next_planet();
    }
    
    mahadashas
}

/// W1-S6-06: Calculate Antardasha sub-periods within a Mahadasha
///
/// # Arguments
/// * `mahadasha` - The parent Mahadasha period
///
/// # Returns
/// Vec of 9 Antardasha periods
///
/// # Logic
/// - Antardasha sequence starts with Mahadasha lord
/// - Each Antardasha duration = (Mahadasha_years × Antardasha_planet_years) / 120
/// - Cycles through all 9 planets in Vimshottari order
pub fn calculate_antardashas(mahadasha: &Mahadasha) -> Vec<crate::models::Antardasha> {
    use crate::models::Antardasha;
    
    let maha_planet = mahadasha.planet;
    let maha_start = mahadasha.start_date;
    
    let mut antardashas = Vec::new();
    let mut current_start = maha_start;
    
    // Antardasha sequence starts with Mahadasha lord, cycles through all 9
    let mut planet = maha_planet;
    
    for _ in 0..9 {
        // Duration formula: (Mahadasha_years × Antardasha_planet_years) / 120
        let antar_duration_years = (mahadasha.duration_years * planet.period_years() as f64) / 120.0;
        let antar_duration_days = antar_duration_years * 365.25;
        
        let end_date = current_start + Duration::days(antar_duration_days as i64);
        
        antardashas.push(Antardasha {
            planet,
            start_date: current_start,
            end_date,
            duration_years: antar_duration_years,
            pratyantardashas: vec![], // Will be filled by W1-S6-07
        });
        
        current_start = end_date;
        planet = planet.next_planet();
    }
    
    antardashas
}

/// W1-S6-07: Calculate Pratyantardasha sub-sub-periods within an Antardasha
///
/// # Arguments
/// * `antardasha` - The parent Antardasha period
///
/// # Returns
/// Vec of 9 Pratyantardasha periods
///
/// # Logic
/// - Pratyantardasha sequence starts with Antardasha lord
/// - Each Pratyantardasha duration = (Antardasha_years × Pratyantardasha_planet_years) / 120
/// - Cycles through all 9 planets in Vimshottari order
pub fn calculate_pratyantardashas(antardasha: &crate::models::Antardasha) -> Vec<crate::models::Pratyantardasha> {
    use crate::models::Pratyantardasha;
    
    let antar_planet = antardasha.planet;
    let antar_start = antardasha.start_date;
    
    let mut pratyantardashas = Vec::new();
    let mut current_start = antar_start;
    
    // Pratyantardasha sequence starts with Antardasha lord
    let mut planet = antar_planet;
    
    for _ in 0..9 {
        // Duration formula: (Antardasha_years × Pratyantardasha_planet_years) / 120
        let pratyantar_duration_years = (antardasha.duration_years * planet.period_years() as f64) / 120.0;
        let pratyantar_duration_days = pratyantar_duration_years * 365.25;
        
        let end_date = current_start + Duration::days(pratyantar_duration_days as i64);
        
        pratyantardashas.push(Pratyantardasha {
            planet,
            start_date: current_start,
            end_date,
            duration_days: pratyantar_duration_days,
        });
        
        current_start = end_date;
        planet = planet.next_planet();
    }
    
    pratyantardashas
}

/// Complete timeline calculation: Build full 3-level nested structure
///
/// # Arguments
/// * `mahadashas` - Vec of 9 Mahadasha periods (from calculate_mahadashas)
///
/// # Returns
/// Complete timeline with Antardashas and Pratyantardashas populated
///
/// # Structure
/// - 9 Mahadashas (120 years total)
/// - Each Mahadasha → 9 Antardashas
/// - Each Antardasha → 9 Pratyantardashas
/// - Total: 9 × 9 × 9 = 729 Pratyantardasha periods
pub fn calculate_complete_timeline(mahadashas: Vec<Mahadasha>) -> Vec<Mahadasha> {
    mahadashas.into_iter().map(|mut maha| {
        // Calculate Antardashas for this Mahadasha
        let antardashas = calculate_antardashas(&maha);
        
        // Calculate Pratyantardashas for each Antardasha
        maha.antardashas = antardashas.into_iter().map(|mut antar| {
            antar.pratyantardashas = calculate_pratyantardashas(&antar);
            antar
        }).collect();
        
        maha
    }).collect()
}

/// Helper function to get nakshatra by number (1-27)
pub fn get_nakshatra(number: u8) -> Option<&'static Nakshatra> {
    if number < 1 || number > 27 {
        return None;
    }
    Some(&NAKSHATRAS[(number - 1) as usize])
}

/// W1-S6-08: Find current active period (Pratyantardasha) via binary search
///
/// # Arguments
/// * `mahadashas` - Complete timeline with all nested periods
/// * `current_time` - Time to query for active period
///
/// # Returns
/// Current period with all 3 levels (Mahadasha, Antardasha, Pratyantardasha)
///
/// # Algorithm
/// 1. Flatten 3-level structure into linear array (729 pratyantardashas)
/// 2. Binary search: O(log 729) ≈ 10 comparisons
/// 3. Walk up tree to find parent Antardasha and Mahadasha
pub fn find_current_period(
    mahadashas: &[Mahadasha],
    current_time: DateTime<Utc>
) -> Option<crate::models::CurrentPeriod> {
    use crate::models::{CurrentPeriod, CurrentMahadasha, CurrentAntardasha, CurrentPratyantardasha};
    
    // Flatten 3-level structure into linear array
    let mut all_periods: Vec<(&Pratyantardasha, &crate::models::Antardasha, &Mahadasha)> = Vec::new();
    
    for mahadasha in mahadashas {
        for antardasha in &mahadasha.antardashas {
            for pratyantardasha in &antardasha.pratyantardashas {
                all_periods.push((pratyantardasha, antardasha, mahadasha));
            }
        }
    }
    
    // Binary search: O(log 729) = ~10 comparisons
    let idx = all_periods.binary_search_by(|(period, _, _)| {
        if current_time < period.start_date {
            std::cmp::Ordering::Greater  // Current time before this period
        } else if current_time > period.end_date {
            std::cmp::Ordering::Less     // Current time after this period
        } else {
            std::cmp::Ordering::Equal    // FOUND!
        }
    }).ok()?;
    
    let (active_pratyantardasha, parent_antardasha, parent_mahadasha) = all_periods[idx];
    
    Some(CurrentPeriod {
        mahadasha: CurrentMahadasha {
            planet: parent_mahadasha.planet,
            start: parent_mahadasha.start_date,
            end: parent_mahadasha.end_date,
            years: parent_mahadasha.duration_years,
        },
        antardasha: CurrentAntardasha {
            planet: parent_antardasha.planet,
            start: parent_antardasha.start_date,
            end: parent_antardasha.end_date,
            years: parent_antardasha.duration_years,
        },
        pratyantardasha: CurrentPratyantardasha {
            planet: active_pratyantardasha.planet,
            start: active_pratyantardasha.start_date,
            end: active_pratyantardasha.end_date,
            days: active_pratyantardasha.duration_days,
        },
        current_time,
    })
}

/// W1-S6-09: Calculate next N transitions (at all 3 levels)
///
/// # Arguments
/// * `mahadashas` - Complete timeline with all nested periods
/// * `current_time` - Starting time for transition calculation
/// * `count` - Number of transitions to return
///
/// # Returns
/// Vec of upcoming transitions, chronologically ordered
///
/// # Algorithm
/// 1. Find current position via find_current_period
/// 2. Iterate forward through periods
/// 3. Detect transitions at all 3 levels (Mahadasha > Antardasha > Pratyantardasha)
/// 4. Return first N transitions
pub fn calculate_upcoming_transitions(
    mahadashas: &[Mahadasha],
    current_time: DateTime<Utc>,
    count: usize
) -> Vec<crate::models::UpcomingTransition> {
    use crate::models::{UpcomingTransition, TransitionType};
    
    let mut transitions = Vec::new();
    
    // Find current position
    let current = match find_current_period(mahadashas, current_time) {
        Some(cp) => cp,
        None => return transitions,
    };
    
    // Flatten timeline with parent references
    let mut all_periods: Vec<(&Pratyantardasha, &crate::models::Antardasha, &Mahadasha)> = Vec::new();
    
    for mahadasha in mahadashas {
        for antardasha in &mahadasha.antardashas {
            for pratyantardasha in &antardasha.pratyantardashas {
                all_periods.push((pratyantardasha, antardasha, mahadasha));
            }
        }
    }
    
    // Find current index
    let current_idx = all_periods.iter().position(|(p, _, _)| 
        p.start_date == current.pratyantardasha.start
    );
    
    let current_idx = match current_idx {
        Some(idx) => idx,
        None => return transitions,
    };
    
    // Iterate forward through periods
    let (mut prev_pratyantardasha, mut prev_antardasha, mut prev_mahadasha) = all_periods[current_idx];
    
    for i in (current_idx + 1)..all_periods.len() {
        if transitions.len() >= count {
            break;
        }
        
        let (period, antardasha, mahadasha) = all_periods[i];
        
        // Check for Mahadasha transition (highest priority)
        if mahadasha.planet != prev_mahadasha.planet {
            transitions.push(UpcomingTransition {
                transition_type: TransitionType::Mahadasha,
                from_planet: prev_mahadasha.planet,
                to_planet: mahadasha.planet,
                transition_date: mahadasha.start_date,
                days_until: (mahadasha.start_date - current_time).num_days(),
            });
            prev_mahadasha = mahadasha;
        }
        // Check for Antardasha transition
        else if antardasha.planet != prev_antardasha.planet {
            transitions.push(UpcomingTransition {
                transition_type: TransitionType::Antardasha,
                from_planet: prev_antardasha.planet,
                to_planet: antardasha.planet,
                transition_date: antardasha.start_date,
                days_until: (antardasha.start_date - current_time).num_days(),
            });
            prev_antardasha = antardasha;
        }
        // Pratyantardasha transition
        else {
            transitions.push(UpcomingTransition {
                transition_type: TransitionType::Pratyantardasha,
                from_planet: prev_pratyantardasha.planet,
                to_planet: period.planet,
                transition_date: period.start_date,
                days_until: (period.start_date - current_time).num_days(),
            });
        }
        
        prev_pratyantardasha = period;
    }
    
    transitions
}

/// W1-S6-10: Enrich period with planetary qualities
///
/// # Arguments
/// * `mahadasha_planet` - Planet ruling the Mahadasha
/// * `antardasha_planet` - Planet ruling the Antardasha
/// * `pratyantardasha_planet` - Planet ruling the Pratyantardasha
///
/// # Returns
/// Combined enrichment with themes, life areas, opportunities, and challenges
pub fn enrich_period_with_qualities(
    mahadasha_planet: &VedicPlanet,
    antardasha_planet: &VedicPlanet,
    pratyantardasha_planet: &VedicPlanet,
) -> crate::models::PeriodEnrichment {
    use crate::models::PeriodEnrichment;
    use crate::wisdom_data::PLANETARY_PERIOD_QUALITIES;
    
    let maha_qualities = PLANETARY_PERIOD_QUALITIES.get(mahadasha_planet)
        .expect("Mahadasha planet qualities not found");
    let antar_qualities = PLANETARY_PERIOD_QUALITIES.get(antardasha_planet)
        .expect("Antardasha planet qualities not found");
    let pratyantar_qualities = PLANETARY_PERIOD_QUALITIES.get(pratyantardasha_planet)
        .expect("Pratyantardasha planet qualities not found");
    
    PeriodEnrichment {
        mahadasha_themes: maha_qualities.themes.clone(),
        antardasha_themes: antar_qualities.themes.clone(),
        pratyantardasha_themes: pratyantar_qualities.themes.clone(),
        combined_description: format!(
            "During {}'s Mahadasha ({}), within {}'s Antardasha ({}), the current {} Pratyantardasha brings {}. {}",
            mahadasha_planet.as_str(),
            maha_qualities.themes.join(", "),
            antardasha_planet.as_str(),
            antar_qualities.themes.join(", "),
            pratyantardasha_planet.as_str(),
            pratyantar_qualities.themes.join(", "),
            pratyantar_qualities.description,
        ),
        life_areas: pratyantar_qualities.life_areas.clone(),
        opportunities: pratyantar_qualities.opportunities.clone(),
        challenges: pratyantar_qualities.challenges.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_nakshatra_count() {
        assert_eq!(NAKSHATRAS.len(), 27);
    }

    #[test]
    fn test_nakshatra_coverage() {
        // First nakshatra starts at 0°
        assert_eq!(NAKSHATRAS[0].start_degree, 0.0);
        assert_eq!(NAKSHATRAS[0].name, "Ashwini");
        
        // Last nakshatra ends at 360°
        assert_eq!(NAKSHATRAS[26].end_degree, 360.0);
        assert_eq!(NAKSHATRAS[26].name, "Revati");
    }

    #[test]
    fn test_nakshatra_ruling_planets() {
        // Verify the pattern: Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury (repeats 3 times)
        assert_eq!(NAKSHATRAS[0].ruling_planet, VedicPlanet::Ketu);   // Ashwini
        assert_eq!(NAKSHATRAS[1].ruling_planet, VedicPlanet::Venus);  // Bharani
        assert_eq!(NAKSHATRAS[2].ruling_planet, VedicPlanet::Sun);    // Krittika
        assert_eq!(NAKSHATRAS[9].ruling_planet, VedicPlanet::Ketu);   // Magha (10th)
        assert_eq!(NAKSHATRAS[18].ruling_planet, VedicPlanet::Ketu);  // Mula (19th)
    }

    #[test]
    fn test_get_nakshatra_from_longitude() {
        // Test Magha (10th nakshatra): 120° - 133.333°
        let nak = get_nakshatra_from_longitude(125.0);
        assert_eq!(nak.number, 10);
        assert_eq!(nak.name, "Magha");
        assert_eq!(nak.ruling_planet, VedicPlanet::Ketu);
        
        // Test Ashwini (1st): 0° - 13.333°
        let nak = get_nakshatra_from_longitude(5.0);
        assert_eq!(nak.number, 1);
        assert_eq!(nak.name, "Ashwini");
        
        // Test Revati (27th): 346.667° - 360°
        let nak = get_nakshatra_from_longitude(355.0);
        assert_eq!(nak.number, 27);
        assert_eq!(nak.name, "Revati");
    }

    #[test]
    fn test_dasha_balance_calculation() {
        // Test case from spec: Moon at 125° in Magha
        let nakshatra = get_nakshatra(10).unwrap(); // Magha
        let balance = calculate_dasha_balance(125.0, nakshatra);
        
        // Expected: (133.333 - 125) / 13.333 = 0.625
        // Ketu period = 7 years
        // Balance = 0.625 * 7 = 4.375 years
        assert!((balance - 4.375).abs() < 0.01, "Balance should be ~4.375 years, got {}", balance);
    }

    #[test]
    fn test_dasha_balance_at_start() {
        // Moon exactly at start of nakshatra = full period remaining
        let nakshatra = get_nakshatra(10).unwrap(); // Magha starts at 120°
        let balance = calculate_dasha_balance(120.0, nakshatra);
        
        // Should be full 7 years for Ketu
        assert!((balance - 7.0).abs() < 0.01, "Balance should be ~7.0 years, got {}", balance);
    }

    #[test]
    fn test_dasha_balance_at_end() {
        // Moon near end of nakshatra = minimal period remaining
        let nakshatra = get_nakshatra(10).unwrap(); // Magha ends at 133.333°
        let balance = calculate_dasha_balance(133.3, nakshatra);
        
        // Should be very small
        assert!(balance < 0.1, "Balance should be near 0, got {}", balance);
    }

    #[test]
    fn test_120_year_cycle_total() {
        // Generate Mahadashas and verify total = 120 years
        let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
        let starting_planet = VedicPlanet::Ketu;
        let balance = 4.375; // From example
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        
        assert_eq!(mahadashas.len(), 9);
        
        // Verify first has balance duration
        assert!((mahadashas[0].duration_years - 4.375).abs() < 0.01);
        
        // Calculate total duration
        let total: f64 = mahadashas.iter().map(|m| m.duration_years).sum();
        assert!((total - 120.0).abs() < 0.1, "Total should be 120 years, got {}", total);
    }

    #[test]
    fn test_mahadasha_sequence() {
        // Test planet sequence cycles correctly
        let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
        let starting_planet = VedicPlanet::Jupiter;
        let balance = 10.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        
        // First should be Jupiter
        assert_eq!(mahadashas[0].planet, VedicPlanet::Jupiter);
        
        // Second should be Saturn (Jupiter.next_planet())
        assert_eq!(mahadashas[1].planet, VedicPlanet::Saturn);
        
        // Continue through cycle
        assert_eq!(mahadashas[2].planet, VedicPlanet::Mercury);
        assert_eq!(mahadashas[3].planet, VedicPlanet::Ketu);
    }

    #[test]
    fn test_mahadasha_date_progression() {
        // Verify dates progress correctly
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Sun;
        let balance = 3.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        
        // First ends after balance period
        let first_end = birth + Duration::days((3.0 * 365.25) as i64);
        assert_eq!(mahadashas[0].end_date, first_end);
        
        // Second starts where first ends
        assert_eq!(mahadashas[1].start_date, mahadashas[0].end_date);
        
        // All should be continuous
        for i in 1..mahadashas.len() {
            assert_eq!(mahadashas[i].start_date, mahadashas[i-1].end_date);
        }
    }

    #[test]
    fn test_planet_period_years() {
        assert_eq!(VedicPlanet::Sun.period_years(), 6);
        assert_eq!(VedicPlanet::Moon.period_years(), 10);
        assert_eq!(VedicPlanet::Mars.period_years(), 7);
        assert_eq!(VedicPlanet::Rahu.period_years(), 18);
        assert_eq!(VedicPlanet::Jupiter.period_years(), 16);
        assert_eq!(VedicPlanet::Saturn.period_years(), 19);
        assert_eq!(VedicPlanet::Mercury.period_years(), 17);
        assert_eq!(VedicPlanet::Ketu.period_years(), 7);
        assert_eq!(VedicPlanet::Venus.period_years(), 20);
        
        // Total should be 120
        let total = VedicPlanet::Sun.period_years()
            + VedicPlanet::Moon.period_years()
            + VedicPlanet::Mars.period_years()
            + VedicPlanet::Rahu.period_years()
            + VedicPlanet::Jupiter.period_years()
            + VedicPlanet::Saturn.period_years()
            + VedicPlanet::Mercury.period_years()
            + VedicPlanet::Ketu.period_years()
            + VedicPlanet::Venus.period_years();
        assert_eq!(total, 120);
    }

    // ============ W1-S6-06 TESTS: ANTARDASHA ============

    #[test]
    fn test_antardasha_subdivision_count() {
        // Create test Mahadasha (Jupiter, 16 years)
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let end = birth + Duration::days((16.0 * 365.25) as i64);
        let maha = Mahadasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years: 16.0,
            antardashas: Vec::new(),
            qualities: crate::models::PlanetaryQualities {
                themes: vec![],
                qualities: vec![],
                element: String::new(),
                description: String::new(),
                consciousness_lessons: vec![],
                optimal_practices: vec![],
                challenges: vec![],
            },
        };
        
        let antardashas = calculate_antardashas(&maha);
        
        // Verify 9 Antardashas
        assert_eq!(antardashas.len(), 9, "Should have exactly 9 Antardashas");
    }

    #[test]
    fn test_antardasha_starts_with_mahadasha_lord() {
        // Create test Mahadasha (Jupiter)
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let end = birth + Duration::days((16.0 * 365.25) as i64);
        let maha = Mahadasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years: 16.0,
            antardashas: Vec::new(),
            qualities: crate::models::PlanetaryQualities {
                themes: vec![],
                qualities: vec![],
                element: String::new(),
                description: String::new(),
                consciousness_lessons: vec![],
                optimal_practices: vec![],
                challenges: vec![],
            },
        };
        
        let antardashas = calculate_antardashas(&maha);
        
        // Verify first is Jupiter (starts with Mahadasha lord)
        assert_eq!(antardashas[0].planet, VedicPlanet::Jupiter);
        
        // Verify sequence follows Vimshottari order
        assert_eq!(antardashas[1].planet, VedicPlanet::Saturn);
        assert_eq!(antardashas[2].planet, VedicPlanet::Mercury);
        assert_eq!(antardashas[3].planet, VedicPlanet::Ketu);
    }

    #[test]
    fn test_antardasha_duration_formula() {
        // Jupiter Mahadasha: 16 years
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let end = birth + Duration::days((16.0 * 365.25) as i64);
        let maha = Mahadasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years: 16.0,
            antardashas: Vec::new(),
            qualities: crate::models::PlanetaryQualities {
                themes: vec![],
                qualities: vec![],
                element: String::new(),
                description: String::new(),
                consciousness_lessons: vec![],
                optimal_practices: vec![],
                challenges: vec![],
            },
        };
        
        let antardashas = calculate_antardashas(&maha);
        
        // Verify duration formula: (16 × 16) / 120 = 2.133 years
        let jupiter_antar = &antardashas[0];
        let expected_duration = (16.0 * 16.0) / 120.0;
        assert!((jupiter_antar.duration_years - expected_duration).abs() < 0.001,
                "Jupiter Antardasha should be ~{} years, got {}", expected_duration, jupiter_antar.duration_years);
        
        // Verify Saturn Antardasha: (16 × 19) / 120 = 2.533 years
        let saturn_antar = &antardashas[1];
        let expected_saturn = (16.0 * 19.0) / 120.0;
        assert!((saturn_antar.duration_years - expected_saturn).abs() < 0.001,
                "Saturn Antardasha should be ~{} years, got {}", expected_saturn, saturn_antar.duration_years);
    }

    #[test]
    fn test_antardasha_durations_sum_to_mahadasha() {
        // Create test Mahadasha
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let end = birth + Duration::days((16.0 * 365.25) as i64);
        let maha = Mahadasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years: 16.0,
            antardashas: Vec::new(),
            qualities: crate::models::PlanetaryQualities {
                themes: vec![],
                qualities: vec![],
                element: String::new(),
                description: String::new(),
                consciousness_lessons: vec![],
                optimal_practices: vec![],
                challenges: vec![],
            },
        };
        
        let antardashas = calculate_antardashas(&maha);
        
        // Verify durations sum to Mahadasha duration
        let total: f64 = antardashas.iter().map(|a| a.duration_years).sum();
        assert!((total - 16.0).abs() < 0.01, 
                "Antardasha durations should sum to 16.0 years, got {}", total);
    }

    #[test]
    fn test_antardasha_date_continuity() {
        // Create test Mahadasha
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let end = birth + Duration::days((16.0 * 365.25) as i64);
        let maha = Mahadasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years: 16.0,
            antardashas: Vec::new(),
            qualities: crate::models::PlanetaryQualities {
                themes: vec![],
                qualities: vec![],
                element: String::new(),
                description: String::new(),
                consciousness_lessons: vec![],
                optimal_practices: vec![],
                challenges: vec![],
            },
        };
        
        let antardashas = calculate_antardashas(&maha);
        
        // Verify continuity (no gaps or overlaps)
        for i in 0..8 {
            assert_eq!(antardashas[i].end_date, antardashas[i+1].start_date,
                      "Antardasha {} end should match Antardasha {} start", i, i+1);
        }
        
        // Verify first starts at Mahadasha start
        assert_eq!(antardashas[0].start_date, maha.start_date);
        
        // Verify last ends at Mahadasha end (within 1 day tolerance for rounding)
        let last_end = antardashas[8].end_date;
        let diff_days = (last_end - maha.end_date).num_days().abs();
        assert!(diff_days <= 1, "Last Antardasha should end at Mahadasha end, diff: {} days", diff_days);
    }

    // ============ W1-S6-07 TESTS: PRATYANTARDASHA ============

    #[test]
    fn test_pratyantardasha_subdivision_count() {
        use crate::models::Antardasha;
        
        // Create test Antardasha (Jupiter-Jupiter, 2.133 years)
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let duration_years = (16.0 * 16.0) / 120.0; // 2.133
        let end = birth + Duration::days((duration_years * 365.25) as i64);
        
        let antar = Antardasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years,
            pratyantardashas: vec![],
        };
        
        let pratyantardashas = calculate_pratyantardashas(&antar);
        
        // Verify 9 Pratyantardashas
        assert_eq!(pratyantardashas.len(), 9, "Should have exactly 9 Pratyantardashas");
    }

    #[test]
    fn test_pratyantardasha_starts_with_antardasha_lord() {
        use crate::models::Antardasha;
        
        // Create test Antardasha (Jupiter-Jupiter)
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let duration_years = (16.0 * 16.0) / 120.0;
        let end = birth + Duration::days((duration_years * 365.25) as i64);
        
        let antar = Antardasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years,
            pratyantardashas: vec![],
        };
        
        let pratyantardashas = calculate_pratyantardashas(&antar);
        
        // Verify first is Jupiter (starts with Antardasha lord)
        assert_eq!(pratyantardashas[0].planet, VedicPlanet::Jupiter);
        
        // Verify sequence follows Vimshottari order
        assert_eq!(pratyantardashas[1].planet, VedicPlanet::Saturn);
        assert_eq!(pratyantardashas[2].planet, VedicPlanet::Mercury);
    }

    #[test]
    fn test_pratyantardasha_duration_formula() {
        use crate::models::Antardasha;
        
        // Jupiter-Jupiter Antardasha: 2.133 years
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let duration_years = (16.0 * 16.0) / 120.0; // 2.133
        let end = birth + Duration::days((duration_years * 365.25) as i64);
        
        let antar = Antardasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years,
            pratyantardashas: vec![],
        };
        
        let pratyantardashas = calculate_pratyantardashas(&antar);
        
        // Verify duration formula: (2.133 × 16) / 120 = 0.284 years (~104 days)
        let jupiter_pratyantar = &pratyantardashas[0];
        let expected_years = (duration_years * 16.0) / 120.0;
        let expected_days = expected_years * 365.25;
        
        assert!((jupiter_pratyantar.duration_days - expected_days).abs() < 1.0,
                "Jupiter Pratyantardasha should be ~{} days, got {}", expected_days, jupiter_pratyantar.duration_days);
    }

    #[test]
    fn test_pratyantardasha_date_continuity() {
        use crate::models::Antardasha;
        
        // Create test Antardasha
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let duration_years = (16.0 * 16.0) / 120.0;
        let end = birth + Duration::days((duration_years * 365.25) as i64);
        
        let antar = Antardasha {
            planet: VedicPlanet::Jupiter,
            start_date: birth,
            end_date: end,
            duration_years,
            pratyantardashas: vec![],
        };
        
        let pratyantardashas = calculate_pratyantardashas(&antar);
        
        // Verify continuity (no gaps or overlaps)
        for i in 0..8 {
            assert_eq!(pratyantardashas[i].end_date, pratyantardashas[i+1].start_date,
                      "Pratyantardasha {} end should match Pratyantardasha {} start", i, i+1);
        }
        
        // Verify first starts at Antardasha start
        assert_eq!(pratyantardashas[0].start_date, antar.start_date);
    }

    // ============ COMPLETE TIMELINE TESTS ============

    #[test]
    fn test_complete_timeline_structure() {
        // Generate complete chart
        let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
        let mahadashas = calculate_mahadashas(birth, VedicPlanet::Ketu, 4.375);
        
        let complete = calculate_complete_timeline(mahadashas);
        
        // Verify structure
        assert_eq!(complete.len(), 9, "Should have 9 Mahadashas");
        
        for (i, maha) in complete.iter().enumerate() {
            assert_eq!(maha.antardashas.len(), 9, 
                      "Mahadasha {} should have 9 Antardashas", i);
            
            for (j, antar) in maha.antardashas.iter().enumerate() {
                assert_eq!(antar.pratyantardashas.len(), 9,
                          "Mahadasha {} Antardasha {} should have 9 Pratyantardashas", i, j);
            }
        }
    }

    #[test]
    fn test_complete_timeline_total_pratyantardashas() {
        // Generate complete chart
        let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
        let mahadashas = calculate_mahadashas(birth, VedicPlanet::Jupiter, 10.0);
        
        let complete = calculate_complete_timeline(mahadashas);
        
        // Count total Pratyantardashas
        let total_pratyantar: usize = complete.iter()
            .map(|maha| maha.antardashas.iter()
                .map(|antar| antar.pratyantardashas.len())
                .sum::<usize>())
            .sum();
        
        // Should be 9 × 9 × 9 = 729
        assert_eq!(total_pratyantar, 729, "Should have 729 total Pratyantardashas (9×9×9)");
    }

    #[test]
    fn test_complete_timeline_nested_continuity() {
        // Generate complete chart with simple case
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let mahadashas = calculate_mahadashas(birth, VedicPlanet::Sun, 6.0); // Full Sun period
        
        let complete = calculate_complete_timeline(mahadashas);
        
        // Verify first Mahadasha's Antardashas are continuous
        let first_maha = &complete[0];
        for i in 0..8 {
            assert_eq!(first_maha.antardashas[i].end_date, 
                      first_maha.antardashas[i+1].start_date,
                      "Antardashas should be continuous");
        }
        
        // Verify first Antardasha's Pratyantardashas are continuous
        let first_antar = &first_maha.antardashas[0];
        for i in 0..8 {
            assert_eq!(first_antar.pratyantardashas[i].end_date,
                      first_antar.pratyantardashas[i+1].start_date,
                      "Pratyantardashas should be continuous");
        }
    }

    #[test]
    fn test_partial_mahadasha_subdivisions() {
        // Test with partial first Mahadasha (balance case)
        let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
        let balance = 4.375; // Partial Ketu Mahadasha
        let mahadashas = calculate_mahadashas(birth, VedicPlanet::Ketu, balance);
        
        let complete = calculate_complete_timeline(mahadashas);
        
        // First Mahadasha should have 9 Antardashas even though partial
        assert_eq!(complete[0].antardashas.len(), 9);
        
        // Verify Antardashas sum to partial Mahadasha duration
        let antar_total: f64 = complete[0].antardashas.iter()
            .map(|a| a.duration_years)
            .sum();
        assert!((antar_total - balance).abs() < 0.01,
                "Antardasha durations should sum to balance period");
    }

    // ============ W1-S6-08 TESTS: CURRENT PERIOD DETECTION ============

    #[test]
    fn test_find_current_period_basic() {
        // Setup: Create chart with birth date 1985-06-15
        let birth = Utc.with_ymd_and_hms(1985, 6, 15, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Ketu;
        let balance = 4.375;
        
        // Generate complete timeline
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        // Query time: 2 years after birth (still in first Mahadasha)
        let query_time = birth + Duration::days((2.0 * 365.25) as i64);
        
        let current = find_current_period(&complete, query_time).unwrap();
        
        // Should be within Ketu Mahadasha
        assert_eq!(current.mahadasha.planet, VedicPlanet::Ketu);
        
        // current_time should be between pratyantardasha dates
        assert!(query_time >= current.pratyantardasha.start);
        assert!(query_time <= current.pratyantardasha.end);
        
        // Verify nested consistency: pratyantardasha period falls within antardasha
        assert!(current.pratyantardasha.start >= current.antardasha.start);
        assert!(current.pratyantardasha.end <= current.antardasha.end);
        
        // Verify antardasha falls within mahadasha
        assert!(current.antardasha.start >= current.mahadasha.start);
        assert!(current.antardasha.end <= current.mahadasha.end);
    }

    #[test]
    fn test_find_current_period_at_boundary() {
        // Test at exact boundary between periods
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Sun;
        let balance = 6.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        // Query at exact start of second Mahadasha
        let second_maha_start = complete[1].start_date;
        let current = find_current_period(&complete, second_maha_start).unwrap();
        
        // Should detect second Mahadasha
        assert_eq!(current.mahadasha.planet, complete[1].planet);
        assert_eq!(current.mahadasha.start, second_maha_start);
    }

    #[test]
    fn test_binary_search_efficiency() {
        // Verify binary search works with full 729 periods
        let birth = Utc.with_ymd_and_hms(1990, 1, 1, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Mars;
        let balance = 5.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        // Count total pratyantardashas: should be 9 × 9 × 9 = 729
        let mut total_periods = 0;
        for maha in &complete {
            for antar in &maha.antardashas {
                total_periods += antar.pratyantardashas.len();
            }
        }
        assert_eq!(total_periods, 729);
        
        // Test search at various points
        let test_times = vec![
            birth + Duration::days(365),          // 1 year in
            birth + Duration::days(3650),         // 10 years in
            birth + Duration::days(18250),        // 50 years in
            birth + Duration::days(36500),        // 100 years in
        ];
        
        for query_time in test_times {
            let result = find_current_period(&complete, query_time);
            // Should find a period (within 120-year cycle)
            if query_time < complete.last().unwrap().end_date {
                assert!(result.is_some(), "Should find period for time {:?}", query_time);
            }
        }
    }

    #[test]
    fn test_current_period_time_within_range() {
        // Verify current_time always falls within reported period
        let birth = Utc.with_ymd_and_hms(2010, 3, 15, 12, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Venus;
        let balance = 15.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        // Test multiple random times
        for days_offset in [100, 500, 1000, 2000, 5000] {
            let query_time = birth + Duration::days(days_offset);
            
            if let Some(current) = find_current_period(&complete, query_time) {
                // Verify time containment at all levels
                assert!(current.current_time >= current.pratyantardasha.start,
                       "Current time should be after pratyantardasha start");
                assert!(current.current_time <= current.pratyantardasha.end,
                       "Current time should be before pratyantardasha end");
                       
                assert!(current.current_time >= current.antardasha.start);
                assert!(current.current_time <= current.antardasha.end);
                
                assert!(current.current_time >= current.mahadasha.start);
                assert!(current.current_time <= current.mahadasha.end);
            }
        }
    }

    // ============ W1-S6-09 TESTS: UPCOMING TRANSITIONS ============

    #[test]
    fn test_upcoming_transitions_chronological_order() {
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Jupiter;
        let balance = 10.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        let current_time = birth + Duration::days(365);
        let transitions = calculate_upcoming_transitions(&complete, current_time, 10);
        
        // Should get multiple transitions
        assert!(transitions.len() > 0, "Should have upcoming transitions");
        assert!(transitions.len() <= 10, "Should not exceed requested count");
        
        // Verify chronological order
        for i in 1..transitions.len() {
            assert!(transitions[i].transition_date > transitions[i-1].transition_date,
                   "Transitions should be in chronological order");
        }
        
        // Verify days_until is positive
        for t in &transitions {
            assert!(t.days_until > 0, "days_until should be positive for future transitions");
        }
    }

    #[test]
    fn test_transition_hierarchy() {
        // Mahadasha transitions should be rarest
        // Antardasha transitions more common
        // Pratyantardasha transitions most common
        
        let birth = Utc.with_ymd_and_hms(1995, 7, 20, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Moon;
        let balance = 8.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        let current_time = birth + Duration::days(100);
        let transitions = calculate_upcoming_transitions(&complete, current_time, 100);
        
        use crate::models::TransitionType;
        
        let mahadasha_count = transitions.iter()
            .filter(|t| matches!(t.transition_type, TransitionType::Mahadasha))
            .count();
        let antardasha_count = transitions.iter()
            .filter(|t| matches!(t.transition_type, TransitionType::Antardasha))
            .count();
        let pratyantardasha_count = transitions.iter()
            .filter(|t| matches!(t.transition_type, TransitionType::Pratyantardasha))
            .count();
        
        // Verify hierarchy: pratyantardasha > antardasha > mahadasha
        assert!(pratyantardasha_count > antardasha_count,
               "Should have more Pratyantardasha transitions than Antardasha");
        assert!(antardasha_count >= mahadasha_count,
               "Should have more or equal Antardasha transitions than Mahadasha");
    }

    #[test]
    fn test_transition_days_until_accuracy() {
        let birth = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Saturn;
        let balance = 12.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        let current_time = birth + Duration::days(500);
        let transitions = calculate_upcoming_transitions(&complete, current_time, 5);
        
        // Verify days_until calculation
        for t in &transitions {
            let expected_days = (t.transition_date - current_time).num_days();
            assert_eq!(t.days_until, expected_days,
                      "days_until should match date difference");
            assert!(t.days_until > 0, "Future transitions should have positive days_until");
        }
    }

    #[test]
    fn test_transition_count_limit() {
        // Verify we get exactly the requested count (or less if reaching end)
        let birth = Utc.with_ymd_and_hms(2005, 6, 10, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Mercury;
        let balance = 13.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        let current_time = birth + Duration::days(1000);
        
        // Request different counts
        for count in [1, 5, 10, 20, 50] {
            let transitions = calculate_upcoming_transitions(&complete, current_time, count);
            assert!(transitions.len() <= count,
                   "Should not exceed requested count of {}", count);
        }
    }

    #[test]
    fn test_transition_planet_accuracy() {
        // Verify from_planet and to_planet are correctly identified
        let birth = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let starting_planet = VedicPlanet::Rahu;
        let balance = 15.0;
        
        let mahadashas = calculate_mahadashas(birth, starting_planet, balance);
        let complete = calculate_complete_timeline(mahadashas);
        
        let current_time = birth + Duration::days(200);
        let transitions = calculate_upcoming_transitions(&complete, current_time, 3);
        
        // First transition's from_planet should match current period
        if let Some(first) = transitions.first() {
            let current = find_current_period(&complete, current_time).unwrap();
            
            // Depending on transition level, verify appropriate planet
            use crate::models::TransitionType;
            match first.transition_type {
                TransitionType::Pratyantardasha => {
                    assert_eq!(first.from_planet, current.pratyantardasha.planet);
                },
                TransitionType::Antardasha => {
                    assert_eq!(first.from_planet, current.antardasha.planet);
                },
                TransitionType::Mahadasha => {
                    assert_eq!(first.from_planet, current.mahadasha.planet);
                },
            }
        }
    }
}
