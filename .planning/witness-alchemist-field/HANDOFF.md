# Handoff: Witness Alchemist Field v2

Handoff from a claude.ai session on Fri 30 Oct 2026, 04:36 IST. Read this file, then `tasks/todo.md`, then start on the first unchecked P0 item.

## What this is

**Witness Alchemist Field** is a single-page Human Design field for Shesh Iyer (Witness Alchemist). It is published as a claude.ai artifact:

- Live page: https://claude.ai/artifact/UuPvCDpJbSyYg8K7nNm11v (private to Shesh)
- The page has eight rooms, routed by hash: `#sky` (home, three.js night sky with Western/Vedic toggle; 2D SVG wheel under reduced motion, low power or no WebGL), `#today`, `#design`, `#keys`, `#engines` (the eight Selemene engines), `#jyotish` (sidereal chart North/South, dasha drill-down, day Panchanga, Sade Sati, transit aspects; Lagna Scorpio 12°07' is a Swiss Ephemeris figure from compute.py), `#timeline`, `#map`, `#folio`.
- All transit data is precomputed daily (06:00 IST) from 30 Oct 2026 to 31 Dec 2030 and embedded in the page. The page opens on today's IST date by itself.

To republish the same URL, use a session that has the claude.ai Artifact tool: publish the rebuilt `witness-alchemist-field.html` with `url` set to the link above, `root` set to this folder, and `files` mapping each `img/...` path (17 engine stills plus `img/trinity-hex.jpg`) to itself. The page loads those images by relative path; leaving `files` out keeps the ones already published. Read the artifact first; a publish without a prior read is refused. If your Claude Code session has no Artifact tool, build and preview locally, then republish from claude.ai.

## Subject data (verified)

| Item | Value |
|---|---|
| Birth | 13 Aug 1991, 13:31 IST (08:01 UT), Bangalore 12.9716°N, 77.5946°E |
| Design moment | 13 May 1991, 13:59 IST (88° solar arc) |
| Type, Strategy, Authority | Generator, To Respond, Sacral |
| Profile | 2/4 |
| Definition | Split, specifically a **wide split** (needs two gates to bridge: 7+29, 8+29, or channel 20-34) |
| Cross | Right Angle Cross of Explanation 4/49 \| 23/43 |
| Channels | 23-43 Structuring, 42-53 Maturation |
| Defined centers | Ajna, Throat, Sacral, Root. Head, Heart, Spleen completely open; G holds 1, 46; Solar Plexus holds 49 |
| Variables | **PRL DRL**: Determination Thirst/Cold (Right); Environment Shores (Left); Motivation Hope (Right); View Probability (Left). Confirmed against Shesh's HumDes bodygraph |
| Node method | **True node.** Mean node would put Design Node in 61 and define the Head, which contradicts the HumDes chart |
| Vedic (Lahiri) | Lagna Scorpio 12°07' Anuradha p3; Moon Virgo 9°49' Uttara Phalguni p4; Jupiter Cancer 29°46' exalted, 9th house, Atmakaraka |
| Vimshottari | Jupiter MD from **14 Sep 2026** (Selemene; source of truth) to Sep 2042. Jupiter/Jupiter AD to 1 Nov 2028. Jupiter/Jupiter/Jupiter PD to 27 Dec 2026 IST |
| Birth Panchanga (Selemene) | Chaturthi (Shukla), Uttara Phalguni, Siddha yoga, Vishti karana, Mangalavara |
| Numerology (Selemene) | Life Path 5, Expression 1, Soul Urge 11, Personality 8, Birthday 4, Chaldean name 11 |

Full activations are in `chart.json`. Selemene's Human Design engine matches every gate and line of this computation.

### Gene Keys sphere mapping (matches HumDes; an earlier draft had this wrong)

| Sphere | Key | Source |
|---|---|---|
| Life's Work | 4.2 | P Sun |
| Evolution | 49.2 | P Earth |
| Radiance | 23.4 | D Sun |
| Purpose | 43.4 | D Earth |
| Attraction | 24.4 | D Moon |
| IQ | 59.5 | P Venus |
| EQ | 47.1 | P Mars |
| SQ | 52.1 | D Venus |
| Vocation | 62.2 | D Mars |
| Culture | 31.5 | D Jupiter |
| Pearl | 4.5 | P Jupiter |

The Selemene Gene Keys engine returns only the four Activation spheres, so keep the Venus and Pearl mapping above.

## Files in this bundle

| File | Role |
|---|---|
| `compute.py` | Natal, design and one transit snapshot via pyswisseph (Moshier) → `chart.json` |
| `daily.py` | Daily transit records + event list (ingresses, stations, exact contacts, eclipses) → `sky.json` |
| `extra.py` | Adds astrocartography MC/IC/ASC/DSC lines, daily transit MC, daily sidereal positions, Vimshottari periods, land outline → `sky.json`. **Run after `daily.py`**; re-running `daily.py` alone drops these fields |
| `land.js` | world-atlas land-110m → equirectangular SVG path `land.txt` |
| `content.js` | Folio reading text as blocks (`h1`, `h2`, `p`, `note`, `table`) |
| `dict.js` | Gate names, Gene Keys spectra, channels, personal channel notes, center texts, spheres, cities |
| `app.html` | Page template with `%%SKY%%`, `%%CONTENT%%`, `%%DICT%%` placeholders |
| `room-sky.js`, `room-engines.js`, `room-jyotish.js` | The Sky (home) and Engines rooms; `build.py` inlines them at `%%ROOMS%%` |
| `build.py` | Assembles `witness-alchemist-field.html`; bundles `selemene/*.json` into `%%ENGINES%%` and takes the Mahadasha start from the Vimshottari response |
| `t.js` | jsdom smoke test across all rooms (needs `npm i jsdom@24.1.0`) |
| `selemene/calc-*.json` | Selemene engine responses for the natal chart, 30 Oct 2026 (no key inside) |
| `design-system/README.md`, `tokens.json` | Tryambakam Noesis design system, copied from the artifact Shesh selected |

Build:

```bash
pip install pyswisseph --break-system-packages
npm i world-atlas@2.0.2 topojson-client@3.1.0 jsdom@24.1.0
python3 compute.py && python3 daily.py && python3 extra.py
node land.js   # only if land.txt is missing; extra.py reads land.txt
node -e "require('fs').writeFileSync('content.json',JSON.stringify(require('./content.js')))"
python3 build.py && node t.js
```

## Selemene engine

- Repo: `/Volumes/madara/2026/Projects/tryambakam-noesis/Selemene-engine`
- Base URL: `https://selemene.tryambakam.space/api/v1`, header `X-API-Key`
- Key: `SELEMENE_API_KEY` in `/Volumes/madara/sec-key/.env`. **Never print it, commit it, or put it in the page.**
- Request body: `{"birth_data":{"name","date":"1991-08-13","time":"13:31","latitude":12.9716,"longitude":77.5946,"timezone":"Asia/Kolkata"},"consciousness_level":3}` to `POST /engines/{id}/calculate`
- Engines this key can run: panchanga, vimshottari, transits, human-design, gene-keys, numerology, biorhythm, vedic-clock
- Refused (403, free tier): biofield, nadabrahman, raaga, i-ching, sacred-geometry. Untested: tarot, enneagram, workflows, `/witness/interpret`
- Credits: 50 on the free tier; engine calls did not use any
- Transits engine is sidereal and returns aspects to natal planets, `period_quality`, retrogrades and `sade_sati` (inactive: Moon Virgo, Saturn Pisces)
- Vedic Clock is time-of-call specific: current hora, choghadiya, dosha, organ, and `upcoming_transitions`

## Constraints that shape the build

- **Artifact CSP.** Scripts load only from cdnjs, jsDelivr, unpkg; no `fetch` to other hosts. The page cannot call Selemene. Live data must arrive through a scheduled task that writes to the artifact's database (`db` capability) or republishes the page.
- **Design system.** Tryambakam Noesis. Instrument voice on void for operating rooms; Folio voice on parchment for the reading only; never mix voices on one screen. Gold is line and type, never a large fill. One easing curve `cubic-bezier(0.19,1,0.22,1)`. Lexicon: field, room, witness, instrument, engine; banned: journey, path, healing, manifesting, "AI", dashboard, archive, library. Panchang and Satoshi fall back to Unbounded and Manrope (Google Fonts).
- **Copy rules.** No prediction, no diagnosis. Witness, do not forecast. Direct, affirmative sentences, no filler.
- **No edition history in the artifact.** The earlier reading is context only. Do not show "what changed", "previous reading", "corrected" or "upgraded edition" language anywhere.

## Daily refresh (decided 30 Oct 2026: the engine owns the day)

The Selemene engine computes and stores the day; the field only reads it.

- **Engine side** (branch `feat/daily-field-engine`, worktree `../Selemene-engine-daily`): a daily-field module beside the API's existing daily background task. At 05:30 IST, for users whose profile preferences carry `daily_field: true`, it runs panchanga (daily mode), transits, vimshottari, biorhythm and vedic-clock ten minutes after that date's sunrise, plus the Witness Dyad with no biofield capture, and stores each as a `readings` row (`workflow_id = "daily-field"`). `GET /api/v1/daily?date=` reads a day; `POST /api/v1/daily/refresh` computes what is missing. The same branch makes `live_scores` optional and adds `current_time` on `/witness/interpret`, and fixes Panchanga daily mode, which computed IST mornings 5h30m early. **Needs merge and deploy, then opt in with `PATCH /api/v1/users/me {"preferences":{"daily_field":true}}`.**
- **Field side:** `sync_daily.py` pulls the stored day into `selemene/daily/<date>.json` (GET `/daily`, then `/daily/refresh`; until those are deployed it makes the same engine calls, which the engine also stores in `readings`). `build.py` embeds the last 31 days. Today shows an engine-day chip, engine-sourced lens rows, the Witness Dyad panel (checked by the ported no-prediction guardrail) and day checks in the method check drawer.
- **Last mile:** the scheduled task `witness-field-daily-sync` runs at about 06:10 IST on this Mac: sync, build, `node t.js`, republish. The key stays in `/Volumes/madara/sec-key/.env`; the Mac must be on (a missed run fires on next launch).
- Cloudflare blocks Python's default user agent (error 1010); `sync_daily.py` sends its own.

## Known data points worth reusing

- 30 Oct 2026: Mars in 7 plus South Node in 29 bridge the wide split; single definition with motor to Throat until 1 Nov.
- Jupiter return on P Jupiter 4.5: 25 Oct 2026, 31 Jan 2027, 21 Jun 2027 (tropical). Sidereal returns 29 Oct 2026, 26 Jan 2027, 24 Jun 2027.
- Nodes on the cross (NN 49 / SN 4): 24 Nov 2026 to 7 Apr 2027, exact on P Earth 13 Mar 2027.
- Pluto over D Saturn 41.6: 30 Mar 2027, 17 Jun 2027, 29 Jan 2028, 5 Sep 2028, 1 Dec 2028.
- Nodal return (NN 54): 20 Sep 2028 onward, exact on natal nodes Sep to Oct 2028.

## 3D instrument layer (30 Oct 2026, second pass)

The field now lives in the render-bible world: void ground, one warm key, a gold rim, dark bronze with gold as line, one glass per object carrying evidence state.

| File | Role |
|---|---|
| `kit3d.js` | Shared stage: renderer, PMREM studio env, bronze/gold/glass materials, hex and band geometry, constant-pixel labels, starfield, orbit/tilt/pinch/keys, picking, tweens on the brand curve, render-on-demand, one live stage per room (`K3.mount`/`K3.release`) |
| `icons.js` | Line-art glyphs drawn in code: nine rooms, seventeen engines, witness mark, medallion frame |
| `field.css` | Shell (left rail, phone dock, top bar with breadcrumb), numbered hairline instrument frames, stage and HUD styles |
| `room-sky.js` | Home: bronze orrery (emerald beads on arms, turning zodiac band, birth studs, 64-gate ring, aspect arcs), day scrubber with play, telemetry flanks, room medallions |
| `room-mandala.js` | Engines: 19-cell Engine Mandala plate with Instruments Codex Dark stills, lift-on-hover, idle tour, keyboard walk, InstrumentMotion readout |
| `loom3d.js` | Today and Design: nine-socket bodygraph loom; channels draw along the wire on date change |
| `arm3d.js` | Jyotish: dasha armillary, three rings precessing under one index, synced with the drill-down columns |
| `img/` | 17 engine stills and the trinity-hex mark, downscaled from the design system's Instruments Codex Dark set |

Every 3D scene has a 2D path (no WebGL or low-power device) and a static path under reduced motion. `node t.js` exercises the 2D paths; the WebGL paths were checked in a browser at 1440x900 and 375x812.

## Today as a multi-engine day (30 Oct 2026, third pass)

| File | Role |
|---|---|
| `daylens.js` | Engine methods recomputed for any date: NOAA sunrise/sunset, hora, choghadiya, organ and dosha clocks, Rahu Kaal, Abhijit, biorhythm, personal year/month/day, Tarabala, Chandrabala. `DL.verify(ENG)` checks them against the stored Vedic Clock and Biorhythm responses (16 of 16 match) |
| `dial3d.js` | The Day Dial: the Vedic Clock engine's object as a 24-hour bronze face with hora, choghadiya, organ and dosha rings, an hour hand and a 2D fallback |
| `room-today.js` | Today: loom (with Planets and Gene Keys overlays) beside the dial, an eight-engine lens matrix (birth, this day, where they meet, provenance per cell) and a 31-day ribbon (definition, Moon nakshatra, tithi phase, Tarabala, biorhythm, events) |

Provenance words on the page: **Engine** is read from a stored response; **Field · engine method** is recomputed by a method checked against the engine; **Field** is a classical or Swiss Ephemeris computation the engines do not return.
