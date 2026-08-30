# GitHub Issue Next Wave Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` when dispatching independent implementation tasks, or `superpowers:executing-plans` for inline execution with review checkpoints. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the live GitHub issue corpus into the next executable Selemene delivery wave without treating declared issue coverage as implementation proof.

**Architecture:** The plan keeps GitHub issues as the roadmap authority, local evidence files as implementation receipts, and live Railway/Cloudflare readback as operational receipts. Work proceeds through fail-closed gates, contract/runtime catalogue convergence, engine truth repair, then state/distribution/ops hardening.

**Tech Stack:** Rust workspace, TypeScript `ts-engines`, Python sidecars, GitHub Actions, Railway, Cloudflare Workers/KV/R2/D1/Vectorize, `gh`, `cargo audit`, `pnpm`, `bun`.

**Spec:** Live GitHub issues `#893`, `#896`, `#901`, `#894`, `#897`, `#913`, `#908`, `#914`, and the 570 open `[W3E:engine:NN]` engine issues read back on 2026-08-27.

## Global Constraints

- No remote issue edit, issue close, PR merge, push, deployment, variable read, secret read, KV read, D1 write, queue write, or R2 object read is authorized by this plan alone.
- Preserve the current dirty worktree unless a task explicitly owns the changed files.
- Source evidence, local test evidence, remote CI evidence, deployment evidence, and live runtime evidence must be reported separately.
- `Declared`, `Implemented`, `Executable`, `Integrated`, `Deployed`, and `Operational` remain separate evidence axes.
- W3E issues are detailed backlog rows, not proof that any engine is operational.
- GitHub issue closure requires local verification plus the relevant remote or live receipt named by that issue.
- The current local branch is ahead of `origin/main`; reconcile through PR flow, not direct push to `main`.

---

## 1) Status Report

### Scope and assumptions

- Repository: `Sheshiyer/Selemene-engine`.
- Default branch: `main`.
- Local branch at scan time: `codex/selemene-whitepaper-clean`.
- Local HEAD: `c96de19d3fde414cb03b95562f7a1d5c3006a8f6` (scan time). **Update 2026-08-31:** local branch has since been fast-forward merged onto `origin/main`; local HEAD is now `ae3e2cef402bd0cf28d1c7a102800d215cc5f2c2`, with zero file overlap against the uncommitted runtime-capability slice.
- Remote `origin/main`: `b0827e1a6e870277e6b86cfc1ee8cfd2fe930709` (scan time) -> now `ae3e2cef402bd0cf28d1c7a102800d215cc5f2c2` after PR #1485 (`codex/selemene-security-audit-recovery`) merged, recovering the Security Audit gate (commits `07fb220`, `1348460`, `3252b79`).
- Railway production and public health readback are still tied to `b0827e1a6e870277e6b86cfc1ee8cfd2fe930709`, not the local branch. This remains true after the sync: the live deployment has not been redeployed past this commit, so it is still on the pre-fix, pre-capability-endpoint revision.

### Repo Pulse

- Open issues: 587.
- W3E issues: 570 open issues across the canonical 19 engines and 30 slots per engine.
- Non-W3E roadmap/control issues: 17 open issues.
- Recent issue creation completed the W3E corpus through Tarot slot 30.
- Local evidence exists for gate foundation, contract authority v1, and TypeScript runtime capability adoption.

### Issue Landscape

- Roadmap anchor: `#893` sets the sequence `gates -> contracts -> engines`.
- Authority and evidence ledger: `#896`.
- Remote gates and release evidence: `#901`.
- Contract/catalogue convergence: `#894` and media-contract issue `#898`.
- Engine/media truth gaps: `#897`.
- State/auth/durability: `#913`.
- Distribution compatibility: `#908`.
- Ops/assets/deployment receipts: `#914`.
- W3E engine backlog: all 570 are open, with labels `plan-sync`, `roadmap`, `area:engine-integration`, and a canonical `engine-*` label.

### Active PR and CI Health

