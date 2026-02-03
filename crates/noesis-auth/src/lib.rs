//! Noesis Auth -- JWT + API key authentication and authorization
//!
//! Migrated from the original Selemene Engine auth system.
//! Provides Claims, ApiKey, AuthUser, AuthService, UserRateLimiter, and TierLimits.

use noesis_core::EngineError;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

pub mod password;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // Subject (user ID)
    pub exp: usize,            // Expiration time
    pub iat: usize,            // Issued at
    pub tier: String,          // User tier (free, premium, enterprise)
    pub permissions: Vec<String>, // User permissions
    pub consciousness_level: u8,  // User consciousness level (0-5)
}

/// API key structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub user_id: String,
    pub tier: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub rate_limit: u32,       // Requests per minute
    pub consciousness_level: u8, // User consciousness level (0-5)
}

/// User authentication information
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub tier: String,
    pub permissions: Vec<String>,
    pub rate_limit: u32,
    pub consciousness_level: u8,
}

/// Authentication service
pub struct AuthService {
    jwt_secret: String,
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    jwt_validation: Validation,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["exp", "iat", "sub"]);

        Self {
            jwt_secret,
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            jwt_validation: validation,
        }
    }

    /// Validate JWT token
    pub async fn validate_jwt_token(&self, token: &str) -> Result<AuthUser, EngineError> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());

        let token_data = decode::<Claims>(token, &decoding_key, &self.jwt_validation)
            .map_err(|e| EngineError::AuthError(format!("Invalid JWT token: {}", e)))?;

        let claims = token_data.claims;

        // Check if token is expired
        let now = Utc::now().timestamp() as usize;
        if claims.exp < now {
            return Err(EngineError::AuthError("Token expired".to_string()));
        }

        // Get rate limit based on tier
        let rate_limit = self.get_rate_limit_for_tier(&claims.tier);

        Ok(AuthUser {
            user_id: claims.sub,
            tier: claims.tier,
            permissions: claims.permissions,
            rate_limit,
            consciousness_level: claims.consciousness_level,
        })
    }

    /// Validate API key
    pub async fn validate_api_key(&self, api_key: &str) -> Result<AuthUser, EngineError> {
        let keys = self.api_keys.read().await;

        if let Some(key_info) = keys.get(api_key) {
            // Check if key is expired
            if let Some(expires_at) = key_info.expires_at {
                if Utc::now() > expires_at {
                    return Err(EngineError::AuthError("API key expired".to_string()));
                }
            }

            // Update last used timestamp
            let user_id = key_info.user_id.clone();
            let tier = key_info.tier.clone();
            let permissions = key_info.permissions.clone();
            let rate_limit = key_info.rate_limit;
            let consciousness_level = key_info.consciousness_level;
            drop(keys);
            self.update_api_key_usage(api_key).await?;

            Ok(AuthUser {
                user_id,
                tier,
                permissions,
                rate_limit,
                consciousness_level,
            })
        } else {
            Err(EngineError::AuthError("Invalid API key".to_string()))
        }
    }

    /// Generate JWT token
    pub fn generate_jwt_token(&self, user_id: &str, tier: &str, permissions: &[String], consciousness_level: u8) -> Result<String, EngineError> {
        let now = Utc::now();
        let exp = (now + Duration::hours(24)).timestamp() as usize; // 24 hour expiration

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
            iat: now.timestamp() as usize,
            tier: tier.to_string(),
            permissions: permissions.to_vec(),
            consciousness_level,
        };

        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());

        encode(&Header::default(), &claims, &encoding_key)
            .map_err(|e| EngineError::AuthError(format!("Failed to generate JWT: {}", e)))
    }

    /// Add API key
    pub async fn add_api_key(&self, api_key: ApiKey) -> Result<(), EngineError> {
        let mut keys = self.api_keys.write().await;
        keys.insert(api_key.key.clone(), api_key);
        Ok(())
    }

    /// Remove API key
    pub async fn remove_api_key(&self, key: &str) -> Result<(), EngineError> {
        let mut keys = self.api_keys.write().await;
        keys.remove(key);
        Ok(())
    }

    /// Update API key usage
    async fn update_api_key_usage(&self, key: &str) -> Result<(), EngineError> {
        let mut keys = self.api_keys.write().await;
        if let Some(key_info) = keys.get_mut(key) {
            key_info.last_used = Some(Utc::now());
        }
        Ok(())
    }

    /// Get rate limit for user tier
    fn get_rate_limit_for_tier(&self, tier: &str) -> u32 {
        match tier {
            "free" => 60,        // 60 requests per minute
            "premium" => 1000,   // 1000 requests per minute
            "enterprise" => 10000, // 10000 requests per minute
            _ => 10,             // Default: 10 requests per minute
        }
    }

    /// Check if user has permission
    pub fn has_permission(user: &AuthUser, permission: &str) -> bool {
        user.permissions.contains(&permission.to_string())
    }

    /// Check if user can access endpoint
    pub fn can_access_endpoint(user: &AuthUser, endpoint: &str) -> bool {
        // Define endpoint permissions
        let endpoint_permissions = match endpoint {
            "/api/v1/panchanga" => vec!["panchanga:read"],
            "/api/v1/panchanga/batch" => vec!["panchanga:batch"],
            "/api/v1/admin/users" => vec!["admin:users"],
            "/api/v1/admin/analytics" => vec!["admin:analytics"],
            _ => vec!["basic:access"], // Default permission
        };

        // Check if user has any of the required permissions
        endpoint_permissions.iter().any(|perm| Self::has_permission(user, perm))
    }

    /// Get user tier limits
    pub fn get_tier_limits(tier: &str) -> TierLimits {
        match tier {
            "free" => TierLimits {
                max_concurrent_calculations: 1,
                max_batch_size: 10,
                max_precision: "standard",
                cache_ttl_hours: 1,
            },
            "premium" => TierLimits {
                max_concurrent_calculations: 10,
                max_batch_size: 1000,
                max_precision: "high",
                cache_ttl_hours: 24,
            },
            "enterprise" => TierLimits {
                max_concurrent_calculations: 100,
                max_batch_size: 10000,
                max_precision: "extreme",
                cache_ttl_hours: 168, // 1 week
            },
            _ => TierLimits {
                max_concurrent_calculations: 1,
                max_batch_size: 5,
                max_precision: "standard",
                cache_ttl_hours: 1,
            },
        }
    }
}

