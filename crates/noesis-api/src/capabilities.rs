//! Shared contract-v1 capability collection for the public and admin routes.
//! Uses only the orchestrator registry and the bridge readiness self-check.

use std::collections::{HashMap, HashSet};

use noesis_core::contract::{
    CapabilityAvailability, ContractVersion, EngineCapability, RuntimeKind,
};

use crate::AppState;

pub async fn collect_capabilities(state: &AppState) -> Vec<EngineCapability> {
    let readiness = state.bridge().readiness_status().await;
    let healthy_by_id: HashMap<String, bool> = match &readiness {
        Ok(status) => status
            .engines
            .iter()
            .map(|e| (e.engine_id.clone(), e.healthy))
            .collect(),
        Err(_) => HashMap::new(),
    };

    let mut rows: Vec<EngineCapability> = state
        .bridge()
        .engines()
        .iter()
        .map(|engine| {
            let engine_id = engine.engine_id().to_string();
            let availability = match healthy_by_id.get(&engine_id) {
                Some(true) => CapabilityAvailability::Available,
                _ => CapabilityAvailability::Unavailable,
            };
            EngineCapability {
                contract_version: ContractVersion::V1,
                engine_id,
                display_name: engine.engine_name().to_string(),
                availability,
                runtime_kind: RuntimeKind::TypeScript,
                dependencies: Vec::new(),
                required_phase: Some(engine.required_phase()),
                implementation_version: None,
            }
        })
        .collect();

    let bridge_ids: HashSet<String> = rows.iter().map(|r| r.engine_id.clone()).collect();
    for engine_id in state.orchestrator.list_engines() {
        if bridge_ids.contains(&engine_id) {
            continue;
        }
        rows.push(EngineCapability {
            contract_version: ContractVersion::V1,
            engine_id: engine_id.clone(),
            display_name: engine_id,
            availability: CapabilityAvailability::Available,
            runtime_kind: RuntimeKind::Native,
            dependencies: Vec::new(),
            required_phase: None,
            implementation_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        });
    }
    rows.sort_by(|a, b| a.engine_id.cmp(&b.engine_id));
    rows
}
