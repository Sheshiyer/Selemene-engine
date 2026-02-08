//! Health check handlers for Kubernetes liveness and readiness probes
//!
//! - `/health/live` - Liveness probe: Is the application running?
//! - `/health/ready` - Readiness probe: Can the application serve traffic?

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::SelemeneEngine;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HealthChecks>,
}

/// Overall health status
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual health checks for readiness
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub cache: CheckResult,
    pub ephemeris: CheckResult,
    pub orchestrator: CheckResult,
}

/// Result of an individual health check
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: HealthStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl CheckResult {
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            latency_ms: None,
        }
    }

    pub fn healthy_with_latency(message: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            latency_ms: Some(latency_ms),
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: message.into(),
            latency_ms: None,
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: message.into(),
            latency_ms: None,
        }
    }
}

/// Liveness probe handler
/// 
/// Returns 200 OK if the application is running.
/// This is a simple check - if we can respond, we're alive.
/// 
/// Kubernetes will restart the pod if this fails.
pub async fn liveness_probe() -> impl IntoResponse {
    let response = HealthResponse {
        status: HealthStatus::Healthy,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        checks: None,
    };

    (StatusCode::OK, Json(response))
}

/// Readiness probe handler
/// 
/// Returns 200 OK if the application is ready to serve traffic.
/// Checks:
/// - Cache system is accessible
/// - Swiss Ephemeris data is available
/// - Orchestrator is initialized
/// 
/// Kubernetes will stop sending traffic if this fails.
pub async fn readiness_probe(
    State(engine): State<Arc<SelemeneEngine>>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    
    // Check 1: Cache system
    let cache_check = check_cache(&engine).await;
    
    // Check 2: Swiss Ephemeris data
    let ephemeris_check = check_ephemeris().await;
    
    // Check 3: Orchestrator
    let orchestrator_check = check_orchestrator(&engine).await;
    
    // Determine overall status
    let overall_status = determine_overall_status(&[
        &cache_check,
        &ephemeris_check,
        &orchestrator_check,
    ]);

    let checks = HealthChecks {
        cache: cache_check,
        ephemeris: ephemeris_check,
        orchestrator: orchestrator_check,
    };

    let response = HealthResponse {
        status: overall_status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        checks: Some(checks),
    };

    let status_code = match overall_status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still serve traffic when degraded
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    tracing::debug!(
        status = ?overall_status,
        latency_ms = start.elapsed().as_millis(),
        "Readiness check completed"
    );

    (status_code, Json(response))
}

/// Check cache system health
async fn check_cache(engine: &SelemeneEngine) -> CheckResult {
    let start = std::time::Instant::now();
    
    // Try to access the cache manager
    // For now, just verify the cache manager exists
    let _cache = &engine.cache_manager;
    
    // TODO: When Redis is configured, add actual connectivity check:
    // match engine.cache_manager.ping().await {
    //     Ok(_) => CheckResult::healthy_with_latency("L1 cache operational", start.elapsed().as_millis() as u64),
    //     Err(e) => CheckResult::degraded(format!("L2 Redis unavailable: {}", e)),
    // }
    
    CheckResult::healthy_with_latency(
        "Cache system operational (L1 in-memory)",
        start.elapsed().as_millis() as u64,
    )
}

/// Check Swiss Ephemeris data availability
async fn check_ephemeris() -> CheckResult {
    let start = std::time::Instant::now();
    
    // Check if ephemeris data directory exists and has files
    let ephemeris_path = std::env::var("SWISS_EPHEMERIS_PATH")
        .unwrap_or_else(|_| "/app/data/ephemeris".to_string());
    
    let path = std::path::Path::new(&ephemeris_path);
    
    if !path.exists() {
        return CheckResult::degraded(format!(
            "Ephemeris directory not found: {}",
            ephemeris_path
        ));
    }
    
    // Check if directory has any .se1 files (Swiss Ephemeris data files)
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let has_data_files = entries
                .filter_map(|e| e.ok())
                .any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "se1" || ext == "dat")
                        .unwrap_or(false)
                });
            
            if has_data_files {
                CheckResult::healthy_with_latency(
                    "Swiss Ephemeris data available",
                    start.elapsed().as_millis() as u64,
                )
            } else {
                // Not having ephemeris files is degraded, not unhealthy
                // The native engines can still work
                CheckResult::degraded("No Swiss Ephemeris data files found (native engines only)")
            }
        }
        Err(e) => CheckResult::unhealthy(format!("Cannot read ephemeris directory: {}", e)),
    }
}

/// Check orchestrator health
async fn check_orchestrator(engine: &SelemeneEngine) -> CheckResult {
    let start = std::time::Instant::now();
    
    // Verify orchestrator is initialized by checking if we can access config
    match engine.config.try_read() {
        Ok(_config) => CheckResult::healthy_with_latency(
            "Calculation orchestrator ready",
            start.elapsed().as_millis() as u64,
        ),
        Err(_) => CheckResult::degraded("Orchestrator config temporarily locked"),
    }
}

/// Determine overall health status from individual checks
fn determine_overall_status(checks: &[&CheckResult]) -> HealthStatus {
    let has_unhealthy = checks.iter().any(|c| c.status == HealthStatus::Unhealthy);
    let has_degraded = checks.iter().any(|c| c.status == HealthStatus::Degraded);
    
    if has_unhealthy {
        HealthStatus::Unhealthy
    } else if has_degraded {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_healthy() {
        let result = CheckResult::healthy("test");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.message, "test");
    }

    #[test]
    fn test_determine_overall_status() {
        let healthy = CheckResult::healthy("ok");
        let degraded = CheckResult::degraded("warning");
        let unhealthy = CheckResult::unhealthy("error");

        // All healthy
        assert_eq!(
            determine_overall_status(&[&healthy, &healthy]),
            HealthStatus::Healthy
        );

        // One degraded
        assert_eq!(
            determine_overall_status(&[&healthy, &degraded]),
            HealthStatus::Degraded
        );

        // One unhealthy (takes precedence)
        assert_eq!(
            determine_overall_status(&[&healthy, &degraded, &unhealthy]),
            HealthStatus::Unhealthy
        );
    }
}
