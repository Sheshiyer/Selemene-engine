/**
 * R2 upload for the Suno bulk/smoke scripts.
 *
 * Thin wrapper over `src/utils/r2.ts` (SigV4 via aws4fetch). The previous
 * version sent `Authorization: Bearer <secret>` to the S3 endpoint, which R2
 * rejects, and hard-coded `audio/mpeg`; content type is now inferred from the
 * key's extension.
 */

import { extname } from 'node:path'
import { type R2Config, putIfAbsent, requireR2Config } from '../src/utils/r2'

export type { R2Config }
export { requireR2Config }

const CONTENT_TYPES: Record<string, string> = {
  '.mp3': 'audio/mpeg',
  '.wav': 'audio/wav',
  '.ogg': 'audio/ogg',
  '.m4a': 'audio/mp4',
  '.flac': 'audio/flac',
  '.json': 'application/json',
  '.txt': 'text/plain; charset=utf-8',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.svg': 'image/svg+xml',
}

export function contentTypeForKey(key: string): string {
  return CONTENT_TYPES[extname(key).toLowerCase()] ?? 'application/octet-stream'
}

export function publicR2Url(publicBaseUrl: string, key: string): string {
  return `${publicBaseUrl.replace(/\/+$/, '')}/${key.replace(/^\/+/, '')}`
}

/**
 * Upload `buffer` under `key` (skipped when the object already exists) and
 * return its public URL.
 */
export async function uploadToR2(
  key: string,
  buffer: Uint8Array,
  config: R2Config = requireR2Config(),
): Promise<string> {
  const { url } = await putIfAbsent(key, buffer, contentTypeForKey(key), {
    cacheControl: 'public, max-age=31536000, immutable',
    config,
  })
  return url
}
