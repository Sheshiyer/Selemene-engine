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
}
