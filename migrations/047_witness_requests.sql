-- Owner-approved, hash-pinned admission for a verified account to request its
-- individual Daily Witness. No human API role may create or widen admissions.
CREATE TABLE witness_request_admissions (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
 subject_id uuid NOT NULL,
 owner_user_id uuid NOT NULL,
 requester_user_id uuid NOT NULL,
 worker_user_id uuid NOT NULL REFERENCES users(id),
 identity_issuer text NOT NULL,
 identity_subject text NOT NULL,
 verified_email text NOT NULL CHECK(verified_email=lower(btrim(verified_email)) AND length(verified_email)>0),
 profile_revision bigint NOT NULL,
 profile_source_hash text NOT NULL CHECK(profile_source_hash ~ '^[0-9a-f]{64}$'),
 admission_sha256 text NOT NULL CHECK(admission_sha256 ~ '^[0-9a-f]{64}$'),
 contract_sha256 text NOT NULL CHECK(contract_sha256 ~ '^[0-9a-f]{64}$'),
 approved_at timestamptz NOT NULL DEFAULT now(),
 revoked_at timestamptz,
 UNIQUE(id,subject_id,requester_user_id,worker_user_id),
 FOREIGN KEY(subject_id,owner_user_id) REFERENCES corpus_subjects(id,owner_user_id),
 FOREIGN KEY(subject_id,profile_revision) REFERENCES subject_profile_revisions(subject_id,revision),
 FOREIGN KEY(identity_issuer,identity_subject,requester_user_id) REFERENCES account_identities(issuer,provider_subject,user_id),
 CHECK(worker_user_id<>requester_user_id AND worker_user_id<>owner_user_id)
);
CREATE UNIQUE INDEX witness_request_admissions_active ON witness_request_admissions(subject_id) WHERE revoked_at IS NULL;
CREATE UNIQUE INDEX witness_request_admissions_active_requester ON witness_request_admissions(requester_user_id) WHERE revoked_at IS NULL;
CREATE INDEX witness_request_admissions_requester ON witness_request_admissions(requester_user_id);
CREATE INDEX witness_request_admissions_worker ON witness_request_admissions(worker_user_id);

CREATE FUNCTION witness_request_admission_immutable() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF (to_jsonb(NEW)-'revoked_at') IS DISTINCT FROM (to_jsonb(OLD)-'revoked_at')
    OR (OLD.revoked_at IS NOT NULL AND NEW.revoked_at IS DISTINCT FROM OLD.revoked_at) THEN
  RAISE EXCEPTION 'Witness admission pins are immutable; revoke and admit a new revision';
 END IF;
 RETURN NEW;
END
$$;
CREATE TRIGGER witness_request_admission_immutable
 BEFORE UPDATE ON witness_request_admissions
 FOR EACH ROW EXECUTE FUNCTION witness_request_admission_immutable();

CREATE TABLE witness_requests (
 id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
 admission_id uuid NOT NULL,
 subject_id uuid NOT NULL,
 requester_user_id uuid NOT NULL,
 worker_user_id uuid NOT NULL,
 report_date date NOT NULL,
 status text NOT NULL DEFAULT 'queued' CHECK(status IN ('queued','running','held','accepted')),
 created_at timestamptz NOT NULL DEFAULT now(),
 updated_at timestamptz NOT NULL DEFAULT now(),
 lease_sha256 text CHECK(lease_sha256 ~ '^[0-9a-f]{64}$'),
 lease_expires_at timestamptz,
 inference_started_at timestamptz,
 reason text CHECK(reason IN ('access_changed','source_unavailable','source_changed','provider_unavailable','invalid_report','publication_unconfirmed','request_expired','generation_failed')),
 sha256 text CHECK(sha256 ~ '^[0-9a-f]{64}$'),
 UNIQUE(subject_id,report_date),
 FOREIGN KEY(admission_id,subject_id,requester_user_id,worker_user_id)
   REFERENCES witness_request_admissions(id,subject_id,requester_user_id,worker_user_id),
 CHECK((lease_sha256 IS NULL)=(lease_expires_at IS NULL)),
 CHECK(status<>'running' OR lease_sha256 IS NOT NULL),
 CHECK((status='held')=(reason IS NOT NULL)),
 CHECK((status='accepted')=(sha256 IS NOT NULL))
);
CREATE INDEX witness_requests_worker_queue ON witness_requests(worker_user_id,created_at) WHERE status='queued';
CREATE INDEX witness_requests_worker_running ON witness_requests(worker_user_id,lease_expires_at) WHERE status='running';
CREATE INDEX witness_requests_admission ON witness_requests(admission_id);

-- The API sets this transaction-local marker only after authenticating the
-- dedicated Witness service API key. It never comes from a client header.
CREATE FUNCTION witness_request_worker() RETURNS uuid LANGUAGE sql STABLE AS $$
 SELECT CASE WHEN current_setting('app.witness_worker',true)='yes' THEN identity_account() END
$$;
ALTER TABLE witness_request_admissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE witness_request_admissions FORCE ROW LEVEL SECURITY;
CREATE POLICY witness_request_admission_read ON witness_request_admissions FOR SELECT USING (
 requester_user_id=identity_account() OR worker_user_id=witness_request_worker()
);
ALTER TABLE witness_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE witness_requests FORCE ROW LEVEL SECURITY;
CREATE POLICY witness_request_read ON witness_requests FOR SELECT USING (
 requester_user_id=identity_account() OR worker_user_id=witness_request_worker()
);
CREATE POLICY witness_request_enqueue ON witness_requests FOR INSERT WITH CHECK (
 requester_user_id=identity_account() AND status='queued' AND lease_sha256 IS NULL
 AND inference_started_at IS NULL AND identity_subject_read(subject_id)
 AND EXISTS (SELECT 1 FROM witness_request_admissions a WHERE a.id=admission_id AND a.revoked_at IS NULL)
);
CREATE POLICY witness_request_work ON witness_requests FOR UPDATE USING (
 worker_user_id=witness_request_worker()
) WITH CHECK (worker_user_id=witness_request_worker());

-- The runtime locks this admission row during validation without the ability to
-- mutate it. PostgreSQL requires column UPDATE privilege to take a row lock.
CREATE POLICY witness_request_admission_lock ON witness_request_admissions FOR UPDATE USING (
 requester_user_id=identity_account() OR worker_user_id=witness_request_worker()
) WITH CHECK (false);

-- The catalog must hold its existing publication grant through the CAS. A
-- grantee can acquire a row lock only in a verified worker transaction, while
-- the pre-existing owner policy remains the sole authority to change a grant.
CREATE POLICY witness_publish_grant_worker_lock ON witness_publish_grants FOR UPDATE USING (
 grantee_user_id=witness_request_worker()
) WITH CHECK (false);
