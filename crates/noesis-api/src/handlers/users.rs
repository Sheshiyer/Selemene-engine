use axum::{
    extract::{State, Json, Extension},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::{AppState, error::ApiError};
use noesis_core::EngineError;
use noesis_auth::AuthUser;
use chrono::{NaiveDate, NaiveTime};

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub tier: String,
    pub consciousness_level: i32,
    pub birth_date: Option<NaiveDate>,
    pub birth_time: Option<NaiveTime>,
    pub birth_location: Option<LocationResponse>,
    pub timezone: Option<String>,
    pub preferences: serde_json::Value,
}

#[derive(Serialize)]
pub struct LocationResponse {
    pub lat: f64,
    pub lng: f64,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub birth_time: Option<NaiveTime>,
    pub birth_location_lat: Option<f64>,
    pub birth_location_lng: Option<f64>,
    pub birth_location_name: Option<String>,
    pub timezone: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

pub async fn get_me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    // Fetch user
    let user_uuid = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| EngineError::AuthError("Invalid user ID in token".to_string()))?;

    let user = state.user_repository.get_user_by_id(user_uuid).await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?
        .ok_or_else(|| EngineError::AuthError("User not found".to_string()))?;

    // Fetch profile
    let profile = state.user_repository.get_profile(user_uuid).await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;

    // Construct response
    let (birth_date, birth_time, birth_location, timezone, preferences) = if let Some(p) = profile {
        (
            p.birth_date,
            p.birth_time,
            if let (Some(lat), Some(lng)) = (p.birth_location_lat, p.birth_location_lng) {
                Some(LocationResponse { lat, lng, name: p.birth_location_name })
            } else { None },
            p.timezone,
            p.preferences
        )
    } else {
        (None, None, None, None, serde_json::json!({}))
    };

    let response = UserResponse {
        id: user.id.to_string(),
        email: user.email,
        full_name: user.full_name,
        tier: user.tier,
        consciousness_level: user.consciousness_level,
        birth_date,
        birth_time,
        birth_location,
        timezone,
        preferences,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

impl UpdateUserRequest {
    fn validate(&self) -> Result<(), EngineError> {
        if let Some(email) = &self.email {
            if !email.contains('@') || !email.contains('.') {
                return Err(EngineError::ValidationError("Invalid email format".into()));
            }
        }
        if let Some(lat) = self.birth_location_lat {
            if lat < -90.0 || lat > 90.0 {
                return Err(EngineError::ValidationError("Latitude must be between -90 and 90".into()));
            }
        }
        if let Some(lng) = self.birth_location_lng {
            if lng < -180.0 || lng > 180.0 {
                return Err(EngineError::ValidationError("Longitude must be between -180 and 180".into()));
            }
        }
        if let Some(tz) = &self.timezone {
            if tz.trim().is_empty() {
                return Err(EngineError::ValidationError("Timezone cannot be empty".into()));
            }
        }
        Ok(())
    }
}

pub async fn update_me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Response, ApiError> {
    payload.validate()?;

     let user_uuid = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| EngineError::AuthError("Invalid user ID in token".to_string()))?;

    // Update User table fields
    if payload.full_name.is_some() || payload.email.is_some() {
        state.user_repository.update_user(user_uuid, payload.full_name, payload.email).await
            .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;
    }

    // Update Profile table fields
    if payload.birth_date.is_some() || payload.birth_time.is_some() || 
       payload.birth_location_lat.is_some() || payload.birth_location_lng.is_some() || 
       payload.birth_location_name.is_some() || payload.timezone.is_some() || 
       payload.preferences.is_some() {
        
        state.user_repository.update_profile(
            user_uuid,
            payload.birth_date,
            payload.birth_time,
            payload.birth_location_lat,
            payload.birth_location_lng,
            payload.birth_location_name,
            payload.timezone,
            payload.preferences,
        ).await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?;
    }

    Ok((StatusCode::OK, Json(serde_json::json!({"message": "Profile updated successfully"}))).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_profile_validation() {
        let req = UpdateUserRequest {
            full_name: None,
            email: Some("valid@example.com".to_string()),
            birth_date: None,
            birth_time: None,
            birth_location_lat: Some(45.0),
            birth_location_lng: Some(90.0),
            birth_location_name: None,
            timezone: Some("UTC".to_string()),
            preferences: None,
        };
        assert!(req.validate().is_ok());

        let bad_email = UpdateUserRequest {
            email: Some("invalid-email".to_string()),
            ..req_base()
        };
        assert!(bad_email.validate().is_err());

        let bad_lat = UpdateUserRequest {
            birth_location_lat: Some(91.0),
            ..req_base()
        };
        assert!(bad_lat.validate().is_err());

        let bad_lng = UpdateUserRequest {
            birth_location_lng: Some(-181.0),
            ..req_base()
        };
        assert!(bad_lng.validate().is_err());
        
        let empty_tz = UpdateUserRequest {
            timezone: Some("   ".to_string()),
            ..req_base()
        };
        assert!(empty_tz.validate().is_err());
    }

    fn req_base() -> UpdateUserRequest {
        UpdateUserRequest {
            full_name: None,
            email: None,
            birth_date: None,
            birth_time: None,
            birth_location_lat: None,
            birth_location_lng: None,
            birth_location_name: None,
            timezone: None,
            preferences: None,
        }
    }
}
