import {readFileSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {verifyEditorialSection} from '../../packages/witness-pipeline/src/orchestrator/editorial-verification.js';

// Component verification only. Whole-report sources, continuity, rendering,
// and bilingual parity still require verify-reference-artifact.mts.
const [dir, section, auditFile, jevFile, version = 'editorial', dispositionFile] = process.argv.slice(2);
if (!dir || !/^(opening|part(?:[1-9]|10|11))$/.test(section ?? '') ||
    !auditFile || !jevFile || !/^(editorial|v[1-9][0-9]*)$/.test(version)) {
  throw new Error('Usage: run-dir section audit-file jev-file [editorial|vN] [disposition-file]');
}
const read = (file: string) => JSON.parse(readFileSync(join(dir, file), 'utf8'));
const baseReceiptRawBytes = readFileSync(join(dir, section + '.receipt.json'), 'utf8');
const originalReceipt = JSON.parse(baseReceiptRawBytes);
const revisionArtifact = read(section + '.editorial-revision' + (version === 'editorial' ? '' : '-' + version) + '.json');
const freshAudit = read(auditFile);
const freshJev = read(jevFile);
let guardrailDisposition;
if (dispositionFile) {
  const disposition = read(dispositionFile);
  const rawEvidenceTexts: Record<string, string> = {};
  for (const ref of disposition.evidence) {
    if (typeof ref.file !== 'string' || ref.file.includes('/') || ref.file.includes('..')) throw new Error('Unsafe evidence filename');
    rawEvidenceTexts[ref.file] = readFileSync(join(dir, ref.file), 'utf8');
  }
  const diagnostics = Object.entries(rawEvidenceTexts).filter(([name]) => name.includes('guardrail-diagnostic'));
  if (diagnostics.length !== 1) throw new Error('Disposition must identify exactly one diagnostic');
  guardrailDisposition = {
    disposition, originalJev: freshJev, revision: revisionArtifact,
    diagnostic: JSON.parse(diagnostics[0][1]), rawEvidenceTexts,
  };
}
const implementationSourceTexts: Record<string, string> = {};
if (freshAudit.supplementalEvidence) {
  const path = freshAudit.supplementalEvidence.path;
  implementationSourceTexts[path] = readFileSync(path, 'utf8');
}
const verdict = verifyEditorialSection({
  sectionId: section, originalReceipt,
  passOutput: revisionArtifact.revisedOutput,
  passOutputHash: revisionArtifact.revisedOutputHash,
  evidence: {
    baseReceiptRawBytes,
    baseReceiptSha256: createHash('sha256').update(baseReceiptRawBytes).digest('hex'),
    revisionArtifact, freshAudit, freshJev, guardrailDisposition,
  },
  implementationSourceTexts,
});
const path = join(dir, section + '.component-verification-' + new Date().toISOString().replace(/:/g, '-') + '.json');
writeFileSync(path, JSON.stringify({section, version, auditFile, jevFile, dispositionFile, verdict, finalAccepted: false}, null, 2) + '\n', {flag: 'wx'});
console.log(JSON.stringify({path, verdict, finalAccepted: false}));
