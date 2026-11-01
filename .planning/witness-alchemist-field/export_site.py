"""Export the Witness Alchemist Field as a static site that people and agents can both read.

    python3 build.py        # first: witness-alchemist-field.html
    python3 export_site.py  # then: dist/

dist/
  index.html            the field (same page as the artifact)
  img/                  engine stills
  today.json, today.md  the current IST day, machine and prose
  daily/<date>.json|md  every day the engine has held (same shape as today.*)
  daily/index.json      the dates available, newest first
  llms.txt              where an agent starts
  _headers, _redirects  CORS on every file, /api/today -> today.json

Nothing here needs the Selemene key: it reads what sync_daily.py already saved.
"""
import datetime as dt
import glob
import json
import os
import pathlib
import shutil

HERE = pathlib.Path(__file__).resolve().parent
DIST = HERE / 'dist'
SITE = os.environ.get('FIELD_SITE', 'https://daily.10869.space').rstrip('/')
IST = dt.timezone(dt.timedelta(hours=5, minutes=30))
TODAY = dt.datetime.now(IST).date().isoformat()

SUBJECT = {
    'name': 'Shesh Iyer (Witness Alchemist)',
    'born': '1991-08-13T13:31:00+05:30, Bangalore 12.9716N 77.5946E',
    'type': 'Generator', 'strategy': 'To Respond', 'authority': 'Sacral', 'profile': '2/4',
    'definition': 'Split (wide split: needs 7+29, 8+29 or channel 20-34 to bridge)',
    'incarnation_cross': 'Right Angle Cross of Explanation 4/49 | 23/43',
    'channels': ['23-43 Structuring', '42-53 Maturation'],
    'defined_centers': ['Ajna', 'Throat', 'Sacral', 'Root'],
    'vedic': 'Lagna Scorpio 12 07 Anuradha p3, Moon Virgo 9 49 Uttara Phalguni p4 (Lahiri)',
}

KEEP = {
    'panchanga': ['tithi_name', 'nakshatra_name', 'yoga_name', 'karana_name', 'vara_name', 'tithi_value', 'nakshatra_value'],
    'transits': ['aspects', 'period_quality', 'retrograde_planets', 'sade_sati'],
    'vimshottari': ['current_period', 'upcoming_transitions'],
    'biorhythm': ['target_date', 'days_alive', 'physical', 'emotional', 'intellectual', 'intuitive', 'aesthetic', 'spiritual', 'overall_energy', 'critical_days'],
    'vedic-clock': ['calculated_for', 'hora', 'choghadiya', 'current_organ', 'current_dosha', 'vedic_day', 'upcoming_transitions'],
}
DEF_WORD = {0: 'none', 1: 'single', 2: 'split', 3: 'triple split', 4: 'quadruple split'}


def engine_day(raw):
    eng = {}
    for k, v in (raw.get('engines') or {}).items():
        env = v.get('result') or {}
        res = env.get('result') or {}
        eng[k] = {'result': {f: res[f] for f in KEEP.get(k, []) if f in res},
                  'calculated_at': env.get('calculated_at'),
                  'engine_version': (env.get('metadata') or {}).get('engine_version')}
    w = (raw.get('witness') or {}).get('result')
    return {'moment': raw.get('moment'), 'moment_anchor': raw.get('moment_anchor'), 'source': raw.get('source'),
            'fetched_at': raw.get('fetched_at'), 'complete': raw.get('complete'), 'missing': raw.get('missing', []),
            'engines': eng, 'witness_dyad': w}


def field_day(sky, date):
    day = next((d for d in sky['days'] if d['d'] == date), None)
    if not day:
        return None
    order = sky['order']
    natal_gates = {p[3] for p in sky['natal']}
    return {
        'transits': [{'planet': order[i], 'gate': p[0], 'line': p[1], 'retrograde': bool(p[2]), 'longitude': p[3],
                      'on_natal_gate': p[0] in natal_gates} for i, p in enumerate(day['p'])],
        'channels': [{'channel': c, 'made_by': s} for c, s in day['ch']],
        'defined_centers': day['cen'],
        'definition': DEF_WORD.get(day['def'], str(day['def'])),
        'motor_to_throat': bool(day['m2t']),
        'contacts': [{'transiting': c[0], 'natal_layer': 'Personality' if c[1] == 'P' else 'Design', 'natal': c[2], 'orb_deg': c[3]} for c in day['con']],
        'events': [{'date': e[0], 'kind': e[1], 'text': e[2], 'planet': e[3], 'gate': e[4]} for e in sky['events'] if e[0] == date],
        'sidereal_lahiri': dict(zip(['Moon', 'Sun', 'Jupiter', 'Saturn', 'Rahu', 'Mars'], day['sid'])),
    }


