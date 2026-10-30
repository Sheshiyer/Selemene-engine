# Sprint: Field v2, Night Sky and Selemene

**Dates:** Fri 30 Oct to Thu 5 Nov 2026
**Goal:** The home room becomes a 3D night sky with a Western/Vedic toggle, and Selemene feeds every Vedic layer with fresh data each morning.
**Load:** 28 pts committed of 36 (78%), 8 pts stretch

## P0

- [x] **Remove edition-history narrative (2 pts).** Delete chapter "0. What Changed"; remove "previous reading", "corrected", "Upgraded Edition", "earlier readings" phrasing in `content.js` (Sections 8, 9, 15 and the Folio header in `app.html`). Renumber chapters. Grep the built page to confirm none remain.
- [x] **3D night-sky home (8 pts).** three.js r128 UMD from cdnjs. Starfield; ecliptic wheel with a Western (tropical signs) / Vedic (Lahiri sidereal signs + 27 nakshatras) toggle; natal ring and today's planets from `sky.json`; alignment lines for conjunction, opposition, square, trine within orb; tap a planet for gate, sign, nakshatra and contact details; optional 64-gate mandala ring. Cap frame rate; 2D wheel fallback under `prefers-reduced-motion` and on low-power devices. Noesis Mandala voice.
- [x] **Selemene natal layer (5 pts).** Shipped as the Engines room (all 8 working engines, locked ones listed), Mahadasha boundary read from the engine. Birth Panchanga, full Vimshottari to pratyantardasha, numerology. Label figures as computed by engine. Switch Mahadasha start to 14 Sep 2026.
- [x] **Jyotish room (5 pts).** Sidereal chart (offer North/South Indian layout toggle), dasha drill-down, today's Panchanga, Sade Sati status, sidereal transit aspects.

## P1

- [x] **Daily refresh at 05:30 IST (5 pts).** Done as engine-owned daily field (see HANDOFF); engine branch awaits deploy. Scheduled task calls Panchanga, Transits, Vedic Clock, Biorhythm, Vimshottari; writes to the artifact database; adds a short daily witness note. Blocked on the key-handling decision in HANDOFF.md.
- [ ] **Day panels on home (3 pts).** Hora and choghadiya schedule, biorhythm curve.

## P2 (stretch)

- [ ] **Locked engines (3 pts).** Biofield, Nadabrahman, Raaga, I Ching, Sacred Geometry once the key tier allows.
- [ ] **3D astrocartography globe (5 pts).**

## Definition of Done

- [ ] Every Vedic figure traces to a Selemene response; everything else to the Swiss Ephemeris
- [ ] Renders cleanly on desktop and phone, with a reduced-motion path; `node t.js` passes
- [ ] The key appears nowhere in the published page or the repo
- [ ] No edition-history language anywhere in the field
- [ ] Shesh signs off on the look and on Vedic accuracy

## Key dates

| Date | Event |
|---|---|
| Fri 30 Oct | Start: narrative cleanup, 3D sky |
| Mon 2 Nov | Mid-sprint review: night sky and Jyotish room |
| Thu 5 Nov | Demo with daily refresh live; retro |

## Review

(Add results here at sprint end.)
