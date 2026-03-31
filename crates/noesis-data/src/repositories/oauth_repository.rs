use crate::models::oauth_account::OAuthAccount;
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct OAuthRepository {
    pool: PgPool,
}

impl OAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find an OAuth account by provider and provider-side user ID.
    pub async fn find_by_provider_user_id(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccount>, Error> {
        sqlx::query_as::<_, OAuthAccount>(
            r#"
            SELECT id, user_id, provider, provider_user_id, provider_email,
                   provider_username, created_at, updated_at
            FROM oauth_accounts
            WHERE provider = $1 AND provider_user_id = $2
            "#,
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Create a new OAuth account link between a user and an external provider.
    pub async fn create(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_user_id: &str,
        provider_email: Option<&str>,
        provider_username: Option<&str>,
    ) -> Result<OAuthAccount, Error> {
        sqlx::query_as::<_, OAuthAccount>(
            r#"
            INSERT INTO oauth_accounts (id, user_id, provider, provider_user_id, provider_email, provider_username)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, provider, provider_user_id, provider_email,
                      provider_username, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(provider)
        .bind(provider_user_id)
        .bind(provider_email)
        .bind(provider_username)
        .fetch_one(&self.pool)
        .await
    }
}
