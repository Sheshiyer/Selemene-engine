# Free plugin publication: factual policy preparation

**DRAFT FOR REVIEW — NOT EFFECTIVE OR PUBLISHED.** Prepared 2026-10-07. These materials propose factual page updates for Selemene Engine 0.2.0. They do not supply a legal notice, approved terms, an effective date, or answers to the publisher questions currently pending. No website files or live resources were changed during preparation.

## Proposed pages and current live coverage

| Proposed copy | Intended route | Website source destination |
| --- | --- | --- |
| [Home](policy-drafts/home.md) | https://www.tryambakam.space/selemene/ | `public/selemene/index.html` |
| [Readings](policy-drafts/readings.md) | https://www.tryambakam.space/selemene/readings/ | `public/selemene/readings/index.html` |
| [Support](policy-drafts/support.md) | https://www.tryambakam.space/selemene/support/ | `public/selemene/support/index.html` |
| [Privacy](policy-drafts/privacy.md) | https://www.tryambakam.space/selemene/privacy/ | `public/selemene/privacy/index.html` |
| [Terms](policy-drafts/terms.md) | https://www.tryambakam.space/selemene/terms/ | `public/selemene/terms/index.html` |

Fresh unauthenticated HTTP reads on 2026-10-07 returned 200 for all five routes, with exact byte equality to the clean local website source. The live privacy, terms and support pages explicitly remain unpublished review drafts without effective dates. Their personal-account/OAuth copy does not describe the sponsored connector: privacy promises the possibility of revisiting personal history, and support requires the customer's own key and says a shared operator key cannot substitute. The home/readings pages still describe an integration in preparation. These statements require factual replacement before representing the pages as coverage for the free release.

The current plugin manifest includes `websiteURL`. Its `supportURL`, `privacyPolicyURL` and `termsOfServiceURL` remain absent. Merely adding reachable URLs would not resolve the draft status, missing commitments, or contradictory copy.

## Implemented facts the drafts may describe

- The free connector requires no customer Selemene account, login, OAuth approval or personal API key. Its sponsor credential is privately installed as a Cloudflare Worker secret. Neither the distributable plugin nor customer tool arguments/results contain that credential.
- ChatGPT's current model supplies explanations, Aletheios/Pichet writing perspectives, interpretation and final prose, subject to the user's applicable ChatGPT limits. Selemene supplies calculations, local Tarot/I Ching draws, static framework meanings and reviewed rule-based synthesis. The reviewed connector does not invoke a hosted inference model, witness pipeline, media analyzer, assets generator or paid generation endpoint. This statement applies to this connector, not to every product in the wider repository.
- The eight tools are discovery, engine information, input preparation, engine calculation, workflow discovery, workflow information, workflow execution and the separate Vedic D1/D9 chart. The reviewed non-media engine IDs are `numerology`, `human-design`, `gene-keys`, `vimshottari`, `panchanga`, `vedic-clock`, `biorhythm`, `transits`, `enneagram`, `tarot` and `i-ching`. The allowlisted workflow is `decision-support`; availability and composition remain checked against the live response. Do not describe every engine as deterministic or every calculation as birth-derived.
- Before calculation, disclose: **Free readings use a shared Selemene administrator account. Birth inputs and calculated results may be stored there; usage, profile, XP and phase may change. There is no separate private user history.** Ordinary calculations/workflows attempt asynchronous persistence; successful calculation does not confirm storage. The connector exposes no personal history, account administration or deletion tool.
- The separate fixed Vedic chart handler does not save a personal reading/history by that handler. This narrower route fact is not an end-to-end no-storage promise. Its output excludes placeholder combustion and does not supply advanced dignities, yogas, Atmakaraka or Darakaraka. It is not a `vedic-kundali` API engine or the richer local supplement.
- Input preparation is local. Discussion/preparation is not execution. Required missing birth time, location or timezone must be requested rather than invented. Only schema-supported fields are sent to calculations; optional contextual writing instructions can stay in the ChatGPT conversation.
- Free access is bounded by service availability and default quotas: 120 requests and 30 calculation attempts per connecting IP per minute; 600 requests and 60 calculation attempts for the entire service per minute; 1,000 calculation attempts for the service per UTC day. These are configurable defaults, not per-person allowances. Shared NAT or ChatGPT gateway addresses can share a quota. An attempt is charged after known-tool and outer-input validation; later preflight/backend failure is not refunded. Do not silently retry ambiguous calculation results or invent retry/reset times.
- Worker request bodies are capped at 32 KiB; upstream/final responses at 256 KiB. Metadata requests have 15-second timeouts and calculations 45-second timeouts. The edge returns sanitized errors and refuses redirects to other destinations.

## Processing and storage boundaries

Requests cross ChatGPT/OpenAI, the Cloudflare sponsored MCP Worker and the Selemene Rust API on Railway. Upstream database/cache infrastructure can process or retain calculation data. This identifies known surfaces; it is not a complete processor inventory, a hosting-region disclosure or a cross-border assurance.

The Worker Durable Object persists counters, timestamps and salted HMAC hashes of Cloudflare's trusted connecting IP. It holds at most 512 active hash entries; inactive entries are pruned using a two-minute TTL during requests and Durable Object alarms. The global daily calculation counter survives IP-entry cleanup and resets by UTC day. The reviewed Worker source does not log or persist raw IPs, reading inputs/results or sponsor credentials. Raw IP is still processed at the Cloudflare edge, and this source-level statement does not establish Cloudflare, Railway, OpenAI or other provider logging/retention practices.

