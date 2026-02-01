//! Mock biofield metrics generation
//!
//! Generates plausible biofield metrics for testing and demonstration.
//! Uses seeded random for reproducibility.

use chrono::Utc;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::models::{BiofieldMetrics, ChakraReading, Chakra};

/// Generate mock biofield metrics with optional seed for reproducibility
///
/// # Arguments
/// * `seed` - Optional seed for reproducible random generation
///
/// # Returns
/// BiofieldMetrics with plausible values
pub fn generate_mock_metrics(seed: Option<u64>) -> BiofieldMetrics {
    let mut rng: StdRng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };
    
    // Generate base metrics within realistic ranges
    let fractal_dimension = generate_fractal_dimension(&mut rng);
    let entropy = generate_entropy(&mut rng);
    let coherence = generate_coherence(&mut rng);
    let symmetry = generate_symmetry(&mut rng);
    
    // Calculate composite vitality index
    let vitality_index = calculate_vitality_index(
        fractal_dimension,
        entropy,
        coherence,
        symmetry,
    );
    
    // Generate chakra readings
    let chakra_readings = generate_chakra_readings(&mut rng);
    
    BiofieldMetrics {
        fractal_dimension,
        entropy,
        coherence,
        symmetry,
        vitality_index,
        chakra_readings,
        timestamp: Utc::now(),
    }
}

/// Generate fractal dimension (1.0-2.0, optimal ~1.5)
fn generate_fractal_dimension(rng: &mut StdRng) -> f64 {
    // Use normal-ish distribution centered around 1.5
    let base: f64 = 1.5;
    let variation: f64 = rng.gen_range(-0.3..0.3);
    (base + variation).clamp(1.0, 2.0)
}

/// Generate entropy (0.0-1.0, optimal ~0.55)
fn generate_entropy(rng: &mut StdRng) -> f64 {
    // Center around optimal with some variation
    let base: f64 = 0.55;
    let variation: f64 = rng.gen_range(-0.25..0.25);
    (base + variation).clamp(0.0, 1.0)
}

/// Generate coherence (0.0-1.0, optimal ~0.65)
fn generate_coherence(rng: &mut StdRng) -> f64 {
    let base: f64 = 0.65;
    let variation: f64 = rng.gen_range(-0.3..0.3);
    (base + variation).clamp(0.0, 1.0)
}

/// Generate symmetry (0.0-1.0, optimal ~0.75)
fn generate_symmetry(rng: &mut StdRng) -> f64 {
    let base: f64 = 0.75;
    let variation: f64 = rng.gen_range(-0.2..0.2);
    (base + variation).clamp(0.0, 1.0)
}

/// Calculate composite vitality index from component metrics
///
/// Uses weighted average with coherence and fractal dimension weighted higher
fn calculate_vitality_index(
    fractal_dimension: f64,
    entropy: f64,
    coherence: f64,
    symmetry: f64,
) -> f64 {
    // Normalize fractal dimension to 0-1 scale
    let fd_normalized = (fractal_dimension - 1.0).clamp(0.0, 1.0);
    
    // Weights: coherence (0.3), fractal (0.3), entropy (0.2), symmetry (0.2)
    let weighted_sum = 
        coherence * 0.30 +
        fd_normalized * 0.30 +
        entropy * 0.20 +
        symmetry * 0.20;
    
    // Apply slight non-linearity to spread values
    (weighted_sum.powf(0.9)).clamp(0.0, 1.0)
}

/// Generate readings for all 7 chakras
fn generate_chakra_readings(rng: &mut StdRng) -> Vec<ChakraReading> {
    Chakra::all()
        .into_iter()
        .map(|chakra| generate_chakra_reading(rng, chakra))
        .collect()
}

/// Generate a single chakra reading
fn generate_chakra_reading(rng: &mut StdRng, chakra: Chakra) -> ChakraReading {
    let activity_level = rng.gen_range(0.3..0.9);
    let balance = rng.gen_range(-0.5..0.5);
    let color_intensity = generate_color_intensity(rng, chakra, activity_level);
    
    ChakraReading {
        chakra,
        activity_level,
        balance,
        color_intensity,
    }
}

