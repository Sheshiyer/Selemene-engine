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

fn analytics_reader_token() -> String {
    AuthService::new(common::TEST_JWT_SECRET.to_string())
        .generate_jwt_token(
            "550e8400-e29b-41d4-a716-446655440001",
            "pro",
            &["admin:analytics:read".to_string()],
            7,
        )
        .expect("analytics reader token")
}

#[tokio::test]
async fn viewer_is_denied_from_every_living_readings_route() {
    let token = viewer_token();
    for uri in [
        "/api/v1/admin/living-readings",
        "/api/v1/admin/living-readings/75c78363-a307-4b4c-a26c-e9cc96470ce7",
    ] {
        let (status, body) =
            common::make_authenticated_request("GET", uri, &token, None).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{uri}: {body}");
        assert_eq!(body["error_code"], "FORBIDDEN", "{uri}: {body}");
        assert_eq!(
            body["details"]["required_permission"],
            "admin:analytics:read",
            "{uri}: {body}"
        );
    }
}

#[tokio::test]
async fn analytics_reader_crosses_authz_for_every_living_readings_route() {
    let token = analytics_reader_token();
    for uri in [
        "/api/v1/admin/living-readings",
        "/api/v1/admin/living-readings/75c78363-a307-4b4c-a26c-e9cc96470ce7",
    ] {
        let (status, body) =
            common::make_authenticated_request("GET", uri, &token, None).await;
        assert_ne!(status, StatusCode::FORBIDDEN, "{uri}: {body}");
        assert_ne!(
            body["error_code"], "FORBIDDEN",
            "{uri}: analytics readers must reach the repository boundary"
        );
    }
}
