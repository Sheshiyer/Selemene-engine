"""Phase 2 enricher — scan the HTML bodies of each reading's tab files for
additional ground-truth fields and merge them into the expected.json fixtures.

Currently extracts (from the ravecard summary table HTML at
`tabs/ravecard/ravecard/`):

  - definition       : Single | Split | TripleSplit | QuadrupleSplit | NoDefinition
  - strategy         : "Wait for an opportunity to respond", etc.
  - not_self_theme   : "Frustration", "Bitterness", "Anger", "Disappointment"

Fallback for `definition`: if the ravecard tab isn't present (multi-person
readings only have `tabs/compatibility/...`), we also scan the mechanics
tab body for the same definition phrases (legacy v1 behaviour).

Designed to be re-runnable. Fields already present in expected.json are
overwritten only when the HTML scan finds a value.

Usage:
    python humdes_html_enrich.py
    python humdes_html_enrich.py --source <bulk2_dir> --target <selemene_fixtures>
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


SOURCE_ROOT = Path(__file__).resolve().parent / "output"
TARGET = Path(
    "/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/"
    "Selemene-engine/tests/fixtures/humdes"
)


# Order matters — multi-word variants must be tested before their suffixes
# (so "Triple Definition" wins over "Definition", etc.). humdes labels them
# without the "Split" word for triple/quadruple — humdes copy uses:
#   "Single Definition", "Split Definition", "Triple Definition",
#   "Quadruple Definition", "No Definition".
DEFINITION_PATTERNS: list[tuple[str, str]] = [
    ("Quadruple Definition",        "QuadrupleSplit"),
    ("Quadruple Split Definition",  "QuadrupleSplit"),
    ("Triple Definition",           "TripleSplit"),
    ("Triple Split Definition",     "TripleSplit"),
    ("Split Definition",            "Split"),
    ("Single Definition",           "Single"),
    ("No Definition",               "NoDefinition"),
]

# Canonicalise the Strategy and Not-Self values to short enum-ish tokens
# the Rust engine could compare against. We keep the raw humdes label too
# for traceability.
STRATEGY_CANON: dict[str, str] = {
    "Wait for an opportunity to respond": "WaitToRespond",
    "Wait for an invitation":             "WaitForInvitation",
    "Inform":                             "Inform",
    "Inform before acting":               "Inform",
    "Make decisions after waiting out the lunar cycle": "LunarCycle",
    "Wait a lunar cycle":                 "LunarCycle",
}

NOT_SELF_CANON: dict[str, str] = {
    "Frustration":                                        "Frustration",
    "Frustration with yourself and others":               "Frustration",
    "Frustration and impatience":                         "Frustration",
    "The bitterness of not recognizing their merits":     "Bitterness",
    "Bitterness from feeling unrecognized":               "Bitterness",
    "Bitterness":                                         "Bitterness",
    "Anger":                                              "Anger",
    "Anger when not free":                                "Anger",
    "Anger from being controlled":                        "Anger",
    "Feeling angry and angry":                            "Anger",  # humdes copy-bug: duplicated word
    "Disappointment":                                     "Disappointment",
    "Disappointment in other people":                     "Disappointment",
    "Disappointment in life":                             "Disappointment",
}


# Match a (label, value) pair inside the ravecard summary table HTML.
# The block looks like:
#   <div class="calculation-results-ravecard__props-item-label ...">  LABEL  </div>
#   <div class="calculation-results-ravecard__props-item-value ...">  VALUE  </div>
_RAVECARD_PROPS_RE = re.compile(
    r'class="calculation-results-ravecard__props-item-label[^"]*"\s*>\s*'
    r'([^<]+?)\s*</div>\s*'
    r'<div class="calculation-results-ravecard__props-item-value[^"]*"\s*>\s*'
    r'(.*?)\s*</div>',
    re.DOTALL,
)


def _strip_html(html: str) -> str:
    """Collapse HTML tags + whitespace + HTML entities into a single line."""
    text = re.sub(r"<[^>]+>", " ", html)
    text = text.replace("&nbsp;", " ")
    return re.sub(r"\s+", " ", text).strip()


def _parse_ravecard_props(html: str) -> dict[str, str]:
    """Return {label: cleaned_value} for every prop row in the ravecard
    summary table. Empty if the HTML doesn't have the expected layout."""
    out: dict[str, str] = {}
    if not html:
        return out
    for label_raw, value_raw in _RAVECARD_PROPS_RE.findall(html):
        label = _strip_html(label_raw)
        value = _strip_html(value_raw)
        if label and value:
            out[label] = value
    return out


