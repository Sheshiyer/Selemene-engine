-- Metadata-only accepted Witness catalog. Bodies remain immutable private R2 objects.
-- 045 is the authority for account state and subject grants. No personal seeds.
CREATE UNIQUE INDEX corpus_subjects_identity_key ON corpus_subjects(id,owner_user_id,subject_key);
CREATE TABLE witness_publish_grants (
 subject_id uuid NOT NULL, owner_user_id uuid NOT NULL, grantee_user_id uuid NOT NULL REFERENCES users(id),
 grantor_user_id uuid NOT NULL REFERENCES users(id), created_at timestamptz NOT NULL DEFAULT now(), revoked_at timestamptz,
 PRIMARY KEY(subject_id,grantee_user_id),
 FOREIGN KEY(subject_id,owner_user_id) REFERENCES corpus_subjects(id,owner_user_id),
 CHECK(grantor_user_id=owner_user_id)
);
CREATE INDEX witness_publish_grants_grantee ON witness_publish_grants(grantee_user_id,subject_id) WHERE revoked_at IS NULL;
CREATE INDEX witness_publish_grants_owner ON witness_publish_grants(owner_user_id);
CREATE INDEX witness_publish_grants_grantor ON witness_publish_grants(grantor_user_id);
ALTER TABLE witness_publish_grants ENABLE ROW LEVEL SECURITY;
ALTER TABLE witness_publish_grants FORCE ROW LEVEL SECURITY;
CREATE POLICY witness_publish_grant_read ON witness_publish_grants FOR SELECT USING(
 (owner_user_id=(SELECT identity_account()) OR grantee_user_id=(SELECT identity_account()))
 AND EXISTS(SELECT 1 FROM user_account_state a WHERE a.user_id=identity_account() AND a.state='active'));
CREATE POLICY witness_publish_grant_insert ON witness_publish_grants FOR INSERT WITH CHECK(
 owner_user_id=(SELECT identity_account()) AND grantor_user_id=(SELECT identity_account()) AND identity_subject_read(subject_id));
CREATE POLICY witness_publish_grant_update ON witness_publish_grants FOR UPDATE USING(
 owner_user_id=(SELECT identity_account()) AND identity_subject_read(subject_id)) WITH CHECK(
 owner_user_id=(SELECT identity_account()) AND grantor_user_id=(SELECT identity_account()) AND identity_subject_read(subject_id));

CREATE FUNCTION witness_subject_read(resource uuid) RETURNS boolean LANGUAGE sql STABLE AS $$
 SELECT EXISTS(SELECT 1 FROM corpus_subjects s WHERE s.id=resource AND s.archived_at IS NULL AND identity_subject_read(s.id))
$$;
CREATE FUNCTION witness_subject_publish(resource uuid) RETURNS boolean LANGUAGE sql STABLE AS $$
 SELECT EXISTS(SELECT 1 FROM corpus_subjects s WHERE s.id=resource AND s.archived_at IS NULL
 AND identity_subject_read(s.id) AND (s.owner_user_id=identity_account() OR EXISTS(
 SELECT 1 FROM witness_publish_grants p WHERE p.subject_id=s.id AND p.owner_user_id=s.owner_user_id
 AND p.grantee_user_id=identity_account() AND p.revoked_at IS NULL)))
$$;

