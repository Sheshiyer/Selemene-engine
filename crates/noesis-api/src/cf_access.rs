use noesis_auth::AuthUser;
use std::collections::BTreeSet;

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

#[derive(Debug, Clone)]
pub struct CfAccessValidator {
    issuer: String,
    audience: String,
}

impl CfAccessValidator {
    pub fn new(issuer: String, audience: String) -> Self {
        Self { issuer, audience }
    }

    pub async fn validate_token(&self,
        _token: &str,
    ) -> Result<CfIdentity, String> {
        Err(format!(
            "Cloudflare Access JWT validation not fully wired for issuer {} and audience {}",
            self.issuer, self.audience
        ))
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
        assert_eq!(roles_from_cf_groups(&groups), vec!["platform-admin", "support"]);
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
}
