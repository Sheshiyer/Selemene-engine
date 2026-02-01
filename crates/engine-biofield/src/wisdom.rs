//! Biofield Wisdom Data
//!
//! Contains chakra descriptions, metric interpretations, and algorithm documentation.
//! This data informs both the analysis interpretations and witness prompt generation.

use crate::models::Chakra;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Comprehensive chakra wisdom data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChakraWisdom {
    /// The chakra this wisdom relates to
    pub chakra: Chakra,
    /// Sanskrit and English name
    pub name: String,
    /// Body location
    pub location: String,
    /// Associated element
    pub element: String,
    /// Associated color in healthy state
    pub color: String,
    /// Core qualities and themes
    pub qualities: Vec<String>,
    /// Signs of balanced/healthy function
    pub balanced_signs: Vec<String>,
    /// Signs of imbalance or blockage
    pub imbalanced_signs: Vec<String>,
    /// Body systems associated
    pub body_systems: Vec<String>,
}

/// Interpretation guidance for biofield metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricInterpretation {
    pub name: String,
    pub description: String,
    pub low_meaning: String,
    pub optimal_meaning: String,
    pub high_meaning: String,
    pub optimal_range: (f64, f64),
}

/// Static chakra wisdom data
static CHAKRA_WISDOM: OnceLock<HashMap<Chakra, ChakraWisdom>> = OnceLock::new();

/// Static metric interpretations
static METRIC_INTERPRETATIONS: OnceLock<HashMap<String, MetricInterpretation>> = OnceLock::new();

/// Get chakra wisdom data
pub fn chakra_wisdom() -> &'static HashMap<Chakra, ChakraWisdom> {
    CHAKRA_WISDOM.get_or_init(|| build_chakra_wisdom())
}

/// Get specific chakra wisdom
pub fn get_chakra_wisdom(chakra: Chakra) -> Option<&'static ChakraWisdom> {
    chakra_wisdom().get(&chakra)
}

/// Get metric interpretation data
pub fn metric_interpretations() -> &'static HashMap<String, MetricInterpretation> {
    METRIC_INTERPRETATIONS.get_or_init(|| build_metric_interpretations())
}

/// Get specific metric interpretation
pub fn get_metric_interpretation(metric_name: &str) -> Option<&'static MetricInterpretation> {
    metric_interpretations().get(metric_name)
}

