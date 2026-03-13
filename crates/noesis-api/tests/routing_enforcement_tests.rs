mod common;

use common::route_inventory::workspace_root;
use common::test_harness::RoutingHarness;
use noesis_orchestrator::WorkflowOrchestrator;
use std::collections::BTreeSet;
use std::fs;

const RUST_ENGINE_IDS: [&str; 11] = [
    "panchanga",
    "numerology",
    "biorhythm",
    "human-design",
    "gene-keys",
    "vimshottari",
    "biofield",
    "vedic-clock",
    "face-reading",
    "nadabrahman",
    "transits",
];

const BRIDGE_ENGINE_IDS: [&str; 5] = [
    "tarot",
    "i-ching",
    "enneagram",
    "sacred-geometry",
    "sigil-forge",
];

macro_rules! routing_enforcement_engine_test {
    ($name:ident, $engine_id:literal) => {
        #[tokio::test]
        async fn $name() {
            let harness = RoutingHarness::with_probe_engines(&RUST_ENGINE_IDS).await;
            harness.assert_engine_calculate_routed($engine_id).await;
        }
    };
}

macro_rules! workflow_routing_test {
    ($name:ident, $workflow_id:literal) => {
        #[tokio::test]
        async fn $name() {
            let all_workflow_engines = canonical_workflow_engine_ids();
            let expected = workflow_engine_ids($workflow_id);
            let all_engine_refs = all_workflow_engines
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            let expected_refs = expected.iter().map(String::as_str).collect::<Vec<_>>();
            let harness = RoutingHarness::with_probe_engines(&all_engine_refs).await;
            harness
                .assert_workflow_execute_routed($workflow_id, &expected_refs)
                .await;
        }
    };
}

routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_panchanga,
    "panchanga"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_numerology,
    "numerology"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_biorhythm,
    "biorhythm"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_human_design,
    "human-design"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_gene_keys,
    "gene-keys"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_vimshottari,
    "vimshottari"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_biofield,
    "biofield"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_vedic_clock,
    "vedic-clock"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_face_reading,
    "face-reading"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_nadabrahman,
    "nadabrahman"
);
routing_enforcement_engine_test!(
    routing_enforcement_calculate_routes_through_orchestrator_for_transits,
    "transits"
);

workflow_routing_test!(
    workflow_routing_execute_routes_through_orchestrator_for_birth_blueprint,
    "birth-blueprint"
);
workflow_routing_test!(
    workflow_routing_execute_routes_through_orchestrator_for_daily_practice,
    "daily-practice"
);
workflow_routing_test!(
    workflow_routing_execute_routes_through_orchestrator_for_decision_support,
    "decision-support"
);
workflow_routing_test!(
    workflow_routing_execute_routes_through_orchestrator_for_creative_expression,
    "creative-expression"
);
workflow_routing_test!(
    workflow_routing_execute_routes_through_orchestrator_for_self_inquiry,
    "self-inquiry"
);
workflow_routing_test!(
    workflow_routing_execute_routes_through_orchestrator_for_full_spectrum,
    "full-spectrum"
);

#[tokio::test]
async fn routing_enforcement_bridge_engine_routes_stay_on_orchestrator_path() {
    let harness = RoutingHarness::with_probe_engines(&BRIDGE_ENGINE_IDS).await;

    for engine_id in BRIDGE_ENGINE_IDS {
        harness.assert_engine_calculate_routed(engine_id).await;
    }
}

#[test]
fn routing_enforcement_handlers_do_not_call_bridge_manager_directly() {
    let handlers_dir = workspace_root().join("crates/noesis-api/src/handlers");
    for file_name in ["auth.rs", "users.rs", "admin.rs"] {
        let source =
            fs::read_to_string(handlers_dir.join(file_name)).expect("handler source should read");
        assert!(
            !source.contains("BridgeManager"),
            "{} should not import BridgeManager directly",
            file_name
        );
        assert!(
            !source.contains("bridge_manager."),
            "{} should not call bridge_manager directly",
            file_name
        );
    }

    let lib_source = fs::read_to_string(workspace_root().join("crates/noesis-api/src/lib.rs"))
        .expect("api lib source should read");
    for handler in [
        "calculate_handler",
        "validate_handler",
        "engine_info_handler",
        "workflow_execute_handler",
        "workflow_info_handler",
        "list_engines_handler",
        "list_workflows_handler",
    ] {
        let body = handler_body(&lib_source, handler)
            .unwrap_or_else(|| panic!("handler body should exist for {}", handler));
        assert!(
            !body.contains("bridge_manager"),
            "{} should route through the orchestrator rather than BridgeManager",
            handler
        );
    }
}

fn canonical_workflow_engine_ids() -> Vec<String> {
    let orchestrator = WorkflowOrchestrator::new();
    let mut ids = BTreeSet::new();
    for workflow in orchestrator.list_workflows() {
        for engine_id in &workflow.engine_ids {
            ids.insert(engine_id.clone());
        }
    }
    ids.into_iter().collect()
}

fn workflow_engine_ids(workflow_id: &str) -> Vec<String> {
    WorkflowOrchestrator::new()
        .get_workflow(workflow_id)
        .unwrap_or_else(|| panic!("missing workflow {}", workflow_id))
        .engine_ids
        .clone()
}

fn handler_body<'a>(source: &'a str, handler_name: &str) -> Option<&'a str> {
    let signature = format!("fn {}(", handler_name);
    let start = source.find(&signature)?;
    let tail = &source[start..];
    let open_idx = start + tail.find('{')?;
    let close_idx = find_matching_brace(source, open_idx);
    Some(&source[open_idx + 1..close_idx])
}

fn find_matching_brace(source: &str, open_idx: usize) -> usize {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut idx = open_idx;
    let mut in_string = false;
    let mut escaped = false;

    while idx < bytes.len() {
        let ch = bytes[idx] as char;

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            idx += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            idx += 1;
            continue;
        }

        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return idx;
            }
        }

        idx += 1;
    }

    panic!("unmatched brace while reading handler body");
}
