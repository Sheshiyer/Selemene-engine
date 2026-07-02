#!/usr/bin/env bun
// Bulk generator for the 72-melakarta library — rewritten for ts-engines (SUNO-04).
//
// Walks 1..72, submits to Suno, polls, downloads, uploads to Cloudflare R2,
// then POSTs /internal/raga/clip to persist the row.
// Resumable via .suno-checkpoint.json — re-running skips completed ragas.
//
// Usage:
//   $ bun ts-engines/scripts/suno-bulk-gen.ts [style] [start_num] [end_num]
//   $ bun ts-engines/scripts/suno-bulk-gen.ts ambient        # full 1..72
//   $ bun ts-engines/scripts/suno-bulk-gen.ts ambient 1 36   # first half only
//
// Required env (copy .env.template to .env and fill):
//   SUNO_BRIDGE_URL
//   SUNO_COOKIE
//   R2_ACCOUNT_ID
//   R2_ACCESS_KEY_ID
//   R2_SECRET_ACCESS_KEY
//   R2_PUBLIC_BASE_URL
//   NOESIS_API_URL
//   INTERNAL_SERVICE_KEY
//
// Optional tuning:
//   SUNO_BATCH_SIZE          (default 5)
//   SUNO_POLL_INTERVAL_MS    (default 5000)
//   SUNO_TIMEOUT_MS          (default 180000)
//   MAX_CREDITS_PER_RUN      (default 200)
//   MIN_CREDITS_TO_START     (default 100)
//   R2_RAGA_CLIPS_BUCKET     (default selemene-raga-clips)

import { uploadToR2 } from './r2-upload';
import fs from 'node:fs';
import path from 'node:path';

type SunoStyle = 'ambient' | 'traditional';
interface Checkpoint {
  lastSuccessAt: string;
  completed: Record<number, string[]>;
  creditsUsed: number;
  initialCredits: number;
}

const CHECKPOINT_PATH = path.resolve(process.cwd(), '.suno-checkpoint.json');
const args = process.argv.slice(2);
const style = (args[0] ?? 'ambient') as SunoStyle;
const startNum = Number.parseInt(args[1] ?? '1', 10);
const endNum = Number.parseInt(args[2] ?? '72', 10);

const SUNO_BRIDGE_URL = process.env.SUNO_BRIDGE_URL ?? '';
const SUNO_COOKIE = process.env.SUNO_COOKIE ?? '';
const NOESIS_API = process.env.NOESIS_API_URL ?? 'https://selemene.tryambakam.space';
const INTERNAL_KEY = process.env.INTERNAL_SERVICE_KEY ?? '';

const BATCH_SIZE = Number.parseInt(process.env.SUNO_BATCH_SIZE ?? '5', 10);
const POLL_INTERVAL_MS = Number.parseInt(process.env.SUNO_POLL_INTERVAL_MS ?? '5000', 10);
const TIMEOUT_MS = Number.parseInt(process.env.SUNO_TIMEOUT_MS ?? '180000', 10);
const MAX_CREDITS_PER_RUN = Number.parseInt(process.env.MAX_CREDITS_PER_RUN ?? '200', 10);
const MIN_CREDITS_TO_START = Number.parseInt(process.env.MIN_CREDITS_TO_START ?? '100', 10);

const log = (msg: string) => console.log(`[bulk ${new Date().toISOString()}] ${msg}`);

function sunoHeaders(): HeadersInit {
  return { 'Content-Type': 'application/json', Accept: 'application/json', Cookie: SUNO_COOKIE };
}

async function getQuota() {
  const r = await fetch(`${SUNO_BRIDGE_URL}/api/get_limit`, { headers: sunoHeaders() });
  if (!r.ok) throw new Error(`get_limit: ${r.status}`);
  return r.json() as Promise<{ credits_left: number; monthly_usage: number; monthly_limit: number }>;
}

async function submit(prompt: object) {
  const r = await fetch(`${SUNO_BRIDGE_URL}/api/custom_generate`, {
    method: 'POST', headers: sunoHeaders(), body: JSON.stringify(prompt),
  });
  if (!r.ok) throw new Error(`custom_generate: ${r.status} ${await r.text()}`);
  return r.json() as Promise<Array<{ id: string; status: string; audio_url: string; duration: number }>>;
}

