"""Build the subjects-dir for witness-agents' composite-triad mode.

Three subjects: Anitha (mother) + WitnessAlchemist (son) + Nateshan (father).

For Anitha + Nateshan we already have HD data from humdes + Selemene (used
by build_synastry_subjects.py). For WitnessAlchemist we pull HD from the
existing 723/integratedreading.config.json which has the canonical chart
summary. All three get Vedic placements from compute_vedic_kundali.py.

Output:
    parents/triad-anitha-witnessalchemist-nateshan/
        01_anitha.json
        02_witnessalchemist.json
        03_nateshan.json
"""

from __future__ import annotations

import json
from datetime import datetime, timezone as tz
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PARENTS_ROOT = ROOT / "parents"
OUT_DIR = PARENTS_ROOT / "triad-anitha-witnessalchemist-nateshan"
OUT_DIR.mkdir(parents=True, exist_ok=True)

# Path constants
WITNESS_READING_DIR = Path(
    "/Volumes/madara/2026/twc-vault/01-Projects/723/witnessalchemist-reading"
)
WITNESS_SOURCE_DOCX = WITNESS_READING_DIR / "inputs/01_Reading_WitnessAlchemist.docx"
WITNESS_INTEGRATED_CFG = Path(
    "/Volumes/madara/2026/twc-vault/01-Projects/723/integratedreading.config.json"
)

ANITHA_DIR = Path("/Volumes/madara/2026/twc-vault/01-Projects/723/anitha-nateshan-mother-reading")
# Nateshan's reading folder was moved into _legacy/ during repo reorg.
# The cached solo synthesis (the only thing the orchestrator needs from
# this path) is still valid — independent of the family-role correction.
NATESHAN_DIR = Path("/Volumes/madara/2026/twc-vault/01-Projects/723/_legacy/cumbipuram-subramaniam-nateshan-father-reading")


def _build_condition_str(planet_name: str, pp: dict) -> str:
    bits = []
    if pp.get("dignity") and pp["dignity"] != "—":
        bits.append(pp["dignity"])
    if pp.get("retrograde"):
        bits.append("retrograde")
    bits.append(f"H{pp['house']}")
    bits.append(f"{pp['nakshatra']} P{pp['pada']} (lord {pp['nakshatra_lord']})")
    return "; ".join(bits)


def _placements_from_kundali(kundali_json: dict) -> list[dict]:
    out = []
    for planet_name in ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu", "ketu"]:
        pp = kundali_json["planets"][planet_name.capitalize()]
        sign_clean = pp["sign"].split(" ")[0].lower()
        placement = {
            "planet": planet_name,
            "sign": sign_clean,
            "sign_english": pp["sign"],
            "house": pp["house"],
            "degree": pp["deg_str"],
            "nakshatra": f"{pp['nakshatra']} P{pp['pada']}",
            "nakshatra_lord": pp["nakshatra_lord"],
            "condition": _build_condition_str(planet_name, pp),
        }
        if pp.get("retrograde"):
            placement["retrograde"] = True
        out.append(placement)
    return out


def _mahadasha_block(kundali_json: dict) -> dict:
    mds = kundali_json["mahadashas"]
    current = next((m for m in mds if m["is_current"]), mds[0])
    next_idx = mds.index(current) + 1
    nxt = mds[next_idx] if next_idx < len(mds) else None
    cur_ad = None
    if current.get("antardashas"):
        now = datetime.now(tz.utc)
        for ad in current["antardashas"]:
            a_start = datetime.fromisoformat(ad["start"].replace("Z", "+00:00"))
            a_end = datetime.fromisoformat(ad["end"].replace("Z", "+00:00"))
            if a_start <= now <= a_end:
                cur_ad = ad
                break
    return {
        "current_lord": current["lord"].lower(),
        "current_started_iso": current["start"],
        "current_ends_iso": current["end"],
        "current_antardasha": cur_ad["lord"].lower() if cur_ad else None,
        "current_antardasha_window": (
            f"{cur_ad['start'][:10]} → {cur_ad['end'][:10]}" if cur_ad else None
        ),
        "next_lord": nxt["lord"].lower() if nxt else None,
        "next_starts_iso": nxt["start"] if nxt else None,
        "next_duration_years": nxt["duration_years"] if nxt else None,
        "yogas": [
            {"name": y["name"], "status": y["status"], "effect": y["effect"]}
            for y in kundali_json["yogas"]
        ],
    }


def _load_kundali(bundle_dir: Path, slug_for_filename: str) -> tuple[dict, Path]:
    """Return (kundali_json_dict, kundali_md_path)."""
    j_path = bundle_dir / f"inputs/Kundali_{slug_for_filename}.json"
    m_path = bundle_dir / f"inputs/Kundali_{slug_for_filename}.md"
    return json.load(open(j_path)), m_path


# ─── Per-subject builders ───────────────────────────────────────────────

