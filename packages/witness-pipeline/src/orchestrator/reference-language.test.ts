import {describe,it,expect} from 'vitest';
import {fileURLToPath} from 'node:url';
import {parseModeDoc} from '../modes/parser.js';
import {localizeReferenceTitles} from './reference-language.js';
const load=()=>parseModeDoc(fileURLToPath(new URL('../../modes/integrated-kundali-reference.md',import.meta.url)));
describe('reference language contract',()=>{
 it('localizes all twelve canonical sections without changing ids or word budgets',()=>{
  const mode=load();const contract=mode.frontmatter.pass_plan.map(({id,target_words,template})=>({id,target_words,template}));
  localizeReferenceTitles(mode,'fr');
  expect(mode.frontmatter.pass_plan.map(({id,target_words,template})=>({id,target_words,template}))).toEqual(contract);
  expect(mode.frontmatter.pass_plan).toHaveLength(12);
  expect(mode.frontmatter.pass_plan[6].title).toBe('Partie VI — Amour et vie de couple');
  expect(mode.frontmatter.pass_plan[11].title).toBe('Partie XI — Synthèse finale');
  expect(new Set(mode.frontmatter.pass_plan.map(p=>p.title)).size).toBe(12);
 });
 it('preserves the English canonical document',()=>{
  const mode=load();const before=JSON.stringify(mode);localizeReferenceTitles(mode,'en');expect(JSON.stringify(mode)).toBe(before);
 });
 it('rejects unsupported language and unknown section contracts',()=>{
  expect(()=>localizeReferenceTitles(load(),'de')).toThrow('Unsupported');
  expect(()=>localizeReferenceTitles({frontmatter:{pass_plan:[{id:'invented',title:'x'}]}},'fr')).toThrow('Unknown');
 });
});
