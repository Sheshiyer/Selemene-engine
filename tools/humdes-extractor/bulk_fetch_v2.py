"""Bulk fetch v2 — navigate to each reading's natural URL and let the SPA fire
its own XHRs. We intercept every JSON response per reading.

Replaces the failed v1 (XHR-from-page.evaluate hit CORS preflight). v2 uses
the same proven pattern as auto_capture.py Phase 2: open the reading via
its natural URL, programmatically click each tab, capture everything.

Usage:
    python bulk_fetch_v2.py                  # uses latest auto_capture run
    python bulk_fetch_v2.py output/<run>     # use a specific directories run

Tunables at top of file:
    HEADLESS, MAX_READINGS, TAB_BUTTON_TEXTS, NAV_SETTLE_SECS
"""

from __future__ import annotations

import json
import re
import sys
import time
from datetime import datetime
from pathlib import Path
from urllib.parse import urlparse

from playwright.sync_api import Response, sync_playwright


# --- knobs ----------------------------------------------------------------
HEADLESS = False             # see it run; flip to True once trusted
MAX_READINGS = None          # None = all; integer for testing
NAV_SETTLE_SECS = 2.0        # wait after each navigation/click
NAV_TIMEOUT_MS = 30_000
CLICK_TIMEOUT_MS = 8_000
# -------------------------------------------------------------------------

ROOT = Path(__file__).resolve().parent
STATE_FILE = ROOT / "storageState.json"
OUTPUT_ROOT = ROOT / "output"
WARMUP_URL = "https://www.humdes.com/en/personal/results/#/personal"

# Tab labels to click on each reading's page. Order matches what we observed
# in auto_capture.py Phase 2. If a tab is missing for a given reading the
# click simply fails and we move on.
TAB_BUTTON_TEXTS = [
    "Ravechart",
    "Mechanics",
    "Variables and PHS",
    "Gates and Lines",
    "Wounds",
    "Hologenetics",
    "Planets Returns",
    "Transit",
    "Dream Rave",
]

TYPES = ["personal", "hologenetic", "compatibility", "business", "family"]

HOST_RE = re.compile(r"(^|\.)humdes\.com$", re.IGNORECASE)
SKIP_EXT = (".js", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp",
            ".woff", ".woff2", ".ttf", ".ico", ".map")


def _slug(text: str, max_len: int = 50) -> str:
    text = re.sub(r"[^\w\-]+", "_", str(text or "x"))
    text = re.sub(r"_+", "_", text).strip("_")
    return text[:max_len] or "x"


def _is_target_xhr(resp: Response) -> bool:
    host = urlparse(resp.url).hostname or ""
    if not HOST_RE.search(host):
        return False
    rtype = getattr(resp.request, "resource_type", "") or ""
    if rtype in {"image", "stylesheet", "font", "media", "manifest", "websocket", "document"}:
        return False
    p = urlparse(resp.url).path.lower()
    if any(p.endswith(ext) for ext in SKIP_EXT):
        return False
    return True


def _find_latest_auto_run() -> Path:
    candidates = [p for p in OUTPUT_ROOT.glob("*_auto") if (p / "raw").is_dir()]
    if not candidates:
        raise FileNotFoundError("No prior auto_capture run found.")
    return max(candidates, key=lambda p: p.stat().st_mtime)


def _extract_readings(auto_run: Path) -> list[dict]:
    raw_dir = auto_run / "raw"
    out: list[dict] = []
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
        for row in (obj.get("data", {}).get("rows") or []):
            if isinstance(row, dict) and "hash" in row and "link" in row:
                out.append({
                    "type": t,
                    "hash": row["hash"],
                    "name": (row.get("name") or "").strip(),
                    "link": row["link"],
                    "row": row,
                })
    # Dedupe by (type, hash)
    seen: set[tuple[str, str]] = set()
    deduped: list[dict] = []
    for r in out:
        key = (r["type"], r["hash"])
        if key in seen:
            continue
        seen.add(key)
        deduped.append(r)
    return deduped


