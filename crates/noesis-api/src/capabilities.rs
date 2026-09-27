//! Shared contract-v1 capability collection for the public and admin routes.
//! Uses only the orchestrator registry and the bridge readiness self-check.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use noesis_core::contract::{
    CapabilityAvailability, ContractVersion, EngineCapability, RuntimeKind,
};

use crate::AppState;

/// How long a bridge readiness self-check result is reused by the
/// capabilities routes. The public route is reachable with any API key and is
/// polled by several consumers; each uncached readiness call fans out to a
/// sidecar self-check across every TypeScript engine.
pub const CAPABILITY_READINESS_TTL: Duration = Duration::from_secs(10);

type HealthyById = HashMap<String, bool>;

/// Single-entry TTL cache for the readiness-derived `engine_id -> healthy`
/// map. The entry is keyed by an owner id (the bridge manager's address) so
/// distinct bridges in one process never share results. The async mutex is
/// held across the load, so concurrent requests coalesce onto one self-check.
#[derive(Default)]
pub(crate) struct ReadinessCache {
    entry: Mutex<Option<(usize, Instant, HealthyById)>>,
}

impl ReadinessCache {
    pub(crate) async fn get_or_load<F, Fut>(
        &self,
        owner: usize,
        ttl: Duration,
        loader: F,
    ) -> HealthyById
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = HealthyById>,
    {
        let mut entry = self.entry.lock().await;
        if let Some((cached_owner, loaded_at, map)) = entry.as_ref() {
            if *cached_owner == owner && loaded_at.elapsed() < ttl {
                return map.clone();
            }
        }
        let map = loader().await;
        *entry = Some((owner, Instant::now(), map.clone()));
        map
    }
}

fn readiness_cache() -> &'static ReadinessCache {
    static CACHE: OnceLock<ReadinessCache> = OnceLock::new();
    CACHE.get_or_init(ReadinessCache::default)
}

async fn healthy_by_id(state: &AppState) -> HealthyById {
    let bridge = state.bridge();
    let owner = Arc::as_ptr(bridge) as usize;
    readiness_cache()
        .get_or_load(owner, CAPABILITY_READINESS_TTL, || async {
            // Both outcomes are cached: a down sidecar yields an empty map
            // (every bridge row unavailable) rather than a retry per request.
            match bridge.readiness_status().await {
                Ok(status) => status
                    .engines
                    .iter()
                    .map(|e| (e.engine_id.clone(), e.healthy))
                    .collect(),
                Err(_) => HashMap::new(),
            }
        })
        .await
}

pub async fn collect_capabilities(state: &AppState) -> Vec<EngineCapability> {
    let healthy_by_id = healthy_by_id(state).await;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn counting_loader(
        calls: &AtomicUsize,
    ) -> impl FnOnce() -> std::future::Ready<HealthyById> + '_ {
        move || {
            calls.fetch_add(1, Ordering::SeqCst);
            std::future::ready(HashMap::from([("tarot".to_string(), true)]))
        }
    }

    #[tokio::test]
    async fn second_call_within_ttl_reuses_cached_map() {
        let cache = ReadinessCache::default();
        let calls = AtomicUsize::new(0);
        let ttl = Duration::from_secs(60);
        let first = cache.get_or_load(1, ttl, counting_loader(&calls)).await;
        let second = cache.get_or_load(1, ttl, counting_loader(&calls)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(first, second);
        assert_eq!(second.get("tarot"), Some(&true));
    }

    #[tokio::test]
    async fn call_after_expiry_reloads() {
        let cache = ReadinessCache::default();
        let calls = AtomicUsize::new(0);
        let ttl = Duration::from_millis(20);
        cache.get_or_load(1, ttl, counting_loader(&calls)).await;
        tokio::time::sleep(Duration::from_millis(40)).await;
        cache.get_or_load(1, ttl, counting_loader(&calls)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn empty_error_map_is_cached_and_owner_change_reloads() {
        let cache = ReadinessCache::default();
        let calls = AtomicUsize::new(0);
        let ttl = Duration::from_secs(60);
        let empty = || {
            calls.fetch_add(1, Ordering::SeqCst);
            std::future::ready(HashMap::new())
        };
        assert!(cache.get_or_load(1, ttl, empty).await.is_empty());
        assert!(cache
            .get_or_load(1, ttl, counting_loader(&calls))
            .await
            .is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        cache.get_or_load(2, ttl, counting_loader(&calls)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn ttl_is_ten_seconds() {
        assert_eq!(CAPABILITY_READINESS_TTL, Duration::from_secs(10));
    }
}