/// Tier limits for different user levels
#[derive(Debug, Clone)]
pub struct TierLimits {
    pub max_concurrent_calculations: u32,
    pub max_batch_size: u32,
    pub max_precision: &'static str,
    pub cache_ttl_hours: u32,
}

/// Rate limiter for individual users
pub struct UserRateLimiter {
    user_limits: Arc<RwLock<HashMap<String, (u32, DateTime<Utc>)>>>,
}

impl UserRateLimiter {
    pub fn new() -> Self {
        Self {
            user_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if user can make request
    pub async fn can_make_request(&self, user_id: &str, rate_limit: u32) -> bool {
        let mut limits = self.user_limits.write().await;
        let now = Utc::now();

        if let Some((count, window_start)) = limits.get_mut(user_id) {
            // Check if window has reset (1 minute)
            if now - *window_start > Duration::minutes(1) {
                *count = 1;
                *window_start = now;
                true
            } else if *count < rate_limit {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            // First request for this user
            limits.insert(user_id.to_string(), (1, now));
            true
        }
    }

    /// Get current usage for user
    pub async fn get_user_usage(&self, user_id: &str) -> Option<(u32, DateTime<Utc>)> {
        let limits = self.user_limits.read().await;
        limits.get(user_id).cloned()
    }
}
