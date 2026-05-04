use crate::error::ApiError;
use crate::handlers::admin::{default_rate_limit_for_tier, generate_secret_api_key};
use crate::handlers::auth::LoginResponse;
use crate::AppState;
use axum::{
    extract::{Json, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use noesis_auth::{sha256_hex, ApiKey};
use noesis_core::EngineError;
use noesis_data::models::user::User;
use noesis_data::repositories::admin_repository::NewApiKeyRecord;
use serde::{Deserialize, Serialize};
use std::future::Future;

const DISCORD_AUTHORIZE_URL: &str = "https://discord.com/api/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
const DISCORD_USER_URL: &str = "https://discord.com/api/users/@me";
const LOCALHOST_HOSTS: &[&str] = &["localhost", "127.0.0.1", "0.0.0.0"];
const OAUTH_DEFAULT_API_KEY_NAME: &str = "Discord OAuth default key";
const OAUTH_DEFAULT_API_KEY_PERMISSIONS: &[&str] = &["basic:access"];
const ALLOWED_DISCORD_CALLBACK_PATHS: &[&str] = &[
    "/admin/login/discord-callback",
    "/admin/auth/discord/callback",
    "/login/discord-callback",
];

fn normalize_callback_path(path: &str) -> String {
    if path == "/" {
        return "/".to_string();
    }

    path.trim_end_matches('/').to_string()
}

#[derive(Serialize)]
pub struct DiscordAuthorizeResponse {
    pub url: String,
}

#[derive(Deserialize)]
pub struct DiscordAuthorizeQuery {
    pub redirect_uri: Option<String>,
}

#[derive(Deserialize)]
pub struct DiscordCallbackRequest {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
    pub redirect_uri: Option<String>,
}

#[derive(Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    email: Option<String>,
}

#[derive(Debug)]
struct DefaultOauthApiKeySeed {
    secret_key: String,
    key_hash: String,
    key_prefix: String,
    tier: String,
    permissions: Vec<String>,
    consciousness_level: u8,
    rate_limit: i32,
}

fn build_default_oauth_api_key_seed(user: &User) -> DefaultOauthApiKeySeed {
    let secret_key = generate_secret_api_key();
    let tier = user.tier.to_ascii_lowercase();
    let permissions = OAUTH_DEFAULT_API_KEY_PERMISSIONS
        .iter()
        .map(|permission| (*permission).to_string())
        .collect::<Vec<_>>();

    DefaultOauthApiKeySeed {
        key_hash: sha256_hex(&secret_key),
        key_prefix: secret_key[..12.min(secret_key.len())].to_string(),
        secret_key,
        tier: tier.clone(),
        permissions,
        consciousness_level: user.consciousness_level.clamp(0, 5) as u8,
        rate_limit: default_rate_limit_for_tier(&tier),
    }
}

async fn ensure_default_api_key_for_oauth_login<CountFn, CountFut, CreateFn, CreateFut>(
    count_active_keys: CountFn,
    create_default_key: CreateFn,
) -> Result<bool, EngineError>
where
    CountFn: FnOnce() -> CountFut,
    CountFut: Future<Output = Result<i64, EngineError>>,
    CreateFn: FnOnce() -> CreateFut,
    CreateFut: Future<Output = Result<(), EngineError>>,
{
    let active_key_count = count_active_keys().await?;
    if active_key_count > 0 {
        return Ok(false);
    }

    create_default_key().await?;
    Ok(true)
}

async fn provision_default_api_key_for_oauth_login(
    state: &AppState,
    user: &User,
) -> Result<bool, EngineError> {
    let repo = state.admin_repository.as_ref().ok_or_else(|| {
        EngineError::ServiceUnavailable(
            "Admin repository not available for Discord OAuth key provisioning".to_string(),
        )
    })?;
    let seed = build_default_oauth_api_key_seed(user);

    ensure_default_api_key_for_oauth_login(
        || async {
            repo.count_api_keys(None, Some(user.id), true)
                .await
                .map_err(|e| {
                    EngineError::InternalError(format!(
                        "Failed to count active API keys for Discord OAuth login: {e}"
                    ))
                })
        },
        || async {
            let created = repo
                .create_api_key(NewApiKeyRecord {
                    key_hash: seed.key_hash,
                    name: Some(OAUTH_DEFAULT_API_KEY_NAME.to_string()),
                    key_prefix: seed.key_prefix,
                    user_id: user.id,
                    created_by_user_id: None,
                    tier: seed.tier.clone(),
                    permissions: serde_json::json!(seed.permissions),
                    consciousness_level: i32::from(seed.consciousness_level),
                    rate_limit: seed.rate_limit,
                    expires_at: None,
                    rotated_from_key_id: None,
                })
                .await
                .map_err(|e| {
                    EngineError::InternalError(format!(
                        "Failed to create default API key for Discord OAuth login: {e}"
                    ))
                })?;

            let _ = state
                .auth
                .add_api_key(ApiKey {
                    key: seed.secret_key,
                    user_id: created.user_id.to_string(),
                    tier: seed.tier,
                    permissions: seed.permissions,
                    created_at: created.created_at,
                    expires_at: created.expires_at,
                    last_used: created.last_used,
                    rate_limit: created.rate_limit as u32,
                    consciousness_level: created.consciousness_level as u8,
                })
                .await;

            Ok(())
        },
    )
    .await
}

fn is_localhost_host(host: &str) -> bool {
    LOCALHOST_HOSTS.contains(&host)
}

fn validate_requested_discord_redirect_uri(uri: &str) -> Result<reqwest::Url, EngineError> {
    let parsed = reqwest::Url::parse(uri).map_err(|_| {
        EngineError::ValidationError("Discord redirect URI must be an absolute URL.".to_string())
    })?;

    let host = parsed.host_str().ok_or_else(|| {
        EngineError::ValidationError("Discord redirect URI must include a host.".to_string())
    })?;

    let scheme = parsed.scheme();
    let scheme_allowed = scheme == "https" || (scheme == "http" && is_localhost_host(host));
    if !scheme_allowed {
        return Err(EngineError::ValidationError(
            "Discord redirect URI must use https, or http on localhost.".to_string(),
        ));
    }

    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(EngineError::ValidationError(
            "Discord redirect URI cannot include a query string or fragment.".to_string(),
        ));
    }

    let normalized_path = normalize_callback_path(parsed.path());
    if !ALLOWED_DISCORD_CALLBACK_PATHS.contains(&normalized_path.as_str()) {
        return Err(EngineError::ValidationError(
            "Discord redirect URI path is not allowed for the admin dashboard.".to_string(),
        ));
    }

    Ok(parsed)
}

