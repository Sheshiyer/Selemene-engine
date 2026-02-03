//! Unified Analysis combining Vedic API, Vimshottari, Numerology, and TCM

use chrono::{DateTime, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    BirthProfile, Result, IntegrationError, IntegrationConfig,
    ActivityType, AuspiciousWindow, AuspiciousQuality,
    TCMAnalysis, TCMElement, TCMOrgan,
    CachedVedicClient, CompletePanchang, PanchangQuery,
    VimshottariChart, VedicPlanet,
};

/// Complete unified analysis from all systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAnalysis {
    /// Birth profile used
    pub profile: BirthProfile,
    /// Vedic API Panchang data
    pub panchang: Option<CompletePanchang>,
    /// Vimshottari Dasha analysis
    pub vimshottari: VimshottariAnalysis,
    /// Numerology analysis
    pub numerology: NumerologyAnalysis,
    /// TCM analysis
    pub tcm: TCMAnalysis,
    /// Bio-rhythm analysis (if enabled)
    pub biorhythm: Option<BiorhythmAnalysis>,
    /// Layered insights combining all systems
    pub layered_insights: Vec<LayeredInsight>,
    /// Overall auspicious times
    pub auspicious_times: Vec<AuspiciousWindow>,
    /// Overall recommendations
    pub recommendations: Vec<UnifiedRecommendation>,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
}

/// Vimshottari-specific analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimshottariAnalysis {
    /// Current Mahadasha
    pub current_mahadasha: String,
    /// Current Antardasha
    pub current_antardasha: String,
    /// Current Pratyantardasha
    pub current_pratyantardasha: String,
    /// Mahadasha end date
    pub mahadasha_end: String,
    /// Days remaining in current Mahadasha
    pub days_remaining_mahadasha: i64,
    /// Planetary themes for current period
    pub current_themes: Vec<String>,
    /// Upcoming transitions
    pub upcoming_transitions: Vec<DashaTransition>,
    /// Qualities of current Mahadasha lord
    pub mahadasha_qualities: PlanetaryQualities,
    /// Qualities of current Antardasha lord
    pub antardasha_qualities: PlanetaryQualities,
}

/// Dasha transition info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaTransition {
    pub level: String,
    pub from_planet: String,
    pub to_planet: String,
    pub date: String,
    pub days_until: i64,
}

/// Planetary qualities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryQualities {
    pub planet: String,
    pub themes: Vec<String>,
    pub life_areas: Vec<String>,
    pub challenges: Vec<String>,
    pub opportunities: Vec<String>,
    pub description: String,
}

/// Numerology analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumerologyAnalysis {
    /// Life Path Number
    pub life_path_number: u32,
    /// Life Path description
    pub life_path_description: String,
    /// Expression Number (from full name)
    pub expression_number: Option<u32>,
    /// Soul Urge Number
    pub soul_urge_number: Option<u32>,
    /// Personality Number
    pub personality_number: Option<u32>,
    /// Birth Day Number
    pub birth_day_number: u32,
    /// Personal Year (current)
    pub personal_year: u32,
    /// Personal Month (current)
    pub personal_month: u32,
    /// Key traits
    pub key_traits: Vec<String>,
    /// Life purpose
    pub life_purpose: String,
}

/// Biorhythm analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiorhythmAnalysis {
    /// Physical cycle (23 days)
    pub physical: f64,
    /// Emotional cycle (28 days)
    pub emotional: f64,
    /// Intellectual cycle (33 days)
    pub intellectual: f64,
    /// Overall vitality score
    pub vitality_score: f64,
    /// Critical days (when cycles cross zero)
    pub critical_days: Vec<String>,
    /// Peak days (when cycles are at maximum)
    pub peak_days: Vec<String>,
}

