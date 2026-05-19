"""Compute a Vedic Kundali markdown for each parent using Swiss Ephemeris
(Lahiri sidereal ayanamsa, matching Selemene's engine-vimshottari + Indian
astrological convention).

For each parent it writes:
    ~/Downloads/humdes-extractor/parents/<slug>/inputs/Kundali_<Name>.md

Contents per file:
  - Birth data (date, time, place, lat, lng, timezone)
  - Lagna (Ascendant) + Lagna nakshatra/pada + Lagna lord
  - 9 grahas (Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu)
    each with sidereal longitude, sign, house, nakshatra, pada, retrograde,
    condition (own sign / exalted / debilitated / friendly / enemy)
  - Atmakaraka + Darakaraka (Jaimini)
  - Pancha Bhuta distribution (element count from rashi placements)
  - Major Yogas detected (Raja, Vipreet Raja, Saraswati, Gajakesari, Hamsa,
    Malavya, Sasa, Ruchaka, Bhadra, Chandra-Mangala, etc.)
  - Vimshottari Mahadasha timeline (9 periods, 120 years, with antardashas
    for the active period)
  - Bedrock summary (the key structural facts the synthesis should ground in)

Designed to be fed to witness-agents' integratedreading.ts via the
`source_reading_path` field in the per-parent config.

Usage:
    python3 compute_vedic_kundali.py
"""

from __future__ import annotations

import json
import math
import re
from dataclasses import dataclass, field
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Optional

import swisseph as swe

# ─── Constants ──────────────────────────────────────────────────────────

# Lahiri = Chitra-paksha ayanamsa — Indian govt standard, what Selemene's
# vimshottari engine + every "Vedic" calculator agrees on.
swe.set_sid_mode(swe.SIDM_LAHIRI, 0, 0)

SIGNS = [
    "Mesha (Aries)", "Vrishabha (Taurus)", "Mithuna (Gemini)", "Karka (Cancer)",
    "Simha (Leo)", "Kanya (Virgo)", "Tula (Libra)", "Vrishchika (Scorpio)",
    "Dhanu (Sagittarius)", "Makara (Capricorn)", "Kumbha (Aquarius)", "Meena (Pisces)",
]
SIGN_LORDS = ["Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury",
              "Venus", "Mars", "Jupiter", "Saturn", "Saturn", "Jupiter"]
SIGN_ELEMENTS = ["Fire", "Earth", "Air", "Water", "Fire", "Earth",
                 "Air", "Water", "Fire", "Earth", "Air", "Water"]

# 27 Nakshatras, each 13°20' = 13.333°
NAKSHATRAS = [
    ("Ashwini",          "Ketu",    "Ashwini Kumars"),
    ("Bharani",          "Venus",   "Yama"),
    ("Krittika",         "Sun",     "Agni"),
    ("Rohini",           "Moon",    "Brahma/Prajapati"),
    ("Mrigashira",       "Mars",    "Soma/Chandra"),
    ("Ardra",            "Rahu",    "Rudra"),
    ("Punarvasu",        "Jupiter", "Aditi"),
    ("Pushya",           "Saturn",  "Brihaspati"),
    ("Ashlesha",         "Mercury", "Nagas"),
    ("Magha",            "Ketu",    "Pitris"),
    ("Purva Phalguni",   "Venus",   "Bhaga"),
    ("Uttara Phalguni",  "Sun",     "Aryaman"),
    ("Hasta",            "Moon",    "Savitar"),
    ("Chitra",           "Mars",    "Vishvakarma/Tvashtar"),
    ("Swati",            "Rahu",    "Vayu"),
    ("Vishakha",         "Jupiter", "Indra-Agni"),
    ("Anuradha",         "Saturn",  "Mitra"),
    ("Jyeshta",          "Mercury", "Indra"),
    ("Mula",             "Ketu",    "Nirriti"),
    ("Purva Ashadha",    "Venus",   "Apas"),
    ("Uttara Ashadha",   "Sun",     "Vishvedevas"),
    ("Shravana",         "Moon",    "Vishnu"),
    ("Dhanishta",        "Mars",    "Eight Vasus"),
    ("Shatabhisha",      "Rahu",    "Varuna"),
    ("Purva Bhadrapada", "Jupiter", "Aja Ekapada"),
    ("Uttara Bhadrapada","Saturn",  "Ahir Budhnya"),
    ("Revati",           "Mercury", "Pushan"),
]

