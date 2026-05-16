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

# Normalise into this directory
python humdes_to_selemene.py
python humdes_html_enrich.py   # currently a no-op for missing sub-tabs
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
"ManifestingGenerator", Selemene engine labels "Generator". This is a
**known taxonomy boundary** in HD software — humdes is permissive (any G
with a defined motor-to-Throat is MG), Selemene is strict (G is default,
MG requires specific channel configuration). Documenting this divergence
is itself a finding worth reviewing.

## Training-data backend design

The 89 fixtures are the seed for a longer-running training corpus. The
plan below maps fixture → storage → consumption.

### Tier 1 — Ground truth (this directory)

Versioned per-snapshot. Each new `humdes_to_selemene.py` run writes a
fresh `_index.json`; old `tests/fixtures/humdes/_snapshots/<date>/` could
preserve history if you cron the refresh.

### Tier 2 — Engine-output deltas (proposed)

Add a `noesis-data` table:

```sql
CREATE TABLE humdes_validation_runs (
    id              UUID PRIMARY KEY,
    run_at          TIMESTAMPTZ NOT NULL,
    engine_version  TEXT NOT NULL,            -- env!("CARGO_PKG_VERSION")
    selemene_commit TEXT NOT NULL,            -- git sha
    fixtures_count  INT  NOT NULL,
    per_field_pct   JSONB NOT NULL,           -- {type: 92.1, profile: 100.0, ...}
    notes           TEXT
);

CREATE TABLE humdes_validation_records (
    id              UUID PRIMARY KEY,
    run_id          UUID REFERENCES humdes_validation_runs(id) ON DELETE CASCADE,
    person_id       TEXT NOT NULL,            -- humdes person_id
    reading_hash    TEXT NOT NULL,
    reading_type    TEXT NOT NULL,            -- personal/holo/compat/business/family
    field           TEXT NOT NULL,            -- 'type'|'profile'|'authority'|'personality_sun'|...
    expected        JSONB,                    -- humdes value
    got             JSONB NOT NULL,           -- engine value
    matched         BOOL NOT NULL,
    notes           TEXT
);
```

A `cargo test --features=record-validation` invocation could insert into
these tables (gate behind a feature so normal CI doesn't write). Lets you:
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
      activations + definition type as ground truth for additional fields.
- [ ] **MG-vs-G disagreement investigation.** Pick one of the 7
      mismatching fixtures, hand-trace through humdes vs Selemene logic,
      and decide whether to align (and which direction).
- [ ] **Snapshot policy.** Cron `humdes_to_selemene.py` weekly so we have
      history for tracking schema drift on humdes's side.
- [ ] **Tier 2 table migrations** in `noesis-data` + a writer behind a
      feature flag.
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
