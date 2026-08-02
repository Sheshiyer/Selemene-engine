use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use noesis_auth::AuthUser;
use reqwest::Client;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfIdentity {
    pub sub: String,
    pub email: String,
    pub groups: Vec<String>,
}

const SUPPORTED_ROLES: &[&str] = &["viewer", "support", "admin", "platform-admin"];

pub fn map_cf_group_to_role(group: &str) -> String {
    match group.trim() {
        "selemene-admin" => "platform-admin".to_string(),
        other => other.to_string(),
    }
}

pub fn roles_from_cf_groups(groups: &[String]) -> Vec<String> {
    let supported: BTreeSet<&str> = SUPPORTED_ROLES.iter().copied().collect();
    let mut roles = groups
        .iter()
        .map(|group| map_cf_group_to_role(group))
        .filter(|role| supported.contains(role.as_str()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if roles.is_empty() {
        roles.push("viewer".to_string());
    }

    roles
}

pub fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    let mut permissions = BTreeSet::from(["basic:access".to_string()]);

    if roles
        .iter()
        .any(|role| role == "platform-admin" || role == "admin")
    {
        permissions.extend([
            "admin:users".to_string(),
            "admin:analytics".to_string(),
            "admin:system:read".to_string(),
            "admin:audit:list".to_string(),
            "admin:audit:read".to_string(),
        ]);
    }

    permissions.into_iter().collect()
}

pub fn development_auth_user(dev_token: &str, provided: Option<&str>) -> Option<AuthUser> {
    if dev_token.is_empty() || provided != Some(dev_token) {
        return None;
    }

    let roles = vec!["platform-admin".to_string()];
    Some(AuthUser {
        user_id: "00000000-0000-0000-0000-000000000001".to_string(),
        tier: "enterprise".to_string(),
        permissions: permissions_for_roles(&roles),
        rate_limit: 10_000,
        consciousness_level: 5,
        jti: None,
        token_exp: None,
    })
}

pub fn auth_user_from_parts(
    user_id: uuid::Uuid,
    tier: &str,
    consciousness_level: i32,
    roles: &[String],
) -> AuthUser {
    AuthUser {
        user_id: user_id.to_string(),
        tier: tier.to_string(),
        permissions: permissions_for_roles(roles),
        rate_limit: match tier {
            "enterprise" => 10_000,
            "premium" => 1_000,
            "free" => 60,
            _ => 10,
        },
        consciousness_level: consciousness_level.clamp(0, 5) as u8,
        jti: None,
        token_exp: None,
    }
}

pub fn role_values_for_sql(groups: &[String]) -> Vec<String> {
    roles_from_cf_groups(groups)
}

/// Resolve roles from a validated Cloudflare Access identity.
///
/// Access rule groups are policy-building blocks, not identity-provider
/// groups, so their names are not guaranteed to appear in the JWT `groups`
/// claim. An explicitly configured email allowlist provides a fail-closed
/// backend mapping after signature, issuer, and audience validation.
pub fn role_values_for_identity(
    identity: &CfIdentity,
    platform_admin_emails: Option<&str>,
) -> Vec<String> {
    let mut roles = role_values_for_sql(&identity.groups);
    let is_platform_admin = platform_admin_emails
        .into_iter()
        .flat_map(|emails| emails.split(','))
        .map(str::trim)
        .filter(|email| !email.is_empty())
        .any(|email| email.eq_ignore_ascii_case(&identity.email));

    if is_platform_admin {
        roles.retain(|role| role != "viewer");
        roles.push("platform-admin".to_string());
        roles.sort();
        roles.dedup();
    }

    roles
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CfAccessClaims {
    pub sub: String,
    pub email: Option<String>,
    pub aud: serde_json::Value,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub identity_nonce: Option<String>,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

pub fn identity_from_claims(claims: CfAccessClaims) -> Result<CfIdentity, String> {
    let email = claims
        .email
        .map(|email| email.trim().to_ascii_lowercase())
        .filter(|email| !email.is_empty())
        .ok_or_else(|| "Cloudflare identity email missing".to_string())?;

    if claims.sub.trim().is_empty() {
        return Err("Cloudflare identity sub missing".to_string());
    }

    Ok(CfIdentity {
        sub: claims.sub,
        email,
        groups: claims.groups,
    })
}

/// A single JWKS key returned by Cloudflare Access.
#[derive(Debug, Clone, Deserialize)]
struct JwksKey {
    kid: String,
    kty: String,
    #[serde(default)]
    #[allow(dead_code)]
    alg: Option<String>,
    n: String,
    e: String,
}

/// JWKS response from Cloudflare Access `/cdn-cgi/access/certs`.
#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<JwksKey>,
}

#[derive(Debug, Clone)]
struct CachedJwks {
    keys: HashMap<String, JwksKey>,
    fetched_at: Instant,
}

impl CachedJwks {
    fn is_stale(&self, ttl: Duration) -> bool {
        self.fetched_at.elapsed() > ttl
    }
}

/// Validates Cloudflare Access JSON Web Tokens against the issuer's JWKS endpoint.
#[derive(Debug, Clone)]
pub struct CfAccessValidator {
    issuer: String,
    audience: String,
    client: Client,
    jwks: Arc<RwLock<Option<CachedJwks>>>,
    jwks_ttl: Duration,
}

impl CfAccessValidator {
    pub fn new(issuer: String, audience: String) -> Self {
        Self {
            issuer: issuer.trim_end_matches('/').to_string(),
            audience,
            client: Client::new(),
            jwks: Arc::new(RwLock::new(None)),
            jwks_ttl: Duration::from_secs(3600),
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    fn with_client(issuer: String, audience: String, client: Client) -> Self {
        Self {
            issuer: issuer.trim_end_matches('/').to_string(),
            audience,
            client,
            jwks: Arc::new(RwLock::new(None)),
            jwks_ttl: Duration::from_secs(3600),
        }
    }

    /// Validates a Cloudflare Access JWT token.
    ///
    /// Steps:
    /// 1. Decode the token header to extract the key ID (`kid`).
    /// 2. Fetch the JWKS from `{issuer}/cdn-cgi/access/certs` (cached for 1 hour).
    /// 3. Build an RSA decoding key from the matching public key.
    /// 4. Verify the signature, issuer, audience, and expiration.
    /// 5. Extract identity claims (email, sub, groups).
    pub async fn validate_token(&self, token: &str) -> Result<CfIdentity, String> {
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| format!("Invalid Cloudflare Access token header: {}", e))?;

        let kid = header
            .kid
            .as_deref()
            .ok_or_else(|| "Cloudflare Access token missing 'kid' header claim".to_string())?;

        let key = self.get_key(kid).await?;

        let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
            .map_err(|e| format!("Invalid Cloudflare Access JWKS key ({}): {}", kid, e))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_audience(&[self.audience.as_str()]);
        validation.leeway = 60; // 1 minute clock skew

        let token_data: TokenData<CfAccessClaims> =
            jsonwebtoken::decode(token, &decoding_key, &validation)
                .map_err(|e| format!("Cloudflare Access JWT validation failed: {}", e))?;

        identity_from_claims(token_data.claims)
    }

    /// Returns a cached JWKS key, fetching the JWKS if missing, stale, or unknown key.
    async fn get_key(&self, kid: &str) -> Result<JwksKey, String> {
        // Fast path: check cache without stale check if the key exists.
        {
            let read = self.jwks.read().await;
            if let Some(cached) = read.as_ref() {
                if let Some(key) = cached.keys.get(kid) {
                    if !cached.is_stale(self.jwks_ttl) {
                        return Ok(key.clone());
                    }
                }
            }
        }

        // Slow path: refresh JWKS and retry.
        self.fetch_jwks().await?;

        let read = self.jwks.read().await;
        let cached = read
            .as_ref()
            .ok_or_else(|| "Cloudflare Access JWKS cache unavailable after fetch".to_string())?;
        cached
            .keys
            .get(kid)
            .cloned()
            .ok_or_else(|| format!("Cloudflare Access key '{}' not found in JWKS", kid))
    }

    /// Fetches the Cloudflare Access JWKS and caches the RSA keys by `kid`.
    async fn fetch_jwks(&self) -> Result<(), String> {
        let certs_url = format!("{}/cdn-cgi/access/certs", self.issuer);

        let response = self.client.get(&certs_url).send().await.map_err(|e| {
            format!(
                "Failed to fetch Cloudflare Access JWKS from {}: {}",
                certs_url, e
            )
        })?;

        if !response.status().is_success() {
            return Err(format!(
                "Cloudflare Access JWKS endpoint returned status {}",
                response.status()
            ));
        }

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Cloudflare Access JWKS response: {}", e))?;

        let keys: HashMap<String, JwksKey> = jwks
            .keys
            .into_iter()
            .filter(|k| k.kty.eq_ignore_ascii_case("RSA"))
            .map(|k| (k.kid.clone(), k))
            .collect();

        if keys.is_empty() {
            return Err("Cloudflare Access JWKS contained no RSA keys".to_string());
        }

        let mut write = self.jwks.write().await;
        *write = Some(CachedJwks {
            keys,
            fetched_at: Instant::now(),
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_selemene_admin_to_platform_admin() {
        assert_eq!(map_cf_group_to_role("selemene-admin"), "platform-admin");
    }

    #[test]
    fn maps_cf_groups_directly_otherwise() {
        assert_eq!(map_cf_group_to_role("support"), "support");
    }

    #[test]
    fn filters_duplicate_empty_and_unsupported_roles() {
        let groups = vec![
            "".to_string(),
            "support".to_string(),
            "support".to_string(),
            "selemene-admin".to_string(),
            "random-group".to_string(),
        ];
        assert_eq!(
            roles_from_cf_groups(&groups),
            vec!["platform-admin", "support"]
        );
    }

    #[test]
    fn development_auth_requires_matching_token() {
        assert!(development_auth_user("dev-secret", Some("wrong")).is_none());
        let user = development_auth_user("dev-secret", Some("dev-secret")).expect("dev user");
        assert_eq!(user.user_id, "00000000-0000-0000-0000-000000000001");
        assert!(user.permissions.contains(&"admin:system:read".to_string()));
        assert_eq!(user.tier, "enterprise");
    }

    #[test]
    fn extracts_identity_from_claims() {
        let claims = CfAccessClaims {
            sub: "cf-sub-123".to_string(),
            email: Some("USER@Example.COM".to_string()),
            aud: serde_json::json!(["aud"]),
            groups: vec!["support".to_string()],
            identity_nonce: None,
            exp: 4_102_444_800,
            iat: 1,
            iss: "https://team.cloudflareaccess.com".to_string(),
        };

        let identity = identity_from_claims(claims).expect("identity");
        assert_eq!(identity.sub, "cf-sub-123");
        assert_eq!(identity.email, "user@example.com");
        assert_eq!(identity.groups, vec!["support"]);
    }

    #[test]
    fn rejects_claims_without_email() {
        let claims = CfAccessClaims {
            sub: "cf-sub-123".to_string(),
            email: None,
            aud: serde_json::json!(["aud"]),
            groups: vec![],
            identity_nonce: None,
            exp: 4_102_444_800,
            iat: 1,
            iss: "https://team.cloudflareaccess.com".to_string(),
        };

        assert_eq!(
            identity_from_claims(claims).unwrap_err(),
            "Cloudflare identity email missing"
        );
    }

    #[test]
    fn auth_user_from_parts_uses_roles_for_permissions() {
        let user_id = uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let roles = vec!["platform-admin".to_string()];
        let user = auth_user_from_parts(user_id, "premium", 4, &roles);

        assert_eq!(user.user_id, user_id.to_string());
        assert_eq!(user.tier, "premium");
        assert_eq!(user.consciousness_level, 4);
        assert!(user.permissions.contains(&"admin:system:read".to_string()));
    }

    #[test]
    fn role_values_for_sql_are_deterministic() {
        let groups = vec![
            "support".to_string(),
            "selemene-admin".to_string(),
            "admin".to_string(),
        ];
        assert_eq!(
            role_values_for_sql(&groups),
            vec!["admin", "platform-admin", "support"]
        );
    }

    #[test]
    fn validated_identity_requires_explicit_email_match_for_platform_admin() {
        let identity = CfIdentity {
            sub: "cf-sub-123".to_string(),
            email: "sheshnarayan.iyer@gmail.com".to_string(),
            groups: vec![],
        };

        assert_eq!(role_values_for_identity(&identity, None), vec!["viewer"]);
        assert_eq!(
            role_values_for_identity(&identity, Some("other@example.com")),
            vec!["viewer"]
        );
        assert_eq!(
            role_values_for_identity(
                &identity,
                Some("other@example.com, SHESHNARAYAN.IYER@GMAIL.COM")
            ),
            vec!["platform-admin"]
        );
    }
}
