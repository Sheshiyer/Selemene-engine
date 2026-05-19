# humdes-extractor — Integration into the Selemene Engine Ecosystem

## Why this lives in `Selemene-engine/tools/`

This directory holds the data-ingestion + chart-computation tooling that
feeds the Selemene Engine ecosystem. It is **not standalone**. It is one
of three pre-pipeline workstreams that produce inputs for the rest of
the system:

```
                                 ┌────────────────────────────────┐
                                 │  tryambakam-noesis (umbrella)  │
                                 └────────────────────────────────┘
                                                │
                ┌───────────────────────────────┼───────────────────────────────┐
                ▼                               ▼                               ▼
       ┌─────────────────┐           ┌──────────────────┐            ┌──────────────────┐
       │ Selemene-engine │           │  witness-agents  │            │   (other layers) │
       │   (Rust core)   │           │  (TS synthesis)  │            │  symbolic media, │
       │                 │           │                  │            │  mentorship, …    │
       │ 16 engines      │ ◀──API──  │ Reading orches-  │            │                  │
       │ + tests/        │           │ trator, modes,   │            │                  │
       │ + tools/  ◀── this dir      │ NVIDIA routing   │            │                  │
       └─────────────────┘           └──────────────────┘            └──────────────────┘
                ▲                               ▲
                │                               │
                │ writes fixtures               │ writes subjects-dir
                │ to tests/fixtures/humdes/     │ for integratedreading-mode
                │                               │
                └───────────────┬───────────────┘
                                │
                          ┌──────────────┐
                          │   THIS DIR   │
                          │              │
                          │ humdes.com   │
                          │  capture +   │
                          │  Vedic       │
                          │  computation │
                          └──────────────┘
```

## What this tooling produces

| Output | Consumer | Path |
|---|---|---|
| 89-person ground-truth fixture corpus | `Selemene-engine/crates/engine-human-design/tests/humdes_validation_tests.rs` | `Selemene-engine/tests/fixtures/humdes/` |
| Per-parent / per-person reading bundles | manual review + `extract_parents.py` | `~/Downloads/humdes-extractor/parents/<slug>/` (private) |
| Subjects-dir JSONs (for synastry / triad / family modes) | `witness-agents/scripts/integratedreading-mode.ts` | `~/Downloads/humdes-extractor/parents/<mode>-<slug>/` (private) |
| Computed Vedic kundali markdown | witness-agents `source_reading_path` ingestion | `~/Downloads/humdes-extractor/parents/<slug>/inputs/Kundali_<name>.md` (private) |

## Privacy boundary

The **tools** are part of the Selemene Engine ecosystem (committed here).
The **outputs** contain personal birth data + authenticated humdes.com
session credentials — they stay private at `~/Downloads/humdes-extractor/`
and are excluded by `.gitignore`:

```
output/        # raw humdes.com captures (sessions, XHR responses, HTML)
parents/       # extracted per-person reading bundles
cookies.env    # auth cookies
storageState.json  # Playwright session
*.har          # network captures
.venv/         # python virtualenv
```

When you re-run any tool here, output writes to the **private** location
(default: `~/Downloads/humdes-extractor/<subdir>/`) unless overridden by
CLI flag.

## Setup

```bash
cd Selemene-engine/tools/humdes-extractor
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
playwright install chromium     # one-time, ~150 MB
```

Environment variable expected by the bulk-fetch + login scripts:
- `NVIDIA_API_KEY` (witness-agents calls; auto-loaded from `~/.claude/.env`)

## The full pipeline (canonical end-to-end run)

```bash
# 1. One-time: log into humdes.com
python login.py

# 2. Capture (re-run any time)
python auto_capture.py         # ~30s
python bulk_fetch_v2.py        # ~5 min, 65 readings × 9 tabs
python bulk_fetch_v3.py        # optional: extra sub-tabs (mechanics/certainty etc.)

# 3. Normalise into Selemene-engine test fixtures
python humdes_to_selemene.py --stable-timestamp
python humdes_html_enrich.py

# 4. Validate against Selemene HD engine
cd ../..    # back to Selemene-engine root
cargo test --package engine-human-design --test humdes_validation_tests -- --ignored --nocapture
```

For Vedic kundali computation (independent of humdes):
```bash
cd Selemene-engine/tools/humdes-extractor
python compute_vedic_kundali.py    # writes Kundali_<name>.md per declared subject
```

For building witness-agents subjects-dir (synastry / triad / family modes):
```bash
python build_synastry_subjects.py    # 2-person partner-synastry
python build_triad_subjects.py       # 3-person family-triad / composite-triad
```

## Scripts inventory

| Script | Purpose |
|---|---|
| `login.py` | One-time interactive Playwright login → `storageState.json` |
| `chrome_cookies.py` | Decrypt Chrome's cookie DB (alternative auth path) |
| `auto_capture.py` | Browser-driven Phase-1 capture (directories + tab metadata) |
| `bulk_fetch_v2.py` | Per-reading parent-tab capture (9 tabs each) |
| `bulk_fetch_v3.py` | Per-reading sub-tab capture (extends v2 with high-value sub-tabs) |
| `capture.py`, `replay.py` | Generic capture / re-fetch utilities |
| `explore.py`, `har_inspector.py` | Diagnostics for humdes SPA structure |
| `humdes_to_selemene.py` | Normalise captured humdes data → Selemene-engine fixtures |
| `humdes_html_enrich.py` | Extract `definition` / `strategy` / `not_self_theme` from HTML bodies |
| `compute_vedic_kundali.py` | pyswisseph + Lahiri sidereal → per-person Kundali markdown |
| `extract_parents.py` | Build per-person reading bundles for inspection |
| `extract.py` | Earlier-iteration single-reading extractor (kept for parity) |
| `build_synastry_subjects.py` | Compose 2-person witness-agents subjects-dir |
| `build_triad_subjects.py` | Compose 3-person family-triad / composite-triad subjects-dir |
| `humdes_client.py` | Reusable HTTP client for humdes endpoints |

## Reading-flow handoff to witness-agents

After this directory produces a `subjects-dir/`, the handoff to
witness-agents is:

```bash
cd ../../../witness-agents
node --import tsx scripts/integratedreading-mode.ts \
    --mode family-triad \
    --subjects-dir <absolute path to subjects-dir> \
    --output-dir <absolute path to 723/<slug>/> \
    --skip-solos
```

Witness-agents reads each subject's `output_dir` field to find cached
solo syntheses; the synastry / triad / family-triad / family-quad /
lineage-triad / blended-family / family-penta / partner-synastry /
business-partners / unmarried-romantic / composite-* / team-synergy
modes all consume the same subjects-dir shape.

## Why "humdes" lives in Selemene-engine specifically (not witness-agents)

The 89-person fixture corpus this tool produces is **test data for
the Selemene HD engine** — it lives at `Selemene-engine/tests/fixtures/
humdes/` because that's what the validation harness reads. The Vedic
kundali markdowns it produces ALSO feed witness-agents but are
generated from this same per-person workspace.

Putting the data-ingestion next to the engine (rather than next to the
orchestration layer) reflects that the GROUND TRUTH (humdes.com's
chart calculations) is what validates the engine's correctness, not
what drives the orchestration. The orchestration consumes the
already-validated chart computations.
