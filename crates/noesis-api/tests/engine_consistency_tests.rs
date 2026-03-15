use chrono::Utc;
use noesis_core::{BirthData, ConsciousnessEngine, EngineInput, Precision};
use std::collections::HashMap;

#[tokio::test]
async fn test_panchanga_and_vimshottari_nakshatra_parity_for_canonical_input() {
    let input = EngineInput {
        birth_data: Some(BirthData {
            name: Some("Canonical".to_string()),
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9340,
            longitude: 77.6214,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    };

    let panchanga = engine_panchanga::PanchangaEngine::new()
        .calculate(input.clone())
        .await
        .expect("panchanga calculation should succeed");

    let vim = engine_vimshottari::VimshottariEngine::new()
        .calculate(input)
        .await
        .expect("vimshottari calculation should succeed");

    let panchanga_nak = panchanga.result["nakshatra_name"]
        .as_str()
        .expect("panchanga nakshatra_name should be present");
    let vim_nak = vim.result["birth_nakshatra"]["name"]
        .as_str()
        .expect("vimshottari birth_nakshatra.name should be present");

    assert_eq!(panchanga_nak, vim_nak);
    assert_eq!(panchanga_nak, "Uttara Phalguni");
}

#[tokio::test]
async fn test_human_design_and_gene_keys_canonical_alignment() {
    let input = EngineInput {
        birth_data: Some(BirthData {
            name: Some("Canonical".to_string()),
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9340,
            longitude: 77.6214,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    };

    let hd_engine = std::sync::Arc::new(engine_human_design::HumanDesignEngine::new());
    let gk_engine = engine_gene_keys::GeneKeysEngine::with_hd_engine(hd_engine.clone());

    let hd = hd_engine
        .calculate(input.clone())
        .await
        .expect("human-design calculation should succeed");
    let gk = gk_engine
        .calculate(input)
        .await
        .expect("gene-keys calculation should succeed");

    let ps = hd.result["personality_activations"]["sun"]["gate"]
        .as_u64()
        .expect("HD personality sun gate");
    let pe = hd.result["personality_activations"]["earth"]["gate"]
        .as_u64()
        .expect("HD personality earth gate");
    let ds = hd.result["design_activations"]["sun"]["gate"]
        .as_u64()
        .expect("HD design sun gate");
    let de = hd.result["design_activations"]["earth"]["gate"]
        .as_u64()
        .expect("HD design earth gate");

    assert_eq!(ps, 4);
    assert_eq!(pe, 49);
    assert_eq!(ds, 23);
    assert_eq!(de, 43);

    assert_eq!(
        gk.result["activation_sequence"]["lifes_work"][0]
            .as_u64()
            .unwrap(),
        ps
    );
    assert_eq!(
        gk.result["activation_sequence"]["lifes_work"][1]
            .as_u64()
            .unwrap(),
        pe
    );
    assert_eq!(
        gk.result["activation_sequence"]["evolution"][0]
            .as_u64()
            .unwrap(),
        ds
    );
    assert_eq!(
        gk.result["activation_sequence"]["evolution"][1]
            .as_u64()
            .unwrap(),
        de
    );
}
