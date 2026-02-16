//! Noesis SDK — Rust client library for Selemene Engine
//!
//! This crate provides everything needed to interact with the Selemene Engine API
//! from Rust applications, TUI clients, and desktop apps.
//!
//! # Modules
//!
//! - [`client`] — Typed HTTP client for all 16 engines and 6 workflows
//! - [`profile`] — Local profile management (birth_data persistence)
//! - [`keychain`] — Secure credential storage (Keychain on macOS, Secret Service on Linux)
//! - [`render`] — Markdown/JSON report rendering from engine outputs
//! - [`config`] — Runtime configuration (endpoints, timeouts, cache TTL)
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use noesis_sdk::{NoesisClient, LocalProfile, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), noesis_sdk::Error> {
//!     // Load config and profile
//!     let config = Config::load()?;
//!     let profile = LocalProfile::load()?;
//!
//!     // Create client
//!     let client = NoesisClient::new(&config)?;
//!
//!     // Run a calculation
//!     let input = profile.to_engine_input();
//!     let output = client.calculate("numerology", input).await?;
//!
//!     println!("Life Path: {}", output.result["life_path"]["value"]);
//!     println!("Witness: {}", output.witness_prompt);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod keychain;
pub mod profile;
pub mod render;

// Re-exports for convenience
pub use client::{NoesisClient, NoesisClientBuilder};
pub use config::Config;
pub use keychain::{CredentialStore, KeychainStore};
pub use profile::LocalProfile;
pub use render::{MarkdownRenderer, ReportFormat};

// Re-export core types
pub use noesis_core::{
    BirthData, CalculationMetadata, Coordinates, EngineInput, EngineOutput, Precision,
    WorkflowResult,
};

/// SDK error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Profile error: {0}")]
    Profile(String),

    #[error("Credential storage error: {0}")]
    Credential(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Render error: {0}")]
    Render(String),
}

/// Result type for SDK operations
pub type Result<T> = std::result::Result<T, Error>;