fn build_chakra_wisdom() -> HashMap<Chakra, ChakraWisdom> {
    let mut map = HashMap::new();
    
    map.insert(Chakra::Root, ChakraWisdom {
        chakra: Chakra::Root,
        name: "Muladhara (Root)".to_string(),
        location: "Base of spine, perineum".to_string(),
        element: "Earth".to_string(),
        color: "Red".to_string(),
        qualities: vec![
            "Grounding and stability".to_string(),
            "Survival and safety".to_string(),
            "Physical vitality".to_string(),
            "Connection to Earth".to_string(),
            "Primal trust".to_string(),
        ],
        balanced_signs: vec![
            "Feeling grounded and secure".to_string(),
            "Healthy relationship with material needs".to_string(),
            "Physical energy and stamina".to_string(),
            "Sense of belonging".to_string(),
        ],
        imbalanced_signs: vec![
            "Anxiety or fear about survival".to_string(),
            "Disconnection from body".to_string(),
            "Financial insecurity patterns".to_string(),
            "Restlessness or inability to settle".to_string(),
        ],
        body_systems: vec![
            "Adrenal glands".to_string(),
            "Spine".to_string(),
            "Legs and feet".to_string(),
            "Large intestine".to_string(),
        ],
    });
    
    map.insert(Chakra::Sacral, ChakraWisdom {
        chakra: Chakra::Sacral,
        name: "Svadhisthana (Sacral)".to_string(),
        location: "Lower abdomen, below navel".to_string(),
        element: "Water".to_string(),
        color: "Orange".to_string(),
        qualities: vec![
            "Creativity and passion".to_string(),
            "Emotional flow".to_string(),
            "Sensuality and pleasure".to_string(),
            "Relationships and connection".to_string(),
            "Adaptability".to_string(),
        ],
        balanced_signs: vec![
            "Creative expression flows easily".to_string(),
            "Healthy emotional processing".to_string(),
            "Comfortable with intimacy".to_string(),
            "Flexible and adaptable".to_string(),
        ],
        imbalanced_signs: vec![
            "Creative blocks".to_string(),
            "Emotional volatility or numbness".to_string(),
            "Relationship difficulties".to_string(),
            "Addictive tendencies".to_string(),
        ],
        body_systems: vec![
            "Reproductive organs".to_string(),
            "Kidneys".to_string(),
            "Bladder".to_string(),
            "Lower back".to_string(),
        ],
    });
    
    map.insert(Chakra::SolarPlexus, ChakraWisdom {
        chakra: Chakra::SolarPlexus,
        name: "Manipura (Solar Plexus)".to_string(),
        location: "Upper abdomen, stomach area".to_string(),
        element: "Fire".to_string(),
        color: "Yellow".to_string(),
        qualities: vec![
            "Personal power and will".to_string(),
            "Self-esteem and confidence".to_string(),
            "Transformation and metabolism".to_string(),
            "Decision-making".to_string(),
            "Inner fire and motivation".to_string(),
        ],
        balanced_signs: vec![
            "Healthy self-esteem".to_string(),
            "Clear sense of purpose".to_string(),
            "Able to set boundaries".to_string(),
            "Good digestion".to_string(),
        ],
        imbalanced_signs: vec![
            "Control issues or powerlessness".to_string(),
            "Low self-worth".to_string(),
            "Digestive problems".to_string(),
            "Difficulty making decisions".to_string(),
        ],
        body_systems: vec![
            "Digestive system".to_string(),
            "Liver and gallbladder".to_string(),
            "Pancreas".to_string(),
            "Spleen".to_string(),
        ],
    });
    
    map.insert(Chakra::Heart, ChakraWisdom {
        chakra: Chakra::Heart,
        name: "Anahata (Heart)".to_string(),
        location: "Center of chest".to_string(),
        element: "Air".to_string(),
        color: "Green".to_string(),
        qualities: vec![
            "Love and compassion".to_string(),
            "Connection and empathy".to_string(),
            "Forgiveness and acceptance".to_string(),
            "Inner peace".to_string(),
            "Bridge between lower and upper chakras".to_string(),
        ],
        balanced_signs: vec![
            "Capacity for deep love".to_string(),
            "Compassion for self and others".to_string(),
            "Healthy relationships".to_string(),
            "Emotional resilience".to_string(),
        ],
        imbalanced_signs: vec![
            "Fear of intimacy".to_string(),
            "Grief or heartbreak".to_string(),
            "Jealousy or codependency".to_string(),
            "Isolation or loneliness".to_string(),
        ],
        body_systems: vec![
            "Heart and circulatory system".to_string(),
            "Lungs".to_string(),
            "Thymus".to_string(),
            "Arms and hands".to_string(),
        ],
    });
    
    map.insert(Chakra::Throat, ChakraWisdom {
        chakra: Chakra::Throat,
        name: "Vishuddha (Throat)".to_string(),
        location: "Throat area".to_string(),
        element: "Ether/Space".to_string(),
        color: "Blue".to_string(),
        qualities: vec![
            "Communication and expression".to_string(),
            "Truth and authenticity".to_string(),
            "Listening and understanding".to_string(),
            "Creative expression".to_string(),
            "Speaking one's truth".to_string(),
        ],
        balanced_signs: vec![
            "Clear, authentic communication".to_string(),
            "Good listening skills".to_string(),
            "Creative self-expression".to_string(),
            "Alignment of words and actions".to_string(),
        ],
        imbalanced_signs: vec![
            "Difficulty expressing oneself".to_string(),
            "Fear of speaking up".to_string(),
            "Talking too much without listening".to_string(),
            "Throat or thyroid issues".to_string(),
        ],
        body_systems: vec![
            "Thyroid".to_string(),
            "Throat and neck".to_string(),
            "Mouth and jaw".to_string(),
            "Ears".to_string(),
        ],
    });
    
    map.insert(Chakra::ThirdEye, ChakraWisdom {
        chakra: Chakra::ThirdEye,
        name: "Ajna (Third Eye)".to_string(),
        location: "Between eyebrows".to_string(),
        element: "Light".to_string(),
        color: "Indigo".to_string(),
        qualities: vec![
            "Intuition and insight".to_string(),
            "Wisdom and clarity".to_string(),
            "Imagination and visualization".to_string(),
            "Inner knowing".to_string(),
            "Connection to higher guidance".to_string(),
        ],
        balanced_signs: vec![
            "Clear intuition".to_string(),
            "Good memory and concentration".to_string(),
            "Ability to see the big picture".to_string(),
            "Trust in inner guidance".to_string(),
        ],
        imbalanced_signs: vec![
            "Confusion or lack of clarity".to_string(),
            "Disconnection from intuition".to_string(),
            "Headaches or vision problems".to_string(),
            "Overthinking or delusion".to_string(),
        ],
        body_systems: vec![
            "Pineal gland".to_string(),
            "Pituitary gland".to_string(),
            "Eyes".to_string(),
            "Brain".to_string(),
        ],
    });
    
    map.insert(Chakra::Crown, ChakraWisdom {
        chakra: Chakra::Crown,
        name: "Sahasrara (Crown)".to_string(),
        location: "Top of head".to_string(),
        element: "Consciousness".to_string(),
        color: "Violet/White".to_string(),
        qualities: vec![
            "Spiritual connection".to_string(),
            "Unity consciousness".to_string(),
            "Transcendence".to_string(),
            "Inner wisdom".to_string(),
            "Connection to the divine".to_string(),
        ],
        balanced_signs: vec![
            "Sense of spiritual connection".to_string(),
            "Open-mindedness".to_string(),
            "Ability to be present".to_string(),
            "Trust in life's unfolding".to_string(),
        ],
        imbalanced_signs: vec![
            "Spiritual disconnection".to_string(),
            "Feeling lost or purposeless".to_string(),
            "Closed-mindedness".to_string(),
            "Attachment to material world".to_string(),
        ],
        body_systems: vec![
            "Cerebral cortex".to_string(),
            "Central nervous system".to_string(),
            "Pineal gland".to_string(),
        ],
    });
    
    map
}

