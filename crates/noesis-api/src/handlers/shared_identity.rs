//! Account-scoped readers. Roles and corpus ownership alone never confer reader grants.
use crate::{AppState,error_mapper::{ErrorMapper,ErrorResponse},cf_access::CfIdentity,desktop_auth::{hash_secret,new_secret}};
use axum::{extract::{State,Path,Json},Extension,http::{StatusCode,header,HeaderValue},response::{Response,IntoResponse}};
use noesis_auth::AuthUser;
use serde::Deserialize;
use serde_json::{Value,json};
use sqlx::{PgPool,Postgres,Transaction};
use uuid::Uuid;
pub type HandlerError=(StatusCode,Json<ErrorResponse>);
pub fn fail(status:StatusCode,code:&str)->HandlerError { ErrorMapper::response(status,code,code,None) }
pub fn storage(e:sqlx::Error)->HandlerError { tracing::error!(error=%e,"shared identity storage"); fail(StatusCode::SERVICE_UNAVAILABLE,"IDENTITY_STORAGE_UNAVAILABLE") }
pub fn private(mut r:Response)->Response { r.headers_mut().insert(header::CACHE_CONTROL,HeaderValue::from_static("private, no-store")); r.headers_mut().insert(header::REFERRER_POLICY,HeaderValue::from_static("no-referrer")); r }
pub fn db(s:&AppState)->Result<&PgPool,HandlerError>{s.auth.pool().filter(|_|s.db_available).ok_or_else(||fail(StatusCode::SERVICE_UNAVAILABLE,"IDENTITY_STORAGE_UNAVAILABLE"))}
pub async fn active(pool:&PgPool,u:&AuthUser)->Result<Uuid,HandlerError>{
    let id=Uuid::parse_str(&u.user_id).map_err(|_|fail(StatusCode::UNAUTHORIZED,"ACCOUNT_REQUIRED"))?;
    let yes:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_account_state WHERE user_id=$1 AND state='active')").bind(id).fetch_one(pool).await.map_err(storage)?;
    if !yes{return Err(fail(StatusCode::FORBIDDEN,"ACCOUNT_INACTIVE"));} Ok(id)
}
async fn transaction<'a>(pool:&'a PgPool,uid:Uuid)->Result<Transaction<'a,Postgres>,HandlerError>{
    let mut tx=pool.begin().await.map_err(storage)?;
    sqlx::query("SELECT set_config('app.account_id',$1,true)").bind(uid.to_string()).execute(&mut *tx).await.map_err(storage)?;Ok(tx)
}
async fn subject_access(tx:&mut Transaction<'_,Postgres>,uid:Uuid,id:Uuid,edit:bool)->Result<Uuid,HandlerError>{
    sqlx::query_scalar("SELECT s.owner_user_id FROM corpus_subjects s JOIN subject_grants g ON g.subject_id=s.id AND g.owner_user_id=s.owner_user_id JOIN user_account_state a ON a.user_id=g.grantee_user_id WHERE s.id=$1 AND g.grantee_user_id=$2 AND g.revoked_at IS NULL AND s.archived_at IS NULL AND a.state='active' AND (NOT $3 OR g.can_edit) FOR SHARE OF g,a")
        .bind(id).bind(uid).bind(edit).fetch_optional(&mut **tx).await.map_err(storage)?.ok_or_else(||fail(StatusCode::NOT_FOUND,"SUBJECT_NOT_FOUND"))
}
pub async fn context(State(s):State<AppState>,Extension(u):Extension<AuthUser>)->Result<Response,HandlerError>{
    let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;
    let account:Value=sqlx::query_scalar("SELECT jsonb_build_object('id',u.id,'email',p.preferred_email,'full_name',u.full_name,'preferred_email',p.preferred_email) FROM users u LEFT JOIN account_preferences p ON p.user_id=u.id WHERE u.id=$1")
        .bind(uid).fetch_one(&mut *tx).await.map_err(storage)?;
    let subjects:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',s.id,'key',s.subject_key,'display_name',s.canonical_name,'can_read',true,'can_edit',g.can_edit,'can_calculate',g.can_calculate,'profile_revision',COALESCE((SELECT max(revision) FROM subject_profile_revisions p WHERE p.subject_id=s.id),0)) FROM corpus_subjects s JOIN subject_grants g ON g.subject_id=s.id AND g.owner_user_id=s.owner_user_id WHERE g.grantee_user_id=$1 AND g.revoked_at IS NULL AND s.archived_at IS NULL ORDER BY s.id")
        .bind(uid).fetch_all(&mut *tx).await.map_err(storage)?;
    let relationships:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',r.id,'key',r.relationship_key,'label',r.label,'can_read',true,'member_subject_ids',COALESCE((SELECT jsonb_agg(m.subject_id ORDER BY m.subject_id) FROM corpus_relationship_members m WHERE m.relationship_id=r.id),'[]'::jsonb)) FROM corpus_relationships r JOIN relationship_grants g ON g.relationship_id=r.id AND g.owner_user_id=r.owner_user_id WHERE g.grantee_user_id=$1 AND g.revoked_at IS NULL AND r.archived_at IS NULL ORDER BY r.id")
        .bind(uid).fetch_all(&mut *tx).await.map_err(storage)?;
    let default:Option<Uuid>=sqlx::query_scalar("SELECT p.default_subject_id FROM account_preferences p JOIN subject_grants g ON g.subject_id=p.default_subject_id AND g.grantee_user_id=p.user_id AND g.revoked_at IS NULL JOIN corpus_subjects s ON s.id=g.subject_id AND s.archived_at IS NULL WHERE p.user_id=$1")
        .bind(uid).fetch_optional(&mut *tx).await.map_err(storage)?.flatten();
    tx.commit().await.map_err(storage)?;
    Ok(private(Json(json!({"schema":"noesis.account-context.v1","account":account,"default_subject_id":default,"subjects":subjects,"relationships":relationships})).into_response()))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct DefaultPatch{default_subject_id:Option<Uuid>}
pub async fn update_context(State(s):State<AppState>,Extension(u):Extension<AuthUser>,Json(p):Json<DefaultPatch>)->Result<Response,HandlerError>{
    let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;
    if let Some(id)=p.default_subject_id{subject_access(&mut tx,uid,id,false).await?;}
    sqlx::query("INSERT INTO account_preferences(user_id,default_subject_id) VALUES($1,$2) ON CONFLICT(user_id) DO UPDATE SET default_subject_id=excluded.default_subject_id").bind(uid).bind(p.default_subject_id).execute(&mut *tx).await.map_err(storage)?;
    tx.commit().await.map_err(storage)?;context(State(s),Extension(u)).await
}
fn profile_resource(id:Uuid,revision:i64,profile:Value)->Value{json!({"schema":"noesis.subject-profile.v1","subject_id":id,"revision":revision,"profile":profile})}
pub async fn profile(State(s):State<AppState>,Extension(u):Extension<AuthUser>,Path(id):Path<Uuid>)->Result<Response,HandlerError>{
    let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;subject_access(&mut tx,uid,id,false).await?;
    let (revision,profile)=sqlx::query_as::<_,(i64,Value)>("SELECT revision,profile FROM subject_profile_revisions WHERE subject_id=$1 ORDER BY revision DESC LIMIT 1").bind(id).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(||fail(StatusCode::NOT_FOUND,"PROFILE_NOT_ADMITTED"))?;
    tx.commit().await.map_err(storage)?;Ok(private(Json(profile_resource(id,revision,profile)).into_response()))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct ProfilePatch{expected_revision:i64,patch:Value}
pub fn validate_patch(p:&Value)->Result<(),HandlerError>{
    let obj=p.as_object().ok_or_else(||fail(StatusCode::BAD_REQUEST,"INVALID_PROFILE"))?;
    if obj.is_empty() || obj.keys().any(|k|!matches!(k.as_str(),"full_name"|"birth_date"|"birth_time"|"birth_location"|"timezone"|"preferences"|"birth_time_confidence"|"source_hashes")){return Err(fail(StatusCode::BAD_REQUEST,"INVALID_PROFILE_FIELD"));}
    for (k,v) in obj{
        let valid=match k.as_str(){
            "full_name"=>v.as_str().is_some_and(|x|!x.trim().is_empty()&&x.len()<=255),
            "birth_date"=>v.is_null()||v.as_str().is_some_and(|x|chrono::NaiveDate::parse_from_str(x,"%Y-%m-%d").is_ok()),
            "birth_time"=>v.is_null()||v.as_str().is_some_and(|x|chrono::NaiveTime::parse_from_str(x,"%H:%M").or_else(|_|chrono::NaiveTime::parse_from_str(x,"%H:%M:%S")).is_ok()),
            "timezone"=>v.as_str().is_some_and(|x|x.parse::<chrono_tz::Tz>().is_ok()),
            "birth_time_confidence"=>v.as_str().is_some_and(|x|matches!(x,"unspecified"|"unknown"|"approximate"|"exact"|"recorded")),
            "preferences"=>v.is_object()&&v.to_string().len()<=16384,
            "source_hashes"=>v.as_object().is_some_and(|o|o.values().all(|x|x.as_str().is_some_and(|h|h.len()==64&&h.bytes().all(|c|c.is_ascii_hexdigit())))),
            "birth_location"=>v.is_null()||v.as_object().is_some_and(|o|o.len()==3&&o.get("lat").and_then(Value::as_f64).is_some_and(|n|(-90.0..=90.0).contains(&n))&&o.get("lng").and_then(Value::as_f64).is_some_and(|n|(-180.0..=180.0).contains(&n))&&o.get("name").and_then(Value::as_str).is_some_and(|x|x.len()<=255)),
            _=>false};
        if !valid{return Err(fail(StatusCode::BAD_REQUEST,"INVALID_PROFILE_VALUE"));}
    } Ok(())
}
pub async fn update_profile(State(s):State<AppState>,Extension(u):Extension<AuthUser>,Path(id):Path<Uuid>,Json(p):Json<ProfilePatch>)->Result<Response,HandlerError>{
    validate_patch(&p.patch)?;let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;let owner=subject_access(&mut tx,uid,id,true).await?;
    // Lock the stable subject row, including a profile with no previous revision.
    sqlx::query("SELECT id FROM corpus_subjects WHERE id=$1 FOR UPDATE").bind(id).execute(&mut *tx).await.map_err(storage)?;
    let current=sqlx::query_as::<_,(i64,Value)>("SELECT revision,profile FROM subject_profile_revisions WHERE subject_id=$1 ORDER BY revision DESC LIMIT 1").bind(id).fetch_optional(&mut *tx).await.map_err(storage)?;
    let (revision,mut profile)=current.unwrap_or((0,json!({"full_name":null,"birth_date":null,"birth_time":null,"birth_location":null,"timezone":"UTC","preferences":{},"birth_time_confidence":"unspecified","source_hashes":{}})));
    if revision!=p.expected_revision{return Err(fail(StatusCode::CONFLICT,"PROFILE_REVISION_CONFLICT"));}
    for(k,v)in p.patch.as_object().unwrap(){profile[k]=v.clone();}
    sqlx::query("INSERT INTO subject_profile_revisions(subject_id,owner_user_id,revision,profile,source_hash,changed_by) VALUES($1,$2,$3,$4,$5,$6)").bind(id).bind(owner).bind(revision+1).bind(&profile).bind(hash_secret(&profile.to_string())).bind(uid).execute(&mut *tx).await.map_err(storage)?;
    tx.commit().await.map_err(storage)?;Ok(private(Json(profile_resource(id,revision+1,profile)).into_response()))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct EnrollmentCreate{subject_id:Uuid,intended_email:Option<String>,sharing_reader_id:Option<Uuid>,expires_in_seconds:Option<i64>}
pub async fn create_enrollment(State(s):State<AppState>,Extension(u):Extension<AuthUser>,identity:Option<Extension<CfIdentity>>,Json(p):Json<EnrollmentCreate>)->Result<Response,HandlerError>{
    if identity.is_none(){return Err(fail(StatusCode::FORBIDDEN,"VERIFIED_IDENTITY_REQUIRED"));}
    let pool=db(&s)?;let uid=active(pool,&u).await?;
    let own:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM corpus_subjects WHERE id=$1 AND owner_user_id=$2 AND archived_at IS NULL)").bind(p.subject_id).bind(uid).fetch_one(pool).await.map_err(storage)?;
    if !own{return Err(fail(StatusCode::NOT_FOUND,"SUBJECT_NOT_FOUND"));}
    let ttl=p.expires_in_seconds.unwrap_or(86400);if !(60..=604800).contains(&ttl){return Err(fail(StatusCode::BAD_REQUEST,"INVALID_EXPIRY"));}
    let token=new_secret();let email=p.intended_email.map(|e|e.trim().to_ascii_lowercase());
    if email.as_ref().is_some_and(|e|!valid_email(e)){return Err(fail(StatusCode::BAD_REQUEST,"INVALID_EMAIL"));}
    let id:Uuid=sqlx::query_scalar("INSERT INTO subject_enrollments(token_hash,subject_id,owner_user_id,intended_email,sharing_reader_id,expires_at) VALUES($1,$2,$3,$4,$5,now()+$6*interval '1 second') RETURNING id")
        .bind(hash_secret(&token)).bind(p.subject_id).bind(uid).bind(email).bind(p.sharing_reader_id).bind(ttl as f64).fetch_one(pool).await.map_err(storage)?;
    Ok(private(Json(json!({"id":id,"token":token,"subject_id":p.subject_id,"sharing_reader_id":p.sharing_reader_id,"expires_in":ttl})).into_response()))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct EnrollmentClaim{token:String,accept_sharing:bool}
async fn bind_enrollment(tx:&mut Transaction<'_,Postgres>,id:Uuid,subject:Uuid,owner:Uuid,claimant:Uuid,reader:Option<Uuid>)->Result<(),HandlerError>{
    // Subject lock plus unique active self account prevent racing/conflicting binds.
    sqlx::query("SELECT id FROM corpus_subjects WHERE id=$1 FOR UPDATE").bind(subject).execute(&mut **tx).await.map_err(storage)?;
    let conflict:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM subject_account_links WHERE subject_id=$1 OR (user_id=$2 AND revoked_at IS NULL))").bind(subject).bind(claimant).fetch_one(&mut **tx).await.map_err(storage)?;
    if conflict{return Err(fail(StatusCode::CONFLICT,"SUBJECT_BINDING_CONFLICT"));}
    sqlx::query("INSERT INTO subject_account_links(subject_id,owner_user_id,user_id,approved_by) VALUES($1,$2,$3,$2)").bind(subject).bind(owner).bind(claimant).execute(&mut **tx).await.map_err(storage)?;
    sqlx::query("INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,can_edit,can_calculate,grantor_user_id) VALUES($1,$2,$3,true,true,$2) ON CONFLICT(subject_id,grantee_user_id) DO UPDATE SET revoked_at=NULL,can_edit=true,can_calculate=true")
        .bind(subject).bind(owner).bind(claimant).execute(&mut **tx).await.map_err(storage)?;
    if let Some(reader)=reader{if reader!=claimant{sqlx::query("INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,can_edit,grantor_user_id) VALUES($1,$2,$3,false,$2) ON CONFLICT(subject_id,grantee_user_id) DO UPDATE SET revoked_at=NULL,can_edit=false").bind(subject).bind(owner).bind(reader).execute(&mut **tx).await.map_err(storage)?;}}
    sqlx::query("UPDATE subject_enrollments SET confirmed_at=now() WHERE id=$1").bind(id).execute(&mut **tx).await.map_err(storage)?;
    sqlx::query("INSERT INTO identity_audit(actor_user_id,event,resource_id) VALUES($1,'enrollment.confirmed',$2)").bind(owner).bind(id).execute(&mut **tx).await.map_err(storage)?;Ok(())
}
pub async fn claim_enrollment(State(s):State<AppState>,Extension(u):Extension<AuthUser>,identity:Option<Extension<CfIdentity>>,Json(p):Json<EnrollmentClaim>)->Result<Response,HandlerError>{
    let Some(Extension(identity))=identity else{return Err(fail(StatusCode::FORBIDDEN,"VERIFIED_IDENTITY_REQUIRED"));};
    if !p.accept_sharing{return Err(fail(StatusCode::BAD_REQUEST,"SHARING_ACCEPTANCE_REQUIRED"));}
    let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;
    let row=sqlx::query_as::<_,(Uuid,Uuid,Uuid,Option<String>,Option<Uuid>)>("UPDATE subject_enrollments SET claimed_by=$2,claimed_email=$3,claimed_at=now() WHERE token_hash=$1 AND claimed_at IS NULL AND revoked_at IS NULL AND expires_at>now() RETURNING id,subject_id,owner_user_id,intended_email,sharing_reader_id")
        .bind(hash_secret(&p.token)).bind(uid).bind(&identity.email).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(||fail(StatusCode::BAD_REQUEST,"INVALID_ENROLLMENT"))?;
    if row.3.as_ref().is_some_and(|e|e!=&identity.email){return Err(fail(StatusCode::FORBIDDEN,"ENROLLMENT_EMAIL_MISMATCH"));}
    let confirmed=row.3.is_some();if confirmed{bind_enrollment(&mut tx,row.0,row.1,row.2,uid,row.4).await?;}
    tx.commit().await.map_err(storage)?;Ok(private(Json(json!({"id":row.0,"subject_id":row.1,"status":if confirmed{"active"}else{"pending_owner_confirmation"},"sharing_reader_id":row.4})).into_response()))
}
pub async fn confirm_enrollment(State(s):State<AppState>,Extension(u):Extension<AuthUser>,identity:Option<Extension<CfIdentity>>,Path(id):Path<Uuid>)->Result<Response,HandlerError>{
    if identity.is_none(){return Err(fail(StatusCode::FORBIDDEN,"VERIFIED_IDENTITY_REQUIRED"));}
    let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;
    let row=sqlx::query_as::<_,(Uuid,Uuid,Option<Uuid>)>("SELECT subject_id,claimed_by,sharing_reader_id FROM subject_enrollments WHERE id=$1 AND owner_user_id=$2 AND claimed_at IS NOT NULL AND confirmed_at IS NULL AND revoked_at IS NULL AND expires_at>now() FOR UPDATE")
        .bind(id).bind(uid).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(||fail(StatusCode::NOT_FOUND,"ENROLLMENT_NOT_FOUND"))?;
    let verified:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM account_identities i JOIN subject_enrollments e ON e.claimed_by=i.user_id AND e.claimed_email=i.verified_email JOIN user_account_state a ON a.user_id=i.user_id WHERE e.id=$1 AND i.revoked_at IS NULL AND a.state='active')").bind(id).fetch_one(&mut *tx).await.map_err(storage)?;
    if !verified{return Err(fail(StatusCode::CONFLICT,"CLAIM_IDENTITY_REVOKED"));}
    bind_enrollment(&mut tx,id,row.0,uid,row.1,row.2).await?;tx.commit().await.map_err(storage)?;
    Ok(private(Json(json!({"id":id,"status":"active"})).into_response()))
}
pub async fn revoke_enrollment(State(s):State<AppState>,Extension(u):Extension<AuthUser>,Path(id):Path<Uuid>)->Result<Response,HandlerError>{
    let pool=db(&s)?;let uid=active(pool,&u).await?;
    sqlx::query("UPDATE subject_enrollments SET revoked_at=now() WHERE id=$1 AND owner_user_id=$2 AND confirmed_at IS NULL").bind(id).bind(uid).execute(pool).await.map_err(storage)?;
    Ok(private(StatusCode::NO_CONTENT.into_response()))
}
fn valid_email(e:&str)->bool{e.len()<=255&&!e.chars().any(char::is_whitespace)&&e.split_once('@').is_some_and(|(l,r)|!l.is_empty()&&r.contains('.')&&!r.contains('@'))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct EmailStart{email:String}
pub async fn start_email_link(State(s):State<AppState>,Extension(u):Extension<AuthUser>,identity:Option<Extension<CfIdentity>>,Json(p):Json<EmailStart>)->Result<Response,HandlerError>{
    // Fresh interactive reauthentication, not an API key or arbitrary account email.
    if identity.is_none(){return Err(fail(StatusCode::FORBIDDEN,"VERIFIED_IDENTITY_REQUIRED"));}
    let pool=db(&s)?;let uid=active(pool,&u).await?;let email=p.email.trim().to_ascii_lowercase();
    if !valid_email(&email){return Err(fail(StatusCode::BAD_REQUEST,"INVALID_EMAIL"));}
    let token=new_secret();sqlx::query("INSERT INTO account_email_links(token_hash,user_id,requested_email,expires_at) VALUES($1,$2,$3,now()+interval '10 minutes')").bind(hash_secret(&token)).bind(uid).bind(email).execute(pool).await.map_err(storage)?;
    Ok(private(Json(json!({"status":"pending_verification","challenge":token,"expires_in":600,"complete_path":"/api/v1/account/email-links/complete"})).into_response()))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct EmailComplete{challenge:String}
/// Public route verifies the provider assertion directly, BEFORE resolving a new
/// email into a different account. Challenge binds proof to the original account.
pub async fn complete_email_link(State(s):State<AppState>,headers:axum::http::HeaderMap,Json(p):Json<EmailComplete>)->Result<Response,HandlerError>{
    let assertion=headers.get("cf-access-jwt-assertion").or_else(||headers.get("cf-authorization")).and_then(|h|h.to_str().ok()).ok_or_else(||fail(StatusCode::UNAUTHORIZED,"VERIFIED_IDENTITY_REQUIRED"))?;
    let validator=s.cf_access_validator.as_ref().ok_or_else(||fail(StatusCode::SERVICE_UNAVAILABLE,"IDENTITY_PROVIDER_UNAVAILABLE"))?;
    let identity=validator.validate_token(assertion).await.map_err(|_|fail(StatusCode::UNAUTHORIZED,"INVALID_IDENTITY_PROOF"))?;
    let pool=db(&s)?;let mut tx=pool.begin().await.map_err(storage)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))").bind(format!("identity:{}:{}",identity.issuer,identity.sub)).execute(&mut *tx).await.map_err(storage)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))").bind(format!("email:{}",identity.email)).execute(&mut *tx).await.map_err(storage)?;
    let uid:Uuid=sqlx::query_scalar("UPDATE account_email_links SET consumed_at=now() WHERE token_hash=$1 AND requested_email=$2 AND consumed_at IS NULL AND expires_at>now() RETURNING user_id")
        .bind(hash_secret(&p.challenge)).bind(&identity.email).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(||fail(StatusCode::BAD_REQUEST,"INVALID_EMAIL_CHALLENGE"))?;
    let conflict:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM account_identities WHERE (verified_email=$1 OR (issuer=$2 AND provider_subject=$3)) AND (user_id<>$4 OR revoked_at IS NOT NULL)) OR EXISTS(SELECT 1 FROM users WHERE lower(btrim(email))=$1 AND id<>$4)")
        .bind(&identity.email).bind(&identity.issuer).bind(&identity.sub).bind(uid).fetch_one(&mut *tx).await.map_err(storage)?;
    if conflict{tx.commit().await.map_err(storage)?;return Err(fail(StatusCode::CONFLICT,"EMAIL_ACCOUNT_CONFLICT"));}
    let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_account_state WHERE user_id=$1 AND state='active')").bind(uid).fetch_one(&mut *tx).await.map_err(storage)?;
    if !active{return Err(fail(StatusCode::FORBIDDEN,"ACCOUNT_INACTIVE"));}
    sqlx::query("INSERT INTO account_identities(issuer,provider_subject,user_id,verified_email) VALUES($1,$2,$3,$4) ON CONFLICT(issuer,provider_subject) DO UPDATE SET verified_email=excluded.verified_email,verified_at=now()")
        .bind(identity.issuer).bind(identity.sub).bind(uid).bind(&identity.email).execute(&mut *tx).await.map_err(storage)?;
    sqlx::query("INSERT INTO account_preferences(user_id,preferred_email) VALUES($1,$2) ON CONFLICT(user_id) DO UPDATE SET preferred_email=excluded.preferred_email").bind(uid).bind(&identity.email).execute(&mut *tx).await.map_err(storage)?;
    sqlx::query("UPDATE registered_session_families SET revoked_at=now() WHERE user_id=$1 AND revoked_at IS NULL").bind(uid).execute(&mut *tx).await.map_err(storage)?;
    sqlx::query("INSERT INTO identity_audit(actor_user_id,event) VALUES($1,'email.verified')").bind(uid).execute(&mut *tx).await.map_err(storage)?;
    tx.commit().await.map_err(storage)?;Ok(private(Json(json!({"status":"verified","account_id":uid,"preferred_email":identity.email,"reauthentication_required":true})).into_response()))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Calculate{expected_revision:i64,#[serde(default)]options:std::collections::HashMap<String,Value>,current_time:Option<chrono::DateTime<chrono::Utc>>}
pub async fn calculate(State(s):State<AppState>,Extension(u):Extension<AuthUser>,key:Option<Extension<crate::middleware::ApiKeyPrincipal>>,Path((id,engine)):Path<(Uuid,String)>,Json(p):Json<Calculate>)->Result<Response,HandlerError>{
    let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;let owner=subject_access(&mut tx,uid,id,false).await?;
    let allowed:bool=sqlx::query_scalar("SELECT can_calculate FROM subject_grants WHERE subject_id=$1 AND grantee_user_id=$2 AND revoked_at IS NULL FOR SHARE").bind(id).bind(uid).fetch_one(&mut *tx).await.map_err(storage)?;
    if !allowed{return Err(fail(StatusCode::FORBIDDEN,"SUBJECT_CALCULATION_DENIED"));}
    let (revision,profile)=sqlx::query_as::<_,(i64,Value)>("SELECT revision,profile FROM subject_profile_revisions WHERE subject_id=$1 ORDER BY revision DESC LIMIT 1").bind(id).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(||fail(StatusCode::NOT_FOUND,"PROFILE_NOT_ADMITTED"))?;
    if revision!=p.expected_revision{return Err(fail(StatusCode::CONFLICT,"PROFILE_REVISION_CONFLICT"));}
    // Inputs come only from the accepted profile. No actor auto-population path is called.
    let birth_data=match(profile["birth_date"].as_str(),profile["birth_location"]["lat"].as_f64(),profile["birth_location"]["lng"].as_f64()){
        (Some(date),Some(latitude),Some(longitude))=>Some(noesis_core::BirthData{name:profile["full_name"].as_str().map(str::to_owned),date:date.into(),time:profile["birth_time"].as_str().map(str::to_owned),latitude,longitude,timezone:profile["timezone"].as_str().unwrap_or("UTC").into()}),_=>None};
    if p.options.keys().any(|k|k.starts_with('_')||matches!(k.as_str(),"auth"|"user_id"|"subject_id"|"consciousness_level"|"birth_data")){return Err(fail(StatusCode::BAD_REQUEST,"RESERVED_INPUT"));}
    let input=noesis_core::EngineInput{birth_data,current_time:p.current_time.unwrap_or_else(chrono::Utc::now),location:None,precision:Default::default(),options:p.options};
    crate::validate_engine_input(&input)?;crate::validate_focus_engine_consent(&engine,&input)?;crate::enforce_engine_scope(&u,key.is_some(),&engine,"subjects/calculate")?;crate::enforce_free_tier_quota(&s,&u).await?;
    let input_snapshot=serde_json::to_value(&input).map_err(|_|fail(StatusCode::BAD_REQUEST,"INVALID_INPUT"))?;
    let mut runtime=crate::inject_internal_auth_context(input,&u);runtime.options.insert("consciousness_level".into(),json!(u.consciousness_level));
    // Release reader locks during potentially slow provider work. Reauthorize before persistence/response.
    tx.commit().await.map_err(storage)?;
    let output=s.orchestrator.execute_engine(&engine,runtime,u.consciousness_level).await.map_err(ErrorMapper::map)?;
    let result=serde_json::to_value(output).map_err(|_|fail(StatusCode::INTERNAL_SERVER_ERROR,"INVALID_OUTPUT"))?;
    let mut tx=transaction(pool,uid).await?;subject_access(&mut tx,uid,id,false).await?;
    let still_allowed:bool=sqlx::query_scalar("SELECT can_calculate FROM subject_grants WHERE subject_id=$1 AND grantee_user_id=$2 AND revoked_at IS NULL FOR SHARE").bind(id).bind(uid).fetch_one(&mut *tx).await.map_err(storage)?;
    if !still_allowed{return Err(fail(StatusCode::FORBIDDEN,"SUBJECT_CALCULATION_DENIED"));}
    let run:Uuid=sqlx::query_scalar("INSERT INTO subject_engine_runs(subject_id,owner_user_id,actor_user_id,profile_revision,engine_id,input_snapshot,result) VALUES($1,$2,$3,$4,$5,$6,$7) RETURNING id").bind(id).bind(owner).bind(uid).bind(revision).bind(engine).bind(input_snapshot).bind(&result).fetch_one(&mut *tx).await.map_err(storage)?;
    tx.commit().await.map_err(storage)?;crate::increment_free_tier_counter(&s,&u);
    Ok(private(Json(json!({"id":run,"subject_id":id,"actor_account_id":uid,"profile_revision":revision,"result":result})).into_response()))
}
pub async fn readings(State(s):State<AppState>,Extension(u):Extension<AuthUser>,Path(id):Path<Uuid>)->Result<Response,HandlerError>{
    let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;subject_access(&mut tx,uid,id,false).await?;
    let rows:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'subject_id',subject_id,'actor_account_id',actor_user_id,'profile_revision',profile_revision,'engine_id',engine_id,'created_at',created_at) FROM subject_engine_runs WHERE subject_id=$1 ORDER BY created_at DESC LIMIT 100").bind(id).fetch_all(&mut *tx).await.map_err(storage)?;
    tx.commit().await.map_err(storage)?;Ok(private(Json(json!({"readings":rows})).into_response()))
}
pub async fn reading(State(s):State<AppState>,Extension(u):Extension<AuthUser>,Path((id,run)):Path<(Uuid,Uuid)>)->Result<Response,HandlerError>{
    let pool=db(&s)?;let uid=active(pool,&u).await?;let mut tx=transaction(pool,uid).await?;subject_access(&mut tx,uid,id,false).await?;
    let row:Value=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'subject_id',subject_id,'actor_account_id',actor_user_id,'profile_revision',profile_revision,'result',result) FROM subject_engine_runs WHERE subject_id=$1 AND id=$2").bind(id).bind(run).fetch_optional(&mut *tx).await.map_err(storage)?.ok_or_else(||fail(StatusCode::NOT_FOUND,"READING_NOT_FOUND"))?;
    tx.commit().await.map_err(storage)?;Ok(private(Json(row).into_response()))
}
#[cfg(test)]mod tests{use super::*;
#[test]fn rejects_authority_and_malformed_profiles(){for p in [json!({"email":"a@example.test"}),json!({"user_id":"x"}),json!({"birth_date":"bad"}),json!({"birth_location":{"lat":99,"lng":1,"name":"x"}}),json!({"timezone":"Mars"})]{assert!(validate_patch(&p).is_err());}assert!(validate_patch(&json!({"birth_time":null,"birth_time_confidence":"unknown","timezone":"UTC"})).is_ok());}
}
