import { afterEach, describe, expect, it, mock } from 'bun:test'
import { SigilForgeEngine } from '../engines/sigil-forge/engine'
import { CloudflareImageProvider } from './cloudflare'
import { createDefaultImageProvider } from './image-provider'

const originalFetch = globalThis.fetch
function installFetch(handler: (...args: Parameters<typeof fetch>) => ReturnType<typeof fetch>) {
  const fetcher = mock(handler)
  globalThis.fetch = Object.assign(fetcher, { preconnect: originalFetch.preconnect })
  return fetcher
}
const envNames = ['CLOUDFLARE_AI_ACCOUNT_ID', 'CLOUDFLARE_AI_API_TOKEN', 'SIGIL_IMAGE_PROVIDER']
const savedEnv = new Map(envNames.map((k) => [k, process.env[k]]))
const png =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=='

afterEach(() => {
  globalThis.fetch = originalFetch
  for (const [k, v] of savedEnv) {
    if (v === undefined) delete process.env[k]
    else process.env[k] = v
  }
})

function configured() {
  process.env.CLOUDFLARE_AI_ACCOUNT_ID = 'a'.repeat(32)
  process.env.CLOUDFLARE_AI_API_TOKEN = 'fixture-inference-only-token'
  return new CloudflareImageProvider()
}

describe('Cloudflare image provider REST contract', () => {
  it('requires dedicated credentials and an exact account id', () => {
    Reflect.deleteProperty(process.env, 'CLOUDFLARE_AI_ACCOUNT_ID')
    Reflect.deleteProperty(process.env, 'CLOUDFLARE_AI_API_TOKEN')
    expect(new CloudflareImageProvider().isAvailable()).toBe(false)
    configured()
    process.env.CLOUDFLARE_AI_ACCOUNT_ID = '../other-account'
    expect(new CloudflareImageProvider().isAvailable()).toBe(false)
  })

  it('sends only supported parameters and records real provider/mime without claiming a seed', async () => {
    const provider = configured()
    installFetch(async (url, options) => {
      expect(String(url)).toBe(
        `https://api.cloudflare.com/client/v4/accounts/${'a'.repeat(32)}/ai/run/@cf/black-forest-labs/flux-1-schnell`,
      )
      expect(options?.method).toBe('POST')
      expect(new Headers(options?.headers).get('Authorization')).toBe(
        'Bearer fixture-inference-only-token',
      )
      expect(options?.signal).toBeInstanceOf(AbortSignal)
      expect(JSON.parse(String(options?.body))).toEqual({ prompt: 'A geometric sigil', steps: 4 })
      return Response.json({ success: true, result: { image: png }, errors: [] })
    })
    const image = await provider.generate({
      prompt: 'A geometric sigil',
      seed: 144,
      width: 1024,
      height: 1024,
    })
    expect(image.b64_json).toBe(png)
    expect(image.metadata.provider).toBe('cloudflare')
    expect(image.metadata.model).toBe('@cf/black-forest-labs/flux-1-schnell')
    expect(image.metadata.mime_type).toBe('image/png')
    expect(image.metadata.seed).toBeUndefined()
    expect(image.mock).toBeUndefined()
  })

  it('rejects model overrides outside its supported endpoint without sending credentials', async () => {
    const provider = configured()
    const fetcher = installFetch(async () => Response.json({}))
    await expect(
      provider.generate({ prompt: 'Sigil', model: 'https://other.invalid/model' }),
    ).rejects.toThrow('Unsupported Cloudflare image model')
    expect(fetcher).not.toHaveBeenCalled()
  })

  it('does not treat HTTP or API errors as real images or leak provider bodies', async () => {
    const provider = configured()
    installFetch(async () => Response.json({ error: 'private-provider-body' }, { status: 429 }))
    await expect(provider.generate({ prompt: 'Sigil' })).rejects.toThrow(
      'Cloudflare image generation failed (429)',
    )
    installFetch(async () =>
      Response.json({ success: false, result: { image: png }, errors: [{ code: 5006 }] }),
    )
    await expect(provider.generate({ prompt: 'Sigil' })).rejects.toThrow(
      'Cloudflare image generation failed',
    )
  })

  it('rejects an empty or non-image successful payload', async () => {
    const provider = configured()
    for (const image of [undefined, Buffer.from('not an image').toString('base64')]) {
      installFetch(async () => Response.json({ success: true, result: { image } }))
      await expect(provider.generate({ prompt: 'Sigil' })).rejects.toThrow('no valid image')
    }
  })

  it('selects Cloudflare only through explicit server configuration', () => {
    configured()
    process.env.SIGIL_IMAGE_PROVIDER = 'cloudflare'
    expect(createDefaultImageProvider().name).toBe('cloudflare')
    Reflect.deleteProperty(process.env, 'SIGIL_IMAGE_PROVIDER')
    expect(createDefaultImageProvider().name).not.toBe('cloudflare')
  })

  it('preserves image MIME and honest provenance through the engine envelope', async () => {
    const provider = configured()
    installFetch(async () => Response.json({ success: true, result: { image: png } }))
    const output = await new SigilForgeEngine(provider).calculate({
      consciousness_level: 5,
      parameters: { intention: 'I practice patient attention', generate_image: true },
      seed: 144,
    })
    expect(output.provenance?.provider_id).toBe('cloudflare')
    expect(output.provenance?.fallback_used).toBe(false)
    expect(output.generated_image?.metadata?.mime_type).toBe('image/png')
    expect(output.generated_image?.metadata?.seed).toBeUndefined()
  })
})
