"""Private-export safety tests with isolated fixtures; no service is contacted."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile

SOURCE = Path(__file__).resolve().parents[2]
ENDPOINT = 'https://selemene-free-mcp.sheshnarayan-iyer.workers.dev/mcp'


class PrivatePackageGuardTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name)
        self.root = self.base / 'selemene-engine'
        (self.root / 'scripts').mkdir(parents=True)
        (self.root / 'assets').mkdir()
        self.skill = self.root / 'skills/selemene-reading/SKILL.md'
        self.skill.parent.mkdir(parents=True)
        shutil.copy2(SOURCE / 'scripts/create-private-package.py', self.root / 'scripts/create-private-package.py')
        shutil.copy2(SOURCE / 'assets/icon.png', self.root / 'assets/icon.png')
        self.manifest = {
            'name': 'selemene-engine', 'version': '0.2.0',
            'description': 'Synthetic private format fixture.',
            'extensions': {'com.openai': {'interface': {
                'displayName': 'Selemene Engine', 'shortDescription': 'Symbolic reflection',
                'longDescription': 'Use the hosted service without a personal API key.',
                'defaultPrompt': ['Discover engines.'],
                'logo': 'assets/icon.png', 'composerIcon': 'assets/icon.png',
            }}},
        }
        self.mcp = {'mcpServers': {'selemene': {'type': 'streamable-http', 'url': ENDPOINT}}}
        self.write_json()
        self.skill.write_text('---\nname: selemene-reading\ndescription: Use for requested symbolic readings.\n---\n\nThe API key stays on the hosted server; never ask for an authorization token.\n')

    def tearDown(self):
        self.temp.cleanup()

    def write_json(self):
        (self.root / 'plugin.json').write_text(json.dumps(self.manifest))
        (self.root / 'mcp.json').write_text(json.dumps(self.mcp))

    def run_guard(self, *args):
        return subprocess.run([sys.executable, '-B', str(self.root / 'scripts/create-private-package.py'), *args], text=True, capture_output=True)

    def assert_rejected(self, expected):
        archive = self.base / 'rejected.zip'
        result = self.run_guard('--output', str(archive))
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(expected, result.stdout)
        self.assertFalse(archive.exists())

    def test_private_export_without_public_fields_and_safe_key_prose(self):
        archive = self.base / 'private.zip'
        result = self.run_guard('--output', str(archive))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        with zipfile.ZipFile(archive) as package:
            self.assertEqual(set(package.namelist()), {
                'selemene-engine/plugin.json', 'selemene-engine/mcp.json',
                'selemene-engine/assets/icon.png', 'selemene-engine/skills/selemene-reading/SKILL.md',
            })
            self.assertIsNone(package.testzip())
            self.assertEqual(json.loads(package.read('selemene-engine/plugin.json')), self.manifest)

    def test_output_inside_source_is_rejected(self):
        archive = self.root / 'private.zip'
        result = self.run_guard('--output', str(archive))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('outside the source', result.stdout)
        self.assertFalse(archive.exists())

    def test_existing_release_is_not_overwritten(self):
        archive = self.base / 'existing.zip'
        archive.write_bytes(b'existing-release-sentinel')
        result = self.run_guard('--output', str(archive))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Output exists', result.stdout)
        self.assertEqual(archive.read_bytes(), b'existing-release-sentinel')

    def test_fixed_endpoint_and_credential_free_transport(self):
        for change in ({'url': ENDPOINT + '?api_key=synthetic'}, {'headers': {'Authorization': 'Bearer synthetic-sentinel'}}, {'env': {'NOESIS_API_KEY': 'synthetic-sentinel'}}, {'url': 'http://localhost:8080/mcp'}):
            with self.subTest(change=change):
                self.mcp['mcpServers']['selemene'] = {'type': 'streamable-http', 'url': ENDPOINT, **change}
                self.write_json()
                self.assert_rejected('fixed HTTPS free endpoint')

    def test_embedded_credential_assignment_is_rejected(self):
        self.skill.write_text(self.skill.read_text() + '\nNOESIS_API_KEY=synthetic_secret_sentinel_123\n')
        self.assert_rejected('Embedded credential')

    def test_asset_directory_symlink_is_rejected(self):
        shutil.move(self.root / 'assets', self.base / 'external-assets')
        (self.root / 'assets').symlink_to(self.base / 'external-assets', target_is_directory=True)
        self.assert_rejected('contained')

    def test_skill_frontmatter_must_match_directory(self):
        self.skill.write_text(self.skill.read_text().replace('name: selemene-reading', 'name: unrelated-workflow'))
        self.assert_rejected('matching name')

    def test_private_env_in_skill_tree_is_rejected(self):
        (self.skill.parent / '.env').write_text('synthetic fixture')
        self.assert_rejected('Forbidden private/dependency')

    def test_nonobject_manifest_fails_closed(self):
        (self.root / 'plugin.json').write_text('[]')
        self.assert_rejected('JSON object')


if __name__ == '__main__':
    unittest.main()
