//! Persistence for browser-based desktop sign-in (migration 042).
//!
//! Callers pass SHA-256 hashes, never raw codes or tokens. Code consumption
//! and refresh rotation are single statements or row-locked transactions so a
//! code or refresh token can be redeemed at most once.

use chrono::{DateTime, Utc};
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct DesktopSessionRepository {
    pool: PgPool,
}

/// Snapshot of the identity a code or refresh token was issued for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopGrant {
    pub user_id: Uuid,
    pub tier: String,
    pub consciousness_level: i16,
}

/// A redeemed authorization code, with what the token request must match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedAuthCode {
    pub grant: DesktopGrant,
    pub code_challenge: String,
    pub redirect_uri: String,
}

/// Result of presenting a refresh token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshOutcome {
    /// The token was live and is now rotated; issue a successor in `family_id`.
    Rotated {
        grant: DesktopGrant,
        family_id: Uuid,
    },
    /// The token was already rotated: treat as theft and revoke the family.
    Reused,
    /// Unknown, expired, or revoked.
    Invalid,
}

impl DesktopSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_auth_code(
        &self,
        code_hash: &str,
        grant: &DesktopGrant,
        code_challenge: &str,
        redirect_uri: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO desktop_auth_codes
                (code_hash, user_id, code_challenge, redirect_uri, tier, consciousness_level, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(code_hash)
        .bind(grant.user_id)
        .bind(code_challenge)
        .bind(redirect_uri)
        .bind(&grant.tier)
        .bind(grant.consciousness_level)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mark a live code consumed and return it. `None` when the code is
    /// unknown, expired, or already consumed.
    pub async fn consume_auth_code(
        &self,
        code_hash: &str,
    ) -> Result<Option<ConsumedAuthCode>, Error> {
        let row = sqlx::query_as::<_, (Uuid, String, i16, String, String)>(
            r#"
            UPDATE desktop_auth_codes
               SET consumed_at = NOW()
             WHERE code_hash = $1
               AND consumed_at IS NULL
               AND expires_at > NOW()
            RETURNING user_id, tier, consciousness_level, code_challenge, redirect_uri
            "#,
        )
        .bind(code_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(user_id, tier, consciousness_level, code_challenge, redirect_uri)| ConsumedAuthCode {
                grant: DesktopGrant {
                    user_id,
                    tier,
                    consciousness_level,
                },
                code_challenge,
                redirect_uri,
            },
        ))
    }

    pub async fn insert_refresh_token(
        &self,
        token_hash: &str,
        family_id: Uuid,
        grant: &DesktopGrant,
        expires_at: DateTime<Utc>,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO desktop_refresh_tokens
                (token_hash, family_id, user_id, tier, consciousness_level, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(token_hash)
        .bind(family_id)
        .bind(grant.user_id)
        .bind(&grant.tier)
        .bind(grant.consciousness_level)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Redeem a refresh token. A live token is marked rotated; a token that
    /// was already rotated revokes its whole family (reuse detection).
    pub async fn rotate_refresh_token(&self, token_hash: &str) -> Result<RefreshOutcome, Error> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query_as::<_, (Uuid, Uuid, String, i16, bool, bool, bool)>(
            r#"
            SELECT family_id, user_id, tier, consciousness_level,
                   rotated_at IS NOT NULL,
                   revoked_at IS NOT NULL,
                   expires_at <= NOW()
              FROM desktop_refresh_tokens
             WHERE token_hash = $1
             FOR UPDATE
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&mut *tx)
        .await?;

        let Some((family_id, user_id, tier, consciousness_level, rotated, revoked, expired)) = row
        else {
            tx.rollback().await?;
            return Ok(RefreshOutcome::Invalid);
        };

        if revoked || expired {
            tx.rollback().await?;
            return Ok(RefreshOutcome::Invalid);
        }

        if rotated {
            sqlx::query(
                "UPDATE desktop_refresh_tokens SET revoked_at = NOW() WHERE family_id = $1 AND revoked_at IS NULL",
            )
            .bind(family_id)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok(RefreshOutcome::Reused);
        }

        sqlx::query("UPDATE desktop_refresh_tokens SET rotated_at = NOW() WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        Ok(RefreshOutcome::Rotated {
            grant: DesktopGrant {
                user_id,
                tier,
                consciousness_level,
            },
            family_id,
        })
    }

    /// Revoke every token in the family that `token_hash` belongs to.
    /// Returns whether a family was found.
    pub async fn revoke_family_of(&self, token_hash: &str) -> Result<bool, Error> {
        let result = sqlx::query(
            r#"
            UPDATE desktop_refresh_tokens
               SET revoked_at = NOW()
             WHERE revoked_at IS NULL
               AND family_id = (SELECT family_id FROM desktop_refresh_tokens WHERE token_hash = $1)
            "#,
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
