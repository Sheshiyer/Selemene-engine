const MIGRATION: &str = include_str!("../../../migrations/037_living_reading_invitations.sql");

#[test]
fn invitation_schema_persists_only_a_digest_and_a_single_reading_binding() {
    assert!(MIGRATION.contains("reading_id UUID NOT NULL REFERENCES archived_readings(id)"));
    assert!(MIGRATION.contains("token_digest VARCHAR(64) NOT NULL UNIQUE"));
    assert!(MIGRATION.contains("token_digest ~ '^[0-9a-f]{64}$'"));
    assert!(MIGRATION.contains("expires_at TIMESTAMPTZ NOT NULL"));
    assert!(MIGRATION.contains("revoked_at TIMESTAMPTZ"));
    assert!(MIGRATION.contains("created_by_user_id"));
    assert!(MIGRATION.contains("revoked_by_user_id"));

    let table = MIGRATION
        .split_once("CREATE TABLE archived_reading_invitations")
        .expect("invitation table")
        .1
        .split_once(");")
        .expect("invitation table end")
        .0;
    for forbidden in [
        " plaintext",
        " token TEXT",
        "owner_email",
        "object_locator",
        "checksum",
    ] {
        assert!(
            !table.to_ascii_lowercase().contains(forbidden),
            "invitation storage must not contain {forbidden}"
        );
    }
}

#[test]
fn invitation_schema_has_active_lookup_and_recoverable_rollback() {
    assert!(MIGRATION.contains("idx_archived_reading_invitations_active_expiry"));
    assert!(MIGRATION.contains("WHERE revoked_at IS NULL"));
    assert!(MIGRATION.contains("-- DROP TABLE IF EXISTS archived_reading_invitations;"));
}
