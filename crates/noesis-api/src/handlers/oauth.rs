use crate::error::ApiError;
use crate::handlers::auth::LoginResponse;
use crate::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use noesis_core::EngineError;
use serde::{Deserialize, Serialize};

const DISCORD_AUTHORIZE_URL: &str = "https://discord.com/api/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
const DISCORD_USER_URL: &str = "https://discord.com/api/users/@me";

#[derive(Serialize)]
pub struct DiscordAuthorizeResponse {
    pub url: String,
}

#[derive(Deserialize)]
pub struct DiscordCallbackRequest {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
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

/// GET /api/v1/auth/discord/authorize — returns the Discord OAuth2 authorize URL
pub async fn discord_authorize(State(state): State<AppState>) -> Result<Response, ApiError> {
    let client_id = state
        .discord_client_id
        .as_ref()
        .ok_or_else(|| EngineError::ConfigError("Discord OAuth not configured".to_string()))?;
    let redirect_uri = state.discord_redirect_uri.as_ref().ok_or_else(|| {
        EngineError::ConfigError("Discord redirect URI not configured".to_string())
    })?;

    // Build a simple state token using a HMAC of a timestamp to prevent CSRF.
    // In production you'd want a nonce stored server-side; this is a lightweight approach.
    let timestamp = chrono::Utc::now().timestamp();
    let state_token = format!("{}", timestamp);

    let url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=identify%20email&state={}",
        DISCORD_AUTHORIZE_URL,
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(&state_token),
    );

    Ok((StatusCode::OK, Json(DiscordAuthorizeResponse { url })).into_response())
}

/// POST /api/v1/auth/discord/callback — exchange Discord auth code for JWT
pub async fn discord_callback(
    State(state): State<AppState>,
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
    let redirect_uri = state.discord_redirect_uri.as_ref().ok_or_else(|| {
        EngineError::ConfigError("Discord redirect URI not configured".to_string())
    })?;

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
