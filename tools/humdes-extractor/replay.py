"""Headless replay of a previous capture.py manifest.

Reads the `_manifest.json` from a prior capture run, then re-issues every
recorded request using Playwright's APIRequestContext (which inherits the
storageState.json session). Saves fresh responses to a new dated folder.

Run this on a schedule for snapshot history without ever opening a browser.

Usage:
    python replay.py                 # replay the most recent capture folder
    python replay.py output/2026-05-16_103900   # replay a specific folder
"""

from __future__ import annotations

import json
import sys
from datetime import datetime
from pathlib import Path

from playwright.sync_api import sync_playwright


ROOT = Path(__file__).resolve().parent
STATE_FILE = ROOT / "storageState.json"
OUTPUT_ROOT = ROOT / "output"

DEFAULT_HEADERS = {
    "Accept": "application/json, text/plain, */*",
    "Accept-Language": "en-US,en;q=0.9",
    "X-Requested-With": "XMLHttpRequest",
    "Referer": "https://www.humdes.com/en/personal/results/",
}


def _find_latest_capture() -> Path:
    candidates = [
        p for p in OUTPUT_ROOT.iterdir()
        if p.is_dir() and (p / "_manifest.json").exists()
    ]
    if not candidates:
        raise FileNotFoundError(
            "No prior capture run with a _manifest.json under ./output/. "
            "Run `python capture.py` at least once first."
        )
    return max(candidates, key=lambda p: p.stat().st_mtime)


def _load_manifest(folder: Path) -> list[dict]:
    manifest_path = folder / "_manifest.json"
    if not manifest_path.exists():
        raise FileNotFoundError(f"No _manifest.json in {folder}")
    data = json.loads(manifest_path.read_text(encoding="utf-8"))
    return data.get("responses", [])


def _file_for(entry: dict, run_dir: Path) -> Path:
    # Mirror capture.py's file naming so diffing between runs is easy.
    return run_dir / entry["file"]


def main() -> int:
    if not STATE_FILE.exists():
        print(f"ERROR: {STATE_FILE} not found. Run `python login.py` first.", file=sys.stderr)
        return 2

    if len(sys.argv) > 1:
        source = Path(sys.argv[1]).resolve()
        if not source.is_absolute():
            source = ROOT / sys.argv[1]
    else:
        source = _find_latest_capture()

    print(f"Source capture : {source}")
    entries = _load_manifest(source)
    if not entries:
        print("Source manifest has no entries; nothing to replay.", file=sys.stderr)
        return 1
    print(f"Entries to replay: {len(entries)}")

    run_dir = OUTPUT_ROOT / datetime.now().strftime("%Y-%m-%d_%H%M%S_replay")
    run_dir.mkdir(parents=True, exist_ok=True)
    print(f"Output         : {run_dir}\n")

    out_manifest: list[dict] = []
    ok = 0
    skipped = 0
    failed = 0

    with sync_playwright() as pw:
        request_ctx = pw.request.new_context(
            storage_state=str(STATE_FILE),
            extra_http_headers=DEFAULT_HEADERS,
        )

        for entry in entries:
            url = entry["url"]
            method = entry.get("method", "GET").upper()

            # We need the source file to recover post_data + request headers.
            source_file = source / entry["file"]
            try:
                src = json.loads(source_file.read_text(encoding="utf-8"))
                meta = src.get("_meta", {})
                post_data = meta.get("post_data")
                hdrs = dict(DEFAULT_HEADERS)
                # Carry over any captured request headers (content-type, csrf, etc.)
                for k, v in (meta.get("request_headers") or {}).items():
                    hdrs[k] = v
            except Exception as e:  # noqa: BLE001
                print(f"  [{entry['seq']:03d}] skip — cannot read source: {e}")
                skipped += 1
                continue

            try:
                kwargs: dict = {"headers": hdrs}
                if post_data is not None and method != "GET":
                    # post_data may be JSON or form-encoded; pass raw.
                    kwargs["data"] = post_data
                resp = request_ctx.fetch(url, method=method, **kwargs)
            except Exception as e:  # noqa: BLE001
                print(f"  [{entry['seq']:03d}] FAIL — {method} {url}  ({e})")
                failed += 1
                out_manifest.append({**entry, "status": None, "error": str(e)})
                continue

            ct = resp.headers.get("content-type", "")
            body_bytes = resp.body()
            parsed = None
            if "json" in ct.lower():
                try:
                    parsed = json.loads(body_bytes)
                except json.JSONDecodeError:
                    pass

            out_file = _file_for(entry, run_dir)
            if parsed is None:
                # Non-JSON or parse failure — save raw with .raw suffix.
                raw_path = out_file.with_suffix(".raw")
                raw_path.write_bytes(body_bytes)
                print(f"  [{entry['seq']:03d}] {resp.status} {method} {url}  "
                      f"(non-JSON, saved as {raw_path.name})")
                failed += 1
                out_manifest.append({**entry, "status": resp.status,
                                     "file": raw_path.name, "note": "non-JSON"})
                continue

            envelope = {
                "_meta": {
                    "url": url,
                    "method": method,
                    "status": resp.status,
                    "captured_at": datetime.now().isoformat(timespec="seconds"),
                    "replay_of": str(source.name),
                },
                "data": parsed,
            }
            out_file.write_text(json.dumps(envelope, indent=2, ensure_ascii=False),
                                encoding="utf-8")
            print(f"  [{entry['seq']:03d}] {resp.status} {method} {url}  → {out_file.name}")
            ok += 1
            out_manifest.append({**entry, "status": resp.status,
                                 "size": out_file.stat().st_size})

        request_ctx.dispose()

    (run_dir / "_manifest.json").write_text(
        json.dumps(
            {"replayed_from": str(source.name), "ok": ok, "failed": failed,
             "skipped": skipped, "responses": out_manifest},
            indent=2, ensure_ascii=False,
        ),
        encoding="utf-8",
    )
    print(f"\nDone. {ok} ok, {failed} failed, {skipped} skipped.")
    if ok == 0:
        print("Hint: if everything failed, your session may have expired — re-run login.py.",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
