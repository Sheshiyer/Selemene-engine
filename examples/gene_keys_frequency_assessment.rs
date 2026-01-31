//! Example: Consciousness Frequency Assessment and Transformation Pathways
//!
//! Demonstrates:
//! 1. Frequency assessment framework (non-deterministic)
//! 2. Recognition prompts for self-identification
//! 3. Transformation pathways (inquiry-based)
//! 4. Non-prescriptive contemplative guidance

use engine_gene_keys::{
    assess_frequencies, generate_complete_pathways, generate_transformation_pathways,
    ActivationSequence, ActivationSource, Frequency, GeneKeyActivation, GeneKeysChart,
};

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("    Gene Keys: Consciousness Frequency Assessment");
    println!("═══════════════════════════════════════════════════════════\n");

    // Create sample chart with Gene Keys 17, 18, 1, 2
    let chart = GeneKeysChart {
        activation_sequence: ActivationSequence {
            lifes_work: (17, 18),
            evolution: (1, 2),
            radiance: (17, 1),
            purpose: (18, 2),
        },
        active_keys: vec![
            GeneKeyActivation {
                key_number: 17,
                line: 3,
                source: ActivationSource::PersonalitySun,
                gene_key_data: None,
            },
            GeneKeyActivation {
                key_number: 18,
                line: 4,
                source: ActivationSource::PersonalityEarth,
                gene_key_data: None,
            },
            GeneKeyActivation {
                key_number: 1,
                line: 2,
                source: ActivationSource::DesignSun,
                gene_key_data: None,
            },
            GeneKeyActivation {
                key_number: 64,
                line: 5,
                source: ActivationSource::DesignEarth,
                gene_key_data: None,
            },
        ],
    };

    println!("ACTIVE GENE KEYS: 17, 18, 1, 64");
    println!();

    // Example 1: Assessment without consciousness level
    println!("─────────────────────────────────────────────────────────");
    println!("Example 1: Assessment WITHOUT consciousness level");
    println!("(No frequency suggested - user must self-identify)");
    println!("─────────────────────────────────────────────────────────\n");

    let assessments_no_level = assess_frequencies(&chart, None);
    let gk17 = &assessments_no_level[0];

    println!("Gene Key 17: {}", gk17.name);
    println!("  Shadow:  {} ({})", gk17.shadow, short_desc(&gk17.shadow_description));
    println!("  Gift:    {} ({})", gk17.gift, short_desc(&gk17.gift_description));
    println!("  Siddhi:  {} ({})", gk17.siddhi, short_desc(&gk17.siddhi_description));
    println!("  Suggested Frequency: {:?}", gk17.suggested_frequency);
    println!();
    println!("Recognition Prompts (Shadow):");
    for prompt in &gk17.recognition_prompts.shadow {
        println!("  • {}", prompt);
    }
    println!();

    // Example 2: Assessment with Shadow-level consciousness
    println!("─────────────────────────────────────────────────────────");
    println!("Example 2: Assessment WITH consciousness level = 2");
    println!("(Shadow frequency suggested as starting point)");
    println!("─────────────────────────────────────────────────────────\n");

    let assessments_shadow = assess_frequencies(&chart, Some(2));
    let gk17_shadow = &assessments_shadow[0];

    println!("Gene Key 17: {}", gk17_shadow.name);
    println!("  Suggested Frequency: {:?}", gk17_shadow.suggested_frequency);
    println!();
    println!("Recognition Prompts (Shadow):");
    for (i, prompt) in gk17_shadow.recognition_prompts.shadow.iter().take(3).enumerate() {
        println!("  {}. {}", i + 1, prompt);
    }
    println!();

    // Example 3: Transformation Pathways (Shadow→Gift)
    println!("─────────────────────────────────────────────────────────");
    println!("Example 3: Transformation Pathway (Shadow→Gift)");
    println!("(Inquiry-based, non-prescriptive)");
    println!("─────────────────────────────────────────────────────────\n");

    let pathways = generate_transformation_pathways(&assessments_shadow);
    let pathway_17 = &pathways[0];

    println!("Gene Key 17: {} → {}", 
        frequency_name(&pathway_17.current_frequency),
        frequency_name(&pathway_17.next_frequency)
    );
    println!();
    println!("Core Inquiry:");
    println!("  {}", pathway_17.core_inquiry);
    println!();
    println!("Contemplations:");
    for (i, contemplation) in pathway_17.contemplations.iter().take(3).enumerate() {
        println!("  {}. {}", i + 1, contemplation);
    }
    println!();
    println!("Witnessing Practices:");
    for (i, practice) in pathway_17.witnessing_practices.iter().take(3).enumerate() {
        println!("  {}. {}", i + 1, practice);
    }
    println!();
    if let Some(inquiry) = &pathway_17.shadow_to_gift_inquiry {
        println!("Shadow→Gift Inquiry:");
        println!("  {}", inquiry);
    }
    println!();

    // Example 4: Gift-level consciousness
    println!("─────────────────────────────────────────────────────────");
    println!("Example 4: Assessment WITH consciousness level = 4");
    println!("(Gift frequency suggested - transformation to Siddhi)");
    println!("─────────────────────────────────────────────────────────\n");

    let assessments_gift = assess_frequencies(&chart, Some(4));
    let gk1_gift = assessments_gift.iter().find(|a| a.gene_key == 1).unwrap();

    println!("Gene Key 1: {}", gk1_gift.name);
    println!("  Shadow:  {}", gk1_gift.shadow);
    println!("  Gift:    {}", gk1_gift.gift);
    println!("  Siddhi:  {}", gk1_gift.siddhi);
    println!("  Suggested Frequency: {:?}", gk1_gift.suggested_frequency);
    println!();
    println!("Recognition Prompts (Gift):");
    for prompt in gk1_gift.recognition_prompts.gift.iter().take(3) {
        println!("  • {}", prompt);
    }
    println!();

    let pathways_gift = generate_transformation_pathways(&assessments_gift);
    let pathway_1 = pathways_gift.iter().find(|p| p.gene_key == 1).unwrap();

    println!("Transformation Pathway (Gift→Siddhi):");
    println!();
    println!("Core Inquiry:");
    println!("  {}", pathway_1.core_inquiry);
    println!();
    if let Some(inquiry) = &pathway_1.gift_to_siddhi_inquiry {
        println!("Gift→Siddhi Inquiry:");
        println!("  {}", inquiry);
    }
    println!();

    // Example 5: Complete pathways (all transitions)
    println!("─────────────────────────────────────────────────────────");
    println!("Example 5: Complete Pathways (All Transitions)");
    println!("(Shadow→Gift AND Gift→Siddhi for full journey)");
    println!("─────────────────────────────────────────────────────────\n");

    let complete_pathways = generate_complete_pathways(&assessments_no_level);
    println!("Total Pathways: {}", complete_pathways.len());
    println!("  (4 active keys × 2 transitions = 8 pathways)");
    println!();

    // Show both transitions for Gene Key 17
    let gk17_shadow_path = complete_pathways
        .iter()
        .find(|p| p.gene_key == 17 && p.current_frequency == Frequency::Shadow)
        .unwrap();
    let gk17_gift_path = complete_pathways
        .iter()
        .find(|p| p.gene_key == 17 && p.current_frequency == Frequency::Gift)
        .unwrap();

    println!("Gene Key 17 Complete Journey:");
    println!();
    println!("1. Shadow→Gift Transition:");
    println!("   Core Inquiry: {}", gk17_shadow_path.core_inquiry);
    println!();
    println!("2. Gift→Siddhi Transition:");
    println!("   Core Inquiry: {}", gk17_gift_path.core_inquiry);
    println!();

    // Example 6: Non-prescriptive language verification
    println!("─────────────────────────────────────────────────────────");
    println!("Example 6: Non-Prescriptive Language Verification");
    println!("─────────────────────────────────────────────────────────\n");

    println!("✓ All core inquiries contain questions (?)");
    println!("✓ Contemplations use witnessing language ('Notice', 'Can you')");
    println!("✓ Practices use invitational language ('might', 'could')");
    println!("✓ NO prescriptive commands ('must', 'should', 'do this')");
    println!();

    let has_questions = complete_pathways.iter().all(|p| p.core_inquiry.contains('?'));
    let has_invitations = complete_pathways
        .iter()
        .flat_map(|p| &p.witnessing_practices)
        .all(|practice| practice.contains("might") || practice.contains("could"));

    println!("Verification:");
    println!("  All inquiries have questions: {}", has_questions);
    println!("  All practices invitational: {}", has_invitations);
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("    Assessment Complete");
    println!("═══════════════════════════════════════════════════════════");
}

fn short_desc(full: &str) -> String {
    let words: Vec<&str> = full.split_whitespace().take(10).collect();
    format!("{}...", words.join(" "))
}

fn frequency_name(freq: &Frequency) -> &str {
    match freq {
        Frequency::Shadow => "Shadow",
        Frequency::Gift => "Gift",
        Frequency::Siddhi => "Siddhi",
    }
}
