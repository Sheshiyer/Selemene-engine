# Phase 2 research

Observed 2026-09-05 against source9a05f5c plus isolated recovery commits.

## Findings and implementation consequences

- GitHub #1486 CI failed only TS Biome formatting; scoped commit8e5a8d3 repairs it. #1487 remains stacked and is not deployed.
- Baseline pnpm production audit:20high,11moderate,2low. Existing npm-style overrides did not govern pnpm. Next16.3.4 and matching ESLint config plus supported transitive patches produced an all-zero audit and successful admin production build. Verify override compatibility before acceptance.
- Admin has four pre-existing effect-state lint errors. Witness tests have a private absolute brand-config path and require an installed Playwright browser. Fix behavior/portability without skipping assertions.
- Rust audit has no known vulnerabilities but chacha20 0.10.1 is yanked; resolve dependency ancestry and disposition explicitly. Python lower bounds are not a reproducible deployment lock; local pip26.1.2 audit finding is not proof of the image's dependencies.
- CI already runs Rust, TS and a biofield contract smoke, but lacks admin and production Node audit coverage. Third-party Actions use floating refs. Main ruleset15597830 protects deletion and force push only, with no required-status/review rules and no bypass actors.
- Railway schema supports build.watchPatterns (repository-root-relative) and deploy.limitOverride, not deploy.watch or deploy.resources. TS/Python effective watch patterns are empty. Explicit config paths select the TOML files. Source repair is needed before a separately approved deployment.
- Scoped Cloudflare9d9d confirms tryambakam.space and three live workers. Add explicit account_id where absent. DNS read is denied; no DNS records were inspected. Pattern-memory's placeholder declarations are not resources.

## Sources

Local .github/workflows/test.yml, deploy.yaml, captured audit JSON, admin/witness logs and INFRASTRUCTURE-MAP.json supply current evidence. Primary provider references: [Railway configuration](https://docs.railway.com/config-as-code/reference), [watch paths](https://docs.railway.com/builds/build-configuration#configure-watch-paths), [Railway schema](https://railway.com/railway.schema.json), [Wrangler config](https://developers.cloudflare.com/workers/wrangler/configuration/).

## Validation strategy

Use proper process exit codes and saved logs. Prove action SHA resolution, YAML/actionlint, validator negative cases, frozen Node/Python installation, affected admin/witness/TS suites, full canonical gate and remote CI on the actual candidate. Independent review challenges semver overrides, watch patterns, mandatory job aggregation and every completion claim. GitHub issue bodies retain original exit criteria and append dated evidence without closing incomplete waves.
