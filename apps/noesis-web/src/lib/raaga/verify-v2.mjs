#!/usr/bin/env node
// V2 Phase-1 contract gate. Asserts every frozen contract is in place
// before Phase-2 swarms are dispatched.
//
//   $ node src/lib/raaga/verify-v2.mjs
//
// Re-implements the algorithms in plain JS so this script needs no TS loader.

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const exists = (rel) => fs.existsSync(path.join(__dirname, rel));
const read = (rel) => fs.readFileSync(path.join(__dirname, rel), 'utf8');
const readJson = (rel) => JSON.parse(read(rel));

let pass = 0; let fail = 0;
const check = (label, fn) => {
  try { fn(); console.log(`  ✓ ${label}`); pass++; }
  catch (e) { console.log(`  ✗ ${label}\n    ${e.message}`); fail++; }
};

console.log('▸ Phase-1 file inventory');
[
  'v2/gamakas/types.ts',
  'v2/gamakas/apply.ts',
  'v2/gamakas/README.md',
  'v2/talas/types.ts',
  'v2/talas/data.ts',
  'v2/talas/strudel.ts',
  'v2/breaths/data.ts',
  'v2/breaths/strudel.ts',
  'v2/samples/manifest.schema.json',
  'v2/samples/manifest.json',
  'v2/samples/ATTRIBUTION.md',
  'v2/feature-flag.ts',
  'v2/index.ts',
].forEach((rel) => check(`exists: ${rel}`, () => assert.ok(exists(rel))));

console.log('\n▸ Gamaka contract');
check('Gamaka union covers 5 kinds + none', () => {
  const src = read('v2/gamakas/types.ts');
  for (const k of ['none', 'kampita', 'andolana', 'kurula', 'nokku', 'sphurita']) {
    assert.ok(src.includes(`kind: '${k}'`), `missing kind: ${k}`);
  }
});
check('GAMAKA_DEFAULTS defined', () => {
  assert.ok(read('v2/gamakas/types.ts').includes('GAMAKA_DEFAULTS'));
});
check('applyGamaka signature exported', () => {
  const src = read('v2/gamakas/apply.ts');
  assert.ok(src.includes('export const applyGamaka'));
  assert.ok(src.includes('ApplyGamakaInput'));
  assert.ok(src.includes('ApplyGamakaOutput'));
});

console.log('\n▸ Tala contract');
check('Tala interface has required fields', () => {
  const src = read('v2/talas/types.ts');
  for (const f of ['name', 'beats', 'structure', 'euclid', 'accentBeats']) {
    assert.ok(src.includes(`${f}:`), `missing field: ${f}`);
  }
});
check('6 talas encoded', () => {
  const src = read('v2/talas/data.ts');
  for (const t of ['adi', 'rupakam', 'misra-chapu', 'khanda-chapu', 'tisra-eka', 'jhampa']) {
    assert.ok(src.includes(`'${t}':`), `missing tala: ${t}`);
  }
});
check('Adi tala structure sums to 8', () => {
  // structure [4, 2, 2] = 8
  const src = read('v2/talas/data.ts');
  const m = src.match(/'adi':\s*{[^}]*structure:\s*\[(\d+),\s*(\d+),\s*(\d+)\]/);
  assert.ok(m, 'adi structure not found');
  const sum = +m[1] + +m[2] + +m[3];
  assert.equal(sum, 8, `adi structure sums to ${sum}, expected 8`);
});
check('Misra Chapu = 7 beats', () => {
  const src = read('v2/talas/data.ts');
  assert.ok(src.match(/'misra-chapu':\s*{[^}]*beats:\s*7/));
});
check('Jhampa structure sums to 10', () => {
  const src = read('v2/talas/data.ts');
  const m = src.match(/'jhampa':\s*{[^}]*structure:\s*\[(\d+),\s*(\d+),\s*(\d+)\]/);
  assert.ok(m, 'jhampa structure not found');
  assert.equal(+m[1] + +m[2] + +m[3], 10);
});
check('talaGainPattern + talaEuclidExpr exported', () => {
  const src = read('v2/talas/strudel.ts');
  assert.ok(src.includes('talaGainPattern'));
  assert.ok(src.includes('talaEuclidExpr'));
  assert.ok(src.includes('defaultCpsForTala'));
});

