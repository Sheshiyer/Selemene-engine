use crate::{living_reading_publication::LivingReadingPublicationResolver, AppState, ErrorMapper};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use noesis_auth::sha256_hex;
use noesis_data::models::living_reading::{
    LivingReadingInvitationResolution, LivingReadingPublishedArtifact,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ResolveInvitationQuery {
    token: String,
}

fn unavailable_response() -> Response {
    ErrorMapper::response(
        StatusCode::NOT_FOUND,
        "INVITATION_UNAVAILABLE",
        "Invitation is unavailable",
        None,
    )
    .into_response()
}

/// Public recipient seam. The bearer token is useful only for the reading UUID
/// in this route, and the repository returns a privacy-filtered DTO.
pub async fn resolve_invitation(
    State(state): State<AppState>,
    Path(reading_id): Path<String>,
    Query(query): Query<ResolveInvitationQuery>,
) -> Response {
    let Ok(reading_id) = Uuid::parse_str(&reading_id) else {
        return unavailable_response();
    };
    if query.token.len() != 43
        || !query
            .token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return unavailable_response();
    }
    let Some(admin_repository) = state.admin_repository.as_ref() else {
        return ErrorMapper::response(
            StatusCode::SERVICE_UNAVAILABLE,
            "ARCHIVE_UNAVAILABLE",
            "Archive is temporarily unavailable",
            None,
        )
        .into_response();
    };

    let record = match admin_repository
        .living_readings()
        .resolve_invitation(reading_id, &sha256_hex(&query.token))
        .await
    {
        Ok(Some(resolution)) => resolution,
        Ok(None) => return unavailable_response(),
        Err(error) => {
            tracing::error!(%error, "Failed to resolve living-reading invitation");
            return ErrorMapper::response(
                StatusCode::SERVICE_UNAVAILABLE,
                "ARCHIVE_UNAVAILABLE",
                "Archive is temporarily unavailable",
                None,
            )
            .into_response();
        }
    };

    let verified = match record.artifact.as_ref() {
        Some(candidate) => match LivingReadingPublicationResolver::from_env()
            .and_then(|resolver| resolver.resolve(candidate))
        {
            Ok(verified) => Some(verified),
            Err(error) => {
                tracing::warn!(
                    reason = %error,
                    "Living-reading invitation artifact failed verification"
                );
                None
            }
        },
        None => None,
    };
    let artifact = verified
        .map(|publication| LivingReadingPublishedArtifact {
            availability: "available".to_string(),
            media_type: Some(publication.media_type),
            content: Some(publication.content),
        })
        .unwrap_or_else(|| LivingReadingPublishedArtifact {
            availability: "publication_unavailable".to_string(),
            media_type: None,
            content: None,
        });
    let resolution = LivingReadingInvitationResolution {
        reading_id: record.reading_id,
        stable_reading_id: record.stable_reading_id,
        title: record.title,
        summary: record.summary,
        reading_type: record.reading_type,
        language_tag: record.language_tag,
        editorial_state: record.editorial_state,
        subjects: record.subjects,
        relationship_label: record.relationship_label,
        artifact,
    };

    let mut response = (StatusCode::OK, Json(resolution)).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_tokens_fail_with_the_same_public_error() {
        for token in ["", "short", "contains+unsafe/characters"] {
            assert!(
                token.len() != 43
                    || !token
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
            );
        }
        let response = unavailable_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