fn build_metric_interpretations() -> HashMap<String, MetricInterpretation> {
    let mut map = HashMap::new();
    
    map.insert("fractal_dimension".to_string(), MetricInterpretation {
        name: "Fractal Dimension".to_string(),
        description: "Measures the complexity and self-similarity of biofield patterns. \
            Healthy biofields exhibit fractal patterns similar to those found in nature.".to_string(),
        low_meaning: "Low fractal dimension (< 1.3) may indicate depleted energy or \
            simplified patterns, potentially associated with fatigue or illness.".to_string(),
        optimal_meaning: "Optimal fractal dimension (1.4-1.7) suggests complex, \
            healthy energy patterns with good vitality and resilience.".to_string(),
        high_meaning: "Very high fractal dimension (> 1.8) may indicate chaotic or \
            unstable patterns, potentially associated with stress or overstimulation.".to_string(),
        optimal_range: (1.4, 1.7),
    });
    
    map.insert("entropy".to_string(), MetricInterpretation {
        name: "Entropy".to_string(),
        description: "Shannon entropy of the biofield color distribution. \
            Measures the information content and variability of the energy field.".to_string(),
        low_meaning: "Low entropy (< 0.3) indicates uniform patterns, potentially \
            suggesting stagnation or blocked energy flow.".to_string(),
        optimal_meaning: "Balanced entropy (0.4-0.7) suggests healthy variety in \
            energy expression with organized complexity.".to_string(),
        high_meaning: "High entropy (> 0.8) may indicate scattered or disorganized \
            energy patterns.".to_string(),
        optimal_range: (0.4, 0.7),
    });
    
    map.insert("coherence".to_string(), MetricInterpretation {
        name: "Coherence".to_string(),
        description: "Measures the phase alignment of biofield interference patterns. \
            High coherence indicates synchronized, aligned energy flow.".to_string(),
        low_meaning: "Low coherence (< 0.4) suggests fragmented or misaligned energy \
            patterns, potentially associated with stress or disharmony.".to_string(),
        optimal_meaning: "Good coherence (0.5-0.8) indicates well-aligned energy flow \
            and integration between body systems.".to_string(),
        high_meaning: "Very high coherence (> 0.9) is rare and may indicate deep \
            meditative states or peak coherence moments.".to_string(),
        optimal_range: (0.5, 0.8),
    });
    
    map.insert("symmetry".to_string(), MetricInterpretation {
        name: "Symmetry".to_string(),
        description: "Left-right symmetry of the biofield. Balanced symmetry \
            indicates equal energy flow on both sides of the body.".to_string(),
        low_meaning: "Low symmetry (< 0.4) indicates significant imbalance between \
            left and right sides, potentially associated with hemispheric or \
            energetic imbalances.".to_string(),
        optimal_meaning: "Good symmetry (0.6-0.9) suggests balanced energy \
            distribution and integrated function.".to_string(),
        high_meaning: "Perfect symmetry (1.0) is rare; slight asymmetry is \
            natural and healthy.".to_string(),
        optimal_range: (0.6, 0.9),
    });
    
    map.insert("vitality_index".to_string(), MetricInterpretation {
        name: "Vitality Index".to_string(),
        description: "Composite score calculated from fractal dimension, entropy, \
            coherence, and symmetry. Provides an overall health indicator.".to_string(),
        low_meaning: "Low vitality (< 0.4) suggests depleted energy or \
            suboptimal biofield health.".to_string(),
        optimal_meaning: "Good vitality (0.5-0.8) indicates healthy, \
            balanced biofield functioning.".to_string(),
        high_meaning: "High vitality (> 0.85) suggests excellent biofield \
            health and strong life force.".to_string(),
        optimal_range: (0.5, 0.8),
    });
    
    map
}

