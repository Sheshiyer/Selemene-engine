//! Witness Prompt Generation for Workflow Synthesis
//!
//! Generates self-inquiry prompts that invite observing cross-system patterns.
//! These prompts are consciousness-level-appropriate and focus on:
//! - Noticing patterns across multiple lenses
//! - Exploring tensions between systems
//! - Shifting relationship to insights

use crate::workflow::models::{InquiryType, SynthesisResult, WitnessPrompt};

/// Generate witness prompts from synthesis results
pub fn generate_workflow_witness_prompts(
    synthesis: &SynthesisResult,
    consciousness_level: u8,
) -> Vec<WitnessPrompt> {
    let mut prompts = Vec::new();

    // Generate theme-based prompts
    for theme in synthesis.themes.iter().filter(|t| t.strength >= 0.4) {
        prompts.push(generate_theme_prompt(theme, consciousness_level));
    }

    // Generate tension-based prompts
    for tension in &synthesis.tensions {
        prompts.push(generate_tension_prompt(tension, consciousness_level));
    }

    // Generate integration prompt if there are alignments
    if !synthesis.alignments.is_empty() {
        prompts.push(generate_alignment_prompt(&synthesis.alignments, consciousness_level));
    }

    // Generate a general synthesis prompt
    prompts.push(generate_synthesis_prompt(synthesis, consciousness_level));

    // Limit to most relevant prompts
    prompts.truncate(4);

    prompts
}

/// Generate prompt for a cross-engine theme
fn generate_theme_prompt(
    theme: &crate::workflow::models::Theme,
    level: u8,
) -> WitnessPrompt {
    let sources_text = if theme.sources.len() > 1 {
        format!("{} different lenses", theme.sources.len())
    } else {
        "this lens".to_string()
    };

    let text = match level {
        0 => format!(
            "Notice what arises when you read about '{}' appearing across {}. No need to interpret — just observe what you feel.",
            theme.name, sources_text
        ),
        1 => format!(
            "What patterns do you notice in how '{}' shows up across these different systems? What feels familiar about this theme?",
            theme.name
        ),
        2 => format!(
            "Who is the one observing '{}' through these multiple lenses? Can you find the one who recognizes this pattern?",
            theme.name
        ),
        3 => format!(
            "Given that '{}' appears across {} systems, how might you consciously embody this theme rather than be run by it?",
            theme.name, sources_text
        ),
        _ => format!(
            "What wants to emerge through '{}' as you hold this pattern from multiple perspectives?",
            theme.name
        ),
    };

    WitnessPrompt::new(text, InquiryType::PatternNoticing)
        .with_context(theme.name.clone())
}

/// Generate prompt for a tension between systems
fn generate_tension_prompt(
    tension: &crate::workflow::models::Tension,
    level: u8,
) -> WitnessPrompt {
    let (system_a, _view_a) = &tension.perspective_a;
    let (system_b, _view_b) = &tension.perspective_b;

    let text = match level {
        0 => format!(
            "Notice the space between what {} shows and what {} suggests about '{}'. Just observe — where do you feel this in your body?",
            system_a, system_b, tension.aspect
        ),
        1 => format!(
            "Where do you feel the tension between {} and {} around '{}'? What does this polarity remind you of in your life?",
            system_a, system_b, tension.aspect
        ),
        2 => format!(
            "Who is aware of both {} and {} perspectives on '{}'? What is unchanging as you hold both views?",
            system_a, system_b, tension.aspect
        ),
        3 => format!(
            "How might you dance with the tension between {} and {} on '{}'? What becomes possible when you hold both as true?",
            system_a, system_b, tension.aspect
        ),
        _ => format!(
            "What wisdom lives in the space between these perspectives on '{}'?",
            tension.aspect
        ),
    };

    WitnessPrompt::new(text, InquiryType::TensionExploration)
        .with_context(tension.aspect.clone())
}