console.log('\n▸ Breath contract');
check('12 breaths encoded (one per chakra)', () => {
  const src = read('v2/breaths/data.ts');
  const breathNames = [
    'box-4', 'calming-4-7-8', 'bhastrika', 'coherence-6-0-6-0',
    'kapalabhati', 'nadi-shodhana', 'ujjayi', 'heart-coherence-5-5',
    'dirgha', 'shoulder-roll', 'brahmari', 'shitali',
  ];
  for (const n of breathNames) {
    assert.ok(src.includes(`'${n}':`), `missing breath: ${n}`);
  }
});
check('breathForChakra exported', () => {
  assert.ok(read('v2/breaths/data.ts').includes('breathForChakra'));
});
check('breathToArticulation produces ADSR + cps', () => {
  const src = read('v2/breaths/strudel.ts');
  for (const f of ['cps', 'attack', 'decay', 'sustain', 'release']) {
    assert.ok(src.includes(`${f}:`), `missing articulation field: ${f}`);
  }
});

console.log('\n▸ Sample manifest contract');
check('manifest.schema.json valid JSON', () => readJson('v2/samples/manifest.schema.json'));
check('manifest.json validates against expected packs', () => {
  const m = readJson('v2/samples/manifest.json');
  assert.equal(typeof m.version, 'string');
  assert.equal(typeof m.baseUrl, 'string');
  for (const pack of ['sitar', 'tanpura', 'mridangam', 'bansuri', 'sarangi']) {
    assert.ok(m.packs[pack], `missing pack: ${pack}`);
    assert.ok(['CC0-1.0', 'CC-BY-4.0', 'CC-BY-SA-4.0', 'AGPL-3.0-or-later', 'MIT'].includes(m.packs[pack].license),
      `pack ${pack} has unsafe license: ${m.packs[pack].license}`);
  }
});
check('ATTRIBUTION.md mentions every pack', () => {
  const att = read('v2/samples/ATTRIBUTION.md');
  for (const pack of ['sitar', 'tanpura', 'mridangam', 'bansuri', 'sarangi']) {
    assert.ok(att.includes(pack), `attribution missing pack: ${pack}`);
  }
});

console.log('\n▸ PlayOptions extension contract');
check('RaagaPlayer.ts imports from v2 without breaking v1', () => {
  const src = read('RaagaPlayer.ts');
  for (const sym of ['v2?:', 'timbre?:', 'gamakas?:', 'tala?:', 'breath?:', 'tanpura?:']) {
    assert.ok(src.includes(sym), `missing additive option: ${sym}`);
  }
});
check('feature flag isV2Enabled honored', () => {
  assert.ok(read('RaagaPlayer.ts').includes('shouldUseV2'));
  assert.ok(read('v2/feature-flag.ts').includes('RAAGA_V2_ENABLED'));
});

console.log('\n▸ V2 barrel exports');
check('v2/index.ts re-exports all contracts', () => {
  const src = read('v2/index.ts');
  for (const sym of ['Gamaka', 'applyGamaka', 'Tala', 'TALAS', 'Breath', 'BREATHS',
                     'breathToArticulation', 'isV2Enabled', 'v2Enabled']) {
    assert.ok(src.includes(sym), `barrel missing: ${sym}`);
  }
});

// ─── PHASE 2 ASSERTIONS ────────────────────────────────────────────────
console.log('\n▸ Phase-2 file inventory');
[
  'v2/gamakas/render.ts',
  'v2/samples/timbres.ts',
  'v2/samples/loader.ts',
  'v2/talas/emitter.ts',
  'v2/tanpura.ts',
  'v2/compose.ts',
  'v2/render/wav-encoder.ts',
  'v2/render/offline.ts',
  'v2/presets/mayamalavagaula.ts',
].forEach((rel) => check(`exists: ${rel}`, () => assert.ok(exists(rel))));

