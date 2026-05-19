# humdes-extractor

Pulls every saved chart/reading from your logged-in humdes.com profile into local JSON files. Works against the live SPA — no API keys, no manual cookie wrangling.

## The working pipeline (3 scripts)

```bash
cd ~/Downloads/humdes-extractor
source .venv/bin/activate

python login.py          # once: log in interactively, saves storageState.json
python auto_capture.py   # any time: fetches all 5 type directories
python bulk_fetch_v2.py  # any time: navigates each reading + captures all tab data
```

What each one does:

| Script | Role | Time |
|---|---|---|
| `login.py` | Opens Chromium; you sign in manually; saves auth to `storageState.json`. Run once and whenever the session expires. | ~30s + your login |
| `auto_capture.py` | Browser-driven. Visits each of the 5 category hashes (personal, hologenetic, compatibility, business, family) so the SPA fires the directory XHRs we intercept. Saves each directory under `output/<run>_auto/raw/`. | ~30s |
| `bulk_fetch_v2.py` | Reads directories from latest `_auto` run to get all reading hashes + links. Navigates to each reading's public URL (`https://www.humdes.com/en/results/<slug>/`) and clicks every tab — the SPA fires its own XHRs which we capture and save. | ~5 min for 65 readings |

## Output layout

```
output/<timestamp>_bulk2/
    readings/
        personal/
            <hash>_<name>/
                _row.json                              # directory metadata
                01_GET_ravecard_<hash>_site_1.json    # main chart data
                02_GET_results_list_….json            # directory snapshot
                03_GET_ravecard_list_….json           # full ravecard list
                04_GET_…_tabs_rav.json                # Ravechart tab (HTML body)
                05_GET_…_tabs_mec.json                # Mechanics tab
                06_GET_…_tabs_phs.json                # Variables & PHS
                07_GET_…_tabs_gat.json                # Gates & Lines
                08_GET_…_tabs_tra.json                # Wounds / Traumas
                09_GET_…_tabs_hol.json                # Hologenetics
                10_GET_…_tabs_dat.json                # Dates / Planets-returns
                11_GET_…_tabs_pla.json                # Birth data (structured)
        hologenetic/
        compatibility/
        business/
        family/
    _manifest.json                                     # index of every reading
```

Each saved JSON is wrapped:
```json
{
  "_meta": {
    "url": "https://app.humdes.com/ravecard/<hash>/tabs/<tab>/?site=1",
    "status": 200,
    "captured_at": "2026-05-16T12:55:00"
  },
  "data": { ...response body... }
}
```

For tabs, `data.body` is the rendered HTML string. For main chart calls (`01_*` and `11_*`), `data` is structured JSON (32 fields including id, type, profile, authority, gates, planets).

## Setup (one-time)

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
playwright install chromium    # ~150 MB
python login.py                # interactive sign-in
```

## Refreshing data

```bash
python auto_capture.py && python bulk_fetch_v2.py
```

If you ever get redirected to login (session expired):
```bash
python login.py
```

## Optional: schedule periodic snapshots

```bash
# Add to crontab -e — daily at 9am
0 9 * * * cd ~/Downloads/humdes-extractor && .venv/bin/python auto_capture.py && .venv/bin/python bulk_fetch_v2.py >> humdes.log 2>&1
```

Flip `HEADLESS = True` at the top of both `auto_capture.py` and `bulk_fetch_v2.py` for true unattended runs.

## Files in this repo

**Working pipeline:**
- `login.py` — interactive login
- `auto_capture.py` — directory fetcher
- `bulk_fetch_v2.py` — per-reading fetcher
- `storageState.json` (gitignored) — saved auth state

**Helpers / diagnostics:**
- `explore.py` — one-shot DOM/XHR exploration (we used this to find the API structure)
- `capture.py` — generic interactive capture-while-you-click (used in earlier debugging)
- `replay.py` — headless replay of a `capture.py` manifest (works for capture-style runs, not bulk_fetch)

**Deprecated / dead-end paths** (kept for reference but not part of the working flow):
- `chrome_cookies.py`, `humdes_client.py`, `extract.py`, `har_inspector.py`, `cookies.env.example` — cookie-DB-decryption approach. macOS Chrome's app-bound encryption and in-memory session cookies made this fragile. Playwright path is much better.
- `bulk_fetch.py` — v1 of bulk fetch, used `XMLHttpRequest` from `page.evaluate`. Blocked by browser CORS preflight when origin differs. Replaced by `bulk_fetch_v2.py`.

## Notes

- `storageState.json` contains live session credentials — it's in `.gitignore`.
- All data stays local; nothing is uploaded.
- The tab body fields are full HTML — if you want structured data per tab, run an HTML parser (`pip install beautifulsoup4`) over `data.body` and extract whatever you need.