def build_anitha() -> Path:
    bundle = PARENTS_ROOT / "Anitha_Nateshan_mother"
    kundali, kundali_md = _load_kundali(bundle, "Anitha_Nateshan")
    hd_input = json.load(open(bundle / "selemene/input.json"))
    hd_expected = json.load(open(bundle / "selemene/expected.json"))["expected"]
    hd_engine = json.load(open(bundle / "engine/output.json"))["result"]

    subject_json = {
        "source_path": str(kundali_md),
        "subject": "Anitha Nateshan",
        "birth_date": "1965-06-01",
        "birth_time": "00:35",
        "birth_place": "Bengaluru, India",
        "latitude": 12.9768,
        "longitude": 77.5901,
        "timezone": "Asia/Kolkata",
        "lagna": kundali["lagna"]["sign"].split(" ")[0].lower(),
        "lagna_degree": kundali["lagna"]["deg_str"],
        "lagna_nakshatra": f"{kundali['lagna']['nakshatra']} P{kundali['lagna']['pada']}",
        "lagna_lord": kundali["lagna"]["lord"].lower(),
        "atmakaraka": kundali["karakas"]["atmakaraka"].lower(),
        "darakaraka": kundali["karakas"]["darakaraka"].lower(),
        "birth_nakshatra": f"{kundali['planets']['Moon']['nakshatra']} P{kundali['planets']['Moon']['pada']}",
        "placements": _placements_from_kundali(kundali),
        "mahadasha": _mahadasha_block(kundali),
        "human_design": {
            "type": hd_engine["hd_type"],
            "authority": hd_engine["authority"],
            "profile": hd_engine["profile"],
            "definition": hd_engine["definition"],
            "incarnation_cross": (hd_expected.get("incarnation_cross") or {}).get("name"),
            "strategy": hd_expected.get("strategy"),
            "not_self_theme": hd_expected.get("not_self_theme"),
            "active_channels": hd_engine.get("active_channels"),
            "defined_centers": sorted(hd_engine.get("defined_centers", [])),
            "variables": hd_expected.get("variables"),
        },
        "pancha_bhuta": kundali["pancha_bhuta"],
        "chart_assets": {
            "vedic_kundali_md": str(kundali_md),
            "_notes": "Vedic via pyswisseph Lahiri sidereal; HD via humdes + Selemene engine.",
        },
        "relationship": {
            "role": "mother",
            "relations": {
                "spouse": "nateshan",
                "children": ["witnessalchemist"],
            },
            "notes": "Married to Cumbipuram Subramaniam Nateshan (the father, file 03_nateshan). Mother of Sheshnarayan / WitnessAlchemist (the son, file 02_witnessalchemist).",
        },
        "output_dir": str(ANITHA_DIR),
    }
    out = OUT_DIR / "01_anitha.json"
    out.write_text(json.dumps(subject_json, indent=2, ensure_ascii=False))
    return out


def build_witnessalchemist() -> Path:
    bundle = PARENTS_ROOT / "Sheshnarayan_witnessalchemist"
    kundali, kundali_md = _load_kundali(bundle, "Sheshnarayan_Cumbipuram_Nateshan_(WitnessAlchemist)")

    # Pull HD from existing 723 integratedreading.config.json
    cfg = json.load(open(WITNESS_INTEGRATED_CFG))
    witness_cfg = next(s for s in cfg["subjects"] if s["name"] == "WitnessAlchemist")
    cs = witness_cfg["chart_summary"]

    # Parse "Right Angle Cross of Explanation (4/49 | 23/43)" → name + gates
    cross_text = cs.get("hd_cross", "")
    cross_name, cross_gates = cross_text, []
    if "(" in cross_text and ")" in cross_text:
        cross_name = cross_text.split("(")[0].strip()
        gates_part = cross_text.split("(", 1)[1].rstrip(")")
        for g in gates_part.replace("|", "/").split("/"):
            try:
                cross_gates.append(int(g.strip()))
            except ValueError:
                pass

    subject_json = {
        "source_path": str(WITNESS_SOURCE_DOCX),
        "subject": "WitnessAlchemist",
        "birth_date": "1991-08-13",
        "birth_time": "13:31",
        "birth_place": "Bengaluru, India",
        "latitude": 12.97,
        "longitude": 77.59,
        "timezone": "Asia/Kolkata",
        "lagna": kundali["lagna"]["sign"].split(" ")[0].lower(),
        "lagna_degree": kundali["lagna"]["deg_str"],
        "lagna_nakshatra": f"{kundali['lagna']['nakshatra']} P{kundali['lagna']['pada']}",
        "lagna_lord": kundali["lagna"]["lord"].lower(),
        "atmakaraka": kundali["karakas"]["atmakaraka"].lower(),
        "darakaraka": kundali["karakas"]["darakaraka"].lower(),
        "birth_nakshatra": f"{kundali['planets']['Moon']['nakshatra']} P{kundali['planets']['Moon']['pada']}",
        "placements": _placements_from_kundali(kundali),
        "mahadasha": _mahadasha_block(kundali),
        "human_design": {
            "type": cs.get("hd_type"),
            "authority": cs.get("hd_authority"),
            "profile": cs.get("hd_profile"),
            "definition": cs.get("hd_definition"),
            "incarnation_cross": cross_name,
            "incarnation_cross_gates": cross_gates,
            "active_channels": cs.get("hd_channels"),
        },
        "gene_keys": {
            "pearl": cs.get("gene_keys_pearl"),
            "personality_sun_gift": cs.get("gene_keys_sun_personality"),
            "iq": cs.get("gene_keys_iq"),
            "sq": cs.get("gene_keys_sq"),
        },
        "pancha_bhuta": kundali["pancha_bhuta"],
        "yogas_from_existing_reading": cs.get("yogas", []),
        "chart_assets": {
            "vedic_kundali_md": str(kundali_md),
            "source_docx": str(WITNESS_SOURCE_DOCX),
            "_notes": "Vedic via pyswisseph Lahiri sidereal; HD + Gene Keys from existing 723/integratedreading.config.json (chart already curated).",
        },
        "relationship": {
            "role": "child",
            "relations": {
                "mother": "anitha",
                "father": "nateshan",
            },
            "notes": "Son of Anitha Nateshan (mother, file 01_anitha) and Cumbipuram Subramaniam Nateshan (father, file 03_nateshan). Read parent-child dyads as Putra-karaka inheritance, NEVER as Vivaha. Not a sibling to either parent.",
        },
        "output_dir": str(WITNESS_READING_DIR),
    }
    out = OUT_DIR / "02_witnessalchemist.json"
    out.write_text(json.dumps(subject_json, indent=2, ensure_ascii=False))
    return out


