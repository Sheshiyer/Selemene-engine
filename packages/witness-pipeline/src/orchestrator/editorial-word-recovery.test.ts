import {describe,it,expect} from 'vitest';
import {createHash} from 'node:crypto';
import {classifyOriginalOutcome,hasRecoveredWordFit} from './editorial-verification.js';
import type {SectionExecutionReceipt} from './reference-execution.js';
const hash=(s:string)=>createHash('sha256').update(s).digest('hex');
function fixture():SectionExecutionReceipt {
 return {outcome:'failed',raw:{aletheios:'structural voice',pichet:'experiential voice',synthesis:'saved draft'},
 aletheios_hash:hash('structural voice'),pichet_hash:hash('experiential voice'),output_hash:hash('saved draft'),
 attempts:[{outcome:'synthesis_transient_retry',reason:'transient transport error'},
 {outcome:'word_count'}, {outcome:'synthesis_error',reason:'The operation was aborted due to timeout'}]} as SectionExecutionReceipt;
}
describe('bounded editorial word-count recovery',()=>{
 it('classifies the saved draft without rewriting its failed outcome',()=>{const r=fixture();expect(classifyOriginalOutcome(r).kind).toBe('word_count_recovery');expect(r.outcome).toBe('failed');});
 it.each(['empty','hash','voice','only-timeout','unknown','early-error','nontransport','null'])('rejects unsafe evidence: %s',kind=>{
  const r=fixture();
  if(kind==='empty')r.raw.synthesis='';
  if(kind==='hash')r.output_hash=hash('other');
  if(kind==='voice')r.raw.pichet='';
  if(kind==='only-timeout')r.attempts=[r.attempts[2]];
  if(kind==='unknown')(r.attempts[1] as any).outcome='retrieval_failure';
  if(kind==='early-error')r.attempts=[r.attempts[2],r.attempts[1]];
  if(kind==='nontransport')r.attempts[0].reason='content failure';
  if(kind==='null')(r.attempts as any[])[0]=null;
  expect(classifyOriginalOutcome(r).kind).toBe('blocked');
 });
 it('recounts output and rejects missing, fake or failing rubric values',()=>{
  const text='one two three four five';const rubric={target_words:5,actual_words:5,word_count_fit:'pass'};
  expect(hasRecoveredWordFit(text,rubric)).toBe(true);
  expect(hasRecoveredWordFit(text,undefined)).toBe(false);
  expect(hasRecoveredWordFit(text,{...rubric,actual_words:4})).toBe(false);
  expect(hasRecoveredWordFit(text,{...rubric,word_count_fit:'fail'})).toBe(false);
  expect(hasRecoveredWordFit(text,{...rubric,target_words:10})).toBe(false);
 });
});

function reauditFixture(): SectionExecutionReceipt {
 const r = fixture(); r.section_id='part11'; r.engine_facts='facts'; r.passages_hash=hash('passages');
 const canonical=`${r.raw.synthesis}\n---SOURCES---\n${r.passages_hash}\n---ENGINE---\n${hash(r.engine_facts)}`;
 r.attempts=[{outcome:'word_count'},{outcome:'ok'}, {
  outcome:'source_audit_reaudit_failed', synthesis_raw:r.raw.synthesis,
  aletheios_raw:r.raw.aletheios,pichet_raw:r.raw.pichet,
 }] as any;
 r.source_audit={section_id:'part11',status:'changes-required',model_raw:'audit',output_sha256:hash('audit'),
  input_sha256:hash(canonical),findings:[{severity:'error'}]} as any;
 return r;
}
describe('bounded saved source re-audit recovery',()=>{
 it('accepts intact terminal evidence without changing the original',()=>{
  const r=reauditFixture(); const before=JSON.stringify(r);
  expect(classifyOriginalOutcome(r).kind).toBe('source_audit_reaudit_recovery');
  expect(JSON.stringify(r)).toBe(before);
 });
 it.each(['nonterminal','mixed','unknown','missing-ok','voice','draft-hash','terminal-draft','audit-hash','audit-input','clean-audit','no-findings','null'])('rejects %s',kind=>{
  const r=reauditFixture();
  if(kind==='nonterminal')r.attempts.push({outcome:'ok'} as any);
  if(kind==='mixed')r.attempts[0].outcome='source_audit_failed';
  if(kind==='unknown')r.attempts[0].outcome='unrecognized';
  if(kind==='missing-ok')r.attempts.splice(1,1);
  if(kind==='voice')r.raw.pichet='changed';
  if(kind==='draft-hash')r.output_hash=hash('changed');
  if(kind==='terminal-draft')r.attempts[2].synthesis_raw='changed';
  if(kind==='audit-hash')r.source_audit!.model_raw='changed';
  if(kind==='audit-input')r.engine_facts='changed';
  if(kind==='clean-audit')r.source_audit!.status='clean';
  if(kind==='no-findings')r.source_audit!.findings=[];
  if(kind==='null')(r.attempts as any[])[0]=null;
  expect(classifyOriginalOutcome(r).kind).toBe('blocked');
 });
});

describe('audit ledger failure after completed word repair',()=>{
 function ledgerFixture() {
  const r=reauditFixture();
  r.attempts[2].outcome='source_audit_failed';
  r.source_audit!.status='insufficient';
  r.source_audit!.findings=[];
  return r;
 }
 it('admits the intact sequence without changing the failed receipt',()=>{
  const r=ledgerFixture();const before=JSON.stringify(r);
  expect(classifyOriginalOutcome(r).kind).toBe('source_audit_after_word_fit_recovery');
  expect(JSON.stringify(r)).toBe(before);
 });
 it.each(['missing-ok','wrong-order','content-rejection','clean','audit-input','audit-bytes','voice','terminal-draft','empty-response','retrieval-failure'])('rejects %s',kind=>{
  const r=ledgerFixture();
  if(kind==='missing-ok')r.attempts.splice(1,1);
  if(kind==='wrong-order')r.attempts.push({outcome:'ok'} as any);
  if(kind==='content-rejection')r.source_audit!.status='changes-required';
  if(kind==='clean')r.source_audit!.status='clean';
  if(kind==='audit-input')r.engine_facts='changed';
  if(kind==='audit-bytes')r.source_audit!.model_raw='changed';
  if(kind==='voice')r.raw.pichet='changed';
  if(kind==='terminal-draft')r.attempts[2].synthesis_raw='changed';
  if(kind==='empty-response')r.source_audit!.model_raw='';
  if(kind==='retrieval-failure')r.attempts[1].outcome='retrieval_failure';
  expect(classifyOriginalOutcome(r).kind).toBe('blocked');
 });
});
