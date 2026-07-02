use crate::error::ApiError;
use crate::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use chrono::{Duration, Utc};
use noesis_auth::password::{hash_password, verify_password};
use noesis_auth::AuthUser;
use noesis_core::EngineError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Return 503 if the database is not available (auth requires DB).
fn require_db(state: &AppState) -> Result<(), ApiError> {
    if !state.db_available {
        return Err(EngineError::ServiceUnavailable(
            "Database not available — auth endpoints require a database connection".to_string(),
        )
        .into());
    }
    Ok(())
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub id: String,
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub tier: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResetPasswordResponse {
    pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Serialize, ToSchema)]
pub struct ChangePasswordResponse {
    pub message: String,
}

/// Validate password meets minimum strength requirements.
///
/// Rules:
/// - At least 8 characters
/// - At least 1 uppercase letter
/// - At least 1 lowercase letter
/// - At least 1 digit
fn validate_password_strength(password: &str) -> Result<(), EngineError> {
    if password.len() < 8 {
        return Err(EngineError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(EngineError::ValidationError(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(EngineError::ValidationError(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(EngineError::ValidationError(
            "Password must contain at least one digit".to_string(),
        ));
    }
    Ok(())
}

/// POST /api/v1/auth/register -- create a new user account
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User created successfully", body = RegisterResponse),
        (status = 401, description = "User already exists", body = crate::ErrorResponse),
        (status = 422, description = "Invalid request body", body = crate::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::ErrorResponse),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, ApiError> {
    require_db(&state)?;
    // Check if user exists
    let existing_user = state
        .user_repository
        .get_user_by_email(&payload.email)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error checking user: {}", e)))?;

    if existing_user.is_some() {
        return Err(EngineError::AuthError("User already exists".to_string()).into());
    }

    // Validate password strength
    validate_password_strength(&payload.password)?;

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Create user
    let user = state
        .user_repository
        .create_user(&payload.email, &password_hash, &payload.full_name)
        .await
        .map_err(|e| EngineError::InternalError(format!("Failed to create user: {}", e)))?;

    tracing::info!(event = "auth.register", user_id = %user.id, email = %payload.email, "User registered");

    // Return 201 Created with ID
    let response = RegisterResponse {
        id: user.id.to_string(),
        message: "User created successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

/// POST /api/v1/auth/login -- authenticate and receive a JWT token
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = crate::ErrorResponse),
        (status = 422, description = "Invalid request body", body = crate::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::ErrorResponse),
    )
)]
pub async fn login(
    State(_state): State<AppState>,
    Json(_payload): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    Ok((
        StatusCode::GONE,
        Json(serde_json::json!({
            "error": "Password login has been retired. Use Cloudflare Access.",
            "error_code": "AUTH_RETIRED"
        })),
    )
        .into_response())
}

/// POST /api/v1/auth/forgot-password -- request a password reset token
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    tag = "auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset link sent (if account exists)", body = ForgotPasswordResponse),
        (status = 422, description = "Invalid request body", body = crate::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::ErrorResponse),
    )
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Response, ApiError> {
    require_db(&state)?;
    // Generate token
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);

    // Save to DB
    state
        .user_repository
        .set_password_reset_token(&payload.email, &token, expires_at)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;

    // Log the reset request (never log the token itself — it's a credential)
    tracing::info!(event = "auth.password_reset_requested", email = %payload.email, "Password reset requested");

    let response = ForgotPasswordResponse {
        message: "If an account exists with this email, a password reset link has been sent."
            .to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// POST /api/v1/auth/reset-password -- reset password using a valid token
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    tag = "auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = ResetPasswordResponse),
        (status = 401, description = "Invalid or expired token", body = crate::ErrorResponse),
        (status = 422, description = "Invalid request body", body = crate::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::ErrorResponse),
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Response, ApiError> {
    require_db(&state)?;
    // 1. Verify token
    let user = state
        .user_repository
        .find_user_by_reset_token(&payload.token)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?
        .ok_or_else(|| {
            EngineError::AuthError("Invalid or expired password reset token".to_string())
        })?;

    // 2. Validate new password strength
    validate_password_strength(&payload.new_password)?;

    // 3. Hash new password
    let password_hash = hash_password(&payload.new_password)?;

    // 4. Update password
    state
        .user_repository
        .update_password(user.id, &password_hash)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;

    tracing::info!(event = "auth.password_reset_completed", user_id = %user.id, "Password reset completed");

    let response = ResetPasswordResponse {
        message: "Password has been successfully reset.".to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

/// POST /api/v1/auth/change-password -- change password while authenticated
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    tag = "auth",
    request_body = ChangePasswordRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Password changed successfully", body = ChangePasswordResponse),
        (status = 401, description = "Invalid current password or not authenticated", body = crate::ErrorResponse),
        (status = 422, description = "New password does not meet requirements", body = crate::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::ErrorResponse),
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Response, ApiError> {
    require_db(&state)?;
    // 1. Fetch user from DB
    let user_id = auth_user
        .user_id
        .parse::<Uuid>()
        .map_err(|_| EngineError::AuthError("Invalid user ID in token".to_string()))?;

    let user = state
        .user_repository
        .get_user_by_id(user_id)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?
        .ok_or_else(|| EngineError::AuthError("User not found".to_string()))?;

    // 2. Verify current password
    let valid = verify_password(&payload.current_password, &user.password_hash)?;
    if !valid {
        return Err(EngineError::AuthError("Current password is incorrect".to_string()).into());
    }

    // 3. Validate new password strength
    validate_password_strength(&payload.new_password)?;

    // 4. Hash and update
    let password_hash = hash_password(&payload.new_password)?;
    state
        .user_repository
        .update_password_authenticated(user_id, &password_hash)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;

    tracing::info!(event = "auth.password_changed", user_id = %user_id, "Password changed");

    let response = ChangePasswordResponse {
        message: "Password changed successfully".to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

// ---------------------------------------------------------------------------
// Logout — revoke the current JWT (#697)
// ---------------------------------------------------------------------------

#[derive(Serialize, ToSchema)]
pub struct LogoutResponse {
    pub message: String,
}

/// POST /api/v1/auth/logout — invalidate the caller's JWT immediately.
///
/// The JWT is added to the in-memory revocation list for the remainder of its
/// natural lifetime. API-key-authenticated requests are accepted but are a no-op
/// (API keys are invalidated via DELETE /api/v1/admin/api-keys/:id instead).
pub async fn logout(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    if let (Some(jti), Some(exp)) = (auth_user.jti, auth_user.token_exp) {
        state.auth.revoke_token(&jti, exp);
        tracing::info!(
            event = "auth.logout",
            user_id = %auth_user.user_id,
            jti = %jti,
            "JWT revoked on logout"
        );
    } else {
        tracing::debug!(
            user_id = %auth_user.user_id,
            "Logout called without jti (API key auth or legacy token) — no-op"
        );
    }

    (
        StatusCode::OK,
        Json(LogoutResponse {
            message: "Logged out successfully".to_string(),
        }),
    )
        .into_response()
}
