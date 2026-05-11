// Client hook: fetch the Suno-rendered clip metadata for a melakarta.
// Returns { url, loading, error }. 404 → url=null (caller falls back to Strudel).

'use client';

import { useEffect, useState, useRef } from 'react';
import type { SunoStyle } from './suno/types';

export interface RagaClipMeta {
  melakarta_num: number;
  style: SunoStyle;
  duration_sec: number;
  cdn_url: string;
  suno_song_id: string;
}

export interface UseRagaClipResult {
  /** CDN URL if an approved clip exists for (num, style); null otherwise. */
  clip: RagaClipMeta | null;
  /** True while the API call is in flight. */
  loading: boolean;
  /** Set if the API returned a non-404 error (DB down, malformed, etc). */
  error: string | null;
  /** Re-fetch (e.g. after a fresh approval). */
  refetch: () => void;
}

const CLIP_CACHE = new Map<string, RagaClipMeta | null>();

/**
 * Looks up a clip via /api/v1/raga/:num/clip?style=:style.
 *
 * - 200 → clip metadata cached in-memory keyed by (num, style)
 * - 404 → clip=null, no error (caller falls back to V2.5 Strudel)
 * - 503/5xx → clip=null + error string set (still falls back to Strudel)
 * - Pass `melakartaNum=null` to disable the hook (no fetch fires)
 */
export function useRagaClip(
  melakartaNum: number | null,
  style: SunoStyle = 'ambient',
): UseRagaClipResult {
  const [clip, setClip] = useState<RagaClipMeta | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const reqIdRef = useRef(0);

  const fetchClip = () => {
    if (!melakartaNum || melakartaNum < 1 || melakartaNum > 72) {
      setClip(null); setLoading(false); setError(null);
      return;
    }
    const key = `${melakartaNum}:${style}`;
    if (CLIP_CACHE.has(key)) {
      setClip(CLIP_CACHE.get(key) ?? null);
      setLoading(false); setError(null);
      return;
    }
    const myReqId = ++reqIdRef.current;
    setLoading(true); setError(null);

    fetch(`/api/v1/raga/${melakartaNum}/clip?style=${style}`)
      .then(async (r) => {
        if (myReqId !== reqIdRef.current) return;  // stale, ignore
        if (r.status === 404) {
          CLIP_CACHE.set(key, null);
          setClip(null); setError(null);
          return;
        }
        if (!r.ok) {
          setError(`HTTP ${r.status}`);
          setClip(null);
          return;
        }
        const json = (await r.json()) as RagaClipMeta;
        CLIP_CACHE.set(key, json);
        setClip(json);
        setError(null);
      })
      .catch((e) => {
        if (myReqId !== reqIdRef.current) return;
        setError((e as Error).message);
        setClip(null);
      })
      .finally(() => {
        if (myReqId !== reqIdRef.current) return;
        setLoading(false);
      });
  };

  useEffect(() => {
    fetchClip();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [melakartaNum, style]);

  return {
    clip,
    loading,
    error,
    refetch: () => {
      if (!melakartaNum) return;
      CLIP_CACHE.delete(`${melakartaNum}:${style}`);
      fetchClip();
    },
  };
}
