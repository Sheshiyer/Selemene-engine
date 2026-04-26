use noesis_api::workflow_parity::workflow_registry_parity;
use noesis_orchestrator::WorkflowOrchestrator;
use std::process::ExitCode;

fn main() -> ExitCode {
    let orchestrator = WorkflowOrchestrator::new();
    let parity = workflow_registry_parity(&orchestrator);

    if parity.is_exact_match() {
        println!(
            "Workflow registry parity: {}/{} canonical workflows registered",
            parity.actual_count, parity.expected_count
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "Workflow registry parity mismatch: missing={:?} unexpected={:?} actual={} expected={}",
            parity.missing_ids, parity.unexpected_ids, parity.actual_count, parity.expected_count
        );
        ExitCode::FAILURE
    }
}
