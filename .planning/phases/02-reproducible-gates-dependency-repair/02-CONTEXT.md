# Phase 2 context

## Boundary

Continue original #901 Wave 1 and the remaining #896 registry/asset/release-receipt authority. This phase prepares a reviewable candidate and proves its gates before any production-triggering merge. It does not expand engine semantics.

## Recommended discussion decisions

User authorized recommended technical defaults and critical HITL only. One discussion pass applies:

- Preserve existing PR branches and their commits. Repair the TS formatting failure independently, then assemble one draft recovery PR to main containing the capability stack and scoped dependency/CI/infra commits. This gives a single reviewable promotion candidate; it does not authorize promotion.
- Prefer supported same-major dependency updates and semver-compatible transitive overrides. Use a verified major update when the audit requires it: Vitest 3.2.6 and the shadcn 4.21.0 development CLI are explicitly reviewed with affected suites. Never waive failures or force an untested major change.
- Replace private-path test dependencies with repository fixtures. Test browser installation is local verification, not a production operation.
- Pin third-party Actions to verified commit SHAs, add admin and full production/development audit jobs, and require complete Python verification. Do not call a skipped job proof.
- Match Railway watch semantics and explicit Cloudflare 9d9d account ownership in source; record source versus effective deployment separately.
- Prepare exact check names, candidate commits, deployment targets and rollback requirements before the critical merge/security decision. Read-only provider inspection remains authorized.

## Deferred

Native/conditional/Python capability parity is Phase 3. The 570 per-engine issues, production schema mutations, provider-generated media, resource creation, credentials and deployment are not silently accepted here.