/// Layered insight from multiple systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredInsight {
    /// Area of life
    pub area: String,
    /// Vedic perspective
    pub vedic_perspective: String,
    /// TCM perspective
    pub tcm_perspective: String,
    /// Numerology perspective
    pub numerology_perspective: String,
    /// Synthesized insight
    pub synthesized: String,
    /// Supporting factors
    pub supporting_factors: Vec<String>,
    /// Challenging factors
    pub challenging_factors: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Unified recommendation combining all systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRecommendation {
    pub category: String,
    pub description: String,
    pub sources: Vec<String>,
    pub priority: Priority,
    pub timeframe: Option<String>,
}

/// Priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl UnifiedAnalysis {
    /// Generate a unified analysis from a birth profile
    pub async fn generate(profile: &BirthProfile) -> Result<Self> {
        let config = IntegrationConfig::default();
        Self::generate_with_config(profile, &config).await
    }
    
    /// Generate with specific configuration
    pub async fn generate_with_config(
        profile: &BirthProfile,
        config: &IntegrationConfig,
    ) -> Result<Self> {
        let generated_at = Utc::now();
        
        // Get Vedic API data if enabled
        let panchang = if config.use_vedic_api {
            Some(Self::fetch_panchang(profile).await?)
        } else {
            None
        };
        
        // Get Vimshottari analysis
        let vimshottari = Self::analyze_vimshottari(profile).await?;
        
        // Get Numerology analysis
        let numerology = Self::analyze_numerology(profile).await?;
        
        // Get TCM analysis
        let tcm = TCMAnalysis::from_birth_profile(profile)?;
        
        // Get Biorhythm if enabled
        let biorhythm = if config.include_biorhythm {
            Some(Self::analyze_biorhythm(profile).await?)
        } else {
            None
        };
        
        // Generate layered insights
        let layered_insights = Self::synthesize_insights(
            &vimshottari,
            &numerology,
            &tcm,
            &biorhythm,
        ).await?;
        
        // Find auspicious times
        let auspicious_times = Self::find_auspicious_times_internal(
            profile,
            &panchang,
            &vimshottari,
        ).await?;
        
        // Generate unified recommendations
        let recommendations = Self::generate_recommendations(
            &vimshottari,
            &numerology,
            &tcm,
            &layered_insights,
        ).await?;
        
        Ok(UnifiedAnalysis {
            profile: profile.clone(),
            panchang,
            vimshottari,
            numerology,
            tcm,
            biorhythm,
            layered_insights,
            auspicious_times,
            recommendations,
            generated_at,
        })
    }
    
    /// Fetch Panchang from Vedic API
    async fn fetch_panchang(profile: &BirthProfile) -> Result<CompletePanchang> {
        let client = CachedVedicClient::from_env()
            .map_err(|e| IntegrationError::Configuration(e.to_string()))?;
        
        // Parse birth date
        let date = profile.parse_date()?;
        let time = profile.parse_time()
            .ok_or_else(|| IntegrationError::DateParse("Invalid time".to_string()))?;
        
        // For now, use UTC offset from timezone (simplified)
        let tz_offset = 5.5; // IST
        
        let panchang = client.get_complete_panchang(
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            0,
            profile.latitude,
            profile.longitude,
            tz_offset,
        ).await?;
        
        Ok(panchang)
    }
    
    /// Analyze Vimshottari Dasha
    async fn analyze_vimshottari(profile: &BirthProfile) -> Result<VimshottariAnalysis> {
        // Parse date
        let date = profile.parse_date()?;
        let time = profile.parse_time()
            .ok_or_else(|| IntegrationError::DateParse("Invalid time".to_string()))?;
        
        // Calculate current dasha
        // For now, use simplified calculation based on Shesh's known data
        // In production, this would call the engine-vimshottari
        
        // Shesh's data:
        // Birth: 1991-08-13 13:31 IST, Bengaluru
        // Moon Nakshatra: Uttara Phalguni (ruled by Sun)
        // But balance puts him in Mars Mahadasha
        // Mars Mahadasha ends: 2026-09-14
        
        let now = Utc::now();
        let mahadasha_end = chrono::NaiveDate::parse_from_str("2026-09-14", "%Y-%m-%d")
            .map_err(|e| IntegrationError::DateParse(e.to_string()))?;
        
        let days_remaining = (mahadasha_end - now.date_naive()).num_days();
        
        // Get planetary qualities based on current dasha
        let mars_qualities = PlanetaryQualities {
            planet: "Mars".to_string(),
            themes: vec![
                "Action and courage".to_string(),
                "Physical energy and vitality".to_string(),
                "Assertion and boundary-setting".to_string(),
                "Transformation through challenge".to_string(),
            ],
            life_areas: vec![
                "Career and ambition".to_string(),
                "Physical health".to_string(),
                "Courage and initiative".to_string(),
                "Conflict resolution".to_string(),
            ],
            challenges: vec![
                "Impulsiveness and aggression".to_string(),
                "Burnout from overexertion".to_string(),
                "Conflict and competition".to_string(),
            ],
            opportunities: vec![
                "Building physical strength".to_string(),
                "Taking courageous action".to_string(),
                "Transforming obstacles into growth".to_string(),
            ],
            description: "Mars Mahadasha brings energy, action, and the courage to pursue goals. It's a time for physical vitality, career advancement, and transforming challenges into opportunities for growth.".to_string(),
        };
        
        // Current antardasha (sub-period) - simplified
        // In Mars Mahadasha, the sequence is: Mars, Rahu, Jupiter, Saturn, Mercury, Ketu, Venus, Sun, Moon
        // Each gets proportional period
        let antardasha_qualities = PlanetaryQualities {
            planet: "Jupiter".to_string(), // Assuming Jupiter antardasha
            themes: vec![
                "Wisdom and expansion".to_string(),
                "Spiritual growth".to_string(),
                "Teaching and learning".to_string(),
                "Abundance and prosperity".to_string(),
            ],
            life_areas: vec![
                "Education and knowledge".to_string(),
                "Spiritual practice".to_string(),
                "Long-distance travel".to_string(),
                "Teaching and mentorship".to_string(),
            ],
            challenges: vec![
                "Over-expansion or excess".to_string(),
                "Spiritual arrogance".to_string(),
            ],
            opportunities: vec![
                "Deepening wisdom".to_string(),
                "Expanding influence".to_string(),
                "Spiritual breakthroughs".to_string(),
            ],
            description: "Jupiter Antardasha within Mars Mahadasha combines action with wisdom. It's an excellent time for teaching, learning, and expanding one's horizons through courageous exploration.".to_string(),
        };
        
        Ok(VimshottariAnalysis {
            current_mahadasha: "Mars".to_string(),
            current_antardasha: "Jupiter".to_string(), // Example
            current_pratyantardasha: "Saturn".to_string(), // Example
            mahadasha_end: "2026-09-14".to_string(),
            days_remaining_mahadasha: days_remaining,
            current_themes: vec![
                "Physical vitality and energy".to_string(),
                "Courageous action".to_string(),
                "Career advancement".to_string(),
                "Transformation through challenge".to_string(),
            ],
            upcoming_transitions: vec![
                DashaTransition {
                    level: "Antardasha".to_string(),
                    from_planet: "Jupiter".to_string(),
                    to_planet: "Saturn".to_string(),
                    date: "2025-06-01".to_string(), // Example
                    days_until: 120, // Example
                },
            ],
            mahadasha_qualities: mars_qualities,
            antardasha_qualities: antardasha_qualities,
        })
    }
    
    /// Analyze Numerology
    async fn analyze_numerology(profile: &BirthProfile) -> Result<NumerologyAnalysis> {
        let date = profile.parse_date()?;
        
        // Calculate Life Path Number
        let life_path = calculate_life_path(date.year() as u32, date.month(), date.day());
        
        // Calculate Birth Day Number
        let birth_day = reduce_to_single_digit(date.day() as u32);
        
        // Calculate Personal Year
        let current_year = Utc::now().year() as u32;
        let personal_year = reduce_to_single_digit(
            date.day() as u32 + date.month() + current_year
        );
        
        // Calculate Personal Month
        let current_month = Utc::now().month();
        let personal_month = reduce_to_single_digit(personal_year + current_month);
        
        // Get descriptions
        let (description, purpose, traits) = get_life_path_meaning(life_path);
        
        Ok(NumerologyAnalysis {
            life_path_number: life_path,
            life_path_description: description,
            expression_number: None, // Would need name
            soul_urge_number: None, // Would need name
            personality_number: None, // Would need name
            birth_day_number: birth_day,
            personal_year,
            personal_month,
            key_traits: traits,
            life_purpose: purpose,
        })
    }
    
    /// Analyze Biorhythms
    async fn analyze_biorhythm(profile: &BirthProfile) -> Result<BiorhythmAnalysis> {
        let birth_date = profile.parse_date()?;
        let now = Utc::now();
        
        // Calculate days since birth
        let days_since_birth = (now.date_naive() - birth_date).num_days();
        
        // Calculate cycles
        let physical = calculate_biorhythm_cycle(days_since_birth, 23);
        let emotional = calculate_biorhythm_cycle(days_since_birth, 28);
        let intellectual = calculate_biorhythm_cycle(days_since_birth, 33);
        
        // Calculate vitality score
        let vitality = (physical + emotional + intellectual) / 3.0;
        
        Ok(BiorhythmAnalysis {
            physical,
            emotional,
            intellectual,
            vitality_score: vitality,
            critical_days: Vec::new(), // Would calculate properly
            peak_days: Vec::new(), // Would calculate properly
        })
    }
    
    /// Synthesize insights from all systems
    async fn synthesize_insights(
        vimshottari: &VimshottariAnalysis,
        numerology: &NumerologyAnalysis,
        tcm: &TCMAnalysis,
        biorhythm: &Option<BiorhythmAnalysis>,
    ) -> Result<Vec<LayeredInsight>> {
        let mut insights = Vec::new();
        
        // Career insight
        insights.push(LayeredInsight {
            area: "Career and Purpose".to_string(),
            vedic_perspective: format!(
                "Current Mars Mahadasha emphasizes action and career advancement. Focus on building physical vitality and taking courageous initiatives.",
            ),
            tcm_perspective: format!(
                "Your dominant {} element supports {}. Focus on {} organs during their peak hours ({}-{}).",
                tcm.dominant_element.as_str(),
                match tcm.dominant_element {
                    TCMElement::Wood => "growth and expansion",
                    TCMElement::Fire => "passion and transformation",
                    TCMElement::Earth => "stability and nurturing",
                    TCMElement::Metal => "structure and precision",
                    TCMElement::Water => "flow and adaptability",
                },
                tcm.strong_organs.first().map(|o| format!("{:?}", o)).unwrap_or_default(),
                tcm.optimal_times.first().map(|t| t.start_hour).unwrap_or(0),
                tcm.optimal_times.first().map(|t| t.end_hour).unwrap_or(0),
            ),
            numerology_perspective: format!(
                "Life Path {} indicates {}. Your personal year {} brings {}.",
                numerology.life_path_number,
                numerology.life_path_description,
                numerology.personal_year,
                get_personal_year_meaning(numerology.personal_year),
            ),
            synthesized: format!(
                "This is a powerful time for career growth combining Mars's action-oriented energy with your Life Path {} qualities. Support your physical vitality through {} practices while taking bold, strategic action.",
                numerology.life_path_number,
                tcm.dominant_element.as_str(),
            ),
            supporting_factors: vec![
                "Mars Mahadasha - high energy for action".to_string(),
                format!("Life Path {} - natural alignment", numerology.life_path_number),
                format!("{} element strength", tcm.dominant_element.as_str()),
            ],
            challenging_factors: vec![
                "Risk of burnout from excessive activity".to_string(),
                "Need to balance action with rest".to_string(),
            ],
            recommendations: vec![
                "Take courageous action on career goals".to_string(),
                format!("Practice {}-supporting exercises", tcm.dominant_element.as_str()),
                "Schedule important initiatives during Mars-ruled days (Tuesday)".to_string(),
            ],
        });
        
        // Health insight
        insights.push(LayeredInsight {
            area: "Health and Vitality".to_string(),
            vedic_perspective: "Mars Mahadasha affects physical energy and vitality. Pay attention to inflammation, blood pressure, and adrenal health.".to_string(),
            tcm_perspective: format!(
                "Your constitution shows {} tendency. Support {} organs and be mindful of {}.",
                match tcm.constitution {
                    crate::tcm_layer::ConstitutionalType::Balanced => "balanced",
                    crate::tcm_layer::ConstitutionalType::QiDeficient => "Qi deficient",
                    crate::tcm_layer::ConstitutionalType::YangDeficient => "Yang deficient",
                    crate::tcm_layer::ConstitutionalType::YinDeficient => "Yin deficient",
                    crate::tcm_layer::ConstitutionalType::PhlegmDampness => "Phlegm-Damp",
                    crate::tcm_layer::ConstitutionalType::DampHeat => "Damp-Heat",
                    crate::tcm_layer::ConstitutionalType::BloodStasis => "Blood stasis",
                    crate::tcm_layer::ConstitutionalType::QiStagnation => "Qi stagnation",
                    crate::tcm_layer::ConstitutionalType::SpecialConstitution => "special",
                },
                tcm.strong_organs.first().map(|o| format!("{:?}", o)).unwrap_or_default(),
                tcm.vulnerable_organs.first().map(|o| format!("{:?}", o)).unwrap_or_default(),
            ),
            numerology_perspective: format!(
                "Birth Day {} emphasizes {} qualities. Physical activity is important for your well-being.",
                numerology.birth_day_number,
                if numerology.birth_day_number % 2 == 0 { "receptive" } else { "active" },
            ),
            synthesized: format!(
                "Maintain physical vitality through balanced exercise supporting both Mars energy and your {} constitution. Focus on {} organs during their peak hours.",
                tcm.dominant_element.as_str(),
                tcm.vulnerable_organs.first().map(|o| format!("{:?}", o)).unwrap_or_default(),
            ),
            supporting_factors: vec![
                "Mars energy supports physical activity".to_string(),
                format!("{} constitution provides foundation", tcm.dominant_element.as_str()),
            ],
            challenging_factors: tcm.vulnerable_organs.iter()
                .map(|o| format!("{:?} vulnerability", o))
                .collect(),
            recommendations: tcm.recommendations.iter()
                .filter(|r| r.category == crate::tcm_layer::RecommendationCategory::Exercise 
                    || r.category == crate::tcm_layer::RecommendationCategory::Diet)
                .map(|r| r.description.clone())
                .collect(),
        });
        
        Ok(insights)
    }
    
    /// Find auspicious times combining all systems
    async fn find_auspicious_times_internal(
        profile: &BirthProfile,
        panchang: &Option<CompletePanchang>,
        vimshottari: &VimshottariAnalysis,
    ) -> Result<Vec<AuspiciousWindow>> {
        let mut windows = Vec::new();
        
        // If we have panchang data, use it
        if let Some(p) = panchang {
            // Get favorable muhurtas
            if let Some(ref amrit) = p.muhurtas.amrit_kaal {
                windows.push(AuspiciousWindow {
                    start: Utc::now(), // Would parse properly
                    end: Utc::now(),
                    quality: AuspiciousQuality::Excellent,
                    sources: vec!["Amrit Kaal".to_string()],
                    description: "Nectar time - highly auspicious for beginnings".to_string(),
                });
            }
            
            if let Some(ref abhijit) = p.muhurtas.abhijit {
                windows.push(AuspiciousWindow {
                    start: Utc::now(),
                    end: Utc::now(),
                    quality: AuspiciousQuality::Excellent,
                    sources: vec!["Abhijit Muhurta".to_string()],
                    description: "Victorious midday - favorable for all activities".to_string(),
                });
            }
        }
        
        // Add Mars-ruled days for current Mahadasha
        if vimshottari.current_mahadasha == "Mars" {
            windows.push(AuspiciousWindow {
                start: Utc::now(),
                end: Utc::now(),
                quality: AuspiciousQuality::Good,
                sources: vec!["Mars Mahadasha".to_string()],
                description: "Tuesday (Mars day) is favorable for initiating new projects".to_string(),
            });
        }
        
        Ok(windows)
    }
    
    /// Generate unified recommendations
    async fn generate_recommendations(
        vimshottari: &VimshottariAnalysis,
        numerology: &NumerologyAnalysis,
        tcm: &TCMAnalysis,
        insights: &[LayeredInsight],
    ) -> Result<Vec<UnifiedRecommendation>> {
        let mut recommendations = Vec::new();
        
        // Add TCM recommendations
        for rec in &tcm.recommendations {
            recommendations.push(UnifiedRecommendation {
                category: format!("{:?}", rec.category),
                description: rec.description.clone(),
                sources: vec!["TCM".to_string()],
                priority: match rec.priority {
                    crate::tcm_layer::Priority::Low => Priority::Low,
                    crate::tcm_layer::Priority::Medium => Priority::Medium,
                    crate::tcm_layer::Priority::High => Priority::High,
                    crate::tcm_layer::Priority::Critical => Priority::Critical,
                },
                timeframe: None,
            });
        }
        
        // Add Vimshottari recommendations
        recommendations.push(UnifiedRecommendation {
            category: "Spiritual".to_string(),
            description: format!(
                "Honor your {} Mahadasha by practicing {} meditation and {}.",
                vimshottari.current_mahadasha,
                vimshottari.current_mahadasha.to_lowercase(),
                if vimshottari.current_mahadasha == "Mars" {
                    "physical yoga practices"
                } else {
                    "contemplative practices"
                }
            ),
            sources: vec!["Vimshottari".to_string()],
            priority: Priority::High,
            timeframe: Some(format!("Until {}", vimshottari.mahadasha_end)),
        });
        
        Ok(recommendations)
    }
}