def _extract_definition(text_or_html: str) -> str | None:
    """Scan a string for a definition phrase and return the canonical token."""
    if not text_or_html:
        return None
    # Accept either raw HTML or already-stripped text.
    text = _strip_html(text_or_html) if "<" in text_or_html else text_or_html
    for pattern, label in DEFINITION_PATTERNS:
        # Case-insensitive whole-phrase match.
        if re.search(rf"\b{re.escape(pattern)}\b", text, re.IGNORECASE):
            return label
    return None


def _canonicalise_strategy(value: str) -> str | None:
    if not value:
        return None
    return STRATEGY_CANON.get(value.strip(), value.strip())


def _canonicalise_not_self(value: str) -> str | None:
    if not value:
        return None
    return NOT_SELF_CANON.get(value.strip(), value.strip())


def _find_latest_bulk2(source: Path) -> Path:
    # Prefer a v3 (bulk3) output if it exists, otherwise fall back to v2.
    for tag in ("*_bulk3", "*_bulk2"):
        cands = [p for p in source.glob(tag) if (p / "readings").is_dir()]
        if cands:
            return max(cands, key=lambda p: p.stat().st_mtime)
    raise FileNotFoundError(f"No bulk output found under {source}")


def _load_index(target: Path) -> dict:
    idx_path = target / "_index.json"
    if not idx_path.exists():
        raise FileNotFoundError(
            f"{idx_path} not found. Run humdes_to_selemene.py first."
        )
    return json.loads(idx_path.read_text(encoding="utf-8"))


def _find_reading_dir(bulk_root: Path, type_name: str, reading_hash: str) -> Path | None:
    parent = bulk_root / "readings" / type_name
    if not parent.exists():
        return None
    for child in parent.iterdir():
        if child.is_dir() and child.name.startswith(reading_hash):
            return child
    return None


def _load_body(path: Path) -> str:
    try:
        obj = json.loads(path.read_text(encoding="utf-8"))
    except Exception:  # noqa: BLE001
        return ""
    data = obj.get("data") or {}
    if isinstance(data, dict):
        return data.get("body", "") or ""
    return ""


def _ravecard_summary_body(reading_dir: Path) -> str:
    """Find the ravecard summary HTML body for a reading.

    The capture order in bulk_fetch_v2/v3 puts the parent `Ravechart` tab
    response under a filename matching `*tabs_rav*.json`. Multi-person
    readings (compatibility/business/family) don't have a per-person
    ravecard tab — they have `*tabs_com*.json` etc. — so this returns ""
    for those.
    """
    for cand in reading_dir.glob("*tabs_rav*.json"):
        body = _load_body(cand)
        if body:
            return body
    return ""


def _mechanics_body(reading_dir: Path) -> str:
    for cand in reading_dir.glob("*tabs_mec*.json"):
        body = _load_body(cand)
        if body:
            return body
    return ""


