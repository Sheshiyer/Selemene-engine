"""Build the subjects-dir for witness-agents' partner-synastry mode.

Combines:
- Vedic placements from compute_vedic_kundali.py output
- HD data from humdes_to_selemene.py output + Selemene engine output
- Existing kundali markdown as source_path for ingestion

Writes:
    parents/synastry-anitha-nateshan/01_anitha.json
    parents/synastry-anitha-nateshan/02_nateshan.json
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PARENTS_ROOT = ROOT / "parents"
OUT_DIR = PARENTS_ROOT / "synastry-anitha-nateshan"
OUT_DIR.mkdir(parents=True, exist_ok=True)

# Ordering: 01 = mother (Anitha) since synastry mode treats partner-A / partner-B
# as ordinal slots; we list her first so partner-A is mother. Order is symmetric
# for the actual analysis — both charts are read in both directions.
SUBJECTS = [
    {
        "idx": "01",
        "slug": "anitha",
        "bundle_name": "Anitha_Nateshan_mother",
        "subject_name": "Anitha Nateshan",
        # The orchestrator's findExistingSolo() looks under <output_dir>/.runs/<ts>/06_synthesis_<slug>.md.
        # We already ran integratedreading.ts for both parents and produced cached synthesis there.
        "output_dir": "/Volumes/madara/2026/twc-vault/01-Projects/723/anitha-nateshan-mother-reading",
    },
    {
        "idx": "02",
        "slug": "nateshan",
        "bundle_name": "Cumbipuram_Subramaniam_Nateshan_father",
        "subject_name": "Cumbipuram Subramaniam Nateshan",
        "output_dir": "/Volumes/madara/2026/twc-vault/01-Projects/723/cumbipuram-subramaniam-nateshan-father-reading",
    },
]


def build_one(s: dict) -> Path:
    bundle = PARENTS_ROOT / s["bundle_name"]
    kundali_json = json.load(open(bundle / f"inputs/Kundali_{s['bundle_name'].rsplit('_', 1)[0]}.json"))
    kundali_md = bundle / f"inputs/Kundali_{s['bundle_name'].rsplit('_', 1)[0]}.md"
    hd_input = json.load(open(bundle / "selemene/input.json"))
    hd_expected = json.load(open(bundle / "selemene/expected.json"))["expected"]
    hd_engine = json.load(open(bundle / "engine/output.json"))["result"]

    # Pull placements as the dyadic format expects (planet/sign/house/degree/nakshatra/condition)
    placements = []
    for planet_name in ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu", "ketu"]:
        pp = kundali_json["planets"][planet_name.capitalize()]
        sign_clean = pp["sign"].split(" ")[0].lower()  # "Vrishabha (Taurus)" -> "vrishabha"
        placement = {
            "planet": planet_name,
            "sign": sign_clean,
            "sign_english": pp["sign"],
            "house": pp["house"],
            "degree": pp["deg_str"],
            "nakshatra": f"{pp['nakshatra']} P{pp['pada']}",
            "nakshatra_lord": pp["nakshatra_lord"],
            "condition": _build_condition(planet_name, pp, hd_engine),
        }
        if pp.get("retrograde"):
            placement["retrograde"] = True
        placements.append(placement)

    # Mahadasha block in the format used by chitra.json
    mahadasha_data = kundali_json["mahadashas"]
    current_md = next((m for m in mahadasha_data if m["is_current"]), mahadasha_data[0])
    next_idx = mahadasha_data.index(current_md) + 1
    next_md = mahadasha_data[next_idx] if next_idx < len(mahadasha_data) else None
    current_antardasha = None
    if current_md.get("antardashas"):
        from datetime import datetime, timezone as tz
        now = datetime.now(tz.utc)
        for ad in current_md["antardashas"]:
            ad_start = datetime.fromisoformat(ad["start"].replace("Z", "+00:00"))
            ad_end = datetime.fromisoformat(ad["end"].replace("Z", "+00:00"))
            if ad_start <= now <= ad_end:
                current_antardasha = ad
                break

    mahadasha_block = {
        "current_lord": current_md["lord"].lower(),
        "current_started_iso": current_md["start"],
        "current_ends_iso": current_md["end"],
        "current_antardasha": current_antardasha["lord"].lower() if current_antardasha else None,
        "current_antardasha_window": (
            f"{current_antardasha['start'][:10]} → {current_antardasha['end'][:10]}"
            if current_antardasha else None
        ),
        "next_lord": next_md["lord"].lower() if next_md else None,
        "next_starts_iso": next_md["start"] if next_md else None,
        "next_duration_years": next_md["duration_years"] if next_md else None,
        "yogas": [{"name": y["name"], "status": y["status"], "effect": y["effect"]} for y in kundali_json["yogas"]],
    }

    # HD block — pulled from Selemene engine + humdes expected
    hd_block = {
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
    }

    subject_json = {
        "source_path": str(kundali_md),
        "subject": s["subject_name"],
        "birth_date": hd_input["birth_data"]["date"],
        "birth_time": hd_input["birth_data"]["time"][:5],
        "birth_place": hd_input["options"].get("humdes_location_string"),
        "latitude": hd_input["birth_data"]["latitude"],
        "longitude": hd_input["birth_data"]["longitude"],
        "timezone": hd_input["birth_data"]["timezone"],
        "lagna": kundali_json["lagna"]["sign"].split(" ")[0].lower(),
        "lagna_degree": kundali_json["lagna"]["deg_str"],
        "lagna_nakshatra": f"{kundali_json['lagna']['nakshatra']} P{kundali_json['lagna']['pada']}",
        "lagna_lord": kundali_json["lagna"]["lord"].lower(),
        "atmakaraka": kundali_json["karakas"]["atmakaraka"].lower(),
        "darakaraka": kundali_json["karakas"]["darakaraka"].lower(),
        "birth_nakshatra": f"{kundali_json['planets']['Moon']['nakshatra']} P{kundali_json['planets']['Moon']['pada']}",
        "placements": placements,
        "mahadasha": mahadasha_block,
        "human_design": hd_block,
        "pancha_bhuta": kundali_json["pancha_bhuta"],
        "chart_assets": {
            "vedic_kundali_md": str(kundali_md),
            "_notes": "Vedic placements computed via pyswisseph Lahiri sidereal. HD data extracted from humdes.com + Selemene engine-human-design.",
        },
        "output_dir": s["output_dir"],
    }

    out_path = OUT_DIR / f"{s['idx']}_{s['slug']}.json"
    out_path.write_text(json.dumps(subject_json, indent=2, ensure_ascii=False), encoding="utf-8")
    return out_path


def _build_condition(planet_name: str, pp: dict, hd: dict) -> str:
    """Synthesize a short condition string the synastry mode can quote from."""
    bits = []
    if pp.get("dignity") and pp["dignity"] != "—":
        bits.append(pp["dignity"])
    if pp.get("retrograde"):
        bits.append("retrograde")
    bits.append(f"H{pp['house']}")
    bits.append(f"{pp['nakshatra']} P{pp['pada']} (lord {pp['nakshatra_lord']})")
    return "; ".join(bits)


def main() -> int:
    print(f"Output dir: {OUT_DIR}")
    for s in SUBJECTS:
        path = build_one(s)
        size = path.stat().st_size
        print(f"  {path.relative_to(OUT_DIR.parent)}  ({size} bytes)")
    print(f"\nReady for: node --import tsx scripts/integratedreading-mode.ts \\")
    print(f"  --mode partner-synastry \\")
    print(f"  --subjects-dir {OUT_DIR} \\")
    print(f"  --output-dir <723/synastry-anitha-nateshan>")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
