# Image Placement Strategy (2026-02-16)

## Objective

Reuse imported assets beyond `README.md` while preserving readability and technical scanability.

## Placement principles

- Prioritize Tier 1 and Tier 2 docs.
- Limit to one intro visual per target section.
- Prefer engine icon sets for API/reference docs and posters for deep engine-specific pages.
- Avoid adding visuals to Tier 3 operational runbooks unless there is direct operational value.

## 15 currently-unused assets mapped to targets

| Asset | Proposed target | Section rationale |
|---|---|---|
| `docs/assets/images/engines/5D-1-vedic-engine-icons-recraft-v2.png` | `docs/API_QUICKSTART.md` | Supports `Engine Reference` scanability. |
| `docs/assets/images/engines/5D-2-western-engine-icons-recraft-v2.png` | `docs/API_QUICKSTART.md` | Supports `Try Each Engine` mixed-mode discovery. |
| `docs/assets/images/engines/5D-3-bridge-engine-icons-recraft-v2.png` | `docs/api/engines.md` | Reinforces bridged TS engine grouping. |
| `docs/assets/images/engines/5D-4-biofield-engine-icons-recraft-v2.png` | `docs/api/engines.md` | Complements native-engine grouping. |
| `docs/assets/images/engines/8A-seeker-inner-architecture-nanobananapro-v3.png` | `docs/api/workflows.md` | Visual anchor for synthesis framing. |
| `docs/assets/images/noesis-hero.png` | `docs/PROJECT_OVERVIEW.md` | High-level narrative entry point. |
| `docs/assets/images/noesis-ecosystem.png` | `docs/PROJECT_OVERVIEW.md` | Ecosystem and documentation landscape context. |
| `docs/assets/images/engines/9A-01-vimshottari-dasha-poster-v1.png` | `docs/ENGINES.md` | Fits Vimshottari section identity. |
| `docs/assets/images/engines/9A-04-human-design-poster-v1.png` | `docs/ENGINES.md` | Fits Human Design section identity. |
| `docs/assets/images/engines/9A-05-gene-keys-poster-v1.png` | `docs/ENGINES.md` | Fits Gene Keys section identity. |
| `docs/assets/images/engines/9A-08-numerology-poster-v1.png` | `docs/ENGINES.md` | Fits Numerology section identity. |
| `docs/assets/images/engines/9A-09-tarot-poster-v1.png` | `docs/ENGINES.md` | Fits Tarot section identity. |
| `docs/assets/images/engines/9A-10-tcm-organ-clock-poster-v1.png` | `docs/ENGINES.md` | Fits Vedic Clock section identity. |
| `docs/assets/images/engines/9A-11-biorhythm-engine-poster-v1.png` | `docs/ENGINES.md` | Fits Biorhythm section identity. |
| `docs/assets/images/engines/9A-13-biofield-and-raga-poster-v1.png` | `docs/ENGINES.md` | Fits Biofield/Nadabrahman thematic sections. |

## Pilot rollout (implemented)

Five low-risk insertions applied (one file each, non-README):

1. `docs/API_QUICKSTART.md` → `Engine Reference`
2. `docs/api/engines.md` → `TypeScript Engines (Bridged)`
3. `docs/api/workflows.md` → `Available Workflows`
4. `docs/PROJECT_OVERVIEW.md` → `1. What Is Working`
5. `docs/ENGINES.md` → `4. Vimshottari Dasha Engine`