def enrich_entry(entry: dict, source: Path, target: Path) -> dict:
    """Enrich one expected.json file in place. Returns a dict of changes."""
    type_name = entry["type"]
    reading_hash = entry["reading_hash"]

    reading_dir = _find_reading_dir(source, type_name, reading_hash)
    if reading_dir is None:
        return {"status": "no_source_dir"}

    expected_path = target / entry["expected"]
    if not expected_path.exists():
        return {"status": "no_expected"}

    expected_obj = json.loads(expected_path.read_text(encoding="utf-8"))
    expected = expected_obj.setdefault("expected", {})

    changes: dict[str, tuple] = {}

    # Try the ravecard summary first — it has structured props.
    rav_body = _ravecard_summary_body(reading_dir)
    rav_props = _parse_ravecard_props(rav_body) if rav_body else {}

    # --- definition ---
    new_def = None
    if rav_props.get("Definition"):
        new_def = _extract_definition(rav_props["Definition"])
    if new_def is None and rav_body:
        new_def = _extract_definition(rav_body)
    if new_def is None:
        # Fallback: scan mechanics HTML.
        mech_body = _mechanics_body(reading_dir)
        new_def = _extract_definition(mech_body) if mech_body else None
    if new_def is not None:
        prev = expected.get("definition")
        if prev != new_def:
            expected["definition"] = new_def
            changes["definition"] = (prev, new_def)

    # --- strategy ---
    if rav_props.get("Strategy"):
        new_strat = _canonicalise_strategy(rav_props["Strategy"])
        if new_strat is not None:
            prev = expected.get("strategy")
            if prev != new_strat:
                expected["strategy"] = new_strat
                changes["strategy"] = (prev, new_strat)
        # Keep the verbatim humdes label too for audit/debug.
        prev_raw = expected.get("strategy_humdes_label")
        if prev_raw != rav_props["Strategy"]:
            expected["strategy_humdes_label"] = rav_props["Strategy"]
            changes.setdefault("strategy_humdes_label", (prev_raw, rav_props["Strategy"]))

    # --- not_self_theme ---
    if rav_props.get("Not-Self"):
        new_ns = _canonicalise_not_self(rav_props["Not-Self"])
        if new_ns is not None:
            prev = expected.get("not_self_theme")
            if prev != new_ns:
                expected["not_self_theme"] = new_ns
                changes["not_self_theme"] = (prev, new_ns)
        prev_raw = expected.get("not_self_humdes_label")
        if prev_raw != rav_props["Not-Self"]:
            expected["not_self_humdes_label"] = rav_props["Not-Self"]
            changes.setdefault("not_self_humdes_label", (prev_raw, rav_props["Not-Self"]))

    if changes:
        expected_path.write_text(
            json.dumps(expected_obj, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        return {"status": "enriched", "changes": list(changes.keys())}
    return {"status": "no_change"}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", type=Path, default=None)
    ap.add_argument("--target", type=Path, default=TARGET)
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    source = args.source or _find_latest_bulk2(SOURCE_ROOT)
    target = args.target
    print(f"Source : {source}")
    print(f"Target : {target}")

    index = _load_index(target)
    counts = {"enriched": 0, "no_change": 0, "no_source_dir": 0, "no_expected": 0}
    by_field: dict[str, int] = {}
    sample_lines: list[str] = []

    for entry in index["entries"]:
        result = enrich_entry(entry, source, target)
        status = result["status"]
        counts[status] = counts.get(status, 0) + 1
        if status == "enriched":
            for k in result.get("changes", []):
                by_field[k] = by_field.get(k, 0) + 1
            if args.verbose or len(sample_lines) < 5:
                fields = ",".join(result["changes"])
                sample_lines.append(
                    f"  {entry['type']:13s} {entry['reading_hash'][:10]}.. "
                    f"person={entry['person_index']} -> {fields}"
                )

    print("\n=== Done ===")
    for line in sample_lines:
        print(line)
    print()
    print(f"  Enriched : {counts['enriched']}")
    print(f"  No change : {counts['no_change']}")
    print(f"  No source dir : {counts['no_source_dir']}")
    print(f"  No expected   : {counts['no_expected']}")
    print()
    print("  Fields populated per kind:")
    for k, v in sorted(by_field.items(), key=lambda x: -x[1]):
        print(f"    {v:4d}  {k}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
