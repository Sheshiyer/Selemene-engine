# Running Selemene, Urania and Sankalpa work in Claude Code cloud sessions

This is the operating guide for running the remaining issues and roadmap
across `Sheshiyer/Selemene-engine`, `Sheshiyer/urania-137` and
`Sheshiyer/sankalpa` in Claude Code cloud sessions. It covers the environment,
the dispatch flow, OmniRoute, and the verification each repo needs now that
GitHub Actions is retired (#1493).

## 1. The environment (one-time, in the Claude app)

Create one environment named `thoughtseed`. Open the environment menu in a
session's title bar, then **Add environment** (or **Edit** on an existing
one).

| Setting | Value |
|---|---|
| Network access | **Custom**. Allowed domains: `omniroute.thoughtseed.space`. Keep **Also include default list of common package managers** ticked. |
| Environment variables | See below. No secrets here: anyone using the environment can read them. |
| Network secrets | `OMNIROUTE_API_KEY`: type Bearer, header `Authorization`, prefix `Bearer`, host `omniroute.thoughtseed.space`. |
| Setup script | The contents of [`setup.sh`](./setup.sh). |

Environment variables:

```bash
SWAGGER_UI_DOWNLOAD_URL=file:///root/.cache/selemene/swagger-ui-5.17.14.zip
SWISS_EPHEMERIS_PATH=/home/user/Selemene-engine/data/ephemeris
OMNIROUTE_BASE_URL=https://omniroute.thoughtseed.space/v1
OMNIROUTE_MODEL=<your OmniRoute model or combo id>
```

Add only the hosts a task actually needs. Live probes against staging or
production stay with you, so neither API host is on the allowlist.

### What runs at session start

- **Setup script** (cached for about 7 days): builds the Swagger UI zip, then
  runs every repo's SessionStart hook so dependencies are on disk in the
  snapshot.
- **Repo SessionStart hooks** (`.claude/hooks/session-start.sh` in each repo;
  single-repo sessions only, and a no-op outside the cloud):
  - **Selemene:** `pnpm install`, ts-engines `bun install`, `cargo fetch`, the
    Swagger zip, and it exports `SWAGGER_UI_DOWNLOAD_URL`,
    `SWISS_EPHEMERIS_PATH` and `DATABASE_URL`.
  - **Urania:** `npm ci`.
  - **Sankalpa:** `npm ci` without the Electron binary.

  Each install is skipped while its lockfile is unchanged. A repeat start costs
  about 10 ms.

### Why the Swagger zip exists

`utoipa-swagger-ui`'s build script downloads
`github.com/swagger-api/swagger-ui/archive/...zip`, and the cloud proxy
refuses GitHub archive downloads (403). Without `SWAGGER_UI_DOWNLOAD_URL`
pointing at a local zip, **every Rust build of `noesis-api` fails**. The zip
is rebuilt from the npm package `swagger-ui-dist@5.17.14`, which has the same
`dist/` contents. Bump that version together with `utoipa-swagger-ui` in
`Cargo.lock`.

## 2. OmniRoute

A cloud session's own reasoning always runs on Anthropic's models. The host
manages that connection, so `ANTHROPIC_BASE_URL` cannot point it elsewhere.
OmniRoute has two other roles.

### a. Default LLM provider for the products

| Path | Configured by | Fallbacks when OmniRoute fails |
|---|---|---|
| `workers/llm-proxy`, which serves Urania's narrator and WitnessRun interpretation through `NARRATOR_LLM_URL` | KV `OMNIROUTE_API_KEY` and `OMNIROUTE_MODEL` | Command Code, Nebius, NVIDIA, Kimi |
| `noesis-witness` (Rust, `/witness/interpret`) | env `OMNIROUTE_API_KEY` and `OMNIROUTE_MODEL`, plus optional `OMNIROUTE_BASE_URL` | NVIDIA, then OpenRouter |
| `witness-pipeline` scripts (L0 reports) | env `OMNIROUTE_API_KEY` and `OMNIROUTE_MODEL` | Command Code, NVIDIA, OpenRouter, OpenAI |

Each path stays inert until **both** the key and the model are set. No model
id is guessed, because OmniRoute model and combo ids are deployment-specific.
Going live is an operator step:

