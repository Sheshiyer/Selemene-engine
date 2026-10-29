//! Browser-based desktop sign-in endpoints. See `crate::desktop_auth` for the
//! flow and the security model.

use crate::desktop_auth::{
    hash_secret, new_secret, pkce_matches, valid_code_challenge, valid_state,
    validate_loopback_redirect, AUTH_CODE_TTL_SECS, DESKTOP_PERMISSIONS, REFRESH_TOKEN_TTL_DAYS,
};
use crate::error_mapper::{ErrorMapper, ErrorResponse};
use crate::middleware::InteractivePrincipal;
use crate::AppState;
use axum::{
    extract::{Json, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use chrono::{Duration, Utc};
use noesis_auth::{AuthUser, JWT_LIFETIME_HOURS};
use noesis_data::repositories::desktop_session_repository::{
    DesktopGrant, DesktopSessionRepository, RefreshOutcome,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

type HandlerError = (StatusCode, Json<ErrorResponse>);

fn error(status: StatusCode, code: &str, message: &str) -> HandlerError {
    ErrorMapper::response(status, code, message, None)
}

fn invalid_grant() -> HandlerError {
    error(
        StatusCode::BAD_REQUEST,
        "INVALID_GRANT",
        "The code or refresh token is invalid, expired, or already used",
    )
}

fn repository(state: &AppState) -> Result<DesktopSessionRepository, HandlerError> {
    state
        .auth
        .pool()
        .filter(|_| state.db_available)
        .map(|pool| DesktopSessionRepository::new(pool.clone()))
        .ok_or_else(|| {
            error(
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                "Desktop sign-in requires a database connection",
            )
        })
}

fn storage_failure(e: sqlx::Error) -> HandlerError {
    tracing::error!(error = %e, "desktop session storage failed");
    error(
        StatusCode::SERVICE_UNAVAILABLE,
        "SERVICE_UNAVAILABLE",
        "Desktop sign-in storage is unavailable",
    )
}

fn no_store(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct AuthorizeParams {
    /// base64url SHA-256 of the client's PKCE verifier.
    pub code_challenge: String,
    /// Must be `S256` when present.
    pub code_challenge_method: Option<String>,
    /// `http://127.0.0.1:<port>/callback`.
    pub redirect_uri: String,
    /// Opaque client value echoed on the redirect.
    pub state: String,
}

/// GET /api/v1/desktop/authorize -- start desktop sign-in from the browser
#[utoipa::path(
    get,
    path = "/api/v1/desktop/authorize",
    tag = "auth",
    params(AuthorizeParams),
    responses(
        (status = 303, description = "Redirect to the desktop app's loopback callback with a single-use code"),
        (status = 400, description = "Invalid PKCE challenge, state, or redirect", body = ErrorResponse),
        (status = 403, description = "Not a browser (Cloudflare Access) identity", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    )
)]
pub async fn authorize(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    interactive: Option<Extension<InteractivePrincipal>>,
    Query(params): Query<AuthorizeParams>,
) -> Result<Response, HandlerError> {
    if interactive.is_none() {
        return Err(error(
            StatusCode::FORBIDDEN,
            "DESKTOP_AUTH_REQUIRES_BROWSER",
            "Desktop sign-in must start from a browser session",
        ));
    }
    if params
        .code_challenge_method
        .as_deref()
        .is_some_and(|m| m != "S256")
    {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            "code_challenge_method must be S256",
        ));
    }
    if !valid_code_challenge(&params.code_challenge) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            "code_challenge is malformed",
        ));
    }
    if !valid_state(&params.state) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            "state is malformed",
        ));
    }
    if let Err(reason) = validate_loopback_redirect(&params.redirect_uri) {
        return Err(error(StatusCode::BAD_REQUEST, "INVALID_REQUEST", reason));
    }
    let user_id = Uuid::parse_str(&user.user_id).map_err(|_| {
        error(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "Identity cannot hold a desktop session",
        )
    })?;

    let repo = repository(&state)?;
    let grant = DesktopGrant {
        user_id,
        tier: user.tier.clone(),
        consciousness_level: i16::from(user.consciousness_level),
    };
    let code = new_secret();
    repo.insert_auth_code(
        &hash_secret(&code),
        &grant,
        &params.code_challenge,
        &params.redirect_uri,
        Utc::now() + Duration::seconds(AUTH_CODE_TTL_SECS),
    )
    .await
    .map_err(storage_failure)?;

    tracing::info!(event = "desktop.authorize", user_id = %user_id, "desktop sign-in code issued");
    // `code` and `state` are base64url / RFC 3986 unreserved, so no escaping is needed.
    let location = format!(
        "{}?code={}&state={}",
        params.redirect_uri, code, params.state
    );
    Ok(no_store(Redirect::to(&location).into_response()))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DesktopTokenRequest {
    /// `authorization_code` or `refresh_token`.
    pub grant_type: String,
    pub code: Option<String>,
    pub code_verifier: Option<String>,
    pub redirect_uri: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DesktopTokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_expires_in: i64,
}

