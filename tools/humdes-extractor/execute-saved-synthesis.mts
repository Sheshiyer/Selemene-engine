/** Execute a separately verified synthesis-only recovery. Original files remain immutable. */
import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {createHash} from 'node:crypto';
import {loadFailedRun,reconstructSectionInput} from './recover-reference-synthesis.mts';
import {executeReferenceSection,validateVoiceReuseEvidence} from '../../packages/witness-pipeline/src/orchestrator/reference-execution.js';
import type {RetrievedPassage,PassageRetrievalReceipt} from '../../packages/witness-pipeline/src/orchestrator/reference-execution.js';
import {isPrimarySourceId} from '../../packages/witness-pipeline/src/orchestrator/primary-passage-source.js';
import {validatePrimaryPassageAdditions,type PrimaryPassageRegistryFile} from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';
import {IntegratedReadingOrchestrator} from '../../packages/witness-pipeline/src/orchestrator/integrated.js';
import {parseModeDoc} from '../../packages/witness-pipeline/src/modes/parser.js';
import {localizeReferenceTitles} from '../../packages/witness-pipeline/src/orchestrator/reference-language.js';
import {VectorizeCorpusAdapter} from '../../packages/witness-pipeline/src/grounding/index.js';
import {createOmniRouteLlmCall} from '../../packages/witness-pipeline/scripts/lib/multi-llm.js';
import {createJevClient,loadTypesafeKey} from '../../packages/witness-pipeline/src/jev/client.js';
import {createJevPassGate} from '../../packages/witness-pipeline/src/jev/pass-gate.js';
const args=process.argv.slice(2);const val=(key:string,fallback?:string)=>{const i=args.indexOf(key);return i<0?fallback:args[i+1];};
const runArg=val('--run-dir');const section=val('--section');
if(!runArg||!section||!/^part(?:[1-9]|10|11)$/.test(section))throw new Error('--run-dir and --section partN required');
const repo=process.cwd();const run=resolve(runArg);const hash=(s:string)=>createHash('sha256').update(s).digest('hex');
const descriptor=await loadFailedRun(run,section);const saved=descriptor.parsedReceipt;
const inputs=JSON.parse(readFileSync(join(run,'inputs.json'),'utf8'));
for(const path of ['packages/witness-pipeline/modes/integrated-kundali-reference.md','packages/witness-pipeline/src/orchestrator/integrated.ts']) {
 if(inputs.runtimeHashes?.[path]!==hash(readFileSync(join(repo,path),'utf8')))throw new Error('Original system-prompt reconstruction code changed: '+path);
}
const snapshotText=readFileSync(join(run,'input.snapshot.json'),'utf8');
if(hash(snapshotText)!==inputs.inputHash)throw new Error('Input snapshot hash mismatch');
const engines=JSON.parse(snapshotText);
const rebuilt=reconstructSectionInput(descriptor,repo);const input=rebuilt.input;
// Generic horizontal rules also occur inside chapters. Only recorded chapter
// wrappers, in manifest order, may delimit the historical prior sections.
const marker='## Accepted Prior Sections (full, untruncated)\n';
const start=saved.synthesis_input_text.indexOf(marker);
const end=saved.synthesis_input_text.indexOf('\n\n## Aletheios (structural interpretation)\n',start);
if(start<0||end<start)throw new Error('Missing historical prior-context block');
const block=saved.synthesis_input_text.slice(start+marker.length,end);
const titles=Object.keys(saved.prior_section_content_hashes);
const mode=parseModeDoc(join(repo,'packages/witness-pipeline/modes/integrated-kundali-reference.md'));
localizeReferenceTitles(mode, inputs.language);
const targetIndex=mode.frontmatter.pass_plan.findIndex(p=>p.id===section);
const expectedTitles=mode.frontmatter.pass_plan.slice(0,targetIndex).map(p=>p.title);
if(JSON.stringify(titles)!==JSON.stringify(expectedTitles))throw new Error('Historical chapter titles/order mismatch');
let cursor=0;
input.acceptedPriorSections=titles.map((title,i)=>{
 const header=`## ${title}\n\n`;
 if(!block.startsWith(header,cursor))throw new Error('Historical chapter wrapper missing: '+title);
 const delimiter=i+1<titles.length?`\n\n---\n\n## ${titles[i+1]}\n\n`:undefined;
 const boundary=delimiter?block.indexOf(delimiter,cursor+header.length):block.length;
 if(boundary<0)throw new Error('Historical next chapter boundary missing');
 const text=block.slice(cursor,boundary);
 if(hash(text)!==saved.prior_section_content_hashes[title])throw new Error('Historical chapter hash mismatch: '+title);
 cursor=boundary+(delimiter?'\n\n---\n\n'.length:0);return text;
});
const pass=mode.frontmatter.pass_plan[targetIndex];
input.passSpec={...input.passSpec,template:mode.sections[pass.template],enforceWordFit:true};
const noWitnessCall=async()=>{throw new Error('Witness regeneration is forbidden during synthesis recovery');};
const orch=new IntegratedReadingOrchestrator({mode,llm:noWitnessCall});
// This calls the original builder only after its saved code hash is verified.
input.systemPrompt=(orch as any).buildSystemPrompt(pass,{subjectNames:input.subjectNames,subjectRoles:[{name:input.subjectNames[0],role:'primary'}],language:inputs.language,consciousnessLevel:4},'l4_l5');
input.maxTokensForSynthesis=Math.max(4096,Math.round(pass.target_words*1.2*10));
input.voiceReuse={rawReceiptBytes:descriptor.rawReceiptBytes,rawReceiptSha256:hash(descriptor.rawReceiptBytes),parsedReceipt:saved,failedReceiptPath:join(run,section+'.receipt.json')};
const persona=(name:string)=>{const sourcePath=resolve(repo,'../witness-agents/agents',name,'IDENTITY.md');const identityText=readFileSync(sourcePath,'utf8');return {name,sourcePath,identityText,sourceHash:hash(identityText)};};
const adapter=new VectorizeCorpusAdapter({accountId:inputs.accountId,indexName:inputs.index,timeoutMs:45000});
// ── Mixed-source recovery: split saved ids into CF vs primary ────────
// A saved receipt may include primary (reviewed-primary-document) ids
// alongside CF ids. Live CF retrieval remains mandatory for CF ids;
// primary ids are loaded from the independently-supplied registry (never
// from CF, never trusted from the receipt itself).
const savedPrimaryIds:string[]=Array.isArray((saved as any).primary_source_ids)?[...(saved as any).primary_source_ids]:[];
const primaryFromReceipt=new Set(savedPrimaryIds);
for(const id of savedPrimaryIds){if(!isPrimarySourceId(id))throw new Error(`saved receipt primary_source_ids contains reserved-prefix id: ${id}`);}
for(const id of saved.source_ids){if(!primaryFromReceipt.has(id)&&!id.startsWith('sw:'))throw new Error(`saved receipt id "${id}" is not sw:-prefixed CF and not declared primary — provenance ambiguous`);}
const cfIds=saved.source_ids.filter(id=>!primaryFromReceipt.has(id));
let primaryTextsById:Record<string,string>={};
const primaryUrlsById:Record<string,string>={};
let primaryRegistryEvidence:unknown;
if(savedPrimaryIds.length>0){
  const registryArg=val('--primary-registry');
  const snapshotPath=join(run,'primary-passage-registry.snapshot.json');
  let registryPath:string;let registryBytes:string;
  if(registryArg){registryPath=resolve(registryArg);registryBytes=readFileSync(registryPath,'utf8');}
  else if(existsSync(snapshotPath)){registryPath=snapshotPath;registryBytes=readFileSync(snapshotPath,'utf8');}
  else throw new Error('saved receipt declares primary_source_ids but no --primary-registry given and no primary-passage-registry.snapshot.json alongside run — pass --primary-registry <path>');
  const registrySha256=hash(registryBytes);
  const recorded=(inputs as any).primaryRegistry;
  if(recorded&&typeof recorded.sha256==='string'&&recorded.sha256!==registrySha256)throw new Error(`primary-registry bytes hash mismatch vs inputs.json.primaryRegistry.sha256: got ${registrySha256} expected ${recorded.sha256}`);
  let parsed:PrimaryPassageRegistryFile;
  try{parsed=JSON.parse(registryBytes) as PrimaryPassageRegistryFile;}
  catch{throw new Error('primary-registry file is not valid JSON');}
  const validated=validatePrimaryPassageAdditions({registry:parsed,reservedCFIds:cfIds});
  for(const id of savedPrimaryIds){
    const rec=validated.additionsById[id];
    if(!rec)throw new Error(`primary-registry missing record for saved primary id "${id}"`);
    const idx=saved.source_ids.indexOf(id);
    if(idx<0||hash(rec.text)!==saved.source_hashes[idx])throw new Error(`primary-registry text hash mismatch for id "${id}"`);
    primaryTextsById[id]=rec.text;
    if(rec.provenance.kind!=='reviewed-primary-document')throw new Error('Unexpected primary provenance');
    primaryUrlsById[id]=rec.provenance.url;
  }
  primaryRegistryEvidence={registryPath,registrySha256,registryBytes};
  console.log(JSON.stringify({stage:'primary-registry-loaded-for-recovery',path:registryPath,sha256:registrySha256,ids:savedPrimaryIds.length}));
}
let cfById:Record<string,string>={};
let cfReadback:unknown;
if(cfIds.length>0){
  const cf=await adapter.getVectors(cfIds);
  cfReadback=cf;
  if(cf.state!=='success')throw new Error('Fresh CF readback failed: '+cf.state);
  for(let i=0;i<cfIds.length;i++){
    const id=cfIds[i];
    const p=cf.passages.find(pp=>pp.id===id);
    const idx=saved.source_ids.indexOf(id);
    if(!p||idx<0||hash(p.text)!==saved.source_hashes[idx])throw new Error('Fresh CF source mismatch: '+id);
    cfById[id]=p.text;
  }
}
const passages:RetrievedPassage[]=saved.source_ids.map(id=>{
  if(primaryFromReceipt.has(id))return {id,text:primaryTextsById[id],source:`reviewed-primary-document:${primaryUrlsById[id]}`};
  return {id,text:cfById[id],source:`cf-vectorize:${inputs.index}/${id}`};
});
const passageHash=hash(passages.map(p=>p.text).join('\n'));if(passageHash!==saved.passages_hash)throw new Error('Fresh corpus hash mismatch');
const dep={enabled:true,aletheiosPersona:persona('aletheios'),pichetPersona:persona('pichet'),retriever:async(sectionId:string)=>{
  const receipt:PassageRetrievalReceipt={section_id:sectionId,state:'success' as const,count:passages.length,source_ids:passages.map(p=>p.id),passages_hash:passageHash};
  if(savedPrimaryIds.length>0){receipt.primary_source_ids=[...savedPrimaryIds];receipt.cf_source_ids=cfIds;}
  return {passages,receipt};
}};
const validation=validateVoiceReuseEvidence(input.voiceReuse,dep,input);if(!validation.ok)throw new Error(JSON.stringify(validation));
const out=join(run,'recovered',section+'-'+new Date().toISOString().replace(/[:.]/g,'-'));mkdirSync(out,{recursive:true});
const save=(file:string,data:unknown)=>writeFileSync(join(out,file),JSON.stringify(data,null,2)+'\n');
save('preflight.json',{section,validation,priorSections:titles,sourceCount:passages.length,cfCount:cfIds.length,primaryCount:savedPrimaryIds.length,originalReceiptHash:hash(descriptor.rawReceiptBytes),systemPromptHash:hash(input.systemPrompt),finalAccepted:false});save('corpus.json',{cfIds,primaryIds:savedPrimaryIds,cfReadback,primaryRegistryEvidence});
writeFileSync(join(out,'original.receipt.json'),descriptor.rawReceiptBytes);save('reconstructed-input.json',input);
console.log(JSON.stringify({stage:'recovery-preflight',out,priorSections:titles.length,sources:passages.length,cf:cfIds.length,primary:savedPrimaryIds.length,valid:true}));
if(!args.includes('--execute'))process.exit(0);
const key=await loadTypesafeKey();if(!key)throw new Error('Jev key required');
const llmReceipts:unknown[]=[];const onReceipt=(receipt:unknown)=>{llmReceipts.push(receipt);save('llm-receipts.json',llmReceipts);};
const synthesis=createOmniRouteLlmCall(val('--combo','noesis-build')!,{temperature:0.4,timeout_ms:240000,onReceipt});
const auditor=createOmniRouteLlmCall('noesis-verify',{temperature:0,timeout_ms:180000,onReceipt});
const result=await executeReferenceSection(input,{...dep,synthesisLlm:synthesis,sourceAuditLlm:(system,user)=>auditor(system,user,{max_tokens:16384}),requireSourceAudit:true},noWitnessCall,{jevGate:createJevPassGate(createJevClient(key),'shadow'),allEngineResults:engines,guardrailPolicy:mode.frontmatter.jev_guardrail??'descriptive',register:'l4_l5',checkpoint:async r=>save(section+'.receipt.json',r)});
save('result.json',result);save('status.json',{outcome:result.receipt.outcome,voicesReused:true,finalAccepted:false});
console.log(JSON.stringify({stage:'recovery-finished',out,outcome:result.receipt.outcome,jev:result.jev?.verdicts,finalAccepted:false}));
if(result.receipt.outcome==='failed')process.exitCode=1;