# Vimshottari Mahadasha sequence + years
DASHA_SEQUENCE = ["Ketu", "Venus", "Sun", "Moon", "Mars",
                  "Rahu", "Jupiter", "Saturn", "Mercury"]
DASHA_YEARS = {"Ketu": 7, "Venus": 20, "Sun": 6, "Moon": 10, "Mars": 7,
               "Rahu": 18, "Jupiter": 16, "Saturn": 19, "Mercury": 17}
TOTAL_DASHA_YEARS = sum(DASHA_YEARS.values())  # 120
# Nakshatra → starting mahadasha lord
NAKSHATRA_LORD_BY_NUM = ["Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu",
                         "Jupiter", "Saturn", "Mercury"] * 3  # 27

# Planet codes for Swiss Ephemeris
SWE_PLANETS = {
    "Sun":     swe.SUN,
    "Moon":    swe.MOON,
    "Mars":    swe.MARS,
    "Mercury": swe.MERCURY,
    "Jupiter": swe.JUPITER,
    "Venus":   swe.VENUS,
    "Saturn":  swe.SATURN,
    "Rahu":    swe.MEAN_NODE,   # Vedic uses Mean Node for Rahu (true node also OK)
}

# Dignity table: own sign / exalted (deg) / debilitated (deg) / mooltrikona
DIGNITY = {
    # planet : (own_signs[], exalted_sign, deb_sign)
    "Sun":     (["Simha (Leo)"], "Mesha (Aries)", "Tula (Libra)"),
    "Moon":    (["Karka (Cancer)"], "Vrishabha (Taurus)", "Vrishchika (Scorpio)"),
    "Mars":    (["Mesha (Aries)", "Vrishchika (Scorpio)"], "Makara (Capricorn)", "Karka (Cancer)"),
    "Mercury": (["Mithuna (Gemini)", "Kanya (Virgo)"], "Kanya (Virgo)", "Meena (Pisces)"),
    "Jupiter": (["Dhanu (Sagittarius)", "Meena (Pisces)"], "Karka (Cancer)", "Makara (Capricorn)"),
    "Venus":   (["Vrishabha (Taurus)", "Tula (Libra)"], "Meena (Pisces)", "Kanya (Virgo)"),
    "Saturn":  (["Makara (Capricorn)", "Kumbha (Aquarius)"], "Tula (Libra)", "Mesha (Aries)"),
    # Nodes: no exaltation/debility in classical Parashara — some traditions use Vrishabha (Rahu exalt)
    "Rahu":    ([], None, None),
    "Ketu":    ([], None, None),
}


# ─── Helpers ────────────────────────────────────────────────────────────

def parse_birth(birth_date: str, birth_time: str, tz: str) -> tuple[datetime, float]:
    """Return (utc_datetime, julian_day_ut)."""
    # Parse local time then convert via tz offset.
    import zoneinfo
    tzinfo = zoneinfo.ZoneInfo(tz)
    local_dt = datetime.fromisoformat(f"{birth_date}T{birth_time}").replace(tzinfo=tzinfo)
    utc_dt = local_dt.astimezone(timezone.utc)
    jd_ut = swe.julday(
        utc_dt.year, utc_dt.month, utc_dt.day,
        utc_dt.hour + utc_dt.minute / 60 + utc_dt.second / 3600,
    )
    return utc_dt, jd_ut


def sidereal_longitude(jd_ut: float, planet_code: int) -> tuple[float, bool]:
    """Return (sidereal_longitude_deg, retrograde)."""
    flags = swe.FLG_SIDEREAL | swe.FLG_SPEED
    pos, _ = swe.calc_ut(jd_ut, planet_code, flags)
    lon = pos[0] % 360
    retro = pos[3] < 0  # negative speed = retrograde
    return lon, retro


