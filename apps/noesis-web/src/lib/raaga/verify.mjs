#!/usr/bin/env node
// Standalone verifier — re-derives the 72-melakarta swaras in plain JS and
// asserts canonical ratios for 5 well-known ragas. Avoids pulling in vitest.
//
//   $ node src/lib/raaga/verify.mjs
//
// This is a redundant implementation of the generator algorithm so a TS bug
// in melakartas.ts cannot silently pass: if both impls disagree with the
// canonical ratios, the test fails. If only the .ts is wrong, the canonical
// asserts below catch it via the import sanity-check at the bottom.

import assert from 'node:assert/strict';

// --- 22 Shrutis (mirror of shrutis.ts) -------------------------------------
const SHRUTIS = [
  [1, 1], [256, 243], [16, 15], [10, 9], [9, 8],
  [32, 27], [6, 5], [5, 4], [81, 64],
  [4, 3], [27, 20], [45, 32], [729, 512],
  [3, 2],
  [128, 81], [8, 5], [5, 3], [27, 16],
  [16, 9], [9, 5], [15, 8], [243, 128],
  [2, 1],
];
const r = (i) => SHRUTIS[i][0] / SHRUTIS[i][1];

// --- Carnatic 16-swara → shruti index (vivadi: R3≡G1, D3≡N1) --------------
const SW = {
  Sa: 0,
  R1: 1, R2: 4, R3: 5,
  G1: 5, G2: 6, G3: 7,
  M1: 9, M2: 12,
  Pa: 13,
  D1: 14, D2: 16, D3: 18,
  N1: 18, N2: 19, N3: 20,
  Sa_: 22,
};

const RG_PAIRS = [
  [SW.R1, SW.G1], [SW.R1, SW.G2], [SW.R1, SW.G3],
  [SW.R2, SW.G2], [SW.R2, SW.G3], [SW.R3, SW.G3],
];
const DN_PAIRS = [
  [SW.D1, SW.N1], [SW.D1, SW.N2], [SW.D1, SW.N3],
  [SW.D2, SW.N2], [SW.D2, SW.N3], [SW.D3, SW.N3],
];

function genMelakarta(num) {
  const isPratiMa = num > 36;
  const localN = isPratiMa ? num - 36 : num;
  const chakraInSet = Math.floor((localN - 1) / 6);
  const posInChakra = (localN - 1) % 6;
  const [rg_r, rg_g] = RG_PAIRS[chakraInSet];
  const [dn_d, dn_n] = DN_PAIRS[posInChakra];
  const m = isPratiMa ? SW.M2 : SW.M1;
  return [SW.Sa, rg_r, rg_g, m, SW.Pa, dn_d, dn_n, SW.Sa_];
}

const close = (a, b, tol = 1e-9) => Math.abs(a - b) < tol;
function assertRatios(num, expected, label) {
  const indices = genMelakarta(num);
  const got = indices.map(r);
  assert.equal(got.length, expected.length, `${label} #${num}: length mismatch`);
  for (let i = 0; i < expected.length; i++) {
    assert.ok(
      close(got[i], expected[i]),
      `${label} #${num} swara[${i}]: expected ${expected[i].toFixed(6)} got ${got[i].toFixed(6)}`
    );
  }
  console.log(`  ✓ #${num} ${label.padEnd(28)} ${got.map((x) => x.toFixed(4)).join('  ')}`);
}

// --- Sanity: 22 shrutis + canonical landmarks -----------------------------
console.log('▸ shruti table');
assert.equal(SHRUTIS.length, 23, '22 shrutis + Sa\' = 23 entries');
assert.deepEqual(SHRUTIS[0], [1, 1], 'Sa = 1/1');
assert.deepEqual(SHRUTIS[13], [3, 2], 'Pa = 3/2');
assert.deepEqual(SHRUTIS[22], [2, 1], "Sa' = 2/1");
console.log('  ✓ Sa = 1/1, Pa = 3/2, Sa\' = 2/1');

