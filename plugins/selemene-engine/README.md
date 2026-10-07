# Selemene Engine — private plugin

Version 0.2.0 ports guided onboarding and the staged Selemene reading flow into the user's current ChatGPT conversation. The hosted service supplies reviewed engine calculations, local symbolic draws, existing framework meanings, and rule-based workflow synthesis. ChatGPT composes explanations, Aletheios/Pichet perspectives, integrated readings, and final prose using the user's active model and applicable ChatGPT limits. The plugin does not call an operator-funded narrative model.

Customers need no personal Selemene account, login, or API key. `mcp.json` connects to the dedicated hosted free service without client credentials. The sponsor API credential is held only on the server and must never be included in this package. Free access is bounded by the service's quota and availability.

Calculations send supplied personal inputs to the hosted Selemene service, which can retain inputs and results under a shared service/admin account and update that account's profile, history, usage or XP. The skills disclose this before execution. This creates no personal linked account or private account history.

## Included workflow

- `selemene-onboarding` introduces the service, discovers actual available engines/workflows, and helps choose a focus, language, depth, and necessary inputs.
- `selemene-reading` preflights the selected request, runs explicitly requested calculations, and composes a factual, source-qualified symbolic reading in ChatGPT.
- `skills/selemene-reading/references/host-reading-flow.md` supplies the staged method: intake, factual source map, separate system explanations, Aletheios, Pichet, and integrated synthesis.

When exposed by the connected service, `selemene_vedic_chart` supplies a separate natal D1/D9 chart source for integrated Kundali prose. It is not a `vedic-kundali` engine or the advanced local supplement. Only returned chart fields can support the reading; unsupported yogas, dignities, strength, Atmakaraka, or combustion conclusions are not invented.

Brief readings and long detailed reports are supported. Requested files can be created through available host-native document/file tools and must be verified. The original hosted witness generator, judge/corpus verification, certified export pipeline, media analysis, account administration, and paid generation are not invoked. Missing facts, unavailable engines, and partial workflows remain explicit.

## Private export

From the repository root, validate the portable package:

```sh
python3 -B plugins/selemene-engine/scripts/create-private-package.py --check
```

Create a new archive outside this source directory:

```sh
python3 -B plugins/selemene-engine/scripts/create-private-package.py --output /absolute/output/selemene-engine-0.2.0.zip
```

The private builder checks format, skill frontmatter, contained icon assets, the fixed credential-free MCP connection, secret patterns, archive inventory, symlinks, and output safety. It refuses to overwrite a release. It bundles only the portable manifest, MCP configuration, skills/references, and assets under one `selemene-engine/` directory; development scripts, dependencies, environment files, and credentials are excluded.

Private export requires no public listing attestations or legal metadata. A successful local export establishes package validity; deployed service execution, installation, and connected-host behavior require separate verification. Saving through Plugin Creator creates a private plugin and does not publish a public listing.

## Public submission remains separate

`scripts/create-package.py` retains the existing public submission evidence gate. Public packaging still requires the final listing, publisher/category/policy information, country and commerce choices, recording, and matching verification receipts. It fails closed until those requirements are supplied. `--submission-check` additionally requires executed cases for the saved submission version. The private builder neither changes nor satisfies that gate.

## Brand asset

`assets/icon.png` preserves the existing transparent 512px bronze Tryambakam sigil rasterized from the repository's original SVG. Its bytes are unchanged by this release.
