# Selemene — The Fivefold Atlas

A standalone Three.js 0.160 atlas of Selemene's engines, infrastructure and connected systems. A Vitruvian figure, circle, square and five colored sheaths organize a source-backed constellation. The 24-second cinematic shot assembles, illuminates, unfolds and returns the system to the body.

## Run

```sh
cd docs/architecture/selemene-atlas
npm ci
npm run dev
```

Open <http://127.0.0.1:5187/>. Use `npm run build` to create a static `dist/` bundle. There are no application API calls, external fonts or CDN assets at runtime. The model and Three.js ship locally. Repository links open only on request.

## Explore

- Choose a Kosha to focus its engines and read the proposed rationale in the inspector.
- Switch between Connections, Anatomy, Exploded and Biofield views.
- Play the cinematic shot, choose a chapter, scrub its 24-second timeline or adjust playback speed.
- Orbit by dragging; zoom with the wheel; use Recenter to restore framing.
- Select a component in the scene, the inspector menu or a relationship button.
- Open **Explore engines**, or **Inside [engine]** in the inspector, to enter an engine's own instrument. Drag its expansion slider, click a form to isolate it, follow **Next component**, and use **Show assembly** to return. **Return to body** preserves the underlying shot and view.
- Preview a PNG of the 3D stage, preview or copy a Markdown briefing, request a download, or copy a link retaining the current view, component, Kosha and shot position. A loopback link opens on the same computer.
- Reduced-motion preference disables automatic playback; manual views and timeline remain available.

The Codex in-app browser cancelled download requests during verification. Export previews and briefing copy work inside the app; file downloads require a browser that permits them.

## What each mark means

| Mark | Meaning |
|---|---|
| Luminous node | One named engine, integration component, infrastructure component or connected consumer |
| Curved path | One source-defined or documented relationship; evidence is accessible through the inspector |
| Moving particle | Schematic direction along a relationship, with no traffic, physiological or health measurement |
| Colored sheath | A proposed Pancha Kosha grouping of engine capabilities |
| Human anatomy | Artistic correspondence used to organize the atlas |
| Engine instrument part | One named software responsibility, with source, symbol and implementation status |
| Instrument artwork | A visual study from the selected brand collection; it does not establish runtime capability |

## Proposed Pancha Kosha mapping

| Kosha | Lens | Engines |
|---|---|---|
| Annamaya | Physical body | Face Reading; Biofield Capture |
| Pranamaya | Vital rhythm | Biofield; Biorhythm; Vedic Clock |
| Manomaya | Mind and symbol | Tarot; I Ching; Enneagram; Sigil Forge |
| Vijnanamaya | Discernment | Human Design; Gene Keys; Numerology; Panchanga; Vimshottari; Transits; Financial Biosensor |
| Anandamaya | Contemplation | Nada Brahman; Raaga; Sacred Geometry |

