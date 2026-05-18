# humdes fixtures + training-data backend design

This directory holds normalised chart data captured from a logged-in
humdes.com profile (Bitrix-based commercial HD platform) — used as
**ground-truth** for validating Selemene's HD engine and as the seed
corpus for the future interpretation/training layer.

## What's here

```
tests/fixtures/humdes/
├── _index.json           # Flat catalogue of every person across all readings
├── _geocache.json        # location string -> {lat, lng, display_name}
└── readings/
    ├── personal/         # 47 readings × 1 person  = 47 fixtures
    ├── hologenetic/      #  2 readings × 1 person  =  2 fixtures
    ├── compatibility/    #  9 readings × 2 people  = 18 fixtures
    ├── business/         #  4 readings × ~2 people =  8 fixtures
    └── family/           #  3 readings × ~4 people = 14 fixtures
                          # Total: 89 person fixtures
```

Each per-person folder contains 3 files:
- `NN_input.json` — `EngineInput`-compatible (feed straight to any engine)
- `NN_expected.json` — humdes's authoritative answers (validation target)
- `NN_metadata.json` — raw humdes fields + provenance pointers

Where `NN` is the person's 1-based index within their reading (compatibility
charts have 01 + 02 etc.).

## Producing/refreshing this directory

Pipeline lives in `~/Downloads/humdes-extractor/`:

```bash
cd ~/Downloads/humdes-extractor
source .venv/bin/activate

# (Re-)capture from humdes.com (requires session)
python login.py            # once, interactive
python auto_capture.py     # ~30s
python bulk_fetch_v2.py    # ~5 min, captures 65 readings × ~9 tabs
python bulk_fetch_v3.py    # optional; v3 extends v2 to also fetch high-value
                           # sub-tabs (mechanics/certainty, gates/design+personal,
                           # etc.) via context.request — requires a fresh session.

# Normalise into this directory (use --stable-timestamp to avoid spurious
# per-fixture diffs from `current_time`/`normalised_at` re-rolls):
python humdes_to_selemene.py --stable-timestamp
python humdes_html_enrich.py   # extracts definition + strategy + not_self_theme
                               # from the Ravechart summary HTML where available
```

## Validation status (last run)

```
cargo test --package engine-human-design --test humdes_validation_tests \
    -- --ignored --nocapture
```

Field-by-field accuracy vs humdes ground truth:

| Field                | Matched   | Notes |
|----------------------|-----------|-------|
| profile              | 89 / 89   | Conscious/unconscious line both match |
| authority            | 89 / 89   | Sacral/Emotional/Splenic/Heart/G/Mental/Lunar |
| incarnation_cross    | 89 / 89   | All 4 cardinal gates (P-sun, P-earth, D-sun, D-earth) |
| personality_sun      | 49 / 49   | Where ground-truth available |
| personality_earth    | 49 / 49   | " |
| design_sun           | 49 / 49   | " |
| design_earth         | 49 / 49   | " |
| type                 | 82 / 89   | 7 cases of MG-vs-G classification disagreement |

The 7 type disagreements all match the same pattern: humdes labels
"ManifestingGenerator", Selemene engine labels "Generator". See
"MG/G boundary decision" below for the resolution.

## MG/G boundary decision

