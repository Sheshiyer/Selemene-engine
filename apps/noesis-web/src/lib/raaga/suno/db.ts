// Postgres helper for raga_clips queries. Singleton pool reused across requests.
//
// Reads DATABASE_URL from env (same as Selemene's other Postgres callers).
// Marked `import 'server-only'` so accidental client-side import fails loudly
// at build time — these helpers must never bundle into the browser.

import 'server-only';
import type { ClipStatus, SunoStyle } from './types';

type PoolType = import('pg').Pool;
type QueryResult<T> = { rows: T[]; rowCount: number | null };

let _pool: PoolType | null = null;

const getPool = (): PoolType => {
  if (_pool) return _pool;
  const { Pool } = require('pg') as typeof import('pg');
  const url = process.env.DATABASE_URL;
  if (!url) throw new Error('DATABASE_URL env var required for raga_clips queries');
  _pool = new Pool({
    connectionString: url,
    max: 5,
    idleTimeoutMillis: 30_000,
    connectionTimeoutMillis: 5_000,
  });
  return _pool;
};

export interface RagaClipDbRow {
  id: number;
  melakarta_num: number;
  style: SunoStyle;
  duration_sec: number;
  suno_song_id: string;
  cdn_url: string;
  status: ClipStatus;
}

/**
 * Look up the approved clip for a (melakarta_num, style). Returns null if
 * no row exists or none is approved yet.
 *
 * Hot path — uses `idx_raga_clips_melakarta_status` for sub-ms lookup.
 */
export const findApprovedClip = async (
  melakartaNum: number,
  style: SunoStyle = 'ambient',
): Promise<RagaClipDbRow | null> => {
  if (melakartaNum < 1 || melakartaNum > 72) return null;
  const pool = getPool();
  const res = (await pool.query(
    `SELECT id, melakarta_num, style, duration_sec, suno_song_id, cdn_url, status
       FROM raga_clips
      WHERE melakarta_num = $1 AND style = $2 AND status = 'approved'
      LIMIT 1`,
    [melakartaNum, style],
  )) as QueryResult<RagaClipDbRow>;
  return res.rows[0] ?? null;
};
