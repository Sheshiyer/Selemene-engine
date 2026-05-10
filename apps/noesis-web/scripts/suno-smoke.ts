#!/usr/bin/env tsx
// Smoke test for the Suno integration. Generates ONE raga end-to-end:
//   1. Build prompt for given melakarta_num + style
//   2. POST to Suno bridge → get song id
//   3. Poll until ready (~30-90s)
//   4. Download MP3
//   5. Upload to R2
//   6. (Optional) INSERT row in raga_clips
//
// Usage:
//   $ tsx apps/noesis-web/scripts/suno-smoke.ts <melakarta_num> [style] [duration_sec]
//   $ tsx apps/noesis-web/scripts/suno-smoke.ts 15
//   $ tsx apps/noesis-web/scripts/suno-smoke.ts 15 ambient 45
//
// Required env (load via .env.local):
//   SUNO_BRIDGE_URL          — e.g. https://suno-bridge.tryambakam.space
//   R2_ACCOUNT_ID
//   R2_ACCESS_KEY_ID
//   R2_SECRET_ACCESS_KEY
//   R2_BUCKET                — e.g. selemene-raga-clips
//   R2_PUBLIC_BASE_URL       — e.g. https://clips.tryambakam.space
//
// Optional:
//   DATABASE_URL             — if set, INSERT row into raga_clips after upload
//   SUNO_SMOKE_DRY_RUN=1     — skip actual Suno call, print prompt + exit

import {
  buildSunoPrompt,
  getQuota,
  submitGeneration,
  pollUntilReady,
  downloadAudio,
  uploadMp3ToR2,
  r2KeyFor,
  type SunoStyle,
} from '../src/lib/raaga/suno/index.js';

const args = process.argv.slice(2);
const melakartaNum = parseInt(args[0] ?? '15', 10);
const style = (args[1] ?? 'ambient') as SunoStyle;
const durationSec = parseInt(args[2] ?? '45', 10);

const log = (msg: string) => console.log(`[smoke ${new Date().toISOString()}] ${msg}`);

async function main() {
  log(`Smoke test: melakarta=${melakartaNum} style=${style} duration=${durationSec}s`);
  const req = buildSunoPrompt(melakartaNum, style, durationSec);
  log(`Prompt: "${req.prompt}"`);
  log(`Tags:   "${req.tags}"`);
  log(`Title:  "${req.title}"`);

  if (process.env.SUNO_SMOKE_DRY_RUN === '1') {
    log('DRY RUN — skipping Suno call. Exiting.');
    return;
  }

  const quotaBefore = await getQuota();
  log(`Quota before: ${quotaBefore.credits_left} credits remaining`);
  if (quotaBefore.credits_left < 20) {
    throw new Error(`Quota too low (${quotaBefore.credits_left} < 20). Aborting.`);
  }

  log('Submitting generation…');
  const songs = await submitGeneration(req);
  log(`Submitted: ${songs.length} variants returned. IDs: ${songs.map(s => s.id).join(', ')}`);

  // Use the FIRST variant for smoke test (Suno returns 2 by default)
  const song = songs[0];
  log(`Polling song ${song.id} for completion…`);
  const ready = await pollUntilReady(song.id, { intervalMs: 5000, timeoutMs: 180_000 });
  log(`Song ready: status=${ready.status}, duration=${ready.duration}s, audio_url=${ready.audio_url}`);

  log('Downloading MP3…');
  const mp3Buffer = await downloadAudio(ready.audio_url);
  log(`Downloaded ${(mp3Buffer.length / 1024).toFixed(1)} KiB`);

  log('Uploading to R2…');
  const key = r2KeyFor(melakartaNum, style, song.id);
  const { cdnUrl } = await uploadMp3ToR2(key, mp3Buffer);
  log(`Uploaded → ${cdnUrl}`);

  const quotaAfter = await getQuota();
  log(`Quota after: ${quotaAfter.credits_left} credits remaining (used ${quotaBefore.credits_left - quotaAfter.credits_left})`);

  log('✅ Smoke test complete. Open the CDN URL above in a browser to listen.');
  log(`To insert into DB: INSERT INTO raga_clips (melakarta_num, style, duration_sec, suno_song_id, suno_prompt, r2_key, cdn_url, status) VALUES (${melakartaNum}, '${style}', ${ready.duration}, '${song.id}', $$${req.prompt}$$, '${key}', '${cdnUrl}', 'generated');`);
}

main().catch((err) => {
  console.error('[smoke] FAILED:', err);
  process.exit(1);
});