def doc(date, sky, raw):
    return {
        'schema': 'witness-alchemist-field/day/v1',
        'date': date, 'timezone': 'Asia/Kolkata', 'generated_at': dt.datetime.now(dt.timezone.utc).isoformat(),
        'subject': SUBJECT,
        'provenance': {'engine': 'Selemene engine responses stored for this day (https://selemene.tryambakam.space)',
                       'field': 'Swiss Ephemeris (Moshier) tropical Human Design computation in this bundle',
                       'rule': 'Witness, do not forecast. No prediction, no diagnosis.'},
        'engine': engine_day(raw) if raw else None,
        'field': field_day(sky, date),
        'links': {'page': f'{SITE}/#today', 'json': f'{SITE}/daily/{date}.json', 'markdown': f'{SITE}/daily/{date}.md',
                  'today_json': f'{SITE}/today.json', 'index': f'{SITE}/daily/index.json', 'llms': f'{SITE}/llms.txt'},
    }


def md(d):
    L = [f"# Witness Alchemist Field: {d['date']} (IST)", '',
         f"Subject: {SUBJECT['name']}. {SUBJECT['type']}, {SUBJECT['strategy']}, {SUBJECT['authority']} authority, profile {SUBJECT['profile']}, {SUBJECT['definition']}.", '',
         'Witness, do not forecast. Figures marked Engine are stored Selemene responses; Field figures are Swiss Ephemeris computations.', '']
    e = d.get('engine')
    if e:
        L += ['## Engine (Selemene)', '', f"Moment: {e['moment']} ({e['moment_anchor']}); source {e['source']}; complete: {e['complete']}."]
        if e['missing']:
            L += ['Missing: ' + '; '.join(f"{m['part']}: {m['reason']}" for m in e['missing'])]
        L += ['']
        r = e['engines'].get('panchanga', {}).get('result')
        if r:
            L += ['### Panchanga', '', f"- Tithi: {r.get('tithi_name')}", f"- Nakshatra: {r.get('nakshatra_name')}", f"- Yoga: {r.get('yoga_name')}", f"- Karana: {r.get('karana_name')}", f"- Vara: {r.get('vara_name')}", '']
        r = e['engines'].get('vimshottari', {}).get('result')
        if r and r.get('current_period'):
            cp = r['current_period']
            L += ['### Vimshottari', '', f"- Mahadasha {cp['mahadasha']['planet']} ({cp['mahadasha']['start'][:10]} to {cp['mahadasha']['end'][:10]})",
                  f"- Antardasha {cp['antardasha']['planet']} (to {cp['antardasha']['end'][:10]})",
                  f"- Pratyantardasha {cp['pratyantardasha']['planet']} (to {cp['pratyantardasha']['end'][:10]})", '']
        r = e['engines'].get('transits', {}).get('result')
        if r:
            L += ['### Transits (sidereal, to natal)', '', f"- Period quality: {r.get('period_quality')}", f"- Retrograde: {', '.join(r.get('retrograde_planets') or []) or 'none'}"]
            ss = r.get('sade_sati') or {}
            if ss:
                L += [f"- Sade Sati: {'active' if ss.get('is_active') or ss.get('active') else 'inactive'}"]
            for a in (r.get('aspects') or [])[:12]:
                L += [f"- {a['transiting_planet']} {a['aspect_type'].lower()} natal {a['natal_planet']}, orb {a['orb']} ({a['nature']})"]
            L += ['']
        r = e['engines'].get('biorhythm', {}).get('result')
        if r:
            L += ['### Biorhythm', '']
            for k in ('physical', 'emotional', 'intellectual', 'intuitive'):
                c = r.get(k) or {}
                if c:
                    L += [f"- {k.title()}: {c.get('phase')} ({c.get('percentage', 0):.0f}%)"]
            L += [f"- Overall energy: {r.get('overall_energy', 0):.0f}", '']
        r = e['engines'].get('vedic-clock', {}).get('result')
        if r:
            vd = r.get('vedic_day') or {}
            L += ['### Vedic clock (at the moment above)', '', f"- Sunrise {vd.get('sunrise')}, sunset {vd.get('sunset')}",
                  f"- Hora: {(r.get('hora') or {}).get('ruler')}", f"- Choghadiya: {(r.get('choghadiya') or {}).get('name')} ({(r.get('choghadiya') or {}).get('nature')})",
                  f"- Organ: {(r.get('current_organ') or {}).get('organ') if isinstance(r.get('current_organ'), dict) else r.get('current_organ')}; dosha: {(r.get('current_dosha') or {}).get('dosha') if isinstance(r.get('current_dosha'), dict) else r.get('current_dosha')}", '']
        if e.get('witness_dyad'):
            L += ['### Witness Dyad', '', '```json', json.dumps(e['witness_dyad'], indent=1)[:4000], '```', '']
    f = d.get('field')
    if f:
        L += ['## Field (Human Design, tropical)', '', f"- Definition: {f['definition']}; motor to Throat: {'yes' if f['motor_to_throat'] else 'no'}",
              f"- Defined centers: {', '.join(f['defined_centers'])}",
              '- Channels: ' + ', '.join(f"{c['channel']} ({c['made_by']})" for c in f['channels']), '', '| Planet | Gate.Line | R | Longitude |', '|---|---|---|---|']
        L += [f"| {t['planet']} | {t['gate']}.{t['line']}{' (natal gate)' if t['on_natal_gate'] else ''} | {'R' if t['retrograde'] else ''} | {t['longitude']} |" for t in f['transits']]
        if f['contacts']:
            L += ['', 'Contacts: ' + '; '.join(f"{c['transiting']} on {c['natal_layer']} {c['natal']} (orb {c['orb_deg']})" for c in f['contacts'])]
        if f['events']:
            L += ['', 'Events today: ' + '; '.join(ev['text'] for ev in f['events'])]
        L += ['']
    L += ['---', f"JSON: {d['links']['json']}  Page: {d['links']['page']}  Index: {d['links']['index']}"]
    return '\n'.join(L) + '\n'


