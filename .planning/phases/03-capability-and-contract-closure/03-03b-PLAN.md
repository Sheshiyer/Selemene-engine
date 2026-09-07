---
phase: 03-capability-and-contract-closure
plan: "03b"
type: execute
wave: 3
depends_on: ["03-03"]
files_modified:
  - packages/noesis-engine-sdk/package.json
  - packages/noesis-sdk-ts/package.json
  - apps/admin-web/package.json
  - packages/witness-pipeline/package.json
  - packages/verification/package.json
  - pnpm-lock.yaml
  - packages/noesis-engine-sdk/dist/index.js
  - packages/noesis-engine-sdk/dist/index.d.ts
autonomous: true
requirements: [CON-01, CON-02]
must_haves:
  truths:
    - "Actual pnpm workspace consumers resolve the sole shared decoder through a locked workspace dependency."
    - "The generated shared SDK JavaScript and declarations exist before any SDK, admin, Witness or verification consumer runs."
    - "The independent Bun ts-engines project remains outside the pnpm workspace and reuses the Plan 03 source authority without a package dependency claim."
  artifacts:
    - path: packages/noesis-engine-sdk/package.json
      provides: "Public SDK package metadata and dist exports"
    - path: pnpm-lock.yaml
      provides: "Locked workspace links for existing consumers"
    - path: packages/noesis-engine-sdk/dist/index.js
      provides: "Generated shared decoder runtime entrypoint"
    - path: packages/noesis-engine-sdk/dist/index.d.ts
      provides: "Generated shared decoder declarations"
  key_links:
    - from: packages/noesis-sdk-ts/package.json
      to: packages/noesis-engine-sdk/package.json
      via: "@selemene/engine-sdk workspace:* dependency"
    - from: packages/noesis-engine-sdk/src/index.ts
      to: packages/noesis-engine-sdk/dist/index.js
      via: "pnpm --filter @selemene/engine-sdk build"
    - from: pnpm-workspace.yaml
      to: ts-engines/package.json
      via: "explicit exclusion from the pnpm workspace"
---

<objective>Wire the shared engine SDK into actual pnpm workspace consumers and produce its public build before consumer plans execute.</objective>
<execution_context>
@/Users/sheshnarayaniyer/.codex/get-shit-done/workflows/execute-plan.md
@/Users/sheshnarayaniyer/.codex/get-shit-done/templates/summary.md
</execution_context>
<context>
@.planning/PROJECT.md
@.planning/phases/03-capability-and-contract-closure/03-CONTEXT.md
@.planning/phases/03-capability-and-contract-closure/03-RESEARCH.md
@packages/noesis-engine-sdk/package.json
@packages/noesis-engine-sdk/src/index.ts
@packages/noesis-engine-sdk/src/contract-v1.ts
@packages/noesis-sdk-ts/package.json
@apps/admin-web/package.json
@packages/witness-pipeline/package.json
@packages/verification/package.json
@pnpm-workspace.yaml
@pnpm-lock.yaml
</context>
<tasks>
<task type="auto">
  <name>Task 1: Declare locked workspace SDK links</name>
  <files>packages/noesis-engine-sdk/package.json, packages/noesis-sdk-ts/package.json, apps/admin-web/package.json, packages/witness-pipeline/package.json, packages/verification/package.json, pnpm-lock.yaml</files>
  <read_first>packages/noesis-engine-sdk/package.json, packages/noesis-sdk-ts/package.json, apps/admin-web/package.json, packages/witness-pipeline/package.json, packages/verification/package.json, pnpm-workspace.yaml, pnpm-lock.yaml, packages/noesis-engine-sdk/src/index.ts</read_first>
  <action>Declare `@selemene/engine-sdk: workspace:*` under runtime dependencies in the four actual pnpm workspace consumers that import the shared public decoder: `packages/noesis-sdk-ts/package.json`, `apps/admin-web/package.json`, `packages/witness-pipeline/package.json` and `packages/verification/package.json`. Preserve the existing shared SDK package name, exports and build metadata without adding a self-dependency or any new package. Do not modify `ts-engines/package.json`: `pnpm-workspace.yaml` excludes that independent Bun project, whose tests use the explicit Plan 03 source import. Update only the workspace link metadata in `pnpm-lock.yaml` with existing packages by running `pnpm install --lockfile-only --offline`, then verify the frozen workspace install; do not hand-edit lockfile resolution data or alter any lockfile owned by the Bun/Python projects.</action>
  <acceptance_criteria>
    - The four listed pnpm consumers each declare exactly `@selemene/engine-sdk: workspace:*` under runtime dependencies, and `ts-engines/package.json` remains outside this workspace change.
    - `pnpm-workspace.yaml` still excludes `ts-engines`, and the lockfile contains only the required existing workspace links.
    - Frozen offline workspace installation succeeds without a new external dependency.
  </acceptance_criteria>
  <verify><automated>pnpm install --lockfile-only --offline &amp;&amp; pnpm install --offline --ignore-scripts --frozen-lockfile &amp;&amp; node -e "const fs=require('fs'); const ws=fs.readFileSync('pnpm-workspace.yaml','utf8'); if (/ts-engines/.test(ws)) process.exit(1); for (const p of ['packages/noesis-sdk-ts/package.json','apps/admin-web/package.json','packages/witness-pipeline/package.json','packages/verification/package.json']) { const j=require('./'+p); if (j.dependencies?.['@selemene/engine-sdk'] !== 'workspace:*') process.exit(1); }"</automated></verify>
  <done>Only actual pnpm workspace consumers have the locked shared SDK link, while the independent Bun project remains excluded.</done>
