import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { applyEditorialRevision } from '../../packages/witness-pipeline/src/orchestrator/editorial-revision.js';
const dir = process.argv[2];
if (!dir) throw new Error('run directory required');
const receipt = JSON.parse(readFileSync(join(dir, 'opening.receipt.json'), 'utf8'));
const review = JSON.parse(readFileSync(join(dir, 'opening.editorial-review.json'), 'utf8'));
const revision = applyEditorialRevision({ baseReceipt: receipt, currentOutput: receipt.raw.synthesis,
 expectedBaseOutputHash: review.outputHash, reviewer: 'assistant-source-review', timestamp: new Date().toISOString(),
 patches: review.findings.map((f: any) => ({ before: f.claim, after: f.correction, rationale: f.issue,
 sourceReferences: [{ sourceId: f.source, quote: 'const longitude = to_sidereal_longitude(pos.longitude, datetime);', label: 'Engine implementation inspected during source review' }] })) });
writeFileSync(join(dir, 'opening.editorial-revision.json'), JSON.stringify(revision, null, 2) + '\n', { flag: 'wx' });
writeFileSync(join(dir, 'opening.editorial-candidate.md'), revision.revisedOutput + '\n', { flag: 'wx' });
console.log(JSON.stringify({ section: revision.sectionId, base: revision.baseOutputHash, revised: revision.revisedOutputHash, reviews: revision.pendingReviews }));