fn origin_from_headers(headers: &HeaderMap) -> Result<Option<reqwest::Url>, EngineError> {
    let Some(origin) = headers.get(header::ORIGIN) else {
        return Ok(None);
    };

    let origin = origin.to_str().map_err(|_| {
        EngineError::ValidationError("Invalid Origin header for Discord OAuth request.".to_string())
    })?;

    let parsed = reqwest::Url::parse(origin).map_err(|_| {
        EngineError::ValidationError("Invalid Origin header for Discord OAuth request.".to_string())
    })?;

    Ok(Some(parsed))
}

fn same_origin(left: &reqwest::Url, right: &reqwest::Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn resolve_discord_redirect_uri(
    configured_redirect_uri: &str,
    requested_redirect_uri: Option<&str>,
    headers: &HeaderMap,
) -> Result<String, EngineError> {
    if let Some(requested_redirect_uri) = requested_redirect_uri {
        let requested = validate_requested_discord_redirect_uri(requested_redirect_uri)?;
        let Some(origin) = origin_from_headers(headers)? else {
            return Err(EngineError::ValidationError(
                "Discord redirect URI override requires a browser Origin header.".to_string(),
            ));
        };

        if !same_origin(&origin, &requested) {
            return Err(EngineError::ValidationError(
                "Discord redirect URI must stay on the current admin origin.".to_string(),
            ));
        }

        return Ok(requested.into());
    }

    Ok(configured_redirect_uri.to_string())
}

/// GET /api/v1/auth/discord/authorize — returns the Discord OAuth2 authorize URL
pub async fn discord_authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DiscordAuthorizeQuery>,
) -> Result<Response, ApiError> {
    let client_id = state
        .discord_client_id
        .as_ref()
        .ok_or_else(|| EngineError::ConfigError("Discord OAuth not configured".to_string()))?;
    let configured_redirect_uri = state.discord_redirect_uri.as_ref().ok_or_else(|| {
        EngineError::ConfigError("Discord redirect URI not configured".to_string())
    })?;
    let redirect_uri = resolve_discord_redirect_uri(
        configured_redirect_uri,
        query.redirect_uri.as_deref(),
        &headers,
    )?;

    // Build a simple state token using a HMAC of a timestamp to prevent CSRF.
    // In production you'd want a nonce stored server-side; this is a lightweight approach.
    let timestamp = chrono::Utc::now().timestamp();
    let state_token = format!("{}", timestamp);

    let url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=identify%20email&state={}",
        DISCORD_AUTHORIZE_URL,
        urlencoding::encode(client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&state_token),
    );

    Ok((StatusCode::OK, Json(DiscordAuthorizeResponse { url })).into_response())
}

