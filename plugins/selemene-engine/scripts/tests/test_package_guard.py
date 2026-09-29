"""Synthetic temp-directory tests. No URLs are contacted or receipts published."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

SOURCE = Path(__file__).resolve().parents[2]

class PackageGuardTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name)
        self.root = self.base / 'selemene-engine'
        shutil.copytree(SOURCE, self.root)
        manifest = json.loads((self.root/'plugin.json').read_text())
        oi = manifest['extensions']['com.openai']
        interface = oi['interface']
        interface.update(developerName='Synthetic test fixture', category='Test category')
        for key in ('websiteURL','supportURL','privacyPolicyURL','termsOfServiceURL'):
            interface[key] = f'https://synthetic.test/{key}'
        oi['review'].update(commerce=False,demo_recording_url='https://synthetic.test/demo')
        oi['publication']['countries']=['FR']
        self.manifest = manifest
        (self.root/'mcp.json').write_text(json.dumps({'mcpServers':{'selemene':{'type':'streamable-http','url':'https://synthetic.test/mcp'}}}))
        self.write_manifest()

    def tearDown(self):
        self.temp.cleanup()

    def write_manifest(self):
        (self.root/'plugin.json').write_text(json.dumps(self.manifest))
        oi=self.manifest['extensions']['com.openai']
        values={key:oi['interface'][key] for key in ('developerName','category','websiteURL','supportURL','privacyPolicyURL','termsOfServiceURL')}
        values.update(endpoint='https://synthetic.test/mcp',demo_recording_url='https://synthetic.test/demo',protocol_contract_tests='passed',tool_annotations='passed',icon_visual_inspection='passed')
        values['review.commerce']=json.dumps(oi['review']['commerce'],separators=(',',':'))
        values['publication.countries']=json.dumps(oi['publication']['countries'],separators=(',',':'))
        receipt={'manifest_sha256':hashlib.sha256((self.root/'plugin.json').read_bytes()).hexdigest(),'checks':{k:{'value':v,'verified_at':'synthetic-test-only','evidence':'Synthetic fixture, not live evidence'} for k,v in values.items()}}
        (self.base/'receipt.json').write_text(json.dumps(receipt))

    def run_guard(self,*extra):
        return subprocess.run([sys.executable,str(self.root/'scripts/create-package.py'),'--evidence',str(self.base/'receipt.json'),*extra],text=True,capture_output=True)

    def test_candidate_zip_does_not_require_saved_portal_cases(self):
        archive=self.base/'candidate.zip'
        result=self.run_guard('--output',str(archive))
        self.assertEqual(result.returncode,0,result.stdout+result.stderr)
        self.assertTrue(archive.is_file())
        result=self.run_guard('--submission-check')
        self.assertNotEqual(result.returncode,0)
        self.assertIn('connected_review_cases',result.stdout)

    def test_nonboolean_commerce_is_rejected(self):
        self.manifest['extensions']['com.openai']['review']['commerce']='false'
        self.write_manifest()
        result=self.run_guard('--check')
        self.assertNotEqual(result.returncode,0)
        self.assertIn('must be a boolean',result.stdout)

    def test_asset_parent_symlink_cannot_escape_upload(self):
        shutil.move(self.root/'assets',self.base/'external-assets')
        (self.root/'assets').symlink_to(self.base/'external-assets',target_is_directory=True)
        result=self.run_guard('--output',str(self.base/'rejected.zip'))
        self.assertNotEqual(result.returncode,0)
        self.assertFalse((self.base/'rejected.zip').exists())
        self.assertIn('contained',result.stdout)

    def test_missing_endpoint_does_not_produce_archive(self):
        (self.root/'mcp.json').unlink()
        result=self.run_guard('--output',str(self.base/'rejected.zip'))
        self.assertNotEqual(result.returncode,0)
        self.assertFalse((self.base/'rejected.zip').exists())
        self.assertIn('endpoint is missing',result.stdout)

if __name__=='__main__':
    unittest.main()
