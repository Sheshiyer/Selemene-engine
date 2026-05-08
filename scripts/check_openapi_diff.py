#!/usr/bin/env python3
"""check_openapi_diff.py — Compare live OpenAPI spec against a cached baseline.

Usage:
    python scripts/check_openapi_diff.py [--baseline FILE] [--url URL] [--update]

Exits with code 0 if no meaningful changes, 1 if additions/removals/changes found.
Used by the doc-currency GitHub Action (.github/workflows/doc-currency.yml).
"""
import argparse
import json
import os
import sys
import urllib.request
from pathlib import Path

LIVE_URL = os.environ.get(
    "NOESIS_OPENAPI_URL", "https://selemene.tryambakam.space/api/openapi.json"
)
DEFAULT_BASELINE = Path(__file__).parent.parent / ".planning" / "openapi-baseline.json"


def fetch_spec(url: str) -> dict:
    req = urllib.request.Request(url, headers={"User-Agent": "noesis-doc-ci/1.0"})
    with urllib.request.urlopen(req, timeout=15) as resp:  # noqa: S310
        return json.loads(resp.read())


def load_baseline(path: Path) -> dict | None:
    if path.exists():
        return json.loads(path.read_text())
    return None


def extract_operations(spec: dict) -> dict[str, dict]:
    """Return {operationId: {method, path, tags, summary}} for every operation."""
    ops = {}
    for path_str, path_item in spec.get("paths", {}).items():
        for method, operation in path_item.items():
            if method in ("get", "post", "put", "patch", "delete", "head", "options"):
                op_id = operation.get("operationId", f"{method}:{path_str}")
                ops[op_id] = {
                    "method": method.upper(),
                    "path": path_str,
                    "tags": operation.get("tags", []),
                    "summary": operation.get("summary", ""),
                }
    return ops


def diff_specs(baseline: dict, live: dict) -> dict:
    b_ops = extract_operations(baseline)
    l_ops = extract_operations(live)

    added = {k: v for k, v in l_ops.items() if k not in b_ops}
    removed = {k: v for k, v in b_ops.items() if k not in l_ops}
    changed = {}
    for k in b_ops:
        if k in l_ops and b_ops[k] != l_ops[k]:
            changed[k] = {"before": b_ops[k], "after": l_ops[k]}

    return {"added": added, "removed": removed, "changed": changed}


def main() -> int:
    parser = argparse.ArgumentParser(description="Diff Noesis OpenAPI spec vs baseline")
    parser.add_argument("--baseline", default=str(DEFAULT_BASELINE), help="Path to baseline JSON")
    parser.add_argument("--url", default=LIVE_URL, help="Live OpenAPI spec URL")
    parser.add_argument("--update", action="store_true", help="Update baseline to current live spec")
    args = parser.parse_args()

    baseline_path = Path(args.baseline)

    print(f"Fetching live spec from {args.url} …", flush=True)
    try:
        live = fetch_spec(args.url)
    except Exception as e:
        print(f"ERROR: Could not fetch live spec: {e}", file=sys.stderr)
        return 1

    if args.update:
        baseline_path.parent.mkdir(parents=True, exist_ok=True)
        baseline_path.write_text(json.dumps(live, indent=2))
        print(f"Baseline updated → {baseline_path}")
        return 0

    baseline = load_baseline(baseline_path)
    if baseline is None:
        print(f"No baseline at {baseline_path}. Run with --update to create one.")
        baseline_path.parent.mkdir(parents=True, exist_ok=True)
        baseline_path.write_text(json.dumps(live, indent=2))
        print("Baseline created from current live spec. Re-run without --update to diff.")
        return 0

    diff = diff_specs(baseline, live)
    has_changes = any(diff[k] for k in ("added", "removed", "changed"))

    if not has_changes:
        live_count = len(extract_operations(live))
        print(f"✅  No API changes detected ({live_count} operations).")
        return 0

    # Report changes
    if diff["added"]:
        print(f"\n🟢 ADDED ({len(diff['added'])}):")
        for op_id, meta in diff["added"].items():
            print(f"   {meta['method']:7} {meta['path']}  [{op_id}]")

    if diff["removed"]:
        print(f"\n🔴 REMOVED ({len(diff['removed'])}) — check docs for stale content:")
        for op_id, meta in diff["removed"].items():
            print(f"   {meta['method']:7} {meta['path']}  [{op_id}]")

    if diff["changed"]:
        print(f"\n🟡 CHANGED ({len(diff['changed'])}):")
        for op_id, delta in diff["changed"].items():
            print(f"   [{op_id}]: {delta['before']} → {delta['after']}")

    print(
        "\n⚠️  API drift detected. Update docs and run:\n"
        "   python scripts/check_openapi_diff.py --update"
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
