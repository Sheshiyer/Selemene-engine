-- Email authorizations are redeemed through the verified Daily sign-in, with
-- legacy token claims retaining their existing target, expiry and consent.
ALTER TABLE subject_enrollments
 ADD COLUMN mode text NOT NULL DEFAULT 'claim',
 ADD COLUMN sharing_reader_accepted_at timestamptz,
 ALTER COLUMN subject_id DROP NOT NULL,
 ALTER COLUMN token_hash DROP NOT NULL,
 ALTER COLUMN expires_at DROP NOT NULL,
 ADD CONSTRAINT subject_enrollments_mode_shape CHECK (
   (mode='claim' AND subject_id IS NOT NULL AND token_hash IS NOT NULL AND expires_at IS NOT NULL)
   OR (mode='sign_in' AND intended_email IS NOT NULL AND token_hash IS NULL AND expires_at IS NULL
       AND (subject_id IS NOT NULL OR sharing_reader_id IS NULL))
 );

CREATE UNIQUE INDEX subject_enrollments_pending_email
 ON subject_enrollments(intended_email)
 WHERE mode='sign_in' AND confirmed_at IS NULL AND revoked_at IS NULL;
CREATE UNIQUE INDEX subject_enrollments_pending_subject
 ON subject_enrollments(subject_id)
 WHERE mode='sign_in' AND subject_id IS NOT NULL AND confirmed_at IS NULL AND revoked_at IS NULL;
CREATE INDEX subject_enrollments_recipient_sharing
 ON subject_enrollments(claimed_by)
 WHERE mode='sign_in' AND confirmed_at IS NOT NULL AND revoked_at IS NULL;

ALTER TABLE subject_enrollment_relationships ADD COLUMN accepted_at timestamptz;
ALTER TABLE registered_auth_codes ADD COLUMN enrollment_email text
 CHECK (enrollment_email=lower(btrim(enrollment_email)));
