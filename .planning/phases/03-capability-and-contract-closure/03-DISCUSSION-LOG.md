# Phase 3: Capability and contract closure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `03-CONTEXT.md`; this log preserves the alternatives considered.

**Date:** 2026-09-06
**Phase:** 03-capability-and-contract-closure
**Mode:** User-authorized recommended defaults (`discuss --auto`), single pass
**Areas discussed:** catalogue identity, dependency states, contract rollout, catalogue ownership, routing and validation, workflow synthesis

---

## Catalogue Identity

| Option | Description | Selected |
|---|---|---|
| Registry identity plus runtime overlay | Keep all 19 canonical IDs and overlay observed availability separately. | ✓ |
| Runtime-only list | Return only processes currently registered. | |
| Static list with booleans | Keep a manual list and attach a simple healthy flag. | |

**User's choice:** Auto-selected the recommended registry-plus-overlay model under the explicit project authorization for recommended defaults.
**Notes:** Preserve 17 public mirrors as a separately qualified count.

## Dependency States

| Option | Description | Selected |
|---|---|---|
| Typed explicit states | Use declared, available, degraded and unavailable with bounded dependency reasons. | ✓ |
| Boolean health | Collapse capability to healthy or unhealthy. | |
| Omit unavailable engines | Hide identities whose dependencies are absent. | |

**User's choice:** Auto-selected typed explicit states.
**Notes:** Required dependency uncertainty fails closed; optional partial service is degraded.

## Contract Rollout and Version Skew

| Option | Description | Selected |
|---|---|---|
| Additive v1 compatibility | Keep supported v1 inputs, add canonical adapters and reject incompatible skew explicitly. | ✓ |
| Flag-day replacement | Replace every surface at once. | |
| Permissive coercion | Guess missing or incompatible fields at runtime. | |

**User's choice:** Auto-selected additive compatibility with fail-closed version skew.
**Notes:** Canonical JSON errors and stable codes are required.

## Catalogue Ownership

| Option | Description | Selected |
|---|---|---|
| Generate or validate from `contracts/v1` | Permit presentation views while rejecting independent identity and envelope lists. | ✓ |
| Network endpoint only | Require every build-time and runtime consumer to fetch one endpoint. | |
| Manual synchronization | Retain independent lists with review guidance. | |

**User's choice:** Auto-selected repository-owned generated or validated projections.
**Notes:** Offline SDK, CLI and test consumers still need deterministic authority.

## Routing and Validation

| Option | Description | Selected |
|---|---|---|
| Prove or remove unsupported validation | Keep a validation route only when backed by real engine validation; otherwise remove the assumption explicitly. | ✓ |
| Stub successful validation | Return success without running a validator. | |
| Expand every engine validator now | Pull Phase 4 calculation semantics into this phase. | |

**User's choice:** Auto-selected prove-or-remove with cross-language routing, auth and timeout tests.
**Notes:** Rejected requests must cause zero engine or provider calls.

## Workflow Synthesis

| Option | Description | Selected |
|---|---|---|
| Wire the existing executor with explicit partial failure | Use the richer current architecture and report unsupported or failed synthesis truthfully. | ✓ |
| Preserve silent `None` | Keep empty synthesis without changing the public claim. | |
| Label fallback as LLM success | Treat an empty or fallback response as successful generation. | |

**User's choice:** Auto-selected existing-executor integration with explicit partial status.
**Notes:** Engine semantic corrections and paid generation remain out of scope.

## the agent's Discretion

- Exact code generator and build integration.
- Module split, compatibility-adapter internals and test-file layout.
- Dependency-ordered plan count and parallel waves.

## Deferred Ideas

- Engine semantics and provider/media truth: Phase 4.
- Stateful durability and production auth: Phase 5.
- Publishable package evidence: Phase 6.
- Provider deployment, DNS, monitoring and rollback: Phase 7 with critical HITL.
