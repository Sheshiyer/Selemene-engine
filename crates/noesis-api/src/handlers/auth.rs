use axum::{
    extract::{State, Json},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use noesis_auth::password::hash_password;
use crate::AppState;
use noesis_core::EngineError;

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

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, EngineError> {
    // Check if user exists
    let existing_user = state.user_repository.get_user_by_email(&payload.email).await
        .map_err(|e| EngineError::DatabaseError(format!("Database error checking user: {}", e)))?;

    if existing_user.is_some() {
        return Err(EngineError::AuthError("User already exists".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Create user
    let user = state.user_repository.create_user(
        &payload.email,
        &password_hash,
        &payload.full_name
    ).await.map_err(|e| EngineError::DatabaseError(format!("Failed to create user: {}", e)))?;

    // Return 201 Created with ID
    let response = RegisterResponse {
        id: user.id.to_string(),
        message: "User created successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)).into_response())
}
