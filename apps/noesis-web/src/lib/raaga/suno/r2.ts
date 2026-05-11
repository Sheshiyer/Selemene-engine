// Supabase Storage upload helper for Suno-generated raga clips.
//
// Replaces the original Cloudflare R2 implementation.
// Requires env vars:
//   NEXT_PUBLIC_SUPABASE_URL      — e.g. https://xxxx.supabase.co
//   SUPABASE_SERVICE_ROLE_KEY     — service role key (server-side only; never expose to browser)
//   SUPABASE_RAGA_CLIPS_BUCKET   — bucket name, defaults to "raga-clips"

import { createClient } from '@supabase/supabase-js';
import type { SunoStyle } from './types';

const getBucket = () => process.env.SUPABASE_RAGA_CLIPS_BUCKET ?? 'raga-clips';

const getSupabaseAdmin = () => {
  const url = process.env.NEXT_PUBLIC_SUPABASE_URL;
  const key = process.env.SUPABASE_SERVICE_ROLE_KEY;
  if (!url) throw new Error('NEXT_PUBLIC_SUPABASE_URL env var required');
  if (!key) throw new Error('SUPABASE_SERVICE_ROLE_KEY env var required');
  return createClient(url, key);
};

/** Build the canonical storage path for a raga clip. */
export const storageKeyFor = (
  melakartaNum: number,
  style: SunoStyle,
  sunoSongId: string,
): string => `clips/${style}/${String(melakartaNum).padStart(2, '0')}-${sunoSongId}.mp3`;

/** @deprecated — kept as alias so any remaining imports compile */
export const r2KeyFor = storageKeyFor;

/** Build the public URL for a storage path. Doesn't require service-role key. */
export const publicUrlFor = (path: string): string => {
  const url = process.env.NEXT_PUBLIC_SUPABASE_URL ?? '';
  const bucket = getBucket();
  return `${url.replace(/\/$/, '')}/storage/v1/object/public/${bucket}/${path}`;
};

/** @deprecated — kept as alias so any remaining imports compile */
export const cdnUrlFor = publicUrlFor;

/**
 * Upload an MP3 buffer to Supabase Storage.
 * Returns the storage path and public URL.
 */
export const uploadMp3ToStorage = async (
  key: string,
  body: Buffer,
  contentType = 'audio/mpeg',
): Promise<{ key: string; cdnUrl: string }> => {
  const supabase = getSupabaseAdmin();
  const { error } = await supabase.storage
    .from(getBucket())
    .upload(key, body, {
      contentType,
      cacheControl: '31536000',
      upsert: true,
    });
  if (error) throw new Error(`Supabase Storage upload failed: ${error.message}`);
  return { key, cdnUrl: publicUrlFor(key) };
};

/** @deprecated — kept as alias so any remaining imports compile */
export const uploadMp3ToR2 = uploadMp3ToStorage;
