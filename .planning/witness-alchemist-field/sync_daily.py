"""Pull the engine-held daily field into this bundle.

The Selemene engine owns the day: it computes the day-scoped engines and the Witness Dyad
and keeps them in `readings` (workflow_id "daily-field"). This script only reads that day
and saves it as selemene/daily/<date>.json for build.py to embed.

    python3 sync_daily.py                 # today (IST)
    python3 sync_daily.py --date 2026-10-30
    python3 sync_daily.py --back 6        # today and the six days before
    python3 sync_daily.py --force         # refetch even when a complete day is saved

Order of sources:
  1. GET  /api/v1/daily?date=D           the stored day
  2. POST /api/v1/daily/refresh          asks the engine to compute what is missing
  3. engine calls (until /daily is deployed): the same five engines through
     /engines/{id}/calculate, which stores each result in `readings`, plus
     /witness/interpret without a biofield capture.

The key comes from SELEMENE_API_KEY or the file named by SELEMENE_ENV_FILE
(default /Volumes/madara/sec-key/.env). It is never printed or written.
"""
import argparse
import datetime as dt
import json
import os
import pathlib
import sys
import urllib.error
import urllib.request

BASE = os.environ.get('SELEMENE_URL', 'https://selemene.tryambakam.space').rstrip('/') + '/api/v1'
DAY_ENGINES = ['panchanga', 'transits', 'vimshottari', 'biorhythm', 'vedic-clock']
IST = dt.timezone(dt.timedelta(hours=5, minutes=30))
OUT = pathlib.Path(__file__).resolve().parent / 'selemene' / 'daily'


def api_key():
    k = os.environ.get('SELEMENE_API_KEY')
    if k:
        return k.strip()
    env = pathlib.Path(os.environ.get('SELEMENE_ENV_FILE', '/Volumes/madara/sec-key/.env'))
    if env.exists():
        for line in env.read_text().splitlines():
            if line.startswith('SELEMENE_API_KEY='):
                return line.split('=', 1)[1].strip().strip('"\'')
    sys.exit('sync_daily: no SELEMENE_API_KEY in the environment or the env file')


def call(method, path, key, body=None, timeout=60):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(BASE + path, data=data, method=method,
                                 headers={'X-API-Key': key, 'Content-Type': 'application/json',
                                          'User-Agent': 'witness-alchemist-field-sync/1.0'})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return r.status, json.loads(r.read() or b'null')
    except urllib.error.HTTPError as e:
        try:
            payload = json.loads(e.read() or b'null')
        except ValueError:
            payload = None
        return e.code, payload
    except (urllib.error.URLError, TimeoutError) as e:
        return 0, {'error': str(e)}


def at_ist(date, hour, minute=0):
    return dt.datetime.combine(date, dt.time(hour, minute), IST).astimezone(dt.timezone.utc)


def iso(t):
    return t.isoformat().replace('+00:00', 'Z')


def sunrise_moment(date, birth, lat, lng, key):
    """Ten minutes after the date's sunrise, read from the vedic-clock engine at noon IST,
    exactly as the engine's daily-field job does. Falls back to 06:00 IST."""
    status, out = call('POST', '/engines/vedic-clock/calculate', key,
                       {'birth_data': birth, 'current_time': iso(at_ist(date, 12)),
                        'options': {'latitude': lat, 'longitude': lng, 'timezone_offset': 330}})
    day = ((out or {}).get('result') or {}).get('vedic_day') or {} if status == 200 else {}
    if day.get('date') == date.isoformat() and day.get('sunrise'):
        rise = dt.datetime.fromisoformat(day['sunrise']).astimezone(dt.timezone.utc)
        return rise + dt.timedelta(minutes=10), 'sunrise+10m'
    return at_ist(date, 6), 'fixed-06:00-ist'


def profile_birth(key):
    """Birth data from the engine profile, the same source the scheduled job uses."""
    status, me = call('GET', '/users/me', key)
    if status != 200 or not me.get('birth_date') or not me.get('birth_location'):
        sys.exit(f'sync_daily: engine profile has no birth data (HTTP {status})')
    loc = me['birth_location']
    birth = {'date': me['birth_date'], 'time': (me.get('birth_time') or '')[:5] or None,
             'latitude': loc['lat'], 'longitude': loc['lng'], 'timezone': me.get('timezone') or 'Asia/Kolkata'}
    here = (me.get('preferences') or {}).get('daily_field_location') or {}
    return birth, here.get('lat', loc['lat']), here.get('lng', loc['lng'])


