# Open Issues Snapshot (2026-03-13)

Total open issues: `366`

| Issue | Title | Labels | Updated |
| --- | --- | --- | --- |
| #16 | [P1-S1-02] Add user_roles and user_account_state tables | plan-sync, roadmap, plan-admin-panel-taskmaster-plan-2026-02-25, sprint-s1, area-data, owner-backend-eng, wave-w1, phase-p1 | 2026-02-25T02:19:35Z |
| #17 | [P1-S1-03] Add api_key_events and key metadata columns | plan-sync, roadmap, plan-admin-panel-taskmaster-plan-2026-02-25, sprint-s1, area-data, owner-backend-eng, wave-w1, phase-p1 | 2026-02-25T02:19:43Z |
| #18 | [P1-S1-04] Add history sync tables and idempotency columns | plan-sync, roadmap, plan-admin-panel-taskmaster-plan-2026-02-25, sprint-s1, area-data, owner-backend-eng, wave-w1, phase-p1 | 2026-02-25T02:19:50Z |
| #19 | [P1-S1-05] Add plan catalog and billing subscription schema | plan-sync, roadmap, plan-admin-panel-taskmaster-plan-2026-02-25, sprint-s1, area-data, owner-backend-eng, wave-w1, phase-p1 | 2026-02-25T02:19:57Z |
| #20 | [P1-S1-06] Implement usage partition maintenance function | plan-sync, roadmap, plan-admin-panel-taskmaster-plan-2026-02-25, sprint-s1, wave-w1, phase-p1, area-infra, owner-devops | 2026-02-25T02:20:05Z |
| #38 | [P1-W1-S2-01] Add compile-time visibility guards to prevent direct engine imports in handlers | roadmap, phase:P1, area:backend, wave:W1, taskmaster | 2026-02-28T17:44:05Z |
| #39 | [P1-W1-S2-02] Implement request tracing middleware to log orchestrator pass-through | roadmap, phase:P1, area:backend, wave:W1, taskmaster | 2026-02-28T17:44:07Z |
| #43 | [P1-W1-S2-06] Add phase-gate enforcement assertions to orchestrator execution path | roadmap, phase:P1, area:backend, wave:W1, taskmaster | 2026-02-28T17:44:16Z |
| #44 | [P1-W1-S2-07] Add runtime assertion that WorkflowOrchestrator is sole entry point for engine execution | roadmap, phase:P1, area:backend, wave:W1, taskmaster | 2026-02-28T17:44:18Z |
| #57 | [P1-W2-S1-08] Add error response correlation between orchestrator partial failures and HTTP response | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-03-11T14:10:02Z |
| #58 | [P1-W2-S2-01] Define input validation schema for EngineInput fields | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:44:51Z |
| #59 | [P1-W2-S2-02] Implement pre-fanout input validation in WorkflowExecutor | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:44:53Z |
| #60 | [P1-W2-S2-03] Implement per-engine input requirement validation | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:44:56Z |
| #61 | [P1-W2-S2-04] Add workflow-level input validation rules per workflow type | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:44:59Z |
| #62 | [P1-W2-S2-05] Implement request body size and structure validation middleware | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:01Z |
| #63 | [P1-W2-S2-06] Add input sanitization for string fields to prevent injection | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:03Z |
| #64 | [P1-W2-S2-07] Write fuzz tests for input validation edge cases | roadmap, phase:P1, area:qa, wave:W2, taskmaster | 2026-02-28T17:45:06Z |
| #65 | [P1-W2-S2-08] Implement validation error aggregation for multi-field failures | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:08Z |
| #66 | [P1-W2-S2-09] Add per-workflow options key validation with allowed-key registry | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:10Z |
| #67 | [P1-W2-S3-01] Audit current cache_key implementations across all engines | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:12Z |
| #68 | [P1-W2-S3-02] Design canonical CacheKeyBuilder with consistent hashing strategy | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:14Z |
| #69 | [P1-W2-S3-03] Migrate engine-numerology cache_key to CacheKeyBuilder | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:17Z |
| #70 | [P1-W2-S3-04] Migrate remaining Rust engine cache_keys to CacheKeyBuilder | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:19Z |
| #71 | [P1-W2-S3-05] Migrate BridgeEngine cache_key to CacheKeyBuilder | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:21Z |
| #72 | [P1-W2-S3-06] Align WorkflowCacheKey with CacheKeyBuilder for workflow-level caching | roadmap, phase:P1, area:backend, wave:W2, taskmaster | 2026-02-28T17:45:23Z |
| #75 | [P1-W3-S1-01] Write end-to-end test for orchestrator-only routing across all engine endpoints | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:29Z |
| #76 | [P1-W3-S1-02] Write end-to-end test for all 6 workflow execution paths | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:32Z |
| #77 | [P1-W3-S1-03] Write error path integration tests for all EngineError variants | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:34Z |
| #78 | [P1-W3-S1-04] Write cache hit/miss integration tests for workflow caching | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:36Z |
| #79 | [P1-W3-S1-05] Write validation rejection integration tests for all input constraints | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:38Z |
| #80 | [P1-W3-S1-06] Write phase-gating integration tests across workflow and engine endpoints | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:40Z |
| #81 | [P1-W3-S1-07] Write degraded-mode integration tests for optional dependencies | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:42Z |
| #82 | [P1-W3-S1-08] Write concurrency stress test for parallel workflow execution | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:44Z |
| #83 | [P1-W3-S1-09] Write cache invalidation integration test for engine version upgrade scenario | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:45:46Z |
| #84 | [P1-W3-S2-01] Add CI gate: orchestrator routing enforcement test suite must pass | roadmap, phase:P1, area:infra, wave:W3, taskmaster | 2026-02-28T17:45:48Z |
| #85 | [P1-W3-S2-02] Add CI gate: error mapping exhaustiveness check must pass | roadmap, phase:P1, area:infra, wave:W3, taskmaster | 2026-02-28T17:45:51Z |
| #86 | [P1-W3-S2-03] Add CI gate: baseline matrix version check | roadmap, phase:P1, area:infra, wave:W3, taskmaster | 2026-02-28T17:45:53Z |
| #87 | [P1-W3-S2-04] Add CI gate: cache key format validation | roadmap, phase:P1, area:infra, wave:W3, taskmaster | 2026-02-28T17:45:55Z |
| #88 | [P1-W3-S2-05] Add CI gate: input validation schema regression check | roadmap, phase:P1, area:infra, wave:W3, taskmaster | 2026-02-28T17:45:57Z |
| #89 | [P1-W3-S2-06] Create CI summary job aggregating all P1 gate results | roadmap, phase:P1, area:infra, wave:W3, taskmaster | 2026-02-28T17:46:01Z |
| #90 | [P1-W3-S2-07] Add CI performance regression gate for engine execution benchmarks | roadmap, phase:P1, area:infra, wave:W3, taskmaster | 2026-02-28T17:46:04Z |
| #91 | [P1-W3-S3-01] Write Architecture Decision Record for orchestrator-only routing pattern | roadmap, phase:P1, area:product, wave:W3, taskmaster | 2026-02-28T17:46:09Z |
| #92 | [P1-W3-S3-02] Write Architecture Decision Record for unified error mapping strategy | roadmap, phase:P1, area:product, wave:W3, taskmaster | 2026-02-28T17:46:11Z |
| #93 | [P1-W3-S3-03] Write Architecture Decision Record for cache key normalization | roadmap, phase:P1, area:product, wave:W3, taskmaster | 2026-02-28T17:46:14Z |
| #94 | [P1-W3-S3-04] Update OpenAPI spec with all error response schemas | roadmap, phase:P1, area:backend, wave:W3, taskmaster | 2026-02-28T17:46:16Z |
| #95 | [P1-W3-S3-05] Update OpenAPI spec with input validation constraints | roadmap, phase:P1, area:backend, wave:W3, taskmaster | 2026-02-28T17:46:18Z |
| #96 | [P1-W3-S3-06] Create P1 completion report with gate status matrix | roadmap, phase:P1, area:product, wave:W3, taskmaster | 2026-02-28T17:46:21Z |
| #97 | [P1-W3-S3-07] Run full regression test suite and document baseline test metrics | roadmap, phase:P1, area:qa, wave:W3, taskmaster | 2026-02-28T17:46:23Z |
| #98 | [P1-W3-S3-08] Gate A sign-off: verify all orchestrator routing and error mapping tests enforced | roadmap, phase:P1, area:product, wave:W3, taskmaster | 2026-02-28T17:46:25Z |
| #99 | [P2-W1-S1-01] Create L2 unavailability test harness | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:46:28Z |
| #100 | [P2-W1-S1-02] Test L1-only fallback on cache get | roadmap, phase:P2, area:qa, wave:W1, taskmaster | 2026-02-28T17:46:30Z |
| #101 | [P2-W1-S1-03] Test L1-only fallback on cache store | roadmap, phase:P2, area:qa, wave:W1, taskmaster | 2026-02-28T17:46:32Z |
| #102 | [P2-W1-S1-04] Test cache stats track L2 misses during degradation | roadmap, phase:P2, area:qa, wave:W1, taskmaster | 2026-02-28T17:46:35Z |
| #103 | [P2-W1-S1-05] Add tracing warn events for Redis degradation | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:46:37Z |
| #104 | [P2-W1-S1-06] Test health_check reports degraded when Redis is down | roadmap, phase:P2, area:qa, wave:W1, taskmaster | 2026-02-28T17:46:39Z |
| #105 | [P2-W1-S1-07] Test L3 disk fallback when both L1 and L2 miss | roadmap, phase:P2, area:qa, wave:W1, taskmaster | 2026-02-28T17:46:41Z |
| #107 | [P2-W1-S2-01] Define BridgeRetryPolicy config struct | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:46:47Z |
| #108 | [P2-W1-S2-02] Implement exponential backoff retry loop | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:46:49Z |
| #109 | [P2-W1-S2-03] Integrate retry policy into BridgeEngine::calculate | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:46:51Z |
| #110 | [P2-W1-S2-04] Integrate retry policy into BridgeEngine::validate | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:46:53Z |
| #111 | [P2-W1-S2-05] Add environment variable overrides for bridge policy | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:46:55Z |
| #112 | [P2-W1-S2-06] Test retry exhaustion returns last error | roadmap, phase:P2, area:qa, wave:W1, taskmaster | 2026-02-28T17:46:57Z |
| #113 | [P2-W1-S2-07] Test non-retryable errors skip retry loop | roadmap, phase:P2, area:qa, wave:W1, taskmaster | 2026-02-28T17:46:59Z |
| #114 | [P2-W1-S2-08] Add retry metrics to BridgeEngine tracing spans | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:47:02Z |
| #117 | [P2-W1-S3-03] Add selfCheck method to TS engine interface | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:47:09Z |
| #118 | [P2-W1-S3-04] Integrate readiness check into BridgeManager health_check | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:47:11Z |
| #119 | [P2-W1-S3-05] Add startup readiness gate to TS server | roadmap, phase:P2, area:backend, wave:W1, taskmaster | 2026-02-28T17:47:14Z |
| #123 | [P2-W2-S1-01] Create shared workflow fixture module | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:47:36Z |
| #124 | [P2-W2-S1-02] Define birth-blueprint output JSON schema | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:47:40Z |
| #125 | [P2-W2-S1-03] Create birth-blueprint deterministic fixtures | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:47:44Z |
| #126 | [P2-W2-S1-04] Implement birth-blueprint happy path contract test | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:47:50Z |
| #127 | [P2-W2-S1-05] Implement birth-blueprint edge case contract tests | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:47:53Z |
| #128 | [P2-W2-S1-06] Define daily-practice output JSON schema | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:47:57Z |
| #129 | [P2-W2-S1-07] Create daily-practice deterministic fixtures | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:05Z |
| #130 | [P2-W2-S1-08] Implement daily-practice happy path contract test | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:11Z |
| #131 | [P2-W2-S1-09] Implement daily-practice edge case contract tests | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:16Z |
| #132 | [P2-W2-S1-10] Test birth-blueprint synthesis determinism with frozen fixtures | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:21Z |
| #133 | [P2-W2-S1-11] Test daily-practice synthesis determinism with frozen fixtures | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:26Z |
| #134 | [P2-W2-S2-01] Define decision-support output JSON schema | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:32Z |
| #135 | [P2-W2-S2-02] Create decision-support deterministic fixtures | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:37Z |
| #136 | [P2-W2-S2-03] Implement decision-support happy path contract test | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:39Z |
| #137 | [P2-W2-S2-04] Implement decision-support edge case contract tests | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:41Z |
| #138 | [P2-W2-S2-05] Define self-inquiry output JSON schema | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:44Z |
| #139 | [P2-W2-S2-06] Create self-inquiry deterministic fixtures | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:46Z |
| #140 | [P2-W2-S2-07] Implement self-inquiry happy path contract test | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:48Z |
| #141 | [P2-W2-S2-08] Implement self-inquiry edge case contract tests | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:50Z |
| #142 | [P2-W2-S2-09] Test decision-support synthesis with bridge retry interaction | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:52Z |
| #143 | [P2-W2-S3-01] Define creative-expression output JSON schema | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:54Z |
| #144 | [P2-W2-S3-02] Create creative-expression deterministic fixtures | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:48:57Z |
| #145 | [P2-W2-S3-03] Implement creative-expression happy path contract test | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:49:00Z |
| #146 | [P2-W2-S3-04] Implement creative-expression edge case contract tests | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:49:02Z |
| #147 | [P2-W2-S3-05] Define full-spectrum output JSON schema | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:49:04Z |
| #148 | [P2-W2-S3-06] Create full-spectrum deterministic fixtures | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:49:07Z |
| #149 | [P2-W2-S3-07] Implement full-spectrum happy path contract test | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:49:09Z |
| #150 | [P2-W2-S3-08] Implement full-spectrum edge case contract tests | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:49:11Z |
| #151 | [P2-W2-S3-09] Test creative-expression synthesis determinism with seeded randomness | roadmap, phase:P2, area:qa, wave:W2, taskmaster | 2026-02-28T17:49:14Z |
| #152 | [P2-W3-S1-01] Test engine selection matrix for birth-blueprint | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:16Z |
| #153 | [P2-W3-S1-02] Test engine selection matrix for daily-practice | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:18Z |
| #154 | [P2-W3-S1-03] Test engine selection matrix for decision-support | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:20Z |
| #155 | [P2-W3-S1-04] Test engine selection matrix for self-inquiry | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:22Z |
| #156 | [P2-W3-S1-05] Test engine selection matrix for creative-expression | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:25Z |
| #157 | [P2-W3-S1-06] Test engine selection matrix for full-spectrum | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:27Z |
| #158 | [P2-W3-S1-07] Test registry consistency between WorkflowOrchestrator and WorkflowRegistry | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:29Z |
| #159 | [P2-W3-S1-08] Test list_for_phase across all phase levels 0-5 | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:32Z |
| #160 | [P2-W3-S2-01] Add contract test job to GitHub Actions CI | roadmap, phase:P2, area:infra, wave:W3, taskmaster | 2026-02-28T17:49:34Z |
| #161 | [P2-W3-S2-02] Add selection matrix test job to CI | roadmap, phase:P2, area:infra, wave:W3, taskmaster | 2026-02-28T17:49:36Z |
| #162 | [P2-W3-S2-03] Create fixture validation CI step | roadmap, phase:P2, area:infra, wave:W3, taskmaster | 2026-02-28T17:49:38Z |
| #163 | [P2-W3-S2-04] Create Gate B verification checklist | roadmap, phase:P2, area:product, wave:W3, taskmaster | 2026-02-28T17:49:40Z |
| #164 | [P2-W3-S2-05] Generate contract test coverage report | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:42Z |
| #165 | [P2-W3-S2-06] Verify all contract tests are deterministic | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:45Z |
| #166 | [P2-W3-S2-07] Add contract test badge to repository README | roadmap, phase:P2, area:product, wave:W3, taskmaster | 2026-02-28T17:49:47Z |
| #167 | [P2-W3-S2-08] Document P2 contract testing methodology | roadmap, phase:P2, area:product, wave:W3, taskmaster | 2026-02-28T17:49:49Z |
| #168 | [P2-W3-S2-09] Run full P2 regression suite and produce Gate B sign-off artifact | roadmap, phase:P2, area:qa, wave:W3, taskmaster | 2026-02-28T17:49:51Z |
| #169 | [P3-W1-S1-01] Define canonical bridge envelope JSON Schema | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:49:53Z |
| #170 | [P3-W1-S1-02] Add envelope version field to TsEngineRequest and TsEngineResponse | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:49:56Z |
| #171 | [P3-W1-S1-03] Add envelope version field to TS EngineInput and EngineOutput types | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:49:58Z |
| #172 | [P3-W1-S1-04] Create parity fixture files for all 5 TS engines | roadmap, phase:P3, area:qa, wave:W1, taskmaster | 2026-02-28T17:50:00Z |
| #173 | [P3-W1-S1-05] Implement Rust-side envelope validation middleware in BridgeEngine | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:02Z |
| #174 | [P3-W1-S2-01] Align tarot engine response envelope to canonical schema | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:04Z |
| #175 | [P3-W1-S2-02] Align i-ching engine response envelope to canonical schema | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:06Z |
| #176 | [P3-W1-S2-03] Align enneagram engine response envelope to canonical schema | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:09Z |
| #177 | [P3-W1-S2-04] Align sacred-geometry engine response envelope to canonical schema | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:11Z |
| #178 | [P3-W1-S2-05] Align sigil-forge engine response envelope to canonical schema | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:13Z |
| #179 | [P3-W1-S2-06] Create cross-engine parity integration test suite | roadmap, phase:P3, area:qa, wave:W1, taskmaster | 2026-02-28T17:50:15Z |
| #180 | [P3-W1-S3-01] Define TS error code enumeration and documentation | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:17Z |
| #181 | [P3-W1-S3-02] Implement TS error_code in all 5 engine error paths | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:20Z |
| #182 | [P3-W1-S3-03] Implement Rust-side error_code to EngineError mapping in BridgeEngine | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:22Z |
| #183 | [P3-W1-S3-04] Add structured error details to BridgeError variant | roadmap, phase:P3, area:backend, wave:W1, taskmaster | 2026-02-28T17:50:24Z |
| #184 | [P3-W1-S3-05] Create error taxonomy mapping documentation | roadmap, phase:P3, area:product, wave:W1, taskmaster | 2026-02-28T17:50:27Z |
| #185 | [P3-W2-S1-01] Extract CircuitBreaker into shared crate module | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:29Z |
| #186 | [P3-W2-S1-02] Add per-engine CircuitBreaker instances to BridgeManager | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:31Z |
| #187 | [P3-W2-S1-03] Integrate CircuitBreaker check into BridgeEngine::calculate() | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:34Z |
| #188 | [P3-W2-S1-04] Add configurable circuit breaker thresholds via environment variables | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:36Z |
| #189 | [P3-W2-S1-05] Expose circuit breaker state via /health/bridge endpoint | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:38Z |
| #190 | [P3-W2-S1-06] Add admin endpoint to manually reset circuit breakers | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:40Z |
| #191 | [P3-W2-S2-01] Define retry policy configuration struct | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:43Z |
| #192 | [P3-W2-S2-02] Implement retry logic in BridgeEngine::calculate() | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:45Z |
| #193 | [P3-W2-S2-03] Add per-engine timeout configuration | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:47Z |
| #194 | [P3-W2-S2-04] Classify retryable vs non-retryable errors | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:49Z |
| #195 | [P3-W2-S2-05] Add retry attempt metadata to engine response | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:51Z |
| #196 | [P3-W2-S3-01] Implement per-engine health probe endpoint in TS sidecar | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:53Z |
| #197 | [P3-W2-S3-02] Implement background health probe loop in BridgeManager | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:55Z |
| #198 | [P3-W2-S3-03] Add startup readiness gate for TS sidecar | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:50:58Z |
| #199 | [P3-W2-S3-04] Emit sidecar availability events to structured logs | roadmap, phase:P3, area:backend, wave:W2, taskmaster | 2026-02-28T17:51:00Z |
| #200 | [P3-W3-S1-01] Add bridge-specific latency histogram to NoesisMetrics | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:02Z |
| #201 | [P3-W3-S1-02] Add circuit breaker state gauge to NoesisMetrics | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:04Z |
| #202 | [P3-W3-S1-03] Wire bridge metrics recording into BridgeEngine::calculate() | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:06Z |
| #203 | [P3-W3-S1-04] Add retry attempt counter metric | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:09Z |
| #204 | [P3-W3-S1-05] Add TS sidecar response time histogram to sidecar itself | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:12Z |
| #205 | [P3-W3-S1-06] Create Prometheus alert rules for bridge latency and circuit breaker | roadmap, phase:P3, area:infra, wave:W3, taskmaster | 2026-02-28T17:51:14Z |
| #206 | [P3-W3-S2-01] Add traceparent header injection in BridgeEngine HTTP requests | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:16Z |
| #207 | [P3-W3-S2-02] Parse traceparent header in TS sidecar and create child spans | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:19Z |
| #208 | [P3-W3-S2-03] Return traceresponse header from TS sidecar | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:21Z |
| #209 | [P3-W3-S2-04] Add bridge span attributes to Rust tracing spans | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:23Z |
| #210 | [P3-W3-S2-05] Add trace context to TS sidecar structured logs | roadmap, phase:P3, area:backend, wave:W3, taskmaster | 2026-02-28T17:51:26Z |
| #211 | [P3-W3-S2-06] Document span naming conventions for bridge traces | roadmap, phase:P3, area:product, wave:W3, taskmaster | 2026-02-28T17:51:28Z |
| #212 | [P3-W3-S3-01] Create Grafana bridge reliability dashboard | roadmap, phase:P3, area:infra, wave:W3, taskmaster | 2026-02-28T17:51:30Z |
| #213 | [P3-W3-S3-02] Create Grafana TS sidecar dashboard | roadmap, phase:P3, area:infra, wave:W3, taskmaster | 2026-02-28T17:51:33Z |
| #214 | [P3-W3-S3-03] Write k6 load test script for bridge endpoints | roadmap, phase:P3, area:qa, wave:W3, taskmaster | 2026-02-28T17:51:35Z |
| #215 | [P3-W3-S3-04] Write chaos test: sidecar crash during traffic | roadmap, phase:P3, area:qa, wave:W3, taskmaster | 2026-02-28T17:51:37Z |
| #216 | [P3-W3-S3-05] Write chaos test: slow sidecar (latency injection) | roadmap, phase:P3, area:qa, wave:W3, taskmaster | 2026-02-28T17:51:39Z |
| #217 | [P3-W3-S3-06] Gate C verification: end-to-end bridge reliability test suite | roadmap, phase:P3, area:qa, wave:W3, taskmaster | 2026-02-28T17:51:42Z |
| #218 | [P3-W3-S3-07] Write runbook for bridge failure scenarios | roadmap, phase:P3, area:infra, wave:W3, taskmaster | 2026-02-28T17:51:44Z |
| #219 | [P3-W3-S3-08] Update CI pipeline with bridge reliability checks | roadmap, phase:P3, area:infra, wave:W3, taskmaster | 2026-02-28T17:51:46Z |
| #220 | [P3-W3-S3-09] Performance baseline: capture bridge latency baselines before and after | roadmap, phase:P3, area:qa, wave:W3, taskmaster | 2026-02-28T17:51:48Z |
| #221 | [P4-W1-S1-01] Replace placeholder calculation_benchmarks.rs with panchanga benchmark suite | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:51:50Z |
| #222 | [P4-W1-S1-02] Create numerology engine benchmark suite | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:51:53Z |
| #223 | [P4-W1-S1-03] Create biorhythm engine benchmark suite | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:51:55Z |
| #224 | [P4-W1-S1-04] Create gene-keys engine benchmark suite | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:51:57Z |
| #225 | [P4-W1-S1-05] Create vimshottari dasha benchmark suite | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:00Z |
| #226 | [P4-W1-S1-06] Create biofield and vedic-clock engine benchmark suites | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:02Z |
| #227 | [P4-W1-S1-07] Create face-reading, nadabrahman, and transits engine benchmark suites | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:04Z |
| #228 | [P4-W1-S1-08] Add Cargo.toml bench entries for all new benchmark files | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:07Z |
| #229 | [P4-W1-S1-09] Create benchmark runner script with baseline capture | roadmap, phase:P4, area:infra, wave:W1, taskmaster | 2026-02-28T17:52:09Z |
| #230 | [P4-W1-S1-10] Add CI workflow step for benchmark regression detection | roadmap, phase:P4, area:infra, wave:W1, taskmaster | 2026-02-28T17:52:12Z |
| #231 | [P4-W1-S1-11] Generate initial baseline benchmark report | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:14Z |
| #232 | [P4-W1-S1-12] Create TS sidecar engine latency benchmark via bridge | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:16Z |
| #233 | [P4-W1-S2-01] Create expected output fixtures for panchanga engine | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:19Z |
| #234 | [P4-W1-S2-02] Create expected output fixtures for numerology, human-design, and gene-keys engines | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:21Z |
| #235 | [P4-W1-S2-03] Create expected output fixtures for biorhythm, vimshottari, vedic-clock, and biofield engines | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:23Z |
| #236 | [P4-W1-S2-04] Create expected output fixtures for face-reading, nadabrahman, and transits engines | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:25Z |
| #237 | [P4-W1-S2-05] Create expected output fixtures for 5 TS bridge engines | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:28Z |
| #238 | [P4-W1-S2-06] Create edge-case fixture pack for boundary condition testing | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:30Z |
| #239 | [P4-W1-S2-07] Create workflow-level expected output fixtures for all 6 workflows | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:33Z |
| #240 | [P4-W1-S2-08] Add fixture version manifest and snapshot test harness | roadmap, phase:P4, area:qa, wave:W1, taskmaster | 2026-02-28T17:52:35Z |
| #241 | [P4-W1-S3-01] Tune engine_calculation_duration histogram buckets based on baseline benchmarks | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:37Z |
| #242 | [P4-W1-S3-02] Add workflow-level duration and engine-count metrics | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:40Z |
| #243 | [P4-W1-S3-03] Add Grafana dashboard panels for per-engine and per-workflow latency heatmaps | roadmap, phase:P4, area:infra, wave:W1, taskmaster | 2026-02-28T17:52:42Z |
| #244 | [P4-W1-S3-04] Add Prometheus alert rules for p95 SLO violations | roadmap, phase:P4, area:infra, wave:W1, taskmaster | 2026-02-28T17:52:44Z |
| #245 | [P4-W1-S3-05] Add TS bridge engine latency metrics to BridgeEngine | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:46Z |
| #246 | [P4-W1-S3-06] Add request-level tracing spans for orchestrator fan-out | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:49Z |
| #247 | [P4-W1-S3-07] Configure Alertmanager rules for engine latency p95 breaches | roadmap, phase:P4, area:infra, wave:W1, taskmaster | 2026-02-28T17:52:51Z |
| #248 | [P4-W1-S3-08] Create consolidated benchmark comparison report generator | roadmap, phase:P4, area:backend, wave:W1, taskmaster | 2026-02-28T17:52:53Z |
| #249 | [P4-W2-S1-01] Add configurable concurrency limit to WorkflowOrchestrator | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:52:55Z |
| #250 | [P4-W2-S1-02] Add configurable concurrency limit to WorkflowExecutor | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:52:57Z |
| #251 | [P4-W2-S1-03] Add per-engine timeout enforcement in orchestrator fan-out | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:00Z |
| #252 | [P4-W2-S1-04] Expose fan-out configuration via environment variables | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:02Z |
| #253 | [P4-W2-S1-05] Add backpressure metrics for orchestrator queue depth | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:04Z |
| #254 | [P4-W2-S1-06] Add graceful degradation when all semaphore permits exhausted | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:07Z |
| #255 | [P4-W2-S1-07] Benchmark fan-out throughput at concurrency limits 4, 8, 12, 16 | roadmap, phase:P4, area:qa, wave:W2, taskmaster | 2026-02-28T17:53:09Z |
| #256 | [P4-W2-S2-01] Design idempotency key storage interface and TTL policy | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:11Z |
| #257 | [P4-W2-S2-02] Implement IdempotencyStore trait and in-memory backend | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:14Z |
| #258 | [P4-W2-S2-03] Implement Redis-backed IdempotencyStore | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:16Z |
| #259 | [P4-W2-S2-04] Add Axum middleware layer for idempotency key extraction and enforcement | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:18Z |
| #260 | [P4-W2-S2-05] Wire idempotency middleware to workflow execution endpoints | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:20Z |
| #261 | [P4-W2-S2-06] Add idempotency metrics (hit rate, store size, replay count) | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:22Z |
| #262 | [P4-W2-S2-07] Add idempotency key expiration and cleanup job | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:25Z |
| #263 | [P4-W2-S3-01] Create 60-minute auth soak test script | roadmap, phase:P4, area:qa, wave:W2, taskmaster | 2026-02-28T17:53:27Z |
| #264 | [P4-W2-S3-02] Add JWT secret rotation support to AuthService | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:29Z |
| #265 | [P4-W2-S3-03] Write key rotation runbook with operational steps | roadmap, phase:P4, area:infra, wave:W2, taskmaster | 2026-02-28T17:53:31Z |
| #266 | [P4-W2-S3-04] Execute key rotation drill during auth soak test | roadmap, phase:P4, area:qa, wave:W2, taskmaster | 2026-02-28T17:53:33Z |
| #267 | [P4-W2-S3-05] Verify rate limiter behavior under sustained 60-minute soak | roadmap, phase:P4, area:qa, wave:W2, taskmaster | 2026-02-28T17:53:35Z |
| #268 | [P4-W2-S3-06] Add auth latency metrics to AuthService validation paths | roadmap, phase:P4, area:backend, wave:W2, taskmaster | 2026-02-28T17:53:37Z |
| #269 | [P4-W2-S3-07] Test JWKS endpoint rotation with cached token validation | roadmap, phase:P4, area:qa, wave:W2, taskmaster | 2026-02-28T17:53:39Z |
| #270 | [P4-W3-S1-01] Design mixed-workflow traffic model with realistic distribution | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:53:41Z |
| #271 | [P4-W3-S1-02] Create k6 mixed-workflow load script with weighted distribution | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:53:43Z |
| #272 | [P4-W3-S1-03] Execute mixed-workflow load profile at 100 VU steady state | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:53:46Z |
| #273 | [P4-W3-S1-04] Execute mixed-workflow load profile at 200 VU steady state | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:53:48Z |
| #274 | [P4-W3-S1-05] Create mixed-workflow stress test with spike to 500 VU | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:53:50Z |
| #275 | [P4-W3-S1-06] Execute stress test and document recovery behavior | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:53:52Z |
| #276 | [P4-W3-S1-07] Run idempotency-key concurrency test under load | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:53:55Z |
| #277 | [P4-W3-S2-01] Implement L1 cache LRU eviction tuning based on load profile | roadmap, phase:P4, area:backend, wave:W3, taskmaster | 2026-02-28T17:53:58Z |
| #278 | [P4-W3-S2-02] Add cache warmup for common workflow inputs | roadmap, phase:P4, area:backend, wave:W3, taskmaster | 2026-02-28T17:54:00Z |
| #279 | [P4-W3-S2-03] Measure and report cache hit rates during mixed-workflow load test | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:54:03Z |
| #280 | [P4-W3-S2-04] Verify Redis L2 cache resilience during load test with Redis restart | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:54:06Z |
| #281 | [P4-W3-S2-05] Add cache stats Grafana panel with L1/L2/L3 breakdown | roadmap, phase:P4, area:infra, wave:W3, taskmaster | 2026-02-28T17:54:08Z |
| #282 | [P4-W3-S2-06] Test cache eviction behavior under sustained high-concurrency load | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:54:10Z |
| #283 | [P4-W3-S2-07] Tune engine-specific cache TTL values based on benchmark data | roadmap, phase:P4, area:backend, wave:W3, taskmaster | 2026-02-28T17:54:12Z |
| #284 | [P4-W3-S3-01] Update API documentation for orchestrator-first workflow execution | roadmap, phase:P4, area:product, wave:W3, taskmaster | 2026-02-28T17:54:15Z |
| #285 | [P4-W3-S3-02] Document idempotency key usage in API reference | roadmap, phase:P4, area:product, wave:W3, taskmaster | 2026-02-28T17:54:17Z |
| #286 | [P4-W3-S3-03] Document fan-out concurrency configuration and tuning guide | roadmap, phase:P4, area:product, wave:W3, taskmaster | 2026-02-28T17:54:19Z |
| #287 | [P4-W3-S3-04] Update llms.txt and README with P4 capability summary | roadmap, phase:P4, area:product, wave:W3, taskmaster | 2026-02-28T17:54:21Z |
| #288 | [P4-W3-S3-05] Create SLO definition document with error budget policy | roadmap, phase:P4, area:infra, wave:W3, taskmaster | 2026-02-28T17:54:23Z |
| #289 | [P4-W3-S3-06] Compile Gate D checklist with evidence artifacts | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:54:25Z |
| #290 | [P4-W3-S3-07] Gate D sign-off: validate p95 SLO under mixed workflow traffic | roadmap, phase:P4, area:qa, wave:W3, taskmaster | 2026-02-28T17:54:27Z |
| #300 | [P5-W1-S2-02] Add Railway deploy webhook notification to Slack/Discord | roadmap, phase:P5, area:infra, wave:W1, taskmaster | 2026-02-28T17:54:49Z |
| #301 | [P5-W1-S2-03] Add cold-start retry logic to smoke runner for Railway sleep recovery | roadmap, phase:P5, area:infra, wave:W1, taskmaster | 2026-02-28T17:54:51Z |
| #302 | [P5-W1-S2-04] Add Docker image size gate to CI pipeline | roadmap, phase:P5, area:infra, wave:W1, taskmaster | 2026-02-28T17:54:53Z |
| #303 | [P5-W1-S2-05] Add deploy timeout and automatic rollback trigger | roadmap, phase:P5, area:infra, wave:W1, taskmaster | 2026-02-28T17:54:55Z |
| #304 | [P5-W1-S2-06] Add admin-web Vercel deploy coordination check | roadmap, phase:P5, area:infra, wave:W1, taskmaster | 2026-03-01T20:00:56Z |
| #305 | [P5-W1-S2-07] Add Docker HEALTHCHECK instruction to Dockerfile.prod | roadmap, phase:P5, area:infra, wave:W1, taskmaster | 2026-03-13T03:01:19Z |
| #306 | [P5-W1-S2-08] Add deploy notification webhook to Slack/Discord on Railway deploy | roadmap, phase:P5, area:infra, wave:W1, taskmaster | 2026-02-28T17:55:02Z |
| #307 | [P5-W1-S3-01] Define ephemeris cache key namespace and invalidation scope | roadmap, phase:P5, area:backend, wave:W1, taskmaster | 2026-02-28T17:55:04Z |
| #308 | [P5-W1-S3-02] Add invalidate_by_prefix method to CacheManager for batch invalidation | roadmap, phase:P5, area:backend, wave:W1, taskmaster | 2026-02-28T17:55:06Z |
| #309 | [P5-W1-S3-03] Add ephemeris file checksum tracking in AppState | roadmap, phase:P5, area:backend, wave:W1, taskmaster | 2026-02-28T17:55:09Z |
| #310 | [P5-W1-S3-04] Add admin endpoint to trigger ephemeris cache invalidation | roadmap, phase:P5, area:backend, wave:W1, taskmaster | 2026-02-28T17:55:11Z |
| #311 | [P5-W1-S3-05] Add file-watcher hook for automatic ephemeris invalidation in dev mode | roadmap, phase:P5, area:backend, wave:W1, taskmaster | 2026-02-28T17:55:14Z |
| #312 | [P5-W1-S3-06] Add Sentry breadcrumb for cache invalidation events | roadmap, phase:P5, area:backend, wave:W1, taskmaster | 2026-02-28T17:55:16Z |
| #313 | [P5-W1-S3-07] Add Prometheus counter for cache invalidation events | roadmap, phase:P5, area:backend, wave:W1, taskmaster | 2026-02-28T17:55:18Z |
| #317 | [P5-W2-S1-04] Add canary deploy mode to GitHub Actions deploy workflow | roadmap, phase:P5, area:infra, wave:W2, taskmaster | 2026-03-13T05:20:52Z |
| #320 | [P5-W2-S1-07] Configure canary error rate threshold and auto-rollback trigger | roadmap, phase:P5, area:infra, wave:W2, taskmaster | 2026-02-28T17:55:34Z |
| #321 | [P5-W2-S1-08] Verify canary traffic split with request tracing headers | roadmap, phase:P5, area:qa, wave:W2, taskmaster | 2026-02-28T17:55:36Z |
| #329 | [P5-W2-S2-08] Write auth system failure runbook | roadmap, phase:P5, area:product, wave:W2, taskmaster | 2026-03-13T06:57:55Z |
| #331 | [P5-W2-S3-02] Execute rollback drill: broken environment variable deploy | roadmap, phase:P5, area:qa, wave:W2, taskmaster | 2026-02-28T17:55:58Z |
| #332 | [P5-W2-S3-03] Execute rollback drill: canary with error injection | roadmap, phase:P5, area:qa, wave:W2, taskmaster | 2026-02-28T17:56:00Z |
| #333 | [P5-W2-S3-04] Execute rollback drill: TS sidecar crash recovery | roadmap, phase:P5, area:qa, wave:W2, taskmaster | 2026-02-28T17:56:02Z |
| #334 | [P5-W2-S3-05] Execute rollback drill: database migration rollback | roadmap, phase:P5, area:infra, wave:W2, taskmaster | 2026-02-28T17:56:04Z |
| #335 | [P5-W2-S3-06] Execute rollback drill: admin-web Vercel instant rollback | roadmap, phase:P5, area:infra, wave:W2, taskmaster | 2026-02-28T17:56:06Z |
| #336 | [P5-W2-S3-07] Measure end-to-end rollback timing for all services | roadmap, phase:P5, area:qa, wave:W2, taskmaster | 2026-02-28T17:56:08Z |
| #337 | [P5-W3-S1-01] Compile release candidate version matrix | roadmap, phase:P5, area:product, wave:W3, taskmaster | 2026-02-28T17:56:11Z |
| #338 | [P5-W3-S1-02] Run final cargo audit and address any new advisories | roadmap, phase:P5, area:qa, wave:W3, taskmaster | 2026-02-28T17:56:13Z |
| #339 | [P5-W3-S1-03] Run TruffleHog secret scan and resolve any findings | roadmap, phase:P5, area:qa, wave:W3, taskmaster | 2026-02-28T17:56:15Z |
| #340 | [P5-W3-S1-04] Verify all P1-P4 gate criteria are still met | roadmap, phase:P5, area:qa, wave:W3, taskmaster | 2026-02-28T17:56:17Z |
| #341 | [P5-W3-S1-05] Verify Railway environment variables match production requirements | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:19Z |
| #342 | [P5-W3-S1-06] Verify Sentry project configuration and alert routing | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:21Z |
| #343 | [P5-W3-S1-07] Create release candidate tag and changelog entry | roadmap, phase:P5, area:product, wave:W3, taskmaster | 2026-02-28T17:56:23Z |
| #344 | [P5-W3-S1-08] Verify all CI pipeline gates are green on release branch | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:25Z |
| #345 | [P5-W3-S1-09] Create GitHub Release draft with changelog and migration notes | roadmap, phase:P5, area:product, wave:W3, taskmaster | 2026-02-28T17:56:28Z |
| #346 | [P5-W3-S2-01] Deploy release candidate to staging and freeze code changes | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:30Z |
| #347 | [P5-W3-S2-02] Configure 72h automated monitoring dashboard | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:32Z |
| #348 | [P5-W3-S2-03] Run continuous smoke test loop during stability window | roadmap, phase:P5, area:qa, wave:W3, taskmaster | 2026-02-28T17:56:34Z |
| #349 | [P5-W3-S2-04] Validate Prometheus alert rules fire correctly during stability window | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:36Z |
| #350 | [P5-W3-S2-05] Compile 72h stability window report | roadmap, phase:P5, area:product, wave:W3, taskmaster | 2026-02-28T17:56:38Z |
| #351 | [P5-W3-S2-06] Establish Sentry error rate baseline for 72h stability window | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:40Z |
| #352 | [P5-W3-S2-07] Monitor TS sidecar memory and CPU during stability window | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:42Z |
| #353 | [P5-W3-S3-01] Update API documentation with all P1-P5 endpoint changes | roadmap, phase:P5, area:product, wave:W3, taskmaster | 2026-02-28T17:56:44Z |
| #354 | [P5-W3-S3-02] Update deployment documentation with Railway and Vercel procedures | roadmap, phase:P5, area:product, wave:W3, taskmaster | 2026-03-01T20:00:57Z |
| #357 | [P5-W3-S3-05] Execute Gate E sign-off meeting | roadmap, phase:P5, area:product, wave:W3, taskmaster | 2026-02-28T17:56:54Z |
| #358 | [P5-W3-S3-06] Tag final release v1.0.0 and trigger release pipeline | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:56Z |
| #359 | [P5-W3-S3-07] Execute production deploy and final smoke validation | roadmap, phase:P5, area:infra, wave:W3, taskmaster | 2026-02-28T17:56:59Z |
| #360 | [P5-W3-S3-08] Send release announcement with operational status | roadmap, phase:P5, area:product, wave:W3, taskmaster | 2026-02-28T17:57:01Z |
| #361 | [V22-W1-S1-01] Extract types into dedicated types.rs module | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:03Z |
| #362 | [V22-W1-S1-02] Extract calculation functions into calculator.rs module | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:06Z |
| #363 | [V22-W1-S1-03] Implement personal year cycle calculation | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:08Z |
| #364 | [V22-W1-S1-04] Implement personal month and personal day cycles | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:11Z |
| #365 | [V22-W1-S1-05] Add personal cycles to NumerologyResult and engine output | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:13Z |
| #366 | [V22-W1-S1-06] Enhance witness prompt with personal cycle awareness | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:15Z |
| #367 | [V22-W1-S1-07] Update cache_key to include personal cycle inputs | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:17Z |
| #368 | [V22-W1-S2-01] Extract types into dedicated types.rs module | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:19Z |
| #369 | [V22-W1-S2-02] Extract calculation functions into calculator.rs module | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:22Z |
| #370 | [V22-W1-S2-03] Add Aesthetic (43-day) secondary rhythm cycle | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:24Z |
| #371 | [V22-W1-S2-04] Add Spiritual (53-day) secondary rhythm cycle | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:26Z |
| #372 | [V22-W1-S2-05] Include secondary rhythms in critical day detection | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:28Z |
| #373 | [V22-W1-S2-06] Include secondary rhythms in forecast output | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:30Z |
| #374 | [V22-W1-S2-07] Implement compatibility biorhythm calculation for two people | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:33Z |
| #375 | [V22-W1-S2-08] Add compatibility mode to engine via options parameter | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:35Z |
| #376 | [V22-W1-S2-09] Update validate() for secondary cycles and compatibility fields | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:37Z |
| #377 | [V22-W1-S2-10] Enhance witness prompt with secondary rhythms and compatibility insight | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:39Z |
| #378 | [V22-W1-S3-01] Define meridian analysis data models | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:41Z |
| #379 | [V22-W1-S3-02] Implement meridian energy calculator using Chinese medicine clock | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:43Z |
| #380 | [V22-W1-S3-03] Integrate meridian analysis into BiofieldEngine calculate() | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:45Z |
| #381 | [V22-W1-S3-04] Define aura layer data models | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:48Z |
| #382 | [V22-W1-S3-05] Implement aura layer assessment based on chakra and planetary data | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:50Z |
| #383 | [V22-W1-S3-06] Integrate aura layer assessment into BiofieldEngine output | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:52Z |
| #384 | [V22-W1-S3-07] Define healing modality recommendation data model | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:54Z |
| #385 | [V22-W1-S3-08] Implement healing recommendation engine based on biofield state | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:56Z |
| #386 | [V22-W1-S3-09] Integrate healing recommendations into BiofieldEngine output | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:57:58Z |
| #387 | [V22-W1-S3-10] Update validate() for meridian, aura, and healing fields | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:58:01Z |
| #388 | [V22-W1-S3-11] Enhance witness prompt with meridian, aura, and healing awareness | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:58:03Z |
| #389 | [V22-W1-S3-12] Update cache_key for expanded biofield output determinism | roadmap, phase:v2.2.0, area:backend, wave:W1, taskmaster | 2026-02-28T17:58:05Z |
| #390 | [V22-W2-S4-01] Add numerology personal cycles to birth-blueprint workflow synthesis | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:07Z |
| #391 | [V22-W2-S4-02] Add personal day to daily-practice workflow synthesis | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:09Z |
| #392 | [V22-W2-S4-03] Update numerology API endpoint documentation with personal cycles | roadmap, phase:v2.2.0, area:product, wave:W2, taskmaster | 2026-02-28T17:58:11Z |
| #393 | [V22-W2-S5-01] Update daily-practice workflow to include secondary rhythms in synthesis | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:14Z |
| #394 | [V22-W2-S5-02] Register compatibility mode in full-spectrum workflow | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:16Z |
| #395 | [V22-W2-S5-03] Update biorhythm API documentation for secondary cycles and compatibility | roadmap, phase:v2.2.0, area:product, wave:W2, taskmaster | 2026-02-28T17:58:18Z |
| #396 | [V22-W2-S6-01] Add biofield meridian data to daily-practice workflow synthesis | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:20Z |
| #397 | [V22-W2-S6-02] Add biofield healing recommendations to self-inquiry workflow | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:22Z |
| #398 | [V22-W2-S6-03] Update full-spectrum workflow to leverage expanded biofield subsystems | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:24Z |
| #399 | [V22-W2-S6-04] Update biofield API documentation for meridian, aura, and healing fields | roadmap, phase:v2.2.0, area:product, wave:W2, taskmaster | 2026-02-28T17:58:26Z |
| #400 | [V22-W2-S7-01] Add numerology to daily-practice workflow engine_ids in registry | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:29Z |
| #401 | [V22-W2-S7-02] Add biofield to self-inquiry workflow engine_ids in registry | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:31Z |
| #402 | [V22-W2-S7-03] Update full-spectrum engine list to reflect all secondary rhythms | roadmap, phase:v2.2.0, area:backend, wave:W2, taskmaster | 2026-02-28T17:58:33Z |
| #403 | [V22-W3-S8-01] Add personal cycle reference validation tests | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:35Z |
| #404 | [V22-W3-S8-02] Add edge case tests for numerology (empty name, unicode, special chars) | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:38Z |
| #405 | [V22-W3-S8-03] Create numerology benchmark suite | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:40Z |
| #406 | [V22-W3-S8-04] ConsciousnessEngine contract test for numerology | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:43Z |
| #407 | [V22-W3-S9-01] Add secondary rhythm cycle validation tests | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:45Z |
| #408 | [V22-W3-S9-02] Add compatibility biorhythm validation tests | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:47Z |
| #409 | [V22-W3-S9-03] Create biorhythm benchmark suite | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:49Z |
| #410 | [V22-W3-S9-04] ConsciousnessEngine contract test for biorhythm | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:51Z |
| #411 | [V22-W3-S10-01] Add meridian analysis validation tests | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:53Z |
| #412 | [V22-W3-S10-02] Add aura layer assessment validation tests | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:56Z |
| #413 | [V22-W3-S10-03] Add healing recommendation validation tests | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:58:58Z |
| #414 | [V22-W3-S10-04] Create biofield benchmark suite | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:59:00Z |
| #415 | [V22-W3-S10-05] ConsciousnessEngine contract test for biofield | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:59:02Z |
| #416 | [V22-W3-S11-01] End-to-end workflow integration test for daily-practice with all 4 engines | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:59:04Z |
| #417 | [V22-W3-S11-02] End-to-end workflow integration test for full-spectrum with expanded engines | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:59:06Z |
| #418 | [V22-W3-S11-03] Cross-engine cache key collision test | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:59:09Z |
| #419 | [V22-W3-S11-04] Engine registration parity test for v2.2.0 | roadmap, phase:v2.2.0, area:qa, wave:W3, taskmaster | 2026-02-28T17:59:11Z |
| #420 | [V22-W3-S11-05] Write v2.2.0 engine specification document | roadmap, phase:v2.2.0, area:product, wave:W3, taskmaster | 2026-02-28T17:59:13Z |
| #421 | [V22-W3-S11-06] Update CHANGELOG and version bump for v2.2.0 | roadmap, phase:v2.2.0, area:product, wave:W3, taskmaster | 2026-02-28T17:59:15Z |
| #455 | [V30-W2-S3-03] Build usage aggregation queries in UsageRepository | roadmap, taskmaster | 2026-02-28T18:00:32Z |
| #459 | [V30-W3-S1-01] Run cargo audit and fix all non-ignored advisories | roadmap, phase:v3.0.0, area:infra, wave:W3, taskmaster | 2026-02-28T18:00:40Z |
| #460 | [V30-W3-S1-02] Audit JWT token handling for OWASP JWT best practices | roadmap, phase:v3.0.0, area:backend, wave:W3, taskmaster | 2026-02-28T18:00:43Z |
| #461 | [V30-W3-S1-03] Add API key scoping and permission model | roadmap, phase:v3.0.0, area:backend, wave:W3, taskmaster | 2026-02-28T18:00:45Z |
| #462 | [V30-W3-S1-04] Audit and harden input validation across all API endpoints | roadmap, phase:v3.0.0, area:backend, wave:W3, taskmaster | 2026-02-28T18:00:48Z |
| #463 | [V30-W3-S1-05] Add CORS configuration hardening for production domains | roadmap, phase:v3.0.0, area:infra, wave:W3, taskmaster | 2026-02-28T18:00:50Z |
| #464 | [V30-W3-S1-06] Implement request signing for SDK-to-API communication | roadmap, phase:v3.0.0, area:backend, wave:W3, taskmaster | 2026-02-28T18:00:52Z |
| #465 | [V30-W3-S1-07] Run npm audit on TypeScript sidecar and admin-web dependencies | roadmap, phase:v3.0.0, area:infra, wave:W3, taskmaster | 2026-02-28T18:00:55Z |
| #466 | [V30-W3-S2-01] Build Raycast extension with engine quicklaunch and daily practice command | roadmap, phase:v3.0.0, area:frontend, wave:W3, taskmaster | 2026-02-28T18:00:57Z |
| #467 | [V30-W3-S2-02] Build macOS menu bar app scaffold with Tauri and noesis-sdk | roadmap, phase:v3.0.0, area:frontend, wave:W3, taskmaster | 2026-02-28T18:00:59Z |
| #468 | [V30-W3-S2-03] Add profile configuration and auth flow to menu bar app | roadmap, phase:v3.0.0, area:frontend, wave:W3, taskmaster | 2026-02-28T18:01:01Z |
| #469 | [V30-W3-S2-04] Add CLI installation and update mechanism for noesis-tui | roadmap, phase:v3.0.0, area:infra, wave:W3, taskmaster | 2026-02-28T18:01:03Z |
| #470 | [V30-W3-S3-01] Build marketing landing page with engine showcase and pricing tiers | roadmap, phase:v3.0.0, area:frontend, wave:W3, taskmaster | 2026-02-28T18:01:06Z |
| #471 | [V30-W3-S3-02] Implement automated changelog generation from git commits | roadmap, phase:v3.0.0, area:infra, wave:W3, taskmaster | 2026-02-28T18:01:08Z |
| #472 | [V30-W3-S3-03] Write v2-to-v3 API migration guide | roadmap, phase:v3.0.0, area:product, wave:W3, taskmaster | 2026-03-13T07:49:52Z |
| #474 | [V30-W3-S3-05] Create v3.0.0-beta.1 release candidate with version bumps | roadmap, phase:v3.0.0, area:infra, wave:W3, taskmaster | 2026-02-28T18:01:15Z |
| #475 | [V30-W3-S3-06] Run full end-to-end regression test suite against beta deployment | roadmap, phase:v3.0.0, area:qa, wave:W3, taskmaster | 2026-02-28T18:01:17Z |
| #476 | [V30-W3-S3-07] Conduct load test with tiered rate limits under concurrent users | roadmap, phase:v3.0.0, area:qa, wave:W3, taskmaster | 2026-02-28T18:01:19Z |
| #477 | [V30-W3-S3-08] Write operational runbook for v3.0.0 launch day | roadmap, phase:v3.0.0, area:infra, wave:W3, taskmaster | 2026-03-02T23:45:04Z |
| #501 | Engine hygiene follow-up: astrology validity regressions (Mar 8, 2026) |  | 2026-03-08T09:39:59Z |
| #504 | Human Design: fix design-side (prenatal) activation drift causing wrong type/authority |  | 2026-03-08T07:35:25Z |
| #505 | Gene Keys: align design-side keys with Human Design activation source of truth |  | 2026-03-08T07:35:26Z |
| #506 | Sigil-forge: investigate and fix runtime 500 (BRIDGE_ERROR) on engine execution |  | 2026-03-08T09:07:39Z |
| #508 | Vedic provider wiring gap: runtime engines ignore FREE_ASTROLOGY_API/VIDER settings |  | 2026-03-08T09:39:51Z |