These assignments are editorial correspondences introduced for this visualization. They are not existing engine contracts or classical prescriptions. In particular, Anandamaya traditionally refers to the bliss sheath; no engine measures bliss. Infrastructure and connected consumers remain in the supporting field rather than being assigned to a Kosha. The conceptual reference is [Swami Krishnananda's account of the five sheaths](https://www.swami-krishnananda.org/panch/panch_03.html).

## Source and scope

The snapshot is tied to repository revision `9a05f5cc90777f083d7065e491484ef727aeada1` on 12 September 2026. The manifest contains 19 canonical engine identities, 42 components and 50 relationships. These are source counts, not evidence that a service is running. The 17 public mirrors are distinct from the 19 runtime identities. Six workflows are declared; declaration does not establish runtime support.

`src/data.js` carries a repository-relative source path, description and evidence class for each component and relationship. Source links use the pinned revision. The native Biofield engine is separate from the API's Python CV boundary, and Pattern Memory is explicitly a placeholder.

`npm run check:data` compares the engine list against `SUPPORTED_ENGINE_IDS`, confirms exact-once coverage across five Koshas, verifies all source paths, and checks key Biofield boundaries. `ISA.md` records browser and build acceptance.

The inner engine catalog contains 126 responsibilities, 134 relationships and 41 code/docs reconciliations across all 19 engines. `npm run check:engines` verifies source paths at the pinned revision, artwork hashes and component-view URL round trips. The engine-room modules load only when opened; each engine's artwork is loaded on demand. A shared engine URL additionally retains `engine`, `explode`, `part` and `isolate`.

The 16 existing `9A-posters-v2` artworks are reused unchanged; Biofield Capture, Financial Biosensor and Raaga have matching new illustrations. See [BRAND-RECONCILIATION.md](BRAND-RECONCILIATION.md) for the audit and [brandkit/generation-prompts.json](brandkit/generation-prompts.json) for the built-in imagegen briefs. The original sigil and local Panchang/Satoshi font files are reused. The procedural 3D instruments remain interpretive software assemblies.

## Model provenance

The anatomical asset was generated with Meshy 7 and adapted in Three.js for the Vitruvian composition. `public/assets/vitruvian.provenance.json` records the job, prompt and asset hashes. Two preview generations consumed 40 Meshy credits total. The first mannequin remains as a source asset; the application uses the revised asset. No Meshy credentials or signed download URLs are included in the application.

The generated body and the Pancha Kosha correspondences are conceptual visual material. This atlas neither senses nor diagnoses the body or a biofield.

## Visual verification

The desktop layout was checked at 1920 × 1080 CSS pixels with no document overflow. The narrow layout was checked at 390 × 844 with working category navigation and a readable inspector. Screenshots are retained in `evidence/`.

## Cloudflare publication

Public hostname: <https://atlas.tryambakam.space/>. Cloudflare fallback: <https://selemene-atlas.pages.dev/>.

The atlas is a static Cloudflare Pages project named `selemene-atlas`, in account `9d9d23b27f32e70ae3afb6a1aa2c0f10`, using the existing OAuth profile `9d9d`. There are no Pages Functions, server-side bindings, runtime secrets or backend requests. `atlas.tryambakam.space` is a proxied CNAME to `selemene-atlas.pages.dev`; the existing engine API hostname remains `selemene.tryambakam.space`.

To publish a later local revision, run from this directory with Wrangler 4.124.0 or a compatible later version:

```sh
npm run check:data
npm run check:engines
npm run build
env -u CF_API_TOKEN -u CLOUDFLARE_API_TOKEN \
  CLOUDFLARE_ACCOUNT_ID=9d9d23b27f32e70ae3afb6a1aa2c0f10 \
  wrangler pages deploy dist --project-name selemene-atlas \
  --branch main --profile 9d9d --commit-dirty=true
```

Pages does not accept `account_id` in `wrangler.jsonc`; the named profile and explicit account environment variable select the destination. Production uses the Pages branch label `main`. The initial artifact was uploaded directly from a dirty local checkout; this does not imply a Git commit, push, merge or Git-triggered deployment.

`public/_headers` sets `Cache-Control: public, max-age=0, must-revalidate, no-transform` on the document. This prevents the zone's automatic analytics injection from adding external scripts to the self-contained atlas. It is scoped to this Pages site; no zone-wide analytics settings are changed.

The custom domain must be associated with the Pages project in addition to its CNAME. Cloudflare's [Direct Upload](https://developers.cloudflare.com/pages/get-started/direct-upload/) and [custom domain documentation](https://developers.cloudflare.com/pages/configuration/custom-domains/) describe the lifecycle. The final public receipt and asset hashes are recorded under `evidence/` and the verification entries in `ISA.md`.

To roll back a future update, restore the previously accepted production deployment in the Pages dashboard. For the initial publication, unpublishing would require removing this atlas hostname's CNAME and Pages custom-domain association; no existing engine service needs to change. Do not run those removal operations unless requested.

The engine-instrument release is production deployment `f6ed2a82-0942-49f5-abda-e95b1da7b8d3`, published 12 September 2026. All 34 public content assets match the final local build over verified HTTPS. Live Biofield expansion, component isolation, source inspection and return were checked after deployment, with no warning or error entries in the browser log. See `evidence/publication-engine-rooms.json` and `evidence/publication-engine-room-assets.json`; local coverage of all 19 engines is in `evidence/engine-room-verification.json`.