/// Documentation for full PIP implementation (future)
pub fn pip_algorithm_documentation() -> &'static str {
    r#"
# PIP (Polycontrast Interference Photography) Algorithm Documentation

## Overview
PIP technology captures and analyzes the interference patterns created when 
coherent light interacts with the human biofield. This documentation describes 
the algorithms that would be used in a full implementation.

## Image Capture
1. Subject is illuminated with polarized light sources
2. Camera captures interference patterns at 30fps
3. Images are processed to extract color and pattern data

## Analysis Algorithms

### Fractal Dimension Calculation
- Box-counting method applied to edge-detected patterns
- Computed across multiple scales (2-256 pixel boxes)
- Regression slope gives fractal dimension D
- D = log(N) / log(1/r), where N = box count, r = box size

### Entropy Calculation  
- Color histogram computed (256 bins per channel)
- Shannon entropy: H = -Σ p(x) log₂ p(x)
- Normalized to 0-1 range

### Coherence Measurement
- 2D FFT applied to grayscale pattern
- Phase coherence computed from power spectrum
- Cross-correlation between sequential frames
- Temporal coherence averaged over capture window

### Symmetry Analysis
- Image split at midline
- Structural similarity index (SSIM) computed
- Normalized difference of left/right histograms

### Chakra Detection
- Region of interest (ROI) defined for each chakra location
- Color analysis within ROI
- Activity level from local intensity variance
- Balance from left/right ROI comparison

## Hardware Requirements
- High-sensitivity camera (10-bit minimum)
- Polarizing filters (circular)
- Controlled lighting environment
- Calibration targets

## Calibration
- White balance with reference target
- Geometric correction for lens distortion
- Color space calibration (sRGB)
- Sensitivity normalization

## Current Status
This engine currently returns mock data. Full implementation requires:
1. PIP camera hardware integration
2. Real-time image processing pipeline
3. Calibration and validation protocols
4. Clinical validation studies
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chakra_wisdom_complete() {
        let wisdom = chakra_wisdom();
        assert_eq!(wisdom.len(), 7);
        
        for chakra in Chakra::all() {
            let cw = wisdom.get(&chakra).expect(&format!("{:?} wisdom missing", chakra));
            assert!(!cw.name.is_empty());
            assert!(!cw.location.is_empty());
            assert!(!cw.qualities.is_empty());
            assert!(!cw.balanced_signs.is_empty());
            assert!(!cw.imbalanced_signs.is_empty());
        }
    }
    
    #[test]
    fn test_metric_interpretations_complete() {
        let interps = metric_interpretations();
        
        let expected_metrics = vec![
            "fractal_dimension",
            "entropy", 
            "coherence",
            "symmetry",
            "vitality_index",
        ];
        
        for metric in expected_metrics {
            let interp = interps.get(metric).expect(&format!("{} interpretation missing", metric));
            assert!(!interp.description.is_empty());
            assert!(interp.optimal_range.0 < interp.optimal_range.1);
        }
    }
    
    #[test]
    fn test_get_chakra_wisdom() {
        let heart = get_chakra_wisdom(Chakra::Heart).unwrap();
        assert!(heart.name.contains("Anahata"));
        assert_eq!(heart.element, "Air");
    }
    
    #[test]
    fn test_pip_documentation() {
        let docs = pip_algorithm_documentation();
        assert!(docs.contains("PIP"));
        assert!(docs.contains("Fractal Dimension"));
        assert!(docs.contains("mock data"));
    }
}
