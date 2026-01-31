//! Static wisdom data loaded at startup

use crate::models::{Nakshatra, PlanetaryQualities, PlanetaryPeriodQualities, VedicPlanet};
use crate::wisdom::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// 9 planetary periods with durations
    pub static ref VIMSHOTTARI_PERIODS: HashMap<VedicPlanet, u8> = load_periods();

    /// 27 nakshatras with ruling planets and details
    pub static ref NAKSHATRAS: Vec<Nakshatra> = load_nakshatras();

    /// 9 planetary qualities (themes, lessons, practices, challenges)
    pub static ref PLANETARY_QUALITIES: HashMap<VedicPlanet, PlanetaryQualities> = load_qualities();

    /// Nakshatra number to ruling planet mapping
    pub static ref NAKSHATRA_RULERS: HashMap<u8, VedicPlanet> = load_nakshatra_rulers();

    /// Planetary cycle order
    pub static ref PLANETARY_ORDER: Vec<VedicPlanet> = vec![
        VedicPlanet::Sun,
        VedicPlanet::Moon,
        VedicPlanet::Mars,
        VedicPlanet::Rahu,
        VedicPlanet::Jupiter,
        VedicPlanet::Saturn,
        VedicPlanet::Mercury,
        VedicPlanet::Ketu,
        VedicPlanet::Venus,
    ];

    /// Planetary period qualities for consciousness work
    pub static ref PLANETARY_PERIOD_QUALITIES: HashMap<VedicPlanet, PlanetaryPeriodQualities> = {
        let mut map = HashMap::new();
        
        map.insert(VedicPlanet::Sun, PlanetaryPeriodQualities {
            planet: VedicPlanet::Sun,
            themes: vec![
                "Self-expression".to_string(),
                "Authority".to_string(),
                "Vitality".to_string(),
                "Recognition".to_string(),
            ],
            life_areas: vec![
                "Career advancement".to_string(),
                "Leadership roles".to_string(),
                "Public visibility".to_string(),
                "Father relationships".to_string(),
                "Government affairs".to_string(),
            ],
            challenges: vec![
                "Ego inflation".to_string(),
                "Conflicts with authority".to_string(),
                "Overwork and burnout".to_string(),
                "Pride and arrogance".to_string(),
            ],
            opportunities: vec![
                "Step into leadership".to_string(),
                "Claim your power".to_string(),
                "Shine your light".to_string(),
                "Build confidence".to_string(),
            ],
            description: "The Sun's period brings themes of self-expression, vitality, and authority. This is a time to step into your power, take on leadership roles, and let your authentic light shine. The soul seeks recognition and purpose. Watch for ego inflation and conflicts with authority figures. Use this time to strengthen your core identity and radiate your unique gifts into the world.".to_string(),
        });
        
        map.insert(VedicPlanet::Moon, PlanetaryPeriodQualities {
            planet: VedicPlanet::Moon,
            themes: vec![
                "Emotions".to_string(),
                "Nurturing".to_string(),
                "Home and family".to_string(),
                "Intuition".to_string(),
            ],
            life_areas: vec![
                "Domestic life".to_string(),
                "Mother relationships".to_string(),
                "Emotional security".to_string(),
                "Creative expression".to_string(),
                "Public connection".to_string(),
            ],
            challenges: vec![
                "Emotional overwhelm".to_string(),
                "Mood swings".to_string(),
                "Dependency issues".to_string(),
                "Clinging to the past".to_string(),
            ],
            opportunities: vec![
                "Deepen emotional intelligence".to_string(),
                "Nurture yourself and others".to_string(),
                "Trust your intuition".to_string(),
                "Create emotional safety".to_string(),
            ],
            description: "The Moon's period emphasizes emotional depth, nurturing, and domestic harmony. This is a time to tend to your inner world, strengthen family bonds, and trust your intuitive guidance. The mind becomes more sensitive and receptive. Watch for emotional overwhelm and dependency patterns. Use this period to honor your feelings, create a sense of home, and connect with the rhythms of your inner tides.".to_string(),
        });
        
        map.insert(VedicPlanet::Mars, PlanetaryPeriodQualities {
            planet: VedicPlanet::Mars,
            themes: vec![
                "Action".to_string(),
                "Courage".to_string(),
                "Conflict".to_string(),
                "Energy".to_string(),
            ],
            life_areas: vec![
                "Physical activity".to_string(),
                "Competition".to_string(),
                "Property and land".to_string(),
                "Brother relationships".to_string(),
                "Technical skills".to_string(),
            ],
            challenges: vec![
                "Anger and aggression".to_string(),
                "Impulsiveness".to_string(),
                "Accidents and injuries".to_string(),
                "Conflict escalation".to_string(),
            ],
            opportunities: vec![
                "Take decisive action".to_string(),
                "Build physical strength".to_string(),
                "Assert boundaries".to_string(),
                "Channel passion constructively".to_string(),
            ],
            description: "Mars's period brings dynamic energy, courage, and the drive to take action. This is a time of initiative, physical pursuits, and facing challenges head-on. The warrior within awakens. Watch for anger, impulsiveness, and unnecessary conflicts. Use this fiery period to build strength, assert healthy boundaries, accomplish ambitious goals, and transform raw energy into focused achievement. Act with courage but temper with wisdom.".to_string(),
        });
        
        map.insert(VedicPlanet::Mercury, PlanetaryPeriodQualities {
            planet: VedicPlanet::Mercury,
            themes: vec![
                "Communication".to_string(),
                "Learning".to_string(),
                "Business".to_string(),
                "Intellect".to_string(),
            ],
            life_areas: vec![
                "Education and study".to_string(),
                "Writing and speaking".to_string(),
                "Commerce and trade".to_string(),
                "Technology".to_string(),
                "Short travel".to_string(),
            ],
            challenges: vec![
                "Mental restlessness".to_string(),
                "Overthinking".to_string(),
                "Superficiality".to_string(),
                "Communication breakdowns".to_string(),
            ],
            opportunities: vec![
                "Develop new skills".to_string(),
                "Improve communication".to_string(),
                "Start a business".to_string(),
                "Connect with siblings".to_string(),
            ],
            description: "Mercury's period emphasizes intellect, communication, and versatile learning. This is a time to develop skills, engage in commerce, and refine your powers of expression. The mind becomes sharp and curious. Watch for mental restlessness, overthinking, and scattered focus. Use this mercurial period to read, write, study, network, negotiate, and master the art of clear communication. Curiosity becomes your greatest teacher.".to_string(),
        });
        
        map.insert(VedicPlanet::Jupiter, PlanetaryPeriodQualities {
            planet: VedicPlanet::Jupiter,
            themes: vec![
                "Growth".to_string(),
                "Wisdom".to_string(),
                "Teaching".to_string(),
                "Expansion".to_string(),
            ],
            life_areas: vec![
                "Higher education".to_string(),
                "Long-distance travel".to_string(),
                "Teaching and mentorship".to_string(),
                "Philosophy and religion".to_string(),
                "Children".to_string(),
            ],
            challenges: vec![
                "Over-optimism".to_string(),
                "Excessive indulgence".to_string(),
                "Lack of boundaries".to_string(),
                "Idealistic expectations".to_string(),
            ],
            opportunities: vec![
                "Pursue higher learning".to_string(),
                "Become a teacher".to_string(),
                "Expand horizons".to_string(),
                "Develop faith".to_string(),
            ],
            description: "Jupiter's period brings growth, wisdom, and expansive opportunities. This is a time of learning, teaching, travel, and spiritual development. Luck and grace often flow more freely. Watch for over-optimism, excessive indulgence, and unrealistic expectations. Use this benefic period to study philosophy, share knowledge, expand your worldview, cultivate faith, and grow in both outer achievement and inner understanding. Wisdom becomes your wealth.".to_string(),
        });
        
        map.insert(VedicPlanet::Venus, PlanetaryPeriodQualities {
            planet: VedicPlanet::Venus,
            themes: vec![
                "Love".to_string(),
                "Beauty".to_string(),
                "Luxury".to_string(),
                "Relationships".to_string(),
            ],
            life_areas: vec![
                "Romantic relationships".to_string(),
                "Marriage".to_string(),
                "Arts and creativity".to_string(),
                "Comfort and luxury".to_string(),
                "Social life".to_string(),
            ],
            challenges: vec![
                "Overindulgence".to_string(),
                "Vanity".to_string(),
                "Relationship dependency".to_string(),
                "Materialism".to_string(),
            ],
            opportunities: vec![
                "Deepen love connections".to_string(),
                "Create beauty".to_string(),
                "Enjoy life's pleasures".to_string(),
                "Cultivate harmony".to_string(),
            ],
            description: "Venus's period brings themes of love, beauty, pleasure, and harmonious relationships. This is a time to enjoy life's comforts, create art, deepen romantic connections, and cultivate aesthetic refinement. The heart opens and attracts. Watch for overindulgence, vanity, and excessive attachment to material comforts. Use this graceful period to honor beauty, build meaningful relationships, express creativity, and find balance between worldly pleasure and spiritual depth.".to_string(),
        });
        
        map.insert(VedicPlanet::Saturn, PlanetaryPeriodQualities {
            planet: VedicPlanet::Saturn,
            themes: vec![
                "Discipline".to_string(),
                "Structure".to_string(),
                "Karma".to_string(),
                "Maturity".to_string(),
            ],
            life_areas: vec![
                "Career responsibility".to_string(),
                "Long-term goals".to_string(),
                "Health challenges".to_string(),
                "Service to others".to_string(),
                "Spiritual discipline".to_string(),
            ],
            challenges: vec![
                "Depression and delay".to_string(),
                "Loneliness".to_string(),
                "Physical limitations".to_string(),
                "Harsh lessons".to_string(),
            ],
            opportunities: vec![
                "Build lasting structures".to_string(),
                "Develop discipline".to_string(),
                "Face karma consciously".to_string(),
                "Cultivate patience".to_string(),
            ],
            description: "Saturn's period brings discipline, responsibility, and karmic lessons. This is a time of maturation, facing limitations, and building enduring structures through sustained effort. Growth comes through challenges. Watch for depression, isolation, and excessive restriction. Use this demanding period to develop patience, accept responsibility, honor commitments, face your shadows, and transform obstacles into stepping stones. Through discipline comes liberation. What you build now lasts.".to_string(),
        });
        
        map.insert(VedicPlanet::Rahu, PlanetaryPeriodQualities {
            planet: VedicPlanet::Rahu,
            themes: vec![
                "Ambition".to_string(),
                "Innovation".to_string(),
                "Foreign connections".to_string(),
                "Obsession".to_string(),
            ],
            life_areas: vec![
                "Unconventional paths".to_string(),
                "Technology and trends".to_string(),
                "Foreign lands".to_string(),
                "Material success".to_string(),
                "Breaking boundaries".to_string(),
            ],
            challenges: vec![
                "Obsessive desire".to_string(),
                "Deception and illusion".to_string(),
                "Ethical compromises".to_string(),
                "Restless dissatisfaction".to_string(),
            ],
            opportunities: vec![
                "Pursue unconventional goals".to_string(),
                "Innovate and experiment".to_string(),
                "Embrace foreign cultures".to_string(),
                "Break limiting patterns".to_string(),
            ],
            description: "Rahu's period brings intense ambition, innovation, and desire for worldly success. This is a time of breaking conventions, embracing foreign connections, and pursuing unconventional paths. The North Node pulls you toward destiny. Watch for obsession, deception, and ethical compromises in pursuit of goals. Use this transformative period to innovate, think outside boundaries, embrace diversity, and channel ambition toward meaningful achievement. Desire can fuel evolution or create suffering.".to_string(),
        });
        
        map.insert(VedicPlanet::Ketu, PlanetaryPeriodQualities {
            planet: VedicPlanet::Ketu,
            themes: vec![
                "Spirituality".to_string(),
                "Detachment".to_string(),
                "Past-life themes".to_string(),
                "Liberation".to_string(),
            ],
            life_areas: vec![
                "Spiritual practice".to_string(),
                "Solitude and retreat".to_string(),
                "Psychic abilities".to_string(),
                "Moksha and liberation".to_string(),
                "Ancestral healing".to_string(),
            ],
            challenges: vec![
                "Isolation".to_string(),
                "Confusion and doubt".to_string(),
                "Loss and letting go".to_string(),
                "Lack of motivation".to_string(),
            ],
            opportunities: vec![
                "Deepen spiritual practice".to_string(),
                "Release attachments".to_string(),
                "Access inner wisdom".to_string(),
                "Heal past wounds".to_string(),
            ],
            description: "Ketu's period brings spirituality, detachment, and liberation from worldly desires. This is a time of introspection, mystical experiences, and releasing what no longer serves. The South Node dissolves material attachments. Watch for isolation, confusion, and loss of direction. Use this subtle period to meditate, develop intuition, heal karmic patterns, surrender control, and discover that true freedom lies not in accumulation but in letting go. Emptiness becomes fullness.".to_string(),
        });
        
        map
    };
}

