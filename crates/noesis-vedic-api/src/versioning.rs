//! API Versioning Strategy for Noesis Vedic API (FAPI-107)
//!
//! This module defines the versioning contract for the Vedic API service layer.
//! It supports multiple API versions (v1, v2) with automatic detection and routing,
//! enabling non-breaking evolution of the API surface.
//!
//! # Versioning Strategy
//!
//! We use **URI-path versioning** (e.g., `/v1/panchang`, `/v2/panchang`) combined
//! with an optional `Accept-Version` header for content negotiation. This approach:
//!
//! - Is explicit and discoverable (version visible in URL)
//! - Allows simultaneous operation of multiple versions
//! - Enables gradual migration without breaking existing clients
//!
//! # Version Lifecycle
//!
//! ```text
//! v1 (current stable) --> v2 (next) --> v3 (future)
//!     |                      |
//!     +--> deprecated        +--> active
//! ```
//!
//! Each version goes through: `active` -> `deprecated` -> `sunset`
//!
//! # Migration Path
//!
//! ```rust
//! use noesis_vedic_api::versioning::{ApiVersion, VersionRouter};
//!
//! let router = VersionRouter::new(ApiVersion::V2);
//!
//! // Detect version from a request path
//! let version = ApiVersion::from_path("/v1/panchang");
//! assert_eq!(version, Some(ApiVersion::V1));
//!
//! // Check if a version is still supported
//! assert!(ApiVersion::V1.is_supported());
//! ```

use std::fmt;

/// Supported API versions.
///
/// V1: Original direct-call API with raw FreeAstrologyAPI.com response shapes.
/// V2: Unified VedicApiService with enriched types, caching, metrics, and fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ApiVersion {
    /// Version 1: Direct API client with basic caching.
    /// - Raw response types from FreeAstrologyAPI.com
    /// - Manual cache key management
    /// - No built-in fallback
    /// - No metrics
    V1,

    /// Version 2: Unified service layer.
    /// - Enriched response types with computed fields
    /// - Automatic cache management
    /// - Native calculation fallback
    /// - Prometheus metrics integration
    /// - Circuit breaker protection
    V2,
}

impl ApiVersion {
    /// The current default version for new integrations.
    pub const CURRENT: ApiVersion = ApiVersion::V2;

    /// The minimum version still accepting traffic.
    pub const MINIMUM_SUPPORTED: ApiVersion = ApiVersion::V1;

    /// Parse a version from a URI path prefix.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use noesis_vedic_api::versioning::ApiVersion;
    ///
    /// assert_eq!(ApiVersion::from_path("/v1/panchang"), Some(ApiVersion::V1));
    /// assert_eq!(ApiVersion::from_path("/v2/birth_chart"), Some(ApiVersion::V2));
    /// assert_eq!(ApiVersion::from_path("/panchang"), None);
    /// ```
    pub fn from_path(path: &str) -> Option<Self> {
        let normalized = path.trim_start_matches('/');
        if normalized.starts_with("v1/") || normalized == "v1" {
            Some(ApiVersion::V1)
        } else if normalized.starts_with("v2/") || normalized == "v2" {
            Some(ApiVersion::V2)
        } else {
            None
        }
    }

    /// Parse a version from an `Accept-Version` header value.
    ///
    /// Accepts formats: "v1", "v2", "1", "2", "1.0", "2.0"
    pub fn from_header(value: &str) -> Option<Self> {
        let cleaned = value.trim().to_lowercase();
        match cleaned.as_str() {
            "v1" | "1" | "1.0" => Some(ApiVersion::V1),
            "v2" | "2" | "2.0" => Some(ApiVersion::V2),
            _ => None,
        }
    }

    /// Check if this version is still supported (not sunset).
    pub fn is_supported(&self) -> bool {
        *self >= Self::MINIMUM_SUPPORTED
    }

    /// Check if this version is deprecated (still works, but migration recommended).
    pub fn is_deprecated(&self) -> bool {
        *self < Self::CURRENT
    }

    /// Get the deprecation notice for this version, if deprecated.
    pub fn deprecation_notice(&self) -> Option<String> {
        if self.is_deprecated() {
            Some(format!(
                "API {} is deprecated. Please migrate to {}. See MIGRATION.md for details.",
                self, Self::CURRENT
            ))
        } else {
            None
        }
    }