/// Generate appropriate color intensity based on chakra and activity
fn generate_color_intensity(rng: &mut StdRng, chakra: Chakra, activity: f64) -> String {
    let base_color = match chakra {
        Chakra::Root => "red",
        Chakra::Sacral => "orange",
        Chakra::SolarPlexus => "yellow",
        Chakra::Heart => "green",
        Chakra::Throat => "blue",
        Chakra::ThirdEye => "indigo",
        Chakra::Crown => "violet",
    };
    
    let intensity = if activity > 0.7 {
        "bright"
    } else if activity > 0.5 {
        "moderate"
    } else if activity > 0.3 {
        "dim"
    } else {
        "faint"
    };
    
    // Occasionally add variation
    let variant = if rng.gen_bool(0.2) {
        match rng.gen_range(0..3) {
            0 => " with white highlights",
            1 => " with some muddiness",
            _ => " pulsating",
        }
    } else {
        ""
    };
    
    format!("{} {}{}", intensity, base_color, variant)
}

/// Generate mock metrics based on a user ID for consistent personal readings
pub fn generate_metrics_for_user(user_id: &str) -> BiofieldMetrics {
    // Create seed from user ID hash
    let seed = user_id.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    generate_mock_metrics(Some(seed))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_mock_metrics_seeded() {
        let metrics1 = generate_mock_metrics(Some(42));
        let metrics2 = generate_mock_metrics(Some(42));
        
        assert_eq!(metrics1.fractal_dimension, metrics2.fractal_dimension);
        assert_eq!(metrics1.entropy, metrics2.entropy);
        assert_eq!(metrics1.coherence, metrics2.coherence);
        assert_eq!(metrics1.symmetry, metrics2.symmetry);
    }
    
    #[test]
    fn test_generate_mock_metrics_different_seeds() {
        let metrics1 = generate_mock_metrics(Some(42));
        let metrics2 = generate_mock_metrics(Some(123));
        
        // Very unlikely to be exactly equal with different seeds
        assert_ne!(metrics1.fractal_dimension, metrics2.fractal_dimension);
    }
    
    #[test]
    fn test_metrics_in_valid_ranges() {
        for seed in 0..100 {
            let metrics = generate_mock_metrics(Some(seed));
            
            assert!(metrics.fractal_dimension >= 1.0 && metrics.fractal_dimension <= 2.0,
                "fractal_dimension {} out of range", metrics.fractal_dimension);
            assert!(metrics.entropy >= 0.0 && metrics.entropy <= 1.0,
                "entropy {} out of range", metrics.entropy);
            assert!(metrics.coherence >= 0.0 && metrics.coherence <= 1.0,
                "coherence {} out of range", metrics.coherence);
            assert!(metrics.symmetry >= 0.0 && metrics.symmetry <= 1.0,
                "symmetry {} out of range", metrics.symmetry);
            assert!(metrics.vitality_index >= 0.0 && metrics.vitality_index <= 1.0,
                "vitality_index {} out of range", metrics.vitality_index);
        }
    }
    
    #[test]
    fn test_chakra_readings_complete() {
        let metrics = generate_mock_metrics(Some(42));
        assert_eq!(metrics.chakra_readings.len(), 7);
        
        for reading in &metrics.chakra_readings {
            assert!(reading.activity_level >= 0.0 && reading.activity_level <= 1.0);
            assert!(reading.balance >= -1.0 && reading.balance <= 1.0);
            assert!(!reading.color_intensity.is_empty());
        }
    }
    
    #[test]
    fn test_chakra_readings_have_all_chakras() {
        let metrics = generate_mock_metrics(Some(42));
        let chakras: Vec<Chakra> = metrics.chakra_readings.iter()
            .map(|r| r.chakra)
            .collect();
        
        assert!(chakras.contains(&Chakra::Root));
        assert!(chakras.contains(&Chakra::Sacral));
        assert!(chakras.contains(&Chakra::SolarPlexus));
        assert!(chakras.contains(&Chakra::Heart));
        assert!(chakras.contains(&Chakra::Throat));
        assert!(chakras.contains(&Chakra::ThirdEye));
        assert!(chakras.contains(&Chakra::Crown));
    }
    
    #[test]
    fn test_vitality_index_calculation() {
        // Optimal values should give high vitality
        let high_vitality = calculate_vitality_index(1.5, 0.55, 0.7, 0.8);
        assert!(high_vitality > 0.5, "Optimal values should give vitality > 0.5");
        
        // Low values should give lower vitality
        let low_vitality = calculate_vitality_index(1.1, 0.2, 0.3, 0.3);
        assert!(low_vitality < high_vitality, "Low values should give lower vitality");
    }
    
    #[test]
    fn test_generate_metrics_for_user() {
        let metrics1 = generate_metrics_for_user("user123");
        let metrics2 = generate_metrics_for_user("user123");
        
        assert_eq!(metrics1.fractal_dimension, metrics2.fractal_dimension);
        
        let metrics3 = generate_metrics_for_user("differentuser");
        assert_ne!(metrics1.fractal_dimension, metrics3.fractal_dimension);
    }
}
