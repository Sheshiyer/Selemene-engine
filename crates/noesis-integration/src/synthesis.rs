//! Synthesis Engine for combining insights from multiple consciousness systems
//!
//! This module provides intelligent synthesis of Vedic, TCM, Numerology, and other
//! system outputs into coherent, actionable insights.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{LayeredInsight, UnifiedRecommendation};
use crate::analysis::Priority;

/// Engine for synthesizing multi-system insights
pub struct SynthesisEngine {
    /// Weightings for different systems
    system_weights: HashMap<String, f64>,
    /// Confidence threshold for including insights
    confidence_threshold: f64,
}

/// Synthesized insight combining multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedInsight {
    /// Unique identifier
    pub id: String,
    /// Category of insight
    pub category: InsightCategory,
    /// Primary theme
    pub theme: String,
    /// Detailed description
    pub description: String,
    /// Contributing systems
    pub sources: Vec<SystemContribution>,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Actionable recommendations
    pub actions: Vec<String>,
    /// Timeframe for application
    pub timeframe: Timeframe,
}

/// Category of insight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightCategory {
    Career,
    Health,
    Relationships,
    Spirituality,
    Finances,
    PersonalGrowth,
    Creativity,
    LifePurpose,
}

/// Contribution from a specific system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContribution {
    /// System name
    pub system: String,
    /// Specific input from this system
    pub input: String,
    /// Weight of this contribution (0.0-1.0)
    pub weight: f64,
}

/// Timeframe for application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    Immediate,    // Today
    ShortTerm,    // This week
    MediumTerm,   // This month
    LongTerm,     // This year
    LifePhase,    // Current major period
}

impl SynthesisEngine {
    /// Create a new synthesis engine with default weights
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert("vedic".to_string(), 0.3);
        weights.insert("vimshottari".to_string(), 0.25);
        weights.insert("tcm".to_string(), 0.2);
        weights.insert("numerology".to_string(), 0.15);
        weights.insert("biorhythm".to_string(), 0.1);
        
