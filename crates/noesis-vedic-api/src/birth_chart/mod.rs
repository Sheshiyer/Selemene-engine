//! Birth chart enrichment helpers
//!
//! PR1 (validation/vedic-hardening) removed `api.rs` and `mappers.rs`. The
//! `api.rs` types modelled a vendor schema that does not exist on the live
//! FreeAstrologyAPI (it called `POST /horoscope-chart`, which returns
//! HTTP 403 — route not found). `mappers.rs` was a no-op consumer of those
//! types. The live D1 path now flows through `VedicApiClient::get_birth_chart`
//! → `/planets` → `crate::chart_mapping::map_planets_envelope_to_birth_chart`.
//!
//! The submodules below (`aspects`, `dignities`, `status`, `types`) continue
//! to provide enrichment helpers over the typed `chart::BirthChart`.

pub mod aspects;
pub mod dignities;
pub mod status;
pub mod types;

pub use types::*;