```bash
cd workers/llm-proxy
npx wrangler kv key put --binding=LLM_SECRETS OMNIROUTE_API_KEY "<key>"
npx wrangler kv key put --binding=LLM_SECRETS OMNIROUTE_MODEL "<model>"
npx wrangler deploy
```

For the Rust witness, set `OMNIROUTE_API_KEY` and `OMNIROUTE_MODEL` on the
Railway `Selemene-engine` service (staging first). Urania needs no change.

### b. External model rail for cloud sessions

The ISA requires accepted patches to carry an OmniRoute run id (ISC-659), and
cross-vendor audits run on OmniRoute. From a cloud session, call it without an
auth header. The network secret attaches the key outside the VM, so the
session never sees it.

```bash
curl -sS https://omniroute.thoughtseed.space/v1/chat/completions \
  -H 'content-type: application/json' \
  -d "{\"model\":\"$OMNIROUTE_MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"Audit this diff: ...\"}]}"
```

Record the response's id in the PR when a task needs OmniRoute provenance.

## 3. The flow

```
cloud-ready issue -> cloud session (one per issue) -> draft PR (auto-watched)
                  -> you review and merge -> you deploy (staging, then production)
```

1. **Queue.** Open issues labelled `cloud-ready` in Selemene, in the order in
   section 5.
2. **Dispatch.** Start a new session in the `thoughtseed` environment with the
   issue's repo selected, and paste the [task prompt](./task-prompt.md) with
   the issue URL filled in. Choose one way to start it:
   - **Claude app or web:** open a new session and pick environment
     `thoughtseed`.
   - **Terminal, from a checkout of the repo:**

     ```bash
     claude --cloud "$(sed -n '/^```text$/,/^```$/p' docs/cloud/task-prompt.md | sed '1d;$d')"
     ```

     Then edit the placeholders in the first message.
   - **Many issues at once:** a Claude Project can start and track one thread
     per issue.
3. **Review.** Each session opens a draft PR and watches it. It fixes review
   comments, and reports back when the PR is green, or explains what is
   blocking.
4. **Merge and deploy** stay with you. Deploy and rollback issues (slots 29 and
   30), Railway, Cloudflare, Access, and key rotation are never dispatched.

Sessions share your plan's rate limits. Two or three parallel sessions on
independent engines is a sensible ceiling.

## 4. Verification per repo (the gate, since there is no CI)

**Selemene-engine:**

```bash
cargo test -p <crate> --lib                                    # crates you touched
cargo clippy -p <crate> --no-deps --all-targets -- -D warnings
bash scripts/cloud/test-db.sh                                  # once per session, for DB tests
cargo test -p noesis-api --test <name> -- --test-threads=1     # DB-backed integration tests
(cd ts-engines && bun test && bun run typecheck)
pnpm --filter <package> test
(cd workers/llm-proxy && npm test)
```

**urania-137:**

```bash
npm test
npx tsc --noEmit && npm run typecheck:functions
npm run build && npm run verify:bundle
```

**sankalpa:**

```bash
npx vitest run
npm run typecheck
npm run check:renderer-budget
```

### Known baseline failures on `main` (not caused by your change)

Compare against these before claiming a regression:

- **Selemene:**
  - `cargo fmt --check` reports 31 diffs in `noesis-api`. This is toolchain
    drift, so format only the files you touch.
  - Workspace clippy fails in `noesis-core` (`large_enum_variant` on
    `EngineResultData`). Use `--no-deps`.
  - `ts-engines` `bun run lint` reports 2 Biome errors.
  - `witness-pipeline` is fully green since #1512 and #1513: the brand
    config is a test fixture (`BRAND_CONFIG_PATH` points real runs at the
    brand docs) and Playwright is pinned to 1.62.1.
- **sankalpa:** `check:renderer-budget` fails (262,527 B against a 256,000 B
  budget).
- **urania-137:** the bundle sits within about 135 B of its 575,000 B budget.

### What cannot run in a cloud session

- MediaPipe and the biofield CV sidecars (Python 3.11 wheels and Docker
  images).
- Live NVIDIA, Command Code or R2 calls (no keys in the environment).
- Cloudflare Access flows and production or staging probes.

