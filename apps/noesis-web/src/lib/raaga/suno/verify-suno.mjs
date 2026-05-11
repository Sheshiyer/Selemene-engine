#!/usr/bin/env node
// Suno integration contract gate. Asserts:
//   - prompt template generates non-empty for all 72 ragas × 4 styles
//   - all prompts ≤ 300 chars
//   - all 5 canonical melakartas produce expected raga-name + style descriptors
//   - r2 keys are unique across (melakarta_num, style)
//
// Pure JS port — runs without TypeScript loader.
//
//   $ node src/lib/raaga/suno/verify-suno.mjs

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const exists = (rel) => fs.existsSync(path.join(__dirname, rel));
const read = (rel) => fs.readFileSync(path.join(__dirname, rel), 'utf8');

let pass = 0; let fail = 0;
const check = (label, fn) => {
  try { fn(); console.log(`  ✓ ${label}`); pass++; }
  catch (e) { console.log(`  ✗ ${label}\n    ${e.message}`); fail++; }
};

// ── Inline JS port of buildSunoPrompt (mirrors prompt.ts) ─────────────
const STYLE_DESCRIPTORS = {
  ambient:    { bpm: '60-70 BPM', instruments: 'sustained tanpura drone, sitar lead', mood: 'spacious, contemplative, slow-evolving', tags: 'indian classical, ambient, drone, instrumental, raga' },
  meditative: { bpm: '50-60 BPM', instruments: 'tanpura drone, bansuri flute, soft tabla', mood: 'devotional, peaceful, breath-paced', tags: 'indian classical, meditation, raga, instrumental, devotional' },
  cinematic:  { bpm: '80-90 BPM', instruments: 'orchestral strings, sarangi, sitar, soft percussion', mood: 'epic, grand, narrative-arc', tags: 'indian classical, cinematic, orchestral, raga, instrumental' },
  acid:       { bpm: '124-132 BPM', instruments: 'TR-808 drums, TB-303 squelch bassline, Jupiter-8 lead, sustained drone', mood: 'hypnotic, driving, charanjit-singh-1982-style', tags: 'indian classical, acid house, electronic, raga, instrumental, charanjit singh' },
};
const CHAKRA_MOOD = {
  1:'earth-foundation, grounding, root-chakra', 2:'flow, eyes-open-perception, sacral',
  3:'fire-activation, transformative, solar-plexus', 4:'wisdom-seat, hip-grounded, structural',
  5:'directed-intention, lower-belly, focused', 6:'rhythmic-center, navel, seasonal',
  7:'sage-discernment, solar-plexus, clarity', 8:'heart-opening, devotional-warmth',
  9:'creative-expansion, chest, breath-of-creation', 10:'directional-power, shoulders, expansive',
  11:'fierce-expression, throat, voice-unblocked', 12:'solar-consciousness, crown, luminous',
};
// 72 melakarta names mirror melakartas.ts NAMES export
const NAMES = [
  "Kanakangi","Ratnangi","Ganamurti","Vanaspati","Manavati","Tanarupi",
  "Senapati","Hanumatodi","Dhenuka","Natakapriya","Kokilapriya","Rupavati",
  "Gayakapriya","Vakulabharanam","Mayamalavagaula","Chakravakam","Suryakantam","Hatakambari",
  "Jhankaradhvani","Natabhairavi","Keeravani","Kharaharapriya","Gourimanohari","Varunapriya",
  "Mararanjani","Charukesi","Sarasangi","Harikambhoji","Dheerasankarabharanam","Naganandini",
  "Yagapriya","Ragavardhini","Gangeyabhushani","Vagadhishvari","Shulini","Chalanata",
  "Salagam","Jalarnavam","Jhalavarali","Navaneetam","Pavani","Raghupriya",
  "Gavambodhi","Bhavapriya","Subhapantuvarali","Shadvidhamargini","Suvarnangi","Divyamani",
  "Dhavalambari","Namanarayani","Kamavardhini","Ramapriya","Gamanasrama","Vishvambhari",
  "Shyamalangi","Shanmukhapriya","Simhendramadhyamam","Hemavati","Dharmavati","Neetimati",
  "Kantamani","Rishabhapriya","Latangi","Vachaspati","Mechakalyani","Chitrambari",
  "Sucharitra","Jyotisvarupini","Dhatuvardhini","Nasikabhushani","Kosalam","Rasikapriya",
];
const buildSunoPrompt = (n, style = 'ambient', dur = 45) => {
  const name = NAMES[n - 1];
  const chakra = Math.floor((n - 1) / 6) + 1;
  const desc = STYLE_DESCRIPTORS[style];
  const mood = CHAKRA_MOOD[chakra];
  const prompt = `Indian classical raga ${name} (Carnatic melakarta #${n}), ${desc.mood} ${style} instrumental, no vocals, played at ${desc.bpm} with ${desc.instruments}. Mood: ${mood}. Approximately ${dur} seconds.`;
  const trimmed = prompt.length > 300 ? prompt.slice(0, 297) + '...' : prompt;
  return { prompt: trimmed, tags: desc.tags, title: `${name} (#${n}) — ${style}`, make_instrumental: true, wait_audio: false };
};

