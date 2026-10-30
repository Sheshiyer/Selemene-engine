"""Assemble witness-alchemist-field.html from app.html + data.

Pipeline (run in order from this folder):
  python3 compute.py   # natal + design + one transit snapshot -> chart.json
  python3 daily.py     # daily transits 2026-10-30..2030-12-31 + events -> sky.json
  python3 extra.py     # astrocartography lines, daily MC + sidereal, dasha -> sky.json (adds land.txt)
  node -e "require('fs').writeFileSync('content.json',JSON.stringify(require('./content.js')))"
  python3 build.py     # -> witness-alchemist-field.html
Requires: pip install pyswisseph; npm i world-atlas@2.0.2 topojson-client@3.1.0 (then node land.js -> land.txt)
"""
import json

sky = json.load(open('sky.json'))
# Dasha events come from the antardasha table, not from hard-coded dates.
sky['events'] = [e for e in sky['events'] if e[1] != 'dasha']
for a in sky['ad']:
    if a[2] >= '2026-10-25':
        sky['events'].append([a[2], 'dasha', f"Antardasha begins: {a[0]} / {a[1]}",
                              'Jupiter' if a[0] == 'Jupiter' else 'Saturn', 0])
sky['events'].sort()

# Selemene is the source of truth for Vedic period boundaries: Jupiter Mahadasha begins 14 Sep 2026.
import glob, os
ENG = {}
for f in sorted(glob.glob('selemene/calc-*.json')):
    d = json.load(open(f)); eid = os.path.basename(f)[5:-5]
    if 'result' in d:
        ENG[eid] = {'id': eid, 'result': d['result'], 'at': d.get('calculated_at'),
                    'backend': (d.get('metadata') or {}).get('backend'),
                    'version': (d.get('metadata') or {}).get('engine_version'),
                    'prompt': d.get('witness_prompt')}
    else:
        ENG[eid] = {'id': eid, 'status': d.get('status'), 'error': d.get('error_code')}
vm = ENG.get('vimshottari', {}).get('result')
if vm:
    # Vimshottari is regenerated from the engine's own timeline so every boundary agrees with the engine.
    # The engine's figures imply 365.25-day years (Jupiter MD 16 y; its first pratyantardasha 103.89 d).
    from datetime import datetime, timedelta
    ORD = ['Ketu', 'Venus', 'Sun', 'Moon', 'Mars', 'Rahu', 'Jupiter', 'Saturn', 'Mercury']
    YRS = {'Ketu': 7, 'Venus': 20, 'Sun': 6, 'Moon': 10, 'Mars': 7, 'Rahu': 18, 'Jupiter': 16, 'Saturn': 19, 'Mercury': 17}
    YEAR = timedelta(days=365.25)
    dt = lambda s: datetime.fromisoformat(s.replace('Z', '+00:00'))
    ist = lambda d: (d + timedelta(hours=5, minutes=30)).strftime('%Y-%m-%d')
    T = vm['timeline']['mahadashas']; birth = dt(T[0]['start_date'])
    sky['dasha'] = [[m['planet'], ist(dt(m['start_date'])), ist(dt(m['end_date']))] for m in T]
    ads, exact = [], []
    for m in T:
        p, mend = m['planet'], dt(m['end_date']); full = YRS[p]; t = mend - YEAR * full
        for k in range(9):
            a = ORD[(ORD.index(p) + k) % 9]; alen = YEAR * (full * YRS[a] / 120); a0, a1 = t, t + alen; t = a1
            if a1 <= birth: continue
            pds, tp = [], a0
            for q in range(9):
                pp = ORD[(ORD.index(a) + q) % 9]; plen = alen * (YRS[pp] / 120); p0, p1 = tp, tp + plen; tp = p1
                if p1 > birth: pds.append([pp, ist(max(p0, birth))]); exact.append(('pd', p, a, pp, max(p0, birth), p1))
            ads.append([p, a, ist(max(a0, birth)), ist(a1), pds]); exact.append(('ad', p, a, None, max(a0, birth), a1))
    sky['ad'] = ads
    # check against the engine: the current period and every upcoming transition must match to the second
    cp = vm['current_period']; near = lambda x, y: abs((x - y).total_seconds()) < 2
    ok_md = any(near(dt(m['start_date']), dt(cp['mahadasha']['start'])) for m in T)
    ok_ad = any(e[0] == 'ad' and e[1] == cp['mahadasha']['planet'] and e[2] == cp['antardasha']['planet'] and near(e[4], dt(cp['antardasha']['start'])) and near(e[5], dt(cp['antardasha']['end'])) for e in exact)
    ok_pd = any(e[0] == 'pd' and e[3] == cp['pratyantardasha']['planet'] and near(e[4], dt(cp['pratyantardasha']['start'])) and near(e[5], dt(cp['pratyantardasha']['end'])) for e in exact)
    ok_up = all(any(near(e[4], dt(u['date'])) and (e[3] if e[0] == 'pd' else e[2]) == u['to_planet'] for e in exact) for u in vm.get('upcoming_transitions', []))
    assert ok_md and ok_ad and ok_pd and ok_up, ('vimshottari mismatch', ok_md, ok_ad, ok_pd, ok_up)
    print('vimshottari: current period and', len(vm.get('upcoming_transitions', [])), 'upcoming transitions match the engine')
    sky['events'] = [e for e in sky['events'] if e[1] != 'dasha']
    for a in sky['ad']:
        if '2026-10-25' <= a[2] <= sky['days'][-1]['d']:
            sky['events'].append([a[2], 'dasha', f"Antardasha begins: {a[0]} / {a[1]}", 'Jupiter' if a[0] == 'Jupiter' else 'Saturn', 0])
    sky['events'].sort()

dump = lambda o: json.dumps(o, separators=(',', ':'), ensure_ascii=False).replace('</', '<\\/')
page = open('app.html').read()
out = (page.replace('%%SKY%%', dump(sky))
           .replace('%%CONTENT%%', dump(json.load(open('content.json'))))
           .replace('%%CSS%%', open('field.css').read())
           .replace('%%ENGINES%%', dump(ENG))
           .replace('%%DICT%%', open('dict.js').read())
           .replace('%%LIB%%', open('icons.js').read() + '\n' + open('kit3d.js').read() + '\n' + open('daylens.js').read())
           .replace('%%ROOMS%%', '\n'.join(open(f).read() for f in ('room-sky.js', 'room-engines.js', 'room-mandala.js', 'loom3d.js', 'arm3d.js', 'room-jyotish.js', 'dial3d.js', 'room-today.js'))))
open('witness-alchemist-field.html', 'w').write(out)
print('built', len(out), 'bytes')
