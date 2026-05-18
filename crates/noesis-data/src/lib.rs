pub mod db;
pub mod models;
pub mod repositories;

pub use db::{create_pool, DbPool};

/// humdes validation persistence (Tier-2 storage). Gated behind the
/// `record-validation` feature so only explicit opt-in callers (a one-shot
/// binary or future post-test hook) pull in the writer. See issue #856.
#[cfg(feature = "record-validation")]
pub mod humdes_validation;
