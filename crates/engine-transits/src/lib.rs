//! Transit Analysis Consciousness Engine
//!
//! Calculates planetary transits relative to natal positions using Swiss Ephemeris,
//! detects natal-to-transit aspects, Sade Sati status, and generates
//! consciousness-level tiered witness prompts.
//!
//! Thread safety: Reuses `EphemerisCalculator` and `EPHE_MUTEX` from
//! `engine-human-design` to serialize Swiss Ephemeris C library access.

pub mod aspects;
pub mod engine;
pub mod ephemeris;
pub mod models;
pub mod sade_sati;
pub mod witness;

pub use engine::TransitsEngine;
