use axum::{
    extract::{State, Json},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use noesis_auth::password::{hash_password, verify_password};
use crate::AppState;
use crate::error::ApiError;
use noesis_core::EngineError;
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::info;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub id: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub tier: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, ApiError> {
    // Check if user exists
    let existing_user = state.user_repository.get_user_by_email(&payload.email).await
        .map_err(|e| EngineError::InternalError(format!("Database error checking user: {}", e)))?;

    if existing_user.is_some() {
        return Err(EngineError::AuthError("User already exists".to_string()).into());
    }

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Create user
    let user = state.user_repository.create_user(
        &payload.email,
        &password_hash,
        &payload.full_name
    ).await.map_err(|e| EngineError::InternalError(format!("Failed to create user: {}", e)))?;

    // Return 201 Created with ID
    let response = RegisterResponse {
        id: user.id.to_string(),
        message: "User created successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    // 1. Get user by email
    let user = state.user_repository.get_user_by_email(&payload.email).await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?
        .ok_or_else(|| EngineError::AuthError("Invalid email or password".to_string()))?;

    // 2. Verify password
    let valid_password = verify_password(&payload.password, &user.password_hash)?;
    
    if !valid_password {
        return Err(EngineError::AuthError("Invalid email or password".to_string()).into());
    }

    // 3. Generate JWT token
    // We'll give them standard permissions based on tier for now
    let permissions = vec!["basic:access".to_string()];
    let consciousness_level = user.consciousness_level as u8;
    
    let token = state.auth.generate_jwt_token(
        &user.id.to_string(),
        &user.tier,
        &permissions,
        consciousness_level
    )?;

    // 4. Return token
    let response = LoginResponse {
        token,
        user_id: user.id.to_string(),
        email: user.email,
        tier: user.tier,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Response, ApiError> {
    // Generate token
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);

    // Save to DB
    state.user_repository.set_password_reset_token(&payload.email, &token, expires_at).await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;

    // Log the token for development
    info!("Password reset requested for {}. Token: {}", payload.email, token);

    let response = ForgotPasswordResponse {
        message: "If an account exists with this email, a password reset link has been sent.".to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Response, ApiError> {
    // 1. Verify token
    let user = state.user_repository.get_user_by_reset_token(&payload.token).await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?
        .ok_or_else(|| EngineError::AuthError("Invalid or expired password reset token".to_string()))?;

    // 2. Hash new password
    let password_hash = hash_password(&payload.new_password)?;

    // 3. Update password
    state.user_repository.update_password(user.id, &password_hash).await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;

    let response = ResetPasswordResponse {
        message: "Password has been successfully reset.".to_string(),
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}
