//! Pure capability resolution for the canonical v1 engine authority.
//!
//! The resolver accepts only bounded registration/dependency observations. It
//! never reads environment variables, opens a database or performs network I/O.

use noesis_core::contract::{
    CapabilityAvailability, CapabilityOperations, CapabilityReasonCode, ContractVersion,
    DependencyKind, DependencyObservation, DependencyRequirement, EngineCapability, RuntimeKind,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyDeclaration {
    pub dependency_id: String,
    pub dependency_kind: DependencyKind,
    pub requirement: DependencyRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDeclaration {
    pub engine_id: String,
    pub display_name: String,
    pub runtime_kind: RuntimeKind,
    pub dependencies: Vec<DependencyDeclaration>,
    pub operations: CapabilityOperations,
    pub required_phase: Option<u8>,
    pub implementation_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationObservation {
    pub engine_id: String,
    pub registered: bool,
    pub reason_code: CapabilityReasonCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCapability {
    pub capability: EngineCapability,
    pub registered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityResolutionError(pub String);

impl std::fmt::Display for CapabilityResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "capability resolution failed: {}", self.0)
    }
}

impl std::error::Error for CapabilityResolutionError {}

/// Parse the checked-in registry into resolver declarations without consulting
/// process configuration or any external service.
pub fn canonical_declarations() -> Result<Vec<CapabilityDeclaration>, CapabilityResolutionError> {
    let authority: serde_json::Value = serde_json::from_str(include_str!(
        "../../../contracts/v1/registries/engines.json"
    ))
    .map_err(|error| CapabilityResolutionError(format!("invalid registry JSON: {error}")))?;
    let rows = authority["engines"]
        .as_array()
        .ok_or_else(|| CapabilityResolutionError("registry engines must be an array".into()))?;
    rows.iter()
        .map(|row| {
            let text = |key: &str| {
                row[key].as_str().map(str::to_string).ok_or_else(|| {
                    CapabilityResolutionError(format!("registry row missing string {key}"))
                })
            };
            let dependencies = row["dependency_requirements"]
                .as_array()
                .ok_or_else(|| {
                    CapabilityResolutionError("dependency_requirements must be an array".into())
                })?
                .iter()
                .map(|dependency| {
                    Ok(DependencyDeclaration {
                        dependency_id: dependency["dependency_id"]
                            .as_str()
                            .ok_or_else(|| {
                                CapabilityResolutionError("dependency id missing".into())
                            })?
                            .to_string(),
                        dependency_kind: serde_json::from_value(
                            dependency["dependency_kind"].clone(),
                        )
                        .map_err(|error| {
                            CapabilityResolutionError(format!("invalid dependency kind: {error}"))
                        })?,
                        requirement: serde_json::from_value(dependency["requirement"].clone())
                            .map_err(|error| {
                                CapabilityResolutionError(format!(
                                    "invalid dependency requirement: {error}"
                                ))
                            })?,
                    })
                })
                .collect::<Result<Vec<_>, CapabilityResolutionError>>()?;
            Ok(CapabilityDeclaration {
                engine_id: text("id")?,
                display_name: text("display_name")?,
                runtime_kind: serde_json::from_value(row["runtime_class"].clone()).map_err(
                    |error| CapabilityResolutionError(format!("invalid runtime kind: {error}")),
                )?,
                dependencies,
                operations: serde_json::from_value(row["operations"].clone()).map_err(|error| {
                    CapabilityResolutionError(format!("invalid operation metadata: {error}"))
                })?,
                required_phase: row["required_phase"].as_u64().map(|value| value as u8),
                implementation_version: row["implementation_version"].as_str().map(str::to_string),
            })
        })
        .collect()
}

/// Resolve the registry declaration plus bounded observations into a stable
/// registry-ordered capability list. Every declaration produces one row.
pub fn resolve_capabilities(
    declarations: &[CapabilityDeclaration],
    registrations: &[RegistrationObservation],
    dependency_observations: &[(String, DependencyObservation)],
) -> Result<Vec<ResolvedCapability>, CapabilityResolutionError> {
    let mut declarations_by_id = BTreeMap::new();
    for declaration in declarations {
        if declarations_by_id
            .insert(declaration.engine_id.as_str(), declaration)
            .is_some()
        {
            return Err(CapabilityResolutionError(format!(
                "duplicate declaration: {}",
                declaration.engine_id
            )));
        }
    }

    let mut registrations_by_id = BTreeMap::new();
    for observation in registrations {
        if !declarations_by_id.contains_key(observation.engine_id.as_str()) {
            return Err(CapabilityResolutionError(format!(
                "registration references unknown engine: {}",
                observation.engine_id
            )));
        }
        if registrations_by_id
            .insert(observation.engine_id.as_str(), observation)
            .is_some()
        {
            return Err(CapabilityResolutionError(format!(
                "conflicting registration observations: {}",
                observation.engine_id
            )));
        }
    }

    let mut dependency_by_key = BTreeMap::new();
    for (engine_id, observation) in dependency_observations {
        let Some(declaration) = declarations_by_id.get(engine_id.as_str()) else {
            return Err(CapabilityResolutionError(format!(
                "dependency observation references unknown engine: {}",
                engine_id
            )));
        };
        let Some(spec) = declaration
            .dependencies
            .iter()
            .find(|spec| spec.dependency_id == observation.dependency_id)
        else {
            return Err(CapabilityResolutionError(format!(
                "dependency {} is not declared for {}",
                observation.dependency_id, engine_id
            )));
        };
        if spec.dependency_kind != observation.dependency_kind
            || spec.requirement != observation.requirement
        {
            return Err(CapabilityResolutionError(format!(
                "dependency metadata mismatch for {}:{}",
                engine_id, observation.dependency_id
            )));
        }
        let key = (engine_id.clone(), observation.dependency_id.clone());
        if dependency_by_key.insert(key.clone(), observation).is_some() {
            return Err(CapabilityResolutionError(format!(
                "conflicting dependency observations: {}:{}",
                key.0, key.1
            )));
        }
    }

    declarations
        .iter()
        .map(|declaration| {
            let registration = registrations_by_id.get(declaration.engine_id.as_str());
            let registered = registration.map(|item| item.registered).unwrap_or(false);
            let registration_reason = registration
                .map(|item| item.reason_code)
                .unwrap_or(CapabilityReasonCode::NotObserved);
            let observations = declaration
                .dependencies
                .iter()
                .filter_map(|spec| {
                    dependency_by_key
                        .get(&(declaration.engine_id.clone(), spec.dependency_id.clone()))
                        .copied()
                })
                .cloned()
                .collect::<Vec<_>>();

            let required_failure = observations.iter().any(|observation| {
                observation.requirement == DependencyRequirement::Required
                    && observation.availability == CapabilityAvailability::Unavailable
            });
            let optional_degraded = observations.iter().any(|observation| {
                observation.requirement == DependencyRequirement::Optional
                    && matches!(
                        observation.availability,
                        CapabilityAvailability::Unavailable | CapabilityAvailability::Degraded
                    )
            });
            let missing_required = declaration.dependencies.iter().any(|spec| {
                spec.requirement == DependencyRequirement::Required
                    && !observations
                        .iter()
                        .any(|observation| observation.dependency_id == spec.dependency_id)
            });
            let all_required_available = declaration
                .dependencies
                .iter()
                .filter(|spec| spec.requirement == DependencyRequirement::Required)
                .all(|spec| {
                    observations.iter().any(|observation| {
                        observation.dependency_id == spec.dependency_id
                            && observation.availability == CapabilityAvailability::Available
                    })
                });

            let (availability, reason_code) = if required_failure {
                (
                    CapabilityAvailability::Unavailable,
                    CapabilityReasonCode::RequiredDependencyUnavailable,
                )
            } else if optional_degraded {
                (
                    CapabilityAvailability::Degraded,
                    CapabilityReasonCode::OptionalDependencyUnavailable,
                )
            } else if registered && all_required_available && !missing_required {
                (
                    CapabilityAvailability::Available,
                    CapabilityReasonCode::CapabilityAvailable,
                )
            } else if registration.is_some()
                && !registered
                && registration_reason != CapabilityReasonCode::NotObserved
            {
                (CapabilityAvailability::Unavailable, registration_reason)
            } else {
                (
                    CapabilityAvailability::Declared,
                    CapabilityReasonCode::NotObserved,
                )
            };

            let capability = EngineCapability {
                contract_version: ContractVersion::V1,
                engine_id: declaration.engine_id.clone(),
                display_name: declaration.display_name.clone(),
                availability,
                runtime_kind: declaration.runtime_kind,
                dependencies: declaration
                    .dependencies
                    .iter()
                    .map(|spec| spec.dependency_id.clone())
                    .collect(),
                required_phase: declaration.required_phase,
                implementation_version: declaration.implementation_version.clone(),
                reason_code: Some(reason_code),
                dependency_observations: Some(observations),
                operations: Some(declaration.operations),
            };
            Ok(ResolvedCapability {
                capability,
                registered,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use noesis_core::contract::OperationSupport;

    fn declarations() -> Vec<CapabilityDeclaration> {
        (0..19)
            .map(|index| CapabilityDeclaration {
                engine_id: format!("engine-{index:02}"),
                display_name: format!("Engine {index:02}"),
                runtime_kind: RuntimeKind::Native,
                dependencies: if index == 0 {
                    vec![DependencyDeclaration {
                        dependency_id: "database:postgres".into(),
                        dependency_kind: DependencyKind::Database,
                        requirement: DependencyRequirement::Required,
                    }]
                } else if index == 1 {
                    vec![DependencyDeclaration {
                        dependency_id: "python:mediapipe".into(),
                        dependency_kind: DependencyKind::Python,
                        requirement: DependencyRequirement::Optional,
                    }]
                } else {
                    Vec::new()
                },
                operations: CapabilityOperations {
                    calculate: OperationSupport::Supported,
                    validate: OperationSupport::Supported,
                    witness_eligible: index < 16,
                },
                required_phase: Some(0),
                implementation_version: Some("test".into()),
            })
            .collect()
    }

    fn registrations() -> Vec<RegistrationObservation> {
        declarations()
            .iter()
            .map(|declaration| RegistrationObservation {
                engine_id: declaration.engine_id.clone(),
                registered: declaration.engine_id != "engine-00",
                reason_code: if declaration.engine_id == "engine-00" {
                    CapabilityReasonCode::DatabaseUnconfigured
                } else {
                    CapabilityReasonCode::Registered
                },
            })
            .collect()
    }

    fn dependency(
        engine_id: &str,
        id: &str,
        requirement: DependencyRequirement,
        availability: CapabilityAvailability,
    ) -> (String, DependencyObservation) {
        (
            engine_id.to_string(),
            DependencyObservation {
                dependency_id: if id == "postgres" {
                    "database:postgres".into()
                } else {
                    "python:mediapipe".into()
                },
                dependency_kind: if id == "postgres" {
                    DependencyKind::Database
                } else {
                    DependencyKind::Python
                },
                requirement,
                availability,
                reason_code: if availability == CapabilityAvailability::Available {
                    CapabilityReasonCode::CapabilityAvailable
                } else {
                    CapabilityReasonCode::DependencyUnavailable
                },
            },
        )
    }

    #[test]
    fn resolver_preserves_all_rows_and_order_without_observations() {
        let resolved = resolve_capabilities(&declarations(), &[], &[]).unwrap();
        assert_eq!(resolved.len(), 19);
        assert_eq!(resolved[0].capability.engine_id, "engine-00");
        assert!(resolved.iter().all(|row| !row.registered));
        assert!(resolved
            .iter()
            .all(|row| row.capability.availability == CapabilityAvailability::Declared));
    }

    #[test]
    fn required_failure_wins_and_optional_failure_degrades() {
        let mut registrations = registrations();
        registrations[0].registered = true;
        let observations = vec![
            dependency(
                "engine-00",
                "postgres",
                DependencyRequirement::Required,
                CapabilityAvailability::Unavailable,
            ),
            dependency(
                "engine-01",
                "mediapipe",
                DependencyRequirement::Optional,
                CapabilityAvailability::Unavailable,
            ),
        ];
        let resolved =
            resolve_capabilities(&declarations(), &registrations, &observations).unwrap();
        assert_eq!(
            resolved[0].capability.availability,
            CapabilityAvailability::Unavailable
        );
        assert_eq!(
            resolved[1].capability.availability,
            CapabilityAvailability::Degraded
        );
    }

    #[test]
    fn registered_and_available_required_dependencies_produce_available() {
        let mut registrations = registrations();
        registrations[0].registered = true;
        let observations = vec![dependency(
            "engine-00",
            "postgres",
            DependencyRequirement::Required,
            CapabilityAvailability::Available,
        )];
        let resolved =
            resolve_capabilities(&declarations(), &registrations, &observations).unwrap();
        assert_eq!(
            resolved[0].capability.availability,
            CapabilityAvailability::Available
        );
        assert_eq!(
            resolved[0].capability.reason_code,
            Some(CapabilityReasonCode::CapabilityAvailable)
        );
    }

    #[test]
    fn duplicate_observations_fail_closed() {
        let observation = dependency(
            "engine-00",
            "postgres",
            DependencyRequirement::Required,
            CapabilityAvailability::Available,
        );
        let result = resolve_capabilities(
            &declarations(),
            &registrations(),
            &[observation.clone(), observation],
        );
        assert!(result.is_err());
    }

    #[test]
    fn conflicting_registration_fails_closed() {
        let mut observations = registrations();
        observations.push(RegistrationObservation {
            engine_id: "engine-00".into(),
            registered: true,
            reason_code: CapabilityReasonCode::Registered,
        });
        assert!(resolve_capabilities(&declarations(), &observations, &[]).is_err());
    }
}
