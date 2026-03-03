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
- engine catalog (16 pages)
- workflow guide (6 pages)