    /// Get the URI prefix for this version.
    pub fn path_prefix(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "/v1",
            ApiVersion::V2 => "/v2",
        }
    }

    /// Strip the version prefix from a path, returning the remaining route.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use noesis_vedic_api::versioning::ApiVersion;
    ///
    /// assert_eq!(ApiVersion::V1.strip_prefix("/v1/panchang"), Some("/panchang"));
    /// assert_eq!(ApiVersion::V2.strip_prefix("/v2/birth_chart"), Some("/birth_chart"));
    /// assert_eq!(ApiVersion::V1.strip_prefix("/v2/panchang"), None);
    /// ```
    pub fn strip_prefix<'a>(&self, path: &'a str) -> Option<&'a str> {
        let prefix = self.path_prefix();
        if path.starts_with(prefix) {
            let rest = &path[prefix.len()..];
            if rest.is_empty() {
                Some("/")
            } else {
                Some(rest)
            }
        } else {
            None
        }
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiVersion::V1 => write!(f, "v1"),
            ApiVersion::V2 => write!(f, "v2"),
        }
    }
}

impl std::str::FromStr for ApiVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        ApiVersion::from_header(s)
            .ok_or_else(|| format!("Unknown API version: '{}'. Supported: v1, v2", s))
    }
}

/// Version-aware request router.
///
/// Detects the API version from incoming requests and routes to the
/// appropriate handler, with automatic deprecation headers.
#[derive(Debug, Clone)]
pub struct VersionRouter {
    /// Default version when none is specified.
    default_version: ApiVersion,
}

impl VersionRouter {
    /// Create a new router with the given default version.
    pub fn new(default_version: ApiVersion) -> Self {
        Self { default_version }
    }

    /// Resolve the API version for a given request.
    ///
    /// Priority order:
    /// 1. URI path version (`/v2/panchang`)
    /// 2. `Accept-Version` header
    /// 3. Default version
    pub fn resolve_version(
        &self,
        path: &str,
        accept_version_header: Option<&str>,
    ) -> VersionResolution {
        // 1. Try path-based detection
        if let Some(version) = ApiVersion::from_path(path) {
            return VersionResolution {
                version,
                source: VersionSource::Path,
                deprecated: version.is_deprecated(),
                deprecation_notice: version.deprecation_notice(),
            };
        }

        // 2. Try header-based detection
        if let Some(header_value) = accept_version_header {
            if let Some(version) = ApiVersion::from_header(header_value) {
                return VersionResolution {
                    version,
                    source: VersionSource::Header,
                    deprecated: version.is_deprecated(),
                    deprecation_notice: version.deprecation_notice(),
                };
            }
        }

        // 3. Fall back to default
        VersionResolution {
            version: self.default_version,
            source: VersionSource::Default,
            deprecated: self.default_version.is_deprecated(),
            deprecation_notice: self.default_version.deprecation_notice(),
        }
    }

    /// Get the default version.
    pub fn default_version(&self) -> ApiVersion {
        self.default_version
    }

    /// List all supported versions.
    pub fn supported_versions() -> Vec<ApiVersion> {
        vec![ApiVersion::V1, ApiVersion::V2]
    }
}

impl Default for VersionRouter {
    fn default() -> Self {
        Self::new(ApiVersion::CURRENT)
    }
}

/// Result of version resolution, including metadata for response headers.
#[derive(Debug, Clone)]
pub struct VersionResolution {
    /// The resolved API version.
    pub version: ApiVersion,
    /// How the version was determined.
    pub source: VersionSource,
    /// Whether this version is deprecated.
    pub deprecated: bool,
    /// Deprecation notice to include in response headers, if applicable.
    pub deprecation_notice: Option<String>,
}

impl VersionResolution {
    /// Get HTTP headers that should be added to the response.
    pub fn response_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            ("X-API-Version".to_string(), self.version.to_string()),
        ];

        if self.deprecated {
            headers.push(("Deprecation".to_string(), "true".to_string()));
            if let Some(ref notice) = self.deprecation_notice {
                headers.push(("Sunset-Notice".to_string(), notice.clone()));
            }
            headers.push((
                "Link".to_string(),
                format!("<{}>; rel=\"successor-version\"", ApiVersion::CURRENT.path_prefix()),
            ));
        }

        headers
    }
}

/// How the API version was determined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSource {
    /// Extracted from URL path (`/v1/...`, `/v2/...`).
    Path,
    /// Extracted from `Accept-Version` header.
    Header,
    /// Used the configured default.
    Default,
}

impl fmt::Display for VersionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionSource::Path => write!(f, "path"),
            VersionSource::Header => write!(f, "header"),
            VersionSource::Default => write!(f, "default"),
        }
    }
}

