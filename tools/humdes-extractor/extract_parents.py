"""Extract self-contained solo-reading bundles for a named set of people.

For each target person, produces under ~/Downloads/humdes-extractor/parents/<slug>/:

  raw/
      _row.json                  # directory metadata
      01_GET_ravecard_<hash>_site_1.json
      02..11_*.json              # all tab captures (HTML bodies + structured)
  selemene/
      input.json                 # EngineInput (post-normalisation)
      expected.json              # humdes ground truth
      metadata.json              # provenance
  engine/
      output.json                # full EngineOutput from HumanDesignEngine
  README.md                      # human-readable chart summary

Defaults are wired for the user's parents — pass --target name=hash overrides
to run on other people.

Usage:
    python extract_parents.py                                # both parents
    python extract_parents.py --target "Some Name=hash..."   # custom
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SELEMENE_WORKTREE = Path(
    "/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/"
    "Selemene-engine/.worktrees/parents-extract"
)
SELEMENE_FIXTURES = SELEMENE_WORKTREE / "tests/fixtures/humdes"
DEFAULT_BULK = ROOT / "output/2026-05-16_124939_bulk2"
PARENTS_ROOT = ROOT / "parents"

DEFAULT_TARGETS = {
    "Cumbipuram Subramaniam Nateshan (father)": "8b92029f67ad279a128c4a70204b2de5",
    "Anitha Nateshan (mother)":                  "bc74efac1cd572f87d9e8fb356fbd81f",
}


def _slug(name: str) -> str:
    s = re.sub(r"[^\w\-]+", "_", name).strip("_")
    return re.sub(r"_+", "_", s) or "x"


def _find_dir(parent: Path, prefix: str) -> Path | None:
    for child in parent.iterdir():
        if child.is_dir() and child.name.startswith(prefix):
            return child
    return None


def _run_engine(input_path: Path, output_path: Path) -> None:
    """Invoke the run_one example from the parents-extract worktree."""
    cmd = [
        "cargo", "run", "--quiet",
        "--package", "engine-human-design",
        "--example", "run_one",
        "--", str(input_path),
    ]
    result = subprocess.run(
        cmd,
        cwd=str(SELEMENE_WORKTREE),
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"engine run_one failed (exit {result.returncode}):\n"
            f"  stderr: {result.stderr[:400]}"
        )
    # Validate JSON
    obj = json.loads(result.stdout)
    output_path.write_text(json.dumps(obj, indent=2, ensure_ascii=False), encoding="utf-8")


def _render_markdown(
    name: str,
    raw_row: dict,
    expected: dict,
    engine_out: dict,
    engine_input: dict,
) -> str:
    """Build the human-readable chart summary."""
    bd = engine_input.get("birth_data", {})
    e = expected.get("expected", {})
    r = engine_out.get("result", {})

    pa = r.get("personality_activations", {})
    da = r.get("design_activations", {})

    def gl(act_dict: dict, planet: str) -> str:
        a = act_dict.get(planet) or {}
        if a.get("gate") is not None and a.get("line") is not None:
            return f"{a['gate']}.{a['line']}"
        return "—"

    centers = sorted(r.get("defined_centers", []))
    channels = r.get("active_channels", [])
    cross = e.get("incarnation_cross", {}) or {}

    lines: list[str] = []
    lines.append(f"# {name} — Human Design solo reading\n")
    lines.append("## Birth\n")
    lines.append(f"- **Name**: {bd.get('name')}")
    lines.append(f"- **Date**: {bd.get('date')}")
    lines.append(f"- **Time** (local): {bd.get('time')}")
    lines.append(f"- **Timezone**: {bd.get('timezone')}")
    lines.append(f"- **Coordinates**: {bd.get('latitude'):.4f}, {bd.get('longitude'):.4f}"
                 if bd.get("latitude") is not None else "- **Coordinates**: —")
    opts = engine_input.get("options", {}) or {}
    if opts.get("humdes_location_string"):
        lines.append(f"- **Location**: {opts['humdes_location_string']}")
    lines.append("")

    lines.append("## Chart at a glance\n")
    lines.append("|              | humdes ground truth | Selemene engine |")
    lines.append("|--------------|---------------------|-----------------|")
    lines.append(f"| **Type**     | {e.get('type','—')} | {r.get('hd_type','—')} |")
    prof_e = e.get("profile") or {}
    prof_text = prof_e.get("text") if isinstance(prof_e, dict) else "—"
    lines.append(f"| **Profile**  | {prof_text} | {r.get('profile','—')} |")
    lines.append(f"| **Authority**| {e.get('authority','—')} | {r.get('authority','—')} |")
    lines.append(f"| **Definition**| {e.get('definition') or '—'} | {r.get('definition','—')} |")
    lines.append(f"| **Strategy** | {e.get('strategy') or '—'} | — |")
    lines.append(f"| **Not-Self** | {e.get('not_self_theme') or '—'} | — |")
    lines.append("")

    lines.append("## Incarnation Cross\n")
    if cross.get("name"):
        lines.append(f"- **{cross['name']}**")
    gates = cross.get("gates") or []
    if gates:
        lines.append(f"- Gates (P-sun · P-earth · D-sun · D-earth): "
                     f"{' · '.join(str(g) if g is not None else '?' for g in gates)}")
    lines.append("")

    lines.append("## Defined centers\n")
    if centers:
        lines.append(", ".join(centers))
    else:
        lines.append("_(none — chart fully open / Reflector)_")
    lines.append("")

    lines.append("## Active channels\n")
    if channels:
        for ch in channels:
            lines.append(f"- {ch}")
    else:
        lines.append("_(no defined channels)_")
    lines.append("")

    lines.append("## All 26 planetary activations\n")
    planets = [
        "sun", "earth", "moon", "north_node", "south_node",
        "mercury", "venus", "mars", "jupiter", "saturn",
        "uranus", "neptune", "pluto",
    ]
    lines.append("| Planet | Personality (gate.line) | Design (gate.line) |")
    lines.append("|---|---|---|")
    for p in planets:
        lines.append(f"| {p.replace('_',' ').title()} | {gl(pa,p)} | {gl(da,p)} |")
    lines.append("")

    lines.append("## Variables (PHS)\n")
    vars_ = e.get("variables") or []
    if vars_:
        lines.append(f"- humdes labels: `{' / '.join(vars_)}`")
        lines.append("  - First letter: Personality Sun arrow (Left/Right brain orientation)")
        lines.append("  - Each two-letter pair: arrow position for that variable")
    else:
        lines.append("_(not present in ground truth)_")
    lines.append("")

    lines.append("## Witness prompt (from Selemene engine)\n")
    wp = engine_out.get("witness_prompt") or "—"
    lines.append(f"> {wp}")
    lines.append("")

    lines.append("## Validation against humdes\n")
    cardinal_match = (
        e.get("type") == r.get("hd_type")
        and prof_text == r.get("profile")
        and e.get("authority") == r.get("authority")
    )
    if cardinal_match:
        lines.append("✓ Selemene engine output matches humdes ground truth on type, profile, authority.")
    else:
        lines.append("⚠ Disagreement on one or more cardinal fields — see table above.")
    p_sun_match = (e.get("personality_sun") or {}).get("gate") == (pa.get("sun") or {}).get("gate")
    if p_sun_match:
        lines.append("✓ Personality-Sun gate matches.")
    else:
        lines.append(f"⚠ P-sun gate: humdes={(e.get('personality_sun') or {}).get('gate')} "
                     f"engine={(pa.get('sun') or {}).get('gate')}")
    lines.append("")

    lines.append("## Source files in this bundle\n")
    lines.append("- `raw/` — all JSON captured from humdes.com (ravecard + 9 tabs + directory metadata)")
    lines.append("- `selemene/input.json` — normalised EngineInput")
    lines.append("- `selemene/expected.json` — humdes ground-truth for validation")
    lines.append("- `selemene/metadata.json` — provenance + raw humdes fields")
    lines.append("- `engine/output.json` — full EngineOutput from Selemene HumanDesign engine")
    lines.append("- `README.md` — this file")
    return "\n".join(lines) + "\n"


def extract_one(label: str, reading_hash: str, bulk_root: Path) -> Path:
    print(f"\n--- {label}  (hash {reading_hash[:10]}..) ---")

    # 1. Locate source folders
    bulk_dir = _find_dir(bulk_root / "readings/personal", reading_hash)
    if bulk_dir is None:
        raise FileNotFoundError(
            f"No bulk2 personal folder for hash {reading_hash} under {bulk_root}"
        )
    fixture_dir = _find_dir(SELEMENE_FIXTURES / "readings/personal", reading_hash)
    if fixture_dir is None:
        raise FileNotFoundError(
            f"No Selemene fixture folder for hash {reading_hash}"
        )
    print(f"  bulk source : {bulk_dir.name}")
    print(f"  fixture     : {fixture_dir.name}")

    # 2. Prep output folder
    out_root = PARENTS_ROOT / _slug(label)
    if out_root.exists():
        shutil.rmtree(out_root)
    (out_root / "raw").mkdir(parents=True)
    (out_root / "selemene").mkdir(parents=True)
    (out_root / "engine").mkdir(parents=True)

    # 3. Copy raw bulk files
    for f in sorted(bulk_dir.iterdir()):
        if f.is_file():
            shutil.copy2(f, out_root / "raw" / f.name)
    raw_count = len(list((out_root / "raw").iterdir()))
    print(f"  raw files   : {raw_count}")

    # 4. Copy normalised fixture
    for stem in ("01_input", "01_expected", "01_metadata"):
        src = fixture_dir / f"{stem}.json"
        if src.exists():
            shutil.copy2(src, out_root / "selemene" / f"{stem.split('_',1)[1]}.json")
    print(f"  selemene    : {len(list((out_root/'selemene').iterdir()))} files")

    # 5. Run engine
    input_path = out_root / "selemene" / "input.json"
    engine_out_path = out_root / "engine" / "output.json"
    _run_engine(input_path, engine_out_path)
    print(f"  engine out  : {engine_out_path.stat().st_size} bytes")

    # 6. Build markdown
    row_path = out_root / "raw" / "_row.json"
    raw_row = json.loads(row_path.read_text(encoding="utf-8")) if row_path.exists() else {}
    expected = json.loads((out_root / "selemene" / "expected.json").read_text(encoding="utf-8"))
    engine_out = json.loads(engine_out_path.read_text(encoding="utf-8"))
    engine_input = json.loads(input_path.read_text(encoding="utf-8"))
    md = _render_markdown(label, raw_row, expected, engine_out, engine_input)
    (out_root / "README.md").write_text(md, encoding="utf-8")
    print(f"  README.md   : {(out_root/'README.md').stat().st_size} bytes")

    return out_root


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--bulk", type=Path, default=DEFAULT_BULK,
                    help="bulk2 source folder")
    ap.add_argument("--target", action="append", default=[],
                    help="extra targets in 'Label=hash' form")
    args = ap.parse_args()

    targets = dict(DEFAULT_TARGETS)
    for t in args.target:
        if "=" not in t:
            print(f"Bad --target {t!r} (expected 'Label=hash')", file=sys.stderr)
            return 2
        label, h = t.split("=", 1)
        targets[label.strip()] = h.strip()

    if not args.bulk.exists():
        print(f"Bulk source folder not found: {args.bulk}", file=sys.stderr)
        return 2

    PARENTS_ROOT.mkdir(parents=True, exist_ok=True)
    extracted: list[Path] = []
    for label, h in targets.items():
        try:
            extracted.append(extract_one(label, h, args.bulk))
        except Exception as e:  # noqa: BLE001
            print(f"  FAILED for {label}: {e}", file=sys.stderr)

    print(f"\n=== Done ===")
    for p in extracted:
        size = sum(f.stat().st_size for f in p.rglob("*") if f.is_file())
        files = sum(1 for f in p.rglob("*") if f.is_file())
        print(f"  {p}")
        print(f"    {files} files, {size/1024:.1f} KB")
    print(f"\nRoot: {PARENTS_ROOT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
