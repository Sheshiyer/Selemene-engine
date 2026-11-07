# Witness catalog integration contract

Migration `046_witness_catalog.sql` follows shared identity 045. Metadata only: no report bodies, private names, birth facts, email seeds or R2 credentials. Upload and verify immutable private bytes before publishing metadata. The API cannot prove R2 object existence or review provenance; the admitted operator pipeline must do both.

Module additions owned by the core integration worker:

- `noesis-data/src/models/mod.rs`: `pub mod witness_catalog;`
- `noesis-data/src/repositories/mod.rs`: `pub mod witness_catalog_repository;`
- `noesis-api/src/handlers/mod.rs`: `pub mod witness_catalog;`
- Shared identity helpers `transaction` and `subject_access` exposed `pub(crate)`.
- Authenticated router: `.route("/subjects/:id/witness", get(handlers::witness_catalog::catalog).post(handlers::witness_catalog::publish))` under `/api/v1`.
- Append migration046 checksum to the authoritative ledger only after final review; this worker does not edit the ledger.

## Read

`GET /api/v1/subjects/:subject_id/witness` requires a current active account and current scoped read grant. It returns private/no-store:

```json
{"schema":"noesis.witness-catalog.v1","subject_id":"uuid","folio":null,"daily":{}}
```

Each populated entry has `date`, `kind`, `sha256`, `object_key`, `report_schema`, `profile_key`, `packet_sha256`, `contract_sha256`, `approval`, `published_at` and `head_revision`. Daily entries are keyed by date. Folio is the latest dated accepted folio. A catalog grant does not authorize broader archive or relationship reads. A downstream private object reader must recheck this catalog/current grant and resolve only its accepted entry.

## Publication and conditional rollback

`POST` on the same route requires the same current readable subject plus either resource ownership or an active dedicated `witness_publish_grants` row. Ordinary read/edit/calculate grants and platform role labels do not confer publication. Publisher grants have composite owner FKs and can only be issued by their subject's resource owner.

The exact body is:

```json
{
  "expected_head": null,
  "artifact": {
    "profile": "synthetic-subject",
    "kind": "daily",
    "date": "2026-11-07",
    "sha256": "64 lowercase hex characters",
    "object_key": "witness-v2/<subject_uuid>/synthetic-subject/daily/2026-11-07/<sha256>.json",
    "report_schema": "noesis.witness-report.v2",
    "packet_sha256": "64 lowercase hex characters",
    "contract_sha256": "64 lowercase hex characters",
    "effective_at": "2026-11-07T00:30:00Z"
  },
  "approval": "owner"
}
```

`expected_head` is required. Initial publication uses explicit null; replacement uses `{ "sha256": "<current hash>", "revision": <current head_revision> }`. `kind` is daily or folio, and approval is owner or automatic. The profile must exactly equal the subject's stored key. The date must be canonical and its effective anchor exactly 00:30 UTC (06:00 IST). Keys are strictly derived from subject UUID/profile/kind/date/hash and end in `.json`; accepted object locators cannot be reused across subjects/owners. `report_schema` accepts v1 or v2 so unchanged reviewed v1 bytes can be copied into the scoped immutable prefix before import. No schema/content rewrite is performed here. The generation descriptor remains frozen under its admitted contract. The delivery adapter copies the exact reviewed bytes into the UUID-scoped immutable key and replaces only `artifact.object_key`; it omits `bytes` from the JSON request. It never rewrites report bodies, hashes, generation receipts or review records.

Success returns `schema: noesis.witness-publication.v1`, subject ID, accepted `entry`, `idempotent`, and the head observed before the request as `previous_head`. Repeating an identical successful request is idempotent while its accepted head is still current. A stale unrelated head returns 409. Revisions increase on each head change; returning to an earlier hash cannot make a stale request valid (ABA protection). Rollback POSTs the original immutable artifact and original approval with the exact current hash/revision; the old version is reused, the new head revision is recorded, and immutable prior acceptance history remains.

Versions and acceptance events reject update/delete. Transaction-scoped account context, explicit repository predicates, FORCE RLS and composite FKs protect reads/writes, including privileged repository connections. Locks are limited to metadata transactions; no network upload occurs under a DB lock. A successful publication is the single DB accepted head, with no second writable R2 index.

## Local validation

Run against the isolated database with 045 and 046 applied:

```sh
WITNESS_TEST_DATABASE_URL=postgresql://sheshnarayaniyer@127.0.0.1:55437/noesis_identity_test cargo test -p noesis-data --test witness_catalog_integration -j 2
```

Tests create synthetic rows without truncating shared tables. Role/grant/ownership changes for the non-superuser FORCE-RLS check are transactionally rolled back. Accepted synthetic versions remain immutable in the isolated test database. Production role grants, deployed API authentication, real immutable R2 bytes, owner review, cross-surface reads and scheduled delivery remain separate live gates.