/// Breaking changes between API versions.
///
/// This constant documents all breaking changes for programmatic access.
/// For human-readable migration guide, see `MIGRATION.md`.
pub const BREAKING_CHANGES_V1_TO_V2: &[BreakingChange] = &[
    BreakingChange {
        category: "Client Construction",
        v1_pattern: "VedicApiClient::new(config)",
        v2_pattern: "VedicApiService::from_env()",
        description: "Service layer replaces direct client construction. Caching, rate limiting, and fallback are automatic.",
    },
    BreakingChange {
        category: "Panchang Response",
        v1_pattern: "client.get_panchang(...) -> Panchang",
        v2_pattern: "service.complete_panchang(...) -> CompletePanchang",
        description: "CompletePanchang includes Muhurtas, Hora, and Choghadiya in addition to base Panchang.",
    },
    BreakingChange {
        category: "Error Handling",
        v1_pattern: "Manual error matching with reqwest::Error",
        v2_pattern: "VedicApiError with is_retryable() and should_fallback()",
        description: "Typed error variants with retry and fallback classification.",
    },
    BreakingChange {
        category: "Cache Management",
        v1_pattern: "Manual cache key generation and lookup",
        v2_pattern: "Automatic transparent caching via CachedVedicClient",
        description: "Cache is fully managed. No manual key management needed.",
    },
    BreakingChange {
        category: "Rate Limiting",
        v1_pattern: "No built-in rate limiting",
        v2_pattern: "Automatic rate limiting with 5-request safety buffer",
        description: "Rate limiter tracks daily usage and throttles at 1 req/sec.",
    },
    BreakingChange {
        category: "Metrics",
        v1_pattern: "No metrics",
        v2_pattern: "NoesisMetrics with Prometheus export",
        description: "Full observability with API calls, cache ratios, response times, and error tracking.",
    },
];

/// A documented breaking change between API versions.
#[derive(Debug, Clone)]
pub struct BreakingChange {
    /// Category of the breaking change.
    pub category: &'static str,
    /// How it was done in V1.
    pub v1_pattern: &'static str,
    /// How it should be done in V2.
    pub v2_pattern: &'static str,
    /// Explanation of the change.
    pub description: &'static str,
}

