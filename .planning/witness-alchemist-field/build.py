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
    md_start = vm['current_period']['mahadasha']['start'][:10]
    for row in sky['dasha']:
        if row[0] == 'Jupiter' and row[1].startswith('2026'): row[1] = md_start
        if row[0] == 'Rahu' and row[2].startswith('2026'): row[2] = md_start
    for a in sky['ad']:
        if a[0] == 'Jupiter' and a[2].startswith('2026-09'): a[2] = md_start
        if a[0] == 'Rahu' and a[3].startswith('2026-09'): a[3] = md_start
        for pd in a[4]:
            if pd[1].startswith('2026-09-1'): pd[1] = md_start
    sky['events'] = [e for e in sky['events'] if e[1] != 'dasha']
    for a in sky['ad']:
        if a[2] >= '2026-10-25':
            sky['events'].append([a[2], 'dasha', f"Antardasha begins: {a[0]} / {a[1]}", 'Jupiter' if a[0] == 'Jupiter' else 'Saturn', 0])
    sky['events'].sort()

dump = lambda o: json.dumps(o, separators=(',', ':'), ensure_ascii=False).replace('</', '<\\/')
page = open('app.html').read()
out = (page.replace('%%SKY%%', dump(sky))
           .replace('%%CONTENT%%', dump(json.load(open('content.json'))))
           .replace('%%CSS%%', open('field.css').read())
           .replace('%%ENGINES%%', dump(ENG))
           .replace('%%DICT%%', open('dict.js').read())
           .replace('%%LIB%%', open('icons.js').read() + '\n' + open('kit3d.js').read())
           .replace('%%ROOMS%%', '\n'.join(open(f).read() for f in ('room-sky.js', 'room-engines.js', 'room-mandala.js', 'loom3d.js', 'arm3d.js', 'room-jyotish.js'))))
open('witness-alchemist-field.html', 'w').write(out)
print('built', len(out), 'bytes')
