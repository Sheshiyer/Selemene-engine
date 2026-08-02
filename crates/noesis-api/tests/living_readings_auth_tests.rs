//! Authorization boundary tests for the read-only living-readings archive.

use axum::http::StatusCode;
use noesis_auth::AuthService;

mod common;

fn viewer_token() -> String {
    AuthService::new(common::TEST_JWT_SECRET.to_string())
        .generate_jwt_token(
            "550e8400-e29b-41d4-a716-446655440000",
            "free",
            &["basic:access".to_string()],
            1,
        )
        .expect("viewer token")
}

#[tokio::test]
async fn viewer_is_denied_from_every_living_readings_route() {
    let token = viewer_token();
    for uri in [
        "/api/v1/admin/living-readings",
        "/api/v1/admin/living-readings/75c78363-a307-4b4c-a26c-e9cc96470ce7",
    ] {
        let (status, body) = common::make_authenticated_request("GET", uri, &token, None).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{uri}: {body}");
        assert_eq!(body["error_code"], "FORBIDDEN", "{uri}: {body}");
        assert_eq!(
            body["details"]["required_permission"], "admin:analytics:read",
            "{uri}: {body}"
        );
    }

    for (method, uri, body) in [
        (
            "GET",
            "/api/v1/admin/living-readings/75c78363-a307-4b4c-a26c-e9cc96470ce7/invitations",
            None,
        ),
        (
            "POST",
            "/api/v1/admin/living-readings/75c78363-a307-4b4c-a26c-e9cc96470ce7/invitations",
            Some(serde_json::json!({ "expires_in_hours": 24 })),
        ),
        (
            "POST",
            "/api/v1/admin/living-readings/75c78363-a307-4b4c-a26c-e9cc96470ce7/invitations/257516fd-9a28-4f1e-a67b-f3cdfcc0fe45/revoke",
            None,
        ),
    ] {
        let (status, response) =
            common::make_authenticated_request(method, uri, &token, body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{uri}: {response}");
    }
}

#[tokio::test]
async fn malformed_public_invitation_fails_closed_without_authentication() {
    let (status, body) = common::make_unauthenticated_request(
        "GET",
        "/api/v1/living-readings/75c78363-a307-4b4c-a26c-e9cc96470ce7/invitation?token=bad",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error_code"], "INVITATION_UNAVAILABLE");
}
