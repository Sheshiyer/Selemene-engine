//! Vimshottari helper utilities and enrichment.
//!
//! PR2 of 3: the previously-orphaned `api` and `types` submodules are now
//! declared so the native facade (`api::compute_vimshottari_native`,
//! `api::VimshottariRequest`, etc.) is reachable from the rest of the crate.

pub mod api;
pub mod enrichment;
pub mod query;
pub mod types;
