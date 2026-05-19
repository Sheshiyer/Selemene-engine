"""Exploration pass: open the humdes results SPA and dump everything we need
to know to drive it programmatically.

Outputs to ./exploration/:
  - page.png            : full-page screenshot
  - dom.html            : the live DOM after JS has run
  - clickables.json     : every candidate "reading" element with selector + label
  - hash_walk.json      : what happens when each candidate is clicked
                          (route change, XHR URLs that fire)
  - initial_xhrs.json   : XHR JSON responses captured on initial load

After this runs, paste the candidate list back to me and I'll wire
auto_capture.py against the real selectors.

Usage:
    python explore.py
"""

from __future__ import annotations

import json
import re
import sys
import time
from datetime import datetime
from pathlib import Path
from urllib.parse import urlparse

from playwright.sync_api import Page, Response, sync_playwright


ROOT = Path(__file__).resolve().parent
STATE_FILE = ROOT / "storageState.json"
OUT_DIR = ROOT / "exploration"
START_URL = "https://www.humdes.com/en/personal/results/#/personal"

HOST_RE = re.compile(r"(^|\.)humdes\.com$", re.IGNORECASE)
SKIP_EXT = (".js", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp",
            ".woff", ".woff2", ".ttf", ".ico", ".map")


def _is_target(url: str, resource_type: str) -> bool:
    host = urlparse(url).hostname or ""
    if not HOST_RE.search(host):
        return False
    if resource_type in {"image", "stylesheet", "font", "media", "manifest", "websocket"}:
        return False
    path = urlparse(url).path.lower()
    return not any(path.endswith(ext) for ext in SKIP_EXT)


def _sniff_json(body: bytes) -> object | None:
    try:
        return json.loads(body)
    except Exception:  # noqa: BLE001
        return None


def _xhr_log_factory(sink: list[dict]):
    def handler(resp: Response) -> None:
        if not _is_target(resp.url, getattr(resp.request, "resource_type", "") or ""):
            return
        try:
            body = resp.body()
        except Exception:  # noqa: BLE001
            return
        parsed = _sniff_json(body)
        sink.append({
            "url": resp.url,
            "method": resp.request.method,
            "status": resp.status,
            "content_type": resp.headers.get("content-type"),
            "is_json": parsed is not None,
            "size": len(body),
            "post_data": resp.request.post_data,
            "json_preview": (
                json.dumps(parsed, ensure_ascii=False)[:300] if parsed is not None
                else body[:120].decode("utf-8", errors="replace")
            ),
        })
    return handler


def _harvest_clickables(page: Page) -> list[dict]:
    """Find every plausible "open this reading" element.

    Looks for hash-routed anchors, items inside list containers, and elements
    whose text suggests names/dates/profiles. Returns dicts with a stable
    locator strategy.
    """
    script = r"""
    () => {
        const out = [];
        const seen = new Set();

        // Strategy 1: anchors that look like hash routes
        document.querySelectorAll('a[href*="#"]').forEach((el, i) => {
            const href = el.getAttribute('href') || '';
            const text = (el.innerText || el.textContent || '').trim().slice(0, 120);
            if (!text) return;
            const key = href + '|' + text;
            if (seen.has(key)) return;
            seen.add(key);
            out.push({
                strategy: 'anchor-hash',
                href, text,
                tag: el.tagName.toLowerCase(),
                classes: el.className?.toString?.() || '',
                dataset: Object.assign({}, el.dataset),
                index: i,
            });
        });

        // Strategy 2: buttons / list items whose text length suggests names
        const candidates = document.querySelectorAll(
            'button, [role="button"], [role="listitem"], li, .reading, .card, ' +
            '[class*="reading"], [class*="chart"], [class*="profile"], [class*="item"]'
        );
        candidates.forEach((el, i) => {
            const text = (el.innerText || el.textContent || '').trim();
            if (!text || text.length > 160 || text.length < 2) return;
            const tag = el.tagName.toLowerCase();
            const cls = el.className?.toString?.() || '';
            const key = tag + '|' + cls + '|' + text.slice(0, 80);
            if (seen.has(key)) return;
            seen.add(key);
            out.push({
                strategy: 'clickable',
                tag,
                classes: cls,
                text: text.slice(0, 160),
                dataset: Object.assign({}, el.dataset),
                index: i,
            });
        });

        return out.slice(0, 200);  // cap so output stays readable
    }
    """
    return page.evaluate(script)


