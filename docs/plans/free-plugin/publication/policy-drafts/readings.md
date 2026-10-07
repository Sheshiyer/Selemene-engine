**DRAFT FOR REVIEW — NOT EFFECTIVE OR PUBLISHED.**

# Discover, prepare and request a Selemene reading

- Proposed route: https://www.tryambakam.space/selemene/readings/
- Website destination: `public/selemene/readings/index.html`
- Prepared: 2026-10-07 for the free 0.2.0 connector.

## Begin with your focus

Tell ChatGPT what you want to explore, the subject of the reading, your preferred language and whether you want brief or detailed prose. Relationship context and other optional writing context can stay in the conversation. Only fields supported by the selected tool are sent to Selemene.

Discover the current catalog, inspect your selected engine or workflow and prepare its inputs. Required fields vary: natal systems may need birth date, exact time, coordinates and timezone; local Tarot/I Ching draws and an Enneagram reading do not all follow that pattern. Missing details must be identified rather than invented. Preparation itself makes no calculation request.

## Understand shared-service storage before execution

**Free readings send submitted inputs to the hosted Selemene service. The service can store submitted inputs and reading results under a shared service/admin account and may update its profile, history, usage, XP or phase. This does not create a personal linked account or provide private account history.**

Ordinary engine calculations and combined workflows attempt asynchronous storage. The returned calculation is not a storage receipt. No ephemeral processing, fixed retention period, deletion action or personal history access is promised by the connector.

Once this is understood, explicitly request the selected reading with the prepared inputs. A preparation-only request does not authorize execution. Multiple subjects' data and results must remain separately identified. Errors, unavailable components and partial workflows remain visible.

## Eight supported operations

| Operation | Tool |
| --- | --- |
| Discover the free engine catalog | `selemene_list_engines` |
| Inspect one engine and its required inputs | `selemene_engine_info` |
| Prepare supplied inputs and identify missing fields | `selemene_prepare_reading` |
| Request one engine calculation | `selemene_calculate` |
| Discover reviewed combined workflows | `selemene_list_workflows` |
| Inspect one reviewed workflow | `selemene_workflow_info` |
| Request a reviewed combined workflow | `selemene_run_workflow` |
| Request a separate natal Vedic D1/D9 chart | `selemene_vedic_chart` |

The reviewed non-media engine IDs are numerology, human-design, gene-keys, vimshottari, panchanga, vedic-clock, biorhythm, transits, enneagram, tarot and i-ching. The reviewed workflow is decision-support. These are the connector's allowlist; a catalog entry or source implementation does not promise that every calculation is operational. Use the current discovery response and actual execution outcome.

The separate Vedic chart supplies returned D1/D9 fields for an integrated natal reading. It is not a `vedic-kundali` engine or the richer local report supplement. The chart handler does not write a personal reading/history through that route, but this is not an end-to-end no-storage guarantee. Its output does not establish combustion, dignities, yogas, Atmakaraka or Darakaraka.

## Compose the reading in ChatGPT

Your current ChatGPT model explains successful returned results, applies the Aletheios structural and Pichet experiential perspectives, and writes the synthesis in the requested language. These are host writing stages, not separate narration-model calls. Distinguish returned values, supplied static meanings, your observations and ChatGPT's interpretation. Missing sources or values remain missing.

Requested document exports can use available host-native file tools and must be verified. The connector does not invoke Selemene's hosted witness renderer, independent judges, certification pipeline, assets generation, media analysis or paid generation.

## Availability and quotas

Default limits are 120 requests and 30 calculation attempts per connecting IP per minute, 600 requests and 60 calculation attempts for the whole service per minute, and 1,000 calculation attempts for the service per UTC day. Connecting IP is an abuse-control signal, not a personal allowance: shared networks and ChatGPT gateways may share it. Limits can change through service configuration.

Calculation attempts are charged after a recognized tool and valid outer input; a later preflight or backend failure is not refunded. Follow any retry guidance actually returned. Do not invent a reset time or silently repeat a timed-out calculation, because another request can create duplicate account effects or records.

[Home](https://www.tryambakam.space/selemene/) · [Privacy](https://www.tryambakam.space/selemene/privacy/) · [Terms](https://www.tryambakam.space/selemene/terms/) · [Support](https://www.tryambakam.space/selemene/support/)

**PENDING before public publication:** publisher/controller identity, supported countries/audience, private support/privacy contact, retention/deletion handling, final rules and effective policies. The current draft is descriptive product copy, not a completed policy or a claim of public directory availability.
