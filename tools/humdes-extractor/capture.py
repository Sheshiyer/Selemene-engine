"""Capture every humdes.com JSON response as you navigate the results SPA.

Loads the authenticated state saved by login.py, opens a real Chromium window
to the personal-results page, and records the body of every XHR/fetch response
served by humdes.com whose content-type is JSON. Each response is saved to
./output/<YYYY-MM-DD>/<sequence>_<sanitised-path>.json and a _manifest.json
indexes them all.

What you do while it's running:
  - Click through every reading / chart you want exported.
  - Switch tabs/sections within the SPA so all data-fetching XHRs fire.
  - When done, return to the terminal and press Enter; the browser closes
    and the manifest is finalised.

Usage:
    python capture.py
"""

from __future__ import annotations

import json
import re
import sys
from datetime import datetime
from pathlib import Path
from urllib.parse import urlparse

from playwright.sync_api import Response, sync_playwright


STATE_FILE = Path(__file__).resolve().parent / "storageState.json"
OUTPUT_ROOT = Path(__file__).resolve().parent / "output"
START_URL = "https://www.humdes.com/en/personal/results/#/personal"

# What we consider "interesting" — JSON-shaped responses from humdes.com
# and its subdomains.
TARGET_HOST_PATTERN = re.compile(r"(^|\.)humdes\.com$", re.IGNORECASE)

# Resource types we never want to save (assets, not data).
SKIP_RESOURCE_TYPES = {"image", "stylesheet", "font", "media", "manifest", "websocket"}

# Path tokens that indicate static assets even on non-static resource_type.
SKIP_PATH_FRAGMENTS = (
    ".js", ".css", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp",
    ".woff", ".woff2", ".ttf", ".ico", ".map",
)


def _slug(text: str, max_len: int = 80) -> str:
    text = text.strip("/").replace("/", "_")
    text = re.sub(r"[^\w.\-]", "_", text)
    text = re.sub(r"_+", "_", text).strip("_")
    return text[:max_len] or "root"


def _classify(resp: Response) -> tuple[str, str]:
    """Return (verdict, reason). verdict in {save, skip, log}."""
    try:
        host = urlparse(resp.url).hostname or ""
    except Exception:  # noqa: BLE001
        return "skip", "bad-url"
    if not TARGET_HOST_PATTERN.search(host):
        return "skip", "off-host"

    rtype = getattr(resp.request, "resource_type", "") or ""
    if rtype in SKIP_RESOURCE_TYPES:
        return "skip", f"asset-{rtype}"

    path_lower = urlparse(resp.url).path.lower()
    if any(path_lower.endswith(ext) for ext in SKIP_PATH_FRAGMENTS):
        return "skip", "asset-ext"

    ct = (resp.headers.get("content-type") or "").lower()
    if "json" in ct:
        return "save", "ct-json"
    if any(s in ct for s in ("text/plain", "text/html", "application/")):
        # Defer decision to content-sniff in the handler.
        return "log", f"ct-{ct.split(';')[0] or 'unknown'}"
    return "log", f"ct-{ct or 'unknown'}"


