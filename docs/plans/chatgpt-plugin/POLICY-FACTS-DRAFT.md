# Plugin policy preparation — facts and unresolved decisions

Draft preparation material only. This is not a published privacy policy or accepted terms, and must not be used as a verified listing URL.

## Verified product behavior to explain

The proposed plugin helps an authenticated person discover Selemene engines, prepare structured inputs and request symbolic or calculated readings. It does not provide medical diagnosis, guaranteed predictions, purchases, camera capture or payment administration through the initial tool surface. Commerce declarations still require publisher confirmation.

The person links their own Selemene API key through a browser consent form. The proposed Cloudflare OAuth adapter validates the key against the account endpoint and uses encrypted OAuth grant properties for delegated use. Final policy text must be checked against the implemented and deployed version before publication.

Reading inputs can include a name, birth date, birth time, location coordinates and timezone, depending on the selected engine. The existing Selemene API attempts asynchronous, best-effort storage of reading inputs/results, may populate the user's profile, records usage, and updates progression. A transient ChatGPT interaction does not imply transient Selemene storage.

Requests cross ChatGPT/OpenAI, the Cloudflare adapter and Selemene's Railway API. Railway's database/cache and relevant engine services process the request. Do not promise that all processors have identical retention or no logging unless separately verified. The initial tool allowlist is designed to exclude image/audio capture and generative media; test this before saying no third-party model is invoked.

OAuth disconnection and upstream API-key revocation stop or invalidate future access, but do not themselves delete existing Selemene reading/history/profile records. Precise expiry and revocation behavior must be verified in the working adapter. Do not promise instantaneous distributed revocation without testing provider consistency.

## Publisher decisions still required

- Verified individual or legal business responsible for the plugin.
- Support contact and procedure for access, correction or deletion requests.
- Reading/profile retention duration and whether deletion is automatic or request-based.
- Backup and operational-log retention, including deletion limitations.
- Actual privacy and terms URLs, or approval of new hosted pages using finalized text.
- Intended countries, age/audience restrictions and commerce declaration.

## Evidence before publication

Confirm the final adapter does not log API keys or personal inputs, verify a user-specific revocation test, and run a two-user isolation test. Review the final backend data behavior against the policy text. The authorized developer completes legal/policy attestations in the submission portal; technical tests cannot attest to business practices or legal agreements.

## Lifecycle audit addendum (2026-09-30)

See DATA-LIFECYCLE-AUDIT.md for source references and fresh provider metadata. No general account/reading deletion endpoint or automatic reading/profile/usage purge was found. Profile updates and reading list/detail endpoints exist. The daily 30-day revoked-key purge in source does not establish a reading retention policy. Railway reports Postgres PITR disabled and empty Postgres/Redis backup schedules; the listed historical manual backup is past its stated expiry, which is not proof of erasure or restorability. API request middleware logs metadata including user IDs; wider provider/telemetry retention remains unverified. These are implementation facts, not business commitments.