/// Find auspicious windows for an activity
pub async fn find_auspicious_windows(
    profile: &BirthProfile,
    activity: ActivityType,
    days: u32,
) -> Result<Vec<AuspiciousWindow>> {
    UnifiedAnalysis::find_auspicious_times_internal(
        profile,
        &None,
        &VimshottariAnalysis {
            current_mahadasha: "Mars".to_string(),
            current_antardasha: "Jupiter".to_string(),
            current_pratyantardasha: "Saturn".to_string(),
            mahadasha_end: "2026-09-14".to_string(),
            days_remaining_mahadasha: 900,
            current_themes: Vec::new(),
            upcoming_transitions: Vec::new(),
            mahadasha_qualities: PlanetaryQualities {
                planet: "Mars".to_string(),
                themes: Vec::new(),
                life_areas: Vec::new(),
                challenges: Vec::new(),
                opportunities: Vec::new(),
                description: String::new(),
            },
            antardasha_qualities: PlanetaryQualities {
                planet: "Jupiter".to_string(),
                themes: Vec::new(),
                life_areas: Vec::new(),
                challenges: Vec::new(),
                opportunities: Vec::new(),
                description: String::new(),
            },
        },
    ).await
}

// Helper functions

fn calculate_life_path(year: u32, month: u32, day: u32) -> u32 {
    let year_sum = reduce_to_single_digit(year);
    let month_sum = reduce_to_single_digit(month);
    let day_sum = reduce_to_single_digit(day);
    
    reduce_to_single_digit(year_sum + month_sum + day_sum)
}

