# Selemene Engine submission preparation

As of 2026-09-30: **customer MCP implemented and deployed; website MotionSkin draft verified locally; public packaging remains blocked on publisher, policy, reviewer and demo facts. No plugin upload, review submission or publication has occurred.**

## Prepared locally

- `plugins/selemene-engine/plugin.json`: Agent Plugins 1.0 identity/version 0.1.0, listing copy, three prompts, five positive and three negative cases, release notes. Subtitle `Personal symbolic reflection` is 28 characters.
- `plugins/selemene-engine/skills/selemene-reading/SKILL.md`: reflection-first workflow for all seven tools, input/auth authority, persistence disclosure, no blind retries and accurate partial-result handling.
- `plugins/selemene-engine/assets/icon.png`: 512 × 512 transparent PNG derived from the existing Tryambakam sigil with preserved geometry/color. Asset is an existing brand conversion, not a generated substitute. The parent inspected256px and48px versions on light/dark backgrounds; see ICON-REVIEW.md. Actual directory/composer placement remains unverified.
- `plugins/selemene-engine/scripts/create-package.py`: offline readiness validator and public-upload builder. Missing endpoint, publisher, URL/demo/publication facts or verification receipts prevent ZIP creation. Upload inventory excludes scripts, private evidence and account bindings.
- `REVIEW-CASES.md`: all eight connected-host cases explicitly **Not run**.
- `DEMO-WALKTHROUGH.md`: concrete rehearsal/recording sequence; no recording claimed.

## Required facts and evidence still pending

| Item | Required next evidence |
| --- | --- |
| Publisher identity | User-selected verified individual/business identity; verify intended portal organization/name |
| Country targeting | Explicit chosen country codes or explicit all-countries choice; omission is intentional |
| Commerce | User-confirmed payment/purchase behavior and where payment occurs; do not infer from the codebase |
| Category | Supported category from the actual intended dashboard |
| Website/support | Actual public umbrella/plugin pages and functioning support arrangement, content inspected |
| Privacy/terms | User confirms no existing pages; website work is authorized separately. Publish content grounded in confirmed collection/use/sharing/retention/deletion facts and legal decisions |
| MCP server | Deployed endpoint/public metadata verified; local actual SDK tests pass. Still require real-account browser approval and connected ChatGPT calls |
| Review cases | Align final tool schemas, run in connected ChatGPT, record evidence against candidate manifest/deployment |
| Demo | Real successful interactions recorded, replayed and hosted at a verified reviewer-accessible URL |
| Reviewer access | Dedicated account and secure portal login instructions; never include credentials in ZIP |
| Final portal steps | Correct identity/domain verification, imported metadata, saved-version connection/tests, scans and developer-completed attestations |

`mcp.json` now points to the deployed, discovery-verified HTTPS endpoint at https://selemene-mcp.sheshnarayan-iyer.workers.dev/mcp. See DEPLOYMENT-RECEIPT.md. Local18-test OAuth/SDK suite passes; live public metadata and keyless browser denial pass. Successful real-account approval and connected ChatGPT calls remain unverified. No author identity, listing URLs, country targeting, commerce declaration, demo URL or category has been invented. Policy publication is not established by draft source text.

## Local verification

The initial `python3 plugins/selemene-engine/scripts/create-package.py --check` exits 1 with `NOT READY` and lists the expected missing facts/receipts. It creates no archive. Root metadata parses successfully; case count and string mappings, prompt lengths, icon PNG dimensions and supported source fields pass the local checks. This is preparation evidence, not MCP, connected-host, portal or published-policy evidence.

Four synthetic temporary-directory guard tests pass: candidate ZIP creation does not require already-uploaded portal cases; submission checking does; nonboolean commerce, a missing endpoint and an escaping asset-parent symlink all fail closed. These fixture receipts are synthetic and are not production evidence.

Candidate ZIP readiness uses local protocol verification and the required verified endpoint, facts and demo. `--submission-check` adds the connected review cases for the saved submission version, avoiding a circular dependency before the first upload.

Once facts arrive, write the supported fields into the manifest, save verification outside the public package, rerun checks, then build and inspect the ZIP. A ZIP can be prepared before portal access, but public submission still requires authenticated setup and authorized developer attestations. Do not describe an uploaded draft as submitted for review, or a submitted candidate as published.

## Website now live

The approved MotionSkin draft is deployed at https://www.tryambakam.space/selemene/ from website source6feceab, deploymentdpl_9YMGkwHxcV3C8ZTjtP5jABfyeBuS. All five routes and animation assets return200 and match source. Desktop/mobile and fallback checks pass. The package websiteURL is now set. The other listing URLs remain unset: public privacy/terms/support drafts do not yet provide effective policies or a functioning private support arrangement.

PREPARATION-EVIDENCE.json contains only the verified website, endpoint, local protocol/annotation and icon evidence for the exact current manifest. Unknown publisher/category/countries/commerce/demo facts remain missing. The validator still rejects a public ZIP; no empty or invented evidence is used to pass it.