def main():
    sky = json.load(open(HERE / 'sky.json'))
    if DIST.exists():
        shutil.rmtree(DIST)
    (DIST / 'daily').mkdir(parents=True)
    shutil.copy(HERE / 'witness-alchemist-field.html', DIST / 'index.html')
    shutil.copytree(HERE / 'img', DIST / 'img')
    dates = []
    for f in sorted(glob.glob(str(HERE / 'selemene' / 'daily' / '*.json'))):
        raw = json.load(open(f))
        d = doc(raw['date'], sky, raw)
        (DIST / 'daily' / f"{raw['date']}.json").write_text(json.dumps(d, separators=(',', ':'), ensure_ascii=False))
        (DIST / 'daily' / f"{raw['date']}.md").write_text(md(d))
        dates.append(raw['date'])
    dates.sort(reverse=True)
    today = TODAY if TODAY in dates else dates[0]
    for ext in ('json', 'md'):
        shutil.copy(DIST / 'daily' / f'{today}.{ext}', DIST / f'today.{ext}')
    (DIST / 'daily' / 'index.json').write_text(json.dumps({'schema': 'witness-alchemist-field/index/v1', 'today': today, 'generated_at': dt.datetime.now(dt.timezone.utc).isoformat(),
                                                           'days': [{'date': x, 'json': f'{SITE}/daily/{x}.json', 'markdown': f'{SITE}/daily/{x}.md'} for x in dates]}, indent=1))
    (DIST / 'llms.txt').write_text(f"""# Witness Alchemist Field

> A daily Human Design and Jyotish field for one subject ({SUBJECT['name']}), computed by the Selemene engine and the Swiss Ephemeris. Witness, do not forecast.

The HTML page at / is a heavy interactive instrument (three.js). Agents should read the data files instead.

## Today
- {SITE}/today.json : today's day (IST) as JSON, schema witness-alchemist-field/day/v1
- {SITE}/today.md : the same day as markdown
- {SITE}/api/today : alias of today.json

## Any day
- {SITE}/daily/index.json : every available date, newest first
- {SITE}/daily/YYYY-MM-DD.json and .md

## Shape of a day
- engine: stored Selemene responses (panchanga, transits, vimshottari, biorhythm, vedic-clock) and the Witness Dyad when the engine returned one; `missing` lists parts the engine did not return
- field: tropical Human Design transits (gate.line per planet), channels, definition, defined centers, contacts to natal planets, events of the day
- subject: the fixed natal facts

All responses carry Access-Control-Allow-Origin: * and refresh once a day after the Selemene engine has held the day (about 06:10 IST).
Rules for anyone writing from this data: no prediction, no diagnosis; name figures by provenance (Engine vs Field).
""")
    (DIST / 'robots.txt').write_text('User-agent: *\nAllow: /\n')
    (DIST / '_headers').write_text("/*\n  Access-Control-Allow-Origin: *\n  X-Content-Type-Options: nosniff\n/today.json\n  Cache-Control: public, max-age=300\n/today.md\n  Cache-Control: public, max-age=300\n/daily/*\n  Cache-Control: public, max-age=3600\n/*.md\n  Content-Type: text/markdown; charset=utf-8\n/llms.txt\n  Content-Type: text/plain; charset=utf-8\n")
    (DIST / '_redirects').write_text('/api/today /today.json 200\n/api/today.md /today.md 200\n/api/daily/:date /daily/:date.json 200\n')
    print('exported', today, 'days:', len(dates), 'to', DIST)


if __name__ == '__main__':
    main()