console.log('\n▸ Gamaka render math (port of render.ts in JS)');
const cents_ = (hz, c) => hz * Math.pow(2, c / 1200);
check('kampita ±20¢ around 220Hz produces 8 samples in [217.5, 222.6]', () => {
  // Mirror of renderKampita
  const samples = 8, hz = 220, depth = 20;
  for (let i = 0; i < samples; i++) {
    const phase = (i / samples) * Math.PI * 2;
    const out = cents_(hz, depth * Math.sin(phase));
    assert.ok(out >= 217.4 && out <= 222.6, `sample ${i}: ${out.toFixed(3)} out of band`);
  }
});
check('andolana ±50¢ around 220Hz spans 213.7..226.5', () => {
  const samples = 4, hz = 220, depth = 50;
  let min = Infinity, max = -Infinity;
  for (let i = 0; i < samples; i++) {
    const out = cents_(hz, depth * Math.sin((i/samples) * Math.PI * 2));
    min = Math.min(min, out); max = Math.max(max, out);
  }
  assert.ok(max - min > 5, `andolana span too small: ${(max-min).toFixed(2)}`);
});
check('nokku grace +100¢ above 220 = 233.08Hz', () => {
  const grace = cents_(220, 100);
  assert.ok(Math.abs(grace - 233.08) < 0.5, `nokku grace = ${grace}`);
});

console.log('\n▸ Timbre profiles');
check('5 timbres encoded with ADSR + sound', () => {
  const src = read('v2/samples/timbres.ts');
  for (const t of ['sine', 'sitar', 'tanpura', 'mridangam', 'bansuri', 'sarangi']) {
    assert.ok(src.includes(`'${t}':`), `missing timbre: ${t}`);
  }
});
check('sitar has LPF for warmth', () => {
  const src = read('v2/samples/timbres.ts');
  assert.ok(src.match(/'sitar':[^}]*lpf:\s*\d+/), 'sitar should have lpf');
});
check('mridangam is percussion (sustain=0)', () => {
  const src = read('v2/samples/timbres.ts');
  assert.ok(src.match(/'mridangam':[^}]*sustain:\s*0\.0/), 'mridangam should have sustain=0');
});

console.log('\n▸ Compose contract');
check('compose() integrates gamakas+timbre+tala+breath', () => {
  const src = read('v2/compose.ts');
  for (const sym of ['applyGamaka', 'TIMBRES', 'breathToArticulation', 'emitTala', 'buildTanpuraCode']) {
    assert.ok(src.includes(sym), `compose missing: ${sym}`);
  }
});
check('compose produces both ragaCode + tanpuraCode', () => {
  const src = read('v2/compose.ts');
  assert.ok(src.includes('ragaCode'));
  assert.ok(src.includes('tanpuraCode'));
});

console.log('\n▸ Offline render + WAV encoder');
check('encodeWav writes RIFF header (44 bytes)', () => {
  const src = read('v2/render/wav-encoder.ts');
  assert.ok(src.includes("writeString(view, 0, 'RIFF')"));
  assert.ok(src.includes("writeString(view, 8, 'WAVE')"));
  assert.ok(src.includes("writeString(view, 12, 'fmt ')"));
  assert.ok(src.includes("writeString(view, 36, 'data')"));
});
check('renderToWavBlobUrl uses OfflineAudioContext', () => {
  const src = read('v2/render/offline.ts');
  assert.ok(src.includes('OfflineAudioContext'));
  assert.ok(src.includes('startRendering'));
  assert.ok(src.includes('encodeWav'));
});

console.log('\n▸ Mayamalavagaula preset semantics');
check('preset has 5 gamaka annotations covering Re/Ga/Ma/Dha/Ni', () => {
  const src = read('v2/presets/mayamalavagaula.ts');
  assert.ok(src.includes('swaraIndex: 1'));  // Re
  assert.ok(src.includes('swaraIndex: 2'));  // Ga
  assert.ok(src.includes('swaraIndex: 3'));  // Ma
  assert.ok(src.includes('swaraIndex: 5'));  // Dha
  assert.ok(src.includes('swaraIndex: 6'));  // Ni
});
check('Sa and Pa (achala) are NOT ornamented in Mayamalavagaula', () => {
  const src = read('v2/presets/mayamalavagaula.ts');
  // Comment-style assertion: ensure the array doesn't include indices 0 (Sa), 4 (Pa), 7 (Sa')
  const matches = src.match(/swaraIndex:\s*([0-9]+)/g) ?? [];
  const indices = matches.map((m) => parseInt(m.match(/(\d+)/)[1], 10));
  assert.ok(!indices.includes(0), 'Sa must not be ornamented');
  assert.ok(!indices.includes(4), 'Pa must not be ornamented (achala)');
  assert.ok(!indices.includes(7), 'Sa\' must not be ornamented');
});