/// POST /api/v1/auth/discord/callback — exchange Discord auth code for JWT
pub async fn discord_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DiscordCallbackRequest>,
) -> Result<Response, ApiError> {
    // Verify Discord OAuth is configured
    let client_id = state
        .discord_client_id
        .as_ref()
        .ok_or_else(|| EngineError::ConfigError("Discord OAuth not configured".to_string()))?;
    let client_secret = state.discord_client_secret.as_ref().ok_or_else(|| {
        EngineError::ConfigError("Discord client secret not configured".to_string())
    })?;
    let configured_redirect_uri = state.discord_redirect_uri.as_ref().ok_or_else(|| {
        EngineError::ConfigError("Discord redirect URI not configured".to_string())
    })?;
    let redirect_uri = resolve_discord_redirect_uri(
        configured_redirect_uri,
        payload.redirect_uri.as_deref(),
        &headers,
    )?;

    // Verify DB is available (OAuth requires DB to look up users)
    if !state.db_available {
        return Err(EngineError::ServiceUnavailable(
            "Database not available — OAuth endpoints require a database connection".to_string(),
        )
        .into());
    }

    let oauth_repo = state.oauth_repository.as_ref().ok_or_else(|| {
        EngineError::ServiceUnavailable("OAuth repository not available".to_string())
    })?;

    // 1. Exchange authorization code for access token
    let client = reqwest::Client::new();
    let token_response = client
        .post(DISCORD_TOKEN_URL)
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", &payload.code),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::error!(event = "oauth.discord.token_exchange_failed", error = %e);
            EngineError::InternalError(format!("Failed to exchange Discord code: {}", e))
        })?;

    if !token_response.status().is_success() {
        let status = token_response.status();
        let body = token_response.text().await.unwrap_or_default();
        tracing::error!(
            event = "oauth.discord.token_exchange_rejected",
            status = %status,
            body = %body,
        );
        return Err(EngineError::AuthError(
            "Discord rejected the authorization code — it may have expired. Please try again."
                .to_string(),
        )
        .into());
    }

    let discord_token: DiscordTokenResponse = token_response.json().await.map_err(|e| {
        tracing::error!(event = "oauth.discord.token_parse_failed", error = %e);
        EngineError::InternalError("Failed to parse Discord token response".to_string())
    })?;

    // 2. Fetch Discord user info
    let user_response = client
        .get(DISCORD_USER_URL)
        .bearer_auth(&discord_token.access_token)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(event = "oauth.discord.user_fetch_failed", error = %e);
            EngineError::InternalError(format!("Failed to fetch Discord user info: {}", e))
        })?;

    if !user_response.status().is_success() {
        return Err(EngineError::AuthError(
            "Failed to retrieve Discord user information".to_string(),
        )
        .into());
    }

    let discord_user: DiscordUser = user_response.json().await.map_err(|e| {
        tracing::error!(event = "oauth.discord.user_parse_failed", error = %e);
        EngineError::InternalError("Failed to parse Discord user response".to_string())
    })?;

    // 3. Look up existing OAuth link
    let existing_link = oauth_repo
        .find_by_provider_user_id("discord", &discord_user.id)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;

    let user = if let Some(link) = existing_link {
        // User already linked — look them up
        state
            .user_repository
            .get_user_by_id(link.user_id)
            .await
            .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?
            .ok_or_else(|| {
                EngineError::AuthError(
                    "Linked user account no longer exists. Contact an administrator.".to_string(),
                )
            })?
    } else {
        // No existing link — try to match by email
        let discord_email = discord_user.email.as_deref().ok_or_else(|| {
            EngineError::AuthError(
                "Discord account does not have a verified email. Cannot link to admin account."
                    .to_string(),
            )
        })?;

        let matched_user = state
            .user_repository
            .get_user_by_email(discord_email)
            .await
            .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?
            .ok_or_else(|| {
                tracing::warn!(
                    event = "oauth.discord.no_matching_user",
                    discord_email = %discord_email,
                    discord_id = %discord_user.id,
                );
                EngineError::AuthError(
                    "No admin account found for this Discord email. Contact an administrator."
                        .to_string(),
                )
            })?;

        // Create the OAuth link for future logins
        let _ = oauth_repo
            .create(
                matched_user.id,
                "discord",
                &discord_user.id,
                Some(discord_email),
                Some(&discord_user.username),
            )
            .await
            .map_err(|e| {
                tracing::warn!(event = "oauth.discord.link_creation_failed", error = %e);
                // Non-fatal: login still works, just won't be linked for next time
            });

        matched_user
    };

    // 4. Update last login
    let _ = state.user_repository.update_last_login(user.id).await;

    let provisioned_default_api_key =
        provision_default_api_key_for_oauth_login(&state, &user).await?;

    // 5. Generate JWT token (same as password login)
    let permissions = vec!["basic:access".to_string()];
    let consciousness_level = user.consciousness_level as u8;

    let token = state.auth.generate_jwt_token(
        &user.id.to_string(),
        &user.tier,
        &permissions,
        consciousness_level,
    )?;

    tracing::info!(
        event = "auth.discord_login_success",
        user_id = %user.id,
        email = %user.email,
        discord_id = %discord_user.id,
        provisioned_default_api_key = provisioned_default_api_key,
        "User logged in via Discord"
    );

    let response = LoginResponse {
        token,
        user_id: user.id.to_string(),
        email: user.email,
        tier: user.tier,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[cfg(test)]
mod tests {
    use super::{
        build_default_oauth_api_key_seed, ensure_default_api_key_for_oauth_login,
        resolve_discord_redirect_uri, OAUTH_DEFAULT_API_KEY_PERMISSIONS,
    };
    use axum::http::{HeaderMap, HeaderValue};
    use chrono::Utc;
    use noesis_core::EngineError;
    use noesis_data::models::user::User;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use uuid::Uuid;

    #[test]
    fn accepts_same_origin_preview_callback_uri() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "origin",
            HeaderValue::from_static("https://enantiodromia-engine-dashboard.vercel.app"),
        );

        let resolved = resolve_discord_redirect_uri(
            "https://144.tryambakam.space/admin/login/discord-callback",
            Some("https://enantiodromia-engine-dashboard.vercel.app/admin/auth/discord/callback"),
            &headers,
        )
        .expect("preview callback uri should be accepted");

        assert_eq!(
            resolved,
            "https://enantiodromia-engine-dashboard.vercel.app/admin/auth/discord/callback"
        );
    }

    #[test]
    fn rejects_cross_origin_callback_uri_override() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "origin",
            HeaderValue::from_static("https://enantiodromia-engine-dashboard.vercel.app"),
        );

        let err = resolve_discord_redirect_uri(
            "https://144.tryambakam.space/admin/login/discord-callback",
            Some("https://malicious.example.com/admin/auth/discord/callback"),
            &headers,
        )
        .expect_err("cross-origin callback uri should be rejected");

        assert!(
            err.to_string()
                .contains("Discord redirect URI must stay on the current admin origin"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn oauth_default_api_key_seed_uses_user_defaults() {
        let user = User {
            id: Uuid::new_v4(),
            email: "discord-seed@example.com".to_string(),
            password_hash: "unused".to_string(),
            full_name: "Discord Seed".to_string(),
            tier: "Premium".to_string(),
            consciousness_level: 7,
            experience_points: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            reset_token: None,
            reset_token_expires_at: None,
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
        };

        let seed = build_default_oauth_api_key_seed(&user);

        assert!(seed.secret_key.starts_with("nk_"));
        assert_eq!(seed.key_prefix.len(), 12);
        assert_eq!(seed.key_prefix, seed.secret_key[..12]);
        assert_eq!(seed.tier, "premium");
        assert_eq!(
            seed.permissions,
            OAUTH_DEFAULT_API_KEY_PERMISSIONS
                .iter()
                .map(|permission| (*permission).to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(seed.consciousness_level, 5);
        assert_eq!(seed.rate_limit, 1_000);
        assert!(!seed.key_hash.is_empty());
    }

    #[tokio::test]
    async fn oauth_default_api_key_is_created_when_user_has_no_active_keys() {
        let created = Arc::new(AtomicBool::new(false));
        let created_for_assert = created.clone();

        let result = ensure_default_api_key_for_oauth_login(
            || async { Ok(0) },
            move || {
                let created = created.clone();
                async move {
                    created.store(true, Ordering::SeqCst);
                    Ok(())
                }
            },
        )
        .await
        .expect("oauth key creation should succeed");

        assert!(result);
        assert!(created_for_assert.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn oauth_default_api_key_is_skipped_when_user_already_has_active_key() {
        let created = Arc::new(AtomicBool::new(false));
        let created_for_assert = created.clone();

        let result = ensure_default_api_key_for_oauth_login(
            || async { Ok(1) },
            move || {
                let created = created.clone();
                async move {
                    created.store(true, Ordering::SeqCst);
                    Ok(())
                }
            },
        )
        .await
        .expect("oauth key skip should succeed");

        assert!(!result);
        assert!(!created_for_assert.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn oauth_default_api_key_propagates_count_failures() {
        let result = ensure_default_api_key_for_oauth_login(
            || async { Err(EngineError::InternalError("boom".to_string())) },
            || async { Ok(()) },
        )
        .await;

        assert!(matches!(result, Err(EngineError::InternalError(message)) if message == "boom"));
    }
}