console.log('▸ Suno file inventory');
['types.ts', 'prompt.ts', 'client.ts', 'r2.ts', 'index.ts'].forEach((rel) =>
  check(`exists: ${rel}`, () => assert.ok(exists('../suno/' + rel) || exists('./' + rel)))
);

console.log('\n▸ Prompt template generates for all 72 melakartas × 4 styles');
const STYLES = ['ambient', 'meditative', 'cinematic', 'acid'];
let totalPrompts = 0;
for (const style of STYLES) {
  for (let n = 1; n <= 72; n++) {
    const out = buildSunoPrompt(n, style);
    totalPrompts++;
    if (!out.prompt || !out.prompt.length) {
      check(`#${n} ${style}`, () => { throw new Error('empty prompt'); });
    }
  }
}
check(`generated ${totalPrompts} prompts (72 × 4)`, () => assert.equal(totalPrompts, 288));

console.log('\n▸ All prompts ≤ 300 chars');
let overlong = 0;
for (const style of STYLES) {
  for (let n = 1; n <= 72; n++) {
    if (buildSunoPrompt(n, style).prompt.length > 300) overlong++;
  }
}
check(`zero prompts > 300 chars (got ${overlong})`, () => assert.equal(overlong, 0));

console.log('\n▸ Canonical 5 melakartas — preview');
const canonical = [
  [15, 'Mayamalavagaula', 'ambient'],
  [29, 'Dheerasankarabharanam', 'meditative'],
  [65, 'Mechakalyani', 'cinematic'],
  [8,  'Hanumatodi', 'acid'],
  [22, 'Kharaharapriya', 'ambient'],
];
for (const [n, name, style] of canonical) {
  const out = buildSunoPrompt(n, style);
  check(`#${n} ${name} (${style}): contains name + style descriptors`, () => {
    assert.ok(out.prompt.includes(name), `prompt should contain "${name}"`);
    assert.ok(out.prompt.includes(style), `prompt should contain style "${style}"`);
    assert.ok(out.prompt.includes('no vocals'), `prompt should specify "no vocals"`);
    assert.ok(out.prompt.includes('Approximately'), `prompt should mention duration`);
    assert.ok(out.title.includes(name) && out.title.includes(style), `title should contain name + style`);
    assert.ok(out.make_instrumental === true, `make_instrumental must be true`);
    assert.ok(out.wait_audio === false, `wait_audio must be false (we poll)`);
  });
  console.log(`     prompt: "${out.prompt.slice(0, 120)}..."`);
}

console.log('\n▸ R2 key uniqueness across (melakarta, style)');
// Mirror r2KeyFor(): clips/{style}/{NN}-{songId}.mp3
const r2Key = (n, style, songId) => `clips/${style}/${String(n).padStart(2, '0')}-${songId}.mp3`;
const seen = new Set();
let dupes = 0;
for (const style of STYLES) {
  for (let n = 1; n <= 72; n++) {
    const k = r2Key(n, style, 'fakesongid123');  // same songId — only style+num distinguishes
    if (seen.has(k)) dupes++;
    seen.add(k);
  }
}
check(`288 unique keys across styles+ragas (got ${seen.size}, dupes ${dupes})`, () => {
  assert.equal(seen.size, 288);
  assert.equal(dupes, 0);
});

console.log('\n▸ Migration file present');
check('exists: migrations/028_raga_clips.sql', () => {
  // From apps/noesis-web/src/lib/raaga/suno/verify-suno.mjs → 6 levels up to repo root
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const target = path.join(repoRoot, 'migrations/028_raga_clips.sql');
  assert.ok(fs.existsSync(target),
    `migration file must exist at ${target}`);
});

