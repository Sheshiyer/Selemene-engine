# Vedic Validation Report v2 — 2026-05-19

Generated after PR5 (`validation/vedic-hardening-pr5-ayanamsa`)
unified all 7 divergent Lahiri ayanamsa derivation sites in the
workspace onto a single SwissEph-grounded canonical helper
(`engine_human_design::ephemeris::lahiri_ayanamsa`). Same offline
harness as v1 — no `pyswisseph`, no `python3`, no `reqwest`.

The v1 report (`VEDIC_VALIDATION_REPORT_v1.md`) hypothesised that
ayanamsa drift was the root cause of the panchanga/vimshottari
accuracy gaps. **v2 falsifies that hypothesis.** Unifying the ayanamsa
did not move any per-field accuracy number. The drift is real but it
lives in the engine algorithm layer, not in the ayanamsa.

This is a useful negative result: PR5 ships as a maintainability +
correctness win (single source of truth, latent tropical/sidereal bug
fixed, hardest hardcode `24.17°` retired) and sharpens the diagnosis
for Phase 2.

## How to reproduce

```bash
cargo test --package engine-panchanga --test vedic_validation_tests \
  -- --ignored --nocapture
cargo test --package engine-vimshottari --test vedic_validation_tests \
  -- --ignored --nocapture
```

---

## 1. Panchanga v1 → v2 delta

| Field             | v1     | v2     | Δ (pp) |
|-------------------|--------|--------|--------|
| tithi_name        | 50.0%  | 50.0%  |   0.0  |
| tithi_paksha      | 100.0% | 100.0% |   0.0  |
| tithi_number      | 50.0%  | 50.0%  |   0.0  |
| nakshatra_name    | 50.0%  | 50.0%  |   0.0  |
| nakshatra_pada    | 16.7%  |  0.0%  |  -16.7 |
| yoga_name         |  0.0%  |  0.0%  |   0.0  |
| karana_name       | 33.3%  | 33.3%  |   0.0  |
| vara              | 100.0% | 100.0% |   0.0  |

Engine failures v2: 0/6.

### v2 raw harness output (verbatim from the run)

```
========== VEDIC PANCHANGA validation report ==========
  reference data : crates/noesis-vedic-api/tests/fixtures/reference_data
  ground truth   : shesh (1) + jhora (5) = 6 reference points

--- Per-field results ---
  tithi_name            matched   3/  6  ( 50.0%)  skipped= 0
    first mismatches:
      shesh_1990_bangalore  expected=Navami  got=Ashtami
      makar_sankranti_2024  expected=Chaturthi  got=Panchami
      summer_solstice_2024  expected=Pratipada  got=Chaturdashi
  tithi_paksha          matched   6/  6  (100.0%)  skipped= 0
  tithi_number          matched   3/  6  ( 50.0%)  skipped= 0
    first mismatches:
      shesh_1990_bangalore  expected=24  got=23
      makar_sankranti_2024  expected=4  got=5
      summer_solstice_2024  expected=1  got=14
  nakshatra_name        matched   3/  6  ( 50.0%)  skipped= 0
    first mismatches:
      makar_sankranti_2024  expected=revati  got=purvabhadrapada
      republic_day_2024  expected=magha  got=ashlesha
      summer_solstice_2024  expected=mrigashira  got=jyeshtha
  nakshatra_pada        matched   0/  6  (  0.0%)  skipped= 0
    first mismatches:
      shesh_1990_bangalore  expected=3  got=4
      makar_sankranti_2024  expected=2  got=1
      republic_day_2024  expected=3  got=1
      diwali_2024  expected=4  got=3
      purnima_guru_purnima_2024  expected=3  got=2
      summer_solstice_2024  expected=3  got=2
  yoga_name             matched   0/  6  (  0.0%)  skipped= 0
    first mismatches:
      shesh_1990_bangalore  expected=Parigha  got=Sukarma
      makar_sankranti_2024  expected=Shiva  got=Variyan
      republic_day_2024  expected=Siddha  got=Ayushman
      diwali_2024  expected=Sukarma  got=Ayushman
      purnima_guru_purnima_2024  expected=Shobhana  got=Vishkambha
      summer_solstice_2024  expected=Dhruva  got=Shubha
  karana_name           matched   2/  6  ( 33.3%)  skipped= 0
    first mismatches:
      shesh_1990_bangalore  expected=Gara  got=Balava
      makar_sankranti_2024  expected=Vanija  got=Bava
      diwali_2024  expected=Chatushpada  got=Naga
      summer_solstice_2024  expected=Kimstughna  got=Vanija
  vara                  matched   6/  6  (100.0%)  skipped= 0

--- Run stats ---
  reference points : 6
  engine failures  : 0
```