/// Initialize all wisdom data (forces lazy_static evaluation)
pub fn init_wisdom() {
    lazy_static::initialize(&VIMSHOTTARI_PERIODS);
    lazy_static::initialize(&NAKSHATRAS);
    lazy_static::initialize(&PLANETARY_QUALITIES);
    lazy_static::initialize(&NAKSHATRA_RULERS);
    lazy_static::initialize(&PLANETARY_ORDER);
    lazy_static::initialize(&PLANETARY_PERIOD_QUALITIES);
}

/// Load period durations for all 9 planets
fn load_periods() -> HashMap<VedicPlanet, u8> {
    let json_str = include_str!("../../../data/vimshottari/dasha_periods.json");
    let data: DashaPeriodsData = serde_json::from_str(json_str)
        .expect("Failed to parse dasha_periods.json");

    data.mahadasha_periods
        .iter()
        .filter_map(|(name, duration)| {
            VedicPlanet::from_str(name).map(|planet| (planet, duration.years))
        })
        .collect()
}

/// Load all 27 nakshatras
fn load_nakshatras() -> Vec<Nakshatra> {
    let json_str = include_str!("../../../data/vimshottari/nakshatras.json");
    let data: NakshatrasData = serde_json::from_str(json_str)
        .expect("Failed to parse nakshatras.json");

    let mut nakshatras: Vec<_> = data.nakshatras
        .into_iter()
        .filter_map(|(_, entry)| {
            VedicPlanet::from_str(&entry.ruling_planet).map(|planet| Nakshatra {
                number: entry.number,
                name: entry.name,
                ruling_planet: planet,
                start_degree: entry.start_degree,
                end_degree: entry.end_degree,
                deity: entry.deity,
                symbol: entry.symbol,
                qualities: entry.qualities,
                description: entry.description,
            })
        })
        .collect();

    // Sort by nakshatra number
    nakshatras.sort_by_key(|n| n.number);
    nakshatras
}

