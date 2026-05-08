#!/usr/bin/env python3
"""check_doc_coverage.py — Flag API endpoints that lack documentation.

Reads the live OpenAPI spec and checks whether each operation has a
corresponding entry in docs/api/. Reports missing coverage.

Usage:
    python scripts/check_doc_coverage.py [--url URL] [--docs-dir DIR]

Exits with 0 if all operations are documented, 1 if gaps found.
"""
import argparse
import json
import os
import re
import sys
import urllib.request
from pathlib import Path

LIVE_URL = os.environ.get(
    "NOESIS_OPENAPI_URL", "https://selemene.tryambakam.space/api/openapi.json"
)
DOCS_DIR = Path(__file__).parent.parent / "docs" / "api"

# Doc files that count as coverage (each covers a topic area, not one-per-endpoint)
COVERAGE_DOC_FILES = [
    "README.md",
    "authentication.md",
    "engines.md",
    "workflows.md",
    "billing.md",
    "admin-analytics.md",
    "admin-reconcile.md",
    "OPENCLAW_INTEGRATION.md",
    "HERMES_INTEGRATION.md",
    "MCP_INTEGRATION.md",
    "TOI_INTEGRATION.md",
    "TUI_INTEGRATION.md",
    "LLM_AGENT_GUIDE.md",
]

# Tags that must have at least one doc file covering them
REQUIRED_TAG_COVERAGE: dict[str, list[str]] = {
    "auth":       ["authentication.md"],
    "engines":    ["engines.md"],
    "workflows":  ["workflows.md"],
    "billing":    ["billing.md"],
    "admin":      ["admin-analytics.md", "admin-reconcile.md"],
    "users":      ["authentication.md"],
}


def fetch_spec(url: str) -> dict:
    req = urllib.request.Request(url, headers={"User-Agent": "noesis-doc-ci/1.0"})
    with urllib.request.urlopen(req, timeout=15) as resp:  # noqa: S310
        return json.loads(resp.read())


def extract_operations(spec: dict) -> list[dict]:
    ops = []
    for path_str, path_item in spec.get("paths", {}).items():
        for method, operation in path_item.items():
            if method in ("get", "post", "put", "patch", "delete"):
                ops.append({
                    "operationId": operation.get("operationId", f"{method}:{path_str}"),
                    "method": method.upper(),
                    "path": path_str,
                    "tags": operation.get("tags", ["untagged"]),
                    "summary": operation.get("summary", ""),
                })
    return ops


def doc_covers_tag(docs_dir: Path, tag: str, required_files: list[str]) -> list[str]:
    """Return list of missing doc files for a tag."""
    missing = []
    for filename in required_files:
        if not (docs_dir / filename).exists():
            missing.append(filename)
    return missing


def doc_mentions_operation(docs_dir: Path, operation_id: str) -> bool:
    """Check if any doc file mentions the operationId or the path fragment."""
    fragment = operation_id.replace("_", "-").lower()
    for doc_file in docs_dir.glob("*.md"):
        content = doc_file.read_text(encoding="utf-8", errors="ignore").lower()
        if fragment in content or operation_id.lower() in content:
            return True
    return False


def main() -> int:
    parser = argparse.ArgumentParser(description="Check Noesis doc coverage for API endpoints")
    parser.add_argument("--url", default=LIVE_URL, help="Live OpenAPI spec URL")
    parser.add_argument("--docs-dir", default=str(DOCS_DIR), help="Path to docs/api/ directory")
    parser.add_argument("--strict", action="store_true",
                        help="Require each operationId to appear in at least one doc")
    args = parser.parse_args()

    docs_dir = Path(args.docs_dir)
    if not docs_dir.exists():
        print(f"ERROR: docs dir not found: {docs_dir}", file=sys.stderr)
        return 1

    print(f"Fetching live spec from {args.url} …", flush=True)
    try:
        spec = fetch_spec(args.url)
    except Exception as e:
        print(f"ERROR: Could not fetch live spec: {e}", file=sys.stderr)
        return 1

    operations = extract_operations(spec)
    gaps: list[str] = []

    # Tag-level coverage check
    print(f"\nChecking tag coverage ({len(REQUIRED_TAG_COVERAGE)} required tags):")
    for tag, required_files in REQUIRED_TAG_COVERAGE.items():
        missing_files = doc_covers_tag(docs_dir, tag, required_files)
        if missing_files:
            msg = f"  🔴 tag '{tag}': missing doc files: {', '.join(missing_files)}"
            print(msg)
            gaps.append(msg)
        else:
            print(f"  ✅ tag '{tag}': covered")

    # Strict: per-operation coverage
    if args.strict:
        print(f"\nStrict per-operation check ({len(operations)} operations):")
        undocumented = []
        for op in operations:
            if not doc_mentions_operation(docs_dir, op["operationId"]):
                undocumented.append(op)
        if undocumented:
            print(f"  🔴 {len(undocumented)} operations not mentioned in any doc:")
            for op in undocumented:
                msg = f"      {op['method']:7} {op['path']}  [{op['operationId']}]"
                print(msg)
                gaps.append(msg)
        else:
            print(f"  ✅ All {len(operations)} operations mentioned in docs")
    else:
        print(f"\n📊 Total operations: {len(operations)} (run --strict for per-operation check)")

    # Summary
    existing_docs = sorted([f.name for f in docs_dir.glob("*.md")])
    print(f"\nDocs present in {docs_dir.name}/: {len(existing_docs)}")
    for f in existing_docs:
        print(f"   {f}")

    if gaps:
        print(f"\n⚠️  {len(gaps)} doc coverage gap(s) found.")
        return 1

    print("\n✅  Doc coverage check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