def main() -> int:
    if not STATE_FILE.exists():
        print(f"ERROR: {STATE_FILE} not found. Run `python login.py` first.", file=sys.stderr)
        return 2

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    print(f"Output: {OUT_DIR}")

    initial_xhrs: list[dict] = []

    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=False)
        context = browser.new_context(
            storage_state=str(STATE_FILE),
            viewport={"width": 1400, "height": 900},
            locale="en-US",
        )
        page = context.new_page()
        page.on("response", _xhr_log_factory(initial_xhrs))

        print(f"Loading {START_URL} ...")
        page.goto(START_URL, wait_until="networkidle", timeout=30_000)

        # Give the SPA an extra moment to settle.
        time.sleep(2)

        # Hard reload to capture the full initial XHR cascade fresh.
        print("Hard reloading to capture full XHR cascade ...")
        initial_xhrs.clear()
        page.evaluate("location.reload(true)")
        page.wait_for_load_state("networkidle", timeout=30_000)
        time.sleep(2)

        print(f"Captured {len(initial_xhrs)} candidate XHRs on initial load.")

        # Screenshot + DOM dump
        page.screenshot(path=str(OUT_DIR / "page.png"), full_page=True)
        (OUT_DIR / "dom.html").write_text(page.content(), encoding="utf-8")
        (OUT_DIR / "url_after_load.txt").write_text(page.url, encoding="utf-8")
        print(f"  screenshot   -> {OUT_DIR / 'page.png'}")
        print(f"  dom          -> {OUT_DIR / 'dom.html'} ({(OUT_DIR / 'dom.html').stat().st_size} bytes)")

        # Find clickable candidates
        print("Harvesting clickable candidates ...")
        clickables = _harvest_clickables(page)
        (OUT_DIR / "clickables.json").write_text(
            json.dumps(clickables, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        print(f"  candidates   -> {len(clickables)} elements written to clickables.json")

        # Save initial XHRs
        (OUT_DIR / "initial_xhrs.json").write_text(
            json.dumps(initial_xhrs, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )

        # Hash walk: try clicking each unique hash anchor and see what happens
        hash_walk: list[dict] = []
        hash_anchors = [c for c in clickables if c.get("strategy") == "anchor-hash"
                        and c.get("href", "").startswith("#")
                        and c["href"] not in {"#", "#/"}]
        seen_hrefs = set()
        unique_anchors = []
        for c in hash_anchors:
            if c["href"] in seen_hrefs:
                continue
            seen_hrefs.add(c["href"])
            unique_anchors.append(c)

        print(f"Hash-walking {len(unique_anchors)} unique hash routes ...")
        for i, c in enumerate(unique_anchors[:30]):  # cap to avoid runaway
            href = c["href"]
            before_url = page.url
            xhrs_during: list[dict] = []
            tmp_handler = _xhr_log_factory(xhrs_during)
            page.on("response", tmp_handler)
            try:
                page.evaluate(f"() => {{ window.location.hash = {json.dumps(href.lstrip('#'))}; }}")
                page.wait_for_load_state("networkidle", timeout=15_000)
            except Exception as e:  # noqa: BLE001
                hash_walk.append({"href": href, "text": c.get("text"),
                                  "error": str(e), "xhrs": []})
                page.remove_listener("response", tmp_handler)
                continue
            time.sleep(1.2)
            page.remove_listener("response", tmp_handler)
            hash_walk.append({
                "href": href,
                "text": c.get("text"),
                "url_after": page.url,
                "xhrs": xhrs_during,
                "xhr_count": len(xhrs_during),
                "json_xhr_count": sum(1 for x in xhrs_during if x["is_json"]),
            })
            print(f"  [{i+1}/{len(unique_anchors)}] {href}  → "
                  f"{len(xhrs_during)} XHRs ({hash_walk[-1]['json_xhr_count']} JSON)")

        (OUT_DIR / "hash_walk.json").write_text(
            json.dumps(hash_walk, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )

        browser.close()

    print("\n=== Summary ===")
    print(f"  initial XHRs (JSON / total): "
          f"{sum(1 for x in initial_xhrs if x['is_json'])} / {len(initial_xhrs)}")
    print(f"  hash routes walked: {len(hash_walk)}")
    print(f"  Files written under: {OUT_DIR}")
    print("\nNext: paste me the contents (or summary) of clickables.json and "
          "hash_walk.json so I can write the real auto-clicker.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
