// Wrangler-based R2 uploader. Alternative to r2.ts (which needs S3 API keys).
// Uses the LOCAL `wrangler` CLI's OAuth — no R2 API token to mint.
//
// Tradeoff: only works in environments where `wrangler` is installed and logged
// into the right Cloudflare account. Production deploy still needs the S3 path
// (r2.ts) because Vercel/Railway runtimes don't have wrangler. But the bulk-gen
// script + smoke test (run from your dev machine) can use this and skip the
// "mint R2 API token" step entirely.

import { execFileSync } from 'node:child_process';
import { writeFileSync, unlinkSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import type { SunoStyle } from './types';
import { r2KeyFor, cdnUrlFor } from './r2';

const BUCKET = process.env.R2_BUCKET ?? 'selemene-raga-clips';

/** Upload an MP3 buffer to R2 via wrangler. Same shape as r2.ts uploadMp3ToR2. */
export const uploadMp3ToR2ViaWrangler = (
  key: string,
  body: Buffer,
  contentType = 'audio/mpeg',
): { key: string; cdnUrl: string } => {
  const tmpDir = mkdtempSync(join(tmpdir(), 'suno-r2-'));
  const tmpFile = join(tmpDir, 'audio.mp3');
  writeFileSync(tmpFile, body);
  try {
    const args = [
      'r2', 'object', 'put', `${BUCKET}/${key}`,
      `--file=${tmpFile}`,
      `--content-type=${contentType}`,
      '--remote',
    ];
    execFileSync('wrangler', args, { stdio: ['ignore', 'pipe', 'inherit'] });
    return { key, cdnUrl: cdnUrlFor(key) };
  } finally {
    try { unlinkSync(tmpFile); } catch { /* ignore */ }
  }
};

export { r2KeyFor };
