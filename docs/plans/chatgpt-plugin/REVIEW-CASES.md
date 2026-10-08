# Selemene Engine review cases

Status: **NOT RUN in ChatGPT or the submission portal**. Drafts authored 2026-09-30 against the initial seven-tool design. This file records execution status; the actual submission cases are in `plugins/selemene-engine/plugin.json` under `extensions.com.openai.review.test_cases` with exact single-server field names. Do not add run-result fields to that manifest.

| Case | Purpose | Status | Evidence still needed |
| --- | --- | --- | --- |
| P1 | Current account engine discovery; no hardcoded live-engine count | Not run | Actual tool arguments, returned catalog and user-visible summary |
| P2 | Incomplete numerology input preparation without calculation | Not run | Tool schema, supplied date, missing-field result, no execution calls |
| P3 | One authorized synthetic numerology reading | Not run | Connected synthetic test account, valid preflight, exactly one calculation, actual values/provenance |
| P4 | Workflow discovery/info without execution | Not run | Returned workflow ID and engine set, or truthful empty result |
| P5 | Fixed-date synthetic biorhythm reading | Not run | Exact target timestamp, one calculation, returned cycle values, symbolic framing |
| N1 | No medical diagnosis or medication selection | Not run | User-visible limitation and zero Selemene tool calls |
| N2 | No account-phase escalation | Not run | User-visible limitation and zero Selemene tool calls |
| N3 | No face/voice analysis or paid Suno generation | Not run | User-visible limitation and zero Selemene tool calls |

## Reproducible setup

Use a dedicated reviewer test account with the real permissions required by the chosen non-media engines. Use the synthetic Test Reader fixture in the manifest; never use the owner's personal birth details. Calculations can persist inputs/results and affect profile, usage, XP or phase; obtain the appropriate execution authorization and use the test account.

Review P2–P5 against the final deployed `tools/list` schema before execution. The proposed argument names are `engine_id`, `workflow_id`, `birth_data` and `current_time`; nested request envelopes and any confirmation field must match the implementation. The current backend requires the BirthData coordinate/timezone envelope even when an engine mathematically uses only a date. Numerology additionally needs a name. Do not reduce inputs to birth date alone based on marketing intuition.

P4 exercises discovery and inspection rather than assuming a wholly allowlisted workflow exists. Before release, add a separate real workflow execution check if `selemene_run_workflow` is enabled: compare expected versus returned engine IDs, report missing engines, verify side-effect annotations, and do not automatically retry. A synthetic transport test is not evidence of a connected ChatGPT case.

## Execution receipts

For each case, record the exact manifest SHA-256, server deployment revision, tool-schema revision, timestamp, host/session reference, fixture identity (no secrets), actual tool names/arguments, redacted output, user-visible response, and Passed/Failed/Blocked status. Preserve actual failures. Retry only after resolving the cause and understanding side effects; record each run separately.

Verify annotations on all seven tools have boolean `readOnlyHint`, `openWorldHint` and `destructiveHint` values matching real behavior. Calculations and workflow execution are not read-only. Verify auth/permission, missing-input, unavailable-engine and partial-workflow recovery separately from the three negative prompts, which test deliberate non-invocation.

Reviewer login URL, credentials and account-recovery details belong exclusively in secure portal access fields. Keep them out of this file, plugin source, ZIP and recording.
