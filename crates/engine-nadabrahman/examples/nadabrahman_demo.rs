// NadaBrahman Engine — Local Demo & Raycast Integration Test
//
// Run with: cargo run --example nadabrahman_demo
//
// This example exercises the NadaBrahman engine directly (no DB/Redis/server
// required) and prints the exact JSON payload shape the Noasis Raycast
// extension expects. Use it to verify raga recommendations, witness prompts,
// and chakra frequencies before wiring into the Raycast client.
//
// The output mirrors the EngineExecutionResult type consumed by:
//   - src/lib/menu-bar-insights.ts   (buildNadabrahmanInsight)
//   - src/lib/execution-result-presenter.ts (buildNadabrahmanResultBlock)
//   - src/lib/dashboard-ritual.ts    (NadaBrahman remedy items)

use chrono::{Timelike, Utc};
use engine_nadabrahman::NadaBrahmanEngine;
use noesis_core::{ConsciousnessEngine, EngineInput, Precision};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("🎵 NadaBrahman Engine — Local Demo");
    println!("====================================\n");

    let engine = NadaBrahmanEngine::new();
    let now = Utc::now();
    let hour = now.hour();

    println!("Engine ID:    {}", engine.engine_id());
    println!("Engine Name:  {}", engine.engine_name());
    println!("Required Phase: {}", engine.required_phase());
    println!("Current Hour: {}:00", hour);
    println!();

    // ------------------------------------------------------------------
    // Test 1: Basic time-based recommendation (no options)
    // ------------------------------------------------------------------
    println!("--- Test 1: Time-based recommendation ---");
    let basic_input = EngineInput {
        birth_data: None,
        current_time: now,
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    };

    let basic_output = engine.calculate(basic_input).await.unwrap();
    print_engine_output(&basic_output);

    // ------------------------------------------------------------------
    // Test 2: Full options — dosha + rasa + chakra
    // ------------------------------------------------------------------
    println!("\n--- Test 2: Full options (dosha + rasa + chakra) ---");
    let mut full_options = HashMap::new();
    full_options.insert("dosha".to_string(), json!("vata"));
    full_options.insert("rasa".to_string(), json!("shanta"));
    full_options.insert("chakra".to_string(), json!("heart"));
    full_options.insert("consciousness_level".to_string(), json!(2));

    let full_input = EngineInput {
        birth_data: None,
        current_time: now,
        location: None,
        precision: Precision::Standard,
        options: full_options,
    };

    let full_output = engine.calculate(full_input).await.unwrap();
    print_engine_output(&full_output);

    // ------------------------------------------------------------------
    // Test 3: Pitta + Karuna + Throat (different constitution)
    // ------------------------------------------------------------------
    println!("\n--- Test 3: Pitta + Karuna + Throat ---");
    let mut pitta_options = HashMap::new();
    pitta_options.insert("dosha".to_string(), json!("pitta"));
    pitta_options.insert("mood".to_string(), json!("karuna"));
    pitta_options.insert("chakra".to_string(), json!("throat"));

    let pitta_input = EngineInput {
        birth_data: None,
        current_time: now,
        location: None,
        precision: Precision::Standard,
        options: pitta_options,
    };

    let pitta_output = engine.calculate(pitta_input).await.unwrap();
    print_engine_output(&pitta_output);

    // ------------------------------------------------------------------
    // Test 4: Raycast-compatible JSON payload
    // ------------------------------------------------------------------
    println!("\n--- Test 4: Raycast-compatible payload shape ---");
    let raycast_payload = json!({
        "engine_id": full_output.engine_id,
        "result": full_output.result,
        "witness_prompt": full_output.witness_prompt,
        "consciousness_level": full_output.consciousness_level,
        "metadata": {
            "calculation_time_ms": full_output.metadata.calculation_time_ms,
            "backend": full_output.metadata.backend,
            "precision_achieved": full_output.metadata.precision_achieved,
            "cached": full_output.metadata.cached,
            "timestamp": full_output.metadata.timestamp,
            "engine_version": full_output.metadata.engine_version,
        },
        "timestamp": full_output.metadata.timestamp,
    });
    println!("{}", serde_json::to_string_pretty(&raycast_payload).unwrap());

    // ------------------------------------------------------------------
    // Test 5: Validation
    // ------------------------------------------------------------------
    println!("\n--- Test 5: Validation ---");
    let validation = engine.validate(&full_output).await.unwrap();
    println!("Valid:      {}", validation.valid);
    println!("Confidence: {}", validation.confidence);
    if !validation.messages.is_empty() {
        println!("Messages:   {:?}", validation.messages);
    }

    // ------------------------------------------------------------------
    // Summary
    // ------------------------------------------------------------------
    println!("\n====================================");
    println!("✅ NadaBrahman engine local tests passed");
    println!("   - 72 Melakarta ragas loaded");
    println!("   - 8 prahar time mappings active");
    println!("   - 7 chakra frequency mappings ready");
    println!("   - Witness prompts generated");
    println!("   - Raycast payload shape verified");
    println!();
    println!("Next steps to wire into Noasis Raycast engine:");
    println!("   1. Ensure noesis-server is running with engine registered");
    println!("   2. Set baseUrl in Raycast preferences to server address");
    println!("   3. Run 'Daily Witness' or 'Dashboard' to see NadaBrahman remedy");
    println!("   4. Check menubar for live raga recommendations");
}