/// Generate prompt for alignments across systems
fn generate_alignment_prompt(
    alignments: &[crate::workflow::models::Alignment],
    level: u8,
) -> WitnessPrompt {
    let alignment_names: Vec<&str> = alignments.iter().map(|a| a.aspect.as_str()).collect();
    let engines: Vec<String> = alignments
        .iter()
        .flat_map(|a| a.engines.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let text = match level {
        0 => format!(
            "Notice how {} systems agree on {}. What's it like to see this convergence?",
            engines.len(),
            alignment_names.join(" and ")
        ),
        1 => format!(
            "When multiple systems point to the same thing — {} — what does that stir in you?",
            alignment_names.join(", ")
        ),
        2 => format!(
            "These systems didn't coordinate, yet they align on {}. Who is the one recognizing this pattern?",
            alignment_names.join(" and ")
        ),
        3 => format!(
            "Given this convergence on {}, what action or non-action wants to arise?",
            alignment_names.join(" and ")
        ),
        _ => format!(
            "What does it mean that {} emerges from multiple independent perspectives?",
            alignment_names.join(" and ")
        ),
    };

    WitnessPrompt::new(text, InquiryType::Understanding)
}

/// Generate overall synthesis prompt
fn generate_synthesis_prompt(synthesis: &SynthesisResult, level: u8) -> WitnessPrompt {
    let has_themes = !synthesis.themes.is_empty();
    let has_tensions = !synthesis.tensions.is_empty();
    let has_alignments = !synthesis.alignments.is_empty();

    let text = match level {
        0 => {
            "As you take in all these perspectives together, what do you notice in your body right now?"
                .to_string()
        }
        1 => {
            "Looking at your patterns from these different angles — what's becoming clearer? What questions are arising?"
                .to_string()
        }
        2 => {
            "The systems point to patterns, but who is the one these patterns belong to? Can you find that one?"
                .to_string()
        }
        3 => {
            if has_tensions && has_alignments {
                "Holding both the alignments and tensions, what response (not reaction) wants to emerge?".to_string()
            } else if has_themes {
                "Given these themes, what is one conscious choice you could make today?".to_string()
            } else {
                "What would it mean to author your day from this awareness?".to_string()
            }
        }
        _ => {
            "What wants to move through you now that you've seen yourself from these perspectives?"
                .to_string()
        }
    };

    WitnessPrompt::new(text, InquiryType::Integration)
}

/// Generate prompts specifically for Birth Blueprint workflow
pub fn generate_birth_blueprint_prompts(
    synthesis: &SynthesisResult,
    level: u8,
) -> Vec<WitnessPrompt> {
    let mut prompts = generate_workflow_witness_prompts(synthesis, level);

    // Add birth-specific prompt
    let birth_prompt = match level {
        0 | 1 => WitnessPrompt::new(
            "These systems describe patterns present from your birth. What's it like to see yourself reflected this way?",
            InquiryType::Understanding,
        ),
        2 => WitnessPrompt::new(
            "These patterns existed before you knew about them. What is aware of them now?",
            InquiryType::PerspectiveShift,
        ),
        _ => WitnessPrompt::new(
            "You are not your patterns, yet they move through you. What wants to be expressed?",
            InquiryType::Integration,
        ),
    };

    prompts.push(birth_prompt);
    prompts
}

/// Generate prompts specifically for Daily Practice workflow
pub fn generate_daily_practice_prompts(
    synthesis: &SynthesisResult,
    level: u8,
) -> Vec<WitnessPrompt> {
    let mut prompts = generate_workflow_witness_prompts(synthesis, level);

    // Add time-specific prompt
    let daily_prompt = match level {
        0 | 1 => WitnessPrompt::new(
            "These rhythms are happening now. What activity feels most aligned with your current state?",
            InquiryType::Integration,
        ),
        2 => WitnessPrompt::new(
            "Time unfolds through you. Who is experiencing this moment?",
            InquiryType::PerspectiveShift,
        ),
        _ => WitnessPrompt::new(
            "How do you want to meet this moment, knowing what you now know?",
            InquiryType::Integration,
        ),
    };

    prompts.push(daily_prompt);
    prompts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::models::{Alignment, Tension, Theme};

    fn sample_synthesis() -> SynthesisResult {
        SynthesisResult {
            themes: vec![
                Theme::new("Leadership", "Natural leadership abilities")
                    .with_sources(vec!["numerology".into(), "human-design".into()]),
            ],
            alignments: vec![
                Alignment::new("Leadership alignment", "Both systems emphasize initiating")
                    .with_engines(vec!["numerology".into(), "human-design".into()]),
            ],
            tensions: vec![
                Tension::new("Visibility vs Introspection", "Inner need meets outer design")
                    .with_perspectives(
                        "numerology",
                        "Soul Urge 7 seeks depth",
                        "human-design",
                        "Manifestor designed for impact",
                    ),
            ],
            summary: "Test summary".to_string(),
        }
    }

    #[test]
    fn generate_prompts_at_level_0() {
        let synthesis = sample_synthesis();
        let prompts = generate_workflow_witness_prompts(&synthesis, 0);

        assert!(!prompts.is_empty());
        // Level 0 prompts should be observational
        assert!(prompts[0].text.contains("Notice") || prompts[0].text.contains("notice"));
    }

    #[test]
    fn generate_prompts_at_level_3() {
        let synthesis = sample_synthesis();
        let prompts = generate_workflow_witness_prompts(&synthesis, 3);

        assert!(!prompts.is_empty());
        // Level 3 prompts should be about authorship/choice
        let has_action_language = prompts.iter().any(|p| {
            p.text.contains("embody")
                || p.text.contains("consciously")
                || p.text.contains("response")
                || p.text.contains("choice")
        });
        assert!(has_action_language);
    }

    #[test]
    fn theme_prompt_includes_context() {
        let theme = Theme::new("Creativity", "Creative expression")
            .with_sources(vec!["numerology".into(), "gene-keys".into()]);
        
        let prompt = generate_theme_prompt(&theme, 1);
        
        assert!(prompt.context.is_some());
        assert_eq!(prompt.context.unwrap(), "Creativity");
    }

    #[test]
    fn tension_prompt_references_both_systems() {
        let tension = Tension::new("Test Tension", "Description")
            .with_perspectives("system-a", "view a", "system-b", "view b");
        
        let prompt = generate_tension_prompt(&tension, 1);
        
        assert!(prompt.text.contains("system-a"));
        assert!(prompt.text.contains("system-b"));
    }

    #[test]
    fn prompts_limited_to_four() {
        let mut synthesis = sample_synthesis();
        // Add more themes and tensions
        for i in 0..10 {
            synthesis.themes.push(
                Theme::new(format!("Theme{}", i), "Description")
                    .with_sources(vec!["a".into(), "b".into()])
            );
        }

        let prompts = generate_workflow_witness_prompts(&synthesis, 2);
        assert!(prompts.len() <= 4);
    }
}
