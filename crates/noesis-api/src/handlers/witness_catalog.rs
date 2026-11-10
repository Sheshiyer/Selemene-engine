//! Shared Witness metadata routes. Private bodies remain in object storage.
use super::shared_identity::{
    active, db, fail, private, storage, subject_access, transaction, HandlerError,
};
use super::witness_requests::WorkerAccess;
use crate::{middleware::ApiKeyPrincipal, AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use noesis_auth::AuthUser;
use noesis_data::{
    models::witness_catalog::{WitnessArtifact, WitnessPublish},
    repositories::witness_catalog_repository::{self as repo, WitnessCatalogError},
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishInput {
    expected_head: serde_json::Value,
    artifact: WitnessArtifact,
    approval: String,
    request_id: Option<Uuid>,
    lease_token: Option<String>,
}
fn error(e: WitnessCatalogError) -> HandlerError {
    match e {
        WitnessCatalogError::Denied => fail(StatusCode::NOT_FOUND, "SUBJECT_NOT_FOUND"),
        WitnessCatalogError::Invalid(code) => fail(StatusCode::BAD_REQUEST, code),
        WitnessCatalogError::Conflict(code) => fail(StatusCode::CONFLICT, code),
        WitnessCatalogError::Storage(e) => storage(e),
    }
}
async fn catalog_result(
    State(s): State<AppState>,
    Extension(u): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    worker: Option<Extension<WorkerAccess>>,
) -> Result<Response, HandlerError> {
    let pool = db(&s)?;
    let actor = active(pool, &u).await?;
    let mut tx = transaction(pool, actor).await?;
    subject_access(&mut tx, actor, id, false).await?;
    if let Some(Extension(scope)) = worker {
        scope.recheck(&mut tx, actor, id).await?;
    }
    let result = repo::catalog(&mut tx, actor, id).await.map_err(error)?;
    tx.commit().await.map_err(storage)?;
    Ok(private(Json(result).into_response()))
}
async fn publish_result(
    State(s): State<AppState>,
    Extension(u): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    key: Option<Extension<ApiKeyPrincipal>>,
    Json(input): Json<PublishInput>,
) -> Result<Response, HandlerError> {
    let pool = db(&s)?;
    let actor = active(pool, &u).await?;
    let mut tx = transaction(pool, actor).await?;
    subject_access(&mut tx, actor, id, false).await?;
    let p = WitnessPublish {
        expected_head: input.expected_head,
        artifact: input.artifact,
        approval: input.approval,
    };
    let worker = key.is_some() && super::witness_requests::worker_permission(&u);
    let request = match (worker, input.request_id, input.lease_token.as_deref()) {
        (false, None, None) => None,
        (true, Some(request), Some(token)) => {
            super::witness_requests::check_publish(&mut tx, actor, id, request, token, &p).await?;
            Some(request)
        }
        _ => {
            return Err(fail(
                StatusCode::FORBIDDEN,
                "WITNESS_REQUEST_PUBLICATION_DENIED",
            ))
        }
    };
    // The repository independently requires resource ownership or the dedicated
    // publisher grant; read/edit/calculate/admin names never imply publication.
    let result = repo::publish(&mut tx, actor, id, &p).await.map_err(error)?;
    if let Some(request) = request {
        super::witness_requests::record_accepted(&mut tx, request, &result.entry.sha256).await?;
    }
    tx.commit().await.map_err(storage)?;
    Ok(private(Json(result).into_response()))
}

// Apply private headers to denials/conflicts as well as successful reads.
pub async fn catalog(
    state: State<AppState>,
    user: Extension<AuthUser>,
    id: Path<Uuid>,
    worker: Option<Extension<WorkerAccess>>,
) -> Response {
    private(match catalog_result(state, user, id, worker).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    })
}
pub async fn publish(
    state: State<AppState>,
    user: Extension<AuthUser>,
    id: Path<Uuid>,
    key: Option<Extension<ApiKeyPrincipal>>,
    payload: Json<PublishInput>,
) -> Response {
    private(match publish_result(state, user, id, key, payload).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    })
}
