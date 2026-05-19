"""Bulk-fetch every reading's tab data via browser-driven XMLHttpRequest.

Reads the directory listings captured by auto_capture.py to extract every
reading's hash (across all 5 types), then opens a Playwright browser
positioned on the SPA's host so cookies + CORS behave normally, and fires
XMLHttpRequest from within page.evaluate for each (reading × tab) URL.

This sidesteps the 501 we got from direct APIRequestContext and the "Failed
to fetch" we got from fetch(): XHR with withCredentials is what the SPA
itself uses, so the server treats it identically.

Output: ./output/<timestamp>_bulk/
  readings/<type>/<hash>_<slug>/
    00_main.json
    01_ravecard.json
    02_mechanics.json
    ...
  _manifest.json

Usage:
    python bulk_fetch.py                  # uses the most recent auto_capture run
    python bulk_fetch.py output/<run>     # uses a specific auto_capture run for directories
"""

from __future__ import annotations

import json
import re
import sys
import time
from datetime import datetime
from pathlib import Path

from playwright.sync_api import sync_playwright


ROOT = Path(__file__).resolve().parent
STATE_FILE = ROOT / "storageState.json"
OUTPUT_ROOT = ROOT / "output"
WARMUP_URL = "https://www.humdes.com/en/personal/results/#/personal"
APP_HOST = "https://app.humdes.com"

TYPES = ["personal", "hologenetic", "compatibility", "business", "family"]

# Tabs we discovered from auto_capture observation. Each entry:
#   (output_filename_prefix, url_suffix_after-hash)
# Empty suffix = the main /ravecard/<hash>/ endpoint.
TABS: list[tuple[str, str]] = [
    ("00_main",              ""),
    ("01_ravecard",          "tabs/ravecard/ravecard"),
    ("02_mechanics",         "tabs/mechanics/type"),
    ("03_variables_phs",     "tabs/phs/variables"),
    ("04_gates_lines",       "tabs/gates/lines"),
    ("05_wounds",            "tabs/traumas/gentrauma"),
    ("06_hologenetics",      "tabs/hologenetic/profile"),
    ("07_planets_returns",   "tabs/dates/sun"),
    ("08_dream_rave",        "tabs/dream/sleepmap"),
]

HEADLESS = True       # XHRs don't need a visible browser
INTER_REQUEST_SLEEP = 0.15   # be polite, avoid hammering the server


def _slug(text: str, max_len: int = 50) -> str:
    text = re.sub(r"[^\w\-]+", "_", str(text or "x"))
    text = re.sub(r"_+", "_", text).strip("_")
    return text[:max_len] or "x"


def _find_latest_auto_run() -> Path:
    candidates = [p for p in OUTPUT_ROOT.glob("*_auto") if (p / "raw").is_dir()]
    if not candidates:
        raise FileNotFoundError(
            "No prior auto_capture run found. Run `python auto_capture.py` first."
        )
    return max(candidates, key=lambda p: p.stat().st_mtime)


def _extract_readings(auto_run: Path) -> list[dict]:
    """Pull the 5 directory JSONs from an auto_capture run and flatten the rows."""
    raw_dir = auto_run / "raw"
    readings: list[dict] = []

    # The directory XHRs are named 0001..0005 (one per type in TYPES order),
    # but the actual order can vary across runs. Match by URL substring.
    for path in sorted(raw_dir.glob("*.json")):
        try:
            obj = json.loads(path.read_text(encoding="utf-8"))
        except Exception:  # noqa: BLE001
            continue
        url = obj.get("_meta", {}).get("url", "")
        m = re.search(r"type=([a-z]+)", url)
        if not m:
            continue
        t = m.group(1)
        if t not in TYPES:
            continue
        rows = obj.get("data", {}).get("rows") or []
        for row in rows:
            if not isinstance(row, dict) or "hash" not in row:
                continue
            readings.append({
                "type": t,
                "hash": row["hash"],
                "name": row.get("name", "").strip(),
                "row": row,
            })

    # Dedupe by (type, hash)
    seen: set[tuple[str, str]] = set()
    out: list[dict] = []
    for r in readings:
        key = (r["type"], r["hash"])
        if key in seen:
            continue
        seen.add(key)
        out.append(r)
    return out


XHR_SCRIPT = r"""
async (url) => {
    return new Promise((resolve) => {
        const xhr = new XMLHttpRequest();
        xhr.open('GET', url, true);
        xhr.withCredentials = true;
        try { xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest'); } catch(e) {}
        try { xhr.setRequestHeader('Accept', 'application/json, text/plain, */*'); } catch(e) {}
        xhr.onload = () => resolve({
            status: xhr.status,
            ok: xhr.status >= 200 && xhr.status < 300,
            body: xhr.responseText,
            content_type: xhr.getResponseHeader('content-type') || ''
        });
        xhr.onerror = (e) => resolve({status: 0, ok: false, error: 'XHR network error'});
        xhr.ontimeout = () => resolve({status: 0, ok: false, error: 'XHR timeout'});
        xhr.timeout = 30000;
        xhr.send();
    });
}
"""


