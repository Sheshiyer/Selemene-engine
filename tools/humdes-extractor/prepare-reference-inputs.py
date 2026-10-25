"""Stage reference report inputs without replacing verified snapshots.

Run with: uv run --python 3.12 --with pyswisseph python <this-file> PACKAGE_DIR
Uses the existing local Swiss Ephemeris calculator and saved source records.
"""
import hashlib
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare(root):
    root = root.resolve()
    request_path = root / 'source-pack.en/request.json'
    provenance_path = root / 'jev-shadow/source-provenance.json'
    request = json.loads(request_path.read_text())
    provenance = json.loads(provenance_path.read_text())
    gary = request['subjects'][0]
    loc = gary['normalized_location']
    subjects = [('gary-abitbol', {
        'name': gary['name'], 'date': gary['birth_date'], 'time': gary['birth_time'],
        'timezone': loc['timezone'], 'latitude': loc['latitude'], 'longitude': loc['longitude'],
        'place': loc['display_name'], 'confidence': gary['birth_time_confidence'],
    }, request_path)]
    for slug, key in [('mohan-kumar', 'mohan'), ('sheshnarayan', 'shesh')]:
        b = provenance[key]['engine_input']
        subjects.append((slug, {**b, 'place': 'Coordinates from saved engine input',
                                'confidence': provenance[key]['birth_time_confidence']}, provenance_path))
    calculator = Path(__file__).with_name('compute_vedic_kundali.py')
    spec = importlib.util.spec_from_file_location('reference_kundali_calculator', calculator)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    import swisseph as swe
    target = root / 'reports-reference/engines'
    target.mkdir(parents=True, exist_ok=True)
    receipt = []
    for slug, b, source in subjects:
        original = root / f'jev-shadow/engines/{slug}.prod.json'
        engines = json.loads(original.read_text())
        calculated_at = datetime.now(timezone.utc).isoformat()
        k = module.compute_kundali(b['name'], b['date'], b['time'], b['timezone'],
                                  b['latitude'], b['longitude'], b['place'])
        _, jd = module.parse_birth(b['date'], b['time'], b['timezone'])
        _, flags = swe.calc_ut(jd, swe.SUN, swe.FLG_SIDEREAL | swe.FLG_SPEED)
        backend = 'swiss-ephemeris' if flags & swe.FLG_SWIEPH else 'moshier' if flags & swe.FLG_MOSEPH else 'jpl'
        # Add natal foundations only. The saved Vimshottari snapshot remains the
        # period authority; this calculator's current-period flags are excluded.
        result = {key: k[key] for key in ['ayanamsa', 'lagna', 'planets', 'houses']}
        result['house_system'] = 'whole-sign'
        result['birth_time_confidence'] = b['confidence']
        result['calculation_timestamp'] = calculated_at
        envelope = {
            'engine_id': 'vedic-kundali', 'result': result, 'calculated_at': calculated_at,
            'metadata': {'backend': f'local-pyswisseph:{backend}', 'version': swe.version,
                         'source_path': str(source), 'source_sha256': digest(source),
                         'calculator_path': str(calculator.resolve()), 'calculator_sha256': digest(calculator),
                         'source_role': 'new natal calculation from saved birth input; not a legacy narrative'},
        }
        engines = [e for e in engines if e['engine_id'] != 'vedic-kundali'] + [envelope]
        output = target / f'{slug}.prod.json'
        output.write_text(json.dumps(engines, indent=2, ensure_ascii=False) + '\n')
        receipt.append({'subject': slug, 'original_sha256': digest(original), 'output_sha256': digest(output),
                        'output': str(output), 'backend': backend, 'planet_count': len(result['planets']),
                        'houses_valid': all(1 <= p['house'] <= 12 for p in result['planets'].values()),
                        'source_sha256': digest(source), 'birth_time_confidence': b['confidence']})
    (target.parent / 'input-preparation.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('Usage: prepare-reference-inputs.py PACKAGE_DIR')
    prepare(Path(sys.argv[1]))