CREATE TABLE witness_versions (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), subject_id uuid NOT NULL, owner_user_id uuid NOT NULL,
 profile_key varchar(128) NOT NULL CHECK(profile_key ~ '^[a-z0-9][a-z0-9_-]{0,127}$'),
 kind text NOT NULL CHECK(kind IN ('daily','folio')), report_date date NOT NULL CHECK(report_date BETWEEN DATE '1900-01-01' AND DATE '9999-12-31'),
 sha256 text NOT NULL CHECK(sha256 ~ '^[0-9a-f]{64}$'),
 object_key text NOT NULL UNIQUE,
 report_schema text NOT NULL CHECK(report_schema IN ('noesis.witness-report.v1','noesis.witness-report.v2')),
 packet_sha256 text NOT NULL CHECK(packet_sha256 ~ '^[0-9a-f]{64}$'),
 contract_sha256 text NOT NULL CHECK(contract_sha256 ~ '^[0-9a-f]{64}$'),
 effective_at timestamptz NOT NULL,
 approval text NOT NULL CHECK(approval IN ('owner','automatic')),
 published_by uuid NOT NULL REFERENCES users(id), published_at timestamptz NOT NULL DEFAULT now(),
 FOREIGN KEY(subject_id,owner_user_id,profile_key) REFERENCES corpus_subjects(id,owner_user_id,subject_key),
 UNIQUE(subject_id,kind,report_date,sha256),
 UNIQUE(id,subject_id,owner_user_id,kind,report_date),
 CONSTRAINT witness_versions_object_scope_check CHECK(object_key='witness-v2/'||subject_id::text||'/'||profile_key||'/'||kind||'/'||to_char(report_date,'YYYY-MM-DD')||'/'||sha256||'.json'),
 CHECK(effective_at=(report_date::timestamp + interval '30 minutes') AT TIME ZONE 'UTC')
);
CREATE INDEX witness_versions_owner ON witness_versions(owner_user_id);
CREATE INDEX witness_versions_publisher ON witness_versions(published_by);
CREATE TABLE witness_heads (
 subject_id uuid NOT NULL, owner_user_id uuid NOT NULL, kind text NOT NULL, report_date date NOT NULL,
 version_id uuid NOT NULL, previous_version_id uuid, revision bigint NOT NULL CHECK(revision>0),
 updated_by uuid NOT NULL REFERENCES users(id), updated_at timestamptz NOT NULL DEFAULT now(),
 PRIMARY KEY(subject_id,kind,report_date),
 FOREIGN KEY(version_id,subject_id,owner_user_id,kind,report_date) REFERENCES witness_versions(id,subject_id,owner_user_id,kind,report_date),
 FOREIGN KEY(previous_version_id,subject_id,owner_user_id,kind,report_date) REFERENCES witness_versions(id,subject_id,owner_user_id,kind,report_date)
);
CREATE INDEX witness_heads_version ON witness_heads(version_id);
CREATE INDEX witness_heads_previous ON witness_heads(previous_version_id);
CREATE INDEX witness_heads_owner ON witness_heads(owner_user_id);
CREATE INDEX witness_heads_actor ON witness_heads(updated_by);
CREATE TABLE witness_head_events (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(), subject_id uuid NOT NULL, owner_user_id uuid NOT NULL,
 kind text NOT NULL, report_date date NOT NULL, revision bigint NOT NULL CHECK(revision>0),
 version_id uuid NOT NULL, previous_version_id uuid, actor_user_id uuid NOT NULL REFERENCES users(id),
 accepted_at timestamptz NOT NULL DEFAULT now(), UNIQUE(subject_id,kind,report_date,revision),
 FOREIGN KEY(version_id,subject_id,owner_user_id,kind,report_date) REFERENCES witness_versions(id,subject_id,owner_user_id,kind,report_date),
 FOREIGN KEY(previous_version_id,subject_id,owner_user_id,kind,report_date) REFERENCES witness_versions(id,subject_id,owner_user_id,kind,report_date)
);
CREATE INDEX witness_head_events_version ON witness_head_events(version_id);
CREATE INDEX witness_head_events_previous ON witness_head_events(previous_version_id);
CREATE INDEX witness_head_events_owner ON witness_head_events(owner_user_id);
CREATE INDEX witness_head_events_actor ON witness_head_events(actor_user_id);

ALTER TABLE witness_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE witness_versions FORCE ROW LEVEL SECURITY;
CREATE POLICY witness_version_read ON witness_versions FOR SELECT USING(witness_subject_read(subject_id));
CREATE POLICY witness_version_insert ON witness_versions FOR INSERT WITH CHECK(published_by=(SELECT identity_account()) AND witness_subject_publish(subject_id));
ALTER TABLE witness_heads ENABLE ROW LEVEL SECURITY;
ALTER TABLE witness_heads FORCE ROW LEVEL SECURITY;
CREATE POLICY witness_head_read ON witness_heads FOR SELECT USING(witness_subject_read(subject_id));
CREATE POLICY witness_head_insert ON witness_heads FOR INSERT WITH CHECK(updated_by=(SELECT identity_account()) AND witness_subject_publish(subject_id));
CREATE POLICY witness_head_update ON witness_heads FOR UPDATE USING(witness_subject_publish(subject_id)) WITH CHECK(updated_by=(SELECT identity_account()) AND witness_subject_publish(subject_id));
ALTER TABLE witness_head_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE witness_head_events FORCE ROW LEVEL SECURITY;
CREATE POLICY witness_event_read ON witness_head_events FOR SELECT USING(witness_subject_read(subject_id));
CREATE POLICY witness_event_insert ON witness_head_events FOR INSERT WITH CHECK(actor_user_id=(SELECT identity_account()) AND witness_subject_publish(subject_id));

CREATE FUNCTION witness_immutable_row() RETURNS trigger LANGUAGE plpgsql AS $$
 BEGIN RAISE EXCEPTION 'Witness accepted versions and history are immutable' USING ERRCODE='55000'; END
$$;
CREATE TRIGGER witness_version_immutable BEFORE UPDATE OR DELETE ON witness_versions FOR EACH ROW EXECUTE FUNCTION witness_immutable_row();
CREATE TRIGGER witness_event_immutable BEFORE UPDATE OR DELETE ON witness_head_events FOR EACH ROW EXECUTE FUNCTION witness_immutable_row();