</task>
<task type="auto">
  <name>Task 2: Build and receipt the public SDK boundary</name>
  <files>packages/noesis-engine-sdk/dist/index.js, packages/noesis-engine-sdk/dist/index.d.ts</files>
  <read_first>packages/noesis-engine-sdk/package.json, packages/noesis-engine-sdk/src/index.ts, packages/noesis-engine-sdk/src/contract-v1.ts, packages/noesis-engine-sdk/tests/contract-v1.test.ts, packages/noesis-sdk-ts/package.json, apps/admin-web/package.json, packages/witness-pipeline/package.json, packages/verification/package.json</read_first>
  <action>Run the existing shared package build `pnpm --filter @selemene/engine-sdk build` after Task 1's frozen install. Verify the generated `packages/noesis-engine-sdk/dist/index.js` and `packages/noesis-engine-sdk/dist/index.d.ts` are non-empty and expose the public decoder boundary consumed by Plans 11, 12, 18 and 19. Treat both dist files as generated artifacts: do not hand-edit them, add source behavior only in Plan 03, and preserve the package's existing build script and exports.</action>
  <acceptance_criteria>
    - The shared SDK build succeeds from the locked workspace.
    - Both generated dist outputs exist, are non-empty and contain the public exports required by downstream consumer tests.
    - Consumer plans 11, 12, 18 and 19 depend transitively on this build receipt before their focused commands run.
  </acceptance_criteria>
  <verify><automated>pnpm --filter @selemene/engine-sdk build &amp;&amp; test -s packages/noesis-engine-sdk/dist/index.js &amp;&amp; test -s packages/noesis-engine-sdk/dist/index.d.ts &amp;&amp; rg -n "decodeEngineCapabilityList" packages/noesis-engine-sdk/dist/index.js packages/noesis-engine-sdk/dist/index.d.ts &amp;&amp; rg -n "decodeWorkflowOutcome" packages/noesis-engine-sdk/dist/index.js packages/noesis-engine-sdk/dist/index.d.ts</automated></verify>
  <done>Generated shared SDK runtime and declaration artifacts are available before every downstream consumer.</done>
</task>
</tasks>
<threat_model>
## Assets
Workspace dependency provenance, strict decoder exports, lock integrity and generated SDK runtime/declaration artifacts.
## Trust Boundaries
Package manifests and lock metadata cross into pnpm resolution; generated dist crosses into consumer module loading.
## Abuse Cases
A consumer silently resolves an ambient or stale SDK, a lockfile gains an unreviewed package, or a hand-edited dist artifact diverges from the sole source decoder.
## STRIDE Threat Register
| Threat | Category | Disposition | Blocking test |
|---|---|---|---|
| T3-01 | Spoofing | mitigate | Frozen workspace install and public import receipts. |
| T3-02 | Information Disclosure | mitigate | Shared SDK decoder tests remain the only runtime reader authority. |
| T3-03 | Tampering/Availability | mitigate | Generated dist non-empty receipt before consumer plans. |
| T3-04 | Tampering | mitigate | Workspace manifest and lock provenance assertion. |
| T3-05 | Improper Inventory | mitigate | Consumer links preserve canonical 19/17 projections. |
| T3-06 | Elevation of Privilege | mitigate | No self-dependency or unreviewed package resolution. |
| T3-07 | Tampering | mitigate | Public `decodeWorkflowOutcome` export receipt. |
| T3-08 | Tampering | mitigate | Consumer import checks use the generated public boundary. |
</threat_model>
<verification>Run the locked workspace install, then the shared SDK build and generated-output receipt before any consumer wave.</verification>
<success_criteria>The actual pnpm workspace consumers resolve the exported shared decoder through a locked link, and generated dist files are proved ready before plans 11, 12, 18 and 19.</success_criteria>
<output>Create `.planning/phases/03-capability-and-contract-closure/03-03b-SUMMARY.md`.</output>