The existing scoped backend lifecycle audit identifies reading/profile/usage persistence and request metadata logging, including method, path, trace identifier, account identifier, status and duration. It found no general customer/reading deletion endpoint, general export function or scheduled reading/profile/usage purge. It also records configured error monitoring with unverified event delivery, payload/redaction and retention. These are bounded audit findings, not a deletion impossibility statement or current provider-retention guarantee. No promised retention period, deletion SLA, backup-erasure behavior, legal basis or processor region can be inferred from them.

The only currently published technical contact is the public [Selemene issue tracker](https://github.com/Sheshiyer/Selemene-engine/issues), which was reachable in the fresh read. It must not receive birth details, full personal readings, credentials, private logs or account/deletion requests. No private support/privacy/deletion address or handling process was found on the live pages. A new channel must come from the publisher's actual answer and be verified before publication; do not substitute a guessed email or the public tracker.

## Answers still pending

| Fact | Current state | How to use the eventual answer |
| --- | --- | --- |
| Publisher/controller identity, address and jurisdiction | **PENDING** | Use the explicitly selected identity and actual contact details; portal availability alone is not a controller designation. |
| Public/private support and privacy contact; request owner | **PENDING** | Verify the named channel works and state the actual handling process. |
| Countries, audience and minimum age | **PENDING** | Reflect the publisher's explicit targeting and permitted audience in both listing and pages. |
| Retention and access/deletion handling | **PENDING** | Specify only justified commitments that the operator can implement and verify for shared data, logs and backups. |
| Permitted-use rules, commerce declaration and legal commitments | **PENDING** | Do not infer pricing, purchases, warranties, liability, dispute rules or refunds from symbolic-reading code. |
| Processor inventory, regions and telemetry/backup details | **UNVERIFIED/PENDING** | Complete the actual inventory; keep provider practices distinct from the narrow Worker-source facts. |
| Effective dates, versioning and final policy approval | **PENDING** | Review the completed text against the deployed connector, then approve and publish the effective versions. |

The drafts keep these gaps visible. No new user questionnaire is introduced here; the parent is already awaiting the publisher/countries/privacy-contact/rules answers.

## Website checkout and deployment path — inspected without editing

- Actual checkout: `/Volumes/madara/2026/Projects/tryambakam-noesis/tryambakam-space`.
- Inspected local branch: `codex/selemene-plugin-pages`, tracking `origin/codex/selemene-plugin-pages`; working tree clean at the read. Local HEAD is `c6a18fd` (2026-09-30 deployment verification receipt), with page implementation `6feceab` and original draft `d9a1127` beneath it.
- Repository/PR evidence: `Sheshiyer/tryambakam-space`, PR 13 merged to `main` as `df7ced81b80cc7a76e3b4f3c353468124db2a8c8`; the engine's historical `website-main-live-checks.json` records this automatic production release. Local branch state and that historical receipt do not by themselves prove the present remote Git-integration settings.
- Fresh read-only Vercel project inspection: `tryambakam-space` in `sheshiyers-projects`, project `prj_It3eejxU1cGWz0hev1X2lyEQrHVG`, root `.`, Vite preset, Node 24.x. Build command is `npm run build` or `vite build`; local package defines `tsc -b && vite build`, producing a static `dist/` application including `public/selemene/`. No repository `.github/` workflow or `vercel.json` was found in this checkout.
- Fresh `vercel inspect https://www.tryambakam.space --scope sheshiyers-projects` read back production deployment `dpl_Ct5sR7cWonh36vyyree5qj3jTerE`, **Ready**, created 2026-09-30, with `www.tryambakam.space`, `tryambakam.space`, and the `git-main` alias. This matches the merged-main deployment recorded in the engine evidence.
- The website's historical deployment receipt also documents the supported staged CLI path: linked existing project, production candidate with `--prod --skip-domain`, verify candidate, then promote the exact deployment. No deployment command was run during this preparation. Any actual publication should use isolated website changes and verify the current release path rather than assuming a dirty checkout or historical deployment is safe to promote.

## Sources used for these drafts

- Current `plugins/selemene-engine/plugin.json`, onboarding/reading skills and bundled host reading flow.
- `workers/selemene-free-mcp/README.md`, `src/env.ts`, `src/rate-limiter.ts`, `src/mcp.ts`, `src/index.ts`, `src/sanitize.ts`, `src/vedic-chart.ts` and `wrangler.jsonc`.
- `docs/plans/free-plugin/deployment-receipt.json`, `plugin-receipt.json`, and `LIVE-CHECKS.md`.
- Historical bounded `docs/plans/chatgpt-plugin/DATA-LIFECYCLE-AUDIT.md`, `SUBMISSION-STATUS.md`, and `website-main-live-checks.json`; current free-flow statements replace their old customer-OAuth assumptions.
- Clean website source under `public/selemene/`, website deployment receipt and fresh read-only Vercel/HTTP inspections.

These documents are reversible source materials only. No effective policy, private contact, new legal commitment, marketplace approval or website publication is claimed.

## Preparation receipt

The preparation document and five Markdown page drafts were written only under this publication source path. The independent `installed-connectivity.json` in the same parent directory belongs to the host verification lane and was not edited. The website checkout remained clean after preparation; unrelated engine ISA edits were preserved.

A bounded loopback `noesis-build` attempt performed source reads but produced no accepted page files within approximately six minutes. Only that attempt's child processes were terminated; `/tmp/selemene-publication-drafts-build.log` preserves the canceled attempt. Actual provider resolution was not established. The page drafts were then completed directly from the verified fact set. No failed producer output was used as evidence of completion, and no shared router/service was restarted or reconfigured.

Content review checks that every page begins with visible draft status, names its canonical route and source destination, preserves shared-storage and host-interpretation boundaries, and explicitly marks relevant unresolved commitments. No executable implementation was changed, so no application test suite was rerun for these documentation-only additions.