impl fmt::Display for BreakingChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} -> {} ({})",
            self.category, self.v1_pattern, self.v2_pattern, self.description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_from_path() {
        assert_eq!(ApiVersion::from_path("/v1/panchang"), Some(ApiVersion::V1));
        assert_eq!(ApiVersion::from_path("/v2/birth_chart"), Some(ApiVersion::V2));
        assert_eq!(ApiVersion::from_path("/v1"), Some(ApiVersion::V1));
        assert_eq!(ApiVersion::from_path("/v2"), Some(ApiVersion::V2));
        assert_eq!(ApiVersion::from_path("/panchang"), None);
        assert_eq!(ApiVersion::from_path("/v3/panchang"), None);
        assert_eq!(ApiVersion::from_path("v1/panchang"), Some(ApiVersion::V1));
    }

    #[test]
    fn test_version_from_header() {
        assert_eq!(ApiVersion::from_header("v1"), Some(ApiVersion::V1));
        assert_eq!(ApiVersion::from_header("v2"), Some(ApiVersion::V2));
        assert_eq!(ApiVersion::from_header("1"), Some(ApiVersion::V1));
        assert_eq!(ApiVersion::from_header("2"), Some(ApiVersion::V2));
        assert_eq!(ApiVersion::from_header("1.0"), Some(ApiVersion::V1));
        assert_eq!(ApiVersion::from_header("2.0"), Some(ApiVersion::V2));
        assert_eq!(ApiVersion::from_header("V1"), Some(ApiVersion::V1));
        assert_eq!(ApiVersion::from_header(" v2 "), Some(ApiVersion::V2));
        assert_eq!(ApiVersion::from_header("v3"), None);
        assert_eq!(ApiVersion::from_header(""), None);
    }

    #[test]
    fn test_version_is_supported() {
        assert!(ApiVersion::V1.is_supported());
        assert!(ApiVersion::V2.is_supported());
    }

    #[test]
    fn test_version_is_deprecated() {
        assert!(ApiVersion::V1.is_deprecated());
        assert!(!ApiVersion::V2.is_deprecated());
    }

    #[test]
    fn test_deprecation_notice() {
        assert!(ApiVersion::V1.deprecation_notice().is_some());
        assert!(ApiVersion::V1.deprecation_notice().unwrap().contains("MIGRATION.md"));
        assert!(ApiVersion::V2.deprecation_notice().is_none());
    }

    #[test]
    fn test_version_path_prefix() {
        assert_eq!(ApiVersion::V1.path_prefix(), "/v1");
        assert_eq!(ApiVersion::V2.path_prefix(), "/v2");
    }

    #[test]
    fn test_strip_prefix() {
        assert_eq!(ApiVersion::V1.strip_prefix("/v1/panchang"), Some("/panchang"));
        assert_eq!(ApiVersion::V2.strip_prefix("/v2/birth_chart"), Some("/birth_chart"));
        assert_eq!(ApiVersion::V1.strip_prefix("/v2/panchang"), None);
        assert_eq!(ApiVersion::V1.strip_prefix("/v1"), Some("/"));
    }

    #[test]
    fn test_version_display() {
        assert_eq!(format!("{}", ApiVersion::V1), "v1");
        assert_eq!(format!("{}", ApiVersion::V2), "v2");
    }

    #[test]
    fn test_version_from_str() {
        assert_eq!("v1".parse::<ApiVersion>().unwrap(), ApiVersion::V1);
        assert_eq!("v2".parse::<ApiVersion>().unwrap(), ApiVersion::V2);
        assert!("v3".parse::<ApiVersion>().is_err());
    }

    #[test]
    fn test_version_ordering() {
        assert!(ApiVersion::V1 < ApiVersion::V2);
    }

    #[test]
    fn test_router_resolve_from_path() {
        let router = VersionRouter::new(ApiVersion::V2);

        let result = router.resolve_version("/v1/panchang", None);
        assert_eq!(result.version, ApiVersion::V1);
        assert_eq!(result.source, VersionSource::Path);
        assert!(result.deprecated);
    }

    #[test]
    fn test_router_resolve_from_header() {
        let router = VersionRouter::new(ApiVersion::V2);

        let result = router.resolve_version("/panchang", Some("v1"));
        assert_eq!(result.version, ApiVersion::V1);
        assert_eq!(result.source, VersionSource::Header);
    }

    #[test]
    fn test_router_resolve_default() {
        let router = VersionRouter::new(ApiVersion::V2);

        let result = router.resolve_version("/panchang", None);
        assert_eq!(result.version, ApiVersion::V2);
        assert_eq!(result.source, VersionSource::Default);
        assert!(!result.deprecated);
    }

    #[test]
    fn test_router_path_takes_priority_over_header() {
        let router = VersionRouter::new(ApiVersion::V2);

        // Path says v1, header says v2 -- path wins
        let result = router.resolve_version("/v1/panchang", Some("v2"));
        assert_eq!(result.version, ApiVersion::V1);
        assert_eq!(result.source, VersionSource::Path);
    }

    #[test]
    fn test_resolution_response_headers_deprecated() {
        let resolution = VersionResolution {
            version: ApiVersion::V1,
            source: VersionSource::Path,
            deprecated: true,
            deprecation_notice: Some("Migrate to v2".to_string()),
        };

        let headers = resolution.response_headers();
        assert!(headers.iter().any(|(k, v)| k == "X-API-Version" && v == "v1"));
        assert!(headers.iter().any(|(k, v)| k == "Deprecation" && v == "true"));
        assert!(headers.iter().any(|(k, _)| k == "Sunset-Notice"));
        assert!(headers.iter().any(|(k, v)| k == "Link" && v.contains("successor-version")));
    }

    #[test]
    fn test_resolution_response_headers_current() {
        let resolution = VersionResolution {
            version: ApiVersion::V2,
            source: VersionSource::Default,
            deprecated: false,
            deprecation_notice: None,
        };

        let headers = resolution.response_headers();
        assert_eq!(headers.len(), 1);
        assert!(headers.iter().any(|(k, v)| k == "X-API-Version" && v == "v2"));
    }

    #[test]
    fn test_supported_versions() {
        let versions = VersionRouter::supported_versions();
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&ApiVersion::V1));
        assert!(versions.contains(&ApiVersion::V2));
    }

    #[test]
    fn test_breaking_changes_documented() {
        assert!(BREAKING_CHANGES_V1_TO_V2.len() >= 6);
        for change in BREAKING_CHANGES_V1_TO_V2 {
            assert!(!change.category.is_empty());
            assert!(!change.v1_pattern.is_empty());
            assert!(!change.v2_pattern.is_empty());
            assert!(!change.description.is_empty());
        }
    }

    #[test]
    fn test_breaking_change_display() {
        let change = &BREAKING_CHANGES_V1_TO_V2[0];
        let display = format!("{}", change);
        assert!(display.contains(change.category));
    }

    #[test]
    fn test_version_source_display() {
        assert_eq!(format!("{}", VersionSource::Path), "path");
        assert_eq!(format!("{}", VersionSource::Header), "header");
        assert_eq!(format!("{}", VersionSource::Default), "default");
    }
}