/// Load planetary qualities
fn load_qualities() -> HashMap<VedicPlanet, PlanetaryQualities> {
    let json_str = include_str!("../../../data/vimshottari/vimshottari_periods.json");
    let data: VimshottariPeriodsData = serde_json::from_str(json_str)
        .expect("Failed to parse vimshottari_periods.json");

    let mut qualities = HashMap::new();

    for (name, period_info) in data.periods {
        if let Some(planet) = VedicPlanet::from_str(&name) {
            // Get detailed qualities if available
            let quality_details = data.planetary_qualities.get(&name);

            let description = format!(
                "{} period brings themes of {}",
                name,
                period_info.themes.join(", ")
            );

            qualities.insert(
                planet,
                PlanetaryQualities {
                    themes: period_info.themes,
                    qualities: period_info.qualities,
                    element: period_info.element,
                    description,
                    consciousness_lessons: quality_details
                        .map(|q| q.consciousness_lessons.clone())
                        .unwrap_or_default(),
                    optimal_practices: quality_details
                        .map(|q| q.optimal_practices.clone())
                        .unwrap_or_default(),
                    challenges: quality_details
                        .map(|q| q.challenges.clone())
                        .unwrap_or_default(),
                },
            );
        }
    }

    qualities
}

/// Load nakshatra to ruling planet mapping
fn load_nakshatra_rulers() -> HashMap<u8, VedicPlanet> {
    let json_str = include_str!("../../../data/vimshottari/dasha_periods.json");
    let data: DashaPeriodsData = serde_json::from_str(json_str)
        .expect("Failed to parse dasha_periods.json");

    data.nakshatra_rulers
        .iter()
        .filter_map(|(num_str, planet_str)| {
            let num = num_str.parse::<u8>().ok()?;
            let planet = VedicPlanet::from_str(planet_str)?;
            Some((num, planet))
        })
        .collect()
}

