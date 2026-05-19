"""Parse a HAR file captured from humdes.com and surface candidate API endpoints.

Run this once after capturing `~/Downloads/humdes.har` from DevTools.
It prints the unique XHR/Fetch requests that returned JSON, grouped by path,
so we can identify the 'list readings' and 'get chart data' endpoints.

Usage:
    python har_inspector.py ~/Downloads/humdes.har
"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path
from urllib.parse import urlparse


def _is_json_resource(entry: dict) -> bool:
    mime = (entry.get("response", {}).get("content", {}) or {}).get("mimeType", "")
    return "json" in mime.lower()


def _short(value: str, limit: int = 80) -> str:
    if len(value) <= limit:
        return value
    return value[: limit - 3] + "..."


def inspect(har_path: Path) -> None:
    if not har_path.exists():
        print(f"HAR file not found: {har_path}", file=sys.stderr)
        sys.exit(1)

    with har_path.open("r", encoding="utf-8") as fh:
        har = json.load(fh)

    entries = har.get("log", {}).get("entries", [])
    print(f"Loaded {len(entries)} entries from {har_path}\n")

    by_path: dict[tuple[str, str, str], list[dict]] = defaultdict(list)
    for entry in entries:
        req = entry.get("request", {})
        resp = entry.get("response", {})
        url = req.get("url", "")
        if "humdes.com" not in url:
            continue
        if not _is_json_resource(entry):
            continue
        parsed = urlparse(url)
        key = (req.get("method", "?"), parsed.netloc, parsed.path)
        by_path[key].append({
            "url": url,
            "query": parsed.query,
            "status": resp.get("status"),
            "size": resp.get("content", {}).get("size", 0),
            "request_body": (req.get("postData", {}) or {}).get("text", ""),
            "headers": {h["name"]: h["value"] for h in req.get("headers", [])},
        })

    if not by_path:
        print("No JSON XHR/Fetch responses found for humdes.com in this HAR.")
        print("Make sure DevTools 'Preserve log' was on and you clicked through readings.")
        return

    print(f"Found {len(by_path)} unique humdes.com JSON endpoints:\n")
    print("=" * 100)

    for (method, host, path), calls in sorted(by_path.items(), key=lambda x: -len(x[1])):
        print(f"\n{method}  {host}{path}")
        print(f"   called {len(calls)} time(s)")
        sample = calls[0]
        print(f"   sample status: {sample['status']}, response size: {sample['size']} bytes")
        if sample["query"]:
            print(f"   sample query : {_short(sample['query'])}")
        if sample["request_body"]:
            print(f"   sample body  : {_short(sample['request_body'])}")
        interesting_headers = {
            k: v
            for k, v in sample["headers"].items()
            if k.lower() in {"x-bitrix-csrf-token", "x-requested-with", "content-type",
                             "x-csrf-token", "authorization", "x-bx-sessid"}
        }
        if interesting_headers:
            print(f"   key headers  : {interesting_headers}")

    print("\n" + "=" * 100)
    print("\nLook for endpoints whose paths suggest:")
    print("  - 'list/readings/charts'  (returns an ARRAY of readings)")
    print("  - 'reading/chart/data'    (returns ONE chart's data, called per reading)")
    print("\nNote the exact path, method, query/body shape, and any CSRF header.")
    print("Then we can hard-code them in humdes_client.py.")


if __name__ == "__main__":
    default = Path.home() / "Downloads" / "humdes.har"
    har_arg = Path(sys.argv[1]) if len(sys.argv) > 1 else default
    inspect(har_arg)
