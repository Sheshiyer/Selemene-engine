import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { createJevClient, loadTypesafeKey, noul } from '../../packages/witness-pipeline/src/jev/client.js';
import { SENTENCE_QUESTION } from '../../packages/witness-pipeline/src/jev/pass-gate.js';
const [dir, section, version='editorial'] = process.argv.slice(2);
if (!dir || !/^(opening|part(?:[1-9]|10|11))$/.test(section??'') || !/^(editorial|v[1-9][0-9]*)$/.test(version)) throw new Error('invalid arguments');
const revision=JSON.parse(readFileSync(join(dir,section+'.editorial-revision'+(version==='editorial'?'':'-'+version)+'.json'),'utf8'));
const hash=(s:string)=>createHash('sha256').update(s).digest('hex');
if(hash(revision.revisedOutput)!==revision.revisedOutputHash) throw new Error('output hash mismatch');
const units=revision.revisedOutput.split(/\n/).map((text:string,index:number)=>({text,index})).filter((u:any)=>u.text.trim());
const key=await loadTypesafeKey(); if(!key) throw new Error('Jev key missing');const client=createJevClient(key);
const batches:any[]=[];const flags:any[]=[];const missing:number[]=[];
for(let start=0;start<units.length;start+=20){
 const batch=units.slice(start,start+20);const [clause,criteria]=SENTENCE_QUESTION.descriptive;
 const questions=Object.fromEntries(batch.map((u:any)=>['u'+u.index,noul('Is it true that report unit '+u.index+' '+clause+'?',criteria)]));
 const state={section:revision.revisedOutput,units:batch};
 const result=await client({state,questions}); batches.push({state_sha256:hash(JSON.stringify(state)),units:batch,questions,result});
 for(const u of batch){ const a=result.answers?.['u'+u.index];if(!a||a.type!=='noul'||typeof a.noul!=='number'||!Number.isFinite(a.noul)||a.noul<0||a.noul>1)missing.push(u.index);else if(a.noul>=0.5)flags.push({...u,score:a.noul}); }
}
const artifact={kind:'full-nonblank-line-guardrail-diagnostic',section,revisionVersion:version,sectionOutputHash:revision.revisedOutputHash,policy:'descriptive',unitCount:units.length,complete:missing.length===0,missing,flags,batches,finalAccepted:false};
const path=join(dir,section+'.guardrail-diagnostic-'+new Date().toISOString().replace(/:/g,'-')+'.json');writeFileSync(path,JSON.stringify(artifact,null,2)+'\n',{flag:'wx'});console.log(JSON.stringify({path,units:units.length,complete:artifact.complete,flags}));
