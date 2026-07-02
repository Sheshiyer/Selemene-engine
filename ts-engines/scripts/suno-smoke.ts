#!/usr/bin/env bun
// Smoke test for the Suno integration — rewritten for ts-engines (SUNO-04).
// Generates ONE raga end-to-end:
//   1. Build prompt for given melakarta_num + style
//   2. POST to Suno bridge → get song id
//   3. Poll until ready (~30-90s)
//   4. Download MP3
//   5. Upload to Cloudflare R2
//   6. POST /internal/raga/clip to persist the row
//
// Usage:
//   $ bun ts-engines/scripts/suno-smoke.ts <melakarta_num> [style] [duration_sec]
//   $ bun ts-engines/scripts/suno-smoke.ts 15
//   $ bun ts-engines/scripts/suno-smoke.ts 15 ambient 45
//
// Required env:
//   SUNO_BRIDGE_URL            — e.g. https://suno-bridge.tryambakam.space
//   SUNO_COOKIE                — Suno session cookie
//   R2_ACCOUNT_ID
//   R2_ACCESS_KEY_ID
//   R2_SECRET_ACCESS_KEY
//   R2_PUBLIC_BASE_URL
//   NOESIS_API_URL             — e.g. https://selemene.tryambakam.space
//   INTERNAL_SERVICE_KEY       — shared secret for /internal/raga/clip
//
// Optional:
//   R2_RAGA_CLIPS_BUCKET     — optional, default selemene-raga-clips
//   SUNO_SMOKE_DRY_RUN=1       — skip actual Suno call, print prompt + exit

import { uploadToR2 } from './r2-upload';

type SunoStyle = 'ambient' | 'traditional';

const args = process.argv.slice(2);
const melakartaNum = Number.parseInt(args[0] ?? '15', 10);
const style = (args[1] ?? 'ambient') as SunoStyle;
const durationSec = Number.parseInt(args[2] ?? '45', 10);

const SUNO_BRIDGE_URL = process.env.SUNO_BRIDGE_URL ?? 'https://suno-bridge.tryambakam.space';
const SUNO_COOKIE = process.env.SUNO_COOKIE ?? '';
const NOESIS_API = process.env.NOESIS_API_URL ?? 'https://selemene.tryambakam.space';
const INTERNAL_KEY = process.env.INTERNAL_SERVICE_KEY ?? '';

const log = (msg: string) => console.log(`[smoke ${new Date().toISOString()}] ${msg}`);

function sunoHeaders(): HeadersInit {
  return { 'Content-Type': 'application/json', Accept: 'application/json', Cookie: SUNO_COOKIE };
}

async function getQuota() {
  const r = await fetch(`${SUNO_BRIDGE_URL}/api/get_limit`, { headers: sunoHeaders() });
  if (!r.ok) throw new Error(`get_limit: ${r.status}`);
  return r.json() as Promise<{ credits_left: number; monthly_usage: number; monthly_limit: number }>;
}

async function submit(prompt: { prompt: string; tags: string; title: string; make_instrumental: boolean; duration: number }) {
  const r = await fetch(`${SUNO_BRIDGE_URL}/api/custom_generate`, {
    method: 'POST', headers: sunoHeaders(), body: JSON.stringify(prompt),
  });
  if (!r.ok) throw new Error(`custom_generate: ${r.status} ${await r.text()}`);
  return r.json() as Promise<Array<{ id: string; status: string; audio_url: string; duration: number }>>;
}

async function pollUntilReady(id: string, intervalMs = 5000, timeoutMs = 180_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const r = await fetch(`${SUNO_BRIDGE_URL}/api/get?ids=${id}`, { headers: sunoHeaders() });
    if (!r.ok) throw new Error(`poll: ${r.status}`);
    const songs = await r.json() as Array<{ id: string; status: string; audio_url: string; duration: number }>;
    const song = songs.find(s => s.id === id);
    if (song?.status === 'streaming' || song?.status === 'complete') return song;
    if (song?.status === 'error') throw new Error(`Song ${id} errored`);
    await new Promise(res => setTimeout(res, intervalMs));
  }
  throw new Error(`Poll timeout for ${id}`);
}