async function pollUntilReady(id: string) {
  const deadline = Date.now() + TIMEOUT_MS;
  while (Date.now() < deadline) {
    const r = await fetch(`${SUNO_BRIDGE_URL}/api/get?ids=${id}`, { headers: sunoHeaders() });
    if (!r.ok) throw new Error(`poll: ${r.status}`);
    const songs = await r.json() as Array<{ id: string; status: string; audio_url: string; duration: number }>;
    const song = songs.find(s => s.id === id);
    if (song?.status === 'streaming' || song?.status === 'complete') return song;
    if (song?.status === 'error') throw new Error(`Song ${id} errored`);
    await new Promise(res => setTimeout(res, POLL_INTERVAL_MS));
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

// Minimal prompt builder (mirrors RaagaEngine logic)
const RAGA_NAMES = ['Kanakangi','Ratnangi','Ganamurti','Vanaspati','Manavati','Tanarupi','Senavati','Hanumatodi','Dhenuka','Natakapriya','Kokilapriya','Rupavati','Gayakapriya','Vakulabharanam','Mayamalavagowla','Chakravakam','Suryakantam','Hatakambari','Jhankaradhvani','Natabhairavi','Keeravani','Kharaharapriya','Gaurimanohari','Varunapriya','Mararanjani','Charukesi','Sarasangi','Harikambhoji','Dheerasankarabharanam','Naganandini','Yagapriya','Ragavardhini','Gangeyabhushani','Vagadheeswari','Shulini','Chalanata','Salagam','Jalarnavam','Jhalavarali','Navaneetam','Pavani','Raghupriya','Gavambodhi','Bhavapriya','Shubhapantuvarali','Shadvidamargini','Suvarnangi','Divyamani','Dhavalambari','Namanarayani','Kamavardhini','Ramapriya','Gamanashrama','Vishvambhari','Shamalangi','Shanmukhapriya','Simhendramadhyamam','Hemavati','Dharmavati','Neetimati','Kantamani','Rishabhapriya','Latangi','Vachaspati','Mechakalyani','Chitrambari','Sucharitra','Jyotisvarupini','Dhatuvardhini','Nasikabhushani','Kosalam','Rasikapriya'];

function buildPrompt(num: number, style: SunoStyle, duration = 45) {
  const name = RAGA_NAMES[num - 1] ?? `Melakarta ${num}`;
  const tags = style === 'ambient' ? 'ambient, meditation, Indian classical, drone, raga, instrumental' : 'Indian classical, Carnatic, traditional, raga, sitar, veena';
  return { prompt: `A ${style} rendition of the ${name} raga — Carnatic melakarta #${num}. ${style === 'ambient' ? 'Meditative, atmospheric, with tanpura drone' : 'Traditional Carnatic style with proper gamaka ornaments'}.`, tags, title: `${name} (${style})`, make_instrumental: true, duration };
}

function loadCheckpoint(): Checkpoint {
  if (fs.existsSync(CHECKPOINT_PATH)) {
    try { return JSON.parse(fs.readFileSync(CHECKPOINT_PATH, 'utf8')); }
    catch { /* fresh */ }
  }
  return { lastSuccessAt: '', completed: {}, creditsUsed: 0, initialCredits: 0 };
}

function saveCheckpoint(cp: Checkpoint) {
  fs.writeFileSync(CHECKPOINT_PATH, JSON.stringify(cp, null, 2));
}

async function generateOne(num: number, style: SunoStyle): Promise<{ songId: string; cdnUrl: string; durationSec: number }> {
  const prompt = buildPrompt(num, style);
  log(`  #${num} ${style} → submitting`);
  const songs = await submit(prompt);
  const song = songs[0];
  if (!song) throw new Error(`#${num}: no song returned`);
  const ready = await pollUntilReady(song.id);
  const r = await fetch(ready.audio_url);
  if (!r.ok) throw new Error(`MP3 download for ${song.id}: ${r.status}`);
  const buffer = Buffer.from(await r.arrayBuffer());
  const key = `clips/${style}/${String(num).padStart(2, '0')}-${song.id}.mp3`;
  const cdnUrl = await uploadToR2(key, buffer);
  await upsertClip(num, style, song.id, prompt.prompt, key, cdnUrl, ready.duration);
  log(`  #${num} ${style} ✓ ${(buffer.length / 1024).toFixed(0)} KiB → ${cdnUrl}`);
  return { songId: song.id, cdnUrl, durationSec: ready.duration };
}

async function main() {
  // Pre-flight checks
  const missing = [
    !SUNO_BRIDGE_URL && 'SUNO_BRIDGE_URL',
    !SUNO_COOKIE && 'SUNO_COOKIE',
    !INTERNAL_KEY && 'INTERNAL_SERVICE_KEY',
  ].filter(Boolean);
  if (missing.length) throw new Error(`Missing env vars: ${missing.join(', ')}`);

  log(`Bulk gen: style=${style} range=[${startNum}..${endNum}] batch=${BATCH_SIZE}`);

  const quota = await getQuota();
  log(`Quota: ${quota.credits_left} credits`);
  if (quota.credits_left < MIN_CREDITS_TO_START) {
    throw new Error(`Quota too low (${quota.credits_left} < ${MIN_CREDITS_TO_START})`);
  }

  const cp = loadCheckpoint();
  if (!cp.initialCredits) cp.initialCredits = quota.credits_left;

  const remaining = Array.from({ length: endNum - startNum + 1 }, (_, i) => startNum + i)
    .filter(n => !(cp.completed[n] ?? []).includes(style));
  log(`${remaining.length} ragas remaining`);

  for (let i = 0; i < remaining.length; i += BATCH_SIZE) {
    if (cp.creditsUsed >= MAX_CREDITS_PER_RUN) {
      log(`⏸  Hit max-credits-per-run cap (${MAX_CREDITS_PER_RUN}). Re-run to continue.`);
      break;
    }
    const batch = remaining.slice(i, i + BATCH_SIZE);
    log(`Batch ${Math.floor(i / BATCH_SIZE) + 1}: [${batch.join(', ')}]`);
    const results = await Promise.allSettled(batch.map(n => generateOne(n, style)));
    for (let j = 0; j < batch.length; j++) {
      const n = batch[j];
      if (results[j].status === 'fulfilled') {
        cp.completed[n] = [...(cp.completed[n] ?? []), style];
        cp.creditsUsed += 10;
        cp.lastSuccessAt = new Date().toISOString();
      } else {
        log(`  ✗ #${n} FAILED: ${(results[j] as PromiseRejectedResult).reason}`);
      }
    }
    saveCheckpoint(cp);
  }

  const finalQuota = await getQuota();
  log(`Done. Final quota: ${finalQuota.credits_left} (consumed ~${quota.credits_left - finalQuota.credits_left})`);
}

main().catch(err => { console.error('[bulk] FAILED:', err); process.exit(1); });