async fn issue_tokens(
    state: &AppState,
    repo: &DesktopSessionRepository,
    grant: &DesktopGrant,
    family_id: Uuid,
) -> Result<Response, HandlerError> {
    let permissions: Vec<String> = DESKTOP_PERMISSIONS.iter().map(|p| p.to_string()).collect();
    let level = grant.consciousness_level.clamp(0, 5) as u8;
    let access_token = state
        .auth
        .generate_jwt_token(&grant.user_id.to_string(), &grant.tier, &permissions, level)
        .map_err(|e| {
            tracing::error!(error = %e, "desktop access token generation failed");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Could not issue an access token",
            )
        })?;

    let refresh_token = new_secret();
    repo.insert_refresh_token(
        &hash_secret(&refresh_token),
        family_id,
        grant,
        Utc::now() + Duration::days(REFRESH_TOKEN_TTL_DAYS),
    )
    .await
    .map_err(storage_failure)?;

    let body = DesktopTokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: JWT_LIFETIME_HOURS * 3600,
        refresh_token,
        refresh_expires_in: REFRESH_TOKEN_TTL_DAYS * 86_400,
    };
    Ok(no_store((StatusCode::OK, Json(body)).into_response()))
}

/// POST /api/v1/desktop/token -- exchange a code or refresh token for a session
#[utoipa::path(
    post,
    path = "/api/v1/desktop/token",
    tag = "auth",
    request_body = DesktopTokenRequest,
    responses(
        (status = 200, description = "Access and refresh tokens", body = DesktopTokenResponse),
        (status = 400, description = "Invalid request or grant", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    )
)]
pub async fn token(
    State(state): State<AppState>,
    Json(request): Json<DesktopTokenRequest>,
) -> Result<Response, HandlerError> {
    let repo = repository(&state)?;
    match request.grant_type.as_str() {
        "authorization_code" => {
            let (Some(code), Some(verifier), Some(redirect_uri)) =
                (request.code, request.code_verifier, request.redirect_uri)
            else {
                return Err(error(
                    StatusCode::BAD_REQUEST,
                    "INVALID_REQUEST",
                    "code, code_verifier and redirect_uri are required",
                ));
            };
            let consumed = repo
                .consume_auth_code(&hash_secret(&code))
                .await
                .map_err(storage_failure)?
                .ok_or_else(invalid_grant)?;
            if consumed.redirect_uri != redirect_uri
                || !pkce_matches(&verifier, &consumed.code_challenge)
            {
                tracing::warn!(event = "desktop.token.pkce_mismatch", user_id = %consumed.grant.user_id);
                return Err(invalid_grant());
            }
            tracing::info!(event = "desktop.token.issued", user_id = %consumed.grant.user_id);
            issue_tokens(&state, &repo, &consumed.grant, Uuid::new_v4()).await
        }
        "refresh_token" => {
            let refresh_token = request.refresh_token.ok_or_else(|| {
                error(
                    StatusCode::BAD_REQUEST,
                    "INVALID_REQUEST",
                    "refresh_token is required",
                )
            })?;
            match repo
                .rotate_refresh_token(&hash_secret(&refresh_token))
                .await
                .map_err(storage_failure)?
            {
                RefreshOutcome::Rotated { grant, family_id } => {
                    issue_tokens(&state, &repo, &grant, family_id).await
                }
                RefreshOutcome::Reused => {
                    tracing::warn!(
                        event = "desktop.refresh.reuse",
                        "refresh token reuse; family revoked"
                    );
                    Err(invalid_grant())
                }
                RefreshOutcome::Invalid => Err(invalid_grant()),
            }
        }
        _ => Err(error(
            StatusCode::BAD_REQUEST,
            "UNSUPPORTED_GRANT_TYPE",
            "grant_type must be authorization_code or refresh_token",
        )),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DesktopRevokeRequest {
    pub refresh_token: String,
}

/// POST /api/v1/desktop/revoke -- end a desktop session (RFC 7009 semantics)
#[utoipa::path(
    post,
    path = "/api/v1/desktop/revoke",
    tag = "auth",
    request_body = DesktopRevokeRequest,
    responses(
        (status = 200, description = "Revoked, or the token was already unknown"),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    )
)]
pub async fn revoke(
    State(state): State<AppState>,
    Json(request): Json<DesktopRevokeRequest>,
) -> Result<Response, HandlerError> {
    let repo = repository(&state)?;
    let revoked = repo
        .revoke_family_of(&hash_secret(&request.refresh_token))
        .await
        .map_err(storage_failure)?;
    if revoked {
        tracing::info!(event = "desktop.revoke", "desktop session revoked");
    }
    // Same response either way, so the endpoint can't be used to probe tokens.
    Ok(no_store(StatusCode::OK.into_response()))
}