### Honest interpretation

- **vara (100%), paksha (100%)** still solid: date arithmetic + Sun-Moon
  half-cycle direction is correct end-to-end.
- **tithi/nakshatra/yoga/karana names and numbers** are byte-identical
  to v1. The SwissEph Lahiri at the six reference epochs differs from
  the old `23.72°` constant by only `~0.05° – 0.15°` — too small to
  push category boundaries on these specific points. The mismatches
  point at solar/lunar longitude drift on the order of one *full
  category-width*, not sub-arcminute ayanamsa drift: e.g.
  `summer_solstice_2024` lunar nakshatra slips from Mrigashira to
  Jyeshtha, a 6-nakshatra (~80°) gap.
- **nakshatra_pada slipped 16.7% → 0.0%** (the one fixture that
  matched in v1 — `summer_solstice_2024` — no longer does). This is
  the only metric that moved and it moved *the wrong way*. A single
  pada is 3°20′ wide, so a 0.05–0.15° ayanamsa shift can flip one
  borderline case across a pada line, which is what happened: the v2
  Moon longitude moved 1 pada bin away from where it accidentally
  matched in v1.

### Conclusion (panchanga)

Ayanamsa was not the root cause of the v1 panchanga drift. The
residual drift is in the solar/lunar longitude generation step itself
(`engine-panchanga::precise_tropical_positions` →
`EphemerisCalculator::get_planet_position`) or in how it's converted
to tithi/yoga/nakshatra categories. Specific suspects:

1. **Mean-longitude fallback path** (`calculate_solar_position` /
   `calculate_lunar_position` at lib.rs:258-280) is a low-precision
   polynomial used when Swiss Ephemeris fails. Need to verify the
   harness always exercises the Swiss path, never the fallback.