Engines that depend on these (face-reading, biofield-capture, sigil-forge and
raaga beyond unit level) stay out of the autonomous queue.

## 5. Queue

<!-- queue:start -->
Labelled `cloud-ready` on 2026-10-28. Every W3E issue also needs its six-axis
ledger row in #896 before it can close.

Within a wave, issues on different engines can run in parallel. Issues on the
same engine run in the order listed.

**Wave A: unblock the witness pipeline.** Done 2026-10-28: #1510, #1511,
#1512 (with #1513 closing #890).

| Issue | Ask |
|---|---|
| [#887](https://github.com/Sheshiyer/Selemene-engine/issues/887) | Engine result ingestion: `integrated.test.ts` asserts the prompts contain engine data. |
| [#888](https://github.com/Sheshiyer/Selemene-engine/issues/888) | L0 verification gates: placeholder gate and chart-fidelity gate fail closed. |
| [#889](https://github.com/Sheshiyer/Selemene-engine/issues/889) | Brand HTML/PDF renderer. First make the hard-coded `/Volumes/madara/...` brand-config path configurable and align Playwright with the preinstalled browsers, then the PDF test can pass. |

**Wave B: provenance holes left after #1508.**

| Issue | Ask |
|---|---|
| [#1341](https://github.com/Sheshiyer/Selemene-engine/issues/1341) i-ching:07 | Emit a provenance block. None is emitted today. |
| [#1311](https://github.com/Sheshiyer/Selemene-engine/issues/1311) enneagram:07 | Emit a provenance block that separates assessment-derived from direct type. |
| [#1401](https://github.com/Sheshiyer/Selemene-engine/issues/1401) sacred-geometry:07 | Emit a provenance block that separates symbolic, calculated and rendered output. |
| [#921](https://github.com/Sheshiyer/Selemene-engine/issues/921) biofield:07 | The Vedic path hard-codes `fallback_used:false` and never checks Moshier (`engine-biofield/src/engine.rs:173`). |
| [#1191](https://github.com/Sheshiyer/Selemene-engine/issues/1191) panchanga:07 | Ephemeris vs mean-longitude fallback. First check how much #1508 already covers. |

**Wave C: domain-truth bugs.**

| Issue | Ask |
|---|---|
| [#1344](https://github.com/Sheshiyer/Selemene-engine/issues/1344) i-ching:10 | Changing lines must drive the relating hexagram. Run after #1341. |
| [#1074](https://github.com/Sheshiyer/Selemene-engine/issues/1074) gene-keys:10 | Calculated line data is replaced with fixed values. |
| [#1052](https://github.com/Sheshiyer/Selemene-engine/issues/1052) financial-biosensor:18 | An omitted source must fail or mark degraded, with no silent renormalisation. |

**Wave D: contracts.**

| Issue | Ask |
|---|---|
| [#1460](https://github.com/Sheshiyer/Selemene-engine/issues/1460) tarot:06 | Bind replay to deck version, spread, seed, order and orientation. |
| [#1459](https://github.com/Sheshiyer/Selemene-engine/issues/1459) tarot:05 | Fail-closed error taxonomy. Run after #1460. |
| [#1159](https://github.com/Sheshiyer/Selemene-engine/issues/1159) numerology:05 | Fail-closed error taxonomy. |
| [#1411](https://github.com/Sheshiyer/Selemene-engine/issues/1411) sacred-geometry:17 | Keep deterministic output separate from witness narrative. Run after #1401. |

**Wave E: golden fixtures.** These become the template for slot 27 on the
other engines.

| Issue | Ask |
|---|---|
| [#1181](https://github.com/Sheshiyer/Selemene-engine/issues/1181) numerology:27 | Golden fixtures. Run after #1159. |
| [#1001](https://github.com/Sheshiyer/Selemene-engine/issues/1001) biorhythm:27 | Golden fixtures. |

**Deliberately not queued:**

- **The 19 slot-21 billing issues:** Dodo is deferred.
- **Slots 29 and 30:** release and deploy.
- **Media engines beyond unit level:** face-reading, biofield-capture, sigil-forge and raaga.
- **#896, #901, #908, #913 and #914:** human decisions or production access.
- **The PR #1488 decision.**
<!-- queue:end -->