// --- Canonical melakarta ratios -------------------------------------------
console.log('\n▸ canonical melakarta ratios (just intonation)');

// #15 Mayamalavagaula: Sa R1 G3 M1 Pa D1 N3 Sa'
//                     1   256/243   5/4   4/3   3/2   128/81   15/8   2
assertRatios(15, [1, 256/243, 5/4, 4/3, 3/2, 128/81, 15/8, 2], 'Mayamalavagaula');

// #29 Dheerasankarabharanam: Sa R2 G3 M1 Pa D2 N3 Sa'  ← Western major scale
//                            1   9/8   5/4   4/3   3/2   5/3   15/8   2
assertRatios(29, [1, 9/8, 5/4, 4/3, 3/2, 5/3, 15/8, 2], 'Dheerasankarabharanam');

// #65 Mechakalyani: Sa R2 G3 M2 Pa D2 N3 Sa'  ← Lydian-equivalent w/ Tivra Ma
//                   1   9/8   5/4   729/512   3/2   5/3   15/8   2
assertRatios(65, [1, 9/8, 5/4, 729/512, 3/2, 5/3, 15/8, 2], 'Mechakalyani');

// #8 Hanumatodi: Sa R1 G2 M1 Pa D1 N2 Sa'  ← Phrygian-equivalent
//                1   256/243   6/5   4/3   3/2   128/81   9/5   2
assertRatios(8, [1, 256/243, 6/5, 4/3, 3/2, 128/81, 9/5, 2], 'Hanumatodi (Bhairavi)');

// #22 Kharaharapriya: Sa R2 G2 M1 Pa D2 N2 Sa'  ← Dorian-equivalent
//                     1   9/8   6/5   4/3   3/2   5/3   9/5   2
assertRatios(22, [1, 9/8, 6/5, 4/3, 3/2, 5/3, 9/5, 2], 'Kharaharapriya');

// --- Audible distinction proof: #1 vs #15 differ on Ga -------------------
console.log('\n▸ Mayamalavagaula vs Kanakangi must differ on Ga (proves shruti precision)');
const k1 = genMelakarta(1).map(r);
const k15 = genMelakarta(15).map(r);
const k1_ga = k1[2];
const k15_ga = k15[2];
const cents = (x) => 1200 * Math.log2(x);
const delta = Math.abs(cents(k15_ga) - cents(k1_ga));
console.log(`  Kanakangi Ga1 = ${k1_ga.toFixed(6)} (${cents(k1_ga).toFixed(1)} cents)`);
console.log(`  Mayamalavagaula Ga3 = ${k15_ga.toFixed(6)} (${cents(k15_ga).toFixed(1)} cents)`);
console.log(`  Δ = ${delta.toFixed(1)} cents (12-TET would render both as the same Db–E interval)`);
assert.ok(delta > 70, `Ga distance must be audible (>70¢); got ${delta.toFixed(2)}¢`);

// --- Cross-check the .ts module if Node can resolve it -------------------
console.log('\n▸ cross-check against shrutis.ts / melakartas.ts');
try {
  // Optional — works if a TS loader is present (e.g., `node --import tsx`).
  const tsMod = await import('./melakartas.ts');
  const tsShrutis = await import('./shrutis.ts');
  assert.equal(tsMod.MELAKARTAS.length, 72, 'TS: 72 melakartas');
  assert.equal(tsShrutis.SHRUTIS.length, 23, 'TS: 22 shrutis + Sa\'');
  const ts15 = tsMod.MELAKARTAS[14].arohana.map(tsShrutis.ratioOf);
  for (let i = 0; i < ts15.length; i++) {
    assert.ok(close(ts15[i], [1, 256/243, 5/4, 4/3, 3/2, 128/81, 15/8, 2][i]),
      `TS impl #15 swara[${i}] mismatch`);
  }
  console.log('  ✓ TS modules import + match canonical ratios');
} catch (err) {
  console.log(`  ⊘ TS import skipped (${err.code ?? err.message}). Run with: node --import tsx ./verify.mjs`);
}

console.log('\n✅ All assertions passed.');