def build_nateshan() -> Path:
    bundle = PARENTS_ROOT / "Cumbipuram_Subramaniam_Nateshan_father"
    kundali, kundali_md = _load_kundali(bundle, "Cumbipuram_Subramaniam_Nateshan")
    hd_input = json.load(open(bundle / "selemene/input.json"))
    hd_expected = json.load(open(bundle / "selemene/expected.json"))["expected"]
    hd_engine = json.load(open(bundle / "engine/output.json"))["result"]

    subject_json = {
        "source_path": str(kundali_md),
        "subject": "Cumbipuram Subramaniam Nateshan",
        "birth_date": "1960-11-20",
        "birth_time": "17:15",
        "birth_place": "Bengaluru, India",
        "latitude": 12.9768,
        "longitude": 77.5901,
        "timezone": "Asia/Kolkata",
        "lagna": kundali["lagna"]["sign"].split(" ")[0].lower(),
        "lagna_degree": kundali["lagna"]["deg_str"],
        "lagna_nakshatra": f"{kundali['lagna']['nakshatra']} P{kundali['lagna']['pada']}",
        "lagna_lord": kundali["lagna"]["lord"].lower(),
        "atmakaraka": kundali["karakas"]["atmakaraka"].lower(),
        "darakaraka": kundali["karakas"]["darakaraka"].lower(),
        "birth_nakshatra": f"{kundali['planets']['Moon']['nakshatra']} P{kundali['planets']['Moon']['pada']}",
        "placements": _placements_from_kundali(kundali),
        "mahadasha": _mahadasha_block(kundali),
        "human_design": {
            "type": hd_engine["hd_type"],
            "authority": hd_engine["authority"],
            "profile": hd_engine["profile"],
            "definition": hd_engine["definition"],
            "incarnation_cross": (hd_expected.get("incarnation_cross") or {}).get("name"),
            "strategy": hd_expected.get("strategy"),
            "not_self_theme": hd_expected.get("not_self_theme"),
            "active_channels": hd_engine.get("active_channels"),
            "defined_centers": sorted(hd_engine.get("defined_centers", [])),
            "variables": hd_expected.get("variables"),
        },
        "pancha_bhuta": kundali["pancha_bhuta"],
        "chart_assets": {
            "vedic_kundali_md": str(kundali_md),
            "_notes": "Vedic via pyswisseph Lahiri sidereal; HD via humdes + Selemene engine.",
        },
        "relationship": {
            "role": "father",
            "relations": {
                "spouse": "anitha",
                "children": ["witnessalchemist"],
            },
            "notes": "Married to Anitha Nateshan (the mother, file 01_anitha). Father of Sheshnarayan / WitnessAlchemist (the son, file 02_witnessalchemist).",
        },
        "output_dir": str(NATESHAN_DIR),
    }
    out = OUT_DIR / "03_nateshan.json"
    out.write_text(json.dumps(subject_json, indent=2, ensure_ascii=False))
    return out


def main() -> int:
    print(f"Output: {OUT_DIR}")
    for name, builder in [("anitha", build_anitha), ("witnessalchemist", build_witnessalchemist), ("nateshan", build_nateshan)]:
        p = builder()
        size = p.stat().st_size
        print(f"  {p.name}  ({size} bytes)")
    print()
    print("Next:")
    print(f"  node --import tsx scripts/integratedreading-mode.ts \\")
    print(f"    --mode composite-triad \\")
    print(f"    --subjects-dir {OUT_DIR} \\")
    print(f"    --output-dir /Volumes/madara/2026/twc-vault/01-Projects/723/family-triad-anitha-witnessalchemist-nateshan \\")
    print(f"    --skip-solos")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
