# Gene Keys Frequency Assessment - Quick Reference

## Overview
Framework for consciousness frequency assessment (Shadow/Gift/Siddhi) and transformation pathway guidance. **NON-DETERMINISTIC** - frequencies depend on consciousness level, not birth data.

---

## Core Concepts

### Frequency Spectrum
- **Shadow**: Reactive unconscious pattern (level 0-2)
- **Gift**: Constructive conscious expression (level 3-4)
- **Siddhi**: Transcendent realization (level 5-6)

### Key Principle
Unlike HD Type/Authority (deterministic), Gene Keys frequencies are:
- ❌ NOT predictable from birth data alone
- ✅ Depend on consciousness evolution
- ✅ Self-identified through recognition prompts
- ✅ Suggested as starting point only

---

## API Reference

### 1. Frequency Assessment

```rust
use engine_gene_keys::{assess_frequencies, Frequency, FrequencyAssessment};

// Without consciousness level (no suggestion)
let assessments = assess_frequencies(&chart, None);

// With consciousness level 0-6 (suggestion provided)
let assessments = assess_frequencies(&chart, Some(3));

// Access assessment data
for assessment in assessments {
    println!("Gene Key {}: {}", assessment.gene_key, assessment.name);
    println!("Shadow: {} - {}", assessment.shadow, assessment.shadow_description);
    println!("Gift: {} - {}", assessment.gift, assessment.gift_description);
    println!("Siddhi: {} - {}", assessment.siddhi, assessment.siddhi_description);
    println!("Suggested: {:?}", assessment.suggested_frequency);
    
    // Recognition prompts for self-identification
    for prompt in &assessment.recognition_prompts.shadow {
        println!("  {}", prompt);
    }
}
```

### 2. Transformation Pathways

```rust
use engine_gene_keys::{
    generate_transformation_pathways,
    generate_complete_pathways,
    TransformationPathway,
};

// Generate pathways based on current frequency
let pathways = generate_transformation_pathways(&assessments);

// OR generate all transitions (Shadow→Gift + Gift→Siddhi)
let complete = generate_complete_pathways(&assessments);

// Access pathway data
for pathway in pathways {
    println!("Pathway: {:?} → {:?}", 
        pathway.current_frequency, 
        pathway.next_frequency
    );
    
    println!("Core Inquiry: {}", pathway.core_inquiry);
    
    for contemplation in &pathway.contemplations {
        println!("  • {}", contemplation);
    }
    
    for practice in &pathway.witnessing_practices {
        println!("  • {}", practice);
    }
    
    if let Some(inquiry) = &pathway.shadow_to_gift_inquiry {
        println!("Shadow→Gift: {}", inquiry);
    }
    
    if let Some(inquiry) = &pathway.gift_to_siddhi_inquiry {
        println!("Gift→Siddhi: {}", inquiry);
    }
}
```

---

## Data Structures

### FrequencyAssessment
```rust
pub struct FrequencyAssessment {
    pub gene_key: u8,
    pub name: String,
    pub shadow: String,                      // e.g., "Entropy"
    pub gift: String,                        // e.g., "Freshness"
    pub siddhi: String,                      // e.g., "Beauty"
    pub shadow_description: String,          // Full archetypal text
    pub gift_description: String,            // Full archetypal text
    pub siddhi_description: String,          // Full archetypal text
    pub suggested_frequency: Option<Frequency>, // Based on consciousness_level
    pub recognition_prompts: RecognitionPrompts,
}
```

### TransformationPathway
```rust
pub struct TransformationPathway {
    pub gene_key: u8,
    pub name: String,
    pub current_frequency: Frequency,
    pub next_frequency: Frequency,
    pub core_inquiry: String,
    pub contemplations: Vec<String>,
    pub witnessing_practices: Vec<String>,
    pub shadow_to_gift_inquiry: Option<String>,
    pub gift_to_siddhi_inquiry: Option<String>,
}
```

---

## Consciousness Level Mapping

| Level | Frequency | Description |
|-------|-----------|-------------|
| 0-2   | Shadow    | Unconscious patterns dominate, reactivity |
| 3-4   | Gift      | Conscious expression emerging, witnessing capacity |
| 5-6   | Siddhi    | Transcendent awareness, impersonal serving |
| None  | (none)    | User must self-identify via prompts |

**Note**: This is a rough guide only. Actual frequency must be self-identified through recognition prompts and inner witnessing.

---

## Non-Prescriptive Language Guidelines

### ❌ NEVER Use:
- "You should do X"
- "You must practice Y"
- "Do this exercise for 30 minutes"
- Prescriptive commands
- Fixed instructions

### ✅ ALWAYS Use:
- "You might notice..."
- "What happens when..."
- "Can you witness..."
- Questions over commands
- Invitations over obligations
- Contemplation over prescription

### Example Comparison:

**❌ Prescriptive** (incorrect):
> "You should meditate on your opinions for 20 minutes daily to transcend the shadow."

**✅ Inquiry-based** (correct):
> "What happens when you notice the rigidity of your opinions without trying to change them? Can you see the larger pattern they're protecting?"

---

## Common Use Cases

