#!/usr/bin/env python3
"""patch_docs.py — Surgical doc insert for Noesis releases.

Applies structured doc patches from a YAML patch file without rewriting
entire documentation files. Designed for use after every release.

Usage:
    python scripts/patch_docs.py --patch-file scripts/release-patches/v3.4.0.yaml [--dry-run]

Patch file format (YAML):
    version: "3.4.0"
    patches:
      - file: docs/api/engines.md
        after: "## Engines (16)"
        insert: |
          ### new-engine — Description here
          `POST /api/v1/engines/new-engine/calculate`

      - file: llms.txt
        after: "- [Transits]"
        insert: "- [New Engine](https://selemene.tryambakam.space/api/v1/engines/new-engine/info): New engine description"

      - file: docs/api/workflows.md
        replace_section: "## Workflows (6)"
        new_content: "## Workflows (7)"
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

try:
    import yaml  # type: ignore
except ImportError:
    print("pyyaml not installed. Run: pip install pyyaml", file=sys.stderr)
    sys.exit(1)


def apply_after(content: str, after: str, insert: str) -> tuple[str, bool]:
    """Insert text immediately after the first occurrence of `after`."""
    idx = content.find(after)
    if idx == -1:
        return content, False
    end = idx + len(after)
    # Insert after the anchor, preserve trailing newline
    if not insert.endswith("\n"):
        insert += "\n"
    new = content[:end] + "\n" + insert + content[end:]
    return new, True


def apply_replace_section(content: str, old_heading: str, new_heading: str) -> tuple[str, bool]:
    """Replace a section heading (first occurrence only)."""
    if old_heading not in content:
        return content, False
    return content.replace(old_heading, new_heading, 1), True


def apply_patch(patch: dict, root: Path, dry_run: bool) -> bool:
    """Apply a single patch operation. Returns True on success."""
    rel_path = patch.get("file")
    if not rel_path:
        print(f"  SKIP: patch missing 'file' key: {patch}")
        return False

    target = root / rel_path
    if not target.exists():
        print(f"  ❌ File not found: {target}")
        return False

    content = target.read_text(encoding="utf-8")
    changed = False

    if "after" in patch and "insert" in patch:
        content, changed = apply_after(content, patch["after"], patch["insert"])
        if not changed:
            print(f"  ⚠️  Anchor not found in {rel_path}: '{patch['after'][:60]}'")
            return False

    elif "replace_section" in patch and "new_content" in patch:
        content, changed = apply_replace_section(
            content, patch["replace_section"], patch["new_content"]
        )
        if not changed:
            print(f"  ⚠️  Section not found in {rel_path}: '{patch['replace_section']}'")
            return False

    else:
        print(f"  ❌ Unknown patch operation in: {patch}")
        return False

    if dry_run:
        print(f"  [DRY RUN] Would update {rel_path}")
    else:
        target.write_text(content, encoding="utf-8")
        print(f"  ✅ Patched {rel_path}")

    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="Apply surgical doc patches for a release")
    parser.add_argument("--patch-file", required=True, help="Path to the YAML patch file")
    parser.add_argument("--dry-run", action="store_true", help="Preview changes without writing")
    parser.add_argument(
        "--root", default=str(Path(__file__).parent.parent), help="Repo root directory"
    )
    args = parser.parse_args()

    patch_file = Path(args.patch_file)
    if not patch_file.exists():
        print(f"ERROR: patch file not found: {patch_file}", file=sys.stderr)
        return 1

    with patch_file.open() as f:
        doc = yaml.safe_load(f)

    version = doc.get("version", "unknown")
    patches = doc.get("patches", [])
    root = Path(args.root)

    print(f"Applying doc patches for v{version} ({len(patches)} patch(es))…")
    if args.dry_run:
        print("[DRY RUN mode — no files will be modified]")

    success = 0
    fail = 0
    for patch in patches:
        ok = apply_patch(patch, root, args.dry_run)
        if ok:
            success += 1
        else:
            fail += 1

    print(f"\nResult: {success} succeeded, {fail} failed")
    return 0 if fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
