use noesis_auth::AuthUser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfIdentity {
    pub sub: String,
    pub email: String,
    pub groups: Vec<String>,
}

pub fn map_cf_group_to_role(group: &str) -> String {
    todo!("map CF group to local role")
}

pub fn roles_from_cf_groups(groups: &[String]) -> Vec<String> {
    todo!("map CF groups to user_roles values")
}

pub fn development_auth_user(dev_token: &str, provided: Option<&str>) -> Option<AuthUser> {
    todo!("return platform-admin user when dev token matches")
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