        Self {
            system_weights: weights,
            confidence_threshold: 0.6,
        }
    }
    
    /// Set custom weight for a system
    pub fn with_weight(mut self, system: &str, weight: f64) -> Self {
        self.system_weights.insert(system.to_string(), weight);
        self
    }
    
    /// Synthesize insights from layered inputs
    pub fn synthesize(&self, insights: &[LayeredInsight]) -> Vec<SynthesizedInsight> {
        let mut synthesized = Vec::new();
        
        for insight in insights {
            let confidence = self.calculate_confidence(insight);
            
            if confidence >= self.confidence_threshold {
                synthesized.push(SynthesizedInsight {
                    id: format!("insight_{}", synthesized.len()),
                    category: self.categorize(&insight.area),
                    theme: insight.area.clone(),
                    description: insight.synthesized.clone(),
                    sources: vec![
                        SystemContribution {
                            system: "Vedic".to_string(),
                            input: insight.vedic_perspective.clone(),
                            weight: *self.system_weights.get("vedic").unwrap_or(&0.3),
                        },
                        SystemContribution {
                            system: "TCM".to_string(),
                            input: insight.tcm_perspective.clone(),
                            weight: *self.system_weights.get("tcm").unwrap_or(&0.2),
                        },
                        SystemContribution {
                            system: "Numerology".to_string(),
                            input: insight.numerology_perspective.clone(),
                            weight: *self.system_weights.get("numerology").unwrap_or(&0.15),
                        },
                    ],
                    confidence,
                    actions: insight.recommendations.clone(),
                    timeframe: self.determine_timeframe(&insight.area),
                });
            }
        }
        
        // Sort by confidence
        synthesized.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        synthesized
    }
    
    /// Calculate confidence score for an insight
    fn calculate_confidence(&self, insight: &LayeredInsight) -> f64 {
        let mut score = 0.5; // Base score
        
        // Boost for supporting factors
        score += insight.supporting_factors.len() as f64 * 0.1;
        
        // Penalty for challenging factors
        score -= insight.challenging_factors.len() as f64 * 0.05;
        
        // Boost for having recommendations
        if !insight.recommendations.is_empty() {
            score += 0.1;
        }
        
        (score as f64).min(1.0).max(0.0)
    }
    
    /// Categorize an insight area
    fn categorize(&self, area: &str) -> InsightCategory {
        match area.to_lowercase().as_str() {
            a if a.contains("career") || a.contains("purpose") => InsightCategory::Career,
            a if a.contains("health") || a.contains("vitality") => InsightCategory::Health,
            a if a.contains("relationship") || a.contains("love") => InsightCategory::Relationships,
            a if a.contains("spiritual") || a.contains("meditation") => InsightCategory::Spirituality,
            a if a.contains("money") || a.contains("finance") => InsightCategory::Finances,
            a if a.contains("growth") || a.contains("personal") => InsightCategory::PersonalGrowth,
            a if a.contains("creative") || a.contains("expression") => InsightCategory::Creativity,
            _ => InsightCategory::LifePurpose,
        }
    }
    
    /// Determine timeframe based on area
    fn determine_timeframe(&self, area: &str) -> Timeframe {
        match area.to_lowercase().as_str() {
            a if a.contains("health") => Timeframe::Immediate,
            a if a.contains("career") => Timeframe::MediumTerm,
            a if a.contains("relationship") => Timeframe::ShortTerm,
            a if a.contains("spiritual") => Timeframe::LifePhase,
            _ => Timeframe::MediumTerm,
        }
    }
    
    /// Generate a witness prompt from synthesized insights
    pub fn generate_witness_prompt(&self, insights: &[SynthesizedInsight]) -> String {
        let mut prompt = String::from("# Multi-System Synthesis\n\n");
        
        for insight in insights.iter().take(3) {
            prompt.push_str(&format!("## {}\n", insight.theme));
            prompt.push_str(&format!("{}\n\n", insight.description));
            prompt.push_str("**Sources:** ");
            
            let sources: Vec<String> = insight.sources.iter()
                .map(|s| format!("{} ({}%)", s.system, (s.weight * 100.0) as i32))
                .collect();
            prompt.push_str(&sources.join(", "));
            prompt.push_str("\n\n");
            
            if !insight.actions.is_empty() {
                prompt.push_str("**Actions:**\n");
                for action in &insight.actions {
                    prompt.push_str(&format!("- {}\n", action));
                }
                prompt.push('\n');
            }
        }
        
        prompt.push_str("\n*Witness this synthesis and notice which insights resonate with your current experience.*");
        
        prompt
    }
    
    /// Prioritize recommendations from multiple sources
    pub fn prioritize_recommendations(
        &self,
        recommendations: &[UnifiedRecommendation],
    ) -> Vec<UnifiedRecommendation> {
        let mut prioritized = recommendations.to_vec();
        
        // Sort by priority
        prioritized.sort_by(|a, b| {
            let priority_order = |p: &Priority| match p {
                Priority::Critical => 0,
                Priority::High => 1,
                Priority::Medium => 2,
                Priority::Low => 3,
            };
            priority_order(&a.priority).cmp(&priority_order(&b.priority))
        });
        
        prioritized
    }
    
    /// Find patterns across multiple analyses
    pub fn find_patterns(&self, insights: &[LayeredInsight]) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        
        // Look for common themes
        let mut theme_counts: HashMap<String, usize> = HashMap::new();
        for insight in insights {
            *theme_counts.entry(insight.area.clone()).or_insert(0) += 1;
        }
        
        // Extract patterns from repeated themes
        for (theme, count) in theme_counts {
            if count > 1 {
                patterns.push(Pattern {
                    name: format!("Recurring theme: {}", theme),
                    description: format!("This theme appears {} times across analyses", count),
                    significance: if count >= 3 { 
                        PatternSignificance::High 
                    } else { 
                        PatternSignificance::Medium 
                    },
                    related_areas: vec![theme],
                });
            }
        }
        
        patterns
    }
}

impl Default for SynthesisEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern detected across analyses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub significance: PatternSignificance,
    pub related_areas: Vec<String>,
}

/// Significance of a pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternSignificance {
    Low,
    Medium,
    High,
    Critical,
}