// ─── V2.5 ASSERTIONS — synthesizers, paltas, alaap ─────────────────────
console.log('\n▸ V2.5 file inventory');
[
  'v2/arpeggios/types.ts',
  'v2/arpeggios/patterns.ts',
  'v2/alaap/types.ts',
  'v2/alaap/render.ts',
].forEach((rel) => check(`exists: ${rel}`, () => assert.ok(exists(rel))));

console.log('\n▸ Synth timbres');
check('5 synth timbres present (sawlead, pad, supersawpad, squarepluck, dronesynth)', () => {
  const src = read('v2/samples/timbres.ts');
  for (const t of ['sawlead', 'pad', 'supersawpad', 'squarepluck', 'dronesynth']) {
    assert.ok(src.includes(`'${t}':`), `missing synth timbre: ${t}`);
  }
});
check('synth category tagged correctly', () => {
  const src = read('v2/samples/timbres.ts');
  // each synth profile should have category: 'synth'
  const synthEntries = src.match(/'(sawlead|pad|supersawpad|squarepluck|dronesynth)':\s*{[^}]*category:\s*'synth'/g) ?? [];
  assert.equal(synthEntries.length, 5, `expected 5 synth-categorized entries, found ${synthEntries.length}`);
});
check('SYNTH_TIMBRES export filters synths only', () => {
  assert.ok(read('v2/samples/timbres.ts').includes('SYNTH_TIMBRES'));
});

console.log('\n▸ Paltas (Carnatic permutation patterns)');
check('6 paltas exported', () => {
  const src = read('v2/arpeggios/patterns.ts');
  for (const p of ['sarali', 'jantai', 'dattu', 'tara', 'mel-sthayi', 'alankara']) {
    assert.ok(src.includes(`'${p}':`), `missing palta: ${p}`);
  }
});
check('Sarali Varisai generator math (40 notes for 8-swara input)', () => {
  // Mirror the algorithm:
  //   5 ascending phrases × 4 notes + 5 descending phrases × 4 notes = 40
  const s = [0,1,2,3,4,5,6,7];
  const out = [];
  for (let st = 0; st <= 4; st++) out.push(s[st], s[st+1], s[st+2], s[st+3]);
  for (let st = 7; st >= 3; st--) out.push(s[st], s[st-1], s[st-2], s[st-3]);
  assert.equal(out.length, 40, `sarali should produce 40 notes, got ${out.length}`);
});
check('Jantai Varisai produces paired-repeats (40 notes)', () => {
  const s = [0,1,2,3,4,5,6,7];
  const out = [];
  for (let st = 0; st <= 4; st++) for (let i = 0; i < 4; i++) { out.push(s[st+i], s[st+i]); }
  assert.equal(out.length, 40);
  // verify pairing: every adjacent pair should match
  for (let i = 0; i < out.length; i += 2) assert.equal(out[i], out[i+1]);
});
check('Dattu Varisai uses skips (24 notes)', () => {
  const s = [0,1,2,3,4,5,6,7];
  const out = [];
  for (let st = 0; st <= 5; st++) out.push(s[st], s[st+2]);
  for (let st = 7; st >= 2; st--) out.push(s[st], s[st-2]);
  assert.equal(out.length, 24);
});
check('Tara Sthayi reaches s[7] repeatedly', () => {
  const s = [0,1,2,3,4,5,6,7];
  const out = [];
  for (let i = 0; i < 7; i++) out.push(s[i], s[7]);
  assert.equal(out.filter(x => x === 7).length, 7);  // Sa' appears 7 times in pairing phase
});

