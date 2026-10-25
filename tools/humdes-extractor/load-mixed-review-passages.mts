import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {VectorizeCorpusAdapter} from '../../packages/witness-pipeline/src/grounding/index.js';
import {validatePrimaryPassageAdditions} from '../../packages/witness-pipeline/src/orchestrator/editorial-passage-evidence.js';
import type {SectionExecutionReceipt,RetrievedPassage} from '../../packages/witness-pipeline/src/orchestrator/reference-execution.js';
const hash=(s:string)=>createHash('sha256').update(s).digest('hex');
/** Reconstruct the exact mixed generation evidence, preserving source origins. */
export async function loadMixedReviewPassages(dir:string, base:SectionExecutionReceipt):Promise<RetrievedPassage[]> {
 const meta=JSON.parse(readFileSync(join(dir,'inputs.json'),'utf8'));
 const primaryIds=base.primary_source_ids??[];
 if(!primaryIds.length)throw new Error('Mixed review requires primary IDs');
 if(base.source_hashes.length!==base.source_ids.length)throw new Error('Source hash cardinality mismatch');
 if(new Set(base.source_ids).size!==base.source_ids.length||new Set(primaryIds).size!==primaryIds.length)throw new Error('Duplicate source IDs');
 const ids=new Set(primaryIds);const cfIds=base.source_ids.filter(id=>!ids.has(id));
 if(!cfIds.length||cfIds.some(id=>!id.startsWith('sw:'))||primaryIds.some(id=>!base.source_ids.includes(id)))throw new Error('Invalid source partition');
 if(base.cf_source_ids && JSON.stringify(base.cf_source_ids)!==JSON.stringify(cfIds))throw new Error('Declared CF partition mismatch');
 const binding=meta.primaryRegistry;
 if(!binding?.path||!binding?.sha256)throw new Error('Independent primary registry binding missing');
 const bytes=readFileSync(binding.path,'utf8');
 if(hash(bytes)!==binding.sha256)throw new Error('Primary registry bytes changed');
 const primary=validatePrimaryPassageAdditions({registry:JSON.parse(bytes),reservedCFIds:cfIds});
 const cf=await new VectorizeCorpusAdapter({accountId:meta.accountId,indexName:meta.index,timeoutMs:45000}).getVectors(cfIds);
 if(cf.state!=='success')throw new Error('Fresh CF readback unavailable: '+cf.state);
 const passages=base.source_ids.map((id,i)=>{
  const record=ids.has(id)?primary.additionsById[id]:cf.passages.find(p=>p.id===id);
  if(!record||hash(record.text)!==base.source_hashes[i])throw new Error('Generation source changed or missing: '+id);
  const addition=primary.additionsById[id];
  return {id,text:record.text,source:ids.has(id)&&addition.provenance.kind==='reviewed-primary-document'?`reviewed-primary-document:${addition.provenance.url}`:`cf-vectorize:${meta.index}/${id}`};
 });
 if(hash(passages.map(p=>p.text).join('\n'))!==base.passages_hash)throw new Error('Combined generation source hash mismatch');
 writeFileSync(join(dir,`${base.section_id}.mixed-review-readback-${Date.now()}.json`),JSON.stringify({registryPath:binding.path,registrySha256:hash(bytes),cf,combinedHash:base.passages_hash,primaryIds,finalAccepted:false},null,2)+'\n',{flag:'wx'});
 return passages;
}