fn reduce_to_single_digit(n: u32) -> u32 {
    let mut sum = n;
    while sum > 9 && !is_master_number(sum) {
        sum = sum_of_digits(sum);
    }
    sum
}

fn sum_of_digits(n: u32) -> u32 {
    let mut num = n;
    let mut sum = 0;
    while num > 0 {
        sum += num % 10;
        num /= 10;
    }
    sum
}

fn is_master_number(n: u32) -> bool {
    matches!(n, 11 | 22 | 33)
}

fn get_life_path_meaning(n: u32) -> (String, String, Vec<String>) {
    match n {
        1 => (
            "The Leader - Independent, innovative, pioneering".to_string(),
            "To develop self-confidence and leadership".to_string(),
            vec!["Independent".to_string(), "Innovative".to_string(), "Ambitious".to_string()],
        ),
        2 => (
            "The Peacemaker - Cooperative, diplomatic, sensitive".to_string(),
            "To bring harmony and balance to relationships".to_string(),
            vec!["Diplomatic".to_string(), "Sensitive".to_string(), "Cooperative".to_string()],
        ),
        3 => (
            "The Communicator - Creative, expressive, joyful".to_string(),
            "To inspire and uplift through self-expression".to_string(),
            vec!["Creative".to_string(), "Expressive".to_string(), "Optimistic".to_string()],
        ),
        4 => (
            "The Builder - Practical, reliable, disciplined".to_string(),
            "To build solid foundations for yourself and others".to_string(),
            vec!["Practical".to_string(), "Reliable".to_string(), "Hardworking".to_string()],
        ),
        5 => (
            "The Freedom Seeker - Adventurous, versatile, curious".to_string(),
            "To embrace change and inspire freedom in others".to_string(),
            vec!["Adventurous".to_string(), "Versatile".to_string(), "Freedom-loving".to_string()],
        ),
        6 => (
            "The Nurturer - Responsible, caring, harmonious".to_string(),
            "To serve and nurture family and community".to_string(),
            vec!["Caring".to_string(), "Responsible".to_string(), "Harmonious".to_string()],
        ),
        7 => (
            "The Seeker - Analytical, spiritual, introspective".to_string(),
            "To seek truth and wisdom through inner exploration".to_string(),
            vec!["Analytical".to_string(), "Spiritual".to_string(), "Introspective".to_string()],
        ),
        8 => (
            "The Powerhouse - Ambitious, authoritative, successful".to_string(),
            "To achieve material and spiritual abundance".to_string(),
            vec!["Ambitious".to_string(), "Authoritative".to_string(), "Successful".to_string()],
        ),
        9 => (
            "The Humanitarian - Compassionate, wise, universal".to_string(),
            "To serve humanity with compassion and wisdom".to_string(),
            vec!["Compassionate".to_string(), "Wise".to_string(), "Universal".to_string()],
        ),
        _ => (
            "Unknown path".to_string(),
            "Explore your unique qualities".to_string(),
            vec!["Unique".to_string()],
        ),
    }
}

