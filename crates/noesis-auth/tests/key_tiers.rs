use noesis_auth::key_tiers::KeyTier;

#[test]
fn aliases_and_bundles_are_canonical_and_do_not_grant_admin() {
    for (input, code, label, level, limit) in [
        ("basic", "free", "Basic", 0, 60),
        ("free", "free", "Basic", 0, 60),
        ("pro", "premium", "Pro", 3, 1000),
        ("premium", "premium", "Pro", 3, 1000),
        (" Enterprise ", "enterprise", "Enterprise", 5, 10000),
    ] {
        let tier = KeyTier::parse(input).unwrap();
        let e = tier.entitlements();
        assert_eq!((tier.code(), tier.label()), (code, label));
        assert_eq!((e.consciousness_level, e.rate_limit), (level, limit));
        assert!(e.permissions.contains(&"basic:access".into()));
        assert!(e
            .permissions
            .iter()
            .all(|p| !p.starts_with("admin:") && !p.contains('*')));
    }
    assert!(KeyTier::parse("standard").is_none());
    assert!(KeyTier::parse("").is_none());
    assert_eq!(KeyTier::Basic.entitlements().permissions.len(), 3);
    assert_eq!(KeyTier::Pro.entitlements().permissions.len(), 20);
}

#[test]
fn overrides_can_only_restrict_the_selected_tier() {
    let tier = KeyTier::Basic;
    assert!(tier
        .restrict(Some(vec!["human-design:read".into()]), None, None)
        .is_err());
    assert!(tier
        .restrict(Some(vec!["admin:*".into()]), None, None)
        .is_err());
    assert!(tier.restrict(None, Some(1), None).is_err());
    assert!(tier.restrict(None, None, Some(0)).is_err());
    assert!(tier.restrict(None, None, Some(61)).is_err());
    let e = tier
        .restrict(
            Some(vec!["numerology:read".into(), "numerology:read".into()]),
            Some(0),
            Some(10),
        )
        .unwrap();
    assert_eq!(e.permissions, vec!["basic:access", "numerology:read"]);
    assert_eq!(e.rate_limit, 10);
}

#[test]
fn tier_transition_preserves_explicit_restrictions_and_handles_managed_bundles() {
    let old = KeyTier::Basic.entitlements();
    let upgraded = KeyTier::Enterprise.transition(true, &old.permissions, 0, 60);
    assert_eq!(
        upgraded.permissions,
        KeyTier::Enterprise.entitlements().permissions
    );
    assert_eq!(upgraded.consciousness_level, 5);
    let limited = vec![
        "basic:access".into(),
        "human-design:read".into(),
        "numerology:read".into(),
    ];
    let down = KeyTier::Basic.transition(false, &limited, 2, 10000);
    assert_eq!(down.permissions, vec!["basic:access", "numerology:read"]);
    assert_eq!((down.consciousness_level, down.rate_limit), (0, 60));
}

#[test]
fn restricted_tier_round_trip_never_becomes_managed() {
    let basic = KeyTier::Basic.entitlements();
    let down = KeyTier::Basic.transition(false, &basic.permissions, 0, 60);
    let up = KeyTier::Enterprise.transition(false, &down.permissions, 0, 60);
    assert_eq!(up.permissions, basic.permissions);
    assert_eq!(up.consciousness_level, 0);
}