def main() -> int:
    if not STATE_FILE.exists():
        print(f"ERROR: {STATE_FILE} not found. Run `python login.py` first.", file=sys.stderr)
        return 2

    auto_run = (Path(sys.argv[1]).resolve() if len(sys.argv) > 1
                else _find_latest_auto_run())
    print(f"Directories source : {auto_run}")
    readings = _extract_readings(auto_run)
    if not readings:
        print("No readings found.", file=sys.stderr)
        return 1
    if MAX_READINGS:
        readings = readings[:MAX_READINGS]
    print(f"Total readings     : {len(readings)}\n")

    run_dir = OUTPUT_ROOT / (datetime.now().strftime("%Y-%m-%d_%H%M%S") + "_bulk2")
    run_dir.mkdir(parents=True, exist_ok=True)
    print(f"Output: {run_dir}\n")

    summary: list[dict] = []
    total_xhrs = 0

    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=HEADLESS)
        context = browser.new_context(
            storage_state=str(STATE_FILE),
            viewport={"width": 1400, "height": 900},
            locale="en-US",
        )
        page = context.new_page()

        # Per-reading capture buffer (rebuilt at start of each reading)
        current_buffer: list[dict] = []

        def on_response(resp: Response) -> None:
            if not _is_target_xhr(resp):
                return
            try:
                body = resp.body()
            except Exception:  # noqa: BLE001
                return
            try:
                parsed = json.loads(body)
            except Exception:  # noqa: BLE001
                return
            current_buffer.append({
                "url": resp.url,
                "method": resp.request.method,
                "status": resp.status,
                "size": len(body),
                "data": parsed,
            })

        page.on("response", on_response)

        # Warm session on www.humdes.com so cookies + origin are set.
        page.goto(WARMUP_URL, wait_until="domcontentloaded", timeout=NAV_TIMEOUT_MS)
        try:
            page.wait_for_load_state("networkidle", timeout=15_000)
        except Exception:  # noqa: BLE001
            pass
        time.sleep(NAV_SETTLE_SECS)

        for idx, r in enumerate(readings, start=1):
            current_buffer.clear()
            slug = _slug(r["name"])
            reading_dir = run_dir / "readings" / r["type"] / f"{r['hash']}_{slug}"
            reading_dir.mkdir(parents=True, exist_ok=True)
            (reading_dir / "_row.json").write_text(
                json.dumps(r["row"], indent=2, ensure_ascii=False), encoding="utf-8"
            )

            print(f"[{idx:3d}/{len(readings)}] {r['type']:13s} {r['hash'][:10]}.. "
                  f"{r['name']!r:40s}")
            print(f"           navigate -> {r['link']}")
            try:
                page.goto(r["link"], wait_until="domcontentloaded", timeout=NAV_TIMEOUT_MS)
            except Exception as e:  # noqa: BLE001
                print(f"           goto FAIL: {e}")
                summary.append({"type": r["type"], "hash": r["hash"], "name": r["name"],
                                "status": "nav_fail", "xhrs": 0, "error": str(e)})
                continue
            try:
                page.wait_for_load_state("networkidle", timeout=NAV_TIMEOUT_MS)
            except Exception:  # noqa: BLE001
                pass
            time.sleep(NAV_SETTLE_SECS)
            after_nav_count = len(current_buffer)
            print(f"           after-nav XHRs: {after_nav_count}")

            # Click each tab label if present (best-effort).
            clicked_tabs: list[str] = []
            for label in TAB_BUTTON_TEXTS:
                before = len(current_buffer)
                try:
                    loc = page.get_by_text(label, exact=True).first
                    loc.scroll_into_view_if_needed(timeout=2000)
                    loc.click(timeout=CLICK_TIMEOUT_MS)
                except Exception:  # noqa: BLE001
                    continue
                try:
                    page.wait_for_load_state("networkidle", timeout=15_000)
                except Exception:  # noqa: BLE001
                    pass
                time.sleep(0.6)
                if len(current_buffer) > before:
                    clicked_tabs.append(label)

            # Save each captured XHR for this reading
            seq = 0
            saved_files: list[str] = []
            for entry in current_buffer:
                seq += 1
                parsed_url = urlparse(entry["url"])
                tag = _slug(parsed_url.path + "_" + parsed_url.query)
                out_name = f"{seq:02d}_{entry['method']}_{tag}.json"
                out_path = reading_dir / out_name
                out_path.write_text(
                    json.dumps(
                        {"_meta": {"url": entry["url"], "method": entry["method"],
                                   "status": entry["status"],
                                   "captured_at": datetime.now().isoformat(timespec="seconds")},
                         "data": entry["data"]},
                        indent=2, ensure_ascii=False,
                    ),
                    encoding="utf-8",
                )
                saved_files.append(out_name)

            total_xhrs += len(current_buffer)
            print(f"           clicked tabs: {clicked_tabs}")
            print(f"           xhrs saved  : {len(current_buffer)}")
            summary.append({
                "type": r["type"],
                "hash": r["hash"],
                "name": r["name"],
                "link": r["link"],
                "status": "ok" if current_buffer else "no_xhrs",
                "xhrs": len(current_buffer),
                "clicked_tabs": clicked_tabs,
                "files": saved_files,
            })

        browser.close()

    (run_dir / "_manifest.json").write_text(
        json.dumps({
            "source_directories": str(auto_run.name),
            "total_readings": len(readings),
            "total_xhrs_saved": total_xhrs,
            "summary": summary,
        }, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )
    ok = sum(1 for s in summary if s["status"] == "ok")
    print("\n=== Done ===")
    print(f"  Readings processed : {len(summary)}")
    print(f"  Readings with data : {ok}")
    print(f"  Total XHRs saved   : {total_xhrs}")
    print(f"  Output             : {run_dir}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
