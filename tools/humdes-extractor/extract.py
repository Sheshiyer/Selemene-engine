"""End-to-end extractor: list every saved reading on your humdes.com profile
and dump the raw JSON for each to ./output/YYYY-MM-DD/.

Pre-reqs:
  1. Be logged in to humdes.com in Chrome.
  2. QUIT Chrome (so the cookie DB isn't write-locked).
  3. Once: capture ~/Downloads/humdes.har and run `python har_inspector.py` to
     identify the endpoints; copy them into ENDPOINTS in humdes_client.py.
  4. `pip install -r requirements.txt`.
  5. `python extract.py`.
"""

from __future__ import annotations

import json
import re
import sys
from datetime import date
from pathlib import Path

from humdes_client import AuthError, EndpointNotConfigured, HumdesClient


OUTPUT_ROOT = Path(__file__).resolve().parent / "output"


def _slug(text: str, max_len: int = 60) -> str:
    text = re.sub(r"[^\w\s-]", "", text, flags=re.UNICODE).strip().lower()
    text = re.sub(r"[-\s]+", "-", text)
    return text[:max_len] or "reading"


def _reading_label(reading: dict) -> str:
    for key in ("name", "title", "label", "fio", "full_name"):
        val = reading.get(key)
        if val:
            return str(val)
    return f"reading_{reading.get('id', 'unknown')}"


def _reading_id(reading: dict) -> str:
    for key in ("id", "ID", "reading_id", "uid"):
        if key in reading and reading[key] is not None:
            return str(reading[key])
    raise KeyError(f"No id field on reading: {list(reading.keys())}")


def main() -> int:
    today_dir = OUTPUT_ROOT / date.today().isoformat()
    today_dir.mkdir(parents=True, exist_ok=True)

    print(f"Output directory: {today_dir}")
    client = HumdesClient()

    try:
        client.warm_up()
    except AuthError as e:
        print(f"\n[FATAL] Authentication failed: {e}", file=sys.stderr)
        print("Fix: log in to humdes.com in Chrome, quit Chrome, re-run.", file=sys.stderr)
        return 2

    print(f"Auth OK. Session probe: {json.dumps(client.probe(), indent=2)}")

    try:
        readings = client.list_readings()
    except EndpointNotConfigured as e:
        print(f"\n[BLOCKED] {e}", file=sys.stderr)
        return 3

    print(f"\nFound {len(readings)} readings. Fetching each...")

    manifest: list[dict] = []
    for idx, reading in enumerate(readings, start=1):
        try:
            rid = _reading_id(reading)
            label = _reading_label(reading)
        except KeyError as e:
            print(f"  [{idx}] skipped (cannot identify): {e}")
            continue

        try:
            data = client.get_reading(rid)
        except Exception as e:  # noqa: BLE001 - keep going
            print(f"  [{idx}] {rid} ({label!r}): FAILED — {e}")
            manifest.append({"id": rid, "label": label, "status": "error", "error": str(e)})
            continue

        out_path = today_dir / f"{rid}_{_slug(label)}.json"
        out_path.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")
        size = out_path.stat().st_size
        print(f"  [{idx}] {rid} ({label!r}) → {out_path.name} ({size} bytes)")
        manifest.append({"id": rid, "label": label, "status": "ok", "file": out_path.name, "size": size})

    manifest_path = today_dir / "_manifest.json"
    manifest_path.write_text(
        json.dumps({"date": date.today().isoformat(), "readings": manifest}, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )
    print(f"\nManifest: {manifest_path}")
    ok = sum(1 for m in manifest if m["status"] == "ok")
    print(f"Done. {ok}/{len(manifest)} readings exported successfully.")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
