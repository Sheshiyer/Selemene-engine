"""Phase 1 normaliser — convert humdes bulk_fetch output into Selemene-engine
test fixtures.

For each reading captured by bulk_fetch_v2.py, this writes per-person:

    <SELEMENE>/tests/fixtures/humdes/
        readings/
            <type>/
                <reading_hash>_<slug>/
                    01_input.json        # EngineInput-compatible
                    01_expected.json     # humdes's authoritative answers
                    01_metadata.json     # raw humdes + provenance
                    (02_*, 03_*, ... if reading has multiple people)
        _index.json                      # flat index of every fixture
        _geocache.json                   # location string -> (lat, lng)

EngineInput schema (from noesis-core/src/types.rs):
    {
        "birth_data": { "name", "date" YYYY-MM-DD, "time" HH:MM:SS,
                        "latitude", "longitude", "timezone" IANA },
        "current_time": ISO 8601 UTC,
        "location": Coordinates | null,
        "precision": "Standard",
        "options": { humdes-specific metadata }
    }

Expected schema (reference_charts.json shape):
    {
        "birth_date", "birth_time", "name", "latitude", "longitude", "timezone",
        "expected": {
            "type", "profile", "authority",
            "personality_sun": {"gate", "line"},
            "personality_earth": {...},
            "design_sun": {...},
            "design_earth": {...},
            "variables": [...],
            "incarnation_cross": { "name", "gates": [num, num, num, num] },
            "active_channels": [...]   # filled in by Phase 2 from HTML
            "defined_centers": [...]   # filled in by Phase 2
        }
    }
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

import requests


SOURCE_ROOT = Path(__file__).resolve().parent / "output"
TARGET = Path(
    "/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/"
    "Selemene-engine/tests/fixtures/humdes"
)

# Stable pinned timestamp used when --stable-timestamp is set. Pinned to the
# date of the original 89-fixture import so re-runs of the normaliser don't
# create spurious per-fixture diffs from `current_time` / `normalised_at`.
STABLE_TS_ISO = "2026-05-16T00:00:00+00:00"

# Set by main() before process_reading() is called. None = use wall clock.
_PINNED_TS: str | None = None


def _now_iso() -> str:
    return _PINNED_TS if _PINNED_TS is not None else datetime.now(timezone.utc).isoformat()

# humdes -> Selemene HD type enum
TYPE_MAP = {
    "P":  "Projector",
    "G":  "Generator",
    "MG": "ManifestingGenerator",
    "M":  "Manifestor",
    "R":  "Reflector",
}

# humdes authority abbreviations -> Selemene Authority enum
AUTHORITY_MAP = {
    "Splen.":     "Splenic",
    "Splenic":    "Splenic",
    "Emot.":      "Emotional",
    "Emotional":  "Emotional",
    "Sacr.":      "Sacral",
    "Sacral":     "Sacral",
    "SelfProj.":  "GCenter",
    "Self-Proj.": "GCenter",
    "GCenter":    "GCenter",
    "Ego":        "Heart",
    "Heart":      "Heart",
    "Ment.":      "Mental",
    "Mental":     "Mental",
    "Lunar":      "Lunar",
    "":           "Lunar",  # Reflectors sometimes blank
    "None":       "Lunar",
}


def _slug(text: str, max_len: int = 50) -> str:
    text = re.sub(r"[^\w\-]+", "_", str(text or "x"))
    text = re.sub(r"_+", "_", text).strip("_")
    return text[:max_len] or "x"


def _find_latest_bulk2(source: Path) -> Path:
    candidates = [p for p in source.glob("*_bulk2") if (p / "readings").is_dir()]
    if not candidates:
        raise FileNotFoundError(f"No bulk2 output found under {source}")
    return max(candidates, key=lambda p: p.stat().st_mtime)


# ---------- Geocoding (Nominatim, rate-limited, cached) ----------

NOMINATIM_URL = "https://nominatim.openstreetmap.org/search"
NOMINATIM_HEADERS = {
    "User-Agent": "humdes-to-selemene/1.0 (offline normaliser; one-time geocode)"
}


class GeoCache:
    def __init__(self, path: Path):
        self.path = path
        self.cache: dict[str, dict] = {}
        if path.exists():
            try:
                self.cache = json.loads(path.read_text(encoding="utf-8"))
            except Exception:  # noqa: BLE001
                self.cache = {}
        self.last_call = 0.0

    def get(self, key: str) -> dict | None:
        return self.cache.get(key)

    def set(self, key: str, value: dict) -> None:
        self.cache[key] = value
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.write_text(
            json.dumps(self.cache, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )

    def lookup(self, query: str) -> dict | None:
        """Return {lat, lng, display_name} or None. Caches everything,
        including misses (so we don't retry endlessly)."""
        if not query or not query.strip():
            return None
        key = query.strip()
        cached = self.get(key)
        if cached is not None:
            return cached if cached else None  # {} sentinel = known miss

        # Throttle: 1.1 s minimum gap between Nominatim hits.
        delta = time.monotonic() - self.last_call
        if delta < 1.1:
            time.sleep(1.1 - delta)

        try:
            resp = requests.get(
                NOMINATIM_URL,
                params={"q": query, "format": "json", "limit": 1},
                headers=NOMINATIM_HEADERS,
                timeout=15,
            )
            self.last_call = time.monotonic()
            resp.raise_for_status()
            items = resp.json()
        except Exception as e:  # noqa: BLE001
            print(f"  [geocode] {query!r}: HTTP error {e}", file=sys.stderr)
            self.set(key, {})  # mark miss so we don't keep retrying this run
            return None

        if not items:
            self.set(key, {})
            return None

        item = items[0]
        result = {
            "lat": float(item["lat"]),
            "lng": float(item["lon"]),
            "display_name": item.get("display_name", query),
        }
        self.set(key, result)
        return result


# ---------- Normalisation helpers ----------

def _map_type(humdes_code: str | None) -> str | None:
    if not humdes_code:
        return None
    return TYPE_MAP.get(humdes_code.strip(), None)


def _map_authority(humdes_label: str | None) -> str | None:
    if not humdes_label:
        return AUTHORITY_MAP.get("", None)
    return AUTHORITY_MAP.get(humdes_label.strip(), None)


def _profile_struct(profile_num: list) -> dict | None:
    if not profile_num or len(profile_num) < 2:
        return None
    try:
        return {
            "conscious_line": int(profile_num[0]),
            "unconscious_line": int(profile_num[1]),
            "text": f"{int(profile_num[0])}/{int(profile_num[1])}",
        }
    except (TypeError, ValueError):
        return None


def _safe_int(v) -> int | None:
    try:
        return int(v)
    except (TypeError, ValueError):
        return None


# ---------- Per-reading processor ----------

def process_reading(
    reading_dir: Path,
    type_name: str,
    target_root: Path,
    geocache: GeoCache,
    index: list,
    skip_geocode: bool,
) -> int:
    """Process one reading folder. Writes per-person fixtures.
    Returns number of persons written."""
    ravecard_files = sorted(reading_dir.glob("01_GET_ravecard_*.json"))
    if not ravecard_files:
        return 0
    obj = json.loads(ravecard_files[0].read_text(encoding="utf-8"))
    rc_data = obj.get("data", {})
    ravecards = rc_data.get("ravecards", [])
    if not ravecards:
        return 0

    # Directory row (sun_d/earth_d/sun_p/earth_p gates etc. live here)
    row_file = reading_dir / "_row.json"
    row_data = {}
    if row_file.exists():
        try:
            row_data = json.loads(row_file.read_text(encoding="utf-8"))
        except Exception:  # noqa: BLE001
            row_data = {}

    reading_hash = rc_data.get("id") or reading_dir.name.split("_", 1)[0]
    reading_code = rc_data.get("code", "")
    reading_name = rc_data.get("name") or reading_dir.name

    slug = _slug(reading_name)
    out_dir = target_root / "readings" / type_name / f"{reading_hash}_{slug}"
    out_dir.mkdir(parents=True, exist_ok=True)

    written = 0
    for person_idx, person in enumerate(ravecards, start=1):
        prefix = f"{person_idx:02d}"

        # ----- Birth data -----
        date_iso = person.get("date")            # already YYYY-MM-DD
        time_str = person.get("time")            # HH:MM
        if time_str and time_str.count(":") == 1:
            time_str = f"{time_str}:00"          # HH:MM -> HH:MM:SS
        tz = person.get("timezone") or ""
        location_str = (person.get("location") or "").strip()
        location_id = person.get("location_id") or ""

        # Geocode
        geo = None
        if not skip_geocode and location_str:
            geo = geocache.lookup(location_str)

        latitude = geo["lat"] if geo else None
        longitude = geo["lng"] if geo else None

        # Map enums
        type_code = person.get("type") or ""
        type_full = _map_type(type_code)
        authority_label = (person.get("labelList", {}) or {}).get("authority", "")
        authority_full = _map_authority(authority_label)
        profile_num = (person.get("profile") or {}).get("num") or []
        profile_struct = _profile_struct(profile_num)
        variables = person.get("variables") or []
        cross_obj = person.get("cross") or {}
        cross_num = cross_obj.get("num") or []
        cross_name = cross_obj.get("name") or ""

        # ----- Build input.json -----
        engine_input = {
            "birth_data": {
                "name": person.get("name") or "Unknown",
                "date": date_iso,
                "time": time_str,
                "latitude": latitude,
                "longitude": longitude,
                "timezone": tz,
            },
            "current_time": _now_iso(),
            "location": (
                {"latitude": latitude, "longitude": longitude}
                if latitude is not None and longitude is not None
                else None
            ),
            "precision": "Standard",
            "options": {
                "humdes_reading_hash": reading_hash,
                "humdes_reading_code": reading_code,
                "humdes_reading_type": type_name,
                "humdes_person_index": person_idx,
                "humdes_person_id": person.get("id"),
                "humdes_link": rc_data.get("linkShort") or "",
                "humdes_type_code": type_code,
                "humdes_authority_short": authority_label,
                "humdes_variables": variables,
                "humdes_location_string": location_str,
                "humdes_location_id": location_id,
                "humdes_sex": person.get("sex"),
                "geocode_source": "nominatim" if geo else "none",
                "geocode_display_name": geo["display_name"] if geo else None,
            },
        }

        # Strip None values from birth_data (engine validators care)
        engine_input["birth_data"] = {
            k: v for k, v in engine_input["birth_data"].items() if v is not None
        }

        # ----- Build expected.json -----
        expected = {
            "name": person.get("name"),
            "birth_date": date_iso,
            "birth_time": time_str,
            "timezone": tz,
            "latitude": latitude,
            "longitude": longitude,
            "expected": {
                "type": type_full,
                "type_humdes_code": type_code,
                "profile": profile_struct,
                "authority": authority_full,
                "authority_humdes_label": authority_label,
                "variables": variables,
                "incarnation_cross": {
                    "name": cross_name,
                    "gates": [_safe_int(g) for g in cross_num],
                },
                # Gates live on _row.json (directory metadata) for the primary
                # person of the reading. Multi-person readings only have row
                # data for person[0]; later people fall back to null and would
                # need Phase-2 HTML parsing to recover.
                "personality_sun":  {
                    "gate": _safe_int(
                        person.get("sun_p")
                        or (row_data.get("sun_p") if person_idx == 1 else None)
                    )
                },
                "personality_earth": {
                    "gate": _safe_int(
                        person.get("earth_p")
                        or (row_data.get("earth_p") if person_idx == 1 else None)
                    )
                },
                "design_sun": {
                    "gate": _safe_int(
                        person.get("sun_d")
                        or (row_data.get("sun_d") if person_idx == 1 else None)
                    )
                },
                "design_earth": {
                    "gate": _safe_int(
                        person.get("earth_d")
                        or (row_data.get("earth_d") if person_idx == 1 else None)
                    )
                },
                # These will be enriched by Phase 2 (HTML extractor)
                "active_channels": None,
                "defined_centers": None,
                "active_gates": None,
                "definition": None,
                "strategy": None,
                "strategy_humdes_label": None,
                "not_self_theme": None,
                "not_self_humdes_label": None,
            },
        }

        # ----- Build metadata.json -----
        metadata = {
            "humdes_reading": {
                "hash": reading_hash,
                "code": reading_code,
                "name": reading_name,
                "type": type_name,
                "person_count": len(ravecards),
            },
            "person": {
                "index": person_idx,
                "humdes_person_id": person.get("id"),
                "label": person.get("label"),
                "raw": person,
            },
            "source_files": {
                "reading_dir": str(reading_dir.relative_to(SOURCE_ROOT.parent)),
                "ravecard_file": ravecard_files[0].name,
            },
            "captured_at": (obj.get("_meta") or {}).get("captured_at"),
            "normalised_at": _now_iso(),
        }

        (out_dir / f"{prefix}_input.json").write_text(
            json.dumps(engine_input, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        (out_dir / f"{prefix}_expected.json").write_text(
            json.dumps(expected, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )
        (out_dir / f"{prefix}_metadata.json").write_text(
            json.dumps(metadata, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )

        index.append({
            "type": type_name,
            "reading_hash": reading_hash,
            "reading_name": reading_name,
            "person_index": person_idx,
            "person_name": person.get("name"),
            "input": f"readings/{type_name}/{reading_hash}_{slug}/{prefix}_input.json",
            "expected": f"readings/{type_name}/{reading_hash}_{slug}/{prefix}_expected.json",
            "metadata": f"readings/{type_name}/{reading_hash}_{slug}/{prefix}_metadata.json",
            "has_coords": latitude is not None,
            "hd_type": type_full,
            "authority": authority_full,
            "profile": profile_struct["text"] if profile_struct else None,
        })

        written += 1

    return written


# ---------- Main ----------

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", type=Path, default=None,
                    help="bulk2 source folder (default: latest under ./output)")
    ap.add_argument("--target", type=Path, default=TARGET,
                    help=f"Selemene fixtures root (default: {TARGET})")
    ap.add_argument("--skip-geocode", action="store_true",
                    help="Do not call Nominatim; leave lat/long as null")
    ap.add_argument(
        "--stable-timestamp",
        action="store_true",
        help=(
            "Use a fixed pinned timestamp for `current_time` and "
            "`normalised_at`, instead of `datetime.now(UTC)`. This is what "
            "you want when regenerating fixtures for a PR — otherwise the "
            "timestamps create 89 spurious diff entries."
        ),
    )
    args = ap.parse_args()

    global _PINNED_TS
    if args.stable_timestamp:
        _PINNED_TS = STABLE_TS_ISO

    source = args.source or _find_latest_bulk2(SOURCE_ROOT)
    target_root = args.target

    print(f"Source : {source}")
    print(f"Target : {target_root}")
    target_root.mkdir(parents=True, exist_ok=True)

    geocache = GeoCache(target_root / "_geocache.json")
    index: list = []
    total_persons = 0

    types_root = source / "readings"
    if not types_root.exists():
        print(f"ERROR: {types_root} missing", file=sys.stderr)
        return 2

    for type_dir in sorted(types_root.iterdir()):
        if not type_dir.is_dir():
            continue
        type_name = type_dir.name
        print(f"\n[{type_name}]")
        for reading_dir in sorted(type_dir.iterdir()):
            if not reading_dir.is_dir():
                continue
            n = process_reading(
                reading_dir, type_name, target_root, geocache, index,
                args.skip_geocode,
            )
            print(f"  {reading_dir.name:60s} -> {n} person(s)")
            total_persons += n

    # Sort entries deterministically so re-runs of the normaliser produce
    # byte-stable _index.json (no spurious diffs in PRs). Sort key matches
    # the natural fixture path order: type, then reading_hash, then person.
    index.sort(key=lambda e: (
        e.get("type") or "",
        e.get("reading_hash") or "",
        e.get("person_index") or 0,
    ))

    (target_root / "_index.json").write_text(
        json.dumps({
            "source": str(source.name),
            "normalised_at": _now_iso(),
            "total_persons": total_persons,
            "entries": index,
        }, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )

    geocoded = sum(1 for e in index if e["has_coords"])
    print(f"\n=== Done ===")
    print(f"  Total persons written : {total_persons}")
    print(f"  With geocoded coords  : {geocoded}")
    print(f"  Without coords        : {total_persons - geocoded}")
    print(f"  Index                 : {target_root / '_index.json'}")
    print(f"  Geocache              : {target_root / '_geocache.json'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