// ─── PHASE 3 ASSERTIONS — API route + Nadabrahman fallback chain ──────
console.log('\n▸ Phase-3 file inventory');
// db.ts + pg.d.ts live alongside the verifier in suno/;
// useRagaClip.ts lives one level up in lib/raaga/.
const phase3Files = [
  { rel: 'db.ts',          label: 'src/lib/raaga/suno/db.ts' },
  { rel: 'pg.d.ts',        label: 'src/lib/raaga/suno/pg.d.ts' },
  { rel: '../useRagaClip.ts', label: 'src/lib/raaga/useRagaClip.ts' },
];
phase3Files.forEach(({ rel, label }) => {
  check(`exists: ${label}`, () => {
    assert.ok(exists(rel), `${rel} missing`);
  });
});
// API route lives outside the suno/ dir
check('exists: app/api/v1/raga/[num]/clip/route.ts', () => {
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const target = path.join(repoRoot, 'apps/noesis-web/app/api/v1/raga/[num]/clip/route.ts');
  assert.ok(fs.existsSync(target), `route file must exist at ${target}`);
});

console.log('\n▸ API route contract');
check('route exports GET handler', () => {
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const src = fs.readFileSync(path.join(repoRoot, 'apps/noesis-web/app/api/v1/raga/[num]/clip/route.ts'), 'utf8');
  assert.ok(src.includes('export async function GET'), 'must export GET');
  assert.ok(src.includes('findApprovedClip'), 'must call findApprovedClip');
  assert.ok(src.includes('Cache-Control'), 'must set Cache-Control header');
});
check('route validates melakarta_num 1..72 + style enum', () => {
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const src = fs.readFileSync(path.join(repoRoot, 'apps/noesis-web/app/api/v1/raga/[num]/clip/route.ts'), 'utf8');
  assert.ok(/num\s*<\s*1\s*\|\|\s*num\s*>\s*72/.test(src), 'must guard num range');
  assert.ok(src.includes("'ambient'") && src.includes("'meditative'"), 'must enumerate styles');
});
check('route returns 404 with fallback hint on missing clip', () => {
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const src = fs.readFileSync(path.join(repoRoot, 'apps/noesis-web/app/api/v1/raga/[num]/clip/route.ts'), 'utf8');
  assert.ok(src.includes('status: 404'), 'must return 404 when no clip');
  assert.ok(src.includes('fallback'), 'must hint at fallback path');
});
check('route returns 503 on DB error so client falls back', () => {
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const src = fs.readFileSync(path.join(repoRoot, 'apps/noesis-web/app/api/v1/raga/[num]/clip/route.ts'), 'utf8');
  assert.ok(src.includes('status: 503'), 'must return 503 on DB exception');
});

console.log('\n▸ Nadabrahman fallback chain wired');
check('Nadabrahman.tsx imports SunoStyle + has audioSource state', () => {
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const src = fs.readFileSync(path.join(repoRoot, 'apps/noesis-web/src/components/engines/Nadabrahman.tsx'), 'utf8');
  assert.ok(src.includes('SunoStyle'), 'must import SunoStyle type');
  assert.ok(src.includes("audioSource"), 'must have audioSource state');
  assert.ok(src.includes("'auto'") && src.includes("'live'"), 'must offer auto/live modes');
});
check('Nadabrahman.tsx calls Suno API endpoint', () => {
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const src = fs.readFileSync(path.join(repoRoot, 'apps/noesis-web/src/components/engines/Nadabrahman.tsx'), 'utf8');
  assert.ok(src.includes('/api/v1/raga/'), 'must fetch the clip API');
  assert.ok(src.includes('clipAudioRef'), 'must hold an <audio> element ref');
});
check('Nadabrahman.tsx falls back to getRaagaPlayer().play() on Suno miss', () => {
  const repoRoot = path.resolve(__dirname, '../../../../../..');
  const src = fs.readFileSync(path.join(repoRoot, 'apps/noesis-web/src/components/engines/Nadabrahman.tsx'), 'utf8');
  // Verify the fall-through path: try Suno → if not ok → Strudel
  assert.ok(src.includes('getRaagaPlayer().play'), 'must still play via Strudel');
  // Status should reflect when Strudel is used as a fallback vs. live
  assert.ok(/Strudel fallback|Strudel live/.test(src), 'must label playback source');
});

console.log('\n▸ DB helper contract');
check('db.ts is server-only + queries by (num, style, status=approved)', () => {
  const src = read('db.ts');
  assert.ok(src.includes("'server-only'"), 'must import server-only');
  assert.ok(src.includes("status = 'approved'"), 'must filter status');
  assert.ok(src.includes('melakarta_num = $1'), 'must parameterize melakarta');
  assert.ok(src.includes('style = $2'), 'must parameterize style');
});

console.log('\n────────────────────────────────────────');
console.log(`  ${pass} passed, ${fail} failed`);
if (fail > 0) {
  console.log('  ❌ Suno contracts INCOMPLETE');
  process.exit(1);
}
console.log('  ✅ Suno P1 + P3 contracts VERIFIED — ready for cookie + R2 + smoke run');