async function upsertClip(melakartaNum: number, style: string, songId: string, sunoPrompt: string, r2Key: string, cdnUrl: string, duration: number) {
  const r = await fetch(`${NOESIS_API}/internal/raga/clip`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'x-internal-key': INTERNAL_KEY },
    body: JSON.stringify({
      melakarta_num: melakartaNum,
      style,
      suno_song_id: songId,
      suno_prompt: sunoPrompt,
      r2_key: r2Key,
      cdn_url: cdnUrl,
      duration_sec: Math.round(duration),
      status: 'generated'
    }),
  });
  if (!r.ok) throw new Error(`upsert clip: ${r.status} ${await r.text()}`);
}

function buildPrompt(num: number, style: SunoStyle, duration: number) {
  // Minimal prompt builder — real version is in the Raaga engine
  const names = ['Kanakangi','Ratnangi','Ganamurti','Vanaspati','Manavati','Tanarupi','Senavati','Hanumatodi',
    'Dhenuka','Natakapriya','Kokilapriya','Rupavati','Gayakapriya','Vakulabharanam','Mayamalavagowla','Chakravakam',
    'Suryakantam','Hatakambari','Jhankaradhvani','Natabhairavi','Keeravani','Kharaharapriya','Gaurimanohari',
    'Varunapriya','Mararanjani','Charukesi','Sarasangi','Harikambhoji','Dheerasankarabharanam','Naganandini',
    'Yagapriya','Ragavardhini','Gangeyabhushani','Vagadheeswari','Shulini','Chalanata','Salagam','Jalarnavam',
    'Jhalavarali','Navaneetam','Pavani','Raghupriya','Gavambodhi','Bhavapriya','Shubhapantuvarali','Shadvidamargini',
    'Suvarnangi','Divyamani','Dhavalambari','Namanarayani','Kamavardhini','Ramapriya','Gamanashrama','Vishvambhari',
    'Shamalangi','Shanmukhapriya','Simhendramadhyamam','Hemavati','Dharmavati','Neetimati','Kantamani',
    'Rishabhapriya','Latangi','Vachaspati','Mechakalyani','Chitrambari','Sucharitra','Jyotisvarupini','Dhatuvardhini',
    'Nasikabhushani','Kosalam','Rasikapriya'];
  const name = names[num - 1] ?? `Melakarta ${num}`;
  const tags = style === 'ambient' ? 'ambient, meditation, Indian classical, drone, raga, instrumental' : 'Indian classical, Carnatic, traditional, raga, sitar, veena';
  return { prompt: `A ${style} rendition of the ${name} raga — Carnatic melakarta #${num}. ${style === 'ambient' ? 'Meditative, atmospheric, with tanpura drone' : 'Traditional Carnatic style with proper gamaka ornaments'}.`, tags, title: `${name} (${style})`, make_instrumental: true, duration };
}

async function main() {
  log(`Smoke test: melakarta=${melakartaNum} style=${style} duration=${durationSec}s`);
  const prompt = buildPrompt(melakartaNum, style, durationSec);
  log(`Prompt: "${prompt.prompt}"`);

  if (process.env.SUNO_SMOKE_DRY_RUN === '1') {
    log('DRY RUN — exiting.'); return;
  }

  if (!SUNO_COOKIE) throw new Error('SUNO_COOKIE env var required');
  if (!INTERNAL_KEY) throw new Error('INTERNAL_SERVICE_KEY env var required');

  const quotaBefore = await getQuota();
  log(`Quota: ${quotaBefore.credits_left} credits`);
  if (quotaBefore.credits_left < 20) throw new Error(`Quota too low (${quotaBefore.credits_left})`);

  const songs = await submit(prompt);
  const song = songs[0];
  log(`Submitted: ${song.id}`);

  const ready = await pollUntilReady(song.id);
  log(`Ready: ${ready.audio_url}`);

  const r = await fetch(ready.audio_url);
  if (!r.ok) throw new Error(`MP3 download: ${r.status}`);
  const buffer = Buffer.from(await r.arrayBuffer());
  log(`Downloaded ${(buffer.length / 1024).toFixed(1)} KiB`);

  const key = `clips/${style}/${String(melakartaNum).padStart(2, '0')}-${song.id}.mp3`;
  const cdnUrl = await uploadToR2(key, buffer);
  log(`Uploaded → ${cdnUrl}`);

  await upsertClip(melakartaNum, style, song.id, prompt.prompt, key, cdnUrl, ready.duration);
  log(`DB row upserted → status=generated`);

  const quotaAfter = await getQuota();
  log(`Quota after: ${quotaAfter.credits_left} (used ${quotaBefore.credits_left - quotaAfter.credits_left})`);
  log('✅ Smoke test complete.');
}

main().catch(err => { console.error('[smoke] FAILED:', err); process.exit(1); });