def via_engine_calls(date, key):
    """Fallback until /daily is deployed: the same parts, each stored by the engine as it runs."""
    birth, lat, lng = profile_birth(key)
    when_t, anchor = sunrise_moment(date, birth, lat, lng, key)
    when = iso(when_t)
    engines, missing = {}, []
    for eid in DAY_ENGINES:
        options = {}
        if eid == 'panchanga':
            options['mode'] = 'daily'
        if eid == 'vedic-clock':
            options.update(latitude=lat, longitude=lng, timezone_offset=330)
        status, out = call('POST', f'/engines/{eid}/calculate', key,
                           {'birth_data': birth, 'current_time': when, 'options': options})
        if status == 200:
            engines[eid] = {'reading_id': None, 'created_at': dt.datetime.now(dt.timezone.utc).isoformat(), 'result': out}
        else:
            missing.append({'part': eid, 'reason': f"engine call HTTP {status}: {(out or {}).get('error_code') or (out or {}).get('error', '')}"})
    status, dyad = call('POST', '/witness/interpret', key, {'birth_data': birth, 'current_time': when}, timeout=120)
    witness = None
    if status == 200:
        witness = {'reading_id': None, 'created_at': dt.datetime.now(dt.timezone.utc).isoformat(), 'result': dyad}
    else:
        if status == 422:
            reason = 'this deployment still requires a biofield capture (live_scores); the daily-field engine change makes it optional'
        else:
            reason = f"witness HTTP {status}: {((dyad or {}).get('error') or (dyad or {}).get('error_code') or '')[:160]}"
        missing.append({'part': 'witness-dyad', 'reason': reason})
    return {'date': date.isoformat(), 'moment': when, 'moment_anchor': anchor, 'method': 'daily-field/v1', 'complete': not missing,
            'engines': engines, 'witness': witness, 'missing': missing, 'source': 'engine-calls'}


def fetch_day(date, key):
    status, bundle = call('GET', f'/daily?date={date.isoformat()}', key)
    if status == 200 and bundle.get('complete'):
        return dict(bundle, source='daily-api')
    if status in (200, 422, 503) or (status == 404 and (bundle or {}).get('error_code')):
        rstatus, refreshed = call('POST', '/daily/refresh', key, {'date': date.isoformat()}, timeout=180)
        if rstatus == 200:
            return dict(refreshed, source='daily-api')
        if status == 200:
            return dict(bundle, source='daily-api')
    # The route is not on this deployment yet: ask the engines directly.
    return via_engine_calls(date, key)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--date', help='IST date YYYY-MM-DD (default today)')
    ap.add_argument('--back', type=int, default=0, help='also fetch this many earlier days')
    ap.add_argument('--force', action='store_true', help='refetch complete days')
    a = ap.parse_args()
    end = dt.date.fromisoformat(a.date) if a.date else dt.datetime.now(IST).date()
    key = api_key()
    OUT.mkdir(parents=True, exist_ok=True)
    report = []
    for i in range(a.back, -1, -1):
        d = end - dt.timedelta(days=i)
        path = OUT / f'{d.isoformat()}.json'
        if path.exists() and not a.force:
            saved = json.loads(path.read_text())
            if saved.get('complete'):
                report.append(f'{d} kept (complete, {saved.get("source")})')
                continue
        bundle = fetch_day(d, key)
        bundle['fetched_at'] = dt.datetime.now(dt.timezone.utc).isoformat()
        path.write_text(json.dumps(bundle, separators=(',', ':'), ensure_ascii=False))
        gaps = ', '.join(m['part'] for m in bundle.get('missing', [])) or 'none'
        report.append(f"{d} {bundle.get('source')} · engines {len(bundle.get('engines', {}))}/5 · witness {'yes' if bundle.get('witness') else 'no'} · missing {gaps}")
    print('\n'.join(report))


if __name__ == '__main__':
    main()
