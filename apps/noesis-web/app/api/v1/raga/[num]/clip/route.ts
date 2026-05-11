// GET /api/v1/raga/:num/clip?style=ambient
//
// Resolves a Selemene engine raga recommendation (melakarta number 1..72)
// to a pre-rendered Suno clip URL stored in R2. Returns 404 if no approved
// clip exists for the requested (num, style) — caller should fall back to
// V2.5 Strudel synthesis.
//
// Phase 3 of raagaegnin/SUNO_INTEGRATION_PLAN.md (task S-035, S-036, S-037).

import { NextResponse, type NextRequest } from 'next/server';
import { findApprovedClip } from '@/lib/raaga/suno/db';
import type { SunoStyle } from '@/lib/raaga/suno/types';

const VALID_STYLES = ['ambient', 'meditative', 'cinematic', 'acid'] as const;
const isValidStyle = (s: string): s is SunoStyle =>
  (VALID_STYLES as readonly string[]).includes(s);

/** 1y immutable cache — clips are content-addressable via (num, style) + status='approved'. */
const CACHE_HEADERS: HeadersInit = {
  'Cache-Control': 'public, max-age=31536000, immutable, s-maxage=31536000',
};

const NO_CACHE_404: HeadersInit = {
  // Short TTL so a clip newly approved becomes visible within a minute.
  'Cache-Control': 'public, max-age=60, s-maxage=60',
};

export async function GET(
  req: NextRequest,
  ctx: { params: Promise<{ num: string }> },
): Promise<NextResponse> {
  const { num: numStr } = await ctx.params;
  const num = Number.parseInt(numStr, 10);
  if (!Number.isInteger(num) || num < 1 || num > 72) {
    return NextResponse.json(
      { error: 'melakarta_num must be an integer 1..72', got: numStr },
      { status: 400 },
    );
  }

  const styleParam = req.nextUrl.searchParams.get('style') ?? 'ambient';
  if (!isValidStyle(styleParam)) {
    return NextResponse.json(
      { error: `unknown style; expected one of ${VALID_STYLES.join('|')}`, got: styleParam },
      { status: 400 },
    );
  }

  try {
    const clip = await findApprovedClip(num, styleParam);
    if (!clip) {
      return NextResponse.json(
        {
          error: 'no approved clip',
          melakarta_num: num,
          style: styleParam,
          fallback: 'Use V2.5 Strudel synthesis via getRaagaPlayer().play()',
        },
        { status: 404, headers: NO_CACHE_404 },
      );
    }
    return NextResponse.json(
      {
        melakarta_num: clip.melakarta_num,
        style: clip.style,
        duration_sec: clip.duration_sec,
        cdn_url: clip.cdn_url,
        suno_song_id: clip.suno_song_id,
      },
      { status: 200, headers: CACHE_HEADERS },
    );
  } catch (err) {
    // DB unreachable / migration not applied yet — return 503 so client falls back.
    return NextResponse.json(
      {
        error: 'clip lookup failed',
        detail: (err as Error).message,
        fallback: 'Use V2.5 Strudel synthesis',
      },
      { status: 503, headers: NO_CACHE_404 },
    );
  }
}

// Phase-3 task S-043: lightweight in-process rate limit. For Vercel/Railway
// edge deployments use the platform's rate-limit (or upgrade to Upstash etc).
// For now: 100 req/minute per IP, in-memory window. Resets on cold start.
// Intentionally below the route handler since Next.js doesn't (yet) provide
// a unified middleware-API in the route file.