/// Get nakshatra from Moon longitude (0-360°)
pub fn get_nakshatra_from_longitude(longitude: f64) -> Option<&'static Nakshatra> {
    let normalized = longitude.rem_euclid(360.0);
    NAKSHATRAS.iter().find(|nak| {
        normalized >= nak.start_degree && normalized < nak.end_degree
    })
}

/// Get nakshatra by number (1-27)
pub fn get_nakshatra_by_number(number: u8) -> Option<&'static Nakshatra> {
    NAKSHATRAS.iter().find(|nak| nak.number == number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_periods() {
        let periods = load_periods();
        assert_eq!(periods.len(), 9);
        assert_eq!(periods.get(&VedicPlanet::Sun), Some(&6));
        assert_eq!(periods.get(&VedicPlanet::Moon), Some(&10));
        assert_eq!(periods.get(&VedicPlanet::Venus), Some(&20));
    }

    #[test]
    fn test_load_nakshatras() {
        let nakshatras = load_nakshatras();
        assert_eq!(nakshatras.len(), 27);
        assert_eq!(nakshatras[0].number, 1);
        assert_eq!(nakshatras[0].name, "Ashwini");
        assert_eq!(nakshatras[26].number, 27);
        assert_eq!(nakshatras[26].name, "Revati");
    }

    #[test]
    fn test_load_qualities() {
        let qualities = load_qualities();
        assert_eq!(qualities.len(), 9);
        assert!(qualities.contains_key(&VedicPlanet::Jupiter));
        assert!(qualities.contains_key(&VedicPlanet::Saturn));
    }

    #[test]
    fn test_load_nakshatra_rulers() {
        let rulers = load_nakshatra_rulers();
        assert_eq!(rulers.len(), 27);
        assert_eq!(rulers.get(&1), Some(&VedicPlanet::Ketu));
        assert_eq!(rulers.get(&2), Some(&VedicPlanet::Venus));
        assert_eq!(rulers.get(&27), Some(&VedicPlanet::Mercury));
    }

    #[test]
    fn test_planetary_order() {
        assert_eq!(PLANETARY_ORDER.len(), 9);
        assert_eq!(PLANETARY_ORDER[0], VedicPlanet::Sun);
        assert_eq!(PLANETARY_ORDER[8], VedicPlanet::Venus);
    }

    #[test]
    fn test_vimshottari_total() {
        let total: u32 = VIMSHOTTARI_PERIODS.values().map(|&y| y as u32).sum();
        assert_eq!(total, 120);
    }
}
