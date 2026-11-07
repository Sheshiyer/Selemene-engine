//! Shared Witness metadata routes. Private bodies remain in object storage.
use super::shared_identity::{
    active, db, fail, private, storage, subject_access, transaction, HandlerError,
};
use crate::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use noesis_auth::AuthUser;
use noesis_data::{
    models::witness_catalog::WitnessPublish,
    repositories::witness_catalog_repository::{self as repo, WitnessCatalogError},
};
use uuid::Uuid;
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
) -> Result<Response, HandlerError> {
    let pool = db(&s)?;
    let actor = active(pool, &u).await?;
    let mut tx = transaction(pool, actor).await?;
    subject_access(&mut tx, actor, id, false).await?;
    let result = repo::catalog(&mut tx, actor, id).await.map_err(error)?;
    tx.commit().await.map_err(storage)?;
    Ok(private(Json(result).into_response()))
}
async fn publish_result(
    State(s): State<AppState>,
    Extension(u): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(p): Json<WitnessPublish>,
) -> Result<Response, HandlerError> {
    let pool = db(&s)?;
    let actor = active(pool, &u).await?;
    let mut tx = transaction(pool, actor).await?;
    subject_access(&mut tx, actor, id, false).await?;
    // The repository independently requires resource ownership or the dedicated
    // publisher grant; read/edit/calculate/admin names never imply publication.
    let result = repo::publish(&mut tx, actor, id, &p).await.map_err(error)?;
    tx.commit().await.map_err(storage)?;
    Ok(private(Json(result).into_response()))
}

// Apply private headers to denials/conflicts as well as successful reads.
pub async fn catalog(
    state: State<AppState>,
    user: Extension<AuthUser>,
    id: Path<Uuid>,
) -> Response {
    private(match catalog_result(state, user, id).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    })
}
pub async fn publish(
    state: State<AppState>,
    user: Extension<AuthUser>,
    id: Path<Uuid>,
    payload: Json<WitnessPublish>,
) -> Response {
    private(match publish_result(state, user, id, payload).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    })
}
