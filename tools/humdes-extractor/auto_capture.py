"""Fully automated browser-driven capture.

Drives the SPA the same way a user does (no manual clicking, no direct API
calls that get rejected by the server). All XHRs the SPA fires are intercepted
and saved.

Pipeline:
  1. Open browser with saved storageState
  2. Visit /en/personal/results/#/personal
  3. For each of 5 types: navigate via hash change, wait, capture directory XHR
  4. Return to first type, locate "reading" elements via robust heuristic,
     click each one in turn, intercept the per-reading XHRs, navigate back,
     repeat. Loop across all 5 types.
  5. Save everything to ./output/<timestamp>_auto/

Tunables at top of file: HEADLESS, MAX_PER_TYPE, NAV_SETTLE_SECS.

Usage:
    python auto_capture.py
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
HEADLESS = False          # set True once you trust it; False lets you watch
MAX_PER_TYPE = None       # None = all; integer caps each type for testing
NAV_SETTLE_SECS = 1.5     # delay after each navigation/click for SPA renders
CLICK_TIMEOUT_MS = 10_000
NAV_TIMEOUT_MS = 30_000
# -------------------------------------------------------------------------

ROOT = Path(__file__).resolve().parent
STATE_FILE = ROOT / "storageState.json"
OUTPUT_ROOT = ROOT / "output"
START_URL = "https://www.humdes.com/en/personal/results/#/personal"
TYPES = ["personal", "hologenetic", "compatibility", "business", "family"]

HOST_RE = re.compile(r"(^|\.)humdes\.com$", re.IGNORECASE)
SKIP_EXT = (".js", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp",
            ".woff", ".woff2", ".ttf", ".ico", ".map")


def _slug(text: str, max_len: int = 60) -> str:
    text = re.sub(r"[^\w\-]", "_", str(text or "x"))
    text = re.sub(r"_+", "_", text).strip("_")
    return text[:max_len] or "x"


def _parse_loose(body: bytes) -> object | None:
    try:
        return json.loads(body)
    except Exception:  # noqa: BLE001
        return None


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


# JS helper: find clickable "reading"-like elements in the main content area,
# excluding chrome (nav, header, footer, sidebars). Returns a stable selector
# (text + role) per candidate so Playwright can re-locate them after re-renders.
FIND_CARDS_JS = r"""
() => {
    // Try to identify the SPA root (where readings render).
    const roots = [
        document.querySelector('#app'),
        document.querySelector('main'),
        document.querySelector('[class*="results"]'),
        document.querySelector('[class*="content"]'),
        document.body,
    ].filter(Boolean);
    const root = roots[0];

    // All candidates: things you can click.
    const all = Array.from(root.querySelectorAll(
        'a, button, [role="button"], [role="link"], li[class], .card, ' +
        '[class*="result"], [class*="reading"], [class*="profile"], ' +
        '[class*="item"], [class*="card"]'
    ));

    const isInChrome = (el) => !!el.closest(
        'nav, header, footer, .header, .footer, .navigation, .menu, ' +
        '.tabbar, .breadcrumb, .header-menu, .footer-menu'
    );

    const out = [];
    const seen = new Set();
    for (const el of all) {
        if (isInChrome(el)) continue;
        const text = (el.innerText || el.textContent || '').trim();
        if (!text) continue;
        if (text.length < 2 || text.length > 200) continue;
        // Skip the type tabs themselves
        if (/^(Personal|Hologonetics|Compatibility|Business Penta|Family Penta)\s*\d+$/i.test(text)) continue;
        // Dedupe by text within the same root
        if (seen.has(text)) continue;
        seen.add(text);

        const rect = el.getBoundingClientRect();
        if (rect.width < 30 || rect.height < 20) continue;  // not visible

        out.push({
            text: text.slice(0, 200),
            tag: el.tagName.toLowerCase(),
            classes: (el.className || '').toString().slice(0, 200),
            href: el.getAttribute('href') || '',
            dataId: el.getAttribute('data-id') || el.dataset?.id || '',
            box: {x: rect.x, y: rect.y, w: rect.width, h: rect.height},
        });
    }
    return out;
}
"""


def main() -> int:
    if not STATE_FILE.exists():
        print(f"ERROR: {STATE_FILE} not found. Run `python login.py` first.", file=sys.stderr)
        return 2

    run_dir = OUTPUT_ROOT / (datetime.now().strftime("%Y-%m-%d_%H%M%S") + "_auto")
    raw_dir = run_dir / "raw"
    raw_dir.mkdir(parents=True, exist_ok=True)
    print(f"Output: {run_dir}\n")

    # Buffer of every intercepted XHR (we'll save them organised at the end).
    captured: list[dict] = []
    seq = 0
    seq_lock = [0]  # box for closure

    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=HEADLESS)
        context = browser.new_context(
            storage_state=str(STATE_FILE),
            viewport={"width": 1400, "height": 900},
            locale="en-US",
        )
        page = context.new_page()

        def on_response(resp: Response) -> None:
            if not _is_target_xhr(resp):
                return
            try:
                body = resp.body()
            except Exception:  # noqa: BLE001
                return
            parsed = _parse_loose(body)
            if parsed is None:
                return  # we only want JSON-shaped data

            seq_lock[0] += 1
            this_seq = seq_lock[0]
            parsed_url = urlparse(resp.url)
            fname = f"{this_seq:04d}_{resp.request.method}_{_slug(parsed_url.path + '_' + parsed_url.query)}.json"
            out_path = raw_dir / fname

            envelope = {
                "_meta": {
                    "url": resp.url,
                    "method": resp.request.method,
                    "status": resp.status,
                    "captured_at": datetime.now().isoformat(timespec="seconds"),
                    "post_data": resp.request.post_data,
                },
                "data": parsed,
            }
            out_path.write_text(json.dumps(envelope, indent=2, ensure_ascii=False),
                                encoding="utf-8")
            captured.append({
                "seq": this_seq,
                "url": resp.url,
                "method": resp.request.method,
                "status": resp.status,
                "file": fname,
                "size": out_path.stat().st_size,
            })
            print(f"    📥 {this_seq:04d} {resp.request.method} "
                  f"{parsed_url.path}?{parsed_url.query[:60]}  → {fname}")

        page.on("response", on_response)

        # ============================================================
        # PHASE 1 — visit each of the 5 type hashes, collect directories
        # ============================================================
        print("PHASE 1 — collect 5 type directories")
        page.goto(START_URL, wait_until="domcontentloaded", timeout=NAV_TIMEOUT_MS)
        page.wait_for_load_state("networkidle", timeout=NAV_TIMEOUT_MS)
        time.sleep(NAV_SETTLE_SECS)

        for t in TYPES:
            print(f"  -> #/{t}")
            page.evaluate("(h) => { window.location.hash = h; }", f"#/{t}")
            try:
                page.wait_for_load_state("networkidle", timeout=NAV_TIMEOUT_MS)
            except Exception:  # noqa: BLE001
                pass
            time.sleep(NAV_SETTLE_SECS)

        # ============================================================
        # PHASE 2 — for each type, find cards in the rendered DOM and click each
        # ============================================================
        print(f"\nPHASE 2 — drive UI to load every reading per type")

        per_type_summary: dict[str, int] = {}

        for t in TYPES:
            print(f"\n  TYPE: {t}")
            # Navigate to type
            page.evaluate("(h) => { window.location.hash = h; }", f"#/{t}")
            try:
                page.wait_for_load_state("networkidle", timeout=NAV_TIMEOUT_MS)
            except Exception:  # noqa: BLE001
                pass
            time.sleep(NAV_SETTLE_SECS)

            # Snapshot DOM for diagnostics
            (run_dir / f"dom_{t}.html").write_text(page.content(), encoding="utf-8")

            cards = page.evaluate(FIND_CARDS_JS)
            print(f"    found {len(cards)} clickable card candidates")

            if MAX_PER_TYPE:
                cards = cards[:MAX_PER_TYPE]

            (run_dir / f"cards_{t}.json").write_text(
                json.dumps(cards, indent=2, ensure_ascii=False), encoding="utf-8"
            )

            clicked = 0
            for i, c in enumerate(cards, start=1):
                text = c.get("text", "")
                short = text.replace("\n", " | ")[:60]
                # Re-locate the element by text (re-renders shift DOM references)
                try:
                    locator = page.get_by_text(text[:80], exact=False).first
                    locator.scroll_into_view_if_needed(timeout=3000)
                    locator.click(timeout=CLICK_TIMEOUT_MS)
                except Exception as e:  # noqa: BLE001
                    print(f"    [{i:3d}] CLICK FAIL  {short!r:65s}  ({e.__class__.__name__})")
                    continue

                try:
                    page.wait_for_load_state("networkidle", timeout=NAV_TIMEOUT_MS)
                except Exception:  # noqa: BLE001
                    pass
                time.sleep(NAV_SETTLE_SECS)
                clicked += 1
                print(f"    [{i:3d}] ✓ clicked     {short!r}")

                # Navigate back to the type listing for the next card
                page.evaluate("(h) => { window.location.hash = h; }", f"#/{t}")
                try:
                    page.wait_for_load_state("networkidle", timeout=NAV_TIMEOUT_MS)
                except Exception:  # noqa: BLE001
                    pass
                time.sleep(NAV_SETTLE_SECS)

            per_type_summary[t] = clicked

        browser.close()

    # ============================================================
    # Manifest
    # ============================================================
    manifest = {
        "started_url": START_URL,
        "captured": len(captured),
        "per_type_clicked": per_type_summary,
        "responses": captured,
    }
    (run_dir / "_manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8"
    )

    print("\n=== Done ===")
    print(f"  XHRs captured (JSON): {len(captured)}")
    for t, n in per_type_summary.items():
        print(f"  clicks on {t:15s}: {n}")
    print(f"  Output: {run_dir}")
    print(f"  Manifest: {run_dir / '_manifest.json'}")
    return 0 if captured else 1


if __name__ == "__main__":
    sys.exit(main())
