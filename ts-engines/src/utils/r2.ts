/**
 * Cloudflare R2 helper — SigV4-signed S3 API access via aws4fetch.
 *
 * Used by the raaga `r2` clip mode. aws4fetch is
 * fetch + WebCrypto based, so it runs unchanged under Bun; the AWS SDK's Node
 * stream adapters do not.
 *
 * Env (read on demand, never logged):
 *   R2_ACCOUNT_ID          — Cloudflare account id (S3 endpoint host)
 *   R2_ACCESS_KEY_ID       — R2 API token access key
 *   R2_SECRET_ACCESS_KEY   — R2 API token secret
 *   R2_PUBLIC_BASE_URL     — public read base (r2.dev or custom domain), no trailing slash needed
 *   R2_RAGA_CLIPS_BUCKET   — bucket name (default "selemene-raga-clips")
 *
 * Error messages only ever carry the HTTP status, the object key and the
 * variable *names* that are missing — never values.
 */

import { AwsClient } from 'aws4fetch'

export type Env = Record<string, string | undefined>

export interface R2Config {
  accountId: string
  accessKeyId: string
  secretAccessKey: string
  bucket: string
  publicBaseUrl: string
}

export const DEFAULT_RAGA_CLIPS_BUCKET = 'selemene-raga-clips'

const REQUIRED_VARS = [
  'R2_ACCOUNT_ID',
  'R2_ACCESS_KEY_ID',
  'R2_SECRET_ACCESS_KEY',
  'R2_PUBLIC_BASE_URL',
] as const

/** Names of required R2 variables that are unset or blank. Empty array = configured. */
export function missingR2Vars(env: Env = process.env): string[] {
  return REQUIRED_VARS.filter((name) => !(env[name] ?? '').trim())
}

/** Read the R2 config, or null when any required variable is missing. */
export function readR2Config(env: Env = process.env): R2Config | null {
  if (missingR2Vars(env).length > 0) return null
  return {
    accountId: (env.R2_ACCOUNT_ID as string).trim(),
    accessKeyId: (env.R2_ACCESS_KEY_ID as string).trim(),
    secretAccessKey: (env.R2_SECRET_ACCESS_KEY as string).trim(),
    bucket: (env.R2_RAGA_CLIPS_BUCKET ?? '').trim() || DEFAULT_RAGA_CLIPS_BUCKET,
    publicBaseUrl: (env.R2_PUBLIC_BASE_URL as string).trim().replace(/\/+$/, ''),
  }
}

/** Read the R2 config or throw naming (only) the missing variables. */
export function requireR2Config(env: Env = process.env): R2Config {
  const config = readR2Config(env)
  if (!config) throw new Error(`Missing R2 env vars: ${missingR2Vars(env).join(', ')}`)
  return config
}

function normalizeKey(key: string): string {
  const trimmed = key.replace(/^\/+/, '')
  if (!trimmed || trimmed.includes('..')) throw new Error(`invalid R2 object key: ${key}`)
  return trimmed
}

/** Public read URL for a key: `${R2_PUBLIC_BASE_URL}/${key}`. */
export function publicUrl(key: string, configOrEnv: R2Config | Env = process.env): string {
  const base = isConfig(configOrEnv)
    ? configOrEnv.publicBaseUrl
    : ((configOrEnv.R2_PUBLIC_BASE_URL ?? '').trim() || '').replace(/\/+$/, '')
  if (!base) throw new Error('Missing R2 env vars: R2_PUBLIC_BASE_URL')
  return `${base}/${normalizeKey(key)}`
}

/** S3-API object URL (signed access only; not the public read URL). */
export function objectUrl(key: string, config: R2Config): string {
  return `https://${config.accountId}.r2.cloudflarestorage.com/${config.bucket}/${normalizeKey(key)}`
}

function isConfig(value: R2Config | Env): value is R2Config {
  return typeof (value as R2Config).accountId === 'string' && 'secretAccessKey' in value
}

export function createR2Client(config: R2Config): AwsClient {
  return new AwsClient({
    accessKeyId: config.accessKeyId,
    secretAccessKey: config.secretAccessKey,
    service: 's3',
    region: 'auto',
    // One attempt per call: the caller decides whether a failure is retryable
    // and tests need deterministic HEAD/PUT sequences.
    retries: 0,
  })
}

export interface PutIfAbsentOptions {
  cacheControl?: string
  config?: R2Config
  env?: Env
}

export interface PutIfAbsentResult {
  /** Public read URL for the object. */
  url: string
  /** true when HEAD found the object and no PUT was issued. */
  existed: boolean
  key: string
}

/**
 * HEAD the key; PUT the body only when the object is absent (404).
 * Any other HEAD status, or a non-2xx PUT, throws an Error carrying only the
 * status and key.
 */
export async function putIfAbsent(
  key: string,
  body: Uint8Array | ArrayBuffer | string,
  contentType: string,
  opts: PutIfAbsentOptions = {},
): Promise<PutIfAbsentResult> {
  const config = opts.config ?? requireR2Config(opts.env ?? process.env)
  const normalizedKey = normalizeKey(key)
  const client = createR2Client(config)
  const url = objectUrl(normalizedKey, config)
  const result = { url: publicUrl(normalizedKey, config), key: normalizedKey }

  const head = await client.fetch(url, { method: 'HEAD' })
  if (head.ok) return { ...result, existed: true }
  if (head.status !== 404) {
    throw new Error(`R2 HEAD ${normalizedKey} failed with status ${head.status}`)
  }

  const headers: Record<string, string> = { 'Content-Type': contentType }
  if (opts.cacheControl) headers['Cache-Control'] = opts.cacheControl
  const put = await client.fetch(url, { method: 'PUT', headers, body })
  if (!put.ok) {
    throw new Error(`R2 PUT ${normalizedKey} failed with status ${put.status}`)
  }
  return { ...result, existed: false }
}
