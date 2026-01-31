//! Gene Keys Wisdom Data Loader
//!
//! Loads the 64 Gene Keys archetypes with full shadow/gift/siddhi descriptions.
//! Preserves archetypal depth - NO TEXT SUMMARIZATION.

use crate::models::{GeneKey, GeneKeysData};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Static reference to all 64 Gene Keys wisdom data
static GENE_KEYS: OnceLock<HashMap<u8, GeneKey>> = OnceLock::new();

/// Get reference to all Gene Keys wisdom data
pub fn gene_keys() -> &'static HashMap<u8, GeneKey> {
    GENE_KEYS.get_or_init(|| {
        load_gene_keys().expect("Failed to load Gene Keys wisdom data")
    })
}

/// Get a specific Gene Key by number (1-64)
pub fn get_gene_key(number: u8) -> Option<&'static GeneKey> {
    gene_keys().get(&number)
}

/// Load Gene Keys from embedded JSON file
fn load_gene_keys() -> Result<HashMap<u8, GeneKey>, Box<dyn std::error::Error>> {
    // Embedded JSON data at compile time
    const ARCHETYPES_JSON: &str = include_str!("../../../data/gene-keys/archetypes.json");
    
    let data: GeneKeysData = serde_json::from_str(ARCHETYPES_JSON)?;
    
    // Convert string keys to u8 keys
    let mut gene_keys_map = HashMap::new();
    
    for (key_str, mut gene_key) in data.gene_keys {
        let key_num = key_str.parse::<u8>()?;
        
        // Ensure number field matches key
        gene_key.number = key_num;
        
        gene_keys_map.insert(key_num, gene_key);
    }
    
    // Validate we have all 64 keys
    if gene_keys_map.len() != 64 {
        return Err(format!(
            "Expected 64 Gene Keys, found {}",
            gene_keys_map.len()
        )
        .into());
    }
    
    // Validate key numbers 1-64 are present
    for i in 1..=64 {
        if !gene_keys_map.contains_key(&i) {
            return Err(format!("Missing Gene Key {}", i).into());
        }
    }
    
    Ok(gene_keys_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_all_gene_keys() {
        let keys = gene_keys();
        assert_eq!(keys.len(), 64, "Should have exactly 64 Gene Keys");
    }
    
    #[test]
    fn test_gene_key_structure() {
        let key_1 = get_gene_key(1).expect("Gene Key 1 should exist");
        
        assert_eq!(key_1.number, 1);
        assert_eq!(key_1.shadow, "Entropy");
        assert_eq!(key_1.gift, "Freshness");
        assert_eq!(key_1.siddhi, "Beauty");
        
        // Verify descriptions are NOT empty (archetypal depth preserved)
        assert!(!key_1.shadow_description.is_empty());
        assert!(!key_1.gift_description.is_empty());
        assert!(!key_1.siddhi_description.is_empty());
        
        // Verify descriptions have substance (not just placeholders)
        assert!(
            key_1.shadow_description.len() > 50,
            "Shadow description too short - archetypal depth not preserved"
        );
    }
    
    #[test]
    fn test_programming_partners() {
        let key_1 = get_gene_key(1).expect("Gene Key 1 should exist");
        assert_eq!(key_1.programming_partner, Some(33));
        
        let key_17 = get_gene_key(17).expect("Gene Key 17 should exist");
        assert_eq!(key_17.programming_partner, Some(49));
    }
    
    #[test]
    fn test_all_keys_present() {
        for i in 1..=64 {
            let key = get_gene_key(i);
            assert!(
                key.is_some(),
                "Gene Key {} should be present",
                i
            );
        }
    }
    
    #[test]
    fn test_archetypal_depth_preservation() {
        // Spot check several keys for full descriptions
        let test_keys = [1, 17, 33, 47, 64];
        
        for &key_num in &test_keys {
            let key = get_gene_key(key_num).expect(&format!("Gene Key {} missing", key_num));
            
            // Descriptions should be substantial (typical: 100-500 words)
            assert!(
                key.shadow_description.len() > 50,
                "Gene Key {} shadow too short",
                key_num
            );
            assert!(
                key.gift_description.len() > 50,
                "Gene Key {} gift too short",
                key_num
            );
            assert!(
                key.siddhi_description.len() > 50,
                "Gene Key {} siddhi too short",
                key_num
            );
        }
    }
}