/// Cross-system correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSystemCorrelation {
    /// Systems involved
    pub systems: Vec<String>,
    /// Correlated factor
    pub factor: String,
    /// Correlation strength (0.0-1.0)
    pub strength: f64,
    /// Interpretation
    pub interpretation: String,
}

/// Generate a comprehensive report
pub fn generate_comprehensive_report(
    insights: &[SynthesizedInsight],
    patterns: &[Pattern],
) -> String {
    let mut report = String::from("# Comprehensive Multi-System Analysis Report\n\n");
    
    // Summary
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **Total Insights:** {}\n", insights.len()));
    report.push_str(&format!("- **Patterns Detected:** {}\n", patterns.len()));
    report.push_str(&format!("- **Average Confidence:** {:.0}%\n\n", 
        if insights.is_empty() { 
            0.0 
        } else { 
            insights.iter().map(|i| i.confidence).sum::<f64>() / insights.len() as f64 * 100.0 
        }
    ));
    
    // Key Insights
    report.push_str("## Key Insights\n\n");
    for (i, insight) in insights.iter().take(5).enumerate() {
        report.push_str(&format!("### {}. {}\n", i + 1, insight.theme));
        report.push_str(&format!("**Confidence:** {:.0}%\n\n", insight.confidence * 100.0));
        report.push_str(&format!("{}\n\n", insight.description));
    }
    
    // Patterns
    if !patterns.is_empty() {
        report.push_str("## Detected Patterns\n\n");
        for pattern in patterns {
            report.push_str(&format!("- **{}**: {}\n", pattern.name, pattern.description));
        }
        report.push('\n');
    }
    
    // Recommendations
    report.push_str("## Priority Recommendations\n\n");
    let all_actions: Vec<String> = insights.iter()
        .flat_map(|i| i.actions.clone())
        .take(7)
        .collect();
    
    for (i, action) in all_actions.iter().enumerate() {
        report.push_str(&format!("{}. {}\n", i + 1, action));
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesis_engine_creation() {
        let engine = SynthesisEngine::new();
        assert!(!engine.system_weights.is_empty());
    }

    #[test]
    fn test_categorization() {
        let engine = SynthesisEngine::new();
        
        assert_eq!(engine.categorize("Career Growth"), InsightCategory::Career);
        assert_eq!(engine.categorize("Physical Health"), InsightCategory::Health);
        assert_eq!(engine.categorize("Love Life"), InsightCategory::Relationships);
    }

    #[test]
    fn test_confidence_calculation() {
        let engine = SynthesisEngine::new();
        
        let insight = LayeredInsight {
            area: "Test".to_string(),
            vedic_perspective: "Test".to_string(),
            tcm_perspective: "Test".to_string(),
            numerology_perspective: "Test".to_string(),
            synthesized: "Test".to_string(),
            supporting_factors: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            challenging_factors: vec!["X".to_string()],
            recommendations: vec!["Do this".to_string()],
        };
        
        let confidence = engine.calculate_confidence(&insight);
        assert!(confidence > 0.5);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_pattern_detection() {
        let engine = SynthesisEngine::new();
        
        let insights = vec![
            LayeredInsight {
                area: "Career".to_string(),
                vedic_perspective: "A".to_string(),
                tcm_perspective: "B".to_string(),
                numerology_perspective: "C".to_string(),
                synthesized: "D".to_string(),
                supporting_factors: vec![],
                challenging_factors: vec![],
                recommendations: vec![],
            },
            LayeredInsight {
                area: "Career".to_string(),
                vedic_perspective: "A".to_string(),
                tcm_perspective: "B".to_string(),
                numerology_perspective: "C".to_string(),
                synthesized: "D".to_string(),
                supporting_factors: vec![],
                challenging_factors: vec![],
                recommendations: vec![],
            },
            LayeredInsight {
                area: "Health".to_string(),
                vedic_perspective: "A".to_string(),
                tcm_perspective: "B".to_string(),
                numerology_perspective: "C".to_string(),
                synthesized: "D".to_string(),
                supporting_factors: vec![],
                challenging_factors: vec![],
                recommendations: vec![],
            },
        ];
        
        let patterns = engine.find_patterns(&insights);
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.name.contains("Career")));
    }
}