def sign_for(longitude: float) -> tuple[int, str, str, str]:
    """Return (sign_index_0_11, sign_name, lord, element)."""
    idx = int(longitude // 30) % 12
    return idx, SIGNS[idx], SIGN_LORDS[idx], SIGN_ELEMENTS[idx]


def nakshatra_for(longitude: float) -> tuple[int, str, int, str, str]:
    """Return (nakshatra_num_1_27, name, pada_1_4, lord, deity)."""
    nak_num_0 = int(longitude // (360 / 27)) % 27
    name, lord, deity = NAKSHATRAS[nak_num_0]
    # pada = quarter within the nakshatra (3°20' each = 360/108)
    within = longitude % (360 / 27)
    pada = int(within // (360 / 108)) + 1
    return nak_num_0 + 1, name, pada, lord, deity


def deg_min_sec(longitude: float) -> str:
    """Format absolute longitude within sign as 'DD°MM'SS\"'."""
    within = longitude % 30
    d = int(within)
    m_full = (within - d) * 60
    m = int(m_full)
    s = int(round((m_full - m) * 60))
    return f"{d}°{m:02d}'{s:02d}\""


def lagna_for(jd_ut: float, lat: float, lng: float) -> float:
    """Return sidereal ascendant in degrees [0, 360)."""
    flags = swe.FLG_SIDEREAL
    cusps, ascmc = swe.houses_ex(jd_ut, lat, lng, b'P', flags)
    # ascmc[0] = ascendant in sidereal degrees
    return ascmc[0] % 360


def house_for(longitude: float, lagna_long: float) -> int:
    """Whole-sign house (1-12) — the Indian convention."""
    lagna_sign = int(lagna_long // 30)
    planet_sign = int(longitude // 30)
    return ((planet_sign - lagna_sign) % 12) + 1


def dignity(planet: str, sign_name: str) -> str:
    if planet not in DIGNITY:
        return "—"
    own_signs, exalt, deb = DIGNITY[planet]
    if exalt and sign_name == exalt:
        return "Exalted"
    if deb and sign_name == deb:
        return "Debilitated"
    if sign_name in own_signs:
        return "Own sign"
    return "—"


# ─── Mahadasha ──────────────────────────────────────────────────────────

@dataclass
class Mahadasha:
    lord: str
    start: datetime
    end: datetime
    duration_years: float
    is_current: bool = False
    antardashas: list = field(default_factory=list)


def compute_mahadashas(moon_longitude: float, birth_dt_utc: datetime) -> list[Mahadasha]:
    """Vimshottari from Moon's nakshatra position."""
    nak_num, nak_name, _, _, _ = nakshatra_for(moon_longitude)
    nak_lord = NAKSHATRA_LORD_BY_NUM[nak_num - 1]

    # Balance of starting dasha (how much remains of birth-nakshatra lord's period)
    nak_size = 360 / 27
    moon_within_nak = moon_longitude % nak_size
    consumed_frac = moon_within_nak / nak_size
    remaining_frac = 1 - consumed_frac
    balance_years = DASHA_YEARS[nak_lord] * remaining_frac

    # Walk the sequence starting at nak_lord
    start_idx = DASHA_SEQUENCE.index(nak_lord)

    out: list[Mahadasha] = []
    cursor = birth_dt_utc - timedelta(days=DASHA_YEARS[nak_lord] * 365.25 * consumed_frac)
    # First dasha at birth started at cursor; its end is birth_dt_utc + balance_years
    first_end = birth_dt_utc + timedelta(days=balance_years * 365.25)
    out.append(Mahadasha(nak_lord,
                         cursor,
                         first_end,
                         DASHA_YEARS[nak_lord]))

    period_start = first_end
    for i in range(1, 9):
        lord = DASHA_SEQUENCE[(start_idx + i) % 9]
        years = DASHA_YEARS[lord]
        period_end = period_start + timedelta(days=years * 365.25)
        out.append(Mahadasha(lord, period_start, period_end, years))
        period_start = period_end

    # Mark which dasha is currently active (as of today UTC)
    now = datetime.now(timezone.utc)
    for d in out:
        if d.start <= now <= d.end:
            d.is_current = True
            d.antardashas = compute_antardashas(d)
    return out


def compute_antardashas(md: Mahadasha) -> list[dict]:
    """9 sub-periods of a mahadasha, each proportional to dasha years × 1/120 of M.D. years."""
    start = md.start
    duration_days = (md.end - md.start).total_seconds() / 86400
    out = []
    md_idx = DASHA_SEQUENCE.index(md.lord)
    for i in range(9):
        lord = DASHA_SEQUENCE[(md_idx + i) % 9]
        share = DASHA_YEARS[lord] / TOTAL_DASHA_YEARS
        ad_days = duration_days * share
        ad_end = start + timedelta(days=ad_days)
        out.append({
            "lord": lord,
            "start": start.isoformat(),
            "end": ad_end.isoformat(),
            "duration_months": round(ad_days / 30.4375, 1),
        })
        start = ad_end
    return out


# ─── Atmakaraka / Darakaraka ────────────────────────────────────────────

def compute_jaimini_karakas(planets: dict) -> dict:
    """Atmakaraka = graha at highest degree-within-sign; Darakaraka = lowest.
    Note: classical Jaimini uses 7 grahas (excluding Ketu, sometimes Rahu).
    """
    candidates = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"]
    by_within = {p: planets[p]["longitude_in_sign"] for p in candidates}
    # Sort descending — highest = atmakaraka
    ranked = sorted(by_within.items(), key=lambda kv: -kv[1])
    return {
        "atmakaraka": ranked[0][0],
        "darakaraka": ranked[-1][0],
        "ranked": ranked,
    }


# ─── Yoga detection (basic, expandable) ─────────────────────────────────

def detect_yogas(planets: dict, lagna_sign_idx: int, houses: dict) -> list[dict]:
    """Detect a few prominent classical yogas. Not exhaustive — gives the
    synthesis phase enough yoga-vocabulary to anchor in."""
    out = []
    moon_house = houses["Moon"]
    jup_house = houses["Jupiter"]

    # Gajakesari Yoga — Jupiter in kendra (1/4/7/10) from Moon
    diff = (jup_house - moon_house) % 12
    if diff in (0, 3, 6, 9):
        out.append({
            "name": "Gajakesari Yoga",
            "status": "active",
            "effect": "Jupiter in kendra from Moon — wisdom-graced intelligence, recognition through gravitas",
        })

    # Chandra-Mangala — Moon + Mars in same sign or aspect
    if planets["Moon"]["sign_index"] == planets["Mars"]["sign_index"]:
        out.append({
            "name": "Chandra-Mangala Yoga",
            "status": "active",
            "effect": "Moon and Mars conjunction — entrepreneurial heat, mother-figure with martial energy",
        })

    # Saraswati Yoga — Jupiter + Venus + Mercury all in kendra or trikona, none debilitated
    saraswati_houses = {1, 2, 4, 5, 7, 9, 10}
    if all(houses[p] in saraswati_houses for p in ("Jupiter", "Venus", "Mercury")):
        debilitated = any(planets[p]["dignity"] == "Debilitated" for p in ("Jupiter", "Venus", "Mercury"))
        if not debilitated:
            out.append({
                "name": "Saraswati Yoga",
                "status": "active",
                "effect": "Jupiter+Venus+Mercury in kendras/trikonas — gifts of speech, learning, art",
            })

    # Hamsa Yoga (Pancha Mahapurusha) — Jupiter exalted/own in kendra (1/4/7/10)
    if planets["Jupiter"]["dignity"] in ("Exalted", "Own sign") and houses["Jupiter"] in (1, 4, 7, 10):
        out.append({
            "name": "Hamsa Yoga",
            "status": "active",
            "effect": "Jupiter in dignity in a kendra — pure dharmic authority, scholar's gravitas",
        })

    # Malavya Yoga — Venus exalted/own in kendra
    if planets["Venus"]["dignity"] in ("Exalted", "Own sign") and houses["Venus"] in (1, 4, 7, 10):
        out.append({
            "name": "Malavya Yoga",
            "status": "active",
            "effect": "Venus in dignity in a kendra — beauty-blessed, relational grace, aesthetic authority",
        })

    # Sasa Yoga — Saturn exalted/own in kendra
    if planets["Saturn"]["dignity"] in ("Exalted", "Own sign") and houses["Saturn"] in (1, 4, 7, 10):
        out.append({
            "name": "Sasa Yoga",
            "status": "active",
            "effect": "Saturn in dignity in a kendra — capacity to govern, structural patience, longevity",
        })

    # Ruchaka Yoga — Mars exalted/own in kendra
    if planets["Mars"]["dignity"] in ("Exalted", "Own sign") and houses["Mars"] in (1, 4, 7, 10):
        out.append({
            "name": "Ruchaka Yoga",
            "status": "active",
            "effect": "Mars in dignity in a kendra — commander's frame, decisive action, physical vitality",
        })

    # Bhadra Yoga — Mercury exalted/own in kendra
    if planets["Mercury"]["dignity"] in ("Exalted", "Own sign") and houses["Mercury"] in (1, 4, 7, 10):
        out.append({
            "name": "Bhadra Yoga",
            "status": "active",
            "effect": "Mercury in dignity in a kendra — communication mastery, agile intellect, business gift",
        })

    # Vipreet Raja Yoga — lords of dusthanas (6/8/12) sit in dusthanas (6/8/12) together
    dust_houses = (6, 8, 12)
    # House lords of 6/8/12 from lagna
    lagna_sign = SIGNS[lagna_sign_idx]
    lord_of_6 = SIGN_LORDS[(lagna_sign_idx + 5) % 12]
    lord_of_8 = SIGN_LORDS[(lagna_sign_idx + 7) % 12]
    lord_of_12 = SIGN_LORDS[(lagna_sign_idx + 11) % 12]
    dust_lords = {lord_of_6, lord_of_8, lord_of_12}
    placements_in_dusthanas = [
        p for p in dust_lords
        if p in houses and houses[p] in dust_houses
    ]
    if len(placements_in_dusthanas) >= 2:
        out.append({
            "name": "Vipreet Raja Yoga (partial)",
            "status": "active",
            "effect": f"Lords of dusthanas ({', '.join(placements_in_dusthanas)}) placed in dusthanas — "
                      "the inverted-king yoga: difficulty itself becomes the path to rise",
        })

    # Kemadruma — Moon with no planet on either side (2nd or 12th from Moon) AND no planet conjunct
    moon_sign = planets["Moon"]["sign_index"]
    adjacent_or_with_moon = False
    for other in ("Sun", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"):
        s = planets[other]["sign_index"]
        if s == moon_sign or s == (moon_sign + 1) % 12 or s == (moon_sign - 1) % 12:
            adjacent_or_with_moon = True
            break
    if not adjacent_or_with_moon:
        out.append({
            "name": "Kemadruma Yoga (caution)",
            "status": "active",
            "effect": "Moon isolated (no graha in conjunction or in 2nd/12th from Moon) — "
                      "tendency toward emotional self-isolation; cancelled if benefic aspects exist",
        })

    return out


# ─── Pancha Bhuta distribution ──────────────────────────────────────────

def pancha_bhuta_distribution(planets: dict) -> dict:
    """Count of grahas (excluding Rahu/Ketu by default) in each element-sign group."""
    counts = {"Fire": 0, "Earth": 0, "Air": 0, "Water": 0}
    for p, data in planets.items():
        if p in ("Rahu", "Ketu"):
            continue
        counts[data["element"]] = counts.get(data["element"], 0) + 1
    # Vedic also speaks of Ether/Akasha — represented by Lagna/Atmakaraka rather than a graha-count
    counts["Ether"] = "carried by Lagna + Atmakaraka (not a graha-count metric)"
    return counts


# ─── Per-parent driver ──────────────────────────────────────────────────

def compute_kundali(name: str, birth_date: str, birth_time: str,
                    timezone_iana: str, latitude: float, longitude: float,
                    place: str) -> dict:
    utc_dt, jd_ut = parse_birth(birth_date, birth_time, timezone_iana)

    # Planet positions
    planets: dict = {}
    for name_p, code in SWE_PLANETS.items():
        lon, retro = sidereal_longitude(jd_ut, code)
        sign_idx, sign_name, sign_lord, element = sign_for(lon)
        nak_num, nak_name, pada, nak_lord, nak_deity = nakshatra_for(lon)
        planets[name_p] = {
            "longitude": lon,
            "longitude_in_sign": lon % 30,
            "deg_str": deg_min_sec(lon),
            "sign_index": sign_idx,
            "sign": sign_name,
            "sign_lord": sign_lord,
            "element": element,
            "nakshatra_num": nak_num,
            "nakshatra": nak_name,
            "pada": pada,
            "nakshatra_lord": nak_lord,
            "nakshatra_deity": nak_deity,
            "retrograde": retro,
        }

    # Ketu = 180° from Rahu
    rahu_lon = planets["Rahu"]["longitude"]
    ketu_lon = (rahu_lon + 180) % 360
    sign_idx, sign_name, sign_lord, element = sign_for(ketu_lon)
    nak_num, nak_name, pada, nak_lord, nak_deity = nakshatra_for(ketu_lon)
    planets["Ketu"] = {
        "longitude": ketu_lon,
        "longitude_in_sign": ketu_lon % 30,
        "deg_str": deg_min_sec(ketu_lon),
        "sign_index": sign_idx,
        "sign": sign_name,
        "sign_lord": sign_lord,
        "element": element,
        "nakshatra_num": nak_num,
        "nakshatra": nak_name,
        "pada": pada,
        "nakshatra_lord": nak_lord,
        "nakshatra_deity": nak_deity,
        "retrograde": True,  # nodes are always retro in Vedic
    }

    # Lagna
    lagna_lon = lagna_for(jd_ut, latitude, longitude)
    lagna_sign_idx, lagna_sign, lagna_lord, lagna_element = sign_for(lagna_lon)
    lagna_nak_num, lagna_nak, lagna_pada, lagna_nak_lord, lagna_deity = nakshatra_for(lagna_lon)

    # Houses (whole-sign)
    houses = {p: house_for(planets[p]["longitude"], lagna_lon) for p in planets}
    for p in planets:
        planets[p]["house"] = houses[p]
        planets[p]["dignity"] = dignity(p, planets[p]["sign"])

    # Mahadasha
    mahadashas = compute_mahadashas(planets["Moon"]["longitude"], utc_dt)

    # Karakas
    karakas = compute_jaimini_karakas(planets)

    # Yogas
    yogas = detect_yogas(planets, lagna_sign_idx, houses)

    # Pancha Bhuta
    bhutas = pancha_bhuta_distribution(planets)

    return {
        "name": name,
        "birth_date": birth_date,
        "birth_time": birth_time,
        "timezone": timezone_iana,
        "latitude": latitude,
        "longitude": longitude,
        "place": place,
        "ayanamsa": "Lahiri (Chitra-paksha)",
        "lagna": {
            "longitude": lagna_lon,
            "deg_str": deg_min_sec(lagna_lon),
            "sign": lagna_sign,
            "sign_index": lagna_sign_idx,
            "lord": lagna_lord,
            "element": lagna_element,
            "nakshatra": lagna_nak,
            "pada": lagna_pada,
            "nakshatra_lord": lagna_nak_lord,
            "deity": lagna_deity,
        },
        "planets": planets,
        "houses": houses,
        "karakas": karakas,
        "yogas": yogas,
        "pancha_bhuta": bhutas,
        "mahadashas": [
            {
                "lord": m.lord,
                "start": m.start.isoformat(),
                "end": m.end.isoformat(),
                "duration_years": m.duration_years,
                "is_current": m.is_current,
                "antardashas": m.antardashas if m.is_current else [],
            }
            for m in mahadashas
        ],
    }


# ─── Markdown writer ────────────────────────────────────────────────────

def render_kundali_md(k: dict) -> str:
    p = k["planets"]
    l = k["lagna"]
    lines: list[str] = []
    push = lines.append

    push(f"# Vedic Kundali — {k['name']}\n")
    push(f"## Birth\n")
    push(f"- Date: {k['birth_date']}")
    push(f"- Time (local): {k['birth_time']}")
    push(f"- Place: {k['place']}")
    push(f"- Coordinates: {k['latitude']:.4f}, {k['longitude']:.4f}")
    push(f"- Timezone: {k['timezone']}")
    push(f"- Ayanamsa: {k['ayanamsa']}\n")

    push(f"## Lagna (Ascendant)\n")
    push(f"- Sign: **{l['sign']}** at {l['deg_str']}")
    push(f"- Lagna lord: {l['lord']}")
    push(f"- Lagna nakshatra: **{l['nakshatra']}** Pada {l['pada']} "
         f"(lord: {l['nakshatra_lord']}, deity: {l['deity']})")
    push(f"- Element: {l['element']}\n")

    push(f"## Planetary placements (Rasi / D1)\n")
    push(f"| Graha | Sign | Degree | House | Nakshatra (Pada) | Nakshatra lord | Dignity | Retro |")
    push(f"|---|---|---|---|---|---|---|---|")
    order = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"]
    for planet in order:
        pp = p[planet]
        push(f"| {planet} | {pp['sign']} | {pp['deg_str']} | {pp['house']} | "
             f"{pp['nakshatra']} ({pp['pada']}) | {pp['nakshatra_lord']} | "
             f"{pp['dignity']} | {'R' if pp['retrograde'] else ''} |")
    push("")

    push(f"## Jaimini karakas (Char karaka, top + bottom)\n")
    ranked = k['karakas']['ranked']
    push(f"- **Atmakaraka** (soul significator): **{k['karakas']['atmakaraka']}** at {p[k['karakas']['atmakaraka']]['deg_str']} in {p[k['karakas']['atmakaraka']]['sign']}, House {p[k['karakas']['atmakaraka']]['house']}")
    push(f"- **Darakaraka** (spouse significator): **{k['karakas']['darakaraka']}** at {p[k['karakas']['darakaraka']]['deg_str']} in {p[k['karakas']['darakaraka']]['sign']}, House {p[k['karakas']['darakaraka']]['house']}")
    push("- Full ranking (highest degree → lowest):")
    for planet, deg in ranked:
        push(f"  - {planet}: {deg:.2f}° within sign")
    push("")

    push(f"## Yogas detected\n")
    if k["yogas"]:
        for y in k["yogas"]:
            push(f"- **{y['name']}** ({y['status']}) — {y['effect']}")
    else:
        push("- (No major Pancha-Mahapurusha or named yogas detected at this scan depth)")
    push("")

    push(f"## Pancha Bhuta (element distribution across 7 grahas)\n")
    bhutas = k['pancha_bhuta']
    push(f"- Fire:  {bhutas['Fire']} (Sun in Fire-sign: {p['Sun']['element']=='Fire'} · Mars: {p['Mars']['element']=='Fire'})")
    push(f"- Earth: {bhutas['Earth']}")
    push(f"- Air:   {bhutas['Air']}")
    push(f"- Water: {bhutas['Water']}")
    push(f"- Ether: {bhutas['Ether']}")
    push("")

    push(f"## Vimshottari Mahadasha timeline (120 years)\n")
    push(f"| # | Lord | Start | End | Years | Active |")
    push(f"|---|---|---|---|---|---|")
    for i, m in enumerate(k["mahadashas"], 1):
        active_mark = "✦ ACTIVE" if m["is_current"] else ""
        start_short = m["start"][:10]
        end_short = m["end"][:10]
        push(f"| {i} | {m['lord']} | {start_short} | {end_short} | {m['duration_years']:.1f} | {active_mark} |")
    push("")

    # Active antardashas
    for m in k["mahadashas"]:
        if m["is_current"] and m["antardashas"]:
            push(f"### Active Mahadasha: **{m['lord']}** — Antardashas\n")
            push(f"| Lord | Start | End | Months |")
            push(f"|---|---|---|---|")
            for a in m["antardashas"]:
                push(f"| {a['lord']} | {a['start'][:10]} | {a['end'][:10]} | {a['duration_months']} |")
            push("")
            break

    push(f"## Bedrock — key structural facts\n")
    am = k['karakas']['atmakaraka']
    push(f"- Atmakaraka: **{am}** at {p[am]['deg_str']} in {p[am]['sign']}, House {p[am]['house']} ({p[am]['nakshatra']} Pada {p[am]['pada']}, lord {p[am]['nakshatra_lord']}). {p[am]['dignity']}.")
    push(f"- Lagna: **{l['sign']}** ({l['nakshatra']} Pada {l['pada']}, lord {l['nakshatra_lord']}).")
    push(f"- Sun: {p['Sun']['sign']} House {p['Sun']['house']}, {p['Sun']['nakshatra']} Pada {p['Sun']['pada']}.")
    push(f"- Moon: {p['Moon']['sign']} House {p['Moon']['house']}, {p['Moon']['nakshatra']} Pada {p['Moon']['pada']} (birth nakshatra — Mahadasha sequence starts from {p['Moon']['nakshatra_lord']}).")
    current_md = next((m for m in k["mahadashas"] if m["is_current"]), None)
    if current_md:
        push(f"- Active Mahadasha: **{current_md['lord']}** ({current_md['start'][:10]} → {current_md['end'][:10]}, {current_md['duration_years']:.1f} years).")
        next_idx = k["mahadashas"].index(current_md) + 1
        if next_idx < len(k["mahadashas"]):
            nm = k["mahadashas"][next_idx]
            push(f"- Next Mahadasha: **{nm['lord']}** starting {nm['start'][:10]} ({nm['duration_years']:.1f} years).")
    push(f"- Pancha Bhuta distribution (7 grahas): Fire {bhutas['Fire']} · Earth {bhutas['Earth']} · Air {bhutas['Air']} · Water {bhutas['Water']}.")
    if k["yogas"]:
        push(f"- Active yogas: {', '.join(y['name'] for y in k['yogas'])}.")
    push("")

    return "\n".join(lines) + "\n"


# ─── Main ───────────────────────────────────────────────────────────────

PARENTS = [
    {
        "slug": "father",
        "name": "Cumbipuram Subramaniam Nateshan",
        "birth_date": "1960-11-20",
        "birth_time": "17:15",
        "timezone": "Asia/Kolkata",
        "latitude": 12.9768,   # Bengaluru
        "longitude": 77.5901,
        "place": "Bengaluru, India",
        "bundle_dir": Path.home() / "Downloads/humdes-extractor/parents/Cumbipuram_Subramaniam_Nateshan_father",
    },
    {
        "slug": "mother",
        "name": "Anitha Nateshan",
        "birth_date": "1965-06-01",
        "birth_time": "00:35",
        "timezone": "Asia/Kolkata",
        "latitude": 12.9768,
        "longitude": 77.5901,
        "place": "Bengaluru, India",
        "bundle_dir": Path.home() / "Downloads/humdes-extractor/parents/Anitha_Nateshan_mother",
    },
    {
        "slug": "witnessalchemist",
        "name": "Sheshnarayan Cumbipuram Nateshan (WitnessAlchemist)",
        "birth_date": "1991-08-13",
        "birth_time": "13:31",
        "timezone": "Asia/Kolkata",
        "latitude": 12.97,
        "longitude": 77.59,
        "place": "Bengaluru, India",
        "bundle_dir": Path.home() / "Downloads/humdes-extractor/parents/Sheshnarayan_witnessalchemist",
    },
]


def main() -> int:
    for parent in PARENTS:
        print(f"\n=== {parent['name']} ({parent['slug']}) ===")
        kundali = compute_kundali(
            name=parent["name"],
            birth_date=parent["birth_date"],
            birth_time=parent["birth_time"],
            timezone_iana=parent["timezone"],
            latitude=parent["latitude"],
            longitude=parent["longitude"],
            place=parent["place"],
        )

        l = kundali["lagna"]
        p = kundali["planets"]
        print(f"  Lagna       : {l['sign']} ({l['nakshatra']} Pada {l['pada']})")
        print(f"  Sun         : {p['Sun']['sign']} House {p['Sun']['house']} {p['Sun']['nakshatra']} P{p['Sun']['pada']}")
        print(f"  Moon        : {p['Moon']['sign']} House {p['Moon']['house']} {p['Moon']['nakshatra']} P{p['Moon']['pada']}")
        print(f"  Atmakaraka  : {kundali['karakas']['atmakaraka']}")
        print(f"  Darakaraka  : {kundali['karakas']['darakaraka']}")
        print(f"  Yogas       : {', '.join(y['name'] for y in kundali['yogas']) or '(none)'}")
        current = next((m for m in kundali["mahadashas"] if m["is_current"]), None)
        if current:
            print(f"  Active MD   : {current['lord']} ({current['start'][:10]} → {current['end'][:10]})")

        # Write markdown
        md = render_kundali_md(kundali)
        inputs_dir = parent["bundle_dir"] / "inputs"
        inputs_dir.mkdir(parents=True, exist_ok=True)
        md_path = inputs_dir / f"Kundali_{parent['name'].replace(' ', '_')}.md"
        md_path.write_text(md, encoding="utf-8")

        # Also dump JSON for traceability
        json_path = inputs_dir / f"Kundali_{parent['name'].replace(' ', '_')}.json"
        json_path.write_text(json.dumps(kundali, indent=2, ensure_ascii=False, default=str), encoding="utf-8")

        print(f"  → {md_path.name} ({md_path.stat().st_size} bytes, {len(md.split())} words)")
        print(f"  → {json_path.name} ({json_path.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
