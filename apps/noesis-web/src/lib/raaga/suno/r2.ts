// R2 upload helper. Uses S3-compatible API via @aws-sdk/client-s3.
//
// Requires env vars:
//   R2_ACCOUNT_ID         — Cloudflare account ID
//   R2_ACCESS_KEY_ID      — R2 access key
//   R2_SECRET_ACCESS_KEY  — R2 secret key
//   R2_BUCKET             — bucket name (e.g. "selemene-raga-clips")
//   R2_PUBLIC_BASE_URL    — public CDN URL prefix (e.g. "https://clips.tryambakam.space")

import type { SunoStyle } from './types';

interface R2Config {
  accountId: string;
  accessKeyId: string;
  secretAccessKey: string;
  bucket: string;
  publicBaseUrl: string;
}

const getConfig = (): R2Config => {
  // Defaults reflect the already-provisioned Selemene infra:
  //   - bucket: selemene-raga-clips (created via wrangler)
  //   - public URL: pub-1f3a1b9dd04b4178b521c06332f81a37.r2.dev (auto-assigned)
  //   - account: Sheshiyer's Cloudflare account
  // Override any of these via env. accessKeyId+secret MUST come from env.
  const cfg: R2Config = {
    accountId: process.env.R2_ACCOUNT_ID ?? '9d9d23b27f32e70ae3afb6a1aa2c0f10',
    accessKeyId: process.env.R2_ACCESS_KEY_ID ?? '',
    secretAccessKey: process.env.R2_SECRET_ACCESS_KEY ?? '',
    bucket: process.env.R2_BUCKET ?? 'selemene-raga-clips',
    publicBaseUrl: process.env.R2_PUBLIC_BASE_URL ?? 'https://pub-1f3a1b9dd04b4178b521c06332f81a37.r2.dev',
  };
  // Only the SECRETS are required. Everything else has a sensible default.
  if (!cfg.accessKeyId) throw new Error('R2_ACCESS_KEY_ID env var required (mint via Cloudflare dashboard)');
  if (!cfg.secretAccessKey) throw new Error('R2_SECRET_ACCESS_KEY env var required (mint via Cloudflare dashboard)');
  return cfg;
};

/** Build the canonical R2 object key for a raga clip. */
export const r2KeyFor = (melakartaNum: number, style: SunoStyle, sunoSongId: string): string => {
  // shape: clips/{style}/{melakarta_num}-{suno_id}.mp3
  // melakarta first so we can browse by raga; suno id ensures uniqueness across re-rolls
  return `clips/${style}/${String(melakartaNum).padStart(2, '0')}-${sunoSongId}.mp3`;
};

/** Build the public CDN URL for a given key. */
export const cdnUrlFor = (key: string): string => {
  const cfg = getConfig();
  return `${cfg.publicBaseUrl.replace(/\/$/, '')}/${key}`;
};

/**
 * Upload an MP3 buffer to R2. Returns the R2 key + public CDN URL.
 *
 * Uses dynamic import of @aws-sdk/client-s3 so the rest of the suno/ module
 * stays buildable without that dep (it's only needed at runtime in scripts).
 */
export const uploadMp3ToR2 = async (
  key: string,
  body: Buffer,
  contentType = 'audio/mpeg',
): Promise<{ key: string; cdnUrl: string }> => {
  const cfg = getConfig();
  const { S3Client, PutObjectCommand } = await import(/* webpackIgnore: true */ '@aws-sdk/client-s3');
  const client = new S3Client({
    region: 'auto',
    endpoint: `https://${cfg.accountId}.r2.cloudflarestorage.com`,
    credentials: {
      accessKeyId: cfg.accessKeyId,
      secretAccessKey: cfg.secretAccessKey,
    },
  });
  await client.send(new PutObjectCommand({
    Bucket: cfg.bucket,
    Key: key,
    Body: body,
    ContentType: contentType,
    CacheControl: 'public, max-age=31536000, immutable',
  }));
  return { key, cdnUrl: cdnUrlFor(key) };
};
