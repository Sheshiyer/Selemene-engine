# Selemene Engine — public submission source

Draft version 0.1.0. This directory is not a submission-ready bundle. Its seven-tool skill and listing depend on the actual deployed, verified MCP server. No endpoint is fabricated and no `mcp.json` is present until deployment verification supplies one.

`plugin.json` contains supported listing copy, 5 positive cases, 3 negative cases, release notes and existing brand assets. Unknown publisher, category, URL, commerce, country and recording fields are omitted intentionally. Review cases are **not run**; see `../../docs/plans/chatgpt-plugin/REVIEW-CASES.md`.

## Package checks

Run `python3 plugins/selemene-engine/scripts/create-package.py --check` from the repository root. This is an offline fail-closed local readiness checker, not the target portal validator. It reports all missing evidence and exits nonzero until preparation is complete. It does not fetch remote URLs, authenticate, upload, submit or attest.

Once a real endpoint is verified, write portable `mcp.json` with `$schema` `https://agent-plugins.org/schemas/1.0.0/mcp.schema.json` and one `mcpServers.selemene` entry containing `type: streamable-http` and the actual HTTPS `url`. Do not add bearer headers, credentials, `.app.json` or app bindings to the upload.

Provide an external JSON receipt via `--evidence /absolute/path/submission-evidence.json`. It must contain `manifest_sha256` matching the final manifest plus `checks` entries with nonempty `value`, `verified_at` and `evidence` strings. Required keys: `endpoint` (value = MCP URL), all four listing URL field names (value = their manifest URL), `demo_recording_url` (value = verified video URL), `developerName` (value = verified identity), `category` (value = supported dashboard category), `publication.countries` (value = JSON encoding of the explicitly chosen country array), `review.commerce` (value = JSON encoding of the actual declaration), `protocol_contract_tests` (value = `passed`), `tool_annotations` (value = `passed`), `icon_visual_inspection` (value = `passed`). The evidence text identifies the receipt or observation, not a claim inferred from a successful HTTP status. Never put credentials into that file or the package.

A candidate ZIP does not require cases to have already run against an uploaded submission version. After upload and final connection, run the same checker with `--submission-check`; that additionally requires a `connected_review_cases` receipt with value `passed` identifying the saved version and real runs. This prevents a circular dependency between creating the first ZIP and testing its saved portal version.

Then use `--output /absolute/path/selemene-engine-0.1.0.zip` with `--evidence` to create the public upload in a separate archive. Only the portable manifest, verified MCP configuration, skill and assets are included. The checker inspects archive inventory and metadata; readiness does not establish portal authentication, developer/domain verification, secure reviewer access or legal attestations. Those remain separate final setup steps.

## Existing icon provenance

`assets/icon.png` is a transparent 512px square rasterization of the repository's existing `docs/architecture/selemene-atlas/public/assets/tryambakam-sigil.svg` from the original checkout. Its geometry and bronze color are preserved, with a centered square canvas. No new logo is generated. Verify legibility on light/dark backgrounds and at small sizes before recording icon evidence.