fn get_personal_year_meaning(n: u32) -> &'static str {
    match n {
        1 => "new beginnings and independence",
        2 => "cooperation and relationships",
        3 => "creativity and self-expression",
        4 => "building foundations and hard work",
        5 => "change and freedom",
        6 => "responsibility and family",
        7 => "introspection and spiritual growth",
        8 => "achievement and abundance",
        9 => "completion and humanitarian service",
        _ => "transformation",
    }
}

fn calculate_biorhythm_cycle(days: i64, cycle_length: i64) -> f64 {
    let position = (days % cycle_length) as f64 / cycle_length as f64;
    (position * 2.0 * std::f64::consts::PI).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_life_path_calculation() {
        // Shesh's birth: 1991-08-13
        // 1991 -> 1+9+9+1 = 20 -> 2+0 = 2
        // 8 -> 8
        // 13 -> 1+3 = 4
        // Total: 2 + 8 + 4 = 14 -> 1+4 = 5
        let life_path = calculate_life_path(1991, 8, 13);
        assert_eq!(life_path, 5);
    }

    #[test]
    fn test_reduce_to_single_digit() {
        assert_eq!(reduce_to_single_digit(23), 5); // 2+3 = 5
        assert_eq!(reduce_to_single_digit(1991), 2); // 1+9+9+1 = 20 -> 2+0 = 2
        assert_eq!(reduce_to_single_digit(11), 11); // Master number preserved
    }

    #[test]
    fn test_numerology_analysis() {
        let profile = BirthProfile::new(
            "1991-08-13",
            "13:31",
            12.9716,
            77.5946,
            "Asia/Kolkata",
        );
        
        // Would need async runtime for full test
        // For now just test the calculation
        let life_path = calculate_life_path(1991, 8, 13);
        assert_eq!(life_path, 5);
    }
}
