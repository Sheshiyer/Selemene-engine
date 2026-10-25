import {loadMixedReviewPassages} from './load-mixed-review-passages.mjs';
import { referenceModeFile, triadEngineFactsFromSnapshots } from '../../packages/witness-pipeline/src/orchestrator/reference-mode-contract.js';
import { BUSINESS_TRIAD_SLUGS, BUSINESS_TRIAD_DISPLAY_NAMES, localizeBusinessTriadTitles } from '../../packages/witness-pipeline/src/orchestrator/business-triad-support.js';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { localizeReferenceTitles } from '../../packages/witness-pipeline/src/orchestrator/reference-language.js';
import { parseModeDoc } from '../../packages/witness-pipeline/src/modes/parser.js';
import { createJevClient, loadTypesafeKey } from '../../packages/witness-pipeline/src/jev/client.js';
import { createJevPassGate } from '../../packages/witness-pipeline/src/jev/pass-gate.js';
import { auditSectionOutput } from '../../packages/witness-pipeline/src/orchestrator/rubric.js';
import { VectorizeCorpusAdapter } from '../../packages/witness-pipeline/src/grounding/index.js';
import { validateEditorialPassageEvidence, type TrustedSourceRecord } from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';
const argv = process.argv.slice(2);
const [dir, section, version='editorial'] = argv;
const registryFlag = argv.indexOf('--passage-registry');
const registryPath = registryFlag < 0 ? undefined : argv[registryFlag + 1];
if (registryFlag >= 0 && !registryPath) throw new Error('--passage-registry requires a path');
if (!dir || !/^(opening|part(?:[1-9]|10|11))$/.test(section ?? '') || !/^(editorial|v[1-9][0-9]*)$/.test(version)) throw new Error('run directory, section and optional revision version required');
const revision = JSON.parse(readFileSync(join(dir, section+'.editorial-revision'+(version==='editorial'?'':'-'+version)+'.json'),'utf8'));
const base = JSON.parse(readFileSync(join(dir, section+'.receipt.json'),'utf8'));
const hash=(s:string)=>createHash('sha256').update(s).digest('hex');
if(revision.baseOutputHash!==base.output_hash || hash(revision.revisedOutput)!==revision.revisedOutputHash) throw new Error('revision hash mismatch');
const meta = JSON.parse(readFileSync(join(dir,'inputs.json'),'utf8'));
const modeFile=referenceModeFile(meta);
const isTriad=modeFile==='business-triad-reference.md';
const subjectNames=isTriad ? BUSINESS_TRIAD_SLUGS.map(s=>BUSINESS_TRIAD_DISPLAY_NAMES[s]) : [BUSINESS_TRIAD_DISPLAY_NAMES[meta.subject as keyof typeof BUSINESS_TRIAD_DISPLAY_NAMES]];
let engines;
if(isTriad){
 const snapshots=Object.fromEntries(BUSINESS_TRIAD_SLUGS.map(slug=>[slug,readFileSync(join(dir,`input.${slug}.snapshot.json`),'utf8')]));
 if(base.engine_facts!==triadEngineFactsFromSnapshots(meta,snapshots))throw new Error('Triad engine facts differ from subject snapshots');
 // Only the deterministic counting rubric consumes this flattened view;
 // the reviewer receives the independently bound named source blocks.
 engines=BUSINESS_TRIAD_SLUGS.flatMap(slug=>JSON.parse(snapshots[slug]));
}else engines=JSON.parse(readFileSync(join(dir,'input.snapshot.json'),'utf8'));
const mode=parseModeDoc(join('packages/witness-pipeline/modes',modeFile));
if(isTriad)localizeBusinessTriadTitles(mode,meta.language);
else localizeReferenceTitles(mode,meta.language);
const pass=mode.frontmatter.pass_plan.find(p=>p.id===section);if(!pass) throw new Error('Unknown section');
const corpus=JSON.parse(readFileSync(join(dir,section+'.corpus.json'),'utf8'));
if(base.primary_source_ids?.length&&registryPath)throw new Error('Mixed generation already includes its primary sources; additional editorial supplements require separate support');
let passages=base.primary_source_ids?.length ? await loadMixedReviewPassages(dir,base) : corpus.passages.map((p:any)=>({id:p.id,text:p.text,source:'cf-vectorize:witness-wisdom-corpus/'+p.id}));
let passageEvidence;
if (registryPath) {
  const registryBytes = readFileSync(registryPath, 'utf8');
  const registry = JSON.parse(registryBytes) as { records: Record<string, TrustedSourceRecord>; additionIds: string[] };
  if (!registry.records || !Array.isArray(registry.additionIds)) throw new Error('Independent registry invalid');
  const additions = registry.additionIds.map(id => { const r = registry.records[id]; if (!r) throw new Error('Registry record missing: '+id); return r; });
  const cfAdditions = additions.filter(a => a.provenance.kind === 'cf-vectorize');
  const cf = await new VectorizeCorpusAdapter({accountId:meta.accountId,indexName:meta.index,timeoutMs:45000})
    .getVectors([...new Set<string>([...base.source_ids,...cfAdditions.map(a=>a.id)])]);
  if (cf.state !== 'success') throw new Error('Fresh CF readback failed: '+cf.state);
  for (let i=0;i<base.source_ids.length;i++) {
    const live=cf.passages.find(p=>p.id===base.source_ids[i]);
    if (!live || hash(live.text)!==base.source_hashes[i] || passages[i]?.id!==live.id) throw new Error('Original CF mismatch');
  }
  for (const added of cfAdditions) {
    const live=cf.passages.find(p=>p.id===added.id);
    if (!live || live.text!==added.text || hash(live.text)!==added.sha256) throw new Error('Added CF mismatch: '+added.id);
  }
  const validated=validateEditorialPassageEvidence({expectedCF:{account:meta.accountId,index:meta.index},basePassages:passages,
    basePassagesHash:base.passages_hash,additions,trustedSourceRecords:registry.records,
    trustedOriginalTexts:Object.fromEntries(cf.passages.map(p=>[p.id,p.text]))});
  passages=validated.effectivePassages.map(p=>({...p}));
  passageEvidence={effectivePassagesHash:validated.effectivePassagesHash,registrySha256:hash(registryBytes),cfReadback:cf};
}
const key=await loadTypesafeKey();if(!key) throw new Error('Jev key missing');
const rubric=auditSectionOutput({sectionId:section,title:pass.title,targetWords:pass.target_words,output:revision.revisedOutput,modelRequested:'editorial-revision',modelUsed:'assistant-source-review',latencyMs:0,engineResults:engines});
const receipt=await createJevPassGate(createJevClient(key),'shadow').judge({passId:section,passTitle:pass.title,output:revision.revisedOutput,register:'l4_l5',subjectNames,...(isTriad?{relationshipType:'business-partners'}:{}),engineFacts:base.engine_facts,rubric,guardrailPolicy:mode.frontmatter.jev_guardrail,frameworkPassages:passages,fullContext:true});
const path=join(dir,section+'.editorial-jev-'+new Date().toISOString().replace(/:/g,'-')+'.json');
writeFileSync(path,JSON.stringify({sectionOutputHash:revision.revisedOutputHash,revisionVersion:version,rubric,receipt,...(passageEvidence?{passageEvidence}:{}),finalAccepted:false},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({path,verdicts:receipt.verdicts,status:receipt.status}));
