//! Fasting rules and guidelines

use super::FastType;

/// Get fasting rules for a type
pub fn get_fasting_rules(fast_type: &FastType) -> FastingRules {
    match fast_type {
        FastType::Nirjala => FastingRules {
            name: "Nirjala Fast".to_string(),
            description: "Complete fast without food or water".to_string(),
            duration: "24 hours or sunrise to sunrise".to_string(),
            allowed: vec!["Nothing (strict observance)".to_string()],
            restricted: vec!["All food and water".to_string()],
            breaking_guidelines: vec![
                "Break with water first".to_string(),
                "Then have light fruits".to_string(),
                "Avoid heavy meals immediately".to_string(),
            ],
            health_notes: vec![
                "Not recommended for elderly".to_string(),
                "Not for pregnant women".to_string(),
                "Not for those with health conditions".to_string(),
                "Consult doctor if unsure".to_string(),
            ],
            spiritual_benefits: vec![
                "Maximum spiritual merit".to_string(),
                "Complete detoxification".to_string(),
                "Heightened spiritual awareness".to_string(),
            ],
        },
        FastType::Phalahara => FastingRules {
            name: "Phalahara (Fruit Fast)".to_string(),
            description: "Only fruits, milk, and select items allowed".to_string(),
            duration: "One day or as specified".to_string(),
            allowed: vec![
                "Fresh fruits".to_string(),
                "Dry fruits and nuts".to_string(),
                "Milk and milk products".to_string(),
                "Sabudana".to_string(),
                "Potato (sendha namak)".to_string(),
            ],
            restricted: vec![
                "Grains (rice, wheat)".to_string(),
                "Regular salt".to_string(),
                "Onion and garlic".to_string(),
                "Non-vegetarian food".to_string(),
            ],
            breaking_guidelines: vec![
                "End at prescribed time".to_string(),
                "Start with light meal".to_string(),
            ],
            health_notes: vec![
                "Generally safe for most people".to_string(),
                "Good hydration from fruits".to_string(),
            ],
            spiritual_benefits: vec![
                "Purification of body and mind".to_string(),
                "Sattvic energy increase".to_string(),
            ],
        },
        FastType::Ekadashi => FastingRules {
            name: "Ekadashi Fast".to_string(),
            description: "Abstain from grains and beans".to_string(),
            duration: "From sunrise on Ekadashi to sunrise next day".to_string(),
            allowed: vec![
                "Fruits".to_string(),
                "Milk products".to_string(),
                "Sendha namak".to_string(),
                "Kuttu (buckwheat)".to_string(),
                "Singhara (water chestnut)".to_string(),
                "Sabudana".to_string(),
            ],
            restricted: vec![
                "Rice and wheat".to_string(),
                "All grains".to_string(),
                "Beans and lentils".to_string(),
                "Regular salt".to_string(),
                "Onion and garlic".to_string(),
            ],
            breaking_guidelines: vec![
                "Break only during Dwadashi".to_string(),
                "Check parana time".to_string(),
                "Avoid breaking during hari vasara".to_string(),
            ],
            health_notes: vec![
                "Bi-monthly detox".to_string(),
                "Good for digestive system".to_string(),
            ],
            spiritual_benefits: vec![
                "Most dear to Lord Vishnu".to_string(),
                "Destroys sins of many births".to_string(),
                "Grants moksha".to_string(),
            ],
        },
        FastType::AnnaVrat => FastingRules {
            name: "Anna Vrat (No Grains)".to_string(),
            description: "Abstain from all grains".to_string(),
            duration: "As specified by the vrat".to_string(),
            allowed: vec![
                "Fruits and vegetables".to_string(),
                "Milk products".to_string(),
                "Nuts".to_string(),
            ],
            restricted: vec![
                "All grains".to_string(),
                "Flour products".to_string(),
            ],
            breaking_guidelines: vec!["Break at prescribed time".to_string()],
            health_notes: vec!["Manageable for most people".to_string()],
            spiritual_benefits: vec!["Purification and discipline".to_string()],
        },
        FastType::DudhVrat => FastingRules {
            name: "Dudh Vrat (Milk Fast)".to_string(),
            description: "Only milk and milk products".to_string(),
            duration: "One day typically".to_string(),
            allowed: vec![
                "Milk".to_string(),
                "Buttermilk".to_string(),
                "Curd/Yogurt".to_string(),
                "Paneer".to_string(),
            ],
            restricted: vec!["All solid foods".to_string(), "Non-dairy items".to_string()],
            breaking_guidelines: vec![
                "End with light meal".to_string(),
                "Avoid heavy food immediately".to_string(),
            ],
            health_notes: vec![
                "Good protein intake".to_string(),
                "Not for lactose intolerant".to_string(),
            ],
            spiritual_benefits: vec!["Sattvic nourishment".to_string()],
        },
        FastType::Partial => FastingRules {
            name: "Partial Fast".to_string(),
            description: "Reduced eating, typically one meal".to_string(),
            duration: "Varies".to_string(),
            allowed: vec!["Light sattvic food".to_string(), "As per specific vrat".to_string()],
            restricted: vec!["Heavy meals".to_string(), "Non-veg".to_string()],
            breaking_guidelines: vec!["Follow specific vrat rules".to_string()],
            health_notes: vec!["Easiest to follow".to_string()],
            spiritual_benefits: vec!["Discipline and mindfulness".to_string()],
        },
    }
}

/// Fasting rules structure
#[derive(Debug, Clone)]
pub struct FastingRules {
    pub name: String,
    pub description: String,
    pub duration: String,
    pub allowed: Vec<String>,
    pub restricted: Vec<String>,
    pub breaking_guidelines: Vec<String>,
    pub health_notes: Vec<String>,
    pub spiritual_benefits: Vec<String>,
}

/// Check if fasting is recommended for a person
pub fn can_observe_fast(
    fast_type: &FastType,
    age: u8,
    is_pregnant: bool,
    has_health_issues: bool,
) -> (bool, String) {
    match fast_type {
        FastType::Nirjala => {
            if is_pregnant {
                (false, "Nirjala fast not recommended during pregnancy".to_string())
            } else if age < 12 || age > 70 {
                (false, "Nirjala fast not recommended for this age group".to_string())
            } else if has_health_issues {
                (false, "Consult doctor before Nirjala fast".to_string())
            } else {
                (true, "Can observe with proper preparation".to_string())
            }
        }
        FastType::Phalahara | FastType::Ekadashi => {
            if is_pregnant {
                (true, "Can observe with modifications - ensure adequate nutrition".to_string())
            } else {
                (true, "Safe to observe".to_string())
            }
        }
        _ => (true, "Generally safe to observe".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fasting_rules() {
        let rules = get_fasting_rules(&FastType::Ekadashi);
        assert!(rules.restricted.iter().any(|r| r.contains("Rice")));
    }

    #[test]
    fn test_can_observe() {
        let (can, _) = can_observe_fast(&FastType::Nirjala, 30, false, false);
        assert!(can);
        
        let (can, _) = can_observe_fast(&FastType::Nirjala, 30, true, false);
        assert!(!can);
    }
}
