#!/usr/bin/env python3
import argparse
import json
import re
from datetime import date
from pathlib import Path

def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return ""

def extract_project_name(text: str) -> str:
    match = re.search(r"^#\s+(.+)$", text, re.MULTILINE)
    return match.group(1).strip() if match else "Unknown Project"

def extract_last_updated(text: str) -> str:
    match = re.search(r"\|\s*Last Updated\s*\|\s*([^|]+)\|", text)
    return match.group(1).strip() if match else ""

def main() -> None:
    parser = argparse.ArgumentParser(description="Generate task-master plan JSON from blueprint.")
    parser.add_argument("repo_root", help="Path to repo root")
    parser.add_argument("--output", default="task_master_plan.json", help="Output JSON filename")
    parser.add_argument("--mode", choices=["initial", "evolve"], default="initial", help="Plan mode")
    parser.add_argument("--baseline", default="", help="Baseline plan JSON to extend (evolve mode)")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    architecture_path = repo_root / "ProjectArchitecture.md"

    architecture_text = read_text(architecture_path)
    project_name = extract_project_name(architecture_text) or "Unknown Project"
    last_updated = extract_last_updated(architecture_text)

    assets_dir = Path(__file__).resolve().parent.parent / "assets"
    if args.mode == "evolve":
        blueprint_path = assets_dir / "task-master-evolution.json"
    else:
        blueprint_path = assets_dir / "task-master-blueprint.json"
    blueprint = json.loads(blueprint_path.read_text(encoding="utf-8"))

    baseline = None
    if args.baseline:
        baseline_path = (repo_root / args.baseline).resolve() if not Path(args.baseline).is_absolute() else Path(args.baseline)
        if baseline_path.exists():
            baseline = json.loads(baseline_path.read_text(encoding="utf-8"))

    if baseline:
        output = baseline
        output.setdefault("project", {})
        output["project"]["repo_root"] = str(repo_root)
        output["project"]["generated_on"] = date.today().isoformat()
        output["project"]["generated_by"] = "task-master-planner"
        output["project"]["generation_mode"] = args.mode
        existing_phases = output.get("phases", [])
        output["phases"] = existing_phases + blueprint.get("phases", [])
        output["project"].setdefault("assumptions", [])
        output["project"].setdefault("risks", [])
    else:
        output = {
            "schema_version": blueprint.get("schema_version", "1.0"),
            "project": {
                "name": project_name,
                "repo_root": str(repo_root),
                "generated_on": date.today().isoformat(),
                "generated_by": "task-master-planner",
                "generation_mode": args.mode,
                "assumptions": blueprint.get("assumptions", []),
                "risks": blueprint.get("risks", [])
            },
            "phases": blueprint.get("phases", [])
        }

    if last_updated:
        output["project"]["source_last_updated"] = last_updated

    output_path = repo_root / args.output
    output_path.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")

if __name__ == "__main__":
    main()
