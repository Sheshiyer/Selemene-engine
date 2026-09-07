# Selemene Developer Portal

Docusaurus-based documentation site for Wave W2 developer portal deliverables.

## Local development

```bash
npm --prefix docs/portal install
npm --prefix docs/portal run start
```

## Build

```bash
npm --prefix docs/portal run build
```

## Vercel deployment

- **Root Directory:** `docs/portal`
- **Build Command:** `npm run build`
- **Output Directory:** `build`

This site includes:
- API overview
- authentication guide
- rate limits
- SDK quickstarts (Rust + TypeScript)
- OpenAPI explorer links
- engine catalogue (19 runtime identities; 17 public mirrors)
- workflow guide (6 pages)
## Contract v1 capability boundary

The portal documents 19 runtime identities and 17 public mirrors. API calls require exactly one `X-API-Key` or `Authorization: Bearer` header and preserve `declared`, `available`, `degraded`, and `unavailable`. Full Spectrum is explicitly `unsupported`.