- PR `#907` is draft, conflicting, and dirty against `main`.
- PR `#886` is non-draft, conflicting, and has failing CI from its stale branch.
- Latest `CI - Test & Lint` on remote `main` at `b0827e1a6e870277e6b86cfc1ee8cfd2fe930709` failed (scan time).
- The failed remote CI lane was `Security Audit`; lint, TS engines, Python sidecars, workflow parity, tests, integration tests, secret scanning, and build were green in that run.
- **RESOLVED 2026-08-31:** PR #1485 (`codex/selemene-security-audit-recovery`) merged as `ae3e2cef402bd0cf28d1c7a102800d215cc5f2c2`. CI run `33048319281` (created `2026-08-27T07:05:41Z`) succeeded at that sha, with the `Security Audit` job specifically reporting success alongside every other job (`Workspace Gate`, `Lint`, `Secret Scanning`, `TS Engines`, `Python Sidecars`, `Workflow Registry Parity`, `Test`, `Integration Tests`, `Build`, `CI Gate`). Remote `main` is green as of this sha.
- `cargo audit` locally reports two `h2` vulnerabilities under `RUSTSEC-2026-0258`: `h2 0.3.27` and `h2 0.4.13`, both patched by `>=0.4.16`.
- `cargo audit` locally reports informational warnings for `event-listener 5.4.1` patched by `>=5.4.2`, and `lru 0.12.5` / `lru 0.18.1` patched by `>=0.18.2`.

### Delivery Risks

- ~~Remote `main` is not green because `Security Audit` fails.~~ **RESOLVED 2026-08-31** via PR #1485 (`ae3e2cef`); remote `main` CI is green. Remaining delivery risk: production is still deployed at the pre-fix revision `b0827e1a`, so Deployed/Operational evidence axes are unchanged until the next release.
- The local checkout contains valuable gate/contract/runtime capability evidence that has not been pushed or deployed.
- Existing open PRs are stale/conflicting and should not be merged without a fresh review-prep-merge flow.
- Cloudflare `selemene-pattern-memory` is declared in source but not live under the `9d9d` profile.
- Live `ts-engines` does not expose `/engines/capabilities`; the local endpoint is not deployed.
- The W3E corpus is large enough that unsequenced engine work would create false completion pressure.

### Readiness Score

**Red for release promotion; Yellow for local planning and implementation.**

