/** Fresh, hash-bound Jev + reference verification for an immutable run. */
import { promises as fs } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve, join } from 'node:path';
import { createJevClient, loadTypesafeKey } from '../../packages/witness-pipeline/src/jev/client.js';
import { createJevPassGate } from '../../packages/witness-pipeline/src/jev/pass-gate.js';
import { verifyReferenceExecution } from '../../packages/witness-pipeline/src/orchestrator/reference-verification.js';
import { parseModeDoc } from '../../packages/witness-pipeline/src/modes/parser.js';
import { localizeReferenceTitles } from '../../packages/witness-pipeline/src/orchestrator/reference-language.js';

const args = process.argv.slice(2);
const val = (key: string, fallback?: string) => {
  const index = args.indexOf(key);
  return index < 0 ? fallback : args[index + 1];
};
const runArg = val('--run-dir');
if (!runArg) throw new Error('Required: --run-dir <immutable reference run>');
const runDir = resolve(runArg);
const register = val('--register', 'l4_l5') as 'l1_l3' | 'l4_l5';
if (!['l1_l3', 'l4_l5'].includes(register)) throw new Error('Unsupported --register');
const guardrailPolicy = val('--guardrail', 'descriptive') as 'descriptive' | 'forecast-allowed';
if (!['descriptive', 'forecast-allowed'].includes(guardrailPolicy)) throw new Error('Unsupported --guardrail');
const sha256 = (text: string) => createHash('sha256').update(text).digest('hex');
const readText = (name: string) => fs.readFile(join(runDir, name), 'utf8');
const readJson = async (name: string) => JSON.parse(await readText(name));

const [resultBytes, inputsBytes, assembled, priorFinal] = await Promise.all([
  readText('result.json'),
  readText('inputs.json'),
  readText('reading.md'),
  readJson('final-verification.json'),
]);
const result = JSON.parse(resultBytes);
const inputs = JSON.parse(inputsBytes);
const subjectNames: string[] = Array.isArray(result.subject_names) ? result.subject_names : [];
if (!subjectNames.length) throw new Error('Run result has no subject_names');

const modePath = resolve('packages/witness-pipeline/modes/integrated-kundali-reference.md');
const mode = parseModeDoc(modePath);
localizeReferenceTitles(mode, inputs.language ?? 'en');
const requiredSectionIds = mode.frontmatter.pass_plan
  .filter(pass => result.passes.some((candidate: any) => candidate.id === pass.id))
  .map(pass => pass.id);
if (!requiredSectionIds.length) throw new Error('No run sections match the reference mode');

const receipts = await Promise.all(requiredSectionIds.map(id => readJson(`${id}.receipt.json`)));
const passById = new Map(result.passes.map((pass: any) => [pass.id, pass]));
const key = await loadTypesafeKey();
if (!key) throw new Error('Jev key required for reverification');
const gate = createJevPassGate(createJevClient(key), 'active');

const refreshedPasses = [];
const refreshedReceipts = [];
const jevBySection: Record<string, unknown> = {};
for (let index = 0; index < requiredSectionIds.length; index++) {
  const id = requiredSectionIds[index];
  const pass: any = passById.get(id);
  const receipt = receipts[index];
  if (!pass || !receipt) throw new Error(`Missing pass or receipt for ${id}`);
  const jev = await gate.judge({
    passId: id,
    passTitle: pass.title,
    output: pass.output,
    register,
    subjectNames,
    engineFacts: receipt.engine_facts,
    rubric: pass.rubric,
    guardrailPolicy,
    fullContext: true,
  });
  jevBySection[id] = jev;
  refreshedPasses.push({ ...pass, jev });
  refreshedReceipts.push({ ...receipt, jev });
}

const sourceTexts: Record<string, string> = {};
for (const id of requiredSectionIds) {
  const corpus = await readJson(`${id}.corpus.json`);
  for (const passage of corpus.passages ?? []) sourceTexts[passage.id] = passage.text;
}
try {
  const registry = await readJson('primary-passage-registry.snapshot.json');
  for (const record of Object.values(registry.records ?? {}) as any[]) {
    if (record?.id && typeof record.text === 'string') sourceTexts[record.id] = record.text;
  }
} catch (error) {
  if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
}

const firstReceipt = refreshedReceipts[0];
const [aletheiosIdentity, pichetIdentity] = await Promise.all([
  fs.readFile(firstReceipt.aletheios_persona.sourcePath, 'utf8'),
  fs.readFile(firstReceipt.pichet_persona.sourcePath, 'utf8'),
]);
const subsectionMap = Object.fromEntries(mode.frontmatter.pass_plan
  .filter(pass => requiredSectionIds.includes(pass.id))
  .map(pass => [pass.id, [...(mode.sections[pass.template] ?? '').matchAll(/^\s*-\s+(\d+\.\d+)\s/gm)].map(match => match[1])]));

const verification = verifyReferenceExecution({
  receipts: refreshedReceipts,
  passes: refreshedPasses,
  requiredSectionIds,
  expectedPersonas: { aletheios: aletheiosIdentity, pichet: pichetIdentity },
  subsectionMap,
  rawSynthesisInputs: Object.fromEntries(refreshedReceipts.map(receipt => [receipt.section_id, receipt.synthesis_input_text])),
  sourceTexts,
  assembled,
  requireSourceAudit: true,
  sectionEngineFacts: Object.fromEntries(refreshedReceipts.map(receipt => [receipt.section_id, receipt.engine_facts])),
});

const timestamp = new Date().toISOString();
const evidence = {
  schema: 'witness-reference-reverification-v1',
  timestamp,
  run_dir: runDir,
  bindings: {
    result_sha256: sha256(resultBytes),
    inputs_sha256: sha256(inputsBytes),
    assembled_sha256: sha256(assembled),
    section_output_sha256: Object.fromEntries(refreshedPasses.map(pass => [pass.id, sha256(pass.output)])),
    receipt_sha256: Object.fromEntries(await Promise.all(requiredSectionIds.map(async id => [id, sha256(await readText(`${id}.receipt.json`))]))),
  },
  register,
  guardrail_policy: guardrailPolicy,
  jev_by_section: jevBySection,
  verification,
  reference_route_verification: priorFinal,
  accepted: verification.passed && priorFinal.passed === true,
};
const filename = `reverification-${timestamp.replace(/[:.]/g, '-')}.json`;
const outputPath = join(runDir, filename);
await fs.writeFile(outputPath, JSON.stringify(evidence, null, 2), { flag: 'wx' });
console.log(JSON.stringify({ output: outputPath, accepted: evidence.accepted, verification, jev_by_section: jevBySection }));
