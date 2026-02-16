# README Image Context Baseline (2026-02-16)

- Total assets in `docs/assets/images`: **119**
- Assets referenced in `README.md`: **13**
- Currently unused assets: **106**

## Currently Used in README

- `engines/logo-wide.mp4`
- `2C-glass-logo-nanobananapro-v2.png`
- `5A-heritage-engraving-recraft-v2.png`
- `noesis-architecture.png`
- `noesis-identifiers.png`
- `4A-ritual-blend-catalog-layout-nanobananapro-v2.png`
- `3A-ritual-kit-nanobananapro-v2.png`
- `3B-somatic-book-nanobananapro-v2.png`
- `3C-essential-oil-bottle-nanobananapro-v2.png`
- `4B-ritual-object-flat-lay-nanobananapro-v2.png`
- `2A-brand-kit-bento-nanobananapro-v1.png`
- `5B-campaign-visual-identity-grid-nanobananapro-v2.png`
- `2B-wax-seal-nanobananapro-v2.png`

## High-Value Reuse Candidates (Initial)

- `engines/2A-brand-kit-bento-nanobananapro-v2.png`
- `engines/2C-stained-glass-logo-nanobananapro-v2.png`
- `engines/5D-1-vedic-engine-icons-recraft-v2.png`
- `engines/5D-2-western-engine-icons-recraft-v2.png`
- `engines/5D-3-bridge-engine-icons-recraft-v2.png`
- `engines/5D-4-biofield-engine-icons-recraft-v2.png`
- `engines/9A-01-vimshottari-dasha-poster-v1.png`
- `engines/9A-02-nakshatra-engine-poster-v1.png`
- `engines/9A-03-chakra-kosha-mapping-poster-v1.png`
- `engines/9A-04-human-design-poster-v1.png`
- `engines/9A-05-gene-keys-poster-v1.png`
- `engines/9A-06-astrocartography-poster-v1.png`
- `engines/9A-07-enneagram-poster-v1.png`
- `engines/9A-08-numerology-poster-v1.png`
- `engines/9A-09-tarot-poster-v1.png`
- `engines/9A-10-tcm-organ-clock-poster-v1.png`
- `engines/9A-11-biorhythm-engine-poster-v1.png`
- `engines/9A-12-hrv-engine-poster-v1.png`
- `engines/9A-13-biofield-and-raga-poster-v1.png`
- `engines/8A-seeker-inner-architecture-nanobananapro-v3.png`

## Suggested Section Realignment Opportunities

- Add one visual bridge image in the new philosophy section (`8A-seeker-inner-architecture-*`).
- Add a compact icon strip near `The 16 Engines` intro using the `5D-*engine-icons*` set.
- Add one poster carousel/table in an expandable block for engine families (`9A-*poster*`).
- Keep architecture visuals where they are; avoid moving them above philosophy copy.
- Add a Somatic Canticles callout card near the physical embodiment block using `3B-somatic-book-*` and link to `https://1319.tryambakam.space`.

## Naming Context Strategy (Recommended)

Use a metadata sidecar index (JSON/CSV) keyed by filename with:
- `theme` (`v1-grounded-depth` | `v2-bioluminescent-architecture`)
- `section_target` (`hero`, `philosophy`, `engines`, `workflows`, `embodiment`, `brand`)
- `asset_type` (`logo`, `poster`, `icon-set`, `photo`, `diagram`, `video`)
- `status` (`used`, `candidate`, `archive`)
- `notes` (1-line intent)