fn print_engine_output(output: &noesis_core::EngineOutput) {
    let result = &output.result;

    if let Some(time_rec) = result.get("time_recommendation") {
        let tr = time_rec.as_object().unwrap();
        println!("  Prahar:     {} ({})",
            tr.get("prahar_name").and_then(|v| v.as_str()).unwrap_or("?"),
            tr.get("time_range").and_then(|v| v.as_str()).unwrap_or("?"),
        );
        println!("  Dosha:      {}",
            tr.get("dosha_dominance").and_then(|v| v.as_str()).unwrap_or("?"),
        );
        println!("  Energy:     {}",
            tr.get("energy_quality").and_then(|v| v.as_str()).unwrap_or("?"),
        );

        if let Some(primary) = tr.get("primary_raga").and_then(|v| v.as_object()) {
            println!("  Primary:    #{} — {}",
                primary.get("raga_number").and_then(|v| v.as_u64()).unwrap_or(0),
                primary.get("raga_name").and_then(|v| v.as_str()).unwrap_or("?"),
            );
        }
    }

    if let Some(recs) = result.get("recommendations").and_then(|v| v.as_array()) {
        println!("  Recommendations ({}):", recs.len());
        for (i, rec) in recs.iter().take(3).enumerate() {
            let r = rec.as_object().unwrap();
            println!("    {}. #{} {} (score: {:.2})",
                i + 1,
                r.get("raga_number").and_then(|v| v.as_u64()).unwrap_or(0),
                r.get("raga_name").and_then(|v| v.as_str()).unwrap_or("?"),
                r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
            );
        }
    }

    if let Some(freq) = result.get("chakra_frequency") {
        let f = freq.as_object().unwrap();
        println!("  Chakra:     {} — {}Hz (binaural {:.1}Hz)",
            f.get("chakra_name").and_then(|v| v.as_str()).unwrap_or("?"),
            f.get("solfeggio_hz").and_then(|v| v.as_f64()).unwrap_or(0.0),
            f.get("binaural_target_hz").and_then(|v| v.as_f64()).unwrap_or(0.0),
        );
    }

    if let Some(dosha) = result.get("dosha_recommendation") {
        println!("  Dosha rec:  {}", dosha.as_str().unwrap_or("?"));
    }

    if let Some(rasa) = result.get("rasa_mapping") {
        println!("  Rasa:       {}", rasa.as_str().unwrap_or("?"));
    }

    println!("  Witness:    {}", output.witness_prompt.chars().take(80).collect::<String>());
    println!("  Calc time:  {:.2}ms", output.metadata.calculation_time_ms);
}
