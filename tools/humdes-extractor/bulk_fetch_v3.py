"""Bulk fetch v3 — same proven pattern as v2, but also iterates each
ravecard's `tabs[*].childs[*]` sub-tabs and fetches their URLs directly.

The humdes SPA exposes per-parent sub-tabs (e.g. mechanics has 8 children:
type, iauthority, profile, certainty, centerswhat, centersclosed, centersopen,
channelsactive). v2 only captured the FIRST child per parent (the one loaded
when the parent label is clicked). v3 additionally fetches a curated set of
high-value child URLs that contain the structured payloads we need for the
HD validation harness:

  - mechanics/certainty      -> Single/Split/TripleSplit/QuadrupleSplit definition
  - mechanics/centersopen    -> open centers list
  - mechanics/centersclosed  -> defined centers list (closed = defined)
  - mechanics/channelsactive -> active channels
  - gates/design             -> 13 design-side gate activations
  - gates/personal           -> 13 personality-side gate activations

Each child URL responds with `{"body": "<html>..."}` (same shape as v2's
parent-tab captures), so the on_response handler stays unchanged.

Usage:
    python bulk_fetch_v3.py                  # uses latest auto_capture run
    python bulk_fetch_v3.py output/<run>     # use a specific directories run

Tunables at top of file:
    HEADLESS, MAX_READINGS, TAB_BUTTON_TEXTS, SUB_TAB_CODES, NAV_SETTLE_SECS
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
SUB_TAB_SETTLE_SECS = 0.8    # gap between sub-tab fetches
SUB_TAB_TIMEOUT_MS = 20_000
# -------------------------------------------------------------------------

ROOT = Path(__file__).resolve().parent
STATE_FILE = ROOT / "storageState.json"
OUTPUT_ROOT = ROOT / "output"
WARMUP_URL = "https://www.humdes.com/en/personal/results/#/personal"

# Parent tab labels we click on each reading's page (same as v2).
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

# After parent-tab clicks complete, fetch these (parent_code, child_code)
# combinations directly from each ravecard's child URL. The HTML payload
# at these endpoints contains the structured data the validation harness
# needs but which is NOT in the default child loaded with the parent tab.
SUB_TAB_CODES: list[tuple[str, str]] = [
    ("mechanics", "certainty"),
    ("mechanics", "centersopen"),
    ("mechanics", "centersclosed"),
    ("mechanics", "channelsactive"),
    ("gates", "design"),
    ("gates", "personal"),
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


def _sub_tab_urls(ravecard_data: dict) -> dict[tuple[str, str], str]:
    """Given the GET /ravecard/<hash> response data, return a map of
    {(parent_code, child_code): child_url} for every sub-tab we want."""
    wanted = set(SUB_TAB_CODES)
    out: dict[tuple[str, str], str] = {}
    for t in ravecard_data.get("tabs", []) or []:
        parent = t.get("code")
        for c in (t.get("childs") or []):
            key = (parent, c.get("code"))
            if key in wanted and c.get("url"):
                out[key] = c["url"]
    return out


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

    run_dir = OUTPUT_ROOT / (datetime.now().strftime("%Y-%m-%d_%H%M%S") + "_bulk3")
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
        # Cache of the first GET /ravecard/<hash> response per reading so
        # we can look up sub-tab URLs after the parent clicks finish.
        current_ravecard: dict = {}

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
            # Capture the ravecard root payload so sub-tab discovery works.
            # The root URL looks like:
            #   https://app.humdes.com/ravecard/<hash>?site=1
            # (no /tabs/ segment)
            path = urlparse(resp.url).path
            if (isinstance(parsed, dict)
                    and "/tabs/" not in path
                    and "ravecards" in parsed
                    and "tabs" in parsed):
                current_ravecard.clear()
                current_ravecard.update(parsed)

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
            current_ravecard.clear()
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

            # Click each parent tab label if present (best-effort, same as v2).
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

            # Sub-tab capture: look up child URLs from the ravecard root
            # payload (captured above), then directly fetch each one via
            # context.request — same auth as the page, no DOM interaction.
            sub_fetched: list[str] = []
            sub_failed: list[str] = []
            if current_ravecard:
                url_map = _sub_tab_urls(current_ravecard)
                for (parent, child), child_url in sorted(url_map.items()):
                    # Add ?site=1 if not present; parent URLs always have it
                    target = child_url
                    if "?site=" not in target:
                        target = target.rstrip("/") + "/?site=1"
                    before = len(current_buffer)
                    try:
                        # context.request fires through the same cookie jar;
                        # the response goes through on_response listener too?
                        # No — page.on('response') only fires for page nav.
                        # So we capture manually here.
                        api_resp = context.request.get(
                            target, timeout=SUB_TAB_TIMEOUT_MS
                        )
                        if api_resp.ok:
                            try:
                                parsed = api_resp.json()
                            except Exception:  # noqa: BLE001
                                parsed = None
                            if parsed is not None:
                                current_buffer.append({
                                    "url": target,
                                    "method": "GET",
                                    "status": api_resp.status,
                                    "size": len(api_resp.body() or b""),
                                    "data": parsed,
                                })
                                sub_fetched.append(f"{parent}/{child}")
                            else:
                                sub_failed.append(f"{parent}/{child}:bad-json")
                        else:
                            sub_failed.append(f"{parent}/{child}:{api_resp.status}")
                    except Exception as e:  # noqa: BLE001
                        sub_failed.append(f"{parent}/{child}:err")
                    time.sleep(SUB_TAB_SETTLE_SECS)
            else:
                print(f"           WARN: no ravecard root payload captured; "
                      f"skipping sub-tab fetch")

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
            print(f"           sub-tabs    : {sub_fetched}")
            if sub_failed:
                print(f"           sub-fail    : {sub_failed}")
            print(f"           xhrs saved  : {len(current_buffer)}")
            summary.append({
                "type": r["type"],
                "hash": r["hash"],
                "name": r["name"],
                "link": r["link"],
                "status": "ok" if current_buffer else "no_xhrs",
                "xhrs": len(current_buffer),
                "clicked_tabs": clicked_tabs,
                "sub_tabs": sub_fetched,
                "sub_failed": sub_failed,
                "files": saved_files,
            })

        browser.close()

    (run_dir / "_manifest.json").write_text(
        json.dumps({
            "source_directories": str(auto_run.name),
            "total_readings": len(readings),
            "total_xhrs_saved": total_xhrs,
            "sub_tab_codes": [f"{p}/{c}" for p, c in SUB_TAB_CODES],
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
