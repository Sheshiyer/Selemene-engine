# Selemene Engine Patch Registry

Central tracking for all engine patches discovered through premium reading verification.

**Workflow:** Verify reading → Register patches → Fix engine → Re-verify → Mark verified

---

## Status Legend

| Status | Meaning |
|--------|---------|
| `open` | Discovered, not yet started |
| `in-progress` | Being worked on |
| `pr-ready` | PR submitted, awaiting review |
| `merged` | Merged to main |
| `verified` | Re-verified against original reading |
| `wont-fix` | Intentional behavior or out of scope |

---

## Patch Registry

| ID | Severity | Engine | Status | Found In | Summary | Verified In |
|----|----------|--------|--------|----------|---------|-------------|
| P0.1 | CRITICAL | gene-keys | open | sahil-2026-06-24 | Full Hologenetic Profile missing (only 4/11 positions returned) | — |
| P0.2 | HIGH | gene-keys | open | sahil-2026-06-24 | EQ "Reaction" field missing for legacy terminology support | — |
| P1.1 | HIGH | panchanga | open | sahil-2026-06-24 | Vara (weekday) timezone bug — returns UTC day, not local civil day | — |
| P1.2 | HIGH | human-design | open | sahil-2026-06-24 | Spleen center not detected when defined (missing from defined_centers) | — |
| P1.3 | HIGH | human-design | open | sahil-2026-06-24 | Active channels returns 2, visual shows ~13 (root cause: P1.2) | — |
| P1.4 | MEDIUM | vimshottari | open | sahil-2026-06-24 | Dasha date boundaries ~15 days earlier than Kundli tool (ayanamsa or node type) | — |
| P2.1 | MEDIUM | birth-blueprint | open | sahil-2026-06-24 | Workflow returns synthesis: null (cross-engine narrative missing) | — |
| P2.2 | MEDIUM | gene-keys | open | sahil-2026-06-24 | Pearl Sequence metadata incomplete (4 positions named, not all 4 sequences) | — |
| P2.3 | MEDIUM | vedic-chart | open | sahil-2026-06-24 | Planet dignity/debilitation flags not exposed (exalted, own sign, etc.) | — |
| P2.4 | MEDIUM | vedic-chart | open | sahil-2026-06-24 | Navamsa (D9) chart lacks full planet structure like D1 | — |
| P2.5 | LOW | numerology | open | sahil-2026-06-24 | destiny_number alias missing for life_path_number (terminology mismatch) | — |
| P3.1 | LOW | vedic-chart | open | sahil-2026-06-24 | Planet names lack Sanskrit transliteration (Surya, Chandra, etc.) | — |
| P3.2 | LOW | vedic-chart | open | sahil-2026-06-24 | Nakshatra Pada not explicitly returned in D1 planet objects | — |
| P3.3 | LOW | numerology | open | sahil-2026-06-24 | Chaldean vs Pythagorean system flag missing per number | — |
| P3.4 | LOW | vimshottari | open | sahil-2026-06-24 | Antardasha lord functional benefic/malefic classification missing | — |
| P4.1 | DOCS | all | open | sahil-2026-06-24 | Engine output lacks ephemeris/ayanamsa source citation | — |
| P4.2 | DOCS | all | open | sahil-2026-06-24 | Optional calculation_trace for intermediate step transparency | — |

---

## Statistics

| Severity | Count | Open | Fixed |
|----------|-------|------|-------|
| P0 Critical | 2 | 2 | 0 |
| P1 High | 4 | 4 | 0 |
| P2 Medium | 5 | 5 | 0 |
| P3 Low | 4 | 4 | 0 |
| P4 Docs | 2 | 2 | 0 |
| **Total** | **17** | **17** | **0** |

---

## Readings Verified

| Subject | Date | Patches Found | Status |
|---------|------|---------------|--------|
| Sahil Singh Sabharwal | 2026-06-24 | 17 | Complete |

---

## Readings Pending Verification

These readings from `723/` can be run through the engine to discover additional patches:

### Individual Readings
- [ ] witnessalchemist-reading/
- [ ] anitha-nateshan-reading/
- [ ] chitra-reading/
- [ ] cs-nateshan-reading/
- [ ] durgaprasad-reading/
- [ ] harshita-reading/
- [ ] mohan-reading/
- [ ] varsha-reading/
- [ ] anitha-nateshan-mother-reading/

### Synastry / Partnership Readings
- [ ] anitha-cs-nateshan-synastry/
- [ ] mohan-vandana-partnership/
- [ ] witness-x-mohan-partnership/
- [ ] witness-x-sahil-partnership/
- [ ] witnessalchemist-harshita-synastry/

### Family / Triad Readings
- [ ] family-triad-anitha-witnessalchemist-nateshan/
- [ ] family-triad-v2-l1l3/
- [ ] family-triad-v2-l4l5/
- [ ] triad-reading/

---

## Next Release Target: v3.2.0

Ship these 4 patches together (closes all Sahil verification gaps):

1. **P0.1** — Full Hologenetic Profile in Gene Keys engine
2. **P0.2** — Add `reaction` field for EQ shadow
3. **P1.1** — Fix Panchanga Vara timezone bug
4. **P1.2** — Fix Spleen center detection in Human Design

---

## Changelog

| Date | Action |
|------|--------|
| 2026-06-24 | Registry created with 17 patches from Sahil verification |
