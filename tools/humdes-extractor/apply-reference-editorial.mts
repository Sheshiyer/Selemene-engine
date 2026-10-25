import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { applyEditorialRevision } from '../../packages/witness-pipeline/src/orchestrator/editorial-revision.js';
const [dir, section, version = ''] = process.argv.slice(2);
if (!dir || !/^(opening|part(?:[1-9]|10|11))$/.test(section ?? '')) throw new Error('run directory and valid section required');
if (version && !/^v[1-9][0-9]*$/.test(version)) throw new Error('version must be v followed by a positive integer');
const suffix = version ? '-' + version : '';
const baseReceipt = JSON.parse(readFileSync(join(dir, section + '.receipt.json'), 'utf8'));
const patchSet = JSON.parse(readFileSync(join(dir, section + '.editorial-patches' + suffix + '.json'), 'utf8'));
const revision = applyEditorialRevision({ baseReceipt, currentOutput: baseReceipt.raw.synthesis,
 expectedBaseOutputHash: patchSet.expectedBaseOutputHash, patches: patchSet.patches, reviewer: 'assistant-source-review', timestamp: new Date().toISOString() });
writeFileSync(join(dir, section + '.editorial-revision' + suffix + '.json'), JSON.stringify(revision,null,2) + '\n', {flag:'wx'});
writeFileSync(join(dir, section + '.editorial-candidate' + suffix + '.md'), revision.revisedOutput + '\n', {flag:'wx'});
console.log(JSON.stringify({section, baseHash:revision.baseOutputHash, revisedHash:revision.revisedOutputHash, pending:revision.pendingReviews}));
