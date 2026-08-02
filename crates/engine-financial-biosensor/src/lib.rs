//! Financial Biosensor — a decision-reflection surface for the Noesis Engine.
//!
//! This engine composes five registered engines and one optional biometric
//! sample into a single legible reading. It reports what its sources register;
//! it does not decide, and it does not forecast.
//!
//! # What the number is
//!
//! The Daily Decision Index is a **declared house model**. Its weights are
//! house constants, not values fitted to data, and every payload carries a
//! [`Declaration`](models::Declaration) saying so alongside a
//! [`ProvenanceBlock`](models::ProvenanceBlock) recording, per contributor,
//! the source engine, its version, the exact fields read, and the
//! normalization applied. A reader can therefore check the number without
//! reading this crate.
//!
//! # What happens when a source cannot be read
//!
//! Absence is an ordinary outcome. The composite renormalizes over whatever
//! could be read, names what could not, and lowers the confidence it reports
//! about itself. Below a coverage of 0.40 the value is withheld entirely,
//! because at that point the composite would restate a single source.
//!
//! # Kha-Ba-La
//!
//! The biometric sample is the **Ba** leg and carries the largest single
//! weight, because it is the only contributor measured in the present; every
//! other input is a deterministic function of birth data and clock time and so
//! cannot contradict itself. Declared deliberation is the **La** leg: the
//! friction of sitting with a real-stakes decision for as long as one's own
//! authority asks. The remaining four are **Kha**. When a leg cannot be read,
//! the reading says which one is missing rather than absorbing the gap.

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )
)]

pub mod composite;
pub mod contributors;
pub mod engine;
pub mod landscape;
pub mod models;
pub mod reflection;
pub mod witness;

pub use engine::{FinancialBiosensorEngine, SourceEngines, ENGINE_ID, ENGINE_NAME, REQUIRED_PHASE};
pub use models::{
    Contributor, ContributorId, ContributorProvenance, ContributorStatus, Convergence,
    DailyDecisionIndex, Declaration, FinancialBiosensorResult, ProvenanceBlock, Sufficiency,
    AUTHORSHIP_STATEMENT, CLAIM_MODE, FORMULA_VERSION, HOUSE_MODEL_STATEMENT,
};
