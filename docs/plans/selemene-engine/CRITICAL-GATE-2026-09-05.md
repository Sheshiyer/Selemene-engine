# Critical repository protection decision

**Decision state: APPLIED 2026-09-06T10:17:48Z.** The user explicitly approved the additive repository protection change. Immediately before mutation, draft PR #1488 remained at `7a5793d94bc850976f29db9277527e475d7f127b`, run 33973728459 still passed all 16 jobs, and ruleset `15597830` exactly matched the preserved baseline. The update response, a separate fresh readback, and resolved rules for `main` all show strict `CI Gate` with GitHub Actions integration `15368`. Existing deletion and non-fast-forward protections, conditions and the empty bypass list remain intact.

The applied main-branch CI rule and rollback body are in [MAIN-CI-RULE-PROPOSAL.json](MAIN-CI-RULE-PROPOSAL.json). The rule requires an up-to-date successful `CI Gate` check from GitHub Actions app `15368`; it preserves the existing deletion/force-push rules, conditions and empty bypass list. No review-count or deployment policy changes were included.

The application precondition used the reviewed source `7a5793d94bc850976f29db9277527e475d7f127b`, which passes all 16 jobs in [run 33973728459](https://github.com/Sheshiyer/Selemene-engine/actions/runs/33973728459), including CI Gate from GitHub Actions app `15368`; its PR merge tree equals the source tree. Fresh queries immediately before applying prevented a dated receipt from serving as current evidence.

## Candidate and remaining promotion conditions

PR #1488 consolidates #1486 and #1487 plus the scoped recovery repairs. Both original branches remain available. The recovery candidate is draft and has no auto-merge request. A single recovery promotion is the intended eventual merge path; do not merge all three overlapping branches separately.

Production promotion remains **HOLD**, independently of the protection decision:

- Main updates trigger the CD workflow, GHCR image publication, Railway source deployment and Vercel production integration. The changed plan-sync workflow also matches its main-push trigger. Its explicit dry run against this tree finds no `docs/planning/*.json` inputs, so it has no issue rows to create, update, close or reopen.
- The CD workflow builds tested GHCR images but the Railway job deploys source. Equivalence between the built artifact and deployed image, production schema identity, and executable rollback receipts remain original Wave 1/6 obligations.
- Railway targets are project `11eedde4-41e6-4f51-b86b-cf77111cf592`, production environment `702b945e-2c66-4d5a-bae1-4c67ea14c3bb`: API service `48b3bd23-5620-4f7b-8e5d-96bc5c5d7fc4`, TS `94419a41-9003-4a31-8bfe-d55b39ca4cb2`, CV `f596e31b-e190-409c-993d-a3b618d29a73`. The API source revision is still unknown. Source changes are not proof that every service has deployed them.
- Known pre-promotion deployment pointers are API `075f03de-7b60-42fd-8cc2-e0119d5ff2c9`, TS `e13dcbfa-fc0d-4493-85fc-3f3799f6b6cd`, CV `11bcc687-9c4b-4682-b5ff-bbba49f824d3`, and Vercel production `dpl_HhkGqfwkFw4u9YG9ebFe2GfKoe7C`. The Vercel build log identifies main source `ae3e2ce`. Available image digests are in [INFRASTRUCTURE-MAP.json](INFRASTRUCTURE-MAP.json). These are rollback inventory pointers, not a demonstrated rollback.
- The preview deployment `dpl_GUAWAUq8hEo7ka1jNU38oVERCaPC` for source `19b8082` reached Vercel READY. Browser acceptance remains pending because deployment protection requests a separate Vercel login. The authenticated production admin session remains evidence of the existing production deployment.
- Do not create pattern-memory resources from its placeholder IDs, change DNS, mutate schema/data, or claim native/conditional engine parity from this recovery slice.

The public repository Actions variable `ADMIN_WEB_URL` was absent. It now points to the verified existing origin `https://144.tryambakam.space`; readback and the existing unauthenticated smoke script pass. This configures the next required smoke check and does not itself run a deployment. Railway secret names and the smoke API-key name are present; secret values were not read or changed, and token scope was not inferred from their names.

## Continuation after the decision

The approved protection rule has been applied and read back. Continue the remaining registry, immutable release and schema/rollback evidence work in Phase 2 before moving into broad Phase 3/4 execution. All 570 engine issues and the original wave exits retain their individual acceptance obligations.
