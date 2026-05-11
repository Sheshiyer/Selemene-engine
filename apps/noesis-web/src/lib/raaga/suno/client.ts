// Suno bridge client — proxied through the ts-engines Railway sidecar.
//
// Endpoints used (sidecar at SUNO_BRIDGE_URL):
//   GET  /suno/get_limit          — quota check
//   POST /suno/custom_generate    — submit job, returns 2 song objects with status='submitted'
//   GET  /suno/get?ids=a,b        — poll status; returns array of song objects
//
// Cookie auth (SUNO_COOKIE) lives in ts-engines env — never exposed to browser.

import type { SunoCustomGenerateRequest, SunoSongResponse } from './types';

const SUNO_BRIDGE_URL = process.env.SUNO_BRIDGE_URL ?? 'https://suno-bridge.tryambakam.space';

export interface QuotaInfo {
  credits_left: number;
  period: string;
  monthly_limit: number;
  monthly_usage: number;
}

const headers = (): HeadersInit => ({
  'Content-Type': 'application/json',
  'Accept': 'application/json',
});

export const getQuota = async (): Promise<QuotaInfo> => {
  const r = await fetch(`${SUNO_BRIDGE_URL}/suno/get_limit`, { headers: headers() });
  if (!r.ok) throw new Error(`get_limit failed: ${r.status} ${await r.text()}`);
  return r.json();
};

export const submitGeneration = async (
  body: SunoCustomGenerateRequest,
): Promise<SunoSongResponse[]> => {
  const r = await fetch(`${SUNO_BRIDGE_URL}/suno/custom_generate`, {
    method: 'POST',
    headers: headers(),
    body: JSON.stringify(body),
  });
  if (!r.ok) throw new Error(`custom_generate failed: ${r.status} ${await r.text()}`);
  // Sidecar returns an array of 2 song variants
  const data = await r.json();
  if (!Array.isArray(data)) throw new Error(`expected array, got ${typeof data}`);
  return data as SunoSongResponse[];
};

export const getSongs = async (ids: string[]): Promise<SunoSongResponse[]> => {
  const r = await fetch(`${SUNO_BRIDGE_URL}/suno/get?ids=${ids.join(',')}`, { headers: headers() });
  if (!r.ok) throw new Error(`get failed: ${r.status} ${await r.text()}`);
  return r.json();
};

/**
 * Poll a single song until it reaches 'streaming' or 'complete' (or 'error').
 * Returns the final song object. Throws on timeout or error.
 */
export const pollUntilReady = async (
  id: string,
  opts: { intervalMs?: number; timeoutMs?: number } = {},
): Promise<SunoSongResponse> => {
  const intervalMs = opts.intervalMs ?? 5000;
  const timeoutMs = opts.timeoutMs ?? 180_000;  // 3 minutes
  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    const songs = await getSongs([id]);
    const song = songs[0];
    if (!song) throw new Error(`song ${id} disappeared during polling`);
    if (song.status === 'error') {
      throw new Error(`Suno error: ${song.error_message ?? 'unknown'}`);
    }
    if ((song.status === 'streaming' || song.status === 'complete') && song.audio_url) {
      return song;
    }
    await new Promise((r) => setTimeout(r, intervalMs));
  }
  throw new Error(`Polling timeout after ${timeoutMs}ms for song ${id}`);
};

/** Download the MP3 bytes from Suno's CDN. */
export const downloadAudio = async (audioUrl: string): Promise<Buffer> => {
  const r = await fetch(audioUrl);
  if (!r.ok) throw new Error(`audio download failed: ${r.status}`);
  const ab = await r.arrayBuffer();
  return Buffer.from(ab);
};