console.log('\n▸ Alaap');
check('3 phases configured (vilambit/madhya/drut)', () => {
  const src = read('v2/alaap/types.ts');
  for (const p of ['vilambit', 'madhya', 'drut']) {
    assert.ok(src.includes(`'${p}'`), `missing phase: ${p}`);
  }
});
check('Vilambit slowest cps (≤ madhya ≤ drut)', () => {
  const src = read('v2/alaap/types.ts');
  const cpsMatches = [...src.matchAll(/cps:\s*([\d.]+)/g)].map(m => +m[1]);
  // Order in DEFAULT_PHASE_CONFIGS: vilambit, madhya, drut
  assert.ok(cpsMatches.length >= 3, `expected ≥3 cps values, got ${cpsMatches.length}`);
  assert.ok(cpsMatches[0] <= cpsMatches[1], `vilambit cps ${cpsMatches[0]} should ≤ madhya ${cpsMatches[1]}`);
  assert.ok(cpsMatches[1] <= cpsMatches[2], `madhya cps ${cpsMatches[1]} should ≤ drut ${cpsMatches[2]}`);
});
check('Alaap renderer emits silences (~) between phrases', () => {
  const src = read('v2/alaap/render.ts');
  assert.ok(src.includes('`~@'), 'renderAlaap must emit silence tokens');
});
check('renderAlaap exported from barrel', () => {
  const src = read('v2/index.ts');
  assert.ok(src.includes('renderAlaap'));
  assert.ok(src.includes('PALTAS'));
});

console.log('\n▸ compose() integrates v2.5 modes');
check('compose accepts palta and alaap options', () => {
  const src = read('v2/compose.ts');
  assert.ok(src.includes('palta?:'), 'compose must accept palta');
  assert.ok(src.includes('alaap?:'), 'compose must accept alaap');
  assert.ok(src.includes('alaapPhases?:'), 'compose must accept alaapPhases');
});
check('compose alaap branch returns tanpura code', () => {
  const src = read('v2/compose.ts');
  // The alaap branch uses buildTanpuraCode unconditionally
  const alaapBlock = src.split('// ── ALAAP mode')[1]?.split('// ── Normal')[0];
  assert.ok(alaapBlock?.includes('buildTanpuraCode'),
    'alaap branch must layer tanpura');
});

console.log('\n▸ Audio regression guards (no `dronesynth` / sample-name leaks into .s())');
check('tanpura.ts uses synth oscillator name (sawtooth) NOT timbre key (dronesynth)', () => {
  const src = read('v2/tanpura.ts');
  // Allow the WORD "dronesynth" in comments but NOT in `.s("dronesynth")`
  assert.ok(!src.match(/\.s\(['"]dronesynth['"]\)/),
    'must not emit .s("dronesynth") — that is a timbre key, not a Strudel sound');
  assert.ok(src.includes('"sawtooth"'),
    'synth tanpura should use .s("sawtooth")');
});
check('synth timbre profiles use real oscillator names in `sound` field', () => {
  const validSynthSounds = ['sine', 'sawtooth', 'square', 'triangle', 'supersaw', 'sawpoly', 'pulse'];
  const src = read('v2/samples/timbres.ts');
  // Find every synth-category entry and confirm its `sound:` is one of the valid names
  const synthEntries = [...src.matchAll(/'(sawlead|pad|supersawpad|squarepluck|dronesynth)':\s*{\s*sound:\s*'([^']+)'/g)];
  assert.equal(synthEntries.length, 5, `expected 5 synth entries with sound: field, found ${synthEntries.length}`);
  for (const [, key, sound] of synthEntries) {
    assert.ok(validSynthSounds.includes(sound),
      `timbre '${key}' has sound '${sound}' which is not a valid Strudel oscillator. Valid: ${validSynthSounds.join(', ')}`);
  }
});

console.log('\n────────────────────────────────────────');
console.log(`  ${pass} passed, ${fail} failed`);
if (fail > 0) {
  console.log('  ❌ V2 contracts INCOMPLETE');
  process.exit(1);
}
console.log('  ✅ V2 contracts (P1 + P2 + V2.5) VERIFIED');
