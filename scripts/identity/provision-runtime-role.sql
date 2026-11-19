-- Run with the migration principal AFTER root migrations. No credentials here.
-- Set password privately with \password noesis_api_runtime (or secrets manager).
BEGIN;
DO $$ BEGIN
 IF NOT EXISTS(SELECT 1 FROM pg_roles WHERE rolname='noesis_api_runtime') THEN
  CREATE ROLE noesis_api_runtime LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
 END IF;
 IF EXISTS(SELECT 1 FROM pg_auth_members WHERE member=(SELECT oid FROM pg_roles WHERE rolname='noesis_api_runtime')) THEN
  RAISE EXCEPTION 'runtime role has unexpected memberships; inspect before provisioning';
 END IF;
END $$;
ALTER ROLE noesis_api_runtime NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA public TO noesis_api_runtime;
DO $$ DECLARE t record; BEGIN
 EXECUTE format('GRANT CONNECT ON DATABASE %I TO noesis_api_runtime',current_database());
 -- Preserve existing legacy API DML while excluding migration/schema authority.
 FOR t IN SELECT c.relname FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
   WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname NOT IN ('noesis_migration_journal','witness_request_admissions','witness_requests')
 LOOP EXECUTE format('GRANT SELECT,INSERT,UPDATE,DELETE ON TABLE public.%I TO noesis_api_runtime',t.relname); END LOOP;
 IF to_regclass('public.witness_requests') IS NOT NULL THEN
  REVOKE ALL ON public.witness_request_admissions, public.witness_requests FROM noesis_api_runtime;
  GRANT SELECT, UPDATE(revoked_at) ON public.witness_request_admissions TO noesis_api_runtime;
  GRANT SELECT, INSERT(admission_id,subject_id,requester_user_id,worker_user_id,report_date),
    UPDATE(status,updated_at,lease_sha256,lease_expires_at,inference_started_at,reason,sha256)
    ON public.witness_requests TO noesis_api_runtime;
 END IF;
 IF to_regclass('public.noesis_migration_journal') IS NOT NULL THEN
  GRANT SELECT ON public.noesis_migration_journal TO noesis_api_runtime;
 END IF;
 IF has_schema_privilege('noesis_api_runtime','public','CREATE') THEN
  RAISE EXCEPTION 'PUBLIC or runtime grants schema CREATE; remove with separately reviewed DB hardening';
 END IF;
 IF EXISTS(SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relowner=(SELECT oid FROM pg_roles WHERE rolname='noesis_api_runtime')) THEN
  RAISE EXCEPTION 'runtime owns schema objects; migrate ownership to migration principal';
 END IF;
END $$;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO noesis_api_runtime;
COMMIT;
