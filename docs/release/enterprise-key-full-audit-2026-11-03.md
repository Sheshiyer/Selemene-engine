# Original Enterprise key production audit — 3 November 2026

The original `witness-agents-service` key in `~/.claude/.env`
(`SELEMENE_API_KEY`) successfully calculated all **19 registered engines** and
executed all **six workflows** against the production API, including the public
`selemene.tryambakam.space` hostname with an explicit audit client User-Agent. Its secret was not
changed, rotated, or copied into this repository. A SHA-256 lookup matched the
original production record. It remains the sole active key, with Enterprise tier,
all 19 engine scopes plus `basic:access`, access level 5, and rate limit 10,000.
Production retains `ENGINE_SCOPE_MODE=enforce`. Admin access comes from the
existing owner roles; tier selection does not grant new admin roles.

## Live calculation evidence

All 19 direct calculations returned HTTP 200 with results:

`biofield`, `biofield-capture`, `biorhythm`, `enneagram`, `face-reading`,
`financial-biosensor`, `gene-keys`, `human-design`, `i-ching`, `nadabrahman`,
`numerology`, `panchanga`, `raaga`, `sacred-geometry`, `sigil-forge`, `tarot`,
`transits`, `vedic-clock`, and `vimshottari`.

The capture probe created an owned session, checked consent enforcement, uploaded
a synthetic PNG (not a human photograph), calculated from its persisted reading,
and closed the session. Consent omission returned the expected 422; capture with
explicit consent returned 201 and the capture engine returned available=true.

Each workflow returned HTTP 200 with its full declared engine membership:

| Workflow | Engine results |
| --- | ---: |
| birth-blueprint | 5/5 |
| creative-expression | 5/5 |
| daily-practice | 5/5 |
| decision-support | 5/5 |
| full-spectrum | 17/17 |
| self-inquiry | 4/4 |

The full-spectrum workflow intentionally declares 17 engines; the separate
19-engine probe covers Biofield Capture and Financial Biosensor too.

## Tooling repairs and verification

The initial audit covered 44 read routes across profile/usage/history, keys,
tiers, admin analytics, system health, bridge/capabilities, skills, observability,
billing status, Biofield sessions and Witness archives. Forty-three returned
200. Baseline listing returned 500 because the production Supabase-era
`biofield_baselines` table lacked `notes`: migration 018's `CREATE TABLE IF NOT
EXISTS` had preserved its legacy shape without adding the column.

An additional reading-job lookup returned 503 because `asset_jobs` did not exist.
Both environments' migration journals stopped at 041; neither had the desktop
session or reading-job tables. After validating every recorded checksum, the
normal migration runner applied exactly these pending migrations, staging first
and then production (3 applied, 40 already-applied files skipped):

- 042: existing desktop-session tables and indexes.
- 043: existing background reading-job table and index.
- 044: additive, idempotent `notes TEXT` compatibility column for legacy baselines.

The new migration preserves legacy columns, defaults, rows and dependencies.
Staging already had `notes`, so 044 was a no-op there. Journals now match every
repository checksum through 044. No historical migration was rewritten.

Using the original production key after repair:

- Baseline list: 200; synthetic baseline creation: 201 with one reading.
- Reading with baseline comparison: 200; JSON baseline export: 201.
- Missing job lookup: expected 404 `JOB_NOT_FOUND`, replacing the store error.
- Invalid desktop grant: expected 400 `INVALID_GRANT`, with the store available.
- Vedic chart bundle, seeded integrated assets and Witness interpretation: 200.
- Non-writable job mode: expected 422 `MODE_NOT_WRITABLE`, without launching a job.
- Admin session and API liveness: 200; original key still visible and sole active key.

All 44 audited read routes were subsequently rechecked through the public API
hostname and returned 200.

The local PostgreSQL regression reproduces the legacy shape, checks baseline
insert/list behavior and legacy JSON preservation, repeats migration 044, and
checks preservation of existing notes on the canonical schema. The migration
validator and migration-runner tests passed (20 tests). Witness library tests
passed (34 tests), including serialization of the provider-specific thinking
option and its omission for other providers.

## Enterprise NVIDIA execution repair

The initial Witness request returned `llm_powered=false`. Production logs
identified NVIDIA HTTP 410 responses for retired Enterprise model IDs. The live
provider catalog and a completion probe confirmed the supported
`nvidia/nemotron-3.5-super-vl-120b-a12b` endpoint. The provider reported its served
model as `private/nvidia/nemotron-3.5-super-120b-a12b`.

Enterprise calls to NVIDIA now use that endpoint, retaining role prompts and
token budgets. Other providers and tiers keep their previous selection. The
first staging probe became LLM-powered but left the question empty because the
reasoning consumed the answer budget. The client therefore sends NVIDIA's
documented `chat_template_kwargs.enable_thinking=false` for this Enterprise
path. A direct synthesis probe with that option returned nonempty synthesis and
question JSON with no reasoning content. This option is omitted for other
providers. Provider credentials, provider selection settings and OmniRoute
configuration are unchanged.

Final promotion, serving deployment and full response verification are recorded
with the private operation receipts. This report distinguishes the initial
production evidence from subsequent runtime promotion.

```sh
psql -X -v ON_ERROR_STOP=1 -d <test-database> \
  -f tests/migrations/044_biofield_baselines_reconcile_legacy_shape.sql
python3 scripts/validate_migrations.py
python3 -m pytest tests/scripts/test_validate_migrations.py tests/scripts/test_apply_migrations.py -q
```

## Limits of this acceptance

No engine-scope denial occurred in the 19 calculations or six workflows. These
results establish calculation and tooling access, with these execution limits:

- Face Reading used its declared birth-physiognomy fallback. Real face capture
  was not supplied.
- Sigil Forge used symbolic mode with image generation disabled. Raaga clip
  generation was not requested. Paid/optional media generation is not verified.
- The initial Witness fallback was a provider execution failure, separate from
  Selemene key authorization; the runtime repair is described above. The initial
  audit did not launch a multi-pass model-written job.
- Seeded assets passed; their model-written source-pack gate remains correctly
  blocked because those sections were not written by a model.
- Hermes status reports not configured. Billing remains in its existing free
  mode with payment actions disabled. These boundaries were not enabled or
  bypassed. Desktop authorization continues to require an interactive browser
  identity rather than an API key.

## Recovery and evidence

Synthetic reading, capture, baseline and export records remain as ordinary QA
history. Existing user records and local edits were preserved. These additive
schema changes require no runtime deployment; the NVIDIA client patch does.
Retain the new column/tables once
they contain writes; dropping them would discard data and is not an automatic
rollback step.

Private schema snapshots, journal checksums, engine/workflow responses, tooling
responses and operation logs are under
`~/.codex/private/selemene-enterprise-full-audit-2026-11-03/` with restricted
permissions. They exclude the original secret and database credentials. The
initial engine/workflow runtime was main commit
`7f03823fda58526c95d7c08f19112b5ff98c10d3`.