def main() -> int:
    if not STATE_FILE.exists():
        print(f"ERROR: {STATE_FILE} not found. Run `python login.py` first.", file=sys.stderr)
        return 2

    auto_run = (Path(sys.argv[1]).resolve() if len(sys.argv) > 1
                else _find_latest_auto_run())
    print(f"Reading directories from: {auto_run}")

    readings = _extract_readings(auto_run)
    if not readings:
        print("No readings found in directory dumps. Re-run auto_capture.py first.",
              file=sys.stderr)
        return 1

    print(f"Found {len(readings)} readings across {len(TYPES)} types.")
    by_type: dict[str, int] = {}
    for r in readings:
        by_type[r["type"]] = by_type.get(r["type"], 0) + 1
    for t in TYPES:
        print(f"  {t:15s} {by_type.get(t, 0)}")

    run_dir = OUTPUT_ROOT / (datetime.now().strftime("%Y-%m-%d_%H%M%S") + "_bulk")
    run_dir.mkdir(parents=True, exist_ok=True)
    print(f"\nOutput: {run_dir}\n")

    manifest: list[dict] = []

    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=HEADLESS)
        context = browser.new_context(
            storage_state=str(STATE_FILE),
            viewport={"width": 1400, "height": 900},
            locale="en-US",
        )
        page = context.new_page()

        # Warm the browser on the SPA so cookies + origin are set.
        page.goto(WARMUP_URL, wait_until="domcontentloaded", timeout=30_000)
        try:
            page.wait_for_load_state("networkidle", timeout=15_000)
        except Exception:  # noqa: BLE001
            pass
        time.sleep(1)

        total = len(readings) * len(TABS)
        done = 0
        ok = 0
        non_json = 0
        errors = 0

        for r in readings:
            slug = _slug(r["name"])
            reading_dir = run_dir / "readings" / r["type"] / f"{r['hash']}_{slug}"
            reading_dir.mkdir(parents=True, exist_ok=True)

            # Save the directory row for context
            (reading_dir / "_row.json").write_text(
                json.dumps(r["row"], indent=2, ensure_ascii=False), encoding="utf-8"
            )

            for fname_prefix, suffix in TABS:
                done += 1
                if suffix:
                    url = f"{APP_HOST}/ravecard/{r['hash']}/{suffix}/?site=1"
                else:
                    url = f"{APP_HOST}/ravecard/{r['hash']}/?site=1"

                try:
                    result = page.evaluate(XHR_SCRIPT, url)
                except Exception as e:  # noqa: BLE001
                    print(f"  [{done:4d}/{total}] {r['type']:13s} {r['hash'][:10]}.. "
                          f"{fname_prefix} EVAL FAIL: {e}")
                    errors += 1
                    continue

                status = result.get("status")
                body = result.get("body") or ""
                ct = result.get("content_type", "")
                parsed = None
                try:
                    parsed = json.loads(body)
                except Exception:  # noqa: BLE001
                    pass

                entry = {
                    "type": r["type"],
                    "hash": r["hash"],
                    "name": r["name"],
                    "tab": fname_prefix,
                    "url": url,
                    "status": status,
                    "is_json": parsed is not None,
                    "size": len(body),
                }

                if parsed is not None:
                    out_path = reading_dir / f"{fname_prefix}.json"
                    envelope = {
                        "_meta": {
                            "url": url,
                            "type": r["type"],
                            "hash": r["hash"],
                            "name": r["name"],
                            "tab": fname_prefix,
                            "status": status,
                            "content_type": ct,
                            "captured_at": datetime.now().isoformat(timespec="seconds"),
                        },
                        "data": parsed,
                    }
                    out_path.write_text(json.dumps(envelope, indent=2, ensure_ascii=False),
                                        encoding="utf-8")
                    ok += 1
                    entry["file"] = str(out_path.relative_to(run_dir))
                else:
                    out_path = reading_dir / f"{fname_prefix}.raw"
                    out_path.write_text(body, encoding="utf-8")
                    non_json += 1
                    entry["file"] = str(out_path.relative_to(run_dir))

                manifest.append(entry)

                # Progress indicator every 20 requests
                if done % 20 == 0 or done == total:
                    print(f"  [{done:4d}/{total}] ok={ok}  non-json={non_json}  errors={errors}  "
                          f"latest: {r['type']:13s} {fname_prefix} ({status})")

                time.sleep(INTER_REQUEST_SLEEP)

        browser.close()

    # Final manifest
    (run_dir / "_manifest.json").write_text(
        json.dumps({
            "source_directories": str(auto_run.name),
            "totals": {
                "readings": len(readings),
                "requests": total,
                "ok": ok,
                "non_json": non_json,
                "errors": errors,
            },
            "by_type": by_type,
            "entries": manifest,
        }, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )

    print("\n=== Done ===")
    print(f"  Readings           : {len(readings)}")
    print(f"  Requests fired     : {total}")
    print(f"  JSON saved         : {ok}")
    print(f"  Non-JSON responses : {non_json}  (saved as .raw for inspection)")
    print(f"  Errors             : {errors}")
    print(f"  Output             : {run_dir}")
    print(f"  Manifest           : {run_dir / '_manifest.json'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
