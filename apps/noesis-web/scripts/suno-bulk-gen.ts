#!/usr/bin/env tsx
// Bulk generator for the 72-melakarta library.
//
// Walks 1..72, submits generation, polls, downloads, uploads to R2.
// Resumable via .suno-checkpoint.json — re-running skips completed ragas.
// Quota guard: refuses to start if remaining credits < MIN_CREDITS_TO_START.
// Per-batch credit cap: stops after MAX_CREDITS_PER_RUN consumed.
//
// Usage:
//   $ tsx apps/noesis-web/scripts/suno-bulk-gen.ts [style] [start_num] [end_num]
//   $ tsx apps/noesis-web/scripts/suno-bulk-gen.ts ambient        # full 1..72
//   $ tsx apps/noesis-web/scripts/suno-bulk-gen.ts ambient 1 36   # first half only
//
// Env (same as smoke):
//   SUNO_BRIDGE_URL, R2_*, optional DATABASE_URL
//
// Optional tuning:
//   SUNO_BATCH_SIZE          (default 5)        — concurrent generations
//   SUNO_POLL_INTERVAL_MS    (default 5000)
//   SUNO_TIMEOUT_MS          (default 180000)
//   MAX_CREDITS_PER_RUN      (default 200)      — hard stop
//   MIN_CREDITS_TO_START     (default 100)      — abort if quota too low

import {
  buildSunoPrompt, getQuota, submitGeneration, pollUntilReady,
  downloadAudio, r2KeyFor, type SunoStyle,
  type BulkGenCheckpoint,
} from '../src/lib/raaga/suno/index.js';
import { uploadMp3ToR2ViaWrangler } from '../src/lib/raaga/suno/r2-wrangler.js';
import fs from 'node:fs';
import path from 'node:path';

const CHECKPOINT_PATH = path.resolve(process.cwd(), '.suno-checkpoint.json');
const args = process.argv.slice(2);
const style = (args[0] ?? 'ambient') as SunoStyle;
const startNum = parseInt(args[1] ?? '1', 10);
const endNum = parseInt(args[2] ?? '72', 10);

const BATCH_SIZE = parseInt(process.env.SUNO_BATCH_SIZE ?? '5', 10);
const MAX_CREDITS_PER_RUN = parseInt(process.env.MAX_CREDITS_PER_RUN ?? '200', 10);
const MIN_CREDITS_TO_START = parseInt(process.env.MIN_CREDITS_TO_START ?? '100', 10);

const log = (msg: string) => console.log(`[bulk ${new Date().toISOString()}] ${msg}`);

function loadCheckpoint(): BulkGenCheckpoint {
  if (fs.existsSync(CHECKPOINT_PATH)) {
    try { return JSON.parse(fs.readFileSync(CHECKPOINT_PATH, 'utf8')); }
    catch { /* fall through to fresh */ }
  }
  return { lastSuccessAt: '', completed: {}, creditsUsed: 0, initialCredits: 0 };
}

function saveCheckpoint(cp: BulkGenCheckpoint) {
  fs.writeFileSync(CHECKPOINT_PATH, JSON.stringify(cp, null, 2));
}

async function generateOne(num: number, style: SunoStyle): Promise<{
  songId: string; key: string; cdnUrl: string; durationSec: number; prompt: string;
}> {
  const req = buildSunoPrompt(num, style);
  log(`  #${num} ${style} → submitting`);
  const songs = await submitGeneration(req);
  const song = songs[0];
  if (!song) throw new Error(`#${num}: no song returned from submit`);
  const ready = await pollUntilReady(song.id, {
    intervalMs: parseInt(process.env.SUNO_POLL_INTERVAL_MS ?? '5000', 10),
    timeoutMs: parseInt(process.env.SUNO_TIMEOUT_MS ?? '180000', 10),
  });
  const mp3 = await downloadAudio(ready.audio_url);
  const key = r2KeyFor(num, style, song.id);
  const { cdnUrl } = uploadMp3ToR2ViaWrangler(key, mp3);
  log(`  #${num} ${style} ✓ ${(mp3.length / 1024).toFixed(0)} KiB → ${cdnUrl}`);
  return { songId: song.id, key, cdnUrl, durationSec: ready.duration, prompt: req.prompt };
}

async function main() {
  log(`Bulk gen: style=${style} range=[${startNum}..${endNum}] batch=${BATCH_SIZE} max-credits=${MAX_CREDITS_PER_RUN}`);

  const quota = await getQuota();
  log(`Quota: ${quota.credits_left} credits remaining (monthly_usage=${quota.monthly_usage}/${quota.monthly_limit})`);
  if (quota.credits_left < MIN_CREDITS_TO_START) {
    throw new Error(`Quota too low (${quota.credits_left} < ${MIN_CREDITS_TO_START}). Aborting before any work.`);
  }

  const cp = loadCheckpoint();
  if (!cp.initialCredits) cp.initialCredits = quota.credits_left;
  log(`Checkpoint: ${Object.keys(cp.completed).length} ragas already complete; ${cp.creditsUsed} credits used so far`);

  const remaining: number[] = [];
  for (let n = startNum; n <= endNum; n++) {
    if (cp.completed[n]?.includes(style)) continue;  // skip already done
    remaining.push(n);
  }
  log(`${remaining.length} ragas remaining for style=${style}`);

  // Process in batches
  for (let i = 0; i < remaining.length; i += BATCH_SIZE) {
    if (cp.creditsUsed >= MAX_CREDITS_PER_RUN) {
      log(`⏸  Hit max-credits-per-run cap (${MAX_CREDITS_PER_RUN}). Stopping. Re-run to continue.`);
      break;
    }
    const batch = remaining.slice(i, i + BATCH_SIZE);
    log(`Batch ${Math.floor(i / BATCH_SIZE) + 1}: ragas [${batch.join(', ')}]`);
    const results = await Promise.allSettled(batch.map((n) => generateOne(n, style)));
    for (let j = 0; j < batch.length; j++) {
      const res = results[j];
      const n = batch[j];
      if (res.status === 'fulfilled') {
        cp.completed[n] = [...(cp.completed[n] ?? []), style];
        cp.creditsUsed += 10;  // each generation = 10 credits
        cp.lastSuccessAt = new Date().toISOString();
      } else {
        log(`  ✗ #${n} FAILED: ${res.reason}`);
      }
    }
    saveCheckpoint(cp);
    log(`  → checkpoint saved (${cp.creditsUsed} credits used; ${Object.keys(cp.completed).length} ragas done)`);
  }

  const finalQuota = await getQuota();
  log(`Done. Final quota: ${finalQuota.credits_left} credits (consumed ~${quota.credits_left - finalQuota.credits_left})`);
  log(`Checkpoint at ${CHECKPOINT_PATH} — keep for next run`);
  log(`Next: review clips in admin audition page (Phase 2 W2.2)`);
}

main().catch((err) => {
  console.error('[bulk] FAILED:', err);
  process.exit(1);
});