**Decision (HV-T01, issue #853):** Selemene's strict classifier is
**canonically correct**. The 7 humdes "MG" labels are accepted as a
known classifier divergence, not as Selemene bugs. No code change to
`crates/engine-human-design/src/analysis.rs` is required.

### The canonical definition

Ra Uru Hu's original Human Design typology defines a Manifesting
Generator as a chart with:

1. The **Sacral Centre defined**, AND
2. The **Throat Centre connected by a defined channel to one of the
   four motor centres** — Sacral, Heart, Solar Plexus, or Root.

Humdes' own mechanics-tab prose agrees verbatim with this definition.
From the mechanics body for fixture `c517b66135` (`personal/Ghanshyam`):

> "The Manifesting Generator has a defined Sacral Centre in the Rave
> Chart, and the Throat is always connected by Channels to at least one
> of the four motors — the Sacral, the Heart Centre, the Solar Plexus
> or the Root."

Selemene's `determine_type` + `is_throat_connected_to_motor` enforce
exactly this rule (motor centres are explicitly `Heart | SolarPlexus |
Root | Sacral`). Selemene returns `Generator` whenever the throat is
defined but connected only to non-motor centres (Ajna, Spleen, G,
Head). This is faithful to the original system.

### Why the 7 fixtures don't qualify

For each of the 7 disagreements, the engine output shows the Throat is
defined but **none** of the throat-touching channels reach a motor
centre:

| Fixture | Throat channels found | Motors touching Throat |
|---|---|---|
| `compatibility/9ce5ce4168/2` | 17-62 (Ajna), 48-16 (Spleen) | none |
| `family/0e2d49dced/2`         | 17-62 (Ajna), 48-16 (Spleen) | none |
| `personal/2a88b52b48/1`       | 8-1 (G)                       | none |
| `personal/62fed3949a/1`       | 33-13 (G)                     | none |
| `personal/a2b9b1f2b8/1`       | 17-62 (Ajna), 48-16 (Spleen) | none |
| `personal/c517b66135/1`       | 17-62 (Ajna), 48-16 (Spleen) | none |
| `personal/fd3fcad39b/1`       | 17-62 (Ajna), 48-16 (Spleen) | none |

Concretely for `c517b66135` (Ghanshyam, the worked example): defined
centres are `{Ajna, Throat, Sacral, Spleen, Root, G}`; the four active
channels are `17-62` (Ajna↔Throat), `48-16` (Spleen↔Throat), `5-15`
(Sacral↔G) and `32-54` (Spleen↔Root). There is **no path** — direct
**or transitive** — from Sacral to Throat through motor centres. The
Sacral connects only to G; the Throat connects only to Ajna and
Spleen. Per Ra's typology this is a pure Generator with a defined
Throat, not a Manifesting Generator.

The recurring pattern (5 of 7 fixtures share the same gate signature,
suggesting these are the same person captured in multiple readings) is
the **48-16 Spleen-to-Throat channel** ("Wavelength"). Spleen is an
awareness centre, not a motor — so its presence doesn't satisfy the
MG criterion regardless of how broadly the rule is read.

### Why humdes labels these as MG anyway

Humdes' classifier is permissive in a way their own descriptive text
isn't: empirically it labels a chart MG whenever **the Sacral is
defined and the Throat has any active channel**, regardless of which
centre that channel reaches. This is internally inconsistent (their
prose says "one of the four motors") but matches the labels on the
ground.

We do not match humdes' label here because:

1. It contradicts Ra Uru Hu's foundational definition of Type.
2. It contradicts humdes' own type-description prose, which Selemene's
   logic agrees with verbatim.
3. Selemene's strict rule preserves the meaningful distinction between
   "energy type with manifesting aura" (true MG) and "Generator who
   happens to have a defined Throat" (just a Generator). Collapsing
   these two erases the typology's clinical value.

### Operational consequences

- The validation harness's `type` field will continue to report
  82/89 (92.1%) when run against the humdes corpus. This is the
  ceiling for that corpus; further increases on this field require
  switching to a stricter ground-truth source.
- The HV-T01 investigation explicitly accepts this gap. Any future
  drift below 82/89 is a real regression in Selemene and should be
  investigated.
- If a downstream consumer needs to mirror humdes' permissive
  classification (e.g. for a side-by-side comparison UI), add a
  separate `humdes_compat_type` field rather than weakening the
  canonical `hd_type` field. Don't touch `analysis.rs::determine_type`.

### Reproducing the trace

The seven fixtures above were inspected by running the HD engine
against each via `engine_human_design::generate_hd_chart` and dumping
defined centres + active channels. The same can be reproduced by
loading any of the fixture `01_input.json` files into the engine
through `HumanDesignEngine::calculate` and reading the
`centers`/`channels` keys of the JSON result. The validation report
itself (above) emits the seven fixture ids.

### New fields in this revision (HV-T02 / #854)

Three additional ground-truth fields are now populated from the existing
humdes capture, extracted from the per-person Ravechart summary HTML in
`tabs/ravecard/ravecard/`:

| Field                  | Type                | Coverage  | Values |
|------------------------|---------------------|-----------|--------|
| `definition`           | enum                | 49 / 89   | `Single`, `Split`, `TripleSplit`, `QuadrupleSplit`, `NoDefinition` |
| `strategy`             | canonical token     | 49 / 89   | `WaitToRespond`, `WaitForInvitation`, `LunarCycle`, `Inform` |
| `not_self_theme`       | canonical token     | 49 / 89   | `Frustration`, `Bitterness`, `Anger`, `Disappointment` |

Each canonical field is shipped alongside its verbatim humdes label for
audit purposes (`strategy_humdes_label`, `not_self_humdes_label`). The
49/89 coverage corresponds to the 47 personal + 2 hologenetic readings;
multi-person readings (compatibility/business/family) don't expose the
per-person Ravechart summary in their existing capture and would require
an additional sub-tab fetch (blocked here on session re-auth — see "Open
work" below).

Validation harness consumption of these new fields lands in HV-T03.

## Training-data backend design

The 89 fixtures are the seed for a longer-running training corpus. The
plan below maps fixture → storage → consumption.

### Tier 1 — Ground truth (this directory)

Versioned per-snapshot. Each new `humdes_to_selemene.py` run writes a
fresh `_index.json`; old `tests/fixtures/humdes/_snapshots/<date>/` could
preserve history if you cron the refresh.

### Tier 2 — Engine-output deltas (HV-T04 / #856)

Landed as migration `031_humdes_validation.sql` (mirrored at
`supabase/migrations/20260518000031_031_humdes_validation.sql`). The schema:

```sql
CREATE TABLE humdes_validation_runs (
    id              UUID PRIMARY KEY,
    run_at          TIMESTAMPTZ NOT NULL,
    engine_version  TEXT NOT NULL,            -- env!("CARGO_PKG_VERSION")
    selemene_commit TEXT NOT NULL,            -- git sha
    fixtures_count  INT  NOT NULL,
    per_field_pct   JSONB NOT NULL,           -- {"type": 92.1, "profile": 100.0, ...}
    notes           TEXT
);

CREATE TABLE humdes_validation_records (
    id              UUID PRIMARY KEY,
    run_id          UUID NOT NULL
                    REFERENCES humdes_validation_runs(id) ON DELETE CASCADE,
    person_id       TEXT NOT NULL,
    reading_hash    TEXT NOT NULL,
    reading_type    TEXT NOT NULL,            -- personal|hologenetic|compatibility|business|family
    field           TEXT NOT NULL,            -- see noesis_data::humdes_validation::KNOWN_FIELDS
    expected        JSONB,                    -- humdes value (NULL = no ground truth)
    got             JSONB NOT NULL,           -- engine value
    matched         BOOL NOT NULL,
    notes           TEXT
);

CREATE INDEX idx_humdes_records_run_field    ON humdes_validation_records (run_id, field);
CREATE INDEX idx_humdes_records_person_field ON humdes_validation_records (person_id, field);
```

The writer lives at `crates/noesis-data/src/humdes_validation.rs`, gated
behind the `record-validation` Cargo feature. The default build of
`noesis-data` carries no reference to it, so CI and production paths are
untouched unless the feature is explicitly enabled.

Call shape (intended for a one-shot binary or future post-test hook —
deliberately not bolted into `humdes_validation_tests.rs` itself):

```rust
use noesis_data::humdes_validation::{
    record_validation_run, ValidationRun, ValidationRecord,
};

let run_id = record_validation_run(&pool, run, records).await?;
```

A working example is at `crates/noesis-data/examples/record_humdes_run.rs`:

```bash
DATABASE_URL=postgres://localhost/scratch_humdes \
    cargo run --package noesis-data --features record-validation \
        --example record_humdes_run
```

Top-level trend query — per-engine-version drift on the corpus, newest first:

```sql
SELECT
    engine_version,
    selemene_commit,
    fixtures_count,
    (per_field_pct->>'type')::float              AS type_pct,
    (per_field_pct->>'profile')::float           AS profile_pct,
    (per_field_pct->>'authority')::float         AS authority_pct,
    (per_field_pct->>'incarnation_cross')::float AS cross_pct,
    (per_field_pct->>'definition')::float        AS definition_pct,
    (per_field_pct->>'strategy')::float          AS strategy_pct,
    (per_field_pct->>'not_self_theme')::float    AS not_self_pct,
    run_at
FROM humdes_validation_runs
ORDER BY run_at DESC
LIMIT 20;
```

Per-fixture drill-down — "which charts started failing `type` in the
latest run?":

```sql
WITH latest AS (
    SELECT id FROM humdes_validation_runs
    ORDER BY run_at DESC LIMIT 1
)
SELECT person_id, reading_hash, reading_type, expected, got, notes
FROM humdes_validation_records
WHERE run_id = (SELECT id FROM latest)
  AND field = 'type'
  AND NOT matched
ORDER BY person_id;
```

Per-person trajectory — "when did fixture X start failing `type`?":

```sql
SELECT r.run_at, runs.engine_version, r.expected, r.got, r.matched
FROM humdes_validation_records r
JOIN humdes_validation_runs runs ON runs.id = r.run_id
WHERE r.person_id = 'c517b66135'
  AND r.field = 'type'
ORDER BY r.run_at DESC;
```

Lets you:
- Track regressions across engine versions
- Quantify drift after each refactor
- Surface specific birth-times that newly broke

### Tier 3 — Interpretation training (proposed, end-state)

Each humdes reading has 8 HTML tab bodies of professional commentary
(Type Mechanics, Inner Authority, Profile, Definition, Variables, etc.)
**These are the gold training data for the interpretation layer.** Tab
bodies are 3-21 KB each, so 89 persons × 8 tabs ≈ ~700 documents
totalling ~7 MB of curated HD interpretation prose tied to specific chart
configurations.

Recommended flow:

1. **Extraction.** Add `extract_tab_corpus.py` that walks every
   `reading/*/05_*_tabs_mec.json` etc., HTML→Markdown, writes one
   `corpus/<chart-signature>/<tab>.md` per (chart, tab). Chart signature =
   `{type}-{profile}-{authority}-{cross}`.

2. **Schema for interpretation embeddings:**

   ```sql
   CREATE TABLE interpretation_chunks (
       id            UUID PRIMARY KEY,
       chart_sig     TEXT NOT NULL,          -- "Projector-1/3-Splenic-Cross of X"
       tab           TEXT NOT NULL,          -- "mechanics"|"profile"|"variables"|...
       chunk_idx     INT  NOT NULL,
       text          TEXT NOT NULL,
       embedding     VECTOR(1536),            -- pgvector
       source        TEXT NOT NULL DEFAULT 'humdes',
       source_url    TEXT,
       captured_at   TIMESTAMPTZ NOT NULL
   );
   CREATE INDEX ON interpretation_chunks USING ivfflat (embedding vector_cosine_ops);
   ```

3. **Retrieval at reading-time.** When the Noesis API generates a new
   reading, embed the chart signature, top-k nearest neighbours from
   `interpretation_chunks`, and either:
   - Surface as "external references" (retrieval-augmented generation)
   - Or fine-tune the prompt template with retrieved examples
   - Or both

4. **Multi-source provenance.** humdes is one source. As more are added
   (jovianarchive, mybodygraph, geneticmatrix), the `source` column lets
   you weight, A/B, or attribute properly.

### Tier 4 — Cross-engine validation

The same input.json files validate every engine that takes birth data:
- engine-panchanga
- engine-numerology (date only)
- engine-biorhythm (date only)
- engine-vimshottari
- engine-gene-keys (re-uses HD gates)
- engine-vedic-clock
- engine-face-reading (no birth data needed — separate corpus)
- engine-transits

Add `tests/humdes_cross_engine.rs` that runs each engine on the same 89
fixtures and reports calc-time + non-empty-output rate. Even without
humdes ground truth for non-HD engines, this catches "engine X errors on
all charts from 1960 due to ephemeris range" type bugs cheaply.

## Open work

- [ ] **Capture the missing sub-tabs** (`tabs/gates/design/`,
      `tabs/mechanics/certainty/`, etc.) so we get humdes's per-gate
      activations + per-center definition data as ground truth for the
      remaining ≥40 fixtures. `bulk_fetch_v3.py` is the v2 fork that
      handles this — blocked here on a fresh humdes session
      (`storageState.json` expired). Once a fresh login lands, running
      `python bulk_fetch_v3.py` then `python humdes_html_enrich.py`
      against the new bulk3 capture will extend coverage to all 89
      fixtures and unlock `defined_centers` + `active_channels` extraction.
- [ ] **MG-vs-G disagreement investigation.** Pick one of the 7
      mismatching fixtures, hand-trace through humdes vs Selemene logic,
      and decide whether to align (and which direction).
- [ ] **Snapshot policy.** Cron `humdes_to_selemene.py` weekly so we have
      history for tracking schema drift on humdes's side.
- [x] **Tier 2 table migrations** in `noesis-data` + a writer behind a
      feature flag. *(HV-T04 #856 — migration `031_humdes_validation.sql`
      and `noesis_data::humdes_validation` behind `record-validation`.)*
- [ ] **Phase-1 extension for `compatibility`/`family` extra people.**
      Person[2+] doesn't get cardinal gates from `_row.json`; we'd need
      either per-person ravecard fetches or HTML parsing.

## Files in this design

- `_index.json` — index of all 89 fixtures
- `_geocache.json` — IANA location → lat/lng cache
- `readings/<type>/<hash>_<slug>/NN_{input,expected,metadata}.json`
- `../crates/engine-human-design/tests/humdes_validation_tests.rs` —
  validation test + report generator
- `../../../humdes-extractor/humdes_to_selemene.py` — normaliser
- `../../../humdes-extractor/humdes_html_enrich.py` — enricher (Phase 2)
