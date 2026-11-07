//! Registered clients deliberately do not change the native desktop contract.
use super::shared_identity::{active, db, fail, private, storage, HandlerError};
use crate::{cf_access::CfIdentity, desktop_auth::*, AppState};
use axum::{extract::{State, Query, Json}, Extension, response::{Response, IntoResponse, Redirect}, http::StatusCode};
use noesis_auth::AuthUser;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Exact redirects are deployment configuration, never caller-controlled patterns.
/// Antahkarana is the sole dynamic-port exception and uses the native validator.
pub fn redirect_allowed(client: &str, redirect: &str) -> bool {
    if client == "antahkarana" { return validate_loopback_redirect(redirect).is_ok(); }
    let key = match client { "noesis-daily" => "NOESIS_DAILY_REDIRECT_URIS", "noesis-raycast" => "NOESIS_RAYCAST_REDIRECT_URIS", _ => return false };
    let Ok(entries) = std::env::var(key) else { return false; };
    let Ok(url) = reqwest::Url::parse(redirect) else { return false; };
    url.scheme()=="https" && url.username().is_empty() && url.password().is_none() && url.fragment().is_none()
        && entries.split(',').any(|entry| entry.trim()==redirect)
        && (client!="noesis-daily" || (url.path()=="/auth/callback" && url.query().is_none()))
}
#[derive(Deserialize)]
pub struct Authorize { client_id: String, redirect_uri: String, state: String, code_challenge: String, code_challenge_method: String }
pub async fn authorize(State(state): State<AppState>, Extension(user): Extension<AuthUser>, identity: Option<Extension<CfIdentity>>, Query(p): Query<Authorize>) -> Result<Response,HandlerError> {
    let Some(Extension(identity)) = identity else { return Err(fail(StatusCode::FORBIDDEN,"VERIFIED_IDENTITY_REQUIRED")); };
    if p.code_challenge_method!="S256" || !valid_code_challenge(&p.code_challenge) || !valid_state(&p.state) || !redirect_allowed(&p.client_id,&p.redirect_uri) { return Err(fail(StatusCode::BAD_REQUEST,"INVALID_REQUEST")); }
    let pool=db(&state)?; let uid=active(pool,&user).await?; let code=new_secret();
    sqlx::query("INSERT INTO registered_auth_codes(code_hash,client_id,user_id,code_challenge,redirect_uri,expires_at,identity_issuer,identity_subject) VALUES($1,$2,$3,$4,$5,now()+interval '60 seconds',$6,$7)")
        .bind(hash_secret(&code)).bind(&p.client_id).bind(uid).bind(&p.code_challenge).bind(&p.redirect_uri).bind(&identity.issuer).bind(&identity.sub).execute(pool).await.map_err(storage)?;
    let mut location=reqwest::Url::parse(&p.redirect_uri).map_err(|_|fail(StatusCode::BAD_REQUEST,"INVALID_REQUEST"))?;
    location.query_pairs_mut().append_pair("code",&code).append_pair("state",&p.state);
    Ok(private(Redirect::to(location.as_str()).into_response()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Token { grant_type: String, client_id: String, code: Option<String>, redirect_uri: Option<String>, code_verifier: Option<String>, refresh_token: Option<String> }
pub async fn token(State(state): State<AppState>, Json(p): Json<Token>) -> Result<Response,HandlerError> {
    let pool=db(&state)?; let mut tx=pool.begin().await.map_err(storage)?;
    let invalid=||fail(StatusCode::BAD_REQUEST,"INVALID_GRANT");
    let (family,uid)=match p.grant_type.as_str() {
        "authorization_code" => {
            let (Some(code),Some(redirect),Some(verifier))=(p.code,p.redirect_uri,p.code_verifier) else { return Err(invalid()); };
            if !redirect_allowed(&p.client_id,&redirect) { return Err(invalid()); }
            let row=sqlx::query_as::<_,(Uuid,String,String,String,String)>("UPDATE registered_auth_codes SET consumed_at=now() WHERE code_hash=$1 AND client_id=$2 AND consumed_at IS NULL AND expires_at>now() RETURNING user_id,code_challenge,redirect_uri,identity_issuer,identity_subject")
                .bind(hash_secret(&code)).bind(&p.client_id).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(invalid)?;
            if row.2!=redirect || !pkce_matches(&verifier,&row.1) { tx.commit().await.map_err(storage)?; return Err(invalid()); }
            let family=Uuid::new_v4();
            sqlx::query("INSERT INTO registered_session_families(id,client_id,user_id,identity_issuer,identity_subject,expires_at) VALUES($1,$2,$3,$4,$5,now()+interval '30 days')")
                .bind(family).bind(&p.client_id).bind(row.0).bind(row.3).bind(row.4).execute(&mut *tx).await.map_err(storage)?;
            (family,row.0)
        },
        "refresh_token" => {
            let secret=p.refresh_token.ok_or_else(invalid)?;
            // Serialize rotation, reuse and revoke on the FAMILY, including successor insertion.
            let row=sqlx::query_as::<_,(Uuid,Uuid,bool)>("SELECT f.id,f.user_id,t.rotated_at IS NOT NULL FROM registered_session_families f JOIN registered_refresh_tokens t ON t.family_id=f.id WHERE t.token_hash=$1 AND f.client_id=$2 AND f.revoked_at IS NULL AND f.expires_at>now() FOR UPDATE OF f")
                .bind(hash_secret(&secret)).bind(&p.client_id).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(invalid)?;
            // Re-read under the family lock after a competing rotation commits.
            let rotated:bool=sqlx::query_scalar("SELECT rotated_at IS NOT NULL FROM registered_refresh_tokens WHERE token_hash=$1")
                .bind(hash_secret(&secret)).fetch_one(&mut *tx).await.map_err(storage)?;
            if rotated {
                sqlx::query("UPDATE registered_session_families SET revoked_at=now() WHERE id=$1").bind(row.0).execute(&mut *tx).await.map_err(storage)?;
                tx.commit().await.map_err(storage)?; return Err(invalid());
            }
            sqlx::query("UPDATE registered_refresh_tokens SET rotated_at=now() WHERE token_hash=$1").bind(hash_secret(&secret)).execute(&mut *tx).await.map_err(storage)?;
            (row.0,row.1)
        }, _=>return Err(fail(StatusCode::BAD_REQUEST,"UNSUPPORTED_GRANT_TYPE"))
    };
    let account=sqlx::query_as::<_,(String,i32)>("SELECT u.tier,u.consciousness_level FROM users u JOIN user_account_state a ON a.user_id=u.id JOIN registered_session_families f ON f.user_id=u.id JOIN account_identities i ON i.issuer=f.identity_issuer AND i.provider_subject=f.identity_subject AND i.user_id=u.id WHERE f.id=$1 AND u.id=$2 AND a.state='active' AND i.revoked_at IS NULL AND f.revoked_at IS NULL")
        .bind(family).bind(uid).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(invalid)?;
    let access=state.auth.generate_jwt_token(&uid.to_string(),&account.0,&["basic:access".into(),"session:registered".into()],account.1.clamp(0,5) as u8).map_err(|_|fail(StatusCode::INTERNAL_SERVER_ERROR,"TOKEN_FAILED"))?;
    let principal=state.auth.validate_jwt_token(&access).await.map_err(|_|fail(StatusCode::INTERNAL_SERVER_ERROR,"TOKEN_FAILED"))?;
    let refresh=new_secret();
    sqlx::query("INSERT INTO registered_refresh_tokens(token_hash,family_id) VALUES($1,$2)").bind(hash_secret(&refresh)).bind(family).execute(&mut *tx).await.map_err(storage)?;
    sqlx::query("INSERT INTO registered_access_tokens(jti,family_id,expires_at) VALUES($1,$2,now()+interval '1 hour')").bind(principal.jti).bind(family).execute(&mut *tx).await.map_err(storage)?;
    tx.commit().await.map_err(storage)?;
    Ok(private(Json(json!({"access_token":access,"refresh_token":refresh,"token_type":"Bearer","expires_in":3600})).into_response()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revoke { client_id:String, refresh_token:String }
pub async fn revoke(State(state):State<AppState>,Json(p):Json<Revoke>)->Result<Response,HandlerError>{
    sqlx::query("UPDATE registered_session_families SET revoked_at=now() WHERE client_id=$1 AND id=(SELECT family_id FROM registered_refresh_tokens WHERE token_hash=$2)")
        .bind(p.client_id).bind(hash_secret(&p.refresh_token)).execute(db(&state)?).await.map_err(storage)?;
    Ok(private(StatusCode::OK.into_response()))
}
/// Called for every registered bearer request; account, provider link and family revocation are immediate.
pub async fn validate_access(state:&AppState,user:&AuthUser)->Result<(),HandlerError>{
    if !user.permissions.iter().any(|p|p=="session:registered") { return Ok(()); }
    let live:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM registered_access_tokens t JOIN registered_session_families f ON f.id=t.family_id JOIN account_identities i ON i.issuer=f.identity_issuer AND i.provider_subject=f.identity_subject AND i.user_id=f.user_id JOIN user_account_state a ON a.user_id=f.user_id WHERE t.jti=$1 AND f.user_id::text=$2 AND t.expires_at>now() AND f.expires_at>now() AND f.revoked_at IS NULL AND i.revoked_at IS NULL AND a.state='active')")
        .bind(&user.jti).bind(&user.user_id).fetch_one(db(state)?).await.map_err(storage)?;
    if !live {return Err(fail(StatusCode::UNAUTHORIZED,"SESSION_REVOKED"));} Ok(())
}
