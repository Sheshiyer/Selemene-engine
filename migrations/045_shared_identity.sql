-- Additive account/subject identity. No private seeds. Root migration ledger owns allocation.
CREATE TABLE account_identities (
 issuer text NOT NULL, provider_subject text NOT NULL, user_id uuid NOT NULL REFERENCES users(id),
 verified_email text NOT NULL CHECK (verified_email=lower(btrim(verified_email))),
 verified_at timestamptz NOT NULL DEFAULT now(), revoked_at timestamptz,
 PRIMARY KEY(issuer,provider_subject), UNIQUE(issuer,provider_subject,user_id), CHECK(length(issuer)>0 AND length(provider_subject)>0)
);
CREATE INDEX account_identities_user ON account_identities(user_id);
CREATE INDEX account_identities_email ON account_identities(verified_email);
CREATE TABLE account_preferences (
 user_id uuid PRIMARY KEY REFERENCES users(id), preferred_email text,
 default_subject_id uuid REFERENCES corpus_subjects(id)
);
CREATE INDEX account_preferences_subject ON account_preferences(default_subject_id);
CREATE TABLE subject_account_links (
 subject_id uuid PRIMARY KEY, owner_user_id uuid NOT NULL, user_id uuid NOT NULL REFERENCES users(id),
 approved_by uuid NOT NULL REFERENCES users(id), created_at timestamptz NOT NULL DEFAULT now(), revoked_at timestamptz,
 FOREIGN KEY(subject_id,owner_user_id) REFERENCES corpus_subjects(id,owner_user_id)
);
CREATE UNIQUE INDEX subject_account_links_one_active_self ON subject_account_links(user_id) WHERE revoked_at IS NULL;
CREATE INDEX subject_account_links_owner ON subject_account_links(owner_user_id);
CREATE TABLE subject_grants (
 subject_id uuid NOT NULL, owner_user_id uuid NOT NULL, grantee_user_id uuid NOT NULL REFERENCES users(id),
 can_edit boolean NOT NULL DEFAULT false, can_calculate boolean NOT NULL DEFAULT false, grantor_user_id uuid NOT NULL REFERENCES users(id),
 created_at timestamptz NOT NULL DEFAULT now(), revoked_at timestamptz,
 PRIMARY KEY(subject_id,grantee_user_id),
 FOREIGN KEY(subject_id,owner_user_id) REFERENCES corpus_subjects(id,owner_user_id)
);
CREATE INDEX subject_grants_grantee ON subject_grants(grantee_user_id,subject_id) WHERE revoked_at IS NULL;
CREATE INDEX subject_grants_owner ON subject_grants(owner_user_id);
CREATE TABLE relationship_grants (
 relationship_id uuid NOT NULL, owner_user_id uuid NOT NULL, grantee_user_id uuid NOT NULL REFERENCES users(id),
 grantor_user_id uuid NOT NULL REFERENCES users(id), created_at timestamptz NOT NULL DEFAULT now(), revoked_at timestamptz,
 PRIMARY KEY(relationship_id,grantee_user_id),
 FOREIGN KEY(relationship_id,owner_user_id) REFERENCES corpus_relationships(id,owner_user_id)
);
CREATE INDEX relationship_grants_grantee ON relationship_grants(grantee_user_id,relationship_id) WHERE revoked_at IS NULL;
CREATE INDEX relationship_grants_owner ON relationship_grants(owner_user_id);
CREATE TABLE subject_profile_revisions (
 subject_id uuid NOT NULL, owner_user_id uuid NOT NULL, revision bigint NOT NULL CHECK(revision>0),
 profile jsonb NOT NULL CHECK(jsonb_typeof(profile)='object'), source_hash text NOT NULL CHECK(source_hash ~ '^[0-9a-f]{64}$'),
 changed_by uuid NOT NULL REFERENCES users(id), created_at timestamptz NOT NULL DEFAULT now(),
 PRIMARY KEY(subject_id,revision), FOREIGN KEY(subject_id,owner_user_id) REFERENCES corpus_subjects(id,owner_user_id)
);
CREATE INDEX subject_profile_revisions_owner ON subject_profile_revisions(owner_user_id);
CREATE TABLE subject_enrollments (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), token_hash text NOT NULL UNIQUE CHECK(token_hash ~ '^[0-9a-f]{64}$'),
 subject_id uuid NOT NULL, owner_user_id uuid NOT NULL REFERENCES users(id), intended_email text,
 sharing_reader_id uuid REFERENCES users(id), expires_at timestamptz NOT NULL,
 claimed_by uuid REFERENCES users(id), claimed_email text, claimed_at timestamptz, confirmed_at timestamptz,
 revoked_at timestamptz, created_at timestamptz NOT NULL DEFAULT now(),
 FOREIGN KEY(subject_id,owner_user_id) REFERENCES corpus_subjects(id,owner_user_id),
 CHECK (intended_email IS NULL OR intended_email=lower(btrim(intended_email)))
);
CREATE INDEX subject_enrollments_subject ON subject_enrollments(subject_id);
CREATE INDEX subject_enrollments_owner ON subject_enrollments(owner_user_id);
CREATE TABLE account_email_links (
 token_hash text PRIMARY KEY CHECK(token_hash ~ '^[0-9a-f]{64}$'), user_id uuid NOT NULL REFERENCES users(id),
 requested_email text NOT NULL CHECK(requested_email=lower(btrim(requested_email))),
 expires_at timestamptz NOT NULL, consumed_at timestamptz, created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX account_email_links_user ON account_email_links(user_id);
CREATE TABLE identity_audit (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), actor_user_id uuid NOT NULL REFERENCES users(id),
 event text NOT NULL, resource_id uuid, created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX identity_audit_actor ON identity_audit(actor_user_id,created_at);
CREATE TABLE registered_auth_codes (
 code_hash text PRIMARY KEY, client_id text NOT NULL, user_id uuid NOT NULL REFERENCES users(id),
 code_challenge text NOT NULL, redirect_uri text NOT NULL, expires_at timestamptz NOT NULL, consumed_at timestamptz
);
CREATE INDEX registered_auth_codes_user ON registered_auth_codes(user_id);
CREATE TABLE registered_session_families (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), client_id text NOT NULL, user_id uuid NOT NULL REFERENCES users(id),
 identity_issuer text NOT NULL, identity_subject text NOT NULL,
 expires_at timestamptz NOT NULL, revoked_at timestamptz,
 FOREIGN KEY(identity_issuer,identity_subject,user_id) REFERENCES account_identities(issuer,provider_subject,user_id)
);
ALTER TABLE registered_auth_codes ADD COLUMN identity_issuer text NOT NULL;
ALTER TABLE registered_auth_codes ADD COLUMN identity_subject text NOT NULL;
ALTER TABLE registered_auth_codes ADD FOREIGN KEY(identity_issuer,identity_subject,user_id) REFERENCES account_identities(issuer,provider_subject,user_id);
CREATE INDEX registered_session_families_user ON registered_session_families(user_id);
CREATE TABLE registered_refresh_tokens (
 token_hash text PRIMARY KEY, family_id uuid NOT NULL REFERENCES registered_session_families(id), rotated_at timestamptz
);
CREATE INDEX registered_refresh_family ON registered_refresh_tokens(family_id);
CREATE TABLE registered_access_tokens (
 jti text PRIMARY KEY, family_id uuid NOT NULL REFERENCES registered_session_families(id), expires_at timestamptz NOT NULL
);
CREATE INDEX registered_access_family ON registered_access_tokens(family_id);
CREATE TABLE subject_engine_runs (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), subject_id uuid NOT NULL, owner_user_id uuid NOT NULL,
 actor_user_id uuid NOT NULL REFERENCES users(id), profile_revision bigint NOT NULL,
 engine_id text NOT NULL, input_snapshot jsonb NOT NULL, result jsonb NOT NULL, created_at timestamptz NOT NULL DEFAULT now(),
 FOREIGN KEY(subject_id,owner_user_id) REFERENCES corpus_subjects(id,owner_user_id),
 FOREIGN KEY(subject_id,profile_revision) REFERENCES subject_profile_revisions(subject_id,revision)
);
CREATE INDEX subject_engine_runs_subject ON subject_engine_runs(subject_id,created_at DESC);
CREATE INDEX subject_engine_runs_actor ON subject_engine_runs(actor_user_id);
-- Reader context is transaction local. Explicit SQL predicates remain mandatory
-- even for privileged operator connections; FORCE also covers table-owner roles.
CREATE FUNCTION identity_account() RETURNS uuid LANGUAGE sql STABLE AS
$$ SELECT NULLIF(current_setting('app.account_id',true),'')::uuid $$;
CREATE FUNCTION identity_subject_read(resource uuid) RETURNS boolean LANGUAGE sql STABLE AS $$
 SELECT EXISTS(SELECT 1 FROM subject_grants g JOIN user_account_state a ON a.user_id=g.grantee_user_id
 WHERE g.subject_id=resource AND g.grantee_user_id=identity_account() AND g.revoked_at IS NULL AND a.state='active')
$$;
ALTER TABLE subject_profile_revisions ENABLE ROW LEVEL SECURITY;
ALTER TABLE subject_profile_revisions FORCE ROW LEVEL SECURITY;
CREATE POLICY subject_profile_read ON subject_profile_revisions FOR SELECT USING(identity_subject_read(subject_id));
CREATE POLICY subject_profile_write ON subject_profile_revisions FOR INSERT WITH CHECK(
 changed_by=identity_account() AND EXISTS(SELECT 1 FROM subject_grants g JOIN user_account_state a ON a.user_id=g.grantee_user_id
 WHERE g.subject_id=subject_profile_revisions.subject_id AND g.grantee_user_id=identity_account() AND g.can_edit AND g.revoked_at IS NULL AND a.state='active'));
ALTER TABLE subject_engine_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE subject_engine_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY subject_runs_read ON subject_engine_runs FOR SELECT USING(identity_subject_read(subject_id));
CREATE POLICY subject_runs_write ON subject_engine_runs FOR INSERT WITH CHECK(actor_user_id=identity_account() AND identity_subject_read(subject_id));
