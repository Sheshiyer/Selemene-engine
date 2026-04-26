# Human Design Witness Prompts - Complete Reference

**Purpose**: Consciousness-oriented self-inquiry questions generated dynamically based on HD chart configuration and user's consciousness development level.

---

## Prompt Generation Logic

```rust
fn generate_witness_prompt(chart: &HDChart, consciousness_level: u8) -> String {
    match consciousness_level {
        0..=2 => generate_basic_witness(chart),        // Type/Strategy
        3..=4 => generate_intermediate_witness(chart), // Profile
        5..   => generate_advanced_witness(chart),     // Authority/Definition
    }
}
```

---

## Level 0-2: Basic Body Awareness (Type/Strategy)

### Generator
> "What does it feel like in your body when you wait to respond to life's invitations rather than initiating?"

### Manifesting Generator
> "How do you experience the pull to respond quickly and then skip steps once momentum builds?"

### Projector
> "How do you experience recognition when it arrives naturally, without effort or seeking?"

### Manifestor
> "Where do you notice the urge to initiate before informing others of what's coming?"

### Reflector
> "What happens in your awareness when you give yourself time to sense the full lunar cycle before deciding?"

---

## Level 3-4: Intermediate Awareness (Profile Dynamics)

### Profile 1/3 - Investigator/Martyr
> "How do you experience the dance between deep investigation and experiential learning through trial and error?"

### Profile 1/4 - Investigator/Opportunist
> "Where do you notice your need for solid foundation meeting your natural gift for sharing within networks?"

### Profile 2/4 - Hermit/Opportunist
> "What happens when your natural talent calls you out from solitude into the realm of connection and networking?"

### Profile 2/5 - Hermit/Heretic
> "How do you experience being called out to solve problems when you'd rather remain in your natural state?"

### Profile 3/5 - Martyr/Heretic
> "Where do you see yourself experimenting until breakthrough, then being projected upon to provide solutions?"

### Profile 3/6 - Martyr/Role Model
> "How do you navigate the transition from experimental engagement to observational wisdom?"

### Profile 4/6 - Opportunist/Role Model
> "What happens when your network-building nature meets your need for experimentation and eventual observation?"

### Profile 4/1 - Opportunist/Investigator
> "How do you bridge your gift for connection with your need for investigative depth and security?"

### Profile 5/1 - Heretic/Investigator
> "Where do you notice the tension between being projected upon for solutions and your need for solid foundation?"

### Profile 5/2 - Heretic/Hermit
> "How do you experience being called to solve problems when your natural state is to be called out from within?"

### Profile 6/2 - Role Model/Hermit
> "What does it feel like to be on the roof observing life while also being called down to share natural gifts?"

### Profile 6/3 - Role Model/Martyr
> "How do you dance between objective observation and the pull to experiment directly with life?"

### Generic Profile (fallback)
> "How do you experience the interplay between your conscious and unconscious life themes?"

---

## Level 5+: Advanced Awareness (Authority + Definition)

### Authority-Based Prompts

#### Sacral Authority
> "Where do you notice the sacral response arising in the present moment, distinct from mental narrative?"

#### Emotional Authority
> "How do you experience yourself riding the emotional wave before making decisions, without forcing clarity?"

#### Splenic Authority
> "What is it like to trust the instantaneous knowing that arises and vanishes in a single moment?"

#### Heart/Ego Authority
> "Where do you feel the alignment between what you will commit to and your authentic power?"

#### G-Center/Self Authority
> "How do you distinguish between true direction arising from your G center versus mental constructs?"

#### Mental/Environmental Authority
> "What happens when you verbalize your thoughts in different environments before arriving at clarity?"

#### Lunar Authority
> "How do you experience the full lunar cycle revealing consistent truth beyond transient impressions?"

### Definition-Based Additions

Authority prompts at level 5+ are enhanced with definition-specific awareness:

#### Single Definition
*No additional layer*

#### Split Definition
> " And how do you notice the bridging energy when others enter your field, connecting what feels separate?"

#### Triple Split
> " And what is your experience of needing multiple bridges to feel a sense of wholeness?"

#### Quadruple Split
> " And how do you witness the complexity of multiple separate islands within your design seeking connection?"

#### No Definition (Reflector)
> " And what is it like to be completely open, sampling and reflecting the energy around you?"

---

## Example Combinations

### Example 1: Generator 1/3, Sacral Authority, Single Definition
- **Level 1**: "What does it feel like in your body when you wait to respond to life's invitations rather than initiating?"
- **Level 3**: "How do you experience the dance between deep investigation and experiential learning through trial and error?"
- **Level 5**: "Where do you notice the sacral response arising in the present moment, distinct from mental narrative?"

### Example 2: Projector 6/2, Emotional Authority, Split Definition
- **Level 1**: "How do you experience recognition when it arrives naturally, without effort or seeking?"
- **Level 3**: "What does it feel like to be on the roof observing life while also being called down to share natural gifts?"
- **Level 5**: "How do you experience yourself riding the emotional wave before making decisions, without forcing clarity? And how do you notice the bridging energy when others enter your field, connecting what feels separate?"