Release promotion is now Yellow, up from Red: remote `main` CI is green as of `ae3e2cef` (PR #1485, 2026-08-31), but production is still at the older deployed revision `b0827e1a` pending a fresh deploy. Local planning is Yellow because the issue corpus, infra inventory, and local gates are sufficiently mapped to continue the constrained next wave.

## 2) Next Wave

### Priority 1: Remote Gate Recovery

**Issues:** `#901`, `#914`, `#893`

**Why now (updated 2026-08-31):** Remote CI was the first blocking proof layer and is now cleared — `Security Audit` passed on `origin/main` at `ae3e2cef` via PR #1485. The next blocking proof layer is landing and shipping the local runtime-capability evidence (this session's `/engines/capabilities` slice) so that Implemented/Executable evidence starts converting to Integrated and, eventually, Deployed/Operational evidence.

**Owner/agent lane:** Security/backend agent, sequential.

**Done criteria:**
- `cargo audit` has zero vulnerability findings.
- Any remaining informational warnings are either remediated or documented as release-accepted with a concrete reason.
- `pnpm run gate` and `cargo audit` pass locally.
- A PR or branch receipt shows remote `CI - Test & Lint` green.

**Verification steps:**
- `cargo audit --json | jq '.vulnerabilities.list | length'` returns `0`.
- `cargo tree -i h2@0.3.27 --locked` no longer returns an active dependency path.
- `cargo tree -i h2@0.4.13 --locked` no longer returns an active dependency path unless its resolved version is patched.
- `pnpm run gate` exits `0`.
- `gh run list --workflow 'CI - Test & Lint' --limit 5 --json conclusion,headSha,url` shows a green run for the submitted branch or PR SHA.

### Priority 2: PR Lane Cleanup

**Issues:** `#893`, `#901`

**Why now:** Conflicting stale PRs create noise and can hide required gate signals.

**Owner/agent lane:** GitHub PR operations agent, sequential and read-first.

**Done criteria:**
- PR `#886` is either rebased through the review-prep-merge pipeline or explicitly superseded in a local report.
- PR `#907` is either refreshed from current `main` or left draft with a written blocking reason.
- No stale PR is merged or closed without explicit operator approval.

**Verification steps:**
- `gh pr view 886 --json mergeable,mergeStateStatus,reviewDecision,statusCheckRollup`.
- `gh pr view 907 --json isDraft,mergeable,mergeStateStatus,statusCheckRollup`.
- If a PR moves forward, use the gated flow from `git-pr-ops-core`: review artifact, prepare artifact, head-SHA-pinned merge only after checks.

### Priority 3: Land Local Authority and Contract Evidence Through Remote CI

**Issues:** `#896`, `#901`, `#894`, `#893`

**Why now:** Local evidence for gate foundation and contract authority v1 exists, but remote `origin/main` and production do not yet carry that proof.

**Owner/agent lane:** Backend/contracts agent, sequential until CI is green.

**Done criteria:**
- Gate foundation evidence and contract authority v1 are on a PR branch derived from current `origin/main`.
- The PR contains only the intended gate/contract changes plus evidence docs.
- Remote CI is green, including the recovered security audit.
- Issues `#896`, `#901`, and `#894` can be updated later with exact PR, SHA, and test receipts.

**Verification steps:**
- `git diff --check`.
- `python3 -m pip install --disable-pip-version-check -r requirements-gates.txt`.
- `pnpm install --frozen-lockfile`.
- `(cd ts-engines && bun install --frozen-lockfile)`.
- `pnpm run gate`.
- `gh pr checks --watch` from the PR branch after a PR exists.

### Priority 4: Runtime Capability Adoption

**Issues:** `#894`, `#896`, W3E slot `04` and slot `11` for each engine, plus local `RUNTIME-CAPABILITY-EVIDENCE.md`

**Why now:** Capability discovery is the bridge between contract authority and engine execution. The local TypeScript endpoint is a useful first slice, but Rust/API and Python sidecar reporting remain separate.

**Owner/agent lane:** Runtime catalogue agent; TypeScript, Rust/API, and Python subtasks can run in parallel after Priority 1 is green.

**Done criteria:**
- TypeScript `GET /engines/capabilities` is merged and tested.
- Rust/API capability discovery uses `contracts/v1` capability schema or contract-tested adapters.
- Python sidecar capability reporting distinguishes available, unavailable, and degraded states without provider calls.
- Deployed runtime probes identify which revision exposes which capability surface.

**Verification steps:**
- `cd ts-engines && bun run typecheck && bun test`.
- `pnpm run gate:contracts`.
- `pnpm run gate:ts`.
- `curl -fsS https://ts-engines-production.up.railway.app/engines/capabilities` only after deployment authorization.
- No provider, generation API, database mutation, KV read, or secret read is used for discovery.

### Priority 5: First Engine Truth Batch

**Issues:** `#897`, W3E slots `06`, `07`, `09`, `10`, `17`, `18`, `25`, `27`, `28` for selected focus engines

**Why now:** Engine semantic work should start only after capability discovery can show partial/degraded states. The first batch should target engines already involved in contracts and live bridges.

**Owner/agent lane:** Parallel engine agents after Priority 4 interfaces are stable.

**Selected focus engines:**
- `biofield`
- `face-reading`
- `raaga`
- `sigil-forge`
- `tarot`
- `i-ching`
- `sacred-geometry`

**Done criteria:**
- Each engine exposes deterministic inputs, generated interpretation boundaries, fallback status, omitted-source status, and provenance where relevant.
- Golden fixtures cover at least one happy path and one missing-provider or missing-sidecar path.
- Result envelopes do not silently convert unavailable inputs into confident output.

**Verification steps:**
- Rust engine tests for native engines touched by the batch.
- `cd ts-engines && bun test tests/integration.test.ts`.
- `cd ts-engines && bun test tests/baseline_registry.test.ts`.
- Fixture or snapshot tests prove deterministic fields separate from generated narrative.
- Local API or bridge tests prove consumers can see fallback/degraded status.

### Priority 6: Stateful Durability and Auth Contract

**Issues:** `#913`, W3E slots `19`, `20`, `21`, `22`

**Why now:** Persistence, auth, cache, billing, and deletion semantics are cross-cutting risks. They should follow contract/runtime catalogue convergence, not precede it.

**Owner/agent lane:** Backend/state agent, mostly sequential.

**Done criteria:**
- Auth/API-key/session semantics are documented and tested across API, SDK, CLI, admin, and tool surfaces.
- Cache keys and idempotency behavior are deterministic and tested.
- Billing/webhooks are fail-closed and idempotent.
- Media/artifact retention and deletion behavior is explicit.

**Verification steps:**
- Existing billing, auth, and migration tests pass.
- New tests cover stale/degraded persistence and deletion behavior.
- No live billing or webhook activation occurs without a separate operator approval.

### Priority 7: Distribution and Operational Receipts

**Issues:** `#908`, `#914`, W3E slots `29`, `30`

**Why now:** Distribution and deployment claims must consume the canonical contract/runtime surfaces and cannot become separate authorities.

**Owner/agent lane:** Infra/distribution agent, sequential near release.

**Done criteria:**
- SDK/CLI/TUI/admin/app-facing clients consume canonical capability and result envelope contracts.
- Service manifest includes commit SHA, build identity, schema revision, runtime role, enabled engines, and dependency status.
- Smoke and rollback receipts are revision-bound.
- Asset manifests cover R2 buckets, Vectorize indexes, D1 databases, Railway volumes, and generated media retention.

**Verification steps:**
- `pnpm run gate`.
- Contract compatibility fixtures pass for SDK and TypeScript engine consumers.
- Railway readback identifies deployed commit and service roles.
- Cloudflare readback identifies Workers/KV/R2/D1/Vectorize resources without reading secrets or object payloads.

## 3) Execution Tasks

### Task 1: Recover Security Audit

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify if needed: `.github/workflows/test.yml`
- Modify: `docs/plans/selemene-engine/GATE-FOUNDATION-EVIDENCE.md`

**Interfaces:**
- Consumes: GitHub issue `#901`, issue `#914`, remote run `32248132223`, local `cargo audit` JSON.
- Produces: Green local `cargo audit`, green `pnpm run gate`, and a PR-ready security-audit receipt.

- [x] **Step 1: Reproduce the audit failure locally** — RESOLVED 2026-08-31 via PR #1485 (`ae3e2cef`, commits `07fb220`/`1348460`/`3252b79`); steps below are kept as the regression-verification record, not open work.

Run:

```bash
cargo audit --json | jq -r '{
  vulnerabilities: (.vulnerabilities.list // [] | map({package:.package.name, version:.package.version, advisory:.advisory.id, patched:.versions.patched})),
  warnings: (.warnings // {})
}'
```

Expected before the fix: `h2 0.3.27` and `h2 0.4.13` report `RUSTSEC-2026-0258`.

- [ ] **Step 2: Map dependency owners**

Run:

```bash
cargo tree -i h2@0.3.27 --locked
cargo tree -i h2@0.4.13 --locked
cargo tree -i event-listener@5.4.1 --locked
cargo tree -i lru@0.12.5 --locked
```

Expected before the fix:
- `h2 0.3.27` is reached through `opentelemetry-otlp 0.15.0 -> tonic 0.11.0 -> hyper 0.14.32`.
- `h2 0.4.13` is reached through `hyper 1.8.1`, `reqwest 0.12.28`, and API/client crates.
- `event-listener 5.4.1` is reached through `sqlx-core 0.8.6`.
- `lru 0.12.5` is reached through `noesis-cache`, `noesis-vedic-api`, and `noesis-western-api`.

- [ ] **Step 3: Apply the smallest dependency repair**

Use the audit remediation and dependency tree from Steps 1 and 2.

Required outcome:
- The old OTLP/tonic path no longer resolves to `h2 0.3.27`.
- The current hyper/reqwest path resolves to patched `h2 >=0.4.16`.
- `event-listener` resolves to `>=5.4.2` or the warning is documented as release-accepted.
- `lru` resolves to `>=0.18.2` wherever the crate is used, or each remaining occurrence is documented as release-accepted.

Do not add a broad dependency upgrade unless the dependency tree proves it is required.

- [ ] **Step 4: Verify locally**

Run:

```bash
cargo audit
pnpm run gate
git diff --check
```

Expected: `cargo audit` has zero vulnerability findings, `pnpm run gate` exits `0`, and `git diff --check` exits `0`.

- [ ] **Step 5: Record receipt**

Update `docs/plans/selemene-engine/GATE-FOUNDATION-EVIDENCE.md` with:
- advisory IDs observed,
- exact dependency path fixed,
- commands run,
- local gate result,
- remote PR/run receipt once available.

### Task 2: Prepare Current Local Evidence for PR

**Files:**
- Modify or keep: `docs/plans/selemene-engine/CONTRACT-V1-EVIDENCE.md`
- Modify or keep: `docs/plans/selemene-engine/GATE-FOUNDATION-EVIDENCE.md`
- Modify or keep: `docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md`
- Modify: current code/test files only if this task owns them.

**Interfaces:**
- Consumes: local commits `c11bc39` through `c96de19`, uncommitted runtime capability slice, issues `#896`, `#901`, `#894`.
- Produces: a PR-ready branch whose diff can be reviewed without unrelated workspace churn.

- [ ] **Step 1: Inspect current diff**

Run:

```bash
git status --short --branch
git log --oneline origin/main..HEAD
git diff --stat
git diff --check
```

Expected: branch is ahead of `origin/main`, and uncommitted files are limited to the runtime capability slice plus planning/evidence docs.

- [ ] **Step 2: Decide branch packaging**

Package into at most two PRs:
- PR A: gate/security/contract authority and evidence.
- PR B: runtime capability endpoint and evidence.

Do not mix engine semantic repairs into either PR.

- [ ] **Step 3: Verify each package locally**

Run for PR A:

```bash
pnpm run gate
cargo audit
git diff --check
```

Run for PR B:

```bash
cd ts-engines
bun run typecheck
bun test
```

Expected: all commands pass before any push.

### Task 3: Finish Runtime Capability Coverage

**Files:**
- Modify: `ts-engines/src/server/app.ts`
- Modify: `ts-engines/src/server/registry.ts`
- Modify: `ts-engines/tests/baseline_registry.test.ts`
- Modify: `ts-engines/tests/health.test.ts`
- Modify: `crates/noesis-api/src/lib.rs`
- Modify: `crates/noesis-core/src/contract.rs`
- Modify: `crates/noesis-bridge/src/lib.rs`
- Create: `crates/noesis-api/tests/capability_route_tests.rs`
- Modify: `python-services/shared/models.py`
- Modify: `python-services/biofield_cv_service/health.py`
- Modify: `python-services/mediapipe_service/health.py`
- Create: `python-services/tests/test_capability_health.py`
- Modify: `docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md`

**Interfaces:**
- Consumes: `contracts/v1` engine capability schema, issues `#894`, `#896`, W3E slots `04` and `11`.
- Produces: one capability discovery story across TypeScript, Rust/API, and Python sidecars.

- [ ] **Step 1: Keep the TypeScript endpoint green**

Run:

```bash
cd ts-engines
bun run typecheck
bun test tests/baseline_registry.test.ts tests/health.test.ts
```

Expected: targeted tests pass and `/engines/capabilities` reports `{ capabilities, count }`.

- [ ] **Step 2: Add Rust/API contract parity**

Implement a Rust/API adapter that serializes capability rows against `contracts/v1` without calling providers, databases, or remote services.

Verification:

```bash
pnpm run gate:contracts
```

Expected: schema and API parity tests fail before implementation and pass after implementation.

- [ ] **Step 3: Add Python sidecar capability states**

Expose sidecar capability status as explicit `available`, `unavailable`, or `degraded` from local self-checks only.

Verification:

```bash
pnpm run gate:ts
python3 -m pytest tests/scripts -q
```

Expected: no provider credentials are required and missing sidecars produce explicit unavailable/degraded records.

- [ ] **Step 4: Update issue receipts**

Prepare issue-comment text for `#894`, `#896`, and the relevant W3E slot `04`/`11` rows after PR creation. Do not post it until the user authorizes GitHub mutation.

### Task 4: Build the First Engine Truth Batch

**Files:**
- Modify: engine files only for the selected focus engine.
- Modify: tests colocated with that engine.
- Modify: fixtures or snapshots used by that engine.
- Modify: `docs/plans/selemene-engine/RUNTIME-CAPABILITY-EVIDENCE.md` or a new engine-specific evidence file.

**Interfaces:**
- Consumes: issue `#897`, W3E engine slots selected for the focus engine.
- Produces: one engine with auditable deterministic/generated/fallback/provenance boundaries.

- [ ] **Step 1: Select one focus engine**

Start with one of:
- `biofield`
- `face-reading`
- `raaga`
- `sigil-forge`
- `tarot`
- `i-ching`
- `sacred-geometry`

Selection rule: choose the engine whose capability discovery and existing tests are already green.

- [ ] **Step 2: Write the missing-state test**

Add a test proving missing provider, sidecar, input, or fixture state is visible as degraded/unavailable instead of silent success.

Expected before implementation: the test fails because the current engine hides or omits the state.

- [ ] **Step 3: Implement minimal truth surface**

Add only the fields required for the selected W3E slot: provenance, fallback status, omitted-source status, generated interpretation boundary, or fixture replay data.

- [ ] **Step 4: Verify**

Run the narrow engine test first, then the relevant aggregate gate:

```bash
cargo test -p engine-biofield
cargo test -p engine-face-reading
```

or:

```bash
cd ts-engines
bun test tests/integration.test.ts
bun test tests/baseline_registry.test.ts
bun run typecheck
```

Expected: narrow tests pass, and no unrelated engine behavior changes.

### Task 5: Draft GitHub Update Packets

**Files:**
- Create: `docs/plans/selemene-engine/GH-ISSUE-UPDATE-PACKETS.md`

**Interfaces:**
- Consumes: this plan, local verification receipts, and issue list.
- Produces: ready-to-post issue comments for `#893`, `#901`, `#894`, `#896`, `#897`, and later W3E rows.

- [ ] **Step 1: Draft one comment per control issue**

Each comment must include:
- scope,
- exact commit or branch,
- commands run,
- pass/fail result,
- remaining evidence axes,
- explicit non-mutation statement for deployments and secrets.

- [ ] **Step 2: Keep comments local until authorized**

Do not run `gh issue comment` until the user authorizes remote GitHub updates.

- [ ] **Step 3: Verify packet hygiene**

Run:

```bash
rg -n 'SECRET|TOKEN|PASSWORD|DATABASE_URL|API_KEY|PRIVATE|BEGIN ' docs/plans/selemene-engine/GH-ISSUE-UPDATE-PACKETS.md
git diff --check
```

Expected: no secrets or raw credentials appear, and whitespace check passes.

## Execution Order

1. Task 1: Recover Security Audit.
2. Task 2: Prepare Current Local Evidence for PR.
3. Task 3: Finish Runtime Capability Coverage.
4. Task 5: Draft GitHub Update Packets for the completed evidence.
5. Task 4: Build the First Engine Truth Batch.

Task 4 starts only after Tasks 1 through 3 have green local evidence. Tasks inside Task 4 can be parallelized by engine after one engine proves the pattern.

## Evidence Commands Used To Create This Plan

```bash
gh repo view --json nameWithOwner,defaultBranchRef,url,isPrivate,viewerPermission
gh issue list --state open --limit 1000 --json number,title,labels,createdAt,updatedAt,assignees,milestone,url
gh issue list --state all --limit 1000 --search 'W3E in:title' --json number,title,state,labels,createdAt,updatedAt,url
gh pr list --state open --limit 50 --json number,title,headRefName,baseRefName,isDraft,mergeable,mergeStateStatus,reviewDecision,updatedAt,labels,statusCheckRollup,url
gh run list --workflow 'CI - Test & Lint' --limit 10 --json databaseId,displayTitle,status,conclusion,createdAt,updatedAt,headBranch,headSha,event,url
gh run view 32248132223 --json jobs,conclusion,displayTitle,headSha,url
cargo audit --json
cargo tree -i h2@0.3.27 --locked
cargo tree -i h2@0.4.13 --locked
git status --short --branch
git log --oneline origin/main..HEAD
```

## Review Checkpoint

**Update 2026-08-31:** the `local-only security audit recovery` option below is now resolved (PR #1485, merged as `ae3e2cef`, remote `main` CI green). The local branch has been fast-forward synced onto that fix. The remaining decision this checkpoint originally posed has been actioned: PR packaging for the runtime-capability evidence slice and drafting the GitHub issue update packets (`docs/plans/selemene-engine/GH-ISSUE-UPDATE-PACKETS.md`) were both produced this session per Task 2 and Task 5 above.

Ready for the next execution slice once the user confirms whether to proceed with:
- Task 3 (Finish Runtime Capability Coverage: native Rust/API adoption, Python/database-conditional capability reporting),
- Task 4 (Build the First Engine Truth Batch, parallelizable by engine once one engine proves the pattern),
- or posting the drafted GitHub issue update packets to their live issues (still gated on explicit authorization per Global Constraints).
