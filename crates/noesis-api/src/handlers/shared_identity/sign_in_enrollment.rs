//! Redeem an owner's email authorization during sign-in and handle optional sharing afterward.
use super::{
    active, bind_self_profile, db, empty_profile, fail, platform_admin, private, project_profile,
    self_profile, set_account_context, storage, transaction, valid_relationship_selection,
    verified_identity, HandlerError,
};
use crate::{cf_access::CfIdentity, desktop_auth::hash_secret, AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use noesis_auth::AuthUser;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Acquire, Postgres, Transaction};
use uuid::Uuid;

pub(crate) async fn activate_sign_in_enrollment(
    tx: &mut Transaction<'_, Postgres>,
    claimant: Uuid,
    email: &str,
) -> Result<(), HandlerError> {
    let invitation: Option<(Uuid, Option<Uuid>, Uuid)> = sqlx::query_as(
        "SELECT e.id,e.subject_id,e.owner_user_id FROM subject_enrollments e
         JOIN user_account_state inviter ON inviter.user_id=e.owner_user_id AND inviter.state='active'
         WHERE e.mode='sign_in' AND e.intended_email=$1 AND e.confirmed_at IS NULL AND e.revoked_at IS NULL
         FOR UPDATE OF e FOR SHARE OF inviter",
    )
    .bind(email)
    .fetch_optional(&mut **tx)
    .await
    .map_err(storage)?;
    let Some((invitation, target, inviter)) = invitation else {
        return Ok(());
    };
    let already_linked: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM subject_account_links WHERE user_id=$1 AND revoked_at IS NULL)",
    )
    .bind(claimant)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    if already_linked || (target.is_none() && !platform_admin(tx, inviter).await?) {
        return Ok(());
    }

    // A competing enrollment can claim the self-link first. The savepoint removes
    // any unlinked subject we just created while allowing normal sign-in to finish.
    let mut binding = tx.begin().await.map_err(storage)?;
    set_account_context(&mut binding, claimant).await?;
    let (subject, owner) = match target {
        Some(subject) => (subject, inviter),
        None => {
            let subject = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO corpus_subjects(id,owner_user_id,subject_key,canonical_name,reconciliation_state)
                 VALUES($1,$2,$3,'My profile','verified')",
            )
            .bind(subject)
            .bind(claimant)
            .bind(format!("profile-{subject}"))
            .execute(&mut *binding)
            .await
            .map_err(storage)?;
            (subject, claimant)
        }
    };
    let Some(name) = bind_self_profile(&mut binding, subject, owner, claimant).await? else {
        binding.rollback().await.map_err(storage)?;
        return Ok(());
    };

    let profile: Option<Value> = sqlx::query_scalar(
        "SELECT profile FROM subject_profile_revisions WHERE subject_id=$1
         ORDER BY revision DESC LIMIT 1",
    )
    .bind(subject)
    .fetch_optional(&mut *binding)
    .await
    .map_err(storage)?;
    let profile = match profile {
        Some(profile) => profile,
        None => {
            let profile = empty_profile(target.map(|_| name.as_str()));
            sqlx::query(
                "INSERT INTO subject_profile_revisions(subject_id,owner_user_id,revision,profile,source_hash,changed_by)
                 VALUES($1,$2,1,$3,$4,$5)",
            )
            .bind(subject)
            .bind(owner)
            .bind(&profile)
            .bind(hash_secret(&profile.to_string()))
            .bind(claimant)
            .execute(&mut *binding)
            .await
            .map_err(storage)?;
            profile
        }
    };
    project_profile(&mut binding, claimant, &profile, false).await?;
    sqlx::query(
        "INSERT INTO account_preferences(user_id,preferred_email,default_subject_id) VALUES($1,$2,$3)
         ON CONFLICT(user_id) DO UPDATE SET default_subject_id=excluded.default_subject_id",
    )
    .bind(claimant)
    .bind(email)
    .bind(subject)
    .execute(&mut *binding)
    .await
    .map_err(storage)?;
    sqlx::query(
        "UPDATE subject_enrollments SET claimed_by=$2,claimed_email=$3,claimed_at=now(),confirmed_at=now()
         WHERE id=$1",
    )
    .bind(invitation)
    .bind(claimant)
    .bind(email)
    .execute(&mut *binding)
    .await
    .map_err(storage)?;
    sqlx::query(
        "INSERT INTO identity_audit(actor_user_id,event,resource_id) VALUES($1,'enrollment.sign_in_activated',$2)",
    )
    .bind(claimant)
    .bind(invitation)
    .execute(&mut *binding)
    .await
    .map_err(storage)?;
    binding.commit().await.map_err(storage)?;
    Ok(())
}

