#!/usr/bin/env python3
"""Offline public-package readiness gate. Never upload or invent evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import sys
from urllib.parse import urlsplit
import zipfile

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check', action='store_true')
parser.add_argument('--submission-check', action='store_true', help='Also require connected review cases for the saved submission version.')
parser.add_argument('--evidence', type=Path)
parser.add_argument('--output', type=Path)
args = parser.parse_args()
errors = []

def require(condition, message):
    if not condition:
        errors.append(message)

def read_json(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError) as exc:
        errors.append(f'{path.name}: {exc}')
        return {}

def https(value):
    if not isinstance(value, str) or len(value) > 1024:
        return False
    try:
        u = urlsplit(value)
        host = u.hostname or ''
        return u.scheme == 'https' and bool(host) and not u.username and not u.password and not u.fragment and host not in ('localhost', 'example.com', 'example.org', 'example.net') and not host.endswith(('.example', '.invalid', '.localhost'))
    except ValueError:
        return False

def nonblank(value):
    return isinstance(value, str) and bool(value.strip())

manifest = read_json(ROOT / 'plugin.json')
oi = manifest.get('extensions', {}).get('com.openai', {})
listing = oi.get('interface', {})
review = oi.get('review', {})
publication = oi.get('publication', {})
require(manifest.get('name') == ROOT.name, 'Manifest name must match the plugin directory.')
require(bool(re.fullmatch(r'\d+\.\d+\.\d+', str(manifest.get('version', '')))), 'Explicit semantic version is required.')
for key, limit in [('displayName',30),('shortDescription',30),('longDescription',4000),('developerName',80)]:
    value = listing.get(key)
    require(nonblank(value) and len(value) <= limit, f'{key} must be present and no longer than {limit} characters.')
require(nonblank(listing.get('category')), 'Verified dashboard category is missing.')
require(manifest.get('author', {}).get('name', listing.get('developerName')) == listing.get('developerName'), 'Author must match the intended verified developer identity.')
prompts = listing.get('defaultPrompt', [])
prompts = [prompts] if isinstance(prompts, str) else prompts
require(isinstance(prompts,list) and 1 <= len(prompts) <= 3, 'Provide 1–3 default prompts.')
if isinstance(prompts,list):
    normalized = [' '.join(p.split()) for p in prompts if isinstance(p,str)]
    require(len(normalized) == len(prompts) == len(set(normalized)), 'Prompts must be unique strings.')
    require(all(nonblank(p) and len(p)<=128 and '\n' not in p and '\r' not in p and '@' not in p for p in prompts if isinstance(p,str)), 'Default prompts need single-line text, <=128 characters, no @mentions.')
urls = ['websiteURL','supportURL','privacyPolicyURL','termsOfServiceURL']
for key in urls:
    require(https(listing.get(key)), f'{key}: verified public HTTPS page is missing.')
require(https(review.get('demo_recording_url')), 'Verified reviewer-accessible demo recording URL is missing.')
require(isinstance(review.get('commerce'), bool), 'Actual commerce declaration must be a boolean.')
if review.get('commerce'):
    require(nonblank(review.get('commerce_description')), 'Commerce requires a truthful purchase/payment description.')
require(nonblank(publication.get('release_notes')), 'Release notes are missing.')
countries = publication.get('countries')
require(isinstance(countries,list) and all(isinstance(c,str) and re.fullmatch(r'[A-Z]{2}',c) for c in countries), 'Explicit country targeting is missing (empty array means all countries only if chosen).')
for kind, count in [('positive',5),('negative',3)]:
    cases = review.get('test_cases',{}).get(kind,[])
    require(isinstance(cases,list) and len(cases)==count, f'Need exactly {count} {kind} cases.')
    for i,case in enumerate(cases if isinstance(cases,list) else []):
        require(isinstance(case,dict), f'{kind} case {i+1} must be an object.')
        if isinstance(case,dict):
            fields=['description','prompt'] + (['tools_triggered','expected_behavior'] if kind=='positive' else [])
            for field in fields:
                require(nonblank(case.get(field)), f'{kind} case {i+1}: {field} must be a string.')
            require(not any(k in case for k in ('status','result','passed')), 'Run results belong in preparation notes, not case metadata.')
for key, minimum in [('logo',256),('composerIcon',48)]:
    value = listing.get(key)
    path = ROOT / value if isinstance(value,str) else ROOT/'MISSING'
    require(path.resolve().is_relative_to(ROOT.resolve()) and not path.is_symlink(), f'{key} must be a contained regular file.')
    try:
        raw = path.read_bytes()
        require(raw[:8] == b'\x89PNG\r\n\x1a\n' and len(raw)>=24, f'{key} must be a real PNG.')
        width,height=struct.unpack('>II',raw[16:24]) if len(raw)>=24 else (0,0)
        require(minimum<=width<=4096 and width==height and len(raw)<=5*1024*1024, f'{key} must be square, {minimum}–4096px and <=5 MiB.')
    except OSError:
        errors.append(f'{key}: actual PNG asset is missing.')
require(not (ROOT/'.app.json').exists(), 'Root .app.json is forbidden in public uploads.')
for path in ROOT.rglob('plugin.json'):
    data=read_json(path)
    require(data.get('apps') is None and data.get('extensions',{}).get('com.openai',{}).get('apps') is None, f'{path}: public app binding is forbidden.')
for key in ('skills','mcpServers','interface'):
    require(key not in manifest, f'{key} is not a portable root-manifest field.')
mcp=read_json(ROOT/'mcp.json')
servers=mcp.get('mcpServers',{})
require(isinstance(servers,dict) and len(servers)==1, 'Exactly one verified MCP server must be configured.')
server=next(iter(servers.values()),{}) if isinstance(servers,dict) else {}
endpoint=server.get('url') if isinstance(server,dict) else None
require(isinstance(server,dict) and server.get('type')=='streamable-http' and https(endpoint), 'Verified Streamable HTTP MCP endpoint is missing.')
require(isinstance(server,dict) and not any(k in server for k in ('headers','env','command','args')), 'Public MCP configuration must not embed credentials or local execution settings.')
receipt=read_json(args.evidence) if args.evidence else {}
digest=hashlib.sha256((ROOT/'plugin.json').read_bytes()).hexdigest()
require(receipt.get('manifest_sha256')==digest, 'External evidence must identify this exact manifest SHA-256.')
checks=receipt.get('checks',{})
required={key:listing.get(key) for key in urls+['developerName','category']}
required.update({'endpoint':endpoint,'demo_recording_url':review.get('demo_recording_url'),'publication.countries':json.dumps(countries,separators=(',',':')),'review.commerce':json.dumps(review.get('commerce'),separators=(',',':')),'protocol_contract_tests':'passed','tool_annotations':'passed','icon_visual_inspection':'passed'})
if args.submission_check:
    required['connected_review_cases'] = 'passed'
for key,expected in required.items():
    item=checks.get(key,{})
    require(isinstance(item,dict) and expected is not None and item.get('value')==expected and nonblank(item.get('verified_at')) and nonblank(item.get('evidence')), f'Current verification evidence is missing: {key}.')
files=[ROOT/'plugin.json',ROOT/'mcp.json']
for folder in ('skills','assets'):
    files.extend(path for path in (ROOT/folder).rglob('*') if path.is_file())
require(any(p.name=='SKILL.md' for p in files), 'Packaged skill is missing.')
require(all(not p.is_symlink() and p.resolve().is_relative_to(ROOT.resolve()) and all(not parent.is_symlink() for parent in p.parents if parent != ROOT and ROOT in parent.parents) for p in files), 'Every archive file and parent path must be contained and free of symlinks.')
if args.output:
    require(not args.output.resolve().is_relative_to(ROOT.resolve()), 'Write the ZIP outside the source directory.')
    require(not args.output.exists(), 'Output exists; choose a new archive path rather than replacing a release.')
if errors:
    print('NOT READY — no archive created.\n'+'\n'.join('- '+e for e in errors))
    sys.exit(1)
if args.output:
    args.output.parent.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(args.output,'x',zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(files):
            archive.write(path, f'{ROOT.name}/{path.relative_to(ROOT).as_posix()}')
    with zipfile.ZipFile(args.output) as archive:
        assert archive.testzip() is None
        assert json.loads(archive.read(f'{ROOT.name}/plugin.json')) == manifest
        assert json.loads(archive.read(f'{ROOT.name}/mcp.json')) == mcp
        assert len(archive.namelist()) == len(files)
    print(f'Archive prepared and inspected: {args.output}')
else:
    print('Offline package checks passed; no archive requested.')
print('Candidate ZIP readiness is separate from submission readiness. Run --submission-check after testing the saved submission version.')
print('Portal verification, secure reviewer access and developer attestations remain separate.')
