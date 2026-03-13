use noesis_orchestrator::WorkflowOrchestrator;
use std::collections::BTreeSet;

pub const CANONICAL_WORKFLOW_IDS: [&str; 6] = [
    "birth-blueprint",
    "daily-practice",
    "decision-support",
    "self-inquiry",
    "creative-expression",
    "full-spectrum",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRegistryParity {
    pub expected_count: usize,
    pub actual_count: usize,
    pub missing_ids: Vec<String>,
    pub unexpected_ids: Vec<String>,
}

impl WorkflowRegistryParity {
    pub fn is_exact_match(&self) -> bool {
        self.missing_ids.is_empty()
            && self.unexpected_ids.is_empty()
            && self.actual_count == self.expected_count
    }
}

pub fn evaluate_workflow_registry_parity<'a, I>(workflow_ids: I) -> WorkflowRegistryParity
where
    I: IntoIterator<Item = &'a str>,
{
    let expected = CANONICAL_WORKFLOW_IDS
        .iter()
        .copied()
        .map(String::from)
        .collect::<BTreeSet<_>>();
    let actual = workflow_ids
        .into_iter()
        .map(String::from)
        .collect::<BTreeSet<_>>();

    let missing_ids = expected.difference(&actual).cloned().collect::<Vec<_>>();
    let unexpected_ids = actual.difference(&expected).cloned().collect::<Vec<_>>();

    WorkflowRegistryParity {
        expected_count: expected.len(),
        actual_count: actual.len(),
        missing_ids,
        unexpected_ids,
    }
}

pub fn workflow_registry_parity(orchestrator: &WorkflowOrchestrator) -> WorkflowRegistryParity {
    let workflow_ids = orchestrator
        .list_workflows()
        .iter()
        .map(|workflow| workflow.id.as_str())
        .collect::<Vec<_>>();
    evaluate_workflow_registry_parity(workflow_ids)
}

pub fn log_workflow_registry_parity(orchestrator: &WorkflowOrchestrator) -> WorkflowRegistryParity {
    let parity = workflow_registry_parity(orchestrator);

    if parity.is_exact_match() {
        tracing::info!(
            expected = parity.expected_count,
            actual = parity.actual_count,
            "Workflow registry parity: {}/{} canonical workflows registered",
            parity.actual_count,
            parity.expected_count
        );
    } else {
        tracing::warn!(
            expected = parity.expected_count,
            actual = parity.actual_count,
            missing_ids = ?parity.missing_ids,
            unexpected_ids = ?parity.unexpected_ids,
            "Workflow registry parity mismatch detected"
        );
    }

    parity
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_workflow_registry_parity, workflow_registry_parity, CANONICAL_WORKFLOW_IDS,
    };
    use noesis_orchestrator::WorkflowOrchestrator;

    #[test]
    fn exact_match_for_default_orchestrator() {
        let orchestrator = WorkflowOrchestrator::new();
        let parity = workflow_registry_parity(&orchestrator);

        assert!(parity.is_exact_match());
        assert_eq!(parity.expected_count, CANONICAL_WORKFLOW_IDS.len());
        assert_eq!(parity.actual_count, CANONICAL_WORKFLOW_IDS.len());
        assert!(parity.missing_ids.is_empty());
        assert!(parity.unexpected_ids.is_empty());
    }

    #[test]
    fn detects_missing_workflow_ids() {
        let parity = evaluate_workflow_registry_parity(vec![
            "birth-blueprint",
            "daily-practice",
            "decision-support",
            "self-inquiry",
            "creative-expression",
        ]);

        assert!(!parity.is_exact_match());
        assert_eq!(parity.expected_count, CANONICAL_WORKFLOW_IDS.len());
        assert_eq!(parity.actual_count, 5);
        assert_eq!(parity.missing_ids, vec!["full-spectrum".to_string()]);
        assert!(parity.unexpected_ids.is_empty());
    }

    #[test]
    fn detects_unexpected_workflow_ids() {
        let parity = evaluate_workflow_registry_parity(vec![
            "birth-blueprint",
            "daily-practice",
            "decision-support",
            "self-inquiry",
            "creative-expression",
            "full-spectrum",
            "custom-lab",
        ]);

        assert!(!parity.is_exact_match());
        assert_eq!(parity.expected_count, CANONICAL_WORKFLOW_IDS.len());
        assert_eq!(parity.actual_count, 7);
        assert!(parity.missing_ids.is_empty());
        assert_eq!(parity.unexpected_ids, vec!["custom-lab".to_string()]);
    }
}