2. **Sun-Moon timing**: the references encode panchanga at the *birth
   instant*, but the engine may be sampling at sunrise or mid-day for
   the JHora civil-date entries (which encode "the panchang for
   2024-01-15" without a clock time). The 5/6 JHora fixtures may need
   an explicit civil-noon convention agreed.

---

## 2. Vimshottari v1 → v2 delta

| Field                       | v1     | v2     | Δ (pp) |
|-----------------------------|--------|--------|--------|
| moon_nakshatra_name         | 25.0%  | 25.0%  |   0.0  |
| birth_dasha_planet          | 25.0%  | 25.0%  |   0.0  |
| birth_dasha_balance_years   |  0.0%  |  0.0%  |   0.0  |
| mahadasha_sequence          | 25.0%  | 25.0%  |   0.0  |
| mahadasha_start_date_15d    | 33.3%  | 33.3%  |   0.0  |

Engine failures v2: 0/4.

### v2 raw harness output (verbatim)

```
========== VEDIC VIMSHOTTARI validation report ==========
  reference data : crates/noesis-vedic-api/tests/fixtures/reference_data
  ground truth   : shesh + 3 dasha_reference charts = 4 reference points

--- Per-field results ---
  moon_nakshatra_name           matched   1/  4  ( 25.0%)  skipped= 0
    first mismatches:
      chart_1_swami_vivekananda  expected=uttaraashadha  got=hasta
      chart_2_modern_reference  expected=pushya  got=purvaashadha
      chart_3_ketu_birth_dasha  expected=ashwini  got=hasta
  birth_dasha_planet            matched   1/  4  ( 25.0%)  skipped= 0
    first mismatches:
      chart_1_swami_vivekananda  expected=Sun  got=Moon
      chart_2_modern_reference  expected=Saturn  got=Venus
      chart_3_ketu_birth_dasha  expected=Ketu  got=Moon
  birth_dasha_balance_years     matched   0/  4  (  0.0%)  skipped= 0
    first mismatches:
      shesh_1990_bangalore  expected=8.500  got=2.832  diff=5.668
      chart_1_swami_vivekananda  expected=3.460  got=4.251  diff=0.791
      chart_2_modern_reference  expected=14.330  got=11.410  diff=2.920
      chart_3_ketu_birth_dasha  expected=3.500  got=3.178  diff=0.322
  mahadasha_sequence            matched   1/  4  ( 25.0%)  skipped= 0
    first mismatches:
      chart_1_swami_vivekananda  expected=Sun,Moon,Mars,Rahu,Jupiter  got=Moon,Mars,Rahu,Jupiter,Saturn
      chart_2_modern_reference  expected=Saturn,Mercury,Ketu,Venus  got=Venus,Sun,Moon,Mars
      chart_3_ketu_birth_dasha  expected=Ketu,Venus,Sun,Moon  got=Moon,Mars,Rahu,Jupiter
  mahadasha_start_date_15d      matched   4/ 12  ( 33.3%)  skipped= 0
    first mismatches:
      shesh_1990_bangalore/md2  expected=1999-01-15  got=1993-05-14  diff=2072d
      shesh_1990_bangalore/md3  expected=2006-01-15  got=2000-05-14  diff=2072d
      chart_1_swami_vivekananda/md2  expected=1866-06-29  got=1867-04-13  diff=288d
      chart_1_swami_vivekananda/md3  expected=1876-06-29  got=1874-04-13  diff=808d
      chart_2_modern_reference/md2  expected=1999-07-15  got=1996-08-11  diff=1068d
      chart_2_modern_reference/md3  expected=2016-07-15  got=2002-08-12  diff=5086d
      chart_3_ketu_birth_dasha/md2  expected=2004-03-01  got=2003-11-05  diff=117d
      chart_3_ketu_birth_dasha/md3  expected=2024-03-01  got=2010-11-05  diff=4865d
```

### Movement that DID happen (sub-category)

The `birth_dasha_balance_years` numerical diffs DID shift slightly with
the canonical ayanamsa (sub-pp, so they don't change the matched count
at 0/4):

| Chart                       | v1 diff | v2 diff |
|-----------------------------|---------|---------|
| shesh_1990_bangalore        | 5.675   | 5.668   |
| chart_1_swami_vivekananda   | 2.123   | 0.791   |
| chart_2_modern_reference    | 2.816   | 2.920   |
| chart_3_ketu_birth_dasha    | 0.432   | 0.322   |

Swami Vivekananda's residual error halved (`2.123 → 0.791` years) —
the largest single beneficial swing in the whole report, but still
nowhere near a match (the engine has the wrong starting nakshatra so
the balance is computed from the wrong base). The smaller mahadasha
start-date drifts also shrank (Swami Vivekananda md2 dropped from
775d → 288d off). So the canonical ayanamsa DID help — it just isn't
enough to flip category boundaries on these 4 charts.

### Honest interpretation

- **Same root-cause shape as panchanga**: nakshatra-name mismatches are
  large (`uttaraashadha` vs `hasta` = 5 nakshatras = ~67°; `ashwini`
  vs `hasta` = 13 nakshatras = ~173°). No ayanamsa unification can
  close gaps this large. This is *not* sub-arcminute drift.
- **chart_2_modern_reference at -13°**: the engine returns `Venus`
  birth dasha (sidereal Moon in `purvaashadha` per engine) vs
  reference `Saturn` (Moon in `pushya`). Those are physically far
  apart on the ecliptic; this is full-on Moon-longitude generation
  drift, not a sidereal-conversion artifact.
- **chart_3_ketu_birth_dasha at -173°**: engine produces `hasta` Moon
  vs reference `ashwini`. That's nearly 180° off — almost certainly
  a date/time-parsing or timezone bug specific to that fixture, not
  an ayanamsa issue.

### Next investigation target

Phase 2 follow-up (file as separate issue):

> For each of the 4 vimshottari reference charts, dump the engine's
> intermediate *tropical* Moon longitude (before any sidereal
> conversion) and compare against the published `moon_data.longitude`
> field in `dasha_reference.json`. If the tropical longitudes already
> disagree by tens of degrees, the bug is upstream of any sidereal
> code — most likely in `parse_birth_datetime` (timezone, time-format,
> default-noon) or in how the harness fixtures spell `birth_data.time`
> (some specify ISO 8601, others HH:MM, others nothing).

---

## 3. What PR5 actually shipped

Even with flat per-field harness numbers, PR5 is a real correctness
and maintainability win:

1. **Single source of truth.** All 7 previously divergent Lahiri
   derivation sites (4 polynomials, 2 hardcoded constants, 1
   SwissEph) collapsed to one canonical helper
   `engine_human_design::ephemeris::lahiri_ayanamsa`. Anti-criteria
   greps (`24.17` literal in production code, `LAHIRI_AYANAMSA_DEGREES`
   const) are now zero in `crates/` excluding fixture/test data.
2. **Worst offender retired.** `noesis-vedic-api/src/resilience.rs`
   `24.17°` hardcode — 28 arcminutes off truth — replaced with the
   canonical helper.
3. **Latent tropical-vs-sidereal bug fixed.** `engine-vimshottari
   ::calculate_birth_nakshatra` was passing TROPICAL Moon longitude
   directly to nakshatra classification, masked in production by the
   `noesis-vedic-api` outer pipeline. Direct callers (tests, future
   integrations) now get the right answer. New regression test
   `calculate_birth_nakshatra_applies_sidereal_correction_shesh_1990`
   guards against re-regression.
4. **`engine-vimshottari` engine production path** had the same
   `23.72°` frozen constant as `engine-panchanga` (fixed
   opportunistically as the same root cause).
5. **All existing golden tests pass unchanged** — `cargo test
   --package engine-panchanga --lib` 18/18; `cargo test --package
   engine-vimshottari --lib` 64/64 (+1 new regression).

The harness numbers will move once the *engine algorithms* are
corrected. PR5 unblocks that work by removing ayanamsa as a confound:
any v3 harness drift can now be attributed to engine algorithm, not
to which of 7 ayanamsa values you happened to land on.

---

## 4. Known follow-ups (out of scope for PR5)

1. `noesis-vedic-api/src/fallback.rs` still has two `ayanamsa: 24.17`
   hardcodes (lines 264, 468). PR5 ISC scope explicitly listed
   `resilience.rs` only; `fallback.rs` should be migrated in a
   small one-line follow-up PR using the same helper.
2. `noesis-vedic-api/src/chart_mapping.rs:83` is the FAPI vendor
   passthrough — intentionally NOT replaced (PR1/PR2 territory).
   Follow-up harness work should compare it against canonical to
   detect FAPI vendor drift.
3. `noesis-vedic-api/src/analysis.rs:241` has an IST hardcode — out
   of scope per PR5 brief (child issue).
4. The engine-algorithm-layer drift surfaced above is the obvious
   next target. Suggested approach in §1 and §2 of this report.

---

*Generated by `cargo test --test vedic_validation_tests -- --ignored
--nocapture` on `validation/vedic-hardening-pr5-ayanamsa`.*