def main() -> int:
    if not STATE_FILE.exists():
        print(f"ERROR: {STATE_FILE} not found. Run `python login.py` first.", file=sys.stderr)
        return 2

    run_dir = OUTPUT_ROOT / datetime.now().strftime("%Y-%m-%d_%H%M%S")
    run_dir.mkdir(parents=True, exist_ok=True)
    print(f"Output directory: {run_dir}")

    manifest: list[dict] = []
    seq = 0

    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=False)
        context = browser.new_context(
            storage_state=str(STATE_FILE),
            viewport={"width": 1400, "height": 900},
            locale="en-US",
        )
        page = context.new_page()

        seen_paths: set[tuple[str, str]] = set()
        non_json_log_path = run_dir / "_non_json_humdes.log"
        non_json_log = non_json_log_path.open("w", encoding="utf-8")

        def on_response(resp: Response) -> None:
            nonlocal seq
            verdict, reason = _classify(resp)
            if verdict == "skip":
                return

            parsed_url = urlparse(resp.url)
            path = parsed_url.path
            key = (resp.request.method, path + ("?" + parsed_url.query if parsed_url.query else ""))

            # Show all humdes activity so user can see things are happening.
            if key not in seen_paths:
                seen_paths.add(key)
                marker = "📥" if verdict == "save" else "  "
                print(f"  {marker} {resp.status} {resp.request.method} {path}  [{reason}]")

            try:
                body = resp.body()
            except Exception as e:  # noqa: BLE001
                print(f"     (could not read body: {e})")
                return

            # Try to parse as JSON regardless of content-type — Bitrix is sloppy.
            parsed = None
            try:
                parsed = json.loads(body)
            except (json.JSONDecodeError, UnicodeDecodeError):
                pass

            if parsed is None:
                # Not JSON. Log a one-line preview to the side-channel file
                # so we can later see if the data we want lives in HTML.
                preview = body[:200].decode("utf-8", errors="replace").replace("\n", " ")
                non_json_log.write(
                    f"{resp.status}\t{resp.request.method}\t{resp.url}\t{preview!r}\n"
                )
                non_json_log.flush()
                return

            seq += 1
            fname = f"{seq:03d}_{resp.request.method}_{_slug(path)}.json"
            out_path = run_dir / fname

            envelope = {
                "_meta": {
                    "url": resp.url,
                    "method": resp.request.method,
                    "status": resp.status,
                    "request_headers": {
                        k.lower(): v
                        for k, v in resp.request.headers.items()
                        if k.lower() in {"content-type", "x-requested-with",
                                         "x-bitrix-csrf-token", "referer"}
                    },
                    "post_data": resp.request.post_data,
                    "captured_at": datetime.now().isoformat(timespec="seconds"),
                },
                "data": parsed,
            }
            out_path.write_text(
                json.dumps(envelope, indent=2, ensure_ascii=False),
                encoding="utf-8",
            )
            manifest.append({
                "seq": seq,
                "file": fname,
                "url": resp.url,
                "method": resp.request.method,
                "status": resp.status,
                "size": out_path.stat().st_size,
            })
            print(f"     ↳ saved as {fname}  ({out_path.stat().st_size} bytes)")

        page.on("response", on_response)

        page.goto(START_URL, wait_until="domcontentloaded")
        print("\n" + "=" * 70)
        print("Browser is open. Watch this terminal AS you click — every humdes.com")
        print("request shows up here:")
        print("   📥 = saved as JSON         (no marker) = logged, non-JSON")
        print("\nWhat to do:")
        print("  - Open every saved reading you want exported")
        print("  - Switch between any chart tabs / sub-sections within a reading")
        print("  - Trigger anything you'd normally use (export, print, etc.)")
        print("\nIf you see lots of (no marker) lines but no 📥, the data is server-")
        print("rendered as HTML — tell me and I'll add an HTML-scrape path.")
        print("=" * 70 + "\n")
        try:
            input("When done, press Enter here to finalise and close the browser... ")
        except (KeyboardInterrupt, EOFError):
            print("\nInterrupted; saving what we have so far.")

        non_json_log.close()
        browser.close()

    manifest_path = run_dir / "_manifest.json"
    manifest_path.write_text(
        json.dumps(
            {"start_url": START_URL, "captured": len(manifest), "responses": manifest},
            indent=2,
            ensure_ascii=False,
        ),
        encoding="utf-8",
    )
    print(f"\nCaptured {len(manifest)} JSON responses.")
    print(f"Manifest: {manifest_path}")
    print(f"Non-JSON humdes activity log: {non_json_log_path}")
    if not manifest and non_json_log_path.stat().st_size > 0:
        print("\nNo JSON was captured, but humdes did serve some non-JSON responses.")
        print("Inspect the log file above — chart data may live inside HTML.")
    return 0 if manifest else 1


if __name__ == "__main__":
    sys.exit(main())
