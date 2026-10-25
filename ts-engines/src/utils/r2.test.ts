/**
 * utils/r2.ts — SigV4 signing + HEAD/PUT sequencing with a mocked global fetch.
 * No network; the credentials below are fake.
 */

import { afterEach, describe, expect, it } from 'bun:test'
import {
  type R2Config,
  missingR2Vars,
  objectUrl,
  publicUrl,
  putIfAbsent,
  readR2Config,
  requireR2Config,
} from './r2'

const config: R2Config = {
  accountId: 'acct123',
  accessKeyId: 'AKIATEST',
  secretAccessKey: 'test-secret-not-real',
  bucket: 'clips-test',
  publicBaseUrl: 'https://pub-test.r2.dev',
}

const env = {
  R2_ACCOUNT_ID: config.accountId,
  R2_ACCESS_KEY_ID: config.accessKeyId,
  R2_SECRET_ACCESS_KEY: config.secretAccessKey,
  R2_PUBLIC_BASE_URL: `${config.publicBaseUrl}/`,
  R2_RAGA_CLIPS_BUCKET: config.bucket,
}

type Call = { method: string; url: string; headers: Headers; body: Uint8Array | null }

const realFetch = globalThis.fetch

/** Install a fetch mock that answers HEAD/PUT with the given statuses and records calls. */
function mockFetch(statuses: Partial<Record<'HEAD' | 'PUT', number>>): Call[] {
  const calls: Call[] = []
  globalThis.fetch = (async (input: Request | string | URL, init?: RequestInit) => {
    const req = input instanceof Request ? input : new Request(String(input), init)
    const body = req.method === 'PUT' ? new Uint8Array(await req.arrayBuffer()) : null
    calls.push({ method: req.method, url: req.url, headers: req.headers, body })
    const status = statuses[req.method as 'HEAD' | 'PUT'] ?? 500
    return new Response(null, { status })
  }) as typeof fetch
  return calls
}

afterEach(() => {
  globalThis.fetch = realFetch
})

describe('r2 config', () => {
  it('names missing vars without values and reads a complete config', () => {
    expect(missingR2Vars({})).toEqual([
      'R2_ACCOUNT_ID',
      'R2_ACCESS_KEY_ID',
      'R2_SECRET_ACCESS_KEY',
      'R2_PUBLIC_BASE_URL',
    ])
    expect(readR2Config({})).toBeNull()
    expect(readR2Config(env)).toEqual(config)
    expect(() => requireR2Config({ ...env, R2_SECRET_ACCESS_KEY: '' })).toThrow(
      /Missing R2 env vars: R2_SECRET_ACCESS_KEY$/,
    )
  })

  it('defaults the bucket and strips trailing slashes from the public base', () => {
    const cfg = readR2Config({ ...env, R2_RAGA_CLIPS_BUCKET: undefined })
    expect(cfg?.bucket).toBe('selemene-raga-clips')
    expect(cfg?.publicBaseUrl).toBe('https://pub-test.r2.dev')
  })

  it('builds public and S3 object URLs', () => {
    expect(publicUrl('raaga/abc.wav', config)).toBe('https://pub-test.r2.dev/raaga/abc.wav')
    expect(publicUrl('/raaga/abc.wav', env)).toBe('https://pub-test.r2.dev/raaga/abc.wav')
    expect(objectUrl('raaga/abc.wav', config)).toBe(
      'https://acct123.r2.cloudflarestorage.com/clips-test/raaga/abc.wav',
    )
    expect(() => publicUrl('../x.wav', config)).toThrow(/invalid R2 object key/)
  })
})

describe('putIfAbsent', () => {
  const body = new TextEncoder().encode('RIFF....WAVE')

  it('HEAD 404 → signed PUT with content type and cache headers → existed=false', async () => {
    const calls = mockFetch({ HEAD: 404, PUT: 200 })
    const result = await putIfAbsent('raaga/abc.wav', body, 'audio/wav', {
      cacheControl: 'public, max-age=31536000, immutable',
      config,
    })
    expect(result).toEqual({
      url: 'https://pub-test.r2.dev/raaga/abc.wav',
      existed: false,
      key: 'raaga/abc.wav',
    })
    expect(calls.map((c) => c.method)).toEqual(['HEAD', 'PUT'])
    for (const call of calls) {
      expect(call.url).toBe('https://acct123.r2.cloudflarestorage.com/clips-test/raaga/abc.wav')
      const auth = call.headers.get('authorization') ?? ''
      expect(auth).toMatch(/^AWS4-HMAC-SHA256 Credential=AKIATEST\/\d{8}\/auto\/s3\/aws4_request/)
      expect(auth).toMatch(/Signature=[0-9a-f]{64}$/)
      expect(call.headers.get('x-amz-date')).toMatch(/^\d{8}T\d{6}Z$/)
      // aws4fetch signs S3 bodies as UNSIGNED-PAYLOAD unless told otherwise; R2 accepts that.
      expect(call.headers.get('x-amz-content-sha256')).toMatch(/^(UNSIGNED-PAYLOAD|[0-9a-f]{64})$/)
      expect(auth).not.toContain(config.secretAccessKey)
    }
    const put = calls[1]
    expect(put.headers.get('content-type')).toBe('audio/wav')
    expect(put.headers.get('cache-control')).toBe('public, max-age=31536000, immutable')
    expect(put.body).toEqual(body)
  })

  it('HEAD 200 → existed=true and no PUT', async () => {
    const calls = mockFetch({ HEAD: 200 })
    const result = await putIfAbsent('raaga/abc.wav', body, 'audio/wav', { config })
    expect(result.existed).toBe(true)
    expect(result.url).toBe('https://pub-test.r2.dev/raaga/abc.wav')
    expect(calls.map((c) => c.method)).toEqual(['HEAD'])
  })

  it('PUT 500 → throws with status and key only', async () => {
    const calls = mockFetch({ HEAD: 404, PUT: 500 })
    await expect(putIfAbsent('raaga/abc.wav', body, 'audio/wav', { config })).rejects.toThrow(
      'R2 PUT raaga/abc.wav failed with status 500',
    )
    expect(calls.map((c) => c.method)).toEqual(['HEAD', 'PUT'])
  })

  it('HEAD 403 → throws without issuing a PUT', async () => {
    const calls = mockFetch({ HEAD: 403 })
    await expect(putIfAbsent('raaga/abc.wav', body, 'audio/wav', { config })).rejects.toThrow(
      'R2 HEAD raaga/abc.wav failed with status 403',
    )
    expect(calls.map((c) => c.method)).toEqual(['HEAD'])
  })

  it('reads config from env when none is passed and throws naming missing vars', async () => {
    mockFetch({ HEAD: 200 })
    const fromEnv = await putIfAbsent('k.wav', body, 'audio/wav', { env })
    expect(fromEnv.existed).toBe(true)
    await expect(putIfAbsent('k.wav', body, 'audio/wav', { env: {} })).rejects.toThrow(
      /^Missing R2 env vars: R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_PUBLIC_BASE_URL$/,
    )
  })
})
