# Report phase authority repair

Date: 2026-09-30

## Change

`POST /api/v1/witness/interpret` and `POST /api/v1/assets/generate` previously selected
`max(request.consciousness_level, authenticated_user.consciousness_level)`.
A phase-0 user could therefore ask for phase 5 and reach restricted engines.

Both handlers now derive the effective engine phase exclusively from
`AuthUser.consciousness_level`, matching the existing single-engine calculate
handler. Witness primary and partner calculations, witness context/persistence,
and asset register selection all use this authenticated value.

The legacy JSON `consciousness_level` field remains accepted and is explicitly
documented as ignored. An omitted field, a lower value, a higher value, and a
representable out-of-range legacy value (255) cannot alter authorization. This
patch does not introduce a new request rejection rule or change the trusted
identity's phase normalization policy. Existing `u8` JSON decoding still applies.

## Verification

Added `crates/noesis-api/tests/report_phase_authority_tests.rs`. It exercises real
JWT authentication middleware, both report handlers and the orchestrator gate
through the Axum router. A synthetic phase-0 panchanga engine and phase-3
numerology engine record actual executions. Assertions prove that the public
engine still executes while the gated engine cannot execute for phase-0 users,
including witness partner calls. Positive controls prove phase-5 authenticated
users retain access when the request phase is lower or omitted. Asset register
and engine output membership are checked as well.

The single test covers ten HTTP request scenarios (five per endpoint):

| Authenticated phase | Legacy request phase | Expected gated execution |
| --- | --- | --- |
| 0 | 5 | None |
| 0 | 255 | None |
| 0 | Omitted | None |
| 5 | 0 | Allowed |
| 5 | Omitted | Allowed |

Actual command, run from the isolated plugin worktree:

```sh
CARGO_TARGET_DIR=/Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine/target cargo test -p noesis-api --test report_phase_authority_tests -- --nocapture
```

Result: **1 passed, 0 failed**, ten request scenarios, 0.19 seconds test runtime;
initial compilation completed in 2 minutes. The existing ignored target directory
was reused as a compilation cache; source changes are confined to the plugin
worktree. `rustfmt --edition 2021 --check` on the added test and `git diff --check`
also passed.

The test uses synthetic birth data, a lazy disconnected database pool, no
persistence repositories, and removes ambient witness provider keys in its
isolated test process. No real readings, paid generation or service writes occur.

## Scope and remaining deployment boundary

Production edits: only `crates/noesis-api/src/handlers/witness.rs` and
`crates/noesis-api/src/handlers/assets.rs` (10 inserted lines, 3 removed lines).
Additional files: the focused regression test and this receipt.

This verifies the local repair. No deployment, push, commit, migration or
credential change was performed. The production API retains its previous behavior
until the repair is reviewed and deployed through the normal release process.