pub async fn sharing(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    identity: Option<Extension<CfIdentity>>,
) -> Result<Response, HandlerError> {
    verified_identity(&state, &user, identity).await?;
    let pool = db(&state)?;
    let uid = active(pool, &user).await?;
    let mut tx = transaction(pool, uid).await?;
    let Some((subject, owner)) = self_profile(&mut tx, uid).await? else {
        return Ok(private(
            Json(json!({"readers":[],"offers":[]})).into_response(),
        ));
    };
    let readers: Vec<String> = sqlx::query_scalar(
        "SELECT COALESCE(NULLIF(btrim(u.full_name),''),'Another approved reader')
         FROM subject_grants g JOIN users u ON u.id=g.grantee_user_id
         JOIN user_account_state a ON a.user_id=u.id AND a.state='active'
         WHERE g.subject_id=$1 AND g.owner_user_id=$2 AND g.grantee_user_id<>$3
           AND g.revoked_at IS NULL
         ORDER BY u.full_name,u.id",
    )
    .bind(subject)
    .bind(owner)
    .bind(uid)
    .fetch_all(&mut *tx)
    .await
    .map_err(storage)?;
    let offers: Vec<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('id',e.id,'reader',
                CASE WHEN reader.id IS NOT NULL THEN COALESCE(NULLIF(btrim(reader.full_name),''),'An invited reader') END,
                'relationships',offered.relationships)
         FROM subject_enrollments e
         JOIN user_account_state inviter ON inviter.user_id=e.owner_user_id AND inviter.state='active'
         LEFT JOIN users reader ON reader.id=e.sharing_reader_id AND reader.id<>$3
           AND e.sharing_reader_accepted_at IS NULL
           AND EXISTS (SELECT 1 FROM user_account_state a WHERE a.user_id=reader.id AND a.state='active')
           AND NOT EXISTS (SELECT 1 FROM subject_grants g WHERE g.subject_id=e.subject_id
                           AND g.grantee_user_id=reader.id AND g.revoked_at IS NULL)
         CROSS JOIN LATERAL (
           SELECT COALESCE(jsonb_agg(jsonb_build_object('id',r.id,'label',r.label) ORDER BY r.id),'[]'::jsonb) AS relationships
           FROM subject_enrollment_relationships er
           JOIN corpus_relationships r ON r.id=er.relationship_id AND r.owner_user_id=er.owner_user_id
           WHERE er.enrollment_id=e.id AND er.accepted_at IS NULL AND r.archived_at IS NULL
             AND NOT EXISTS (SELECT 1 FROM relationship_grants g WHERE g.relationship_id=r.id
                             AND g.grantee_user_id=$3 AND g.revoked_at IS NULL)
         ) offered
         WHERE e.subject_id=$1 AND e.owner_user_id=$2 AND e.claimed_by=$3 AND e.mode='sign_in'
           AND e.confirmed_at IS NOT NULL AND e.revoked_at IS NULL
           AND (reader.id IS NOT NULL OR jsonb_array_length(offered.relationships)>0)
         ORDER BY e.created_at",
    )
    .bind(subject)
    .bind(owner)
    .bind(uid)
    .fetch_all(&mut *tx)
    .await
    .map_err(storage)?;
    tx.commit().await.map_err(storage)?;
    Ok(private(
        Json(json!({"readers":readers,"offers":offers})).into_response(),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharingSelection {
    #[serde(default)]
    reader: bool,
    #[serde(default)]
    relationship_ids: Vec<Uuid>,
}

pub async fn accept_enrollment_sharing(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    identity: Option<Extension<CfIdentity>>,
    Path(id): Path<Uuid>,
    Json(mut selection): Json<SharingSelection>,
) -> Result<Response, HandlerError> {
    verified_identity(&state, &user, identity).await?;
    if (!selection.reader && selection.relationship_ids.is_empty())
        || !valid_relationship_selection(&selection.relationship_ids)
    {
        return Err(fail(StatusCode::BAD_REQUEST, "INVALID_SHARING_SELECTION"));
    }
    let pool = db(&state)?;
    let uid = active(pool, &user).await?;
    let mut tx = transaction(pool, uid).await?;
    let unavailable = || fail(StatusCode::NOT_FOUND, "SHARING_OFFER_UNAVAILABLE");
    let (subject, owner, reader, reader_accepted): (Uuid, Uuid, Option<Uuid>, bool) = sqlx::query_as(
        "SELECT e.subject_id,e.owner_user_id,e.sharing_reader_id,e.sharing_reader_accepted_at IS NOT NULL
         FROM subject_enrollments e
         JOIN user_account_state inviter ON inviter.user_id=e.owner_user_id AND inviter.state='active'
         JOIN user_account_state recipient ON recipient.user_id=e.claimed_by AND recipient.state='active'
         JOIN subject_account_links l ON l.subject_id=e.subject_id AND l.owner_user_id=e.owner_user_id
           AND l.user_id=e.claimed_by AND l.revoked_at IS NULL
         JOIN corpus_subjects s ON s.id=l.subject_id AND s.owner_user_id=l.owner_user_id AND s.archived_at IS NULL
         JOIN subject_grants own ON own.subject_id=l.subject_id AND own.owner_user_id=l.owner_user_id
           AND own.grantee_user_id=l.user_id AND own.revoked_at IS NULL
         WHERE e.id=$1 AND e.claimed_by=$2
           AND e.mode='sign_in' AND e.confirmed_at IS NOT NULL AND e.revoked_at IS NULL
         FOR UPDATE OF e FOR SHARE OF inviter,recipient,l,s,own",
    )
    .bind(id)
    .bind(uid)
    .fetch_optional(&mut *tx)
    .await
    .map_err(storage)?
    .ok_or_else(unavailable)?;
    let mut changed = false;
    if selection.reader {
        let reader = reader
            .filter(|reader| *reader != uid)
            .ok_or_else(unavailable)?;
        if !reader_accepted {
            let eligible: Option<Uuid> = sqlx::query_scalar(
                "SELECT user_id FROM user_account_state WHERE user_id=$1 AND state='active' FOR SHARE",
            )
            .bind(reader)
            .fetch_optional(&mut *tx)
            .await
            .map_err(storage)?;
            if eligible.is_none() {
                return Err(unavailable());
            }
            sqlx::query(
                "INSERT INTO subject_grants(subject_id,owner_user_id,grantee_user_id,grantor_user_id)
                 VALUES($1,$2,$3,$2)
                 ON CONFLICT(subject_id,grantee_user_id) DO UPDATE SET
                     revoked_at=NULL,can_edit=false,can_calculate=false,grantor_user_id=excluded.grantor_user_id
                 WHERE subject_grants.revoked_at IS NOT NULL",
            )
            .bind(subject)
            .bind(owner)
            .bind(reader)
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
            sqlx::query(
                "UPDATE subject_enrollments SET sharing_reader_accepted_at=now() WHERE id=$1",
            )
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
            changed = true;
        }
    }
    selection.relationship_ids.sort_unstable();
    for relationship in selection.relationship_ids {
        let (accepted, available): (bool, bool) = sqlx::query_as(
            "SELECT er.accepted_at IS NOT NULL,r.archived_at IS NULL
             FROM subject_enrollment_relationships er
             JOIN corpus_relationships r ON r.id=er.relationship_id AND r.owner_user_id=er.owner_user_id
             WHERE er.enrollment_id=$1 AND er.relationship_id=$2 FOR SHARE OF r",
        )
        .bind(id)
        .bind(relationship)
        .fetch_optional(&mut *tx)
        .await
        .map_err(storage)?
        .ok_or_else(unavailable)?;
        if accepted {
            continue;
        }
        if !available {
            return Err(unavailable());
        }
        sqlx::query(
            "INSERT INTO relationship_grants(relationship_id,owner_user_id,grantee_user_id,grantor_user_id)
             VALUES($1,$2,$3,$2)
             ON CONFLICT(relationship_id,grantee_user_id) DO UPDATE SET
                 revoked_at=NULL,grantor_user_id=excluded.grantor_user_id
             WHERE relationship_grants.revoked_at IS NOT NULL",
        )
        .bind(relationship)
        .bind(owner)
        .bind(uid)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
        sqlx::query(
            "UPDATE subject_enrollment_relationships SET accepted_at=now()
             WHERE enrollment_id=$1 AND relationship_id=$2",
        )
        .bind(id)
        .bind(relationship)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
        changed = true;
    }
    if changed {
        sqlx::query(
            "INSERT INTO identity_audit(actor_user_id,event,resource_id)
             VALUES($1,'enrollment.sharing_accepted',$2)",
        )
        .bind(uid)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
    }
    tx.commit().await.map_err(storage)?;
    Ok(private(Json(json!({"status":"accepted"})).into_response()))
}
