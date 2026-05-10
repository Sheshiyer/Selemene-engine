# Sample Attribution — Nādashakti V2

All samples shipped in [`manifest.json`](./manifest.json) are redistribution-safe (CC0 / CC-BY / public domain). Anything proprietary is rejected at curation time per plan task **T-012**.

## Sample Sources

| Pack | Source | License | Attribution | Verification URL |
|---|---|---|---|---|
| **sitar** | Versilian Community Sample Library (VCSL) | CC0-1.0 | Sam Gossner & contributors | https://github.com/sgossner/VCSL |
| **tanpura** | VCSL / Freesound community | CC0-1.0 | Community submissions | https://freesound.org/browse/tags/tanpura/ |
| **mridangam** | VCSL Drums | CC0-1.0 | Sam Gossner & contributors | https://github.com/sgossner/VCSL |
| **bansuri** | VCSL Winds | CC0-1.0 | Sam Gossner & contributors | https://github.com/sgossner/VCSL |
| **sarangi** | VCSL Bowed | CC0-1.0 | Sam Gossner & contributors | https://github.com/sgossner/VCSL |

## Why CC0 / public-domain only

- **Redistribution safety:** Strudel's `samples()` loads URLs at runtime. If a sample later turns out to have ambiguous licensing, every browser session that loaded it has effectively redistributed it. CC0 eliminates that risk entirely.
- **AGPL compatibility:** the wider Strudel codebase is AGPL-3.0; CC0 / CC-BY samples mix without conflict.
- **No attribution-on-demand:** users do not need to display credits while playing — but we *do* keep the credits here in the repo for ethical clarity.

## Phase-1 status

The manifest above is **declarative only** at this stage. Tasks **T-007..T-010** in [`V2_AUDIO_RICHNESS_PLAN.md`](../../../../../../../raagaegnin/V2_AUDIO_RICHNESS_PLAN.md) confirm the URLs, host the manifest on a stable CDN (T-011), and run the loader smoke test in Phase 2 (T-026). The chosen base URL is a public GitHub raw mirror of dough-samples / VCSL — Strudel can resolve it directly via `samples('https://...manifest.json')`.

## Process for adding a new pack

1. Verify the source license (must be CC0 / CC-BY / CC-BY-SA / AGPL or MIT).
2. Add an entry to `manifest.json` matching `manifest.schema.json`.
3. Add a row to the table above.
4. Run `node src/lib/raaga/verify-v2.mjs` to assert the manifest validates.
5. Run the Phase-2 loader smoke test (`pnpm dev`, click ▶ on a raga with the new pack selected) before merging.
