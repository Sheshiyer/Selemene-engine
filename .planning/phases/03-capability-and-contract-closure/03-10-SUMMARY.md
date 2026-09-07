---
phase: 03-capability-and-contract-closure
plan: "10"
status: complete
---

# Plan 03-10 Summary: Rust SDK and TUI Capability Projection

The Rust SDK now reads the authenticated canonical capability envelope from
`GET /api/v1/engines/capabilities`. API-key requests use exactly `X-API-Key`,
bearer requests use exactly `Authorization: Bearer`, and non-success responses
are reduced to bounded status errors. The legacy engine list remains available
as an adapter while the typed client validates the v1 envelope before returning
it.

The TUI projects all 19 runtime identities, preserves the four capability
states and safe reasons, derives connected state from a successful authenticated
capability response, and blocks unavailable or unsupported engine dispatch.
Workflow selection retains the five producer-backed workflows and renders Full
Spectrum visibly as unsupported without dispatching it.

## Verification

- `cargo test -p noesis-sdk -p noesis-tui --locked` — 37 SDK tests passed (one ignored) and 2 TUI tests passed.
- The wiremock receipt covered the authenticated canonical route, exact API-key header, 19 rows and 17 public mirrors.
- `git diff --check` remains required after final gate wiring.