### Example 3: Manifestor 4/6, Heart Authority, Single Definition
- **Level 1**: "Where do you notice the urge to initiate before informing others of what's coming?"
- **Level 3**: "What happens when your network-building nature meets your need for experimentation and eventual observation?"
- **Level 5**: "Where do you feel the alignment between what you will commit to and your authentic power?"

### Example 4: Reflector 3/5, Lunar Authority, No Definition
- **Level 1**: "What happens in your awareness when you give yourself time to sense the full lunar cycle before deciding?"
- **Level 3**: "Where do you see yourself experimenting until breakthrough, then being projected upon to provide solutions?"
- **Level 5**: "How do you experience the full lunar cycle revealing consistent truth beyond transient impressions? And what is it like to be completely open, sampling and reflecting the energy around you?"

---

## Design Philosophy

### Inquiry vs. Affirmation
All prompts are questions, not statements. This cultivates:
- **Self-observation** over self-identification
- **Experiential awareness** over conceptual understanding
- **Present-moment noticing** over past-based narratives

### Progressive Depth
- **Level 1-2**: Somatic/body awareness (Type/Strategy)
- **Level 3-4**: Relational patterns (Profile)
- **Level 5+**: Energetic mechanics (Authority/Definition)

Users naturally progress through levels as consciousness develops.

### Non-Prescriptive Language
Prompts avoid:
- ❌ "You should..."
- ❌ "You are..."
- ❌ "You need to..."

Instead use:
- ✅ "What do you notice..."
- ✅ "How do you experience..."
- ✅ "Where do you feel..."

### Rule 5 Compliance
Per `ai-rules.md`, witness_prompt field must NEVER be empty. All 140+ combinations (5 types × 7 authorities × 4 definitions + 12 profiles) generate valid inquiry questions.

---

## Technical Implementation

### Code Structure
```
witness.rs
├── generate_witness_prompt()      // Main entry point
├── generate_basic_witness()       // Level 0-2
├── generate_intermediate_witness() // Level 3-4
└── generate_advanced_witness()    // Level 5+
```

### Test Coverage
```rust
#[test]
fn test_all_prompts_non_empty() {
    // Tests 5 types × 7 authorities × 3 levels = 105 combinations
    // Plus all 12 profile variations
    // Total: 117+ prompt combinations validated
}
```

### Usage in ConsciousnessEngine
```rust
async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
    let chart = generate_hd_chart(...)?;
    
    let consciousness_level = input.options
        .get("consciousness_level")
        .and_then(|v| v.as_u64())
        .map(|v| v as u8)
        .unwrap_or(1);
    
    let witness_prompt = generate_witness_prompt(&chart, consciousness_level);
    
    // Enforce non-empty (Rule 5)
    if witness_prompt.is_empty() {
        return Err(EngineError::CalculationFailed("Empty witness prompt".into()));
    }
    
    // ... return EngineOutput with witness_prompt
}
```

---

## API Integration

### Request
```json
{
  "birth_data": { ... },
  "options": {
    "consciousness_level": 3
  }
}
```

### Response
```json
{
  "engine_id": "human-design",
  "result": { ... },
  "witness_prompt": "What does it feel like to be on the roof observing life while also being called down to share natural gifts?",
  "consciousness_level": 3,
  "metadata": { ... }
}
```

---

## Future Enhancements

### Potential Additions (Not Yet Implemented)
- [ ] Incarnation Cross-specific prompts (Level 6+)
- [ ] Gate-specific contemplations (Level 7+)
- [ ] Line-specific inquiries (Level 8+)
- [ ] Circuitry awareness prompts (Level 9+)
- [ ] Variable-specific questions (Level 10+)

### Localization (Future)
- [ ] Spanish translations
- [ ] German translations
- [ ] French translations
- [ ] Portuguese translations

---

## References

### Human Design Authorities
1. **Sacral**: Gut response (uh-huh/uh-uh)
2. **Emotional**: Wave clarity over time
3. **Splenic**: Instantaneous intuition
4. **Heart/Ego**: Willpower commitment
5. **G-Center/Self**: Identity direction
6. **Mental/Environmental**: Sounding board
7. **Lunar**: 28-day cycle sampling

### Profile Lines
- **Line 1**: Investigation - foundation
- **Line 2**: Hermit - natural talent
- **Line 3**: Martyr - experimentation
- **Line 4**: Opportunist - networking
- **Line 5**: Heretic - problem-solving
- **Line 6**: Role Model - observation

### Definition Types
- **Single**: All centers connected
- **Split**: Two separate areas
- **Triple Split**: Three separate areas
- **Quadruple Split**: Four separate areas
- **No Definition**: All centers open (Reflector only)

---

## Conclusion

The witness prompt system provides 140+ unique consciousness-oriented inquiries tailored to each individual's HD configuration and development level. This creates a personalized meditation practice that evolves with the user's awareness journey.

**Core Principle**: Ask questions that cultivate presence, not answers that reinforce identity.

---

**Related Files**:
- `src/witness.rs` - Implementation
- `src/engine.rs` - Integration with ConsciousnessEngine
- `AGENT_24_COMPLETION_SUMMARY.md` - Full implementation report