### Use Case 1: New User Onboarding
```rust
// Calculate Gene Keys chart from HD
let chart = map_hd_to_gene_keys(&hd_chart);

// Initial assessment (no consciousness level)
let assessments = assess_frequencies(&chart, None);

// Present recognition prompts for self-identification
// User explores Shadow/Gift/Siddhi descriptions
// User identifies current frequency through witnessing
```

### Use Case 2: Returning User (Known Consciousness Level)
```rust
// User has consciousness level from profile
let level = user_profile.consciousness_level; // e.g., 3

// Assessment with suggestion
let assessments = assess_frequencies(&chart, Some(level));

// Generate appropriate pathways
let pathways = generate_transformation_pathways(&assessments);

// Present inquiries for current frequency transition
```

### Use Case 3: Complete Journey View
```rust
// Show full transformation arc for all active keys
let complete_pathways = generate_complete_pathways(&assessments);

// User sees both Shadow→Gift and Gift→Siddhi
// Can explore future transitions regardless of current state
```

---

## Recognition Prompt Categories

### Shadow Prompts (Witnessing Unconscious Patterns)
- "Do you notice yourself reacting unconsciously through [pattern]?"
- "When [shadow] arises, can you feel the contraction in your body?"
- "Do you find yourself caught in [shadow] without realizing until later?"

### Gift Prompts (Witnessing Conscious Expression)
- "When do you experience [gift] arising naturally, without effort?"
- "Can you notice the space between [shadow] and [gift]?"
- "What happens when [gift] flows through you?"

### Siddhi Prompts (Witnessing Transcendent Awareness)
- "Have you had moments where [gift] dissolved into [siddhi]?"
- "When [siddhi] is present, is there still a 'you' experiencing it?"
- "What remains when the personal expression of [gift] falls away?"

---

## Testing

### Run All Tests
```bash
cargo test -p engine-gene-keys --lib
```

### Run Specific Module Tests
```bash
cargo test -p engine-gene-keys --lib frequency
cargo test -p engine-gene-keys --lib transformation
```

### Run Example
```bash
cargo run --example gene_keys_frequency_assessment
```

### Automated Test Script
```bash
./scripts/test_gene_keys_frequency.sh
```

---

## Example Inquiries by Gene Key

### Gene Key 1: Entropy → Freshness → Beauty

**Shadow→Gift**:
> "What happens when you witness entropy without trying to change it? What is entropy protecting? When entropy softens, what quality naturally emerges?"

**Gift→Siddhi**:
> "When freshness is flowing, who is experiencing it? What remains when the 'doer' falls away? Can you sense beauty as a field rather than a quality you possess?"

### Gene Key 17: Opinion → Far-Sightedness → Omniscience

**Shadow→Gift**:
> "What happens when you notice the rigidity of your opinions without trying to change them? Can you see the larger pattern your opinions are protecting?"

**Gift→Siddhi**:
> "When you can hold multiple perspectives simultaneously, what dissolves? What space opens when far-sightedness releases its vantage point?"

### Gene Key 64: Confusion → Imagination → Illumination

**Shadow→Gift**:
> "What happens when you stay present with confusion without needing to resolve it? Can you feel the creative tension in not-knowing?"

**Gift→Siddhi**:
> "When imagination flows freely, is there a 'you' imagining? What opens when the personal mind dissolves into illumination?"

---

## Integration Points

### With HD Engine
```rust
use engine_human_design::calculate_chart;
use engine_gene_keys::{map_hd_to_gene_keys, assess_frequencies};

// Calculate HD chart
let hd_chart = calculate_chart(birth_data).await?;

// Map to Gene Keys
let gk_chart = map_hd_to_gene_keys(&hd_chart);

// Assess frequencies
let assessments = assess_frequencies(&gk_chart, consciousness_level);
```

### With API Layer (future)
```rust
// HTTP endpoint (to be implemented)
POST /api/gene-keys/frequency-assessment
{
  "birth_data": { ... },
  "consciousness_level": 3  // optional
}

// Response
{
  "assessments": [ ... ],
  "pathways": [ ... ]
}
```

---

## Performance Notes

- All Gene Key data loaded once at startup (lazy static)
- Assessment generation is O(n) where n = active keys (~26)
- Pathway generation is lightweight (string formatting)
- Recognition prompts generated dynamically per key
- Full descriptions included (no summarization)

---

## Future Enhancements

1. **Personalized Inquiries**: Generate inquiries based on user's specific life context
2. **Contemplation Tracking**: Track which inquiries/practices user is working with
3. **Frequency Evolution**: Track consciousness level changes over time
4. **Cross-Key Synthesis**: Generate inquiries for key pairs (programming partners)
5. **Synarchic Integration**: Connect with other consciousness engines

---

## Resources

- **Implementation**: `AGENT28_IMPLEMENTATION.md`
- **Gene Keys Data**: `data/gene-keys/archetypes.json`
- **Example**: `examples/gene_keys_frequency_assessment.rs`
- **Tests**: `crates/engine-gene-keys/src/frequency.rs` (unit tests)
- **Tests**: `crates/engine-gene-keys/src/transformation.rs` (unit tests)

---

**Last Updated**: Agent 28  
**Phase**: 3 Sprint 5 (Gene Keys Engine)
