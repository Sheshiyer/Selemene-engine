use crate::{error::ApiError, AppState};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use noesis_auth::AuthUser;
use noesis_core::EngineError;
use serde::Serialize;
use std::collections::BTreeSet;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct AdminSessionResponse {
    pub user_id: String,
    pub email: String,
    pub tier: String,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
    pub has_admin_access: bool,
}

fn has_admin_access(permissions: &[String]) -> bool {
    permissions.iter().any(|p| p.starts_with("admin:"))
        || permissions
            .iter()
            .any(|p| p == "admin:users" || p == "admin:analytics")
}

fn derive_roles(permissions: &[String]) -> Vec<String> {
    let mut roles: BTreeSet<String> = BTreeSet::new();
    let has = |perm: &str| permissions.iter().any(|p| p == perm);

    let viewer_signals = [
        "admin:analytics:read",
        "admin:system:read",
        "admin:audit:list",
        "admin:audit:read",
        "admin:analytics",
    ];
    if viewer_signals.iter().any(|perm| has(perm)) {
        roles.insert("viewer".to_string());
    }

    let support_signals = [
        "admin:users:list",
        "admin:users:read",
        "admin:users:suspend",
        "admin:history-sync:read",
        "admin:users",
    ];
    if support_signals.iter().any(|perm| has(perm)) {
        roles.insert("support".to_string());
    }

    let admin_signals = [
        "admin:keys:list",
        "admin:keys:create",
        "admin:keys:revoke",
        "admin:keys:rotate",
        "admin:users:tier:update",
        "admin:history-sync:retry",
    ];
    if admin_signals.iter().any(|perm| has(perm)) {
        roles.insert("admin".to_string());
    }

    if has("admin:users:roles:update") {
        roles.insert("platform-admin".to_string());
    }

    if roles.is_empty() && has_admin_access(permissions) {
        roles.insert("viewer".to_string());
    }

    roles.into_iter().collect()
}

/// GET /api/v1/admin/session -- return current authenticated admin session shape
#[utoipa::path(
    get,
    path = "/api/v1/admin/session",
    tag = "admin",
    responses(
        (status = 200, description = "Admin session data", body = AdminSessionResponse),
        (status = 401, description = "Unauthorized", body = crate::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::ErrorResponse),
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    )
)]
pub async fn get_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Response, ApiError> {
    let user_uuid = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| EngineError::AuthError("Invalid user ID in token".to_string()))?;

    let user = state
        .user_repository
        .get_user_by_id(user_uuid)
        .await
        .map_err(|e| EngineError::InternalError(format!("Database error: {}", e)))?
        .ok_or_else(|| EngineError::AuthError("User not found".to_string()))?;

    let response = AdminSessionResponse {
        user_id: user.id.to_string(),
        email: user.email,
        tier: auth_user.tier,
        roles: derive_roles(&auth_user.permissions),
        has_admin_access: has_admin_access(&auth_user.permissions),
        permissions: auth_user.permissions,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_platform_admin_role() {
        let permissions = vec![
            "admin:users:roles:update".to_string(),
            "admin:keys:create".to_string(),
        ];
        let roles = derive_roles(&permissions);
        assert!(roles.iter().any(|r| r == "platform-admin"));
        assert!(roles.iter().any(|r| r == "admin"));
    }

    #[test]
    fn derives_legacy_alias_roles() {
        let permissions = vec!["admin:users".to_string(), "admin:analytics".to_string()];
        let roles = derive_roles(&permissions);
        assert!(roles.iter().any(|r| r == "support"));
        assert!(roles.iter().any(|r| r == "viewer"));
    }
}
