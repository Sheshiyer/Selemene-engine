#!/usr/bin/env python3
"""Validate and export this credential-free private plugin; never upload it."""
import argparse
import json
from pathlib import Path
import re
import struct
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
ENDPOINT = 'https://selemene-free-mcp.sheshnarayan-iyer.workers.dev/mcp'
FORBIDDEN_PARTS = {'.git', '.app.json', 'node_modules', '__pycache__', '.DS_Store'}
SECRET_PATTERNS = (
    re.compile(r'(?<![\w])(?:nk_|sk-|sk_(?:live|test)_)[A-Za-z0-9_-]{12,}'),
    re.compile(r'\b(?:NOESIS_API_KEY|SELEMENE_API_KEY|api[_-]?key|access[_-]?token|client[_-]?secret|password)\b[\"\']?\s*[:=]\s*[\"\']?[^\s\"\'<>$]{4,}', re.I),
    re.compile(r'\b(?:Authorization|X-API-Key)[\"\']?\s*[:=]\s*[\"\']?(?:Bearer\s+|Basic\s+)?[A-Za-z0-9_./+\-=]{8,}', re.I),
    re.compile(r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Validate without exporting.')
    parser.add_argument('--output', type=Path, help='New ZIP path outside the plugin source.')
    args = parser.parse_args()
    errors = []

    def require(condition, message):
        if not condition:
            errors.append(message)

    def read_object(path):
        try:
            value = json.loads(path.read_text(encoding='utf-8'))
            if isinstance(value, dict):
                return value
            errors.append(f'{path.name} must contain a JSON object.')
        except (OSError, ValueError):
            errors.append(f'{path.name} is missing or invalid JSON.')
        return {}

    def contained(path):
        if not path.resolve().is_relative_to(ROOT.resolve()):
            return False
        return not path.is_symlink() and all(not p.is_symlink() for p in path.parents if p != ROOT and ROOT in p.parents)

    manifest = read_object(ROOT / 'plugin.json')
    require(manifest.get('name') == ROOT.name and re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', ROOT.name) and len(ROOT.name) <= 64,
            'Manifest name must match its lowercase plugin directory.')
    require(bool(re.fullmatch(r'(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)', str(manifest.get('version', '')))), 'A semantic version is required.')
    require(isinstance(manifest.get('description'), str) and bool(manifest['description'].strip()), 'A plugin description is required.')
    require(not any(key in manifest for key in ('skills', 'mcpServers', 'apps', 'interface')), 'Use the portable manifest fields.')
    extensions = manifest.get('extensions', {})
    openai = extensions.get('com.openai', {}) if isinstance(extensions, dict) else {}
    listing = openai.get('interface', {}) if isinstance(openai, dict) else {}
    if not isinstance(listing, dict):
        listing = {}
    require(isinstance(openai, dict) and not openai.get('apps'), 'This private export cannot include an app binding.')
    for field, limit in (('displayName', 80), ('shortDescription', 30), ('longDescription', 4000)):
        value = listing.get(field)
        require(isinstance(value, str) and bool(value.strip()) and len(value) <= limit, f'{field} is required and must fit its length limit.')
    prompts = listing.get('defaultPrompt', [])
    prompts = [prompts] if isinstance(prompts, str) else prompts
    require(isinstance(prompts, list) and 1 <= len(prompts) <= 3 and all(isinstance(p, str) and p.strip() and len(p) <= 128 and '\n' not in p and '\r' not in p for p in prompts),
            'Use one to three short single-line default prompts.')
    # Private exports do not require publisher attestation, countries, category,
    # policy URLs, a recording, commerce declarations or public review receipts.
    for field in ('logo', 'composerIcon'):
        relative = listing.get(field)
        path = ROOT / relative if isinstance(relative, str) else ROOT / 'MISSING'
        require(isinstance(relative, str) and not Path(relative).is_absolute() and contained(path), f'{field} must be a contained regular asset.')
        if not contained(path):
            continue
        try:
            raw = path.read_bytes()
            width, height = struct.unpack('>II', raw[16:24]) if raw.startswith(b'\x89PNG\r\n\x1a\n') and len(raw) >= 24 else (0, 0)
            require(48 <= width == height <= 4096 and len(raw) <= 5 * 1024 * 1024, f'{field} must be a square PNG, 48–4096 pixels and at most 5 MiB.')
        except OSError:
            errors.append(f'{field} asset is missing.')

    mcp = read_object(ROOT / 'mcp.json')
    servers = mcp.get('mcpServers', {})
    require(isinstance(servers, dict) and set(servers) == {'selemene'}, 'Configure exactly the Selemene MCP server.')
    server = servers.get('selemene', {}) if isinstance(servers, dict) else {}
    require(isinstance(server, dict) and set(server) == {'type', 'url'} and server.get('type') == 'streamable-http' and server.get('url') == ENDPOINT,
            'MCP must use the fixed HTTPS free endpoint without credentials, headers or local execution.')

    files = [ROOT / 'plugin.json', ROOT / 'mcp.json']
    skill_files = []
    for folder in ('skills', 'assets'):
        require((ROOT / folder).is_dir() and contained(ROOT / folder), f'{folder} must be a contained directory.')
        for path in (ROOT / folder).rglob('*'):
            relative = path.relative_to(ROOT)
            require(contained(path), f'Archive path must be contained and free of symlinks: {relative}')
            require(not any(part in FORBIDDEN_PARTS or part.startswith('.env') for part in relative.parts), f'Forbidden private/dependency file: {relative}')
            if path.is_file():
                files.append(path)
                if path.name == 'SKILL.md':
                    skill_files.append(path)
    require(bool(skill_files), 'At least one skill is required.')
    for path in skill_files:
        if not contained(path):
            continue
        try:
            text = path.read_text(encoding='utf-8')
            frontmatter = text.split('---', 2)[1] if text.startswith('---\n') and text.count('---') >= 2 else ''
            name = re.search(r'^name:\s*([^\n]+)', frontmatter, re.M)
            description = re.search(r'^description:\s*([^\n]+)', frontmatter, re.M)
            require(path.parent.parent == ROOT / 'skills' and name is not None and name.group(1).strip().strip('\"\'') == path.parent.name and description is not None and bool(description.group(1).strip()),
                    f'Skill needs matching name and descriptive frontmatter: {path.relative_to(ROOT)}')
        except (OSError, UnicodeError):
            errors.append(f'Skill is unreadable: {path.relative_to(ROOT)}')

    require(not (ROOT / '.app.json').exists() and not any(ROOT.glob('.env*')), 'Remove private app/env files from the package source.')
    for path in files:
        require(contained(path), f'Archive file must be contained and free of symlinks: {path.relative_to(ROOT)}')
        if not contained(path):
            continue
        try:
            raw = path.read_bytes()
            try:
                text = raw.decode('utf-8')
            except UnicodeError:
                continue
            require(not any(pattern.search(text) for pattern in SECRET_PATTERNS), f'Embedded credential detected: {path.relative_to(ROOT)}')
        except OSError:
            errors.append(f'Archive file is unreadable: {path.relative_to(ROOT)}')

    if args.output:
        require(not args.output.resolve().is_relative_to(ROOT.resolve()), 'Write the ZIP outside the source directory.')
        require(not args.output.exists() and not args.output.is_symlink(), 'Output exists; choose a new release path.')
    if errors:
        print('NOT READY — no archive created.\n' + '\n'.join('- ' + e for e in errors))
        return 1
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        inventory = sorted(f'{ROOT.name}/{path.relative_to(ROOT).as_posix()}' for path in files)
        with zipfile.ZipFile(args.output, 'x', zipfile.ZIP_DEFLATED) as archive:
            for path in sorted(files):
                archive.write(path, f'{ROOT.name}/{path.relative_to(ROOT).as_posix()}')
        with zipfile.ZipFile(args.output) as archive:
            assert archive.testzip() is None
            assert sorted(archive.namelist()) == inventory
            assert json.loads(archive.read(f'{ROOT.name}/plugin.json')) == manifest
            assert json.loads(archive.read(f'{ROOT.name}/mcp.json')) == mcp
        print(f'Private archive prepared and inspected: {args.output}')
    else:
        print('Private format and safety checks passed; no archive requested.')
    print('Private export does not establish service deployment, host execution or public publication.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
